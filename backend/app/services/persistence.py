"""Persistence: turn canonical activity dicts into DB rows.

Idempotent by construction — ``(source, external_id)`` is unique, so re-syncing
an activity updates it in place instead of duplicating it. That matters because
Garmin lets you rename an activity, add a description, or re-upload it, and the
sync runs repeatedly.
"""

from __future__ import annotations

import logging
from datetime import datetime, timedelta
from typing import Any

from sqlalchemy import select
from sqlalchemy.orm import Session

from ..models import Activity, ActivityLap, ActivityStream, HrZone
from ..models_phase2 import DailyPhysiology

log = logging.getLogger(__name__)

# The columns that hold a day's measurements. ``date``, ``source`` and
# ``updated_at`` are bookkeeping; every other column is a measurement.
_MEASUREMENT_COLUMNS = tuple(
    column.name
    for column in DailyPhysiology.__table__.columns
    if column.name not in ("date", "source", "updated_at")
)


def physiology_day_is_held(row: DailyPhysiology | None) -> bool:
    """Do we already hold this day's measurements?

    The rule is the one the activity pull uses: **do not fetch what you
    already hold.** A row that carries a measurement is the marker — the day
    was pulled and the pull ran to the end, because a day that fails is
    rolled back and leaves nothing behind.

    A row with no measurement in it is *not* held: that is either a day
    Garmin has nothing for, or a day whose pull failed. Either way there is
    nothing to reuse, so it is fetched again.
    """
    if row is None:
        return False
    return any(getattr(row, name, None) is not None for name in _MEASUREMENT_COLUMNS)

_ACTIVITY_FIELDS = (
    "name",
    "sport_type",
    "description",
    "start_time",
    "local_date",
    "day_of_week",
    "distance_m",
    "moving_time_s",
    "elapsed_time_s",
    "elevation_gain_m",
    "avg_hr",
    "max_hr",
    "calories",
    "avg_cadence",
    "max_cadence",
    "avg_speed_ms",
    "max_speed_ms",
    "running_time_s",
    "walking_time_s",
    "stopped_time_s",
    "running_distance_m",
    "perceived_exertion",
    "device",
    "gear",
    "is_trail",
    "is_commute",
    "polyline",
    "geo_points",
    "raw",
)


def _parse_dt(value: Any) -> datetime | None:
    from ..ingest.normalize import parse_iso

    return parse_iso(value)


def upsert_activity(db: Session, data: dict[str, Any]) -> tuple[Activity, bool]:
    """Insert or update one activity. Returns ``(activity, created)``."""
    source = data.get("source") or "garmin"
    external_id = str(data.get("external_id"))
    existing = db.execute(
        select(Activity).where(
            Activity.source == source, Activity.external_id == external_id
        )
    ).scalar_one_or_none()

    created = existing is None
    if existing is None:
        existing = Activity(source=source, external_id=external_id)
        db.add(existing)

    for field in _ACTIVITY_FIELDS:
        if field not in data:
            continue
        value = data[field]
        if field == "start_time":
            parsed = _parse_dt(value)
            if parsed:
                existing.start_time = parsed
            continue
        setattr(existing, field, value)

    if not existing.local_date and existing.start_time:
        existing.local_date = existing.start_time.date().isoformat()
    if not existing.day_of_week and existing.local_date:
        from ..ingest.normalize import day_of_week_for

        existing.day_of_week = day_of_week_for(existing.local_date)

    db.flush()
    return existing, created


def replace_laps(db: Session, activity: Activity, laps: list[dict[str, Any]]) -> None:
    if not laps:
        return
    db.query(ActivityLap).filter(ActivityLap.activity_id == activity.id).delete()
    for lap in laps:
        db.add(
            ActivityLap(
                activity_id=activity.id,
                lap_index=int(lap.get("lap_index") or 1),
                distance_m=float(lap.get("distance_m") or 0.0),
                duration_s=float(lap.get("duration_s") or 0.0),
                avg_speed_ms=lap.get("avg_speed_ms"),
                max_speed_ms=lap.get("max_speed_ms"),
                avg_hr=lap.get("avg_hr"),
                max_hr=lap.get("max_hr"),
                avg_cadence=lap.get("avg_cadence"),
                max_cadence=lap.get("max_cadence"),
                stride_length_m=lap.get("stride_length_m"),
                elevation_gain_m=lap.get("elevation_gain_m"),
                elevation_loss_m=lap.get("elevation_loss_m"),
                start_elevation_m=lap.get("start_elevation_m"),
                end_elevation_m=lap.get("end_elevation_m"),
                running_distance_m=lap.get("running_distance_m"),
                elapsed_s=lap.get("elapsed_s"),
            )
        )


