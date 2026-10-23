"""Activity listing and detail endpoints."""

from __future__ import annotations

from datetime import date, datetime, timedelta
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import func, or_, select
from sqlalchemy.orm import Session, selectinload

from ..db import get_db
from ..models import Activity, ActivityStream
from ..services.analytics import (
    RUN_SPORTS,
    _duration_label,
    _pace_label as pace_label,
    _pace_s_per_km,
    demo_filter,
)

router = APIRouter(prefix="/activities", tags=["activities"])


def _summary(activity: Activity) -> dict[str, Any]:
    pace = _pace_s_per_km(activity.distance_m, activity.moving_time_s)
    return {
        "id": activity.id,
        "source": activity.source,
        "external_id": activity.external_id,
        "name": activity.name,
        "sport_type": activity.sport_type,
        "description": activity.description,
        "start_time": activity.start_time.isoformat() if activity.start_time else None,
        "local_date": activity.local_date,
        "day_of_week": activity.day_of_week,
        "distance_m": activity.distance_m,
        "moving_time_s": activity.moving_time_s,
        "moving_time_label": _duration_label(activity.moving_time_s),
        "elapsed_time_s": activity.elapsed_time_s,
        "elevation_gain_m": activity.elevation_gain_m,
        "avg_hr": activity.avg_hr,
        "max_hr": activity.max_hr,
        "calories": activity.calories,
        "avg_cadence": activity.avg_cadence,
        "max_cadence": activity.max_cadence,
        "avg_speed_ms": activity.avg_speed_ms,
        "max_speed_ms": activity.max_speed_ms,
        "pace_s_per_km": round(pace, 2) if pace else None,
        "pace_label": pace_label(pace),
        "perceived_exertion": activity.perceived_exertion,
        "device": activity.device,
        "gear": activity.gear,
        "is_trail": activity.is_trail,
        "is_commute": activity.is_commute,
        "visibility": activity.visibility,
        "has_geo": bool(activity.geo_points),
        "route_preview": _route_preview(activity.geo_points),
        "lap_count": len(activity.laps or []),
        "running_time_s": activity.running_time_s,
        "walking_time_s": activity.walking_time_s,
        "stopped_time_s": activity.stopped_time_s,
        "running_distance_m": activity.running_distance_m,
    }


def _route_preview(points: list | None, limit: int = 90) -> list[list[float]] | None:
    """A decimated version of the track, for feed thumbnails.

    Full tracks run to thousands of points; a feed page shows dozens of cards,
    so we sample the track down to a fixed budget before shipping it.
    """
    if not points or len(points) < 2:
        return None
    if len(points) <= limit:
        return [[float(p[0]), float(p[1])] for p in points]
    step = len(points) / limit
    sampled = [points[min(int(i * step), len(points) - 1)] for i in range(limit)]
    sampled.append(points[-1])
    return [[float(p[0]), float(p[1])] for p in sampled]


