"""Where a plan comes from when nobody wrote one down.

Two sources, and they are **not** the same kind of thing:

* **The plan you already have** — inferred from the athlete's own history:
  their weekly volume, their rest days, their long-run day, their quality day
  and their usual distance per weekday. It is a **baseline**, not a
  prescription, and it says so in every sentence it produces.
* **The plan you wrote down** — Garmin's scheduled workouts. It is a
  **prescription**, because the athlete wrote it in Garmin Connect.

The distinction is not cosmetic. A baseline is compared with *less than
usual* / *more than usual* / *on a usual day*; a prescription is compared
with *hit* and *missed*. If the screen mixes the two, the athlete cannot
tell which kind of comparison they are reading, and "missed" — which is a
judgement about a plan somebody else wrote — would be applied to a plan
nobody wrote at all.

Everything here is a **pure function over plain data**: no database, no
network, no clock. The Rust port in ``crates/stride-core/src/plan_sources.rs``
implements the same functions over the same shapes, and the golden fixtures
in ``crates/stride-core/tests/fixtures/plan_sources/`` prove that the two
agree. This module is the reference.

The shapes
----------

An *activity* is anything with ``local_date``, ``sport_type``,
``distance_m`` and ``moving_time_s`` — a SQLAlchemy ``Activity`` row, a
dictionary, or anything else with those attributes. :func:`facts` normalises
one into a plain dictionary.

A *scheduled workout* is what ``GarminWorkoutClient.scheduled_workouts``
returns: ``workout_id``, ``name``, ``date``, ``sport``, ``steps``,
``status``. A *workout detail* is what ``GarminWorkoutClient.workout_detail``
returns: ``session_type``, ``target_distance_m``, ``target_pace_low_s_km``,
``target_pace_high_s_km``, ``hr_zone_target``, ``title``, ``details``.

A *plan session* is the shape the ``plan_session`` table already has:
``id``, ``date``, ``session_type``, ``title``, ``details``,
``target_distance_m``, ``pace_low_s_km``, ``pace_high_s_km``. The schema does
not change; a source produces rows of that shape and the caller persists
them.
"""

from __future__ import annotations

import statistics
from datetime import date, timedelta
from typing import Any, Iterable, Mapping, Sequence

__all__ = [
    "BASELINE_VERDICTS",
    "GARMIN_DESCRIPTION",
    "HISTORY_WEEKS",
    "MIN_HISTORY_RUNS",
    "MIN_HISTORY_WEEKS",
    "MIN_WEEKDAY_RUNS",
    "PACE_BAND_TOLERANCE",
    "RUN_SPORTS",
    "SESSION_TYPES",
    "USUAL_TOLERANCE",
    "VERDICT_LABELS",
    "WEEKDAYS",
    "baseline_verdict",
    "facts",
    "infer_plan",
    "session_from_workout",
    "sessions_from_workouts",
    "workouts_plan",
]


# ----------------------------------------------------------------- constants

#: How many weeks of history the inference looks at.
HISTORY_WEEKS = 12

#: How many of those weeks must contain a run before a pattern is inferred.
#:
#: Eight of twelve means the athlete was seen running in at least two thirds
#: of the window. A habit asserted from fewer weeks is a guess.
MIN_HISTORY_WEEKS = 8

#: How many runs the window must hold before a pattern is inferred.
#:
#: Twelve is three observations of each of the four weekdays a runner
#: usually uses. Below that, the per-weekday median is a single run.
MIN_HISTORY_RUNS = 12

#: How many runs on a weekday before that weekday counts as a habit.
MIN_WEEKDAY_RUNS = 3

#: How far from the usual distance still counts as "on a usual day".
USUAL_TOLERANCE = 0.10

#: How wide the usual pace band is, either side of the usual pace.
PACE_BAND_TOLERANCE = 0.05

#: The sports the plan engine counts as running. A walk is not a run.
RUN_SPORTS = (
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
)

