"""Ingest from the Garmin Connect Personal API.

Two pulls:

``sync_activities``
    Activities, with full fidelity. Garmin's summary endpoints carry the
    authoritative display values (``averageRunCadence``, ``strideLength``,
    training effect); the original FIT file carries what they do not — the GPS
    track, per-second heart rate and the kilometre splits. We take both and
    prefer the API's numbers where the two disagree.

``sync_wellness``
    Sleep, HRV, resting heart rate, stress, body battery and steps for each
    day — the raw material for the readiness score and the plan-cause engine.
"""

from __future__ import annotations

import logging
from datetime import date, datetime, timedelta, timezone
from typing import Any

from sqlalchemy import select
from sqlalchemy.orm import Session

from ..config import DATA_DIR, state
from ..db import SessionLocal
from ..ingest.fit_importer import parse_fit
from ..ingest.garmin_personal import GarminPersonalClient
from ..ingest.normalize import blank_activity, safe_float, safe_int
from ..models import Activity
from ..models_phase2 import DailyPhysiology, Gear
from .persistence import store_activity

log = logging.getLogger(__name__)


def _client() -> GarminPersonalClient:
    return GarminPersonalClient(state.personal_api_key or "")


# ------------------------------------------------------------- activities


def _canonical_from_detail(detail: dict[str, Any], raw: dict[str, Any]) -> dict[str, Any]:
    """Turn an activity detail payload into a canonical activity."""
    activity = blank_activity()
    activity["source"] = "garmin_personal"

    activity_id = detail.get("activityId") or raw.get("activityId")
    if activity_id is None:
        return activity
    activity["external_id"] = str(activity_id)

    summary = detail.get("summaryDTO") or {}
    metadata = detail.get("metadataDTO") or {}
    type_dto = detail.get("activityTypeDTO") or {}

    activity["name"] = (
        detail.get("activityName") or raw.get("activityName") or raw.get("name") or "Run"
    )
    activity["sport_type"] = type_dto.get("typeKey") or "running"

    # The API's local start time is authoritative: the FIT file carries UTC,
    # which would put a late-evening run on the wrong calendar day.
    start_raw = summary.get("startTimeLocal") or raw.get("startTimeLocal")
    start = _parse_dt(start_raw)
    if start:
        activity["start_time"] = start.isoformat()
        activity["local_date"] = start.date().isoformat()
        activity["day_of_week"] = start.strftime("%A")

    activity["distance_m"] = safe_float(summary.get("distance"), 0.0) or 0.0
    activity["moving_time_s"] = safe_float(
        summary.get("movingDuration") or summary.get("duration"), 0.0
    ) or 0.0
    activity["elapsed_time_s"] = safe_float(
        summary.get("elapsedDuration") or summary.get("duration"), 0.0
    ) or 0.0
    activity["elevation_gain_m"] = safe_float(summary.get("elevationGain"), 0.0) or 0.0
    activity["avg_hr"] = safe_float(summary.get("averageHR"))
    activity["max_hr"] = safe_float(summary.get("maxHR"))
    activity["min_hr"] = safe_float(summary.get("minHR"))
    activity["calories"] = safe_float(summary.get("calories"))
    activity["avg_cadence"] = safe_float(summary.get("averageRunCadence"))
    activity["max_cadence"] = safe_float(summary.get("maxRunCadence"))
    activity["avg_speed_ms"] = safe_float(summary.get("averageSpeed"))
    activity["max_speed_ms"] = safe_float(summary.get("maxSpeed"))
    activity["device"] = _device_name(metadata)

    # Points of interest, when the watch recorded them.
    activity["geo_points"] = _geo(summary)
    activity["raw"] = {
        "training_effect": safe_float(summary.get("trainingEffect")),
        "aerobic_training_effect": safe_float(summary.get("aerobicTrainingEffect")),
        "anaerobic_training_effect": safe_float(summary.get("anaerobicTrainingEffect")),
        "stride_length_m": safe_float(summary.get("strideLength")),
        "min_hr": safe_float(summary.get("minHR")),
        "bmr_calories": safe_float(summary.get("bmrCalories")),
        "max_elevation_m": safe_float(summary.get("maxElevation")),
        "min_elevation_m": safe_float(summary.get("minElevation")),
    }
    return activity


