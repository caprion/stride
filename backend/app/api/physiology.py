"""Daily physiology and explainable recovery, including honest missing-data states."""
from __future__ import annotations

from datetime import date, timedelta
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import select
from sqlalchemy.orm import Session

from ..config import state
from ..db import get_db
from ..models_phase2 import DailyPhysiology
from ..services.readiness import baselines, compute_readiness, compute_readiness_range

router = APIRouter(prefix='/physiology', tags=['physiology'])


def date_range(
    start: date | None = Query(None, alias='from'),
    end: date | None = Query(None, alias='to'),
) -> tuple[str, str]:
    """Validate calendar dates before querying lexicographically stored ISO dates.

    The default window is a year: Garmin keeps roughly a year of wellness
    history, and the readiness baseline needs a trailing 30 days plus room to
    see the trend.
    """
    last = end or date.today()
    first = start or last - timedelta(days=364)
    if first > last:
        raise HTTPException(status_code=422, detail='from must not be after to')
    return first.isoformat(), last.isoformat()


def _rows(db: Session, bounds: tuple[str, str], history: bool = False) -> list[DailyPhysiology]:
    start, end = bounds
    if history:
        start = (date.fromisoformat(start) - timedelta(days=30)).isoformat()
    return list(db.scalars(select(DailyPhysiology).where(DailyPhysiology.date >= start, DailyPhysiology.date <= end).order_by(DailyPhysiology.date)))


def _daily(row: DailyPhysiology) -> dict[str, Any]:
    return {column.name: getattr(row, column.name) for column in DailyPhysiology.__table__.columns}


@router.get('/daily')
def daily(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    return {'items': [_daily(row) for row in _rows(db, bounds)]}


@router.get('/latest')
def latest(db: Session = Depends(get_db)) -> dict[str, Any]:
    row = db.scalar(select(DailyPhysiology).order_by(DailyPhysiology.date.desc()).limit(1))
    if row is None:
        return {'day': None, 'readiness': None}
    readiness = compute_readiness(db, row.date)
    db.commit()
    return {'day': _daily(row), 'readiness': readiness}


@router.get('/sleep')
def sleep(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    fields = ('date', 'sleep_s', 'sleep_score', 'deep_sleep_s', 'rem_s', 'light_sleep_s', 'awake_s')
    return {'items': [{field: getattr(row, field) for field in fields} for row in _rows(db, bounds)]}


@router.get('/hrv')
def hrv(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    rows = _rows(db, bounds, history=True)
    items = []
    for row in rows:
        if row.date < bounds[0]:
            continue
        baseline = baselines(rows, row.date)
        items.append({'date': row.date, 'hrv_ms': row.hrv_ms, 'status': row.hrv_status, 'baseline': baseline['hrv_ms'], 'resting_hr': row.resting_hr, 'rhr_baseline': baseline['resting_hr']})
    return {'items': items, 'baseline': items[-1]['baseline'] if items else None}


@router.get('/readiness')
def readiness(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    items = compute_readiness_range(db, *bounds)
    db.commit()
    return {'items': items}


@router.get('/sources')
def sources(db: Session = Depends(get_db)) -> dict[str, Any]:
    """Stored measurements prove availability; connection capability alone does not."""
    families = {
        'sleep': ('sleep_s', 'sleep_score', 'deep_sleep_s', 'rem_s', 'light_sleep_s', 'awake_s'),
        'hrv': ('hrv_ms', 'hrv_status'),
        'resting_hr': ('resting_hr',),
        'stress': ('stress_avg', 'stress_max'),
        'body_battery': ('body_battery_min', 'body_battery_avg', 'body_battery_max'),
        'training_status': ('training_readiness', 'acute_load', 'chronic_load', 'vo2max', 'endurance_score', 'running_tolerance'),
    }
    fields = tuple(column.name for column in DailyPhysiology.__table__.columns if column.name not in ('date', 'source', 'updated_at'))
    rows = list(db.scalars(select(DailyPhysiology)))
    populated = [row for row in rows if any(getattr(row, field) is not None for field in fields)]
    reason = None
    if state.garmin_method == 'developer_key':
        missing_note = 'The Activity API developer key does not grant Health API access; connect a Garmin account.'
    elif not state.garmin_connected:
        missing_note = 'Connect a Garmin account and sync health data.'
    else:
        missing_note = 'No measurements stored for this family; sync health data and check device support.'
    if not populated:
        reason = missing_note
    endpoints = []
    for name, columns in families.items():
        available = any(getattr(row, column) is not None for row in rows for column in columns)
        endpoints.append({'name': name, 'available': available, 'note': 'Stored measurements available.' if available else missing_note})
    updated = [row.updated_at for row in populated if row.updated_at is not None]
    return {'available': bool(populated), 'reason': reason, 'auth_method': state.garmin_method, 'endpoints': endpoints, 'last_synced_at': max(updated).isoformat() if updated else None, 'days_available': len(populated)}