#: The session types the ``plan_session`` table already knows.
#:
#: These are the kinds a plan may *prescribe*. ``unknown`` is not one of
#: them: it is the absence of a kind, and it is what an imported row that
#: names no kind of session carries. It is accepted on the import paths and
#: read as "not known" everywhere else.
SESSION_TYPES = (
    "easy",
    "long",
    "tempo",
    "interval",
    "race",
    "recovery",
    "strength",
    "rest",
)

#: The weekdays, Monday first — the order Python's ``date.weekday()`` uses.
WEEKDAYS = (
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
)

#: The verdicts a baseline comparison can produce. Never "hit", never
#: "missed": nothing prescribed the session.
BASELINE_VERDICTS = ("less than usual", "more than usual", "on a usual day")

#: The slug of each verdict, and the words the athlete reads.
VERDICT_LABELS = {
    "less_than_usual": "less than usual",
    "more_than_usual": "more than usual",
    "on_a_usual_day": "on a usual day",
    "no_usual_session": "no usual session",
}

#: What a Garmin plan says about itself.
GARMIN_DESCRIPTION = (
    "Scheduled in Garmin Connect. This is the plan you wrote down, so every "
    "session is compared against your own targets."
)


# -------------------------------------------------------------------- helpers


def _get(item: Any, key: str, default: Any = None) -> Any:
    """Read a field from a dictionary or from an object."""
    if isinstance(item, Mapping):
        return item.get(key, default)
    return getattr(item, key, default)


def _float(value: Any) -> float | None:
    """A finite number, or ``None``."""
    if value is None or isinstance(value, bool):
        return None
    try:
        number = float(value)
    except (TypeError, ValueError):
        return None
    if number != number or number in (float("inf"), float("-inf")):
        return None
    return number


def _parse_date(value: str) -> date | None:
    """Parse an ISO date, or an epoch in seconds or milliseconds."""
    text = (value or "").strip()
    if not text:
        return None
    if len(text) >= 10 and text[4] == "-" and text[7] == "-":
        try:
            return date.fromisoformat(text[:10])
        except ValueError:
            return None
    if text.isdigit():
        number = int(text)
        days = number // 86_400_000 if abs(number) > 100_000_000_000 else number // 86_400
        return date(1970, 1, 1) + timedelta(days=days)
    return None


def _iso(value: date) -> str:
    """The ISO form of a date."""
    return value.isoformat()


def _iso_week_start(value: date) -> date:
    """The Monday that starts the ISO week the date falls in."""
    return value - timedelta(days=value.weekday())


def _is_run_sport(sport: Any) -> bool:
    """Whether a sport type counts as a run for the history."""
    if not sport:
        return False
    return str(sport).strip().lower() in RUN_SPORTS


def _is_run_workout(sport: Any) -> bool:
    """Whether a Garmin workout's sport is a run.

    Garmin's calendar uses ``running`` for road, trail, track and treadmill
    alike, and the sport strings it returns are not the importer's normalised
    ones, so a run is anything whose sport says "run".
    """
    if not sport:
        return True
    text = str(sport).strip().lower()
    if not text:
        return True
    return text in RUN_SPORTS or "run" in text


def _pace(distance_m: float, moving_time_s: float | None) -> float | None:
    """The average pace of a run, in seconds per kilometre."""
    if not distance_m or distance_m <= 0 or not moving_time_s or moving_time_s <= 0:
        return None
    return moving_time_s / (distance_m / 1000.0)


def _round(value: float | None, digits: int = 0) -> float | None:
    """Python's ``round()``, which the Rust port reproduces exactly."""
    if value is None:
        return None
    return round(value, digits)


def _km(value: float | None) -> str:
    """A distance in kilometres, to one decimal."""
    if value is None:
        return "-"
    return f"{value / 1000:.1f}"


def _id_key(value: Any) -> str | None:
    """The key a scheduled workout and its detail are paired by."""
    if value is None or isinstance(value, bool):
        return None
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        return str(int(value)) if value.is_integer() else str(value)
    text = str(value).strip()
    return text or None


# --------------------------------------------------------------------- facts