def _device_name(metadata: dict[str, Any]) -> str | None:
    sensors = metadata.get("sensors") or []
    if isinstance(sensors, list) and sensors:
        first = sensors[0]
        if isinstance(first, dict):
            manufacturer = first.get("manufacturer")
            product = first.get("product")
            if manufacturer or product:
                return f"{manufacturer or ''} {product or ''}".strip() or None
    manufacturer = metadata.get("manufacturer")
    return str(manufacturer) if manufacturer else None


def _geo(summary: dict[str, Any]) -> list[list[float]] | None:
    """Start/end points are not a route — a route needs the FIT track."""
    return None


def _parse_dt(value: Any) -> datetime | None:
    if not value:
        return None
    text = str(value)
    for fmt in ("%Y-%m-%dT%H:%M:%S.%f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"):
        try:
            return datetime.strptime(text, fmt).replace(tzinfo=timezone.utc)
        except ValueError:
            continue
    try:
        parsed = datetime.fromisoformat(text.replace("Z", "+00:00"))
        return parsed if parsed.tzinfo else parsed.replace(tzinfo=timezone.utc)
    except ValueError:
        return None


def sync_activities(days: int = 90, download_fit: bool = True) -> list[dict[str, Any]]:
    """Fetch activities, enriched with the original FIT where available."""
    client = _client()
    end = date.today()
    start = end - timedelta(days=days)

    raw_activities = client.activities(
        start=0,
        limit=200,
        start_date=start.isoformat(),
        end_date=end.isoformat(),
    )

    out: list[dict[str, Any]] = []
    for raw in raw_activities:
        raw_id = raw.get("activityId")
        if raw_id is None:
            continue
        try:
            detail = client.activity_detail(raw_id)
        except Exception as exc:
            log.debug("Detail fetch failed for %s: %s", raw_id, exc)
            detail = {}

        activity = _canonical_from_detail(detail, raw)
        if not activity.get("external_id"):
            continue

        # Splits and per-second streams come from the original FIT.
        if download_fit:
            try:
                fit_bytes = client.activity_fit(raw_id)
            except Exception as exc:
                log.debug("FIT download failed for %s: %s", raw_id, exc)
                fit_bytes = None
            if fit_bytes:
                try:
                    parsed = parse_fit(fit_bytes, filename=f"{raw_id}.fit")
                except Exception as exc:
                    log.debug("FIT parse failed for %s: %s", raw_id, exc)
                    parsed = {}
                if parsed:
                    activity = _merge_fit(activity, parsed)

        out.append(activity)
    return out


def _merge_fit(activity: dict[str, Any], parsed: dict[str, Any]) -> dict[str, Any]:
    """Overlay the FIT's splits, streams and track onto the API's summary.

    The API's numbers win for distance, pace, heart rate and cadence: the FIT
    stores running cadence in *cycles* (two footfalls), which is half the
    steps-per-minute the watch displays. The FIT wins for everything the
    summary does not carry — kilometre splits, the GPS track and the
    per-second streams.
    """
    merged = dict(activity)

    if parsed.get("laps"):
        merged["laps"] = parsed["laps"]
    if parsed.get("streams"):
        merged["streams"] = parsed["streams"]
    if parsed.get("geo_points"):
        merged["geo_points"] = parsed["geo_points"]
    if parsed.get("device"):
        merged["device"] = parsed["device"]

    # Elevation: the API's summary is the accumulated gain for the activity.
    if not merged.get("elevation_gain_m") and parsed.get("elevation_gain_m"):
        merged["elevation_gain_m"] = parsed["elevation_gain_m"]

    # Local date and times come from the API (the FIT carries UTC).
    if activity.get("start_time"):
        merged["start_time"] = activity["start_time"]
        merged["local_date"] = activity["local_date"]
        merged["day_of_week"] = activity.get("day_of_week")

    return merged


