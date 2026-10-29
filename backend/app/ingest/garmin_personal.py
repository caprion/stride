"""Garmin Connect **Personal API** client.

This is the good path. Garmin's Personal API is a first-party API you can use
for your own account: you generate a Personal API Key from
``connect.garmin.com/app/settings/personalApiToken``, and it needs no
developer-program registration, no consumer secret and no OAuth browser dance.

Flow
----
1. ``POST https://personal.connectapi.garmin.com/personalapitoken-service/
   personal-api/personal-api-key/token`` with ``Authorization: ApiKey <key>``
   returns a JWT access token valid for 24 hours.
2. Every other endpoint is a plain bearer-authenticated GET against
   ``https://personal.connectapi.garmin.com``.

The token carries these scopes:

===========================  ==========================================
``CONNECT_READ``             activities, workouts, calendar
``PERSONAL_API_FITNESS_READ``  activities and their measurements
``PERSONAL_API_WELLNESS_READ``  sleep, HRV, resting HR, stress, body battery
``PERSONAL_API_USER_READ``     profile, settings, zones
``PERSONAL_API_NUTRITION_READ`` calories and nutrition
===========================  ==========================================

Endpoint map (all verified live against Garmin):

======================================================  ====================
``/activitylist-service/activities/search/activities``    activity search
``/activity-service/activity/{id}``                      detail + splits
``/activity-service/activity/{id}/workouts``              associated workouts
``/download-service/files/activity/{id}``                 original FIT (zip)
``/sleep-service/sleep/dailySleep?date=``                one night's sleep
``/sleep-service/sleep/dailySleepSummariesByDate?date=``  sleep summary
``/hrv-service/hrv/{date}``                              one day's HRV
``/hrv-service/hrv/daily/{from}/{to}``                   HRV range
``/wellness-service/wellness/dailyHeartRate?date=``      resting HR + curve
``/wellness-service/wellness/bodyBattery/reports/daily``  body battery
``/usersummary-service/usersummary/daily?calendarDate=``  daily summary
``/usersummary-service/stats/stress/daily/{from}/{to}``  stress series
``/usersummary-service/stats/steps/daily/{from}/{to}``   steps series
``/biometric-service/heartRateZones/v2``                 HR zone model
``/weight-service/weight/all/{from}/{to}``               weight
``/fitnessage-service/fitnessage/v2/{date}``             fitness age
``/userprofile-service/userprofile/settings``            profile
======================================================  ====================
"""

from __future__ import annotations

import io
import logging
import re
import time
import zipfile
import zlib
from urllib.parse import urljoin, urlparse
from typing import Any

import requests

log = logging.getLogger(__name__)

BASE = "https://personal.connectapi.garmin.com"
TOKEN_PATH = "/personalapitoken-service/personal-api/personal-api-key/token"
GARMIN_HOSTS = {"personal.connectapi.garmin.com", "connectapi.garmin.com", "connect.garmin.com"}
MAX_RESPONSE_BYTES = 5 * 1024 * 1024
MAX_FIT_BYTES = 5 * 1024 * 1024
MAX_FIT_MEMBER_BYTES = 20 * 1024 * 1024
MAX_ZIP_ENTRIES = 8


class PersonalApiError(RuntimeError):
    """The Personal API rejected our key or token."""


