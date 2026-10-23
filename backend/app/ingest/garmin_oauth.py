"""Garmin Connect Activity API client — OAuth1 three-legged flow.

This is the "plug in your Garmin key" path. The user pastes a Consumer Key and
Consumer Secret (from developer.garmin.com) into the app; we then run the
standard three-legged OAuth1 dance:

1. ``POST /oauth-service/oauth/request_token``  -> temporary token
2. user approves in a browser at ``/oauth/authorize``
3. ``POST /oauth-service/oauth/access_token``   -> long-lived access token
4. every API call is then OAuth1-signed with the access token

Notes learned the hard way (see PLAN.md §2):

* Only ``connectapi.garmin.com`` serves the OAuth endpoints — the same path on
  ``connect.garmin.com`` returns a 404 HTML page.
* There is **no** OAuth2 client-credentials endpoint; OAuth1 is the only option.
* Garmin throttles request-token issuance hard for datacenter IPs (429). The
  network layer (:mod:`app.network`) exists to work around that.
"""

from __future__ import annotations

import logging
import time
import uuid
from typing import Any
from urllib.parse import parse_qs, urlencode

import oauthlib.oauth1.rfc5849 as oauth1_sig
import requests
from requests_oauthlib import OAuth1Session

from .normalize import (
    blank_activity,
    decode_garmin_polyline,
    decode_polyline,
    duration_to_seconds,
    format_pace,
    normalise_sport,
    safe_float,
    safe_int,
)

log = logging.getLogger(__name__)

CONNECTAPI = "https://connectapi.garmin.com"
CONNECT_WEB = "https://connect.garmin.com"

REQUEST_TOKEN_URL = f"{CONNECTAPI}/oauth-service/oauth/request_token"
ACCESS_TOKEN_URL = f"{CONNECTAPI}/oauth-service/oauth/access_token"

# The user-facing authorisation page. Probed and confirmed:
#   GET /modern/authorize?oauth_token=...  -> 302 /app/authorize -> Garmin sign-in
# The connectapi host has an /oauth-service/oauth/authorize path but it is a
# service-to-service endpoint ("NotAllowedException"), not the browser flow.
AUTHORIZE_URL = f"{CONNECT_WEB}/modern/authorize"

# Activity API endpoints. Several exist for different generations of the API;
# we try them in order and use whichever answers.
LIST_ENDPOINTS = [
    ("GET", f"{CONNECTAPI}/activitylist-service/activities", {"start": "0", "limit": "100"}),
    ("GET", f"{CONNECTAPI}/activity-service/activity/search/activities", None),
]
ACTIVITY_URL = f"{CONNECTAPI}/activity-service/activity/{{}}"
DETAILS_URL = f"{CONNECTAPI}/activity-service/activity/{{}}/details"
SPLITS_URL = f"{CONNECTAPI}/activity-service/activity/{{}}/splits"
POLYLINE_URL = f"{CONNECTAPI}/activity-service/activity/{{}}/activityPolyline"
SUMMARY_URL = f"{CONNECTAPI}/activity-service/activity/{{}}/summary"


class GarminAuthError(RuntimeError):
    """Raised when Garmin rejects our credentials or a token."""


class GarminApiError(RuntimeError):
    """Raised on an unexpected HTTP status from the Activity API."""