def facts(activities: Iterable[Any]) -> list[dict[str, Any]]:
    """Normalise activities into the plain facts the inference runs over.

    Each fact is ``{"date", "sport_type", "distance_m", "moving_time_s"}``.
    A run of zero metres is dropped: it says nothing about how somebody
    usually runs.
    """
    result: list[dict[str, Any]] = []
    for activity in activities:
        raw_date = _get(activity, "local_date") or _get(activity, "date") or ""
        sport = str(_get(activity, "sport_type") or "")
        distance = _float(_get(activity, "distance_m")) or 0.0
        moving = _float(_get(activity, "moving_time_s"))
        parsed = _parse_date(str(raw_date))
        if parsed is None or not _is_run_sport(sport) or distance <= 0:
            continue
        result.append(
            {
                "date": _iso(parsed),
                "sport_type": sport.strip().lower(),
                "distance_m": distance,
                "moving_time_s": moving,
            }
        )
    result.sort(key=lambda fact: fact["date"])
    return result


# ------------------------------------------------------------ inferred plan


def _window(from_date: date) -> tuple[date, date]:
    """The twelve whole weeks that end before the range starts.

    The window is cut at a week boundary on purpose. A partial last week
    would put a part-week's volume into the weekly median, and would leak
    days of the range being compared into the baseline it is compared
    against.
    """
    start = _iso_week_start(from_date) - timedelta(days=7 * HISTORY_WEEKS)
    end = _iso_week_start(from_date) - timedelta(days=1)
    return start, end


def infer_plan(activities: Iterable[Any], from_date: str, to_date: str) -> dict[str, Any]:
    """The plan an athlete already has, inferred from their own history.

    The result is a *baseline*. It is never a prescription, and it declines
    to say anything at all when the history is too thin to support a
    pattern.
    """
    start_date = _parse_date(from_date)
    end_date = _parse_date(to_date)
    if start_date is None or end_date is None:
        return _declined_plan(
            "The range is not a date, so there is nothing to infer.",
            0,
            0,
            None,
            None,
        )

    window_start, window_end = _window(start_date)
    runs = [
        fact
        for fact in facts(activities)
        if window_start.isoformat() <= fact["date"] <= window_end.isoformat()
    ]

    # The weekly volume: one bucket per week of the window, zeros included.
    mondays = [window_start + timedelta(days=7 * index) for index in range(HISTORY_WEEKS)]
    totals: dict[str, float] = {monday.isoformat(): 0.0 for monday in mondays}
    for fact in runs:
        monday = _iso_week_start(date.fromisoformat(fact["date"])).isoformat()
        if monday in totals:
            totals[monday] += fact["distance_m"]

    weekly_totals = [totals[monday.isoformat()] for monday in mondays]
    covered = [total for total in weekly_totals if total > 0]
    history_weeks = len(covered)
    history_runs = len(runs)

    weekly_volume = statistics.median(weekly_totals) if weekly_totals else None
    volume_low = min(covered) if covered else None
    volume_high = max(weekly_totals) if weekly_totals else None

    if history_runs < MIN_HISTORY_RUNS or history_weeks < MIN_HISTORY_WEEKS:
        return _declined_plan(
            "Not enough history to infer a usual week: "
            f"{history_runs} runs over {history_weeks} weeks. "
            f"Stride needs at least {MIN_HISTORY_RUNS} runs over "
            f"{MIN_HISTORY_WEEKS} weeks.",
            history_weeks,
            history_runs,
            weekly_volume,
            (volume_low, volume_high),
            window_start=window_start,
            window_end=window_end,
        )

    # Per-weekday statistics.
    distances: dict[int, list[float]] = {index: [] for index in range(7)}
    paces: dict[int, list[float]] = {index: [] for index in range(7)}
    for fact in runs:
        weekday = date.fromisoformat(fact["date"]).weekday()
        distances[weekday].append(fact["distance_m"])
        pace = _pace(fact["distance_m"], fact["moving_time_s"])
        if pace is not None:
            paces[weekday].append(pace)

    run_weekdays = [
        index for index in range(7) if len(distances[index]) >= MIN_WEEKDAY_RUNS
    ]
    rest_days = [WEEKDAYS[index] for index in range(7) if not distances[index]]

    median_distance = {
        index: statistics.median(distances[index]) for index in run_weekdays
    }
    median_pace = {
        index: statistics.median(paces[index])
        for index in run_weekdays
        if paces[index]
    }

    long_run_day = _long_run_day(runs, mondays)
    quality_day = _quality_day(run_weekdays, median_pace, distances)

    sessions: list[dict[str, Any]] = []
    if end_date >= start_date:
        for offset in range((end_date - start_date).days + 1):
            day = start_date + timedelta(days=offset)
            weekday = day.weekday()
            if weekday not in run_weekdays:
                continue
            sessions.append(
                _inferred_session(
                    day,
                    weekday,
                    median_distance[weekday],
                    median_pace.get(weekday),
                    long_run_day,
                    quality_day,
                )
            )

    return {
        "source": "inferred",
        "prescribed": False,
        "description": (
            f"Inferred from your last {HISTORY_WEEKS} weeks of running. "
            "This is a baseline of how you usually run, not a prescription."
        ),
        "sufficient": True,
        "reason": None,
        "history_weeks": history_weeks,
        "history_runs": history_runs,
        "window_start": window_start.isoformat(),
        "window_end": window_end.isoformat(),
        "weekly_volume_m": _round(weekly_volume, 0),
        "weekly_volume_low_m": _round(volume_low, 0),
        "weekly_volume_high_m": _round(volume_high, 0),
        "rest_days": rest_days,
        "long_run_day": long_run_day,
        "quality_day": quality_day,
        "usual_by_weekday": {
            WEEKDAYS[index]: _round(median_distance[index], 0) for index in run_weekdays
        },
        "sessions": sessions,
    }


