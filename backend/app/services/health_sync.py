"""Persist Garmin Health API measurements and activity facts into Stride's schema."""
from __future__ import annotations

from datetime import date, timedelta
from typing import Any

from ..config import state
from ..db import SessionLocal
from ..ingest.garmin_account import GarminAccountClient
from ..ingest.garmin_health import GarminHealthClient
from ..ingest.garmin_workouts import GarminWorkoutClient
from ..models_phase2 import ActivityFact, DailyPhysiology, Gear
from .persistence import physiology_day_is_held

__all__ = ["sync_health", "sync_activity_facts", "sync_scheduled_workouts", "sources"]


def _account() -> GarminAccountClient | None:
    """Create and authenticate the configured Garmin account client safely."""
    try:
        account = GarminAccountClient(
            state.garmin_email or "",
            state.garmin_password or "",
            state.garmin_tokenstore or "",
        )
        account.login()
        return account
    except Exception:
        return None


def _first(obj: Any, keys: set[str]) -> Any:
    """Find a first matching field recursively through nested Garmin DTOs."""
    wanted = {key.lower() for key in keys}
    if isinstance(obj, dict):
        for key, value in obj.items():
            if key.lower() in wanted and value is not None:
                return value
        for value in obj.values():
            result = _first(value, keys)
            if result is not None:
                return result
    elif isinstance(obj, list):
        for value in obj:
            result = _first(value, keys)
            if result is not None:
                return result
    return None


def _date_index(payload: Any) -> dict[str, dict[str, Any]]:
    """Index range-response rows by their Garmin calendar date."""
    rows = payload if isinstance(payload, list) else []
    indexed: dict[str, dict[str, Any]] = {}
    for row in rows:
        if isinstance(row, dict):
            day = row.get("calendarDate") or row.get("date")
            if day:
                indexed[str(day)[:10]] = row
    return indexed


def sync_health(days: int = 30, force: bool = False) -> dict[str, Any]:
    """Pull recent physiology and upsert DailyPhysiology and Gear rows.

    The pull is incremental: a day we already hold is not fetched again.
    The Health API costs six or seven calls per day, so a 30-day window is
    roughly 200 requests — repeating all of them on every sync is 200
    requests a day, forever.

    Two days are exceptions:

    - **Today.** It is not over, so its body battery, stress and steps are
      still moving. It is pulled every time.
    - **A day with no measurements in it.** Nothing was written for it, so
      there is nothing to reuse.
    """
    account = _account()
    if account is None:
        return {"days": days, "days_written": 0, "errors": ["Garmin account unavailable"]}

    client = account._client
    health = GarminHealthClient(client)
    workouts = GarminWorkoutClient(client)
    errors: list[str] = []
    written = 0
    skipped = 0
    day_count = max(0, int(days))
    end_date = date.today()
    start_date = end_date - timedelta(days=max(0, day_count - 1))
    start = start_date.isoformat()
    end = end_date.isoformat()

    sleep_rows = _date_index(health.sleep_range(start, end))
    hrv_payload = health.hrv_range(start, end)
    hrv_rows = _date_index(hrv_payload.get("hrvDailySummary") or hrv_payload.get("dailyHrv") or hrv_payload.get("individualStats") or []) if isinstance(hrv_payload, dict) else {}

    db = SessionLocal()
    try:
        for offset in range(day_count):
            day = (end_date - timedelta(days=offset)).isoformat()
            row = db.get(DailyPhysiology, day)
            # Today is always pulled: the day is not over.
            if not force and day != end_date.isoformat() and physiology_day_is_held(row):
                skipped += 1
                continue
            try:
                sleep = health.sleep_for(day) if day not in sleep_rows else _normalise_sleep_row(sleep_rows[day])
                hrv = health.hrv(day) if day not in hrv_rows else _normalise_hrv_row(hrv_rows[day])
                battery = health.body_battery(day)
                stress = health.stress(day)
                rhr = health.resting_hr(day)
                summary = health.daily_summary(day)
                training = health.training_status(day)
                values: dict[str, Any] = {
                    "sleep_s": sleep.get("duration_s"),
                    "sleep_score": sleep.get("score"),
                    "deep_sleep_s": sleep.get("deep_s"),
                    "rem_s": sleep.get("rem_s"),
                    "light_sleep_s": sleep.get("light_s"),
                    "awake_s": sleep.get("awake_s"),
                    "hrv_ms": hrv.get("value_ms"),
                    "hrv_status": hrv.get("status"),
                    "resting_hr": rhr.get("resting_hr") or summary.get("resting_hr"),
                    "stress_avg": stress.get("avg") or summary.get("stress"),
                    "stress_max": stress.get("max") or summary.get("stress_max"),
                    "body_battery_min": battery.get("min"),
                    "body_battery_avg": battery.get("avg"),
                    "body_battery_max": battery.get("max"),
                    "steps": summary.get("steps"),
                    "training_readiness": training.get("training_readiness"),
                    "acute_load": training.get("acute_load"),
                    "chronic_load": training.get("chronic_load"),
                    "vo2max": training.get("vo2max") or summary.get("vo2max"),
                    "endurance_score": training.get("endurance_score"),
                    "running_tolerance": training.get("running_tolerance"),
                }
                values = {key: value for key, value in values.items() if value is not None}
                if not values:
                    continue
                if row is None:
                    row = DailyPhysiology(date=day, source="garmin")
                    db.add(row)
                for key, value in values.items():
                    setattr(row, key, value)
                written += 1
            except Exception as exc:
                errors.append(f"{day}: {exc}")

        for item in workouts.gear():
            try:
                uuid = item.get("uuid")
                if not uuid:
                    continue
                row = db.query(Gear).filter_by(uuid=str(uuid)).one_or_none()
                if row is None:
                    row = Gear(uuid=str(uuid))
                    db.add(row)
                for field in ("name", "brand", "model", "status"):
                    value = item.get(field)
                    if value is not None:
                        setattr(row, field, value)
            except Exception as exc:
                errors.append(f"gear: {exc}")
        db.commit()
    except Exception as exc:
        db.rollback()
        errors.append(str(exc))
    finally:
        db.close()
    return {
        "days": days,
        "days_written": written,
        "days_skipped": skipped,
        "requests": health.calls,
        "errors": errors,
    }