class GarminOAuthClient:
    """OAuth1 client for the Garmin Connect Activity API."""

    def __init__(
        self,
        consumer_key: str,
        consumer_secret: str,
        oauth_token: str | None = None,
        oauth_token_secret: str | None = None,
        timeout: int = 30,
    ) -> None:
        self.consumer_key = consumer_key
        self.consumer_secret = consumer_secret
        self.oauth_token = oauth_token
        self.oauth_token_secret = oauth_token_secret
        self.timeout = timeout

    # ------------------------------------------------------------- plumbing

    def _session(self, callback: str | None = None) -> OAuth1Session:
        return OAuth1Session(
            self.consumer_key,
            self.consumer_secret,
            resource_owner_key=self.oauth_token,
            resource_owner_secret=self.oauth_token_secret,
            callback_uri=callback,
            signature_method=oauth1_sig.SIGNATURE_HMAC,
        )

    def _get(self, url: str, params: dict | None = None) -> requests.Response:
        session = self._session()
        try:
            response = session.get(url, params=params, timeout=self.timeout)
        except requests.RequestException as exc:  # pragma: no cover - network
            raise GarminApiError(f"Request to {url} failed: {exc}") from exc
        if response.status_code >= 400:
            raise GarminApiError(
                f"{url} -> HTTP {response.status_code}: {response.text[:200]}"
            )
        return response

    # ------------------------------------------------------- oauth handshake

    def request_token(self, callback: str) -> tuple[str, str]:
        """Step 1+2: get a request token and the URL the user must visit."""
        session = OAuth1Session(
            self.consumer_key,
            self.consumer_secret,
            callback_uri=callback,
            signature_method=oauth1_sig.SIGNATURE_HMAC,
        )
        try:
            response = session.fetch_request_token(REQUEST_TOKEN_URL, timeout=self.timeout)
        except Exception as exc:  # requests_oauthlib raises a grab-bag
            body = getattr(getattr(exc, "response", None), "text", "") or ""
            status = getattr(getattr(exc, "response", None), "status_code", None)
            if status == 429:
                raise GarminAuthError(
                    "Garmin rate-limited the authorization request. Your server IP is "
                    "likely a datacentre IP that Garmin throttles. Switch the network "
                    "route to the home exit node in Settings and try again."
                ) from exc
            raise GarminAuthError(
                f"Garmin rejected the consumer key ({status or 'network error'}): {body[:200]}"
            ) from exc

        parsed = parse_qs(response.text)
        token = (parsed.get("oauth_token") or [""])[0]
        secret = (parsed.get("oauth_token_secret") or [""])[0]
        if not token:
            raise GarminAuthError("Garmin returned an empty request token.")
        return token, secret

    @staticmethod
    def authorize_url(request_token: str) -> str:
        return f"{AUTHORIZE_URL}?{urlencode({'oauth_token': request_token})}"

    def access_token(self, request_token: str, request_secret: str, verifier: str) -> tuple[str, str]:
        """Step 3: exchange the approved request token for an access token."""
        session = OAuth1Session(
            self.consumer_key,
            self.consumer_secret,
            resource_owner_key=request_token,
            resource_owner_secret=request_secret,
            verifier=verifier,
            signature_method=oauth1_sig.SIGNATURE_HMAC,
        )
        try:
            response = session.fetch_access_token(ACCESS_TOKEN_URL, timeout=self.timeout)
        except Exception as exc:
            body = getattr(getattr(exc, "response", None), "text", "") or ""
            status = getattr(getattr(exc, "response", None), "status_code", None)
            raise GarminAuthError(
                f"Token exchange failed ({status or 'network error'}): {body[:200]}"
            ) from exc
        token = response.get("oauth_token")
        secret = response.get("oauth_token_secret")
        if not token or not secret:
            raise GarminAuthError("Garmin did not return an access token.")
        return token, secret

    # --------------------------------------------------------------- reading

    def list_activities(self, limit: int = 100) -> list[dict]:
        """List recent activities. Tries several endpoint shapes."""
        last_error: Exception | None = None
        for method, url, params in LIST_ENDPOINTS:
            try:
                if method == "GET":
                    response = self._get(url, params)
                else:
                    response = self._session().post(
                        url,
                        json={"limit": limit},
                        timeout=self.timeout,
                    )
                    if response.status_code >= 400:
                        raise GarminApiError(f"{url} -> HTTP {response.status_code}")
                payload = response.json()
            except (GarminApiError, ValueError) as exc:
                last_error = exc
                continue

            items = self._extract_list(payload)
            if items:
                return items[:limit]
        raise GarminApiError(f"Could not list activities: {last_error}")

    @staticmethod
    def _extract_list(payload: Any) -> list[dict]:
        """Pull the activity array out of whichever envelope Garmin used."""
        if isinstance(payload, list):
            return [p for p in payload if isinstance(p, dict)]
        if not isinstance(payload, dict):
            return []
        for key in (
            "activityList",
            "activities",
            "activityDTOList",
            "activityListDTO",
            "activityDTO",
        ):
            value = payload.get(key)
            if isinstance(value, list):
                return [p for p in value if isinstance(p, dict)]
            if isinstance(value, dict):
                for inner in ("activityDTOList", "activityList"):
                    if isinstance(value.get(inner), list):
                        return [p for p in value[inner] if isinstance(p, dict)]
        return []

    def activity_detail(self, activity_id: str | int) -> dict:
        try:
            return self._get(ACTIVITY_URL.format(activity_id)).json()
        except (GarminApiError, ValueError):
            return {}

    def activity_details(self, activity_id: str | int) -> dict:
        try:
            return self._get(DETAILS_URL.format(activity_id)).json()
        except (GarminApiError, ValueError):
            return {}

    def activity_splits(self, activity_id: str | int) -> dict:
        try:
            return self._get(SPLITS_URL.format(activity_id)).json()
        except (GarminApiError, ValueError):
            return {}

    def activity_polyline(self, activity_id: str | int) -> str | None:
        try:
            payload = self._get(POLYLINE_URL.format(activity_id)).json()
        except (GarminApiError, ValueError):
            return None
        return _extract_polyline(payload)

    def athlete_profile(self) -> dict:
        for path in (
            "/personal-information-service/userProfile",
            "/user-service/userprofile",
            "/personal-information-service/userSettings",
        ):
            try:
                return self._get(f"{CONNECTAPI}{path}").json()
            except (GarminApiError, ValueError):
                continue
        return {}

    # -------------------------------------------------------- normalisation

    def fetch_activities(self, days: int = 90, limit: int = 100) -> list[dict]:
        """Fetch activities and normalise them into canonical dicts."""
        raw_items = self.list_activities(limit=limit)
        cutoff = time.time() - days * 86400
        out: list[dict] = []
        for raw in raw_items:
            activity = self._normalise(raw)
            if activity is None:
                continue
            start = activity.get("_start_ts")
            if start is not None and start < cutoff:
                continue
            out.append(activity)
        return out

    def _normalise(self, raw: dict) -> dict | None:
        activity = blank_activity()
        external_id = raw.get("activityId") or raw.get("activityIdDTO") or raw.get("id")
        if external_id is None:
            return None
        activity["external_id"] = str(external_id)
        activity["source"] = "garmin"
        activity["raw"] = raw

        activity["name"] = (
            raw.get("activityName") or raw.get("name") or raw.get("activityTypeDTO", {}).get("typeKey", "Run")
        )
        type_key = (
            raw.get("activityType", {}).get("typeKey")
            or raw.get("activityTypeDTO", {}).get("typeKey")
            or raw.get("type")
            or "running"
        )
        activity["sport_type"] = normalise_sport(type_key)

        start_raw = raw.get("startTimeLocal") or raw.get("startTimeGMT") or raw.get("beginTimestamp")
        start = _parse_dt(start_raw)
        if start:
            activity["start_time"] = start.isoformat()
            activity["local_date"] = start.date().isoformat()
            activity["day_of_week"] = start.strftime("%A")
            activity["_start_ts"] = start.timestamp()

        distance = safe_float(raw.get("distance"))
        activity["distance_m"] = distance if distance is not None else 0.0
        activity["moving_time_s"] = duration_to_seconds(
            raw.get("duration") or raw.get("movingDuration") or raw.get("elapsedDuration")
        )
        activity["elapsed_time_s"] = duration_to_seconds(
            raw.get("elapsedDuration") or raw.get("duration")
        )
        activity["elevation_gain_m"] = safe_float(raw.get("elevationGain"), 0.0) or 0.0
        activity["avg_hr"] = safe_int(raw.get("averageHR") or raw.get("avgHeartRate"))
        activity["max_hr"] = safe_int(raw.get("maxHR") or raw.get("maxHeartRate"))
        activity["calories"] = safe_float(raw.get("calories"))
        activity["avg_cadence"] = safe_float(
            raw.get("averageRunningCadenceInStepsPerMinute")
            or raw.get("averageRunCadence")
            or raw.get("avgCadence")
        )
        activity["max_cadence"] = safe_float(raw.get("maxRunningCadenceInStepsPerMinute"))
        activity["avg_speed_ms"] = safe_float(raw.get("averageSpeed"))
        activity["max_speed_ms"] = safe_float(raw.get("maxSpeed"))
        activity["device"] = raw.get("deviceName") or raw.get("deviceId")
        activity["description"] = raw.get("description")
        activity["is_trail"] = "trail" in str(type_key)
        return activity


# ---------------------------------------------------------------- helpers


def _parse_dt(value: Any):
    from .normalize import parse_iso

    return parse_iso(value)


def _extract_polyline(payload: Any) -> str | None:
    """Find an encoded polyline string in a Garmin payload, at any depth."""
    if isinstance(payload, str):
        return payload or None
    if isinstance(payload, list):
        for item in payload:
            found = _extract_polyline(item)
            if found:
                return found
        return None
    if isinstance(payload, dict):
        for key in ("polyline", "polylineDTO", "polylineDTOList", "encodedPolyline"):
            if key in payload:
                found = _extract_polyline(payload[key])
                if found:
                    return found
        for value in payload.values():
            found = _extract_polyline(value)
            if found:
                return found
    return None


def points_from_encoded(encoded: str | None) -> list[list[float]]:
    if not encoded:
        return []
    points = decode_garmin_polyline(encoded)
    if points and all(len(p) == 2 and (p[0] != 0.0 or p[1] != 0.0) for p in points):
        return points
    return decode_polyline(encoded)
