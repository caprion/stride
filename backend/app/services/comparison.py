"""Run comparison.

Two runs of the same athlete at different points in their career answer
different questions, and the interesting ones are comparative: *was that run
really better, or was it just shorter and flatter?*

The trick is to compare on **distance-normalised profiles**. Two runs are
resampled onto the same distance axis, so a 5 km and a 21 km run can be laid
over each other and the heart-rate, cadence and pace curves can be read
directly against one another.

The comparison also says **what it is comparing**. Two runs of the same job
— two easy runs, two tempo runs — can be marked: one of them was stronger.
Two runs of different jobs cannot: a time trial beats an easy run on pace by
design. So when the plan says what each run was for, that is the basis; and
when it does not, distance is the proxy and the page says so.
"""

from __future__ import annotations

import logging
from typing import Any

from sqlalchemy.orm import Session

from ..models import Activity
from ..models_phase2 import RunFingerprint
from ..services.analytics import _duration_label, _pace_s_per_km

log = logging.getLogger(__name__)

# How many points each run's profile is resampled to.
PROFILE_POINTS = 120

# The session types that say what a run was *for*. `rest` and `strength` are
# not in the list: a run is never one of them, so a session of that type
# gives a run no intent.
RUN_INTENT_TYPES = ("easy", "long", "tempo", "interval", "race", "recovery")


def canonical_session_type(raw: str) -> str:
    """The canonical session type for a stored one.

    Lower case, trimmed, with the common variants ("intervals", "long_run",
    "threshold") folded in. The reference for this is
    ``crates/stride-core/src/plan.rs``.
    """
    lower = (raw or "").strip().lower().replace("-", "_").replace(" ", "_")
    return {
        "intervals": "interval",
        "interval_run": "interval",
        "repeats": "interval",
        "reps": "interval",
        "track": "interval",
        "long_run": "long",
        "longrun": "long",
        "threshold": "tempo",
        "tempo_run": "tempo",
        "easy_run": "easy",
    }.get(lower, lower)


def plan_intent(session_type: str | None) -> str | None:
    """The intent of one run: the canonical session type it was matched to.

    ``None`` when the run was not matched to a plan session, or when the
    session it was matched to is not a run — a rest day or a strength
    session says nothing about what the run was for.
    """
    if session_type is None:
        return None
    canonical = canonical_session_type(session_type)
    return canonical if canonical in RUN_INTENT_TYPES else None


def comparison_basis(intents: list[str | None]) -> dict[str, Any]:
    """The basis of a comparison, from the plan intent of each run.

    * every run has an intent and they are all the same — like for like;
    * every run has an intent and they differ — different jobs;
    * any run has no intent — distance is the proxy, and the page says so.
    """
    if not intents:
        return {"kind": "distance"}
    kinds: list[str] = []
    for intent in intents:
        if intent is None:
            return {"kind": "distance"}
        kinds.append(intent)
    first = kinds[0]
    if all(kind == first for kind in kinds):
        return {"kind": "plan_session", "session_type": first}
    return {"kind": "different_jobs"}

# The comparison matrix. `better` says which direction is good, so the UI can
# highlight a winner without hard-coding the rule per row.
PARAMETERS: list[dict[str, Any]] = [
    {"key": "distance_m", "label": "Distance", "unit": "km", "better": "neutral", "format": "distance"},
    {"key": "moving_time_s", "label": "Moving time", "unit": "", "better": "neutral", "format": "duration"},
    {"key": "pace_s_per_km", "label": "Average pace", "unit": "/km", "better": "lower", "format": "pace"},
    {"key": "elevation_gain_m", "label": "Elevation gain", "unit": "m", "better": "neutral", "format": "elevation"},
    {"key": "avg_hr", "label": "Average heart rate", "unit": "bpm", "better": "neutral", "format": "hr"},
    {"key": "hr_drift_bpm", "label": "HR drift", "unit": "bpm", "better": "lower", "format": "number"},
    {"key": "pace_drift_pct", "label": "Pace drift", "unit": "%", "better": "lower", "format": "number"},
    {"key": "decoupling_pct", "label": "Aerobic decoupling", "unit": "%", "better": "lower", "format": "number"},
    {"key": "cadence_avg", "label": "Average cadence", "unit": "spm", "better": "neutral", "format": "number"},
    {"key": "cadence_cv", "label": "Cadence stability", "unit": "%", "better": "lower", "format": "number"},
    {"key": "efficiency_first", "label": "Efficiency factor", "unit": "", "better": "higher", "format": "number"},
    {"key": "form_score", "label": "Form score", "unit": "", "better": "higher", "format": "number"},
    {"key": "running_time_s", "label": "Running time", "unit": "", "better": "neutral", "format": "duration"},
    {"key": "walking_time_s", "label": "Walking excluded", "unit": "", "better": "lower", "format": "duration"},
    {"key": "stopped_time_s", "label": "Standing excluded", "unit": "", "better": "lower", "format": "duration"},
]