def _normalise_sleep_row(row: dict[str, Any]) -> dict[str, Any]:
    """Convert a range sleep summary row into the same client shape as sleep_for."""
    return {
        "duration_s": _first(row, {"sleepTimeSeconds", "sleepDurationSeconds", "durationSeconds"}),
        "score": _first(row, {"sleepScore", "overallSleepScore"}),
        "deep_s": _first(row, {"deepSleepSeconds", "deepSleepTimeSeconds"}),
        "rem_s": _first(row, {"remSleepSeconds", "remSleepTimeSeconds"}),
        "light_s": _first(row, {"lightSleepSeconds", "lightSleepTimeSeconds"}),
        "awake_s": _first(row, {"awakeSleepSeconds", "awakeSleepTimeSeconds"}),
    }


def _normalise_hrv_row(row: dict[str, Any]) -> dict[str, Any]:
    """Convert a range HRV row into a nightly value and status."""
    return {
        "value_ms": _first(row, {"lastNightAvg", "hrvValue", "lastNightAverage"}),
        "status": _first(row, {"status", "feedbackPhrase", "hrvStatus"}),
    }


def sync_activity_facts(activity_ids: list[int]) -> dict[str, Any]:
    """Fetch Garmin activity details and upsert available ActivityFact columns."""
    account = _account()
    if account is None:
        return {"written": 0, "errors": ["Garmin account unavailable"]}
    fields: dict[str, tuple[set[str], bool]] = {
        "aerobic_te": ({"aerobictrainingeffect", "aerobictrainingeffectvalue"}, False),
        "anaerobic_te": ({"anaerobictrainingeffect", "anaerobictrainingeffectvalue"}, False),
        "training_effect_label": ({"trainingeffectlabel"}, True),
        "vo2max": ({"vo2max"}, False),
        "recovery_time_min": ({"recoverytime", "recoverytimeminutes"}, False),
        "avg_gct_ms": ({"averagegroundcontacttime", "avggroundcontacttime"}, False),
        "avg_stride_ratio": ({"avgstrideratio", "averagestrideratio"}, False),
        "vertical_ratio": ({"verticalratio"}, False),
        "avg_power": ({"averagepower", "avgpower"}, False),
        "normalized_power": ({"normalizedpower"}, False),
        "avg_grade_adjusted_pace": ({"gradeadjustedpace", "averagegradeadjustedpace"}, False),
        "temperature_c": ({"averagetemperature", "temperature"}, False),
        "impact_estimate": ({"impactloading", "impactestimate"}, False),
        "gear_uuid": ({"gearuuid"}, True),
        "device_name": ({"devicename"}, True),
    }
    db = SessionLocal()
    errors: list[str] = []
    written = 0
    try:
        for activity_id in activity_ids:
            try:
                payload = account._client.get_activity_details(activity_id)
                row = db.get(ActivityFact, activity_id)
                if row is None:
                    row = ActivityFact(activity_id=activity_id)
                    db.add(row)
                for column, (names, is_text) in fields.items():
                    value = _first(payload, names)
                    if value is not None:
                        setattr(row, column, str(value) if is_text else float(value))
                written += 1
            except Exception as exc:
                errors.append(f"{activity_id}: {exc}")
        db.commit()
    except Exception as exc:
        db.rollback()
        errors.append(str(exc))
    finally:
        db.close()
    return {"written": written, "errors": errors}


def sync_scheduled_workouts(year: int, month: int) -> list[dict[str, Any]]:
    """Return normalized scheduled workouts from Garmin's calendar service."""
    account = _account()
    if account is None:
        return []
    return GarminWorkoutClient(account._client).scheduled_workouts(year, month)


def sources() -> dict[str, Any]:
    """Report physiology families represented by non-null stored daily values."""
    columns = {
        "sleep": ("sleep_s", "sleep_score", "deep_sleep_s", "rem_s", "light_sleep_s", "awake_s"),
        "hrv": ("hrv_ms", "hrv_status"),
        "resting_hr": ("resting_hr",),
        "stress": ("stress_avg", "stress_max"),
        "body_battery": ("body_battery_min", "body_battery_avg", "body_battery_max"),
        "daily_summary": ("steps",),
        "training_status": ("training_readiness", "acute_load", "chronic_load", "vo2max", "endurance_score", "running_tolerance"),
    }
    db = SessionLocal()
    try:
        rows = db.query(DailyPhysiology).all()
        available = {
            family: any(getattr(row, column) is not None for row in rows for column in family_columns)
            for family, family_columns in columns.items()
        }
    finally:
        db.close()
    return {
        "available": available,
        "reason": None if any(available.values()) else "No Garmin physiology values have been stored yet.",
    }