class GarminPersonalClient:
    """Client for the Garmin Connect Personal API.

    The access token is cached in memory and refreshed automatically; the
    long-lived Personal API Key never leaves this object.
    """

    def __init__(self, api_key: str, timeout: int = 30) -> None:
        self.api_key = api_key
        self.timeout = timeout
        self._token: str | None = None
        self._token_expires_at: float = 0.0
        self._seen: set[str] = set()
        #: How many API calls this client has made, token exchange included.
        #: A sync reports it, so the cost of a pull is a number rather than a
        #: guess — and a phone can show it.
        self.calls = 0

    @staticmethod
    def _request(method: str, url: str, headers: dict[str, str], params: Any = None) -> requests.Response:
        """Request Garmin with verified TLS, bounded redirects and response bytes."""
        if urlparse(url).scheme != "https" or urlparse(url).hostname not in GARMIN_HOSTS:
            raise PersonalApiError("Refusing to send Garmin credentials to a non-Garmin host.")
        current = url
        current_headers = dict(headers)
        rate_attempt = 0
        for _ in range(6):
            response = requests.request(
                method, current, headers=current_headers, params=params,
                timeout=(10, 30), verify=True, allow_redirects=False, stream=True,
            )
            if response.is_redirect:
                location = response.headers.get("Location")
                response.close()
                if not location:
                    raise PersonalApiError("Garmin returned a redirect without a location.")
                destination = urljoin(current, location)
                parsed = urlparse(destination)
                if parsed.scheme != "https" or parsed.hostname not in GARMIN_HOSTS:
                    current_headers.pop("Authorization", None)
                    raise PersonalApiError("Refusing redirect outside Garmin hosts.")
                if parsed.hostname != urlparse(current).hostname:
                    current_headers.pop("Authorization", None)
                current, params = destination, None
                continue
            if response.status_code == 429 and rate_attempt < 3:
                response.close()
                time.sleep(2 ** rate_attempt)
                rate_attempt += 1
                continue
            content_length = response.headers.get("Content-Length")
            if content_length is not None:
                try:
                    if int(content_length) > MAX_RESPONSE_BYTES:
                        response.close()
                        raise PersonalApiError("Garmin response exceeds 5 MB limit.")
                except ValueError:
                    response.close()
                    raise PersonalApiError("Garmin returned an invalid Content-Length.")
            chunks = bytearray()
            for chunk in response.iter_content(64 * 1024):
                if chunk:
                    chunks.extend(chunk)
                    if len(chunks) > MAX_RESPONSE_BYTES:
                        response.close()
                        raise PersonalApiError("Garmin response exceeds 5 MB limit.")
            response._content = bytes(chunks)
            response._content_consumed = True
            return response
        raise PersonalApiError("Too many Garmin redirects.")

    # ------------------------------------------------------------- plumbing

    def access_token(self, force: bool = False) -> str | None:
        """Exchange the Personal API Key for a bearer token."""
        if not force and self._token and time.time() < self._token_expires_at - 60:
            return self._token
        try:
            self.calls += 1
            response = self._request("POST", f"{BASE}{TOKEN_PATH}", {"Authorization": f"ApiKey {self.api_key}"})
        except requests.RequestException as exc:
            raise PersonalApiError(f"Could not reach Garmin: {exc}") from exc
        if response.status_code in (401, 403):
            raise PersonalApiError(
                "Garmin rejected the Personal API Key. Generate a new one at "
                "https://connect.garmin.com/app/settings/personalApiToken"
            )
        if response.status_code >= 400:
            raise PersonalApiError(
                f"Garmin token request failed ({response.status_code}): {response.text[:200]}"
            )
        token = (response.text or "").strip().strip('"')
        if not token:
            raise PersonalApiError("Garmin returned an empty access token.")
        self._token = token
        # The token is valid for 24 hours; refresh a minute early.
        self._token_expires_at = time.time() + 23 * 3600
        self._seen.add("access_token")
        return token

    def _get(self, path: str, params: dict[str, Any] | None = None, retry: bool = True) -> Any:
        """Bearer-authenticated GET with a single transparent token refresh."""
        token = self.access_token()
        if not token:
            raise PersonalApiError("No access token.")
        url = f"{BASE}{path}"
        try:
            self.calls += 1
            response = self._request("GET", url, {"Authorization": f"Bearer {token}"}, params=params)
        except requests.RequestException as exc:
            raise PersonalApiError(f"Request to {path} failed: {exc}") from exc

        if response.status_code in (401, 403) and retry:
            # The token may have expired underneath us — refresh once.
            self.access_token(force=True)
            return self._get(path, params, retry=False)

        if response.status_code == 204 or not response.content:
            self._seen.add(path.split("/")[1] if "/" in path else path)
            return None
        if response.status_code >= 400:
            raise PersonalApiError(
                f"{path} -> HTTP {response.status_code}: {response.text[:200]}"
            )
        self._seen.add(path.split("/")[1] if "/" in path else path)
        try:
            return response.json()
        except ValueError:
            return response.content

    def _get_bytes(self, path: str) -> bytes | None:
        token = self.access_token()
        if not token:
            return None
        self.calls += 1
        response = self._request("GET", f"{BASE}{path}", {"Authorization": f"Bearer {token}"})
        if response.status_code >= 400:
            return None
        self._seen.add(path.split("/")[1] if "/" in path else path)
        return response.content

    # ------------------------------------------------------------- profile

    def profile(self) -> dict[str, Any]:
        payload = self._get("/userprofile-service/userprofile/settings")
        return payload if isinstance(payload, dict) else {}

    def heart_rate_zones(self) -> dict[str, Any]:
        payload = self._get("/biometric-service/heartRateZones/v2")
        return payload if isinstance(payload, dict) else {}

    # ---------------------------------------------------------- activities

    def activities(
        self,
        start: int = 0,
        limit: int = 100,
        start_date: str | None = None,
        end_date: str | None = None,
    ) -> list[dict[str, Any]]:
        """Search activities, newest first."""
        try:
            start = max(0, min(int(start), 299))
        except (TypeError, ValueError, OverflowError):
            start = 0
        try:
            limit = max(1, min(int(limit), 100, 300 - start))
        except (TypeError, ValueError, OverflowError):
            limit = 100
        start_date, end_date = _bounded_date_range(start_date, end_date)
        params: dict[str, Any] = {"start": start, "limit": limit}
        if start_date:
            params["startDate"] = start_date
        if end_date:
            params["endDate"] = end_date
        payload = self._get("/activitylist-service/activities/search/activities", params)
        return _as_list(payload, ("activityList", "activities", "activitySummaries"))[:limit]

    def activity_detail(self, activity_id: str | int) -> dict[str, Any]:
        payload = self._get(f"/activity-service/activity/{_activity_id(activity_id)}")
        return payload if isinstance(payload, dict) else {}

    def activity_workouts(self, activity_id: str | int) -> list[dict[str, Any]]:
        payload = self._get(f"/activity-service/activity/{_activity_id(activity_id)}/workouts")
        return _as_list(payload, ("workouts", "activityWorkouts", "workoutSummaries"))

    def activity_fit(self, activity_id: str | int) -> bytes | None:
        """The original FIT file, extracted from Garmin's zip container.

        This is the full-fidelity route: per-second heart rate, cadence and
        GPS, which the summary endpoints do not carry.
        """
        blob = self._get_bytes(f"/download-service/files/activity/{_activity_id(activity_id)}")
        if not blob:
            return None
        if blob[:2] == b"PK":
            try:
                with zipfile.ZipFile(io.BytesIO(blob)) as archive:
                    entries = archive.infolist()
                    if len(entries) > MAX_ZIP_ENTRIES:
                        return None
                    for entry in entries:
                        name = entry.filename
                        parts = name.replace("\\", "/").split("/")
                        if name.startswith(("/", "\\")) or ".." in parts or (len(name) > 1 and name[1] == ":"):
                            return None
                        if entry.file_size > MAX_FIT_MEMBER_BYTES:
                            return None
                        if name.lower().endswith(".fit"):
                            if entry.compress_size > MAX_FIT_BYTES:
                                return None
                            with archive.open(entry) as member:
                                result = member.read(MAX_FIT_MEMBER_BYTES + 1)
                            return result if len(result) <= MAX_FIT_MEMBER_BYTES else None
            except (zipfile.BadZipFile, OSError, RuntimeError, EOFError, ValueError, zlib.error):
                return None
            return None
        # Some activities come back as a bare FIT file.
        return blob if len(blob) <= MAX_FIT_MEMBER_BYTES and blob[:1] == b"\x0e" else None

    # ------------------------------------------------------------ wellness

    def sleep(self, date: str) -> dict[str, Any]:
        payload = self._get("/sleep-service/sleep/dailySleep", {"date": date})
        return payload if isinstance(payload, dict) else {}

    def sleep_range(self, start: str, end: str) -> list[dict[str, Any]]:
        payload = self._get("/sleep-service/sleep/dailySleepSummariesByDate", {"date": start})
        return _as_list(payload, ("sleepSummaries", "dailySleepSummaries"))

    def hrv(self, date: str) -> dict[str, Any]:
        payload = self._get(f"/hrv-service/hrv/{date}")
        return payload if isinstance(payload, dict) else {}

    def hrv_range(self, start: str, end: str) -> list[dict[str, Any]]:
        start, end = _bounded_date_range(start, end)
        payload = self._get(f"/hrv-service/hrv/daily/{start}/{end}")
        return _as_list(payload, ("hrvDailySummaries", "hrvSummaries", "hrvList"))

    def daily_heart_rate(self, date: str) -> dict[str, Any]:
        payload = self._get("/wellness-service/wellness/dailyHeartRate", {"date": date})
        return payload if isinstance(payload, dict) else {}

    def body_battery(self, start: str, end: str) -> list[dict[str, Any]]:
        start, end = _bounded_date_range(start, end)
        payload = self._get(
            "/wellness-service/wellness/bodyBattery/reports/daily",
            {"startDate": start, "endDate": end},
        )
        return _as_list(payload, ("bodyBatteryReports", "bodyBatteryList", "reports"))

    def daily_summary(self, date: str) -> dict[str, Any]:
        payload = self._get("/usersummary-service/usersummary/daily", {"calendarDate": date})
        return payload if isinstance(payload, dict) else {}

    def stress(self, start: str, end: str) -> list[dict[str, Any]]:
        start, end = _bounded_date_range(start, end)
        payload = self._get(f"/usersummary-service/stats/stress/daily/{start}/{end}")
        return _as_list(payload, ("stressValuesArray", "stressDailyList", "stressList"))

    def steps(self, start: str, end: str) -> list[dict[str, Any]]:
        start, end = _bounded_date_range(start, end)
        payload = self._get(f"/usersummary-service/stats/steps/daily/{start}/{end}")
        return _as_list(payload, ("stepsDailyList", "stepsList", "stepsValuesArray"))

    def weight(self, start: str, end: str) -> list[dict[str, Any]]:
        start, end = _bounded_date_range(start, end)
        payload = self._get(f"/weight-service/weight/all/{start}/{end}")
        return _as_list(payload, ("dateWeightList", "weightList", "weights"))

    def fitness_age(self, date: str) -> dict[str, Any]:
        payload = self._get(f"/fitnessage-service/fitnessage/v2/{date}")
        return payload if isinstance(payload, dict) else {}

    # -------------------------------------------------------------- status

    def sources(self) -> dict[str, bool]:
        """Which endpoint families have answered at least once.

        Reports from what has been fetched — it never calls Garmin, because a
        status check must not be able to rate-limit the sync.
        """
        return {
            "activities": "activitylist-service" in self._seen,
            "activity_detail": "activity-service" in self._seen,
            "sleep": "sleep-service" in self._seen,
            "hrv": "hrv-service" in self._seen,
            "daily_heart_rate": "wellness-service" in self._seen,
            "body_battery": "wellness-service" in self._seen,
            "daily_summary": "usersummary-service" in self._seen,
            "stress": "usersummary-service" in self._seen,
            "steps": "usersummary-service" in self._seen,
            "heart_rate_zones": "biometric-service" in self._seen,
            "weight": "weight-service" in self._seen,
            "profile": "userprofile-service" in self._seen,
        }

    def validate(self) -> dict[str, Any]:
        """Prove the key works and describe the athlete it belongs to."""
        token = self.access_token(force=True)
        profile = self.profile()
        return {
            "ok": bool(token),
            "profile": profile,
            "sources": self.sources(),
        }


