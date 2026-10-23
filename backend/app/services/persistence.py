"""Persistence: turn canonical activity dicts into DB rows.

Idempotent by construction — ``(source, external_id)`` is unique, so re-syncing
an activity updates it in place instead of duplicating it. That matters because
Garmin lets you rename an activity, add a description, or re-upload it, and the
sync runs repeatedly.
"""

from __future__ import annotations

import logging
from datetime import datetime
from typing import Any

from sqlalchemy import select
from sqlalchemy.orm import Session

from ..models import Activity, ActivityLap, ActivityStream, HrZone

log = logging.getLogger(__name__)

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
    "visibility",
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


def bootstrap_demo(db: Session) -> int:
    """Seed the demo dataset if the activities table is empty."""
    from ..ingest.demo import activities_from_cache, load_cache

    if db.query(Activity).count() > 0:
        return 0
    cache = load_cache()
    if not cache:
        return 0
    activities = activities_from_cache(cache)
    for activity in activities:
        store_activity(db, activity)
    db.commit()
    log.info("Seeded %d demo activities", len(activities))
    return len(activities)