def _declined_plan(
    reason: str,
    history_weeks: int,
    history_runs: int,
    weekly_volume: float | None,
    volume_range: tuple[float | None, float | None] | None,
    window_start: date | None = None,
    window_end: date | None = None,
) -> dict[str, Any]:
    """A plan that declines to infer a pattern, and says why."""
    low, high = volume_range if volume_range else (None, None)
    return {
        "source": "inferred",
        "prescribed": False,
        "description": (
            f"Inferred from your last {HISTORY_WEEKS} weeks of running. "
            "This is a baseline of how you usually run, not a prescription."
        ),
        "sufficient": False,
        "reason": reason,
        "history_weeks": history_weeks,
        "history_runs": history_runs,
        "window_start": window_start.isoformat() if window_start else None,
        "window_end": window_end.isoformat() if window_end else None,
        "weekly_volume_m": _round(weekly_volume, 0),
        "weekly_volume_low_m": _round(low, 0),
        "weekly_volume_high_m": _round(high, 0),
        "rest_days": [],
        "long_run_day": None,
        "quality_day": None,
        "usual_by_weekday": {},
        "sessions": [],
    }


def _long_run_day(runs: Sequence[dict[str, Any]], mondays: Sequence[date]) -> str | None:
    """The weekday of the longest run of the week, the usual one.

    Every week of the window contributes the weekday of its own longest
    run; the weekday that comes up most often is the long-run day. A tie is
    broken by the earlier weekday.
    """
    votes: dict[int, int] = {index: 0 for index in range(7)}
    for monday in mondays:
        key = monday.isoformat()
        week = [
            fact
            for fact in runs
            if key <= fact["date"] <= (monday + timedelta(days=6)).isoformat()
        ]
        if not week:
            continue
        longest = max(week, key=lambda fact: fact["distance_m"])
        votes[date.fromisoformat(longest["date"]).weekday()] += 1
    best = _argmax(votes, lambda index: votes[index])
    return WEEKDAYS[best] if best is not None else None