def _as_list(payload: Any, keys: tuple[str, ...]) -> list[dict[str, Any]]:
    """Pull an array out of whichever envelope Garmin used."""
    if isinstance(payload, list):
        return [item for item in payload if isinstance(item, dict)]
    if not isinstance(payload, dict):
        return []
    for key in keys:
        value = payload.get(key)
        if isinstance(value, list):
            return [item for item in value if isinstance(item, dict)]
    return []


def _activity_id(value: str | int) -> str:
    text = str(value)
    if not re.fullmatch(r"[0-9]{1,20}", text):
        raise ValueError("Invalid Garmin activity ID")
    return text


def _bounded_date_range(start: str | None, end: str | None) -> tuple[str | None, str | None]:
    """Normalize date inputs to a valid, non-negative range of at most a year."""
    from datetime import date, timedelta

    today = date.today()
    try:
        end_day = date.fromisoformat(end) if end else today
    except (TypeError, ValueError):
        end_day = today
    try:
        start_day = date.fromisoformat(start) if start else end_day - timedelta(days=365)
    except (TypeError, ValueError):
        start_day = end_day - timedelta(days=365)
    if start_day > end_day:
        start_day = end_day
    if (end_day - start_day).days > 365:
        start_day = end_day - timedelta(days=365)
    return start_day.isoformat(), end_day.isoformat()
