"""Analytics: the numbers behind the statistics pages.

Everything is computed on read from the ``activity`` table — no materialised
aggregates to keep in sync, no stale numbers after a re-import. At personal scale
(thousands of activities) this is comfortably fast.
"""

from __future__ import annotations

import statistics
from collections import defaultdict
from datetime import date, datetime, timedelta
from typing import Any, Iterable

from sqlalchemy import func, select
from sqlalchemy.orm import Session

from ..ingest.normalize import week_key
from ..models import Activity, ActivityLap

RUN_SPORTS = ("running", "trail_running", "treadmill_running", "virtualrun", "street_running", "track_running")


def _runs_query(db: Session, sport: str | None = None):
    stmt = select(Activity)
    if sport:
        stmt = stmt.where(Activity.sport_type == sport)
    else:
        stmt = stmt.where(Activity.sport_type.in_(RUN_SPORTS))
    return stmt.order_by(Activity.start_time.desc())

def _pace_s_per_km(distance_m: float, moving_time_s: float) -> float | None:
    if not distance_m or distance_m <= 0 or not moving_time_s or moving_time_s <= 0:
        return None
    return moving_time_s / (distance_m / 1000.0)


def _pace_label(seconds: float | None) -> str | None:
    if not seconds or seconds <= 0:
        return None
    total = int(round(seconds))
    return f"{total // 60}:{total % 60:02d}"


def _duration_label(seconds: float | None) -> str | None:
    if not seconds:
        return None
    total = int(round(seconds))
    hours, remainder = divmod(total, 3600)
    minutes, secs = divmod(remainder, 60)
    if hours:
        return f"{hours}:{minutes:02d}:{secs:02d}"
    return f"{minutes}:{secs:02d}"


def _bucket(activities: list[Activity], label: str) -> dict[str, Any]:
    distance = sum(a.distance_m for a in activities)
    elevation = sum(a.elevation_gain_m for a in activities)
    moving = sum(a.moving_time_s for a in activities)
    paces = [
        p
        for p in (_pace_s_per_km(a.distance_m, a.moving_time_s) for a in activities)
        if p
    ]
    hrs = [a.avg_hr for a in activities if a.avg_hr]
    return {
        "label": label,
        "activity_count": len(activities),
        "distance_m": round(distance, 1),
        "elevation_gain_m": round(elevation, 1),
        "moving_time_s": round(moving, 1),
        "moving_time_label": _duration_label(moving),
        "avg_pace_s_per_km": round(statistics.mean(paces), 2) if paces else None,
        "avg_pace_label": _pace_label(statistics.mean(paces)) if paces else None,
        "avg_hr": round(statistics.mean(hrs), 1) if hrs else None,
        "calories": round(sum(a.calories for a in activities if a.calories)) if any(a.calories for a in activities) else None,
    }


def summary(db: Session) -> dict[str, Any]:
    """Totals & gains — all time, this year, this month, this week."""
    activities = list(db.execute(_runs_query(db)).scalars())
    today = date.today()
    year_start = today.replace(month=1, day=1)
    month_start = today.replace(day=1)
    week_start = today - timedelta(days=today.weekday())

    def within(start: date) -> list[Activity]:
        return [a for a in activities if a.start_time and a.start_time.date() >= start]

    all_time = _bucket(activities, "All time")
    this_year = _bucket(within(year_start), str(year_start.year))
    this_month = _bucket(within(month_start), today.strftime("%B %Y"))
    this_week = _bucket(within(week_start), "This week")

    return {
        "totals": all_time,
        "this_year": this_year,
        "this_month": this_month,
        "this_week": this_week,
        "personal_records": personal_records(db),
        "streaks": streaks(db),
        "sport_breakdown": sport_breakdown(db),
    }


def sport_breakdown(db: Session) -> list[dict[str, Any]]:
    rows = db.execute(
        select(
            Activity.sport_type,
            func.count(Activity.id),
            func.sum(Activity.distance_m),
            func.sum(Activity.elevation_gain_m),
            func.sum(Activity.moving_time_s),
        )
        .group_by(Activity.sport_type)
        .order_by(func.sum(Activity.distance_m).desc())
    ).all()
    return [
        {
            "sport_type": sport,
            "activity_count": count,
            "distance_m": round(distance or 0.0, 1),
            "elevation_gain_m": round(elevation or 0.0, 1),
            "moving_time_s": round(moving or 0.0, 1),
        }
        for sport, count, distance, elevation, moving in rows
    ]