@router.get("")
def list_activities(
    db: Session = Depends(get_db),
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=200),
    sport: str | None = None,
    date_from: str | None = Query(None, alias="from"),
    date_to: str | None = Query(None, alias="to"),
    q: str | None = None,
    sort: str = "date",
) -> dict[str, Any]:
    stmt = select(Activity).options(selectinload(Activity.laps))

    if sport:
        stmt = stmt.where(Activity.sport_type == sport)
    else:
        stmt = stmt.where(Activity.sport_type.in_(RUN_SPORTS))
    stmt = demo_filter(stmt)
    if date_from:
        stmt = stmt.where(Activity.local_date >= date_from)
    if date_to:
        stmt = stmt.where(Activity.local_date <= date_to)
    if q:
        pattern = f"%{q.strip()}%"
        stmt = stmt.where(
            or_(
                Activity.name.ilike(pattern),
                Activity.description.ilike(pattern),
                Activity.local_date.like(pattern),
            )
        )

    total = db.execute(
        select(func.count()).select_from(stmt.subquery())
    ).scalar_one()

    order = Activity.start_time.desc()
    if sort == "distance":
        order = Activity.distance_m.desc()
    elif sort == "pace":
        order = Activity.moving_time_s.asc()
    elif sort == "elevation":
        order = Activity.elevation_gain_m.desc()

    rows = (
        db.execute(stmt.order_by(order).offset((page - 1) * per_page).limit(per_page))
        .scalars()
        .all()
    )
    return {
        "items": [_summary(a) for a in rows],
        "total": total,
        "page": page,
        "per_page": per_page,
        "pages": max(1, (total + per_page - 1) // per_page),
    }


@router.get("/{activity_id}")
def get_activity(activity_id: int, db: Session = Depends(get_db)) -> dict[str, Any]:
    activity = db.get(Activity, activity_id)
    if activity is None:
        raise HTTPException(status_code=404, detail="Activity not found.")

    payload = _summary(activity)
    payload["laps"] = [
        {
            "lap_index": lap.lap_index,
            "distance_m": lap.distance_m,
            "distance_label": f"{lap.distance_m / 1000:.2f} km",
            "duration_s": lap.duration_s,
            "duration_label": _duration_label(lap.duration_s),
            "avg_speed_ms": lap.avg_speed_ms,
            "pace_s_per_km": round(1000.0 / lap.avg_speed_ms, 2) if lap.avg_speed_ms else None,
            "pace_label": pace_label(1000.0 / lap.avg_speed_ms) if lap.avg_speed_ms else None,
            "avg_hr": lap.avg_hr,
            "max_hr": lap.max_hr,
            "avg_cadence": lap.avg_cadence,
            "max_cadence": lap.max_cadence,
            "stride_length_m": lap.stride_length_m,
            "elevation_gain_m": lap.elevation_gain_m,
            "elevation_loss_m": lap.elevation_loss_m,
            "start_elevation_m": lap.start_elevation_m,
            "end_elevation_m": lap.end_elevation_m,
            "running_distance_m": lap.running_distance_m,
            "elapsed_s": lap.elapsed_s,
        }
        for lap in activity.laps
    ]
    payload["streams"] = {
        stream.kind: {
            "seconds": stream.seconds or [],
            "values": stream.values or [],
            "unit": stream.unit,
        }
        for stream in activity.streams
    }
    payload["hr_zones"] = [
        {
            "zone_no": zone.zone_no,
            "name": zone.name,
            "low_bpm": zone.low_bpm,
            "high_bpm": zone.high_bpm,
            "time_s": zone.time_s,
            "time_label": _duration_label(zone.time_s),
        }
        for zone in activity.zones
    ]
    payload["geo_points"] = activity.geo_points or []
    return payload


@router.get("/{activity_id}/geo")
def activity_geo(activity_id: int, db: Session = Depends(get_db)) -> dict[str, Any]:
    activity = db.get(Activity, activity_id)
    if activity is None:
        raise HTTPException(status_code=404, detail="Activity not found.")
    points = activity.geo_points or []
    bbox = None
    if points:
        lats = [p[0] for p in points]
        lons = [p[1] for p in points]
        bbox = [min(lons), min(lats), max(lons), max(lats)]
    return {"points": points, "bbox": bbox, "has_geo": bool(points)}


@router.get("/{activity_id}/streams/{kind}")
def activity_stream(activity_id: int, kind: str, db: Session = Depends(get_db)) -> dict[str, Any]:
    row = db.execute(
        select(ActivityStream).where(
            ActivityStream.activity_id == activity_id, ActivityStream.kind == kind
        )
    ).scalar_one_or_none()
    if row is None:
        raise HTTPException(status_code=404, detail="Stream not found.")
    return {
        "kind": row.kind,
        "seconds": row.seconds or [],
        "values": row.values or [],
        "unit": row.unit,
    }


class ActivityPatch(BaseModel):
    name: str | None = None
    description: str | None = None
    perceived_exertion: int | None = None
    visibility: str | None = None
    is_trail: bool | None = None
    is_commute: bool | None = None


@router.patch("/{activity_id}")
def patch_activity(activity_id: int, body: ActivityPatch, db: Session = Depends(get_db)) -> dict[str, Any]:
    activity = db.get(Activity, activity_id)
    if activity is None:
        raise HTTPException(status_code=404, detail="Activity not found.")
    for field, value in body.model_dump(exclude_unset=True).items():
        if value is not None:
            setattr(activity, field, value)
    db.commit()
    return _summary(activity)
