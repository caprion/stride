"""Running mechanics, chronological trends and shoe mileage.

The exported router bundles a prefixed fitness router and an unprefixed gear
router so mounting it at /api/v1 also exposes /api/v1/gear.
"""
from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import select
from sqlalchemy.orm import Session

from ..db import get_db
from ..models import Activity
from ..models_phase2 import Gear
from ..services.analytics import RUN_SPORTS, demo_filter
from ..services.fingerprint import (
    METRICS, athlete_zones, compute_fingerprint, compute_many,
    fingerprint_trend, max_hr_reference,
)
from .physiology import date_range

router = APIRouter()
fitness_router = APIRouter(prefix='/fitness', tags=['fitness'])
gear_router = APIRouter(tags=['gear'])


def _items(db: Session, bounds: tuple[str, str]) -> list[dict[str, Any]]:
    """Recompute from current laps so subsequent imports cannot stale the picture."""
    ids = list(db.scalars(demo_filter(select(Activity.id).where(
        Activity.sport_type.in_(RUN_SPORTS),
        Activity.local_date >= bounds[0], Activity.local_date <= bounds[1],
    )).order_by(Activity.local_date, Activity.id)))
    items = compute_many(db, ids)['items']
    db.commit()
    return items


def _zone_shares(db: Session, item: dict[str, Any]) -> list[dict[str, Any]]:
    activity = db.get(Activity, item['activity_id'])
    if activity is None:
        raise HTTPException(status_code=404, detail='Activity not found')
    model = athlete_zones(max_hr_reference(db, activity.athlete_id))
    stored = {zone.zone_no: zone for zone in activity.zones}
    distribution = item['zone_distribution'] or {}
    total = sum(distribution.values())
    shares = []
    for zone_no in sorted({int(key) for key in distribution} | {zone['zone_no'] for zone in model}):
        metadata = next((zone for zone in model if zone['zone_no'] == zone_no),
                        {'zone_no': zone_no, 'name': f'Zone {zone_no}', 'low_bpm': None, 'high_bpm': None})
        if zone_no in stored:
            row = stored[zone_no]
            metadata = {'zone_no': zone_no, 'name': row.name, 'low_bpm': row.low_bpm, 'high_bpm': row.high_bpm}
        seconds = distribution.get(str(zone_no), 0.0)
        shares.append({**metadata, 'time_s': seconds, 'percent': seconds / total * 100 if total else 0.0})
    return shares


@fitness_router.get('/fingerprint')
def fingerprints(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    return {'items': _items(db, bounds)}


@fitness_router.get('/activity/{id}')
def activity_fingerprint(id: int, db: Session = Depends(get_db)) -> dict[str, Any]:
    activity = db.scalar(demo_filter(select(Activity).where(Activity.id == id)))
    if activity is None:
        raise HTTPException(status_code=404, detail='Activity not found')
    item = compute_fingerprint(db, id)
    zones = _zone_shares(db, item)
    db.commit()
    return {**item, 'zones': zones}


@fitness_router.get('/trends')
def trends(metric: str = Query('decoupling_pct'), bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    if metric not in METRICS:
        raise HTTPException(status_code=422, detail=f'Unsupported metric; choose from {list(METRICS)}')
    return fingerprint_trend(_items(db, bounds), metric)


@fitness_router.get('/zones')
def zones(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    totals: dict[int, dict[str, Any]] = {}
    for item in _items(db, bounds):
        for zone in _zone_shares(db, item):
            key = zone['zone_no']
            if key not in totals:
                totals[key] = {**zone, 'time_s': 0.0}
            elif any(totals[key][field] != zone[field] for field in ('low_bpm', 'high_bpm')):
                totals[key]['low_bpm'] = totals[key]['high_bpm'] = None
            totals[key]['time_s'] += zone['time_s']
    total = sum(zone['time_s'] for zone in totals.values())
    for zone in totals.values():
        zone['percent'] = zone['time_s'] / total * 100 if total else 0.0
    return {'zones': [totals[key] for key in sorted(totals)], 'total_time_s': total}


@gear_router.get('/gear')
def gear(db: Session = Depends(get_db)) -> dict[str, Any]:
    items = []
    for row in db.scalars(select(Gear).order_by(Gear.id)):
        item = {column.name: getattr(row, column.name) for column in Gear.__table__.columns}
        item['retired'] = row.distance_m >= row.retire_at_m
        items.append(item)
    return {'items': items}


router.include_router(fitness_router)
router.include_router(gear_router)