def weekly_volume(db: Session, weeks: int = 26) -> list[dict[str, Any]]:
    """Per ISO week: volume, elevation, time, count — the main trend chart."""
    activities = list(db.execute(_runs_query(db)).scalars())
    buckets: dict[str, list[Activity]] = defaultdict(list)
    for activity in activities:
        if not activity.local_date:
            continue
        buckets[week_key(activity.local_date)].append(activity)

    keys = sorted(buckets)[-weeks:]
    out: list[dict[str, Any]] = []
    for key in keys:
        bucket = _bucket(buckets[key], key)
        bucket["week_key"] = key
        bucket["start_date"] = _week_start(key)
        out.append(bucket)
    return out


def _week_start(key: str) -> str | None:
    try:
        year, _, week = key.partition("-W")
        return date.fromisocalendar(int(year), int(week), 1).isoformat()
    except (ValueError, TypeError):
        return None


def monthly_volume(db: Session, months: int = 12) -> list[dict[str, Any]]:
    activities = list(db.execute(_runs_query(db)).scalars())
    buckets: dict[str, list[Activity]] = defaultdict(list)
    for activity in activities:
        if activity.local_date:
            buckets[activity.local_date[:7]].append(activity)
    out = []
    for key in sorted(buckets)[-months:]:
        bucket = _bucket(buckets[key], key)
        bucket["month_key"] = key
        out.append(bucket)
    return out


def pace_trends(db: Session, weeks: int = 26) -> list[dict[str, Any]]:
    """Weekly pace statistics — the classic 'am I getting faster' chart."""
    activities = list(db.execute(_runs_query(db)).scalars())
    buckets: dict[str, list[Activity]] = defaultdict(list)
    for activity in activities:
        if activity.local_date:
            buckets[week_key(activity.local_date)].append(activity)

    out = []
    for key in sorted(buckets)[-weeks:]:
        entries = buckets[key]
        paces = [
            p
            for p in (_pace_s_per_km(a.distance_m, a.moving_time_s) for a in entries)
            if p
        ]
        long_runs = [
            p
            for a, p in zip(
                entries,
                (_pace_s_per_km(a.distance_m, a.moving_time_s) for a in entries),
            )
            if p and a.distance_m >= 10000
        ]
        row = {
            "week_key": key,
            "start_date": _week_start(key),
            "run_count": len(entries),
            "median_pace_s_per_km": round(statistics.median(paces), 2) if paces else None,
            "median_pace_label": _pace_label(statistics.median(paces)) if paces else None,
            "best_pace_s_per_km": round(min(paces), 2) if paces else None,
            "best_pace_label": _pace_label(min(paces)) if paces else None,
            "worst_pace_s_per_km": round(max(paces), 2) if paces else None,
            "long_run_pace_s_per_km": round(statistics.median(long_runs), 2) if long_runs else None,
            "long_run_pace_label": _pace_label(statistics.median(long_runs)) if long_runs else None,
        }
        out.append(row)
    return out