def _quality_day(
    run_weekdays: Sequence[int],
    median_pace: Mapping[int, float],
    distances: Mapping[int, Sequence[float]],
) -> str | None:
    """The weekday the athlete runs fastest on.

    The fastest median pace wins. Two weekdays within five seconds a
    kilometre are a tie, and the tie is broken by the weekday with more
    distance behind it — that is the one with the most work at intensity.
    """
    paced = [index for index in run_weekdays if index in median_pace]
    if not paced:
        return None
    fastest = min(median_pace[index] for index in paced)
    candidates = [
        index for index in paced if median_pace[index] <= fastest + 5.0
    ]
    best = _argmax(candidates, lambda index: sum(distances[index]))
    return WEEKDAYS[best] if best is not None else None


def _argmax(items: Sequence[int], score) -> int | None:
    """The item with the highest score; ties go to the first one."""
    best: int | None = None
    best_score = 0.0
    for item in items:
        value = score(item)
        if best is None or value > best_score:
            best = item
            best_score = value
    return best


def _inferred_session(
    day: date,
    weekday: int,
    usual_distance_m: float,
    usual_pace: float | None,
    long_run_day: str | None,
    quality_day: str | None,
) -> dict[str, Any]:
    """One dated session of the inferred week."""
    name = WEEKDAYS[weekday]
    if long_run_day == name:
        session_type = "long"
        title = "Your usual long run"
    elif quality_day == name:
        session_type = "tempo"
        title = "Your usual quality run"
    else:
        session_type = "easy"
        title = f"Your usual {name} run"

    pace_low = pace_high = None
    if usual_pace is not None:
        pace_low = _round(usual_pace * (1.0 - PACE_BAND_TOLERANCE), 0)
        pace_high = _round(usual_pace * (1.0 + PACE_BAND_TOLERANCE), 0)

    return {
        "id": 0,
        "date": day.isoformat(),
        "session_type": session_type,
        "title": title,
        "details": None,
        "target_distance_m": _round(usual_distance_m, 0),
        "pace_low_s_km": pace_low,
        "pace_high_s_km": pace_high,
    }


# ------------------------------------------------------------- baseline verdict


def baseline_verdict(session: Mapping[str, Any], activity: Any = None) -> dict[str, Any]:
    """Compare a run with the athlete's usual, without prescribing.

    The verdict is *less than usual*, *more than usual* or *on a usual day*.
    There is no "missed" and no "hit": nobody wrote this plan down, so
    nothing was missed.
    """
    usual = _float(session.get("target_distance_m"))
    if usual is not None and usual <= 0.0:
        usual = None
    day = _parse_date(str(session.get("date") or ""))
    name = WEEKDAYS[day.weekday()] if day else "day"

    actual = None
    pace = None
    if activity is not None:
        distance = _float(_get(activity, "distance_m"))
        if distance is not None and distance > 0.0:
            actual = distance
            moving = _float(_get(activity, "moving_time_s"))
            pace = _pace(distance, moving)

    usual_text = f"You usually run {_km(usual)} km on a {name}"
    if usual is None:
        return {
            "verdict": "no_usual_session",
            "verdict_label": "no usual session",
            "usual_distance_m": None,
            "actual_distance_m": None,
            "delta_m": None,
            "pace_delta_s_km": None,
            "message": "There is no usual session to compare this run with.",
        }

    delta = None if actual is None else actual - usual
    if actual is None:
        verdict = "less_than_usual"
        message = (
            f"{usual_text}; nothing was logged, which is less than usual."
        )
    elif actual < usual * (1.0 - USUAL_TOLERANCE):
        verdict = "less_than_usual"
        message = (
            f"{usual_text}; you ran {_km(actual)} km, which is less than usual."
        )
    elif actual > usual * (1.0 + USUAL_TOLERANCE):
        verdict = "more_than_usual"
        message = (
            f"{usual_text}; you ran {_km(actual)} km, which is more than usual."
        )
    else:
        verdict = "on_a_usual_day"
        message = f"{usual_text}; you ran {_km(actual)} km, which is on a usual day."

    pace_delta = None
    band_low = _float(session.get("pace_low_s_km"))
    band_high = _float(session.get("pace_high_s_km"))
    if pace is not None and (band_low is not None or band_high is not None):
        bounds = [value for value in (band_low, band_high) if value is not None]
        pace_delta = pace - (sum(bounds) / len(bounds))

    return {
        "verdict": verdict,
        "verdict_label": VERDICT_LABELS[verdict],
        "usual_distance_m": _round(usual, 0),
        "actual_distance_m": _round(actual, 0),
        "delta_m": _round(delta, 0),
        "pace_delta_s_km": _round(pace_delta, 1),
        "message": message,
    }