# --------------------------------------------------------------- wellness


def sync_wellness(days: int = 30) -> dict[str, Any]:
    """Pull sleep, HRV, resting HR, stress, body battery and steps per day."""
    client = _client()
    end = date.today()
    start = end - timedelta(days=days)

    db = SessionLocal()
    written = 0
    errors: list[str] = []
    try:
        # ``usersummary-service/usersummary/daily`` carries nearly everything a
        # day needs — resting heart rate, stress, body battery, steps, intensity
        # minutes — so one call per day is the backbone of this pull. Sleep
        # needs its own endpoint for the stage split.
        day = start
        while day <= end:
            key = day.isoformat()
            row = db.get(DailyPhysiology, key)
            if row is None:
                row = DailyPhysiology(date=key, source="garmin_personal")
                db.add(row)

            try:
                _apply_daily_summary(client, row, key)
                _apply_sleep(client, row, key)
                _apply_hrv(client, row, key)
                _apply_heart_rate(client, row, key)
                _mirror_daily_metric(db, row)
                written += 1
            except Exception as exc:
                errors.append(f"{key}: {exc}")
            day += timedelta(days=1)

        db.commit()
    finally:
        db.close()

    return {"days": written, "errors": errors[:10]}


def _apply_sleep(client: GarminPersonalClient, row: DailyPhysiology, key: str) -> None:
    payload = client.sleep(key)
    if not payload:
        return
    row.sleep_s = safe_float(payload.get("sleepTimeSeconds"), row.sleep_s)
    row.deep_sleep_s = safe_float(payload.get("deepSleepSeconds"), row.deep_sleep_s)
    row.rem_s = safe_float(payload.get("remSleepSeconds"), row.rem_s)
    row.light_sleep_s = safe_float(payload.get("lightSleepSeconds"), row.light_sleep_s)
    row.awake_s = safe_float(payload.get("awakeSleepSeconds"), row.awake_s)


def _apply_hrv(client: GarminPersonalClient, row: DailyPhysiology, key: str) -> None:
    payload = client.hrv(key)
    if not payload:
        return
    summary = payload.get("hrvSummary") or payload
    value = safe_float(
        summary.get("lastNightAvg") or summary.get("weeklyAvg") or summary.get("value")
    )
    if value is not None:
        row.hrv_ms = value
    status = summary.get("status") or summary.get("feedbackPhrase")
    if status:
        row.hrv_status = str(status)


def _apply_heart_rate(client: GarminPersonalClient, row: DailyPhysiology, key: str) -> None:
    payload = client.daily_heart_rate(key)
    if not payload:
        return
    summary = payload.get("heartRateSummary") or payload
    resting = safe_float(
        summary.get("restingHeartRate")
        or summary.get("restingHeartRateValue")
        or summary.get("lowestHeartRate")
    )
    if resting is not None:
        row.resting_hr = resting
    peak = safe_float(summary.get("maxHeartRate") or summary.get("highestHeartRate"))
    if peak is not None:
        row.max_hr = peak


def _apply_body_battery(
    client: GarminPersonalClient, row: DailyPhysiology, key: str, start: date, end: date
) -> None:
    # The report endpoint is a range query, so it is fetched once and cached on
    # the client for the duration of the run.
    reports = getattr(client, "_bb_cache", None)
    if reports is None:
        reports = _safe(client.body_battery, start.isoformat(), end.isoformat())
        client._bb_cache = reports
    for report in reports:
        if str(report.get("date") or report.get("calendarDate")) != key:
            continue
        # The report carries the start/end charge plus the full level series.
        levels = [
            safe_float(entry[1])
            for entry in report.get("bodyBatteryValuesArray") or []
            if isinstance(entry, (list, tuple)) and len(entry) > 1
        ]
        levels = [level for level in levels if level is not None]
        if levels:
            row.body_battery_min = round(min(levels), 1)
            row.body_battery_max = round(max(levels), 1)
            row.body_battery_avg = round(sum(levels) / len(levels), 1)
        else:
            row.body_battery_max = safe_float(report.get("charged"), row.body_battery_max)
            row.body_battery_min = safe_float(report.get("drained"), row.body_battery_min)
        break


