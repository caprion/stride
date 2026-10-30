"""The plan sources: the inferred baseline and Garmin's scheduled workouts.

Run with:

    cd ~/stride/backend && .venv/bin/python -m pytest -q

These are pure functions over plain data, so there is no database and no
network here. The Rust port in ``crates/stride-core/src/plan_sources.rs`` is
measured against this implementation: the golden file
``crates/stride-core/tests/fixtures/plan_sources/expected.json`` is produced
from these functions, and the test at the bottom of this file proves that
file is not stale.
"""

from __future__ import annotations

import json
from datetime import date, timedelta
from pathlib import Path

from app.services.plan_sources import (
    BASELINE_VERDICTS,
    MIN_HISTORY_RUNS,
    MIN_HISTORY_WEEKS,
    baseline_verdict,
    facts,
    infer_plan,
    sessions_from_workouts,
    workouts_plan,
)

REPO = Path(__file__).resolve().parents[2]
FIXTURES = REPO / "crates" / "stride-core" / "tests" / "fixtures" / "plan_sources"


def _history() -> list[dict]:
    """Twelve weeks of a regular week: Tuesday, Thursday, Sunday."""
    activities: list[dict] = []
    for index in range(12):
        monday = date(2025, 12, 8) + timedelta(days=7 * index)
        for offset, distance, pace in ((1, 10000.0, 300.0), (3, 8000.0, 325.0), (6, 14000.0, 314.0)):
            day = monday + timedelta(days=offset)
            activities.append(
                {
                    "local_date": day.isoformat(),
                    "sport_type": "running",
                    "distance_m": distance,
                    "moving_time_s": distance * pace / 1000.0,
                }
            )
    return activities


def test_the_facts_keep_the_runs_and_drop_the_rest():
    activities = [
        {"local_date": "2026-03-02", "sport_type": "running", "distance_m": 5000.0, "moving_time_s": 1500.0},
        {"local_date": "2026-03-03", "sport_type": "walking", "distance_m": 4000.0, "moving_time_s": 2000.0},
        {"local_date": "2026-03-04", "sport_type": "running", "distance_m": 0.0, "moving_time_s": 0.0},
        {"local_date": "", "sport_type": "running", "distance_m": 5000.0, "moving_time_s": 1500.0},
    ]
    kept = facts(activities)
    assert len(kept) == 1
    assert kept[0]["date"] == "2026-03-02"
    assert kept[0]["distance_m"] == 5000.0


def test_the_inferred_plan_derives_a_week():
    plan = infer_plan(_history(), "2026-03-02", "2026-03-08")
    assert plan["sufficient"] is True
    assert plan["reason"] is None
    assert plan["source"] == "inferred"
    assert plan["prescribed"] is False
    assert "not a prescription" in plan["description"]

    # The window is the twelve whole weeks before the range.
    assert plan["window_start"] == "2025-12-08"
    assert plan["window_end"] == "2026-03-01"
    assert plan["history_runs"] == 36
    assert plan["history_weeks"] == 12

    # The usual week.
    assert plan["weekly_volume_m"] == 32000.0
    assert plan["weekly_volume_low_m"] == 32000.0
    assert plan["weekly_volume_high_m"] == 32000.0
    assert plan["rest_days"] == ["Monday", "Wednesday", "Friday", "Saturday"]
    assert plan["long_run_day"] == "Sunday"
    assert plan["quality_day"] == "Tuesday"
    assert plan["usual_by_weekday"] == {
        "Tuesday": 10000.0,
        "Thursday": 8000.0,
        "Sunday": 14000.0,
    }

    # One session per usual running day of the range.
    assert [session["date"] for session in plan["sessions"]] == [
        "2026-03-03",
        "2026-03-05",
        "2026-03-08",
    ]
    assert [session["session_type"] for session in plan["sessions"]] == [
        "tempo",
        "easy",
        "long",
    ]
    assert plan["sessions"][0]["title"] == "Your usual quality run"
    assert plan["sessions"][2]["title"] == "Your usual long run"
    assert plan["sessions"][1]["target_distance_m"] == 8000.0
    assert plan["sessions"][0]["pace_low_s_km"] == 285.0
    assert plan["sessions"][0]["pace_high_s_km"] == 315.0


def test_three_weeks_of_history_declines_rather_than_guessing():
    history = _history()[:9]
    plan = infer_plan(history, "2026-03-02", "2026-03-08")
    assert plan["sufficient"] is False
    assert plan["sessions"] == []
    assert plan["rest_days"] == []
    assert plan["long_run_day"] is None
    assert plan["quality_day"] is None
    assert plan["usual_by_weekday"] == {}
    assert "9 runs over 3 weeks" in plan["reason"]
    assert f"at least {MIN_HISTORY_RUNS} runs over {MIN_HISTORY_WEEKS} weeks" in plan["reason"]


