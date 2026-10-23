"""Lap-based running mechanics, persisted without committing the caller's transaction.

Thirds are floor(n/3) chronological laps at each end; fewer than three
laps cannot establish drift. CV uses sample stdev / mean * 100 (n >= 3).
EF deliberately follows the product contract: mean pace in min/km divided
by mean HR / observed max HR. This is NOT speed-based physiological EF: a
slower pace raises this value. Consumers must not reinterpret the formula.
Form = weighted mean of bounded component scores: cadence 30%, decoupling
45%, pace 25%; components are 100*clamp(1-CV/15),
100*clamp(1-max(decoupling,0)/25), 100*clamp(1-pace_CV/25).
Missing components are excluded and weights renormalised; all missing gives
None. CVs 3/5 and decoupling 5 score 80; 8/15 and decoupling 15 score 42.
"""
from __future__ import annotations

import math
import statistics
from typing import Any

from sqlalchemy import func, select
from sqlalchemy.orm import Session, selectinload

from ..config import state
from ..models import Activity, ActivityLap
from ..models_phase2 import RunFingerprint, utcnow

METRICS = {
    'cadence_avg': ('Average cadence', 'spm', 'higher'),
    'cadence_cv': ('Cadence variability', '%', 'lower'),
    'cadence_drift_pct': ('Cadence drift', '%', 'lower'),
    'hr_drift_pct': ('Heart rate drift', '%', 'lower'),
    'decoupling_pct': ('Aerobic decoupling', '%', 'lower'),
    'efficiency_first': ('Efficiency factor', 'm/s per HR fraction', 'higher'),
    'pace_cv': ('Pace variability', '%', 'lower'),
    'stride_length_cm': ('Stride length', 'cm', 'neutral'),
    'form_score': ('Form score', 'points', 'higher'),
}
ZONE_NAMES = ('Recovery', 'Aerobic', 'Tempo', 'Threshold', 'Anaerobic')


def number(value: Any, positive: bool = False) -> bool:
    """Reject provider sentinels and nonfinite values before JSON serialization."""
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value) and (value > 0 if positive else value >= 0)


def _mean(values: list[float]) -> float | None:
    return statistics.mean(values) if values else None


def _cv(values: list[float]) -> float | None:
    return statistics.stdev(values) / statistics.mean(values) * 100 if len(values) >= 3 and statistics.mean(values) > 0 else None


def _change(first: float | None, last: float | None) -> float | None:
    return (last - first) / first * 100 if first is not None and first != 0 and last is not None else None


def _pace(lap: ActivityLap) -> float | None:
    if number(lap.duration_s, True) and number(lap.distance_m, True):
        return lap.duration_s * 1000 / lap.distance_m
    speed = lap.avg_speed_ms
    return 1000 / speed if speed is not None and number(speed, True) else None


def max_hr_reference(db: Session, athlete_id: int | None) -> float:
    """Never mix another athlete's observed maximum into this run."""
    statement = select(func.max(Activity.max_hr)).where(Activity.athlete_id == athlete_id)
    if not state.demo_visible:
        statement = statement.where(Activity.source != 'demo')
    value = db.scalar(statement)
    return float(value) if value is not None and number(value, True) else 190.0


def zone_model(reference: float, stored: Any = None) -> list[dict[str, Any]]:
    """Use cached athlete zones when available; never make network calls on reads."""
    if isinstance(stored, dict):
        stored = stored.get('zones') or stored.get('heartRateZones')
    if isinstance(stored, list):
        zones = []
        for index, row in enumerate(stored):
            if not isinstance(row, dict):
                continue
            low = row.get('low_bpm', row.get('zoneLowBoundary'))
            high = row.get('high_bpm', row.get('zoneHighBoundary'))
            if number(low) and (high is None or number(high, True)):
                zones.append({'zone_no': row.get('zone_no', row.get('zoneNumber', index + 1)), 'name': row.get('name') or f'Zone {index + 1}', 'low_bpm': low, 'high_bpm': high})
        if len(zones) == 5:
            return sorted(zones, key=lambda zone: zone['low_bpm'])
    bounds = (0, .6, .7, .8, .9)
    return [{'zone_no': i + 1, 'name': ZONE_NAMES[i], 'low_bpm': reference * bounds[i], 'high_bpm': reference * bounds[i + 1] if i < 4 else None} for i in range(5)]


