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

MAX_ACTIVITY_DAYS = 365
MAX_ACTIVITIES_PER_SYNC = 300
MAX_WELLNESS_DAYS = 90
MAX_HISTORY_MONTHS = 12


def _bounded_days(value: Any, default: int, maximum: int) -> int:
    try:
        parsed = int(value)
    except (TypeError, ValueError, OverflowError):
        parsed = default
    return max(1, min(parsed, maximum))


def _history_activity_days(value: Any, fallback_days: int) -> int:
    """Convert the history API's month count to a maximum one-year day window."""
    if value is None:
        return MAX_ACTIVITY_DAYS
    months = _bounded_days(value, max(1, (fallback_days + 29) // 30), MAX_HISTORY_MONTHS)
    return min(MAX_ACTIVITY_DAYS, months * 31)


def _client() -> GarminPersonalClient:
    return GarminPersonalClient(state.personal_api_key or "")


def _has_fit_records(existing: Activity | None) -> bool:
    """Has this activity already been rebuilt from its original FIT file?

    The Activity API returns a *chart* series — a few hundred points that
    Garmin downsamples for the web page. The FIT file carries the original
    per-second record stream, which is what the segmentation, the fingerprint
    and the drift metrics are computed from.

    The two are easy to confuse and expensive to get wrong: an activity with
    a chart series looks imported, so the FIT is never downloaded, and the
    fingerprint silently never appears for it. So this checks the *resolution*
    of the series, not merely its presence.

    Activities imported after this check was added carry an explicit
    ``fit_imported`` marker; older ones are recognised by their per-second
    resolution.
    """
    if existing is None:
        return False
    raw = existing.raw or {}
    if isinstance(raw, dict) and raw.get("fit_imported"):
        return True
    if not existing.streams:
        return False
    longest = max((len(stream.values or []) for stream in existing.streams), default=0)
    elapsed = existing.elapsed_time_s or 0.0
    if not elapsed:
        return False
    # Per-second resolution: the series is about as long as the activity.
    return longest >= 0.8 * elapsed


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


def sync_activities(
    days: int | None = 90,
    download_fit: bool = True,
    progress: Any = None,
) -> list[dict[str, Any]]:
    """Fetch activities, enriched with the original FIT where available.

    ``days=None`` pulls everything the account holds. The Activity API serves
    the account's whole history — there is no short retention window on it, so
    "load all" is a real option rather than a formality.
    """
    client = _client()
    days = _bounded_days(days, 90, MAX_ACTIVITY_DAYS)
    end = date.today()
    start = end - timedelta(days=days - 1)

    raw_activities: list[dict[str, Any]] = []
    page_size = 100
    offset = 0
    while offset < MAX_ACTIVITIES_PER_SYNC:
        page = client.activities(
            start=offset,
            limit=min(page_size, MAX_ACTIVITIES_PER_SYNC - offset),
            start_date=start.isoformat(),
            end_date=end.isoformat(),
        )
        if not page:
            break
        page = page[: MAX_ACTIVITIES_PER_SYNC - offset]
        raw_activities.extend(page)
        if len(page) < page_size:
            break
        offset += len(page)

    db = SessionLocal()
    out: list[dict[str, Any]] = []
    calls = {"detail": 0, "fit": 0}
    try:
        for raw in raw_activities:
            raw_id = raw.get("activityId")
            if raw_id is None:
                continue

            existing = db.execute(
                select(Activity).where(
                    Activity.source == "garmin_personal",
                    Activity.external_id == str(raw_id),
                )
            ).scalar_one_or_none()
            stored = (existing.raw or {}) if existing else {}

            # The FIT file is the source of truth. If this activity has already
            # been rebuilt from its FIT, its track and streams cannot have
            # changed, so the download is skipped and nothing is fetched for
            # it at all. An activity that only has the API's downsampled chart
            # series has *not* been imported — it gets its FIT downloaded.
            already_imported = _has_fit_records(existing)

            if already_imported and stored.get("summary"):
                detail = {"summaryDTO": stored["summary"]}
            else:
                try:
                    detail = client.activity_detail(raw_id)
                    calls["detail"] += 1
                except Exception as exc:
                    log.debug("Detail fetch failed for %s: %s", raw_id, exc)
                    detail = {}

            activity = _canonical_from_detail(detail, raw)
            if not activity.get("external_id"):
                continue
            if detail.get("summaryDTO"):
                activity.setdefault("raw", {})
                if isinstance(activity["raw"], dict):
                    activity["raw"]["summary"] = detail["summaryDTO"]

            if already_imported:
                # Nothing about the file can have changed, so reuse what we
                # already hold. Without this the canonical `None` would
                # overwrite the stored track and the streams.
                _reuse_stored(existing, activity)

            # Splits and per-second streams come from the original FIT.
            if download_fit and not already_imported:
                try:
                    fit_bytes = client.activity_fit(raw_id)
                    calls["fit"] += 1
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
                        raw_payload = activity.get("raw")
                        if isinstance(raw_payload, dict):
                            raw_payload["fit_imported"] = True

            out.append(activity)
            if progress is not None:
                progress(len(out), len(raw_activities))
    finally:
        db.close()

    log.info(
        "Personal API pull: %d activities, %d detail calls, %d FIT downloads",
        len(out),
        calls["detail"],
        calls["fit"],
    )
    return out


def _reuse_stored(existing: Activity, activity: dict[str, Any]) -> None:
    """Carry the stored track, streams, laps and segment split forward."""
    if existing.geo_points:
        activity["geo_points"] = existing.geo_points
    if existing.streams:
        activity["streams"] = {
            stream.kind: {
                "seconds": list(stream.seconds or []),
                "values": list(stream.values or []),
                "unit": stream.unit,
            }
            for stream in existing.streams
        }
    if existing.laps:
        activity["laps"] = [
            {
                "lap_index": lap.lap_index,
                "distance_m": lap.distance_m,
                "duration_s": lap.duration_s,
                "avg_speed_ms": lap.avg_speed_ms,
                "avg_hr": lap.avg_hr,
                "max_hr": lap.max_hr,
                "avg_cadence": lap.avg_cadence,
                "max_cadence": lap.max_cadence,
                "elevation_gain_m": lap.elevation_gain_m,
                "elevation_loss_m": lap.elevation_loss_m,
                "running_distance_m": lap.running_distance_m,
                "elapsed_s": lap.elapsed_s,
            }
            for lap in existing.laps
        ]
    for key in (
        "running_time_s",
        "walking_time_s",
        "stopped_time_s",
        "running_distance_m",
    ):
        value = getattr(existing, key, None)
        if value is not None:
            activity[key] = value

    # Keep whatever the stored payload knows that the API does not. The
    # ``fit_imported`` marker lives here, and it is the only reliable signal
    # that an activity was rebuilt from its FIT file — a resolution check
    # cannot tell a record stream from a chart series when the watch samples
    # every few seconds. Losing the marker makes the next sync re-download
    # the FIT for an activity that already has it.
    stored_raw = existing.raw or {}
    current_raw = activity.get("raw")
    if isinstance(stored_raw, dict) and isinstance(current_raw, dict):
        for key, value in stored_raw.items():
            current_raw.setdefault(key, value)


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
    # The run / walk / stand split comes from the FIT record stream.
    for key in ("running_time_s", "walking_time_s", "stopped_time_s", "running_distance_m"):
        if parsed.get(key) is not None:
            merged[key] = parsed[key]
    if parsed.get("segments"):
        merged["segments"] = parsed["segments"]
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
    days = _bounded_days(days, 30, MAX_WELLNESS_DAYS)
    end = date.today()
    start = end - timedelta(days=days - 1)

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

            savepoint = db.begin_nested()
            try:
                _apply_daily_summary(client, row, key)
                _apply_sleep(client, row, key)
                _apply_hrv(client, row, key)
                _apply_heart_rate(client, row, key)
                _mirror_daily_metric(db, row)
                written += 1
                savepoint.commit()
            except Exception as exc:
                savepoint.rollback()
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


def run(days: int = 90, history: dict[str, Any] | None = None, progress: Any = None) -> dict[str, Any]:
    """Full Personal API sync: activities, then wellness.

    ``history`` overrides the windows: ``activities`` is a number of months or
    ``None`` for the capped one-year history, ``health_days`` is how much
    wellness history to pull.
    """
    activity_days: int | None = days
    health_days = min(days, 30)
    if history:
        if "activities" in history:
            activity_days = _history_activity_days(history["activities"], days)
        if history.get("health_days"):
            health_days = _bounded_days(history["health_days"], 30, MAX_WELLNESS_DAYS)
    activity_days = _bounded_days(activity_days, 90, MAX_ACTIVITY_DAYS)
    health_days = _bounded_days(health_days, 30, MAX_WELLNESS_DAYS)
    activities = sync_activities(days=activity_days, progress=progress)
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
    except Exception:
        db.rollback()
        raise
    finally:
        db.close()

    wellness: dict[str, Any]
    try:
        wellness = sync_wellness(days=health_days)
    except Exception as exc:
        # Health data is a bonus: losing it must not lose the activities.
        log.warning("Wellness pull failed: %s", exc)
        wellness = {"days": 0, "errors": [str(exc)]}

    # Stamp the sync here, not only in the API wrapper: this function is also
    # called from the production tool, and a sync that does not update the
    # timestamp leaves the app claiming the data is older than it is.
    state.update(last_sync_at=datetime.now(timezone.utc).isoformat())

    return {
        "activities_found": len(activities),
        "activities_new": created,
        "activities_updated": updated,
        "wellness_days": wellness.get("days", 0),
        "wellness_errors": wellness.get("errors", []),
    }


# ------------------------------------------------------------- backfill


def backfill_fit_files(limit: int | None = None, progress: Any = None) -> dict[str, Any]:
    """Re-download the FIT file for activities that only have chart data.

    Garmin's activity *list* only returns about a year of history, but the
    download service will still hand over the original FIT for any activity
    id we hold. That is what makes the backfill possible: an activity
    imported years ago through the web API has a downsampled chart series and
    no per-record stream, and without the FIT it has no fingerprint, no
    segmentation and no drift.

    Nothing here talks to the activity list — it works purely from the ids
    already stored locally.
    """
    from ..ingest.fit_importer import parse_fit
    from ..services.persistence import store_activity

    db = SessionLocal()
    try:
        targets = (
            db.query(Activity)
            .filter(Activity.source == "garmin_personal")
            .order_by(Activity.local_date)
            .all()
        )
        pending = [
            activity
            for activity in targets
            if not _has_fit_records(activity)
        ]
        if limit:
            pending = pending[:limit]

        client = _client()
        rebuilt = skipped = failed = 0
        for position, activity in enumerate(pending, start=1):
            try:
                fit_bytes = client.activity_fit(activity.external_id)
            except Exception as exc:
                failed += 1
                log.debug("FIT download failed for %s: %s", activity.external_id, exc)
                continue
            if not fit_bytes:
                failed += 1
                continue
            try:
                parsed = parse_fit(fit_bytes, filename=f"{activity.external_id}.fit")
            except Exception as exc:
                failed += 1
                log.debug("FIT parse failed for %s: %s", activity.external_id, exc)
                continue
            if not parsed:
                failed += 1
                continue

            # Start from what we already know about the activity and let the
            # FIT contribute the parts the web API never carried.
            data = _canonical_from_stored(activity)
            merged = _merge_fit(data, parsed)
            raw_payload = merged.get("raw")
            if isinstance(raw_payload, dict):
                raw_payload["fit_imported"] = True
            store_activity(db, merged)
            rebuilt += 1
            if progress is not None:
                progress(position, len(pending))
        db.commit()
    finally:
        db.close()

    # Mark the activities that already had their FIT, so the resolution
    # heuristic never has to run again. The heuristic is a fallback for
    # activities imported before the marker existed, and it is not reliable
    # for an older watch that samples every few seconds — the marker is.
    marked = 0
    db = SessionLocal()
    try:
        for activity in db.query(Activity).filter(Activity.source == "garmin_personal").all():
            raw = activity.raw
            if not isinstance(raw, dict) or raw.get("fit_imported"):
                continue
            if not _has_fit_records(activity):
                continue
            # A new dict, not an in-place edit: SQLAlchemy does not track
            # mutations to a JSON column's contents, so mutating the dict in
            # place would be silently discarded on commit.
            activity.raw = {**raw, "fit_imported": True}
            marked += 1
        db.commit()
    finally:
        db.close()

    log.info(
        "FIT backfill: %d rebuilt, %d already current, %d unavailable",
        rebuilt,
        skipped,
        failed,
    )
    return {
        "candidates": len(pending),
        "rebuilt": rebuilt,
        "failed": failed,
    }


def _canonical_from_stored(activity: Activity) -> dict[str, Any]:
    """Rebuild the canonical activity dict from a stored row."""
    data = blank_activity()
    data.update(
        {
            "source": activity.source,
            "external_id": activity.external_id,
            "name": activity.name,
            "sport_type": activity.sport_type,
            "description": activity.description,
            "start_time": activity.start_time,
            "local_date": activity.local_date,
            "day_of_week": activity.day_of_week,
            "distance_m": activity.distance_m,
            "moving_time_s": activity.moving_time_s,
            "elapsed_time_s": activity.elapsed_time_s,
            "elevation_gain_m": activity.elevation_gain_m,
            "avg_hr": activity.avg_hr,
            "max_hr": activity.max_hr,
            "calories": activity.calories,
            "avg_cadence": activity.avg_cadence,
            "max_cadence": activity.max_cadence,
            "avg_speed_ms": activity.avg_speed_ms,
            "max_speed_ms": activity.max_speed_ms,
            "perceived_exertion": activity.perceived_exertion,
            "device": activity.device,
            "gear": activity.gear,
            "is_trail": bool(activity.is_trail),
            "is_commute": bool(activity.is_commute),
            "geo_points": activity.geo_points,
            "raw": dict(activity.raw or {}),
        }
    )
    return data
