"""Explainable readiness, with caller-owned transactions.

Baselines use only the preceding 30 calendar days: median sleep duration,
mean HRV and mean resting HR. No personal baseline means that relative
driver is unavailable, rather than inventing a population baseline.
Qualities (clamped to [0,1]): sleep = hours/baseline; sleep score = score/100;
HRV = HRV/baseline; RHR = 1-(RHR-baseline)/(0.2*baseline);
stress = 1-stress/100; battery = battery/100;
load = 1-max(0,acute/chronic-1.3)/1.3.
Weights are 25/10/25/15/10/10/5. Contributions are renormalised to 100
over available drivers. Unknown persists score 0 because the frozen column
is nonnullable, but returns score None so clients cannot mistake it for low.
"""
from __future__ import annotations

import statistics
from datetime import date as calendar_date, timedelta
from typing import Any

from sqlalchemy import select
from sqlalchemy.orm import Session

from ..models_phase2 import DailyPhysiology, Readiness, utcnow
from .fingerprint import number


def baselines(rows: list[DailyPhysiology], day: str) -> dict[str, float | None]:
    """Exclude today's measurements and older observations from the reference."""
    start = (calendar_date.fromisoformat(day) - timedelta(days=30)).isoformat()
    history = [row for row in rows if start <= row.date < day]
    result: dict[str, float | None] = {}
    for field in ('sleep_s', 'hrv_ms', 'resting_hr'):
        values = [float(value) for row in history if (value := getattr(row, field)) is not None and number(value, True)]
        result[field] = (statistics.median(values) if field == 'sleep_s' else statistics.mean(values)) if values else None
    return result


def _hours(value: float) -> str:
    minutes = round(value * 60)
    return f'{minutes // 60}h{minutes % 60:02d}m'


def _round(value: float | None, digits: int = 1) -> float | None:
    """Round for presentation.

    The baselines are means over a trailing window, so they carry the full
    precision of the division — which reads badly in a sentence
    ("57.185185185185185 bpm baseline"). The arithmetic still uses the raw
    values; only what is shown is rounded.
    """
    if value is None:
        return None
    return round(value, digits)