# ------------------------------------------------------------ Garmin workouts


def session_from_workout(
    scheduled: Mapping[str, Any],
    detail: Mapping[str, Any] | None = None,
) -> dict[str, Any] | None:
    """Map one scheduled Garmin workout onto a plan session.

    ``None`` when the workout is not a run on a day — a cycling workout is
    not a run, and a workout with no date cannot be planned.
    """
    detail = detail or {}
    day = _parse_date(str(scheduled.get("date") or ""))
    if day is None:
        return None
    if not _is_run_workout(scheduled.get("sport")):
        return None
    status = str(scheduled.get("status") or "").strip().lower()
    if "cancel" in status or "delete" in status:
        return None

    # A workout that names no kind of session is unknown, not easy. The
    # scheduled workouts are a prescription, so a kind nobody wrote down
    # cannot be filled in with a guess.
    session_type = str(detail.get("session_type") or "unknown").strip().lower()
    if session_type not in SESSION_TYPES:
        session_type = "unknown"

    title = detail.get("title") or scheduled.get("name") or None
    return {
        "id": 0,
        "date": day.isoformat(),
        "session_type": session_type,
        "title": str(title) if title else None,
        "details": detail.get("details") or None,
        "target_distance_m": _float(detail.get("target_distance_m")),
        "pace_low_s_km": _float(detail.get("target_pace_low_s_km")),
        "pace_high_s_km": _float(detail.get("target_pace_high_s_km")),
    }


def sessions_from_workouts(
    scheduled: Iterable[Mapping[str, Any]],
    details: Iterable[Any] = (),
) -> list[dict[str, Any]]:
    """Map Garmin's scheduled workouts onto plan sessions.

    ``details`` is the workout details, paired to their scheduled workout by
    ``workout_id``. It accepts a mapping of id to detail, a sequence of
    ``(id, detail)`` pairs, or a sequence of detail dictionaries that carry
    a ``workout_id`` of their own.
    """
    by_id: dict[str, Mapping[str, Any]] = {}
    if isinstance(details, Mapping):
        for key, value in details.items():
            identifier = _id_key(key)
            if identifier is not None and isinstance(value, Mapping):
                by_id[identifier] = value
    else:
        for item in details:
            if isinstance(item, tuple) and len(item) == 2:
                identifier, value = _id_key(item[0]), item[1]
            elif isinstance(item, Mapping):
                identifier, value = _id_key(item.get("workout_id")), item
            else:
                identifier, value = None, None
            if identifier is not None and isinstance(value, Mapping):
                by_id[identifier] = value

    sessions: list[dict[str, Any]] = []
    for row in scheduled:
        if not isinstance(row, Mapping):
            continue
        identifier = _id_key(row.get("workout_id"))
        detail = by_id.get(identifier) if identifier is not None else None
        session = session_from_workout(row, detail)
        if session is not None:
            sessions.append(session)

    sessions.sort(key=lambda session: (session["date"], session["title"] or ""))
    return sessions


def workouts_plan(
    scheduled: Iterable[Mapping[str, Any]],
    details: Iterable[Any] = (),
) -> dict[str, Any]:
    """The plan the athlete wrote down, as Garmin's scheduled workouts."""
    sessions = sessions_from_workouts(scheduled, details)
    return {
        "source": "garmin",
        "prescribed": True,
        "description": GARMIN_DESCRIPTION,
        "sessions": sessions,
    }