# Which fingerprint column each parameter reads, when it is not on the activity.
_FINGERPRINT_KEYS = {
    "avg_hr": "hr_avg",
    "hr_drift_bpm": "hr_drift_bpm",
    "pace_drift_pct": "pace_drift_pct",
    "decoupling_pct": "decoupling_pct",
    "cadence_avg": "cadence_avg",
    "cadence_cv": "cadence_cv",
    "efficiency_first": "efficiency_first",
    "form_score": "form_score",
    "running_time_s": "running_time_s",
    "walking_time_s": "walking_time_s",
    "stopped_time_s": "stopped_time_s",
}


def _interpolate(samples: list[tuple[float, float]], target: float) -> float | None:
    """Linear interpolation over ``(x, y)`` pairs, sorted by x."""
    if not samples:
        return None
    if target <= samples[0][0]:
        return samples[0][1]
    if target >= samples[-1][0]:
        return samples[-1][1]
    for index in range(1, len(samples)):
        x0, y0 = samples[index - 1]
        x1, y1 = samples[index]
        if x0 <= target <= x1:
            if x1 == x0:
                return y1
            return y0 + (y1 - y0) * (target - x0) / (x1 - x0)
    return None


def profile_for(activity: Activity, points: int = PROFILE_POINTS) -> list[dict[str, Any]]:
    """Resample a run's running streams onto a shared distance axis.

    The streams all share one sample axis (running samples only), so the profile
    is built by walking the distance stream and interpolating every other
    metric at the same point.
    """
    streams = {stream.kind: stream for stream in activity.streams}
    distance_stream = streams.get("distance")
    if distance_stream is None:
        return []

    # The distance stream is the axis everything else is resampled onto, so
    # every other stream is paired with the distance at the same sample.
    raw_seconds = [float(value) for value in (distance_stream.seconds or [])]
    raw_distances = [
        float(value) for value in (distance_stream.values or []) if value is not None
    ]
    count = min(len(raw_seconds), len(raw_distances))
    if count < 2:
        return []
    seconds = raw_seconds[:count]
    distances = raw_distances[:count]

    def series(kind: str) -> list[tuple[float, float]]:
        stream = streams.get(kind)
        if stream is None:
            return []
        values = [float(value) for value in (stream.values or []) if value is not None]
        pairs = [
            (distances[index], values[index])
            for index in range(min(count, len(values)))
        ]
        return sorted(pairs)

    hr = series("hr")
    cadence = series("cadence")
    pace = series("pace")
    elevation = series("elevation")

    start = distances[0]
    end = distances[-1]
    if end <= start:
        return []

    profile: list[dict[str, Any]] = []
    for index in range(points):
        fraction = index / (points - 1)
        metres = start + (end - start) * fraction
        profile.append(
            {
                "distance_m": round(metres, 1),
                "pct": round(fraction * 100, 2),
                "hr": _interpolate(hr, metres),
                "cadence": _interpolate(cadence, metres),
                "pace_s_per_km": _interpolate(pace, metres),
                "elevation": _interpolate(elevation, metres),
            }
        )
    return profile