def _derive(day: str, row: DailyPhysiology | None, baseline: dict[str, float | None]) -> dict[str, Any]:
    drivers: list[dict[str, Any]] = []
    weights: list[float] = []

    def add(key: str, label: str, value: float | None, reference: float | None, weight: float, quality: float | None, detail: str, personal: bool = True) -> None:
        """Add one readiness driver.

        ``reference`` is what the driver's quality is measured against. For
        sleep, HRV and resting heart rate that is the athlete's own trailing
        baseline, and the difference from it is worth showing. For the
        scale-based drivers it is a fixed anchor — 100 for a sleep score, 0
        for stress, 1.3 for the acute/chronic ratio — and a "difference from
        the anchor" means nothing. Those pass ``personal=False`` so the
        comparison is not shown.
        """
        if value is None or quality is None or not number(value):
            return
        drivers.append({'key': key, 'label': label, 'value': value, 'baseline': reference if personal else None, 'delta': value - reference if (personal and reference is not None) else None, 'contribution': weight * max(0.0, min(1.0, quality)), 'detail': detail})
        weights.append(weight)

    if row is not None:
        sleep = row.sleep_s / 3600 if number(row.sleep_s) and row.sleep_s is not None else None
        sleep_base = baseline['sleep_s'] / 3600 if baseline['sleep_s'] else None
        add('sleep', 'Sleep', sleep, sleep_base, 25, sleep / sleep_base if sleep is not None and sleep_base else None, f'{_hours(sleep)} vs {_hours(sleep_base)} baseline' if sleep is not None and sleep_base else 'Personal sleep baseline unavailable')
        add('sleep_score', 'Sleep score', row.sleep_score, 100, 10, row.sleep_score / 100 if row.sleep_score is not None else None, 'Garmin sleep score out of 100', personal=False)
        hrv_base, rhr_base = baseline['hrv_ms'], baseline['resting_hr']
        add('hrv', 'HRV', row.hrv_ms, hrv_base, 25, row.hrv_ms / hrv_base if row.hrv_ms is not None and hrv_base else None, f'{_round(row.hrv_ms)} ms vs {_round(hrv_base)} ms baseline')
        add('resting_hr', 'Resting HR', row.resting_hr, rhr_base, 15, 1 - (row.resting_hr - rhr_base) / (.2 * rhr_base) if row.resting_hr is not None and rhr_base else None, f'{_round(row.resting_hr)} bpm vs {_round(rhr_base)} bpm baseline')
        add('stress', 'Stress', row.stress_avg, 0, 10, 1 - row.stress_avg / 100 if row.stress_avg is not None else None, 'Average daily stress; lower is better', personal=False)
        add('body_battery', 'Body battery', row.body_battery_avg, 100, 10, row.body_battery_avg / 100 if row.body_battery_avg is not None else None, 'Average daily body battery out of 100', personal=False)
        ratio = row.acute_load / row.chronic_load if number(row.acute_load) and row.acute_load is not None and number(row.chronic_load, True) and row.chronic_load is not None else None
        add('load', 'Load', ratio, 1.3, 5, 1 - max(0, ratio - 1.3) / 1.3 if ratio is not None else None, 'Acute/chronic load; ratios above 1.3 reduce readiness', personal=False)
    total_weight = sum(weights)
    score = sum(driver['contribution'] for driver in drivers) / total_weight * 100 if total_weight else None
    costs = sorted(zip(drivers, weights), key=lambda pair: pair[1] - pair[0]['contribution'], reverse=True)
    weak = [driver['label'] for driver, weight in costs if weight - driver['contribution'] > .01][:2]
    for driver in drivers:
        driver['contribution'] = round(driver['contribution'] / total_weight * 100, 2)
    grade = 'unknown' if score is None else 'peak' if score >= 85 else 'good' if score >= 70 else 'fair' if score >= 50 else 'low'
    recommendations = {'peak': "Full session as planned — you're primed.", 'good': 'Train as planned.', 'fair': 'Consider trimming volume by ~20 %; keep the session easy.', 'low': "Downgrade today's session — swap quality work for easy running, or rest.", 'unknown': 'Readiness unavailable; use perceived recovery to guide training.'}
    rationale = (' and '.join(weak) + ' account for the largest readiness deductions.') if weak else ('Available drivers are at their reference levels.' if drivers else 'No usable physiology data or personal baselines are available for this day.')
    return {'date': day, 'score': round(score, 2) if score is not None else None, 'grade': grade, 'drivers': drivers, 'recommendation': recommendations[grade], 'rationale': rationale}


def _store(db: Session, payload: dict[str, Any]) -> None:
    row = db.get(Readiness, payload['date'])
    if row is None:
        row = Readiness(date=payload['date'])
        db.add(row)
    for key, value in payload.items():
        setattr(row, key, 0.0 if key == 'score' and value is None else value)
    row.computed_at = utcnow()


def compute_readiness_range(db: Session, start: str, end: str) -> list[dict[str, Any]]:
    """Include missing days explicitly so gaps never look like poor recovery."""
    first, last = calendar_date.fromisoformat(start), calendar_date.fromisoformat(end)
    if first > last:
        raise ValueError('start must not be after end')
    history = list(db.scalars(select(DailyPhysiology).where(DailyPhysiology.date >= (first - timedelta(days=30)).isoformat(), DailyPhysiology.date <= end).order_by(DailyPhysiology.date)))
    by_date = {row.date: row for row in history}
    items = []
    for offset in range((last - first).days + 1):
        day = (first + timedelta(days=offset)).isoformat()
        payload = _derive(day, by_date.get(day), baselines(history, day))
        _store(db, payload)
        items.append(payload)
    db.flush()
    return items


def compute_readiness(db: Session, date: str) -> dict[str, Any]:
    """Recompute rather than serving a stale score after a health sync."""
    return compute_readiness_range(db, date, date)[0]
