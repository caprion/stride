"""Garmin account (email + password) ingest via the ``garminconnect`` client.

Complements the developer-key path. Useful when the athlete has a Garmin account
but no API key from the (now gated) developer program.

MFA
---
Garmin requires a 6-digit MFA code for many accounts. The library's
``prompt_mfa`` hook is called synchronously inside the login flow, so we bridge
it to the HTTP layer with a small broker: the login runs on a worker thread, the
hook publishes the code and blocks until the browser posts the answer back.
"""

from __future__ import annotations

import logging
import threading
from typing import Any

from .normalize import (
    blank_activity,
    duration_to_seconds,
    normalise_sport,
    parse_iso,
    safe_float,
    safe_int,
)

log = logging.getLogger(__name__)


class MfaBroker:
    """Bridge between the synchronous MFA prompt and the HTTP request cycle.

    Two events matter to the web layer:

    ``requested``  set as soon as Garmin demands an MFA code, so the API can
                   answer ``{"status": "mfa_required"}`` immediately instead of
                   blocking the HTTP request.
    ``answered``   set once the athlete submits the code from their browser.
    """

    def __init__(self) -> None:
        self._code: str | None = None
        self.requested = threading.Event()
        self.answered = threading.Event()
        self._lock = threading.Lock()

    def request_code(self, code: str | None = None) -> None:
        """Garmin handed us the code itself (some flows provide it)."""
        if code:
            with self._lock:
                self._code = code
            self.answered.set()
        self.requested.set()

    def submit_code(self, code: str) -> None:
        with self._lock:
            self._code = code
        self.answered.set()

    def wait_for_code(self, timeout: float = 600.0) -> str:
        # Called from inside garminconnect's login flow.
        self.requested.set()
        if not self.answered.wait(timeout):
            raise TimeoutError("Timed out waiting for the MFA code.")
        with self._lock:
            return self._code or ""


class GarminAccountError(RuntimeError):
    pass


class GarminAccountClient:
    """Thin, defensive wrapper over ``garminconnect.Garmin``."""

    def __init__(self, email: str, password: str, tokenstore: str) -> None:
        self.email = email
        self.password = password
        self.tokenstore = tokenstore
        self._client = None

    # ------------------------------------------------------------- session

    def login(self, broker: MfaBroker | None = None) -> dict:
        try:
            from garminconnect import Garmin
        except ImportError as exc:  # pragma: no cover
            raise GarminAccountError(
                "The 'garminconnect' package is not installed in this environment."
            ) from exc

        def prompt_mfa(code: str | None = None) -> str:
            # garminconnect calls this with no arguments; the athlete types the
            # code they received by email/SMS into the browser instead.
            if broker is None:
                raise GarminAccountError(
                    "Garmin requires an MFA code but none was provided."
                )
            return broker.wait_for_code()

        try:
            client = Garmin(self.email, self.password, prompt_mfa=prompt_mfa)
            client.login(tokenstore=self.tokenstore)
        except Exception as exc:
            raise GarminAccountError(str(exc)) from exc

        self._client = client
        return {
            "display_name": getattr(client, "display_name", None),
            "full_name": getattr(client, "full_name", None),
            "profile_id": getattr(client, "profile_id", None),
        }

    def _require(self):
        if self._client is None:
            raise GarminAccountError("Not authenticated. Call login() first.")
        return self._client

    # ---------------------------------------------------------- activities

    def fetch_activities(self, days: int = 90) -> list[dict]:
        client = self._require()
        try:
            raw_items = client.get_activities(0, max(30, min(days * 2, 200)))
        except Exception as exc:
            raise GarminAccountError(f"Could not list activities: {exc}") from exc

        out: list[dict] = []
        for raw in raw_items or []:
            activity = self._normalise(raw)
            if activity:
                out.append(activity)
        return out

    def fetch_detail(self, activity_id: str) -> dict:
        """Return the extra detail (splits, streams, polyline) for one activity."""
        client = self._require()
        detail: dict[str, Any] = {"laps": [], "streams": {}, "geo_points": None}

        try:
            splits = client.get_activity_splits(activity_id) or {}
        except Exception:
            splits = {}
        laps = _extract_laps(splits)
        if laps:
            detail["laps"] = laps

        for kind, getter in (
            ("hr", lambda: client.get_activity_hr_in_timezones(activity_id)),
            ("polyline", lambda: client.get_activity_details(activity_id)),
        ):
            try:
                payload = getter()
            except Exception:
                payload = None
            if not payload:
                continue
            if kind == "hr":
                series = _extract_hr_series(payload)
                if series:
                    detail["streams"]["hr"] = series
            else:
                points = _extract_geo(payload)
                if points:
                    detail["geo_points"] = points
        return detail

    # -------------------------------------------------------- normalisation

    def _normalise(self, raw: dict) -> dict | None:
        activity = blank_activity()
        external_id = raw.get("activityId")
        if external_id is None:
            return None
        activity["external_id"] = str(external_id)
        activity["source"] = "garmin"
        activity["raw"] = raw

        activity["name"] = raw.get("activityName") or "Run"
        type_key = raw.get("activityType", {}).get("typeKey", "running")
        activity["sport_type"] = normalise_sport(type_key)

        start = parse_iso(raw.get("startTimeLocal"))
        if start:
            activity["start_time"] = start.isoformat()
            activity["local_date"] = start.date().isoformat()
            activity["day_of_week"] = start.strftime("%A")

        activity["distance_m"] = safe_float(raw.get("distance"), 0.0) or 0.0
        activity["moving_time_s"] = duration_to_seconds(raw.get("duration"))
        activity["elapsed_time_s"] = duration_to_seconds(raw.get("duration"))
        activity["elevation_gain_m"] = safe_float(raw.get("elevationGain"), 0.0) or 0.0
        activity["avg_hr"] = safe_int(raw.get("averageHR"))
        activity["max_hr"] = safe_int(raw.get("maxHR"))
        activity["calories"] = safe_float(raw.get("calories"))
        activity["avg_cadence"] = safe_float(
            raw.get("averageRunningCadenceInStepsPerMinute")
        )
        activity["max_cadence"] = safe_float(raw.get("maxRunningCadenceInStepsPerMinute"))
        activity["avg_speed_ms"] = safe_float(raw.get("averageSpeed"))
        activity["max_speed_ms"] = safe_float(raw.get("maxSpeed"))
        activity["device"] = raw.get("deviceName")
        activity["is_trail"] = "trail" in str(type_key)
        return activity


