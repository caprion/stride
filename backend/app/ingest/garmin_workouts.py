"""Garmin workout-plan and shoe data for Stride's plan-versus-actual view."""
from __future__ import annotations

import re
from typing import Any

from .normalize import safe_float

__all__ = ["GarminWorkoutClient"]


def _walk(obj: Any, names: set[str]) -> Any:
    """Find the first matching non-null field in Garmin's nested DTO payloads."""
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


class GarminWorkoutClient:
    """Normalise Garmin calendar workouts, workout templates and gear."""

    def __init__(self, client: Any = None) -> None:
        self.client = client

    def _call(self, method: str, *args: Any) -> Any:
        """Call an account endpoint without surfacing auth or network errors."""
        try:
            getter = getattr(self.client, method, None)
            return getter(*args) if callable(getter) else None
        except Exception:
            return None

    @staticmethod
    def _list(payload: Any, *keys: str) -> list[Any]:
        """Extract a list payload from common Garmin collection wrappers."""
        if isinstance(payload, list):
            return payload
        if isinstance(payload, dict):
            for key in keys:
                value = payload.get(key)
                if isinstance(value, list):
                    return value
        return []

    def scheduled_workouts(self, year: int, month: int) -> list[dict[str, Any]]:
        """Fetch scheduled sessions via get_scheduled_workouts(year, month)."""
        payload = self._call("get_scheduled_workouts", year, month)
        rows = self._list(payload, "scheduledWorkouts", "workouts", "calendarItems", "items")
        result: list[dict[str, Any]] = []
        for row in rows:
            if not isinstance(row, dict):
                continue
            result.append({
                "workout_id": row.get("workoutId") or row.get("id") or row.get("scheduledWorkoutId"),
                "name": row.get("workoutName") or row.get("name") or row.get("title"),
                "date": row.get("date") or row.get("scheduledDate") or row.get("calendarDate"),
                "sport": row.get("sportType") or row.get("sportTypeKey") or row.get("activityType"),
                "steps": row.get("workoutSteps") or row.get("steps") or [],
                "status": row.get("status") or row.get("workoutStatus"),
            })
        return result

    def workout_library(self, limit: int) -> list[dict[str, Any]]:
        """Fetch up to limit workout templates via get_workouts(start, limit)."""
        payload = self._call("get_workouts", 0, max(1, int(limit)))
        return [row for row in self._list(payload, "workouts", "items") if isinstance(row, dict)]

    @staticmethod
    def _flatten_steps(steps: Any) -> list[dict[str, Any]]:
        """Flatten nested repeat stepList structures while preserving order."""
        result: list[dict[str, Any]] = []
        if not isinstance(steps, list):
            return result
        for step in steps:
            if not isinstance(step, dict):
                continue
            result.append(step)
            nested = step.get("stepList")
            if isinstance(nested, list):
                result.extend(GarminWorkoutClient._flatten_steps(nested))
        return result

    @staticmethod
    def _pace(value: Any) -> float | None:
        """Convert a Garmin pace value (s/m or m:ss) into seconds per kilometre."""
        if isinstance(value, str) and ":" in value:
            try:
                minutes, seconds = value.split(":", 1)
                return (float(minutes) * 60 + float(seconds))
            except (TypeError, ValueError):
                return None
        number = safe_float(value)
        if number is None:
            return None
        return number * 1000 if 0 < number < 20 else number

    def workout_detail(self, workout_id: int) -> dict[str, Any]:
        """Fetch template steps and derive session type, distance and pace targets."""
        payload = self._call("get_workout_by_id", workout_id)
        if not isinstance(payload, dict) or not payload:
            return {}
        steps = self._flatten_steps(payload.get("workoutSteps") or payload.get("steps"))
        title = str(payload.get("workoutName") or payload.get("name") or "")
        raw_titles = [str(step.get("description") or step.get("stepName") or step.get("stepType") or "") for step in steps]
        text = (title + " " + " ".join(raw_titles)).lower()
        if "race" in text or "competition" in text:
            session_type = "race"
        elif any(term in text for term in ("tempo", "threshold", "lactate")):
            session_type = "tempo"
        elif "strength" in text or "weight" in text:
            session_type = "strength"
        elif "rest" in text:
            session_type = "rest"
        elif "recovery" in text:
            session_type = "recovery"
        elif any("repeat" in str(step.get("stepType", "")).lower() for step in steps) or any(isinstance(step.get("stepList"), list) for step in steps):
            session_type = "interval"
        else:
            session_type = "easy"

        distance = 0.0
        paces: list[float] = []
        zones: list[int] = []
        long_duration = False
        for step in steps:
            duration_type = str(step.get("durationType") or "").lower()
            target_distance = safe_float(step.get("targetValue"))
            if duration_type == "distance" and target_distance is not None:
                distance += target_distance
            duration = safe_float(step.get("targetValue"))
            if duration_type == "time" and duration is not None and duration > 3600:
                long_duration = True
            target_type = str(step.get("targetType") or step.get("targetValueType") or "").lower()
            if "pace" in target_type or target_type in {"speed", "pace_zone"}:
                for key in ("targetValueLow", "targetValue", "targetValueHigh"):
                    pace = self._pace(step.get(key))
                    if pace is not None:
                        paces.append(pace)
            zone = safe_float(step.get("targetValue") if duration_type.startswith("hr_") else step.get("targetZone"))
            if zone is not None and (duration_type.startswith("hr_") or "heart" in target_type):
                zones.append(int(zone))
        if session_type == "easy" and (distance > 12000 or long_duration):
            session_type = "long"
        if session_type == "easy" and len(steps) == 1:
            session_type = "easy"
        details = "; ".join(item for item in raw_titles if item) or None
        return {
            "session_type": session_type,
            "target_distance_m": distance or None,
            "target_pace_low_s_km": min(paces) if paces else None,
            "target_pace_high_s_km": max(paces) if paces else None,
            "hr_zone_target": min(zones) if zones else None,
            "title": title or (raw_titles[0] if raw_titles else None),
            "details": details,
        }

    def gear(self) -> list[dict[str, Any]]:
        """Fetch user shoes via get_gear(userProfileNumber), normalised by UUID."""
        profile = getattr(self.client, "profile_id", None) or getattr(self.client, "profile_number", None)
        payload = self._call("get_gear", str(profile or ""))
        rows = self._list(payload, "gearItems", "gear", "items", "userGear")
        result: list[dict[str, Any]] = []
        for row in rows:
            if not isinstance(row, dict):
                continue
            result.append({
                "uuid": row.get("uuid") or row.get("gearUUID"),
                "name": row.get("name") or row.get("gearName"),
                "brand": row.get("brand"),
                "model": row.get("model"),
                "status": row.get("status") or row.get("gearStatus") or "active",
            })
        return result