def _value_for(parameter: dict[str, Any], activity: Activity, fingerprint: RunFingerprint | None) -> Any:
    key = parameter["key"]
    if key in _FINGERPRINT_KEYS and fingerprint is not None:
        return getattr(fingerprint, _FINGERPRINT_KEYS[key], None)
    if key == "pace_s_per_km":
        return _pace_s_per_km(activity.distance_m, activity.moving_time_s)
    return getattr(activity, key, None)


def run_payload(db: Session, activity: Activity) -> dict[str, Any]:
    """Everything the comparison view needs about one run."""
    fingerprint = db.get(RunFingerprint, activity.id)
    values = {
        parameter["key"]: _value_for(parameter, activity, fingerprint)
        for parameter in PARAMETERS
    }
    return {
        "id": activity.id,
        "name": activity.name,
        "local_date": activity.local_date,
        "day_of_week": activity.day_of_week,
        "sport_type": activity.sport_type,
        "source": activity.source,
        "distance_m": activity.distance_m,
        "moving_time_s": activity.moving_time_s,
        "moving_time_label": _duration_label(activity.moving_time_s),
        "elevation_gain_m": activity.elevation_gain_m,
        "avg_hr": activity.avg_hr,
        "max_hr": activity.max_hr,
        "calories": activity.calories,
        "pace_s_per_km": _pace_s_per_km(activity.distance_m, activity.moving_time_s),
        "has_geo": bool(activity.geo_points),
        "values": values,
        "profile": profile_for(activity),
    }


def candidates(db: Session, limit: int = 300) -> list[dict[str, Any]]:
    """Runs that can be compared — those with a fingerprint to compare."""
    from sqlalchemy import select

    activities = db.execute(
        select(Activity).where(
            Activity.sport_type.in_(
                ("running", "trail_running", "treadmill_running", "virtualrun")
            )
        )
        .order_by(Activity.start_time.desc())
        .limit(limit)
    ).scalars().all()
    out = []
    for activity in activities:
        fingerprint = db.get(RunFingerprint, activity.id)
        out.append(
            {
                "id": activity.id,
                "name": activity.name,
                "local_date": activity.local_date,
                "distance_m": activity.distance_m,
                "pace_s_per_km": _pace_s_per_km(activity.distance_m, activity.moving_time_s),
                "avg_hr": activity.avg_hr,
                "elevation_gain_m": activity.elevation_gain_m,
                "has_fingerprint": fingerprint is not None,
                "has_profile": bool(activity.streams),
                "source": activity.source,
            }
        )
    return out


def compare(db: Session, activity_ids: list[int]) -> dict[str, Any]:
    """Compare a set of runs."""
    from sqlalchemy import select

    found: list[Activity] = []
    missing: list[int] = []
    for activity_id in activity_ids:
        activity = db.get(Activity, activity_id)
        if activity is None:
            missing.append(activity_id)
        else:
            found.append(activity)

    found.sort(key=lambda a: (a.local_date or "", a.id))
    intents = plan_intents(db, found)
    runs = [run_payload(db, activity) for activity in found]
    for run in runs:
        run["plan_session_type"] = intents.get(run["id"])
    return {
        "runs": runs,
        "parameters": PARAMETERS,
        "missing": missing,
        "basis": comparison_basis([intents.get(activity.id) for activity in found]),
    }


def plan_intents(db: Session, activities: list[Activity]) -> dict[int, str]:
    """The plan intent of each activity.

    A run's intent is the canonical session type of the plan session it was
    matched to — ``easy``, ``long``, ``tempo``, ``interval``, ``race`` or
    ``recovery``. A run that was not matched to a session, or that was
    matched to a rest or strength session, has no intent: nothing says what
    the run was for.

    The plan is matched over the range the runs fall in, so a run from last
    year is matched against the session it was planned for.
    """
    from . import plan_engine

    intents: dict[int, str] = {}
    dates = [activity.local_date for activity in activities if activity.local_date]
    if not dates:
        return intents
    matches = plan_engine.match_sessions(db, min(dates), max(dates))
    for match in matches:
        activity = match.get("activity")
        session = match.get("session") or {}
        if not activity:
            continue
        intent = plan_intent(session.get("session_type"))
        if intent is not None:
            intents[activity["id"]] = intent
    return intents