def _apply_daily_summary(client: GarminPersonalClient, row: DailyPhysiology, key: str) -> None:
    """Fill a day from ``usersummary/daily``.

    One call carries resting heart rate, the seven-day resting average, stress
    levels, the body-battery range, steps, distance and intensity minutes.
    """
    payload = client.daily_summary(key)
    if not payload:
        return
    summary = payload.get("summaryDTO") or payload
    row.steps = safe_float(summary.get("totalSteps"), row.steps)
    row.resting_hr = safe_float(summary.get("restingHeartRate"), row.resting_hr)
    row.max_hr = safe_float(summary.get("maxHeartRate"), row.max_hr)
    row.stress_avg = safe_float(summary.get("averageStressLevel"), row.stress_avg)
    row.stress_max = safe_float(summary.get("maxStressLevel"), row.stress_max)
    row.body_battery_min = safe_float(
        summary.get("bodyBatteryLowestValue"), row.body_battery_min
    )
    row.body_battery_max = safe_float(
        summary.get("bodyBatteryHighestValue"), row.body_battery_max
    )
    row.body_battery_avg = safe_float(
        summary.get("bodyBatteryMostRecentValue"), row.body_battery_avg
    )
    row.vo2max = safe_float(summary.get("vo2Max") or summary.get("vo2MaxValue"), row.vo2max)


def _mirror_daily_metric(db: Session, row: DailyPhysiology) -> None:
    """Mirror into the Phase-1 daily table so /health/daily stays useful."""
    from ..models import DailyMetric

    metric = db.get(DailyMetric, row.date)
    if metric is None:
        metric = DailyMetric(date=row.date)
        db.add(metric)
    metric.resting_hr = row.resting_hr
    metric.max_hr = row.max_hr
    metric.sleep_s = row.sleep_s
    metric.deep_sleep_s = row.deep_sleep_s
    metric.hrv_ms = row.hrv_ms
    metric.stress = row.stress_avg
    metric.steps = row.steps
    metric.body_battery = row.body_battery_avg
    metric.distance_m = 0.0


def _safe(call: Any, *args: Any) -> list[dict[str, Any]]:
    """Call a stats endpoint, returning an empty list instead of raising.

    A window Garmin will not serve must not lose the whole wellness pull.
    """
    try:
        return call(*args) or []
    except Exception as exc:
        log.debug("Stats call %s failed: %s", getattr(call, "__name__", call), exc)
        return []


def _by_day(rows: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    out: dict[str, dict[str, Any]] = {}
    for row in rows:
        key = str(row.get("calendarDate") or row.get("date") or "")[:10]
        if key:
            out[key] = row
    return out


# ------------------------------------------------------------------ entry


def run(days: int = 90) -> dict[str, Any]:
    """Full Personal API sync: activities, then wellness."""
    activities = sync_activities(days=days)
    db = SessionLocal()
    created = updated = 0
    try:
        for activity in activities:
            _, is_new = store_activity(db, activity)
            if is_new:
                created += 1
            else:
                updated += 1
        db.commit()
    finally:
        db.close()

    wellness: dict[str, Any]
    try:
        wellness = sync_wellness(days=min(days, 30))
    except Exception as exc:
        # Health data is a bonus: losing it must not lose the activities.
        log.warning("Wellness pull failed: %s", exc)
        wellness = {"days": 0, "errors": [str(exc)]}

    return {
        "activities_found": len(activities),
        "activities_new": created,
        "activities_updated": updated,
        "wellness_days": wellness.get("days", 0),
        "wellness_errors": wellness.get("errors", []),
    }