def hr_zones(db: Session, weeks: int = 12) -> dict[str, Any]:
    """Heart-rate zone distribution, derived from laps when no zone rows exist."""
    activities = list(db.execute(_runs_query(db)).scalars())
    cutoff = date.today() - timedelta(weeks=weeks)
    recent = [a for a in activities if a.start_time and a.start_time.date() >= cutoff]

    # Garmin's default 5-zone model, anchored on the athlete's observed max HR.
    max_hrs = [a.max_hr for a in recent if a.max_hr]
    peak = max(max_hrs) if max_hrs else 190
    bounds = [0.0, 0.6, 0.7, 0.8, 0.9, 1.01]
    zones = []
    for index in range(5):
        zones.append(
            {
                "zone_no": index + 1,
                "name": ["Recovery", "Aerobic", "Tempo", "Threshold", "Anaerobic"][index],
                "low_bpm": round(bounds[index] * peak),
                "high_bpm": round(bounds[index + 1] * peak),
                "time_s": 0.0,
            }
        )

    total_time = 0.0
    lap_rows = (
        db.execute(
            select(ActivityLap).join(Activity, ActivityLap.activity_id == Activity.id)
        )
        .scalars()
        .all()
    )
    by_activity = {a.id: a for a in activities}
    for lap in lap_rows:
        activity = by_activity.get(lap.activity_id)
        if activity is None or activity.start_time is None:
            continue
        if activity.start_time.date() < cutoff:
            continue
        hr = lap.avg_hr
        if not hr:
            continue
        total_time += lap.duration_s or 0.0
        for zone in zones:
            if zone["low_bpm"] <= hr < zone["high_bpm"]:
                zone["time_s"] += lap.duration_s or 0.0
                break

    for zone in zones:
        zone["time_s"] = round(zone["time_s"], 1)
        zone["percent"] = round(zone["time_s"] / total_time * 100, 1) if total_time else 0.0

    return {
        "zones": zones,
        "total_time_s": round(total_time, 1),
        "peak_hr": peak,
        "weeks": weeks,
    }


def personal_records(db: Session) -> list[dict[str, Any]]:
    """Best performances across the whole history."""
    activities = list(db.execute(_runs_query(db)).scalars())
    records: list[dict[str, Any]] = []

    def add(label: str, value: float, activity: Activity, unit: str, better: str = "lower"):
        records.append(
            {
                "label": label,
                "value": value,
                "display": _display_for(label, value, unit),
                "unit": unit,
                "better": better,
                "activity_id": activity.id,
                "activity_name": activity.name,
                "activity_date": activity.local_date,
            }
        )

    with_distance = [a for a in activities if a.distance_m > 0]
    if with_distance:
        longest = max(with_distance, key=lambda a: a.distance_m)
        add("Longest run", longest.distance_m, longest, "m")
        fastest = min(
            (a for a in with_distance if a.moving_time_s > 0),
            key=lambda a: _pace_s_per_km(a.distance_m, a.moving_time_s) or 1e9,
            default=None,
        )
        if fastest:
            add("Fastest pace", _pace_s_per_km(fastest.distance_m, fastest.moving_time_s) or 0, fastest, "s/km")
        biggest_climb = max(activities, key=lambda a: a.elevation_gain_m)
        add("Biggest climb", biggest_climb.elevation_gain_m, biggest_climb, "m")
        longest_time = max(activities, key=lambda a: a.moving_time_s)
        add("Longest time", longest_time.moving_time_s, longest_time, "s")
        with_hr = [a for a in activities if a.avg_hr]
        if with_hr:
            fastest_hr = max(with_hr, key=lambda a: a.avg_hr)
            add("Fastest avg HR", fastest_hr.avg_hr, fastest_hr, "bpm", better="higher")
        with_cal = [a for a in activities if a.calories]
        if with_cal:
            biggest = max(with_cal, key=lambda a: a.calories)
            add("Most calories", biggest.calories, biggest, "kcal", better="higher")

    records.sort(key=lambda r: r["label"])
    return records


def _display_for(label: str, value: float, unit: str) -> str:
    if unit == "s/km":
        return _pace_label(value) or "-"
    if unit == "s":
        return _duration_label(value) or "-"
    if unit == "m":
        return f"{value / 1000:.2f} km" if value >= 1000 else f"{value:.0f} m"
    return f"{value:.0f} {unit}"


def heatmap(db: Session, year: int | None = None) -> dict[str, Any]:
    """Daily distance for the activity calendar heatmap."""
    target_year = year or date.today().year
    rows = db.execute(
        select(
            Activity.local_date,
            func.sum(Activity.distance_m),
            func.count(Activity.id),
        )
        .where(
            Activity.local_date.like(f"{target_year}-%"),
            Activity.sport_type.in_(RUN_SPORTS),
        )
        .group_by(Activity.local_date)
        .order_by(Activity.local_date)
    ).all()
    days = [
        {
            "date": local_date,
            "distance_m": round(distance or 0.0, 1),
            "activity_count": count,
        }
        for local_date, distance, count in rows
        if local_date
    ]
    return {"year": target_year, "days": days}


