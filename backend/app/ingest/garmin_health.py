"""Garmin Health API ingestion for Stride's daily physiology picture.

These endpoints belong to Garmin's Health API product and are not reachable
with an Activity-API developer key; they require an account session. Every
method degrades to an empty result rather than raising when that session or
health data is unavailable.
"""
from __future__ import annotations

from typing import Any

from .normalize import safe_float

__all__ = ["GarminHealthClient"]


def _walk(obj: Any, names: set[str]) -> Any:
    """Find the first non-empty matching key anywhere in a Garmin payload."""
    wanted = {name.lower() for name in names}
    if isinstance(obj, dict):
        for key, value in obj.items():
            if key.lower() in wanted and value is not None:
                return value
        for value in obj.values():
            found = _walk(value, names)
            if found is not None:
                return found
    elif isinstance(obj, list):
        for value in obj:
            found = _walk(value, names)
            if found is not None:
                return found
    return None


class GarminHealthClient:
    """Defensive normalisers for Garmin health endpoint responses."""

    def __init__(self, client: Any = None) -> None:
        self.client = client
        self._seen: set[str] = set()

    def _get(self, method: str, *args: Any) -> Any:
        """Call one installed Garmin endpoint and remember non-empty replies."""
        try:
            getter = getattr(self.client, method, None)
            payload = getter(*args) if callable(getter) else None
        except Exception:
            return None
        if payload:
            self._seen.add(method)
        return payload

    def sleep_for(self, date: str) -> dict[str, Any]:
        """Fetch sleep seconds, score and stages from dailySleepDTO/level arrays."""
        payload = self._get("get_sleep_data", date)
        if not isinstance(payload, dict) or not payload:
            return {}
        duration = safe_float(_walk(payload, {"sleepTimeSeconds", "sleepDurationSeconds", "durationSeconds"}))
        score = safe_float(_walk(payload, {"sleepScore", "overallSleepScore", "value"}))
        raw_stages = _walk(payload, {"sleepLevels", "sleepLevelsDetailed"}) or []
        stages: dict[str, float | None] = {"deep_s": None, "rem_s": None, "light_s": None, "awake_s": None}
        totals = {"deep_s": 0.0, "rem_s": 0.0, "light_s": 0.0, "awake_s": 0.0}
        found: set[str] = set()
        for stage in raw_stages if isinstance(raw_stages, list) else []:
            if not isinstance(stage, dict):
                continue
            label = str(stage.get("activityLevel") or stage.get("level") or stage.get("sleepLevel") or "").lower()
            seconds = safe_float(_walk(stage, {"seconds", "durationInSeconds", "durationSeconds"}))
            if seconds is None:
                start = safe_float(stage.get("startGMT"))
                end = safe_float(stage.get("endGMT"))
                if start is not None and end is not None:
                    seconds = max(0.0, end - start)
            if seconds is None:
                continue
            for key, terms in (("deep_s", ("deep",)), ("rem_s", ("rem",)), ("light_s", ("light",)), ("awake_s", ("awake", "wake"))):
                if any(term in label for term in terms):
                    totals[key] += seconds
                    found.add(key)
                    break
        for key in found:
            stages[key] = totals[key]
        if duration is None and score is None and not found:
            return {}
        return {"duration_s": duration, "score": score, **stages}

    def sleep_range(self, start: str, end: str) -> list[dict[str, Any]]:
        """Fetch daily sleep summaries for a range using Garmin's range endpoint."""
        payload = self._get("get_sleep_daily", start, end)
        rows = payload if isinstance(payload, list) else []
        return [row for row in rows if isinstance(row, dict)]

    def hrv_range(self, start: str, end: str) -> dict[str, Any]:
        """Fetch Garmin's HRV daily series for an inclusive date range."""
        payload = self._get("get_hrv_data_range", start, end)
        return payload if isinstance(payload, dict) else {}

    def body_battery(self, date: str) -> dict[str, Any]:
        """Fetch daily body battery snapshots and calculate minimum/mean/maximum."""
        payload = self._get("get_body_battery", date, date)
        if not payload:
            return {}
        values: list[float] = []
        raw = _walk(payload, {"bodyBatteryValues", "bodyBatterySnapshotDTO", "bodyBattery"})
        candidates = raw if isinstance(raw, list) else [raw or payload]
        for item in candidates:
            value = safe_float(_walk(item, {"bodyBatteryLevel", "bodyBattery", "charged", "value", "level"}))
            if value is not None:
                values.append(value)
        if not values:
            return {}
        return {"min": min(values), "avg": sum(values) / len(values), "max": max(values)}

    def stress(self, date: str) -> dict[str, Any]:
        """Fetch daily stress average and maximum from stress payload fields/series."""
        payload = self._get("get_stress_data", date)
        if not isinstance(payload, dict) or not payload:
            return {}
        average = safe_float(_walk(payload, {"avgStressLevel", "averageStress", "stressAvg"}))
        maximum = safe_float(_walk(payload, {"maxStressLevel", "maxStress", "stressMax"}))
        values = _walk(payload, {"stressValuesArray", "stressValues"})
        numbers = [safe_float(v[-1] if isinstance(v, (tuple, list)) and v else v) for v in values] if isinstance(values, list) else []
        numbers = [v for v in numbers if v is not None and v >= 0]
        if average is None and numbers:
            average = sum(numbers) / len(numbers)
        if maximum is None and numbers:
            maximum = max(numbers)
        return {"avg": average, "max": maximum} if average is not None or maximum is not None else {}

    def hrv(self, date: str) -> dict[str, Any]:
        """Fetch nightly HRV milliseconds and status from hrvSummary fields."""
        payload = self._get("get_hrv_data", date)
        if not isinstance(payload, dict) or not payload:
            return {}
        value = safe_float(_walk(payload, {"lastNightAvg", "hrvValue", "lastNightAverage", "weeklyAvg"}))
        status = _walk(payload, {"status", "feedbackPhrase", "hrvStatus"})
        return {"value_ms": value, "status": status} if value is not None or status else {}

    def resting_hr(self, date: str) -> dict[str, Any]:
        """Fetch resting heart rate from Garmin's daily RHR response."""
        payload = self._get("get_rhr_day", date)
        value = safe_float(_walk(payload, {"restingHeartRate", "restingHr", "value"}))
        return {"resting_hr": value} if value is not None else {}

    def daily_summary(self, date: str) -> dict[str, Any]:
        """Fetch summaryDTO totals including steps, RHR, stress and VO2 max."""
        payload = self._get("get_user_summary", date)
        if not payload:
            return {}
        result = {
            "steps": safe_float(_walk(payload, {"totalSteps", "steps"})),
            "resting_hr": safe_float(_walk(payload, {"restingHeartRate", "restingHr"})),
            "stress": safe_float(_walk(payload, {"averageStressLevel", "averageStress"})),
            "stress_max": safe_float(_walk(payload, {"maxStressLevel", "maxStress"})),
            "vo2max": safe_float(_walk(payload, {"vo2Max", "vo2max"})),
        }
        return result if any(value is not None for value in result.values()) else {}

    def training_status(self, date: str) -> dict[str, Any]:
        """Fetch daily load/readiness from Garmin's training status and readiness APIs."""
        status = self._get("get_training_status", date)
        readiness = self._get("get_training_readiness", date)
        payload = {"status": status, "readiness": readiness}
        result = {
            "acute_load": safe_float(_walk(payload, {"acuteLoad", "trainingLoad7Days", "load7Days"})),
            "chronic_load": safe_float(_walk(payload, {"chronicLoad", "trainingLoad28Days", "load28Days"})),
            "vo2max": safe_float(_walk(payload, {"vo2Max", "vo2max"})),
            "training_readiness": safe_float(_walk(payload, {"score", "trainingReadiness", "readinessScore"})),
            "endurance_score": safe_float(_walk(payload, {"enduranceScore"})),
            "running_tolerance": safe_float(_walk(payload, {"runningTolerance"})),
        }
        return {key: value for key, value in result.items() if value is not None}

    def heart_rate_zones(self) -> dict[str, Any]:
        """Fetch the athlete's own HR boundaries and zone names from metrics API."""
        payload = self._get("get_heart_rate_zones")
        if isinstance(payload, dict):
            return payload
        if isinstance(payload, list):
            return {"zones": payload}
        return {}

    def health_sources(self) -> dict[str, bool]:
        """Report endpoint families observed during this client's actual pulls."""
        return {
            "sleep": bool(self._seen.intersection({"get_sleep_data", "get_sleep_daily"})),
            "body_battery": "get_body_battery" in self._seen,
            "stress": "get_stress_data" in self._seen,
            "hrv": bool(self._seen.intersection({"get_hrv_data", "get_hrv_data_range"})),
            "resting_hr": "get_rhr_day" in self._seen,
            "daily_summary": "get_user_summary" in self._seen,
            "training_status": bool(self._seen.intersection({"get_training_status", "get_training_readiness"})),
            "heart_rate_zones": "get_heart_rate_zones" in self._seen,
        }