def athlete_zones(reference: float) -> list[dict[str, Any]]:
    # TODO(shared): health_sync must persist /biometric-service/heartRateZones
    # into state.garmin_athlete['heartRateZones']; no schema change is needed.
    return zone_model(reference, state.garmin_athlete.get('heartRateZones'))


def _derive(activity: Activity, reference: float) -> dict[str, Any]:
    laps = sorted(activity.laps, key=lambda lap: lap.lap_index)
    third = len(laps) // 3
    first, last = (laps[:third], laps[-third:]) if third else ([], [])

    def values(group: list[ActivityLap], field: str) -> list[float]:
        return [getattr(lap, field) for lap in group if number(getattr(lap, field), True)]

    def paces(group: list[ActivityLap]) -> list[float]:
        return [pace for lap in group if (pace := _pace(lap)) is not None]

    def efficiency(group: list[ActivityLap]) -> float | None:
        """Minetti efficiency factor: speed per unit of heart rate.

        ``EF = speed / (heart rate as a fraction of max)``. Speed, not pace:
        the pace:HR ratio is the reciprocal, and using it here would make a
        *slower* run look more efficient, which inverts every downstream
        reading (higher is better for this metric, and the trend chart relies
        on that).
        """
        paired = [lap for lap in group if _pace(lap) is not None and number(lap.avg_hr, True)]
        pace, hr = _mean(paces(paired)), _mean(values(paired, 'avg_hr'))
        if pace is None or not pace or not hr:
            return None
        speed_m_per_s = 1000.0 / pace
        return speed_m_per_s / (hr / reference)

    cadence = values(laps, 'avg_cadence')
    ef_first, ef_last = efficiency(first), efficiency(last)
    decoupling = -change if (change := _change(ef_first, ef_last)) is not None else None
    pace_laps = laps[:-1] if laps and number(activity.distance_m, True) and number(laps[-1].distance_m) and laps[-1].distance_m < activity.distance_m * .15 else laps
    cadence_cv, pace_cv = _cv(cadence), _cv(paces(pace_laps))
    pace_first, pace_last = _mean(paces(first)), _mean(paces(last))
    components = [(30, cadence_cv, 15), (45, max(0, decoupling) if decoupling is not None else None, 25), (25, pace_cv, 25)]
    present = [(weight, max(0.0, min(1.0, 1 - value / limit))) for weight, value, limit in components if value is not None]
    form = sum(weight * quality for weight, quality in present) / sum(weight for weight, _ in present) * 100 if present else None
    zones: dict[str, float] = {}
    if activity.zones:
        zones = {str(zone.zone_no): float(zone.time_s) for zone in activity.zones if number(zone.time_s)}
    else:
        model = athlete_zones(reference)
        for lap in laps:
            if not number(lap.avg_hr, True) or not number(lap.duration_s, True):
                continue
            for zone in model:
                if lap.avg_hr >= zone['low_bpm'] and (zone['high_bpm'] is None or lap.avg_hr < zone['high_bpm']):
                    key = str(zone['zone_no'])
                    zones[key] = zones.get(key, 0.0) + lap.duration_s
                    break
    hr_avg = activity.avg_hr if number(activity.avg_hr, True) else _mean(values(laps, 'avg_hr'))
    maxima = values(laps, 'max_hr')
    if activity.max_hr is not None and number(activity.max_hr, True):
        maxima.append(activity.max_hr)
    # TODO(shared): demo.py stores centimetre-valued strideLength as metres;
    # normalise upstream before persistence. Reject implausible >3 m here.
    if hr_avg is None:
        samples = [v for stream in activity.streams if stream.kind == 'hr' for v in stream.values if number(v, True)]
        hr_avg = _mean(samples)
    return {
        'cadence_avg': _mean(cadence), 'cadence_cv': cadence_cv,
        'cadence_drift_pct': _change(_mean(values(first, 'avg_cadence')), _mean(values(last, 'avg_cadence'))),
        'hr_avg': hr_avg, 'hr_max': max(maxima) if maxima else None,
        'hr_drift_pct': _change(_mean(values(first, 'avg_hr')), _mean(values(last, 'avg_hr'))),
        'efficiency_first': ef_first, 'efficiency_last': ef_last, 'decoupling_pct': decoupling,
        'pace_cv': pace_cv, 'negative_split_pct': float(100 if pace_last < pace_first else 0) if pace_first is not None and pace_last is not None else None,
        'stride_length_cm': statistics.mean(strides) * 100 if (strides := [value for value in values(laps, 'stride_length_m') if value <= 3]) else None,
        'form_score': form, 'zone_distribution': zones or None,
        'notes': 'Synthetic demo activity.' if activity.source == 'demo' else None,
    }


