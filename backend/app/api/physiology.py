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

#: The fields of a day that the Body screen draws: today's readiness, sleep,
#: and resting heart rate with HRV.
#:
#: Garmin's own metrics — the body battery, the stress and the acute/chronic
#: load — are not among them, and neither are the training-status columns
#: (``vo2max``, ``endurance_score``, ``running_tolerance``,
#: ``training_readiness``), the step count or the day's maximum heart rate.
#: Nothing draws them, and the plan engine does not read them: it needs the
#: sleep, the HRV and the resting heart rate.
#:
#: The daily endpoint used to answer with every column of every day. A year
#: of that is 189,811 bytes, and almost none of it was drawn. These ten
#: fields are 70,091 bytes for the same year.
#:
#: A caller that names its own fields still gets them, including the ones
#: this list leaves out: the desktop keeps the table, and the harness tool
#: reads the source of a day.
BODY_FIELDS = (
    'date',
    'sleep_s',
    'sleep_score',
    'deep_sleep_s',
    'rem_s',
    'light_sleep_s',
    'awake_s',
    'hrv_ms',
    'hrv_status',
    'resting_hr',
)


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
    """Every stored column of one day."""
    return {column.name: getattr(row, column.name) for column in DailyPhysiology.__table__.columns}


def _body(item: dict[str, Any]) -> dict[str, Any]:
    """Keep only the fields of a day that the Body screen draws.

    The shape of what stays is the shape it had: the same names, the same
    types, the same nulls. Only the columns nothing draws are gone.
    """
    return {key: value for key, value in item.items() if key in BODY_FIELDS}


def _wanted(fields: str | None) -> list[str] | None:
    """The item fields a caller asked for, or None for all of them.

    A year of daily rows is a lot of JSON for a phone to parse and hold: the
    full row carries every Garmin field, most of which no screen draws. The
    caller names the fields it needs and the rest is never sent.
    """
    if not fields:
        return None
    names = [name.strip() for name in fields.split(',') if name.strip()]
    return names or None


def _pick(item: dict[str, Any], wanted: list[str] | None) -> dict[str, Any]:
    """Keep only the requested fields of an item.

    Unknown names are ignored rather than rejected, so a client asking for a
    field this version does not have gets the rest of the row instead of an
    error.
    """
    if wanted is None:
        return item
    return {key: value for key, value in item.items() if key in wanted}


@router.get('/daily')
def daily(
    fields: str | None = Query(None, description='Comma-separated item fields to return'),
    bounds: tuple[str, str] = Depends(date_range),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    wanted = _wanted(fields)
    rows = _rows(db, bounds)
    if wanted is None:
        # No fields asked for: the day as the Body screen draws it.
        return {'items': [_body(_daily(row)) for row in rows]}
    return {'items': [_pick(_daily(row), wanted) for row in rows]}


@router.get('/latest')
def latest(db: Session = Depends(get_db)) -> dict[str, Any]:
    row = db.scalar(select(DailyPhysiology).order_by(DailyPhysiology.date.desc()).limit(1))
    if row is None:
        return {'day': None, 'readiness': None}
    readiness = compute_readiness(db, row.date)
    db.commit()
    return {'day': _body(_daily(row)), 'readiness': readiness}


@router.get('/sleep')
def sleep(bounds: tuple[str, str] = Depends(date_range), db: Session = Depends(get_db)) -> dict[str, Any]:
    fields = ('date', 'sleep_s', 'sleep_score', 'deep_sleep_s', 'rem_s', 'light_sleep_s', 'awake_s')
    return {'items': [{field: getattr(row, field) for field in fields} for row in _rows(db, bounds)]}


@router.get('/hrv')
def hrv(
    fields: str | None = Query(None, description='Comma-separated item fields to return'),
    bounds: tuple[str, str] = Depends(date_range),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    wanted = _wanted(fields)
    rows = _rows(db, bounds, history=True)
    items = []
    latest_baseline = None
    for row in rows:
        if row.date < bounds[0]:
            continue
        baseline = baselines(rows, row.date)
        latest_baseline = baseline['hrv_ms']
        items.append(_pick({'date': row.date, 'hrv_ms': row.hrv_ms, 'status': row.hrv_status, 'baseline': baseline['hrv_ms'], 'resting_hr': row.resting_hr, 'rhr_baseline': baseline['resting_hr']}, wanted))
    return {'items': items, 'baseline': latest_baseline}


@router.get('/readiness')
def readiness(
    fields: str | None = Query(None, description='Comma-separated item fields to return'),
    bounds: tuple[str, str] = Depends(date_range),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    wanted = _wanted(fields)
    items = compute_readiness_range(db, *bounds)
    db.commit()
    # The per-day drivers are what make this endpoint expensive: seven
    # drivers, each with a baseline, a delta and a sentence of prose, for
    # every day of the range. A trend chart needs the score and the grade;
    # only the latest day needs the drivers, and that comes from /latest.
    return {'items': [_pick(item, wanted) for item in items]}


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