# ---------------------------------------------------------------- helpers


def _extract_laps(payload: Any) -> list[dict]:
    """Pull lap rows out of Garmin's split payload."""
    laps_raw: list[dict] = []
    if isinstance(payload, dict):
        for key in ("lapDTOs", "lapDtos", "laps"):
            value = payload.get(key)
            if isinstance(value, list):
                laps_raw = [lap for lap in value if isinstance(lap, dict)]
                break
        if not laps_raw:
            for value in payload.values():
                if isinstance(value, list) and value and isinstance(value[0], dict):
                    if "lapIndex" in value[0] or "averageHR" in value[0]:
                        laps_raw = value
                        break

    out: list[dict] = []
    for index, lap in enumerate(laps_raw):
        distance = safe_float(lap.get("distance"), 0.0) or 0.0
        duration = duration_to_seconds(lap.get("duration"))
        out.append(
            {
                "lap_index": safe_int(lap.get("lapIndex"), index + 1) or index + 1,
                "distance_m": distance,
                "duration_s": duration,
                "avg_speed_ms": safe_float(lap.get("averageSpeed")),
                "max_speed_ms": safe_float(lap.get("maxSpeed")),
                "avg_hr": safe_int(lap.get("averageHR")),
                "max_hr": safe_int(lap.get("maxHR")),
                "avg_cadence": safe_float(lap.get("averageRunCadence")),
                "max_cadence": safe_float(lap.get("maxRunCadence")),
                "stride_length_m": safe_float(lap.get("strideLength")),
                "elevation_gain_m": safe_float(lap.get("elevationGain")),
                "elevation_loss_m": safe_float(lap.get("elevationLoss")),
                "start_elevation_m": safe_float(lap.get("startElevation")),
                "end_elevation_m": safe_float(lap.get("endElevation")),
            }
        )
    return out


def _extract_hr_series(payload: Any) -> dict | None:
    """Extract a ``{seconds: [...], values: [...]}`` HR series."""
    if not isinstance(payload, dict):
        return None
    buckets = payload.get("hrTimeInZones")
    if not isinstance(buckets, list):
        return None
    seconds: list[int] = []
    values: list[float] = []
    offset = 0.0
    for bucket in buckets:
        if not isinstance(bucket, dict):
            continue
        zone = bucket.get("zoneNumber")
        secs = safe_float(bucket.get("secondsInZone"), 0.0) or 0.0
        # Without a per-sample HR stream we can at least model time-in-zone.
        # We only emit a series when Garmin gives us average HR per bucket.
        avg = safe_float(bucket.get("avgHeartRate"))
        if avg is None:
            continue
        seconds.append(int(offset))
        values.append(avg)
        offset += secs
    if not seconds:
        return None
    return {"seconds": seconds, "values": values, "unit": "bpm"}


def _extract_geo(payload: Any) -> list[list[float]] | None:
    """Extract ``[[lat, lon], ...]`` from an activity-details payload."""
    if not isinstance(payload, dict):
        return None
    polyline = payload.get("activityPolylineDTO") or {}
    entries = polyline.get("polylineDTOList") or polyline.get("polylineDTO") or []
    points: list[list[float]] = []
    if isinstance(entries, list):
        for entry in entries:
            if not isinstance(entry, dict):
                continue
            poly = entry.get("polyline")
            if isinstance(poly, str) and poly:
                from .garmin_oauth import points_from_encoded

                points.extend(points_from_encoded(poly))
    return points or None