def test_a_range_that_is_not_a_date_declines():
    plan = infer_plan(_history(), "not a date", "2026-03-08")
    assert plan["sufficient"] is False
    assert plan["sessions"] == []
    assert "not a date" in plan["reason"]


def test_the_baseline_verdict_never_says_missed_or_hit():
    session = {
        "id": 0,
        "date": "2026-03-08",
        "session_type": "long",
        "title": "Your usual long run",
        "details": None,
        "target_distance_m": 12000.0,
        "pace_low_s_km": 300.0,
        "pace_high_s_km": 320.0,
    }
    short = baseline_verdict(session, {"distance_m": 4000.0, "moving_time_s": 1300.0})
    assert short["verdict"] == "less_than_usual"
    assert short["verdict_label"] == "less than usual"
    assert short["verdict_label"] in BASELINE_VERDICTS
    assert short["message"] == (
        "You usually run 12.0 km on a Sunday; you ran 4.0 km, which is less than usual."
    )
    assert short["delta_m"] == -8000.0

    more = baseline_verdict(session, {"distance_m": 15000.0, "moving_time_s": 4800.0})
    assert more["verdict"] == "more_than_usual"

    usual = baseline_verdict(session, {"distance_m": 12000.0, "moving_time_s": 3900.0})
    assert usual["verdict"] == "on_a_usual_day"

    nothing = baseline_verdict(session, None)
    assert nothing["verdict"] == "less_than_usual"
    assert "nothing was logged" in nothing["message"]

    for verdict in (short, more, usual, nothing):
        assert verdict["verdict"] not in ("hit", "missed", "missed_pace", "short")


def test_a_garmin_workout_becomes_a_plan_session():
    scheduled = [
        {
            "workout_id": 101,
            "name": "Easy 5K",
            "date": "2026-03-02",
            "sport": "running",
            "steps": [],
            "status": "scheduled",
        },
        {
            "workout_id": 104,
            "name": "Ride",
            "date": "2026-03-03",
            "sport": "cycling",
            "steps": [],
            "status": "scheduled",
        },
        {
            "workout_id": 105,
            "name": "Cancelled run",
            "date": "2026-03-05",
            "sport": "running",
            "steps": [],
            "status": "cancelled",
        },
        {
            "workout_id": 107,
            "name": "No date",
            "date": None,
            "sport": "running",
            "steps": [],
            "status": "scheduled",
        },
    ]
    details = [
        {
            "workout_id": 101,
            "session_type": "easy",
            "target_distance_m": 5000.0,
            "target_pace_low_s_km": 330.0,
            "target_pace_high_s_km": 360.0,
            "hr_zone_target": None,
            "title": "Easy 5K",
            "details": "Run easy, stay relaxed",
        }
    ]
    sessions = sessions_from_workouts(scheduled, details)
    assert len(sessions) == 1
    assert sessions[0]["date"] == "2026-03-02"
    assert sessions[0]["session_type"] == "easy"
    assert sessions[0]["target_distance_m"] == 5000.0
    assert sessions[0]["pace_low_s_km"] == 330.0
    assert sessions[0]["pace_high_s_km"] == 360.0

    plan = workouts_plan(scheduled, details)
    assert plan["source"] == "garmin"
    assert plan["prescribed"] is True
    assert "the plan you wrote down" in plan["description"]


def test_a_workout_without_a_detail_is_a_session_of_an_unknown_kind():
    """A workout that names no kind of session is unknown, not easy.

    The scheduled workouts are a prescription, so a kind nobody wrote down
    cannot be filled in with a guess.
    """
    scheduled = [
        {
            "workout_id": 900,
            "name": "Trail run",
            "date": "2026-03-07",
            "sport": "trail_running",
            "steps": [],
            "status": "scheduled",
        }
    ]
    sessions = sessions_from_workouts(scheduled, [])
    assert len(sessions) == 1
    assert sessions[0]["date"] == "2026-03-07"
    assert sessions[0]["session_type"] == "unknown"
    assert sessions[0]["session_type"] != "easy"
    assert sessions[0]["title"] == "Trail run"
    assert sessions[0]["target_distance_m"] is None


def test_the_golden_file_is_not_stale():
    """The Rust port is measured against `expected.json`; it must be current."""
    inputs = json.loads((FIXTURES / "inputs.json").read_text())
    expected = json.loads((FIXTURES / "expected.json").read_text())

    assert (
        infer_plan(
            inputs["inferred"]["activities"],
            inputs["inferred"]["from"],
            inputs["inferred"]["to"],
        )
        == expected["inferred"]
    )
    assert (
        infer_plan(
            inputs["inferred_thin"]["activities"],
            inputs["inferred_thin"]["from"],
            inputs["inferred_thin"]["to"],
        )
        == expected["inferred_thin"]
    )
    assert (
        workouts_plan(
            inputs["workouts"]["scheduled"], inputs["workouts"]["details"]
        )
        == expected["workouts"]
    )