def replace_streams(db: Session, activity: Activity, streams: dict[str, Any]) -> None:
    if not streams:
        return
    for kind, payload in streams.items():
        if not isinstance(payload, dict):
            continue
        seconds = payload.get("seconds") or []
        values = payload.get("values") or []
        if not values:
            continue
        row = db.execute(
            select(ActivityStream).where(
                ActivityStream.activity_id == activity.id, ActivityStream.kind == kind
            )
        ).scalar_one_or_none()
        if row is None:
            row = ActivityStream(activity_id=activity.id, kind=kind)
            db.add(row)
        row.seconds = list(seconds)
        row.values = list(values)
        row.unit = payload.get("unit")
        row.label = payload.get("label")


def replace_hr_zones(db: Session, activity: Activity, zones: list[dict[str, Any]]) -> None:
    if not zones:
        return
    db.query(HrZone).filter(HrZone.activity_id == activity.id).delete()
    for zone in zones:
        db.add(
            HrZone(
                activity_id=activity.id,
                zone_no=int(zone.get("zone_no") or 1),
                name=str(zone.get("name") or f"Zone {zone.get('zone_no')}"),
                low_bpm=zone.get("low_bpm"),
                high_bpm=zone.get("high_bpm"),
                time_s=float(zone.get("time_s") or 0.0),
            )
        )


def store_activity(
    db: Session,
    data: dict[str, Any],
    laps: list[dict[str, Any]] | None = None,
    streams: dict[str, Any] | None = None,
    zones: list[dict[str, Any]] | None = None,
) -> tuple[Activity, bool]:
    """Full upsert including child rows."""
    activity, created = upsert_activity(db, data)
    replace_laps(db, activity, laps if laps is not None else data.get("laps") or [])
    replace_streams(db, activity, streams if streams is not None else data.get("streams") or {})
    replace_hr_zones(db, activity, zones if zones is not None else data.get("hr_zones") or [])
    db.flush()
    return activity, created


# How close two runs have to be to be the same run: the start time within
# two minutes, and the distance within two per cent.
DUPLICATE_WINDOW_S = 120
DUPLICATE_DISTANCE_RATIO = 0.02


def find_duplicate(db: Session, data: dict[str, Any]) -> Activity | None:
    """The activity already in the store that is the same run.

    ``(source, external_id)`` makes the write idempotent, so re-importing
    the same file updates the run it wrote the first time. It does not stop
    the *same run* arriving twice under two identities: a run synced from
    Garmin and the same run exported as a file have different external ids,
    and the file's id is the watch's, not Garmin's.

    So the run itself is the key: two rows are the same run when they start
    within two minutes of each other and their distances are within two per
    cent. Anything looser would merge two runs of the same evening.
    """
    start = data.get("start_time")
    distance = data.get("distance_m")
    if not isinstance(start, datetime) or not distance:
        return None
    window = timedelta(seconds=DUPLICATE_WINDOW_S)
    rows = db.execute(
        select(Activity).where(
            Activity.start_time >= start - window,
            Activity.start_time <= start + window,
        )
    ).scalars()
    for row in rows:
        if not row.distance_m:
            continue
        if abs(row.distance_m - distance) <= distance * DUPLICATE_DISTANCE_RATIO:
            return row
    return None


def purge_synthetic_activities(db: Session) -> int:
    """Delete the synthetic activities an older installation may still hold.

    The sample dataset is gone — it is not seeded, not filtered and not
    behind a flag. An installation that was seeded before it was removed
    still has those rows in its database, and without this they would sit in
    the athlete's feed looking like their own runs. They are removed once, at
    startup, and never come back.

    The children go with them: the laps, the streams, the heart-rate zones
    and the fingerprint are all rows keyed on the activity.
    """
    from ..models_phase2 import RunFingerprint

    ids = [
        row[0]
        for row in db.query(Activity.id).filter(Activity.source == "demo").all()
    ]
    if not ids:
        return 0
    for model in (ActivityLap, ActivityStream, HrZone, RunFingerprint):
        db.query(model).filter(model.activity_id.in_(ids)).delete(
            synchronize_session=False
        )
    removed = (
        db.query(Activity)
        .filter(Activity.id.in_(ids))
        .delete(synchronize_session=False)
    )
    db.commit()
    log.info("Removed %d synthetic activities left over from an older install", removed)
    return removed