def _store(db: Session, activity: Activity, reference: float) -> dict[str, Any]:
    payload = _derive(activity, reference)
    row = db.get(RunFingerprint, activity.id)
    if row is None:
        row = RunFingerprint(activity_id=activity.id)
        db.add(row)
    for key, value in payload.items():
        setattr(row, key, value)
    row.computed_at = utcnow()
    return {'activity_id': activity.id, **payload, 'activity_name': activity.name, 'local_date': activity.local_date, 'distance_m': activity.distance_m, 'pace_s_per_km': activity.moving_time_s * 1000 / activity.distance_m if number(activity.distance_m, True) and number(activity.moving_time_s, True) else None, 'source': activity.source, 'is_demo': activity.source == 'demo'}


def compute_fingerprint(db: Session, activity_id: int) -> dict[str, Any]:
    """Upsert one run; missing activities are an explicit error, not empty metrics."""
    activity = db.get(Activity, activity_id)
    if activity is None:
        raise ValueError(f'Activity {activity_id} does not exist')
    result = _store(db, activity, max_hr_reference(db, activity.athlete_id))
    db.flush()
    return result


def compute_many(db: Session, activity_ids: list[int]) -> dict[str, Any]:
    """Batch eager-load inputs and leave transaction ownership to the caller."""
    if not activity_ids:
        return {'items': [], 'computed': 0}
    activities = list(db.scalars(select(Activity).where(Activity.id.in_(activity_ids)).options(selectinload(Activity.laps), selectinload(Activity.zones), selectinload(Activity.streams)).order_by(Activity.start_time, Activity.id)))
    missing = set(activity_ids) - {activity.id for activity in activities}
    if missing:
        raise ValueError(f'Activities do not exist: {sorted(missing)}')
    references = {aid: max_hr_reference(db, aid) for aid in {activity.athlete_id for activity in activities}}
    items = [_store(db, activity, references[activity.athlete_id]) for activity in activities]
    db.flush()
    return {'items': items, 'computed': len(items)}


def fingerprint_trend(rows: list[dict[str, Any]], metric: str) -> dict[str, Any]:
    """Compare chronological halves; a zero baseline has no percentage change."""
    label, unit, better = METRICS[metric]
    points = [{'date': row['local_date'], 'value': row.get(metric), 'activity_id': row['activity_id']} for row in sorted(rows, key=lambda row: (row['local_date'], row['activity_id'])) if row.get(metric) is not None and math.isfinite(row[metric])]
    half = len(points) // 2
    baseline = statistics.mean(point['value'] for point in points[:half]) if half else None
    recent = statistics.mean(point['value'] for point in points[half:]) if half else None
    return {'metric': metric, 'points': points, 'baseline': baseline, 'trend_pct': (recent - baseline) / abs(baseline) * 100 if baseline not in (None, 0) and recent is not None else None, 'label': label, 'unit': unit, 'better': better}