def streaks(db: Session) -> dict[str, Any]:
    """Current and longest run streaks (consecutive weeks with a run)."""
    activities = list(db.execute(_runs_query(db)).scalars())
    weeks = sorted({week_key(a.local_date) for a in activities if a.local_date})
    if not weeks:
        return {"current_weeks": 0, "longest_weeks": 0, "total_active_weeks": 0}

    def to_date(key: str) -> date | None:
        try:
            year, _, week = key.partition("-W")
            return date.fromisocalendar(int(year), int(week), 1)
        except (ValueError, TypeError):
            return None

    longest = current = 0
    previous: date | None = None
    for key in weeks:
        current_date = to_date(key)
        if current_date is None:
            continue
        if previous is not None and (current_date - previous).days == 7:
            current += 1
        else:
            current = 1
        longest = max(longest, current)
        previous = current_date

    # Current streak = consecutive weeks ending at the current week.
    today = date.today()
    current_key = week_key(today.isoformat())
    this_week = today - timedelta(days=today.weekday())
    run_weeks = {to_date(k) for k in weeks}
    current_streak = 0
    cursor = this_week
    while cursor in run_weeks:
        current_streak += 1
        cursor -= timedelta(days=7)

    return {
        "current_weeks": current_streak,
        "longest_weeks": longest,
        "total_active_weeks": len(weeks),
        "current_week_key": current_key,
    }


def calendar(db: Session, year: int, month: int) -> dict[str, Any]:
    """Per-day activity list for the training calendar."""
    prefix = f"{year:04d}-{month:02d}"
    activities = (
        db.execute(
            select(Activity)
            .where(
                Activity.local_date.like(f"{prefix}%"),
                Activity.sport_type.in_(RUN_SPORTS),
            )
            .order_by(Activity.start_time)
        )
        .scalars()
        .all()
    )
    by_day: dict[str, list[Activity]] = defaultdict(list)
    for activity in activities:
        by_day[activity.local_date].append(activity)

    days = []
    day_count = 31
    while True:
        try:
            date(year, month, day_count)
        except ValueError:
            day_count -= 1
            break
        day_count += 1

    for day in range(1, day_count):
        key = f"{prefix}-{day:02d}"
        entries = by_day.get(key, [])
        days.append(
            {
                "date": key,
                "day": day,
                "activities": [_activity_card(a) for a in entries],
                "distance_m": round(sum(a.distance_m for a in entries), 1),
                "moving_time_s": round(sum(a.moving_time_s for a in entries), 1),
            }
        )
    return {
        "year": year,
        "month": month,
        "days": days,
        "total_distance_m": round(sum(a.distance_m for a in activities), 1),
        "activity_count": len(activities),
    }


def _activity_card(activity: Activity) -> dict[str, Any]:
    pace = _pace_s_per_km(activity.distance_m, activity.moving_time_s)
    return {
        "id": activity.id,
        "source": activity.source,
        "name": activity.name,
        "sport_type": activity.sport_type,
        "start_time": activity.start_time.isoformat() if activity.start_time else None,
        "local_date": activity.local_date,
        "distance_m": activity.distance_m,
        "moving_time_s": activity.moving_time_s,
        "moving_time_label": _duration_label(activity.moving_time_s),
        "elevation_gain_m": activity.elevation_gain_m,
        "avg_hr": activity.avg_hr,
        "pace_s_per_km": round(pace, 2) if pace else None,
        "pace_label": _pace_label(pace),
        "has_geo": bool(activity.geo_points),
    }


def recent_activities(db: Session, limit: int = 10) -> list[dict[str, Any]]:
    rows = (
        db.execute(
            select(Activity)
            .where(Activity.sport_type.in_(RUN_SPORTS))
            .order_by(Activity.start_time.desc())
            .limit(limit)
        )
        .scalars()
        .all()
    )
    return [_activity_card(a) for a in rows]
