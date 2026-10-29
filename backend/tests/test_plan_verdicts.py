"""A session that has not happened yet is not a miss.

The plan engine used to report a session dated in the future as ``missed``,
and a future *rest* day as ``hit`` — because a rest day with no activity is
"hit" by the rule. Both were wrong: a session the athlete has not been
given the chance to do yet is not evidence of anything.

Measured on the live data on 29 October: three of the forty-two "missed"
sessions had not happened yet, and one future rest day claimed to have been
hit.

The rule: a session dated after ``today`` is ``not_yet``. It is neither a hit
nor a miss, it does not count in the adherence denominator, and a baseline
(inferred) plan never says "missed" at all.
"""

from __future__ import annotations

from datetime import date, timedelta

import pytest

TODAY = date(2026, 3, 13)


@pytest.fixture()
def plan(db):
    """One block, one past session that was hit, one future session."""
    from app.models import Activity
    from app.models_phase2 import PlanBlock, PlanSession

    past = (TODAY - timedelta(days=1)).isoformat()
    future = (TODAY + timedelta(days=1)).isoformat()
    db.add(
        PlanBlock(
            id=1,
            name="Block",
            status="active",
            start_date=past,
            end_date=future,
        )
    )
    db.add(
        PlanSession(
            id=1,
            block_id=1,
            date=past,
            title="Easy",
            session_type="easy",
            target_distance_m=5000.0,
        )
    )
    db.add(
        PlanSession(
            id=2,
            block_id=1,
            date=future,
            title="Easy",
            session_type="easy",
            target_distance_m=5000.0,
        )
    )
    db.add(
        PlanSession(
            id=3,
            block_id=1,
            date=(TODAY + timedelta(days=2)).isoformat(),
            title="Rest",
            session_type="rest",
            target_distance_m=None,
        )
    )
    db.add(
        Activity(
            source="garmin",
            external_id="a1",
            name="Easy",
            sport_type="running",
            local_date=past,
            distance_m=5000.0,
            moving_time_s=1500.0,
            elapsed_time_s=1500.0,
            raw={},
        )
    )
    db.commit()
    return {"past": past, "future": future}


def test_a_future_session_is_not_yet(db, plan):
    from app.services.plan_engine import match_sessions

    rows = match_sessions(db, plan["past"], plan["future"], today=TODAY)
    verdicts = {row["session"]["date"]: row["verdict"] for row in rows}
    assert verdicts[plan["past"]] == "hit"
    assert verdicts[plan["future"]] == "not_yet"
    assert verdicts[plan["future"]] not in ("hit", "missed")


def test_a_future_rest_day_is_not_a_hit(db, plan):
    """A rest day with no activity is a hit — unless it has not happened yet."""
    from app.services.plan_engine import match_sessions

    rest_day = (TODAY + timedelta(days=2)).isoformat()
    rows = match_sessions(db, plan["past"], rest_day, today=TODAY)
    verdicts = {row["session"]["date"]: row["verdict"] for row in rows}
    assert verdicts[rest_day] == "not_yet"


def test_a_future_session_does_not_count_in_the_adherence(db, plan):
    from app.services.plan_engine import adherence

    summary = adherence(db, plan["past"], plan["future"], today=TODAY)
    # One session was due (the one that was hit). The future one is not in
    # the denominator, so the percentage is about what happened.
    assert summary["planned_sessions"] == 1
    assert summary["hit_sessions"] == 1
    assert summary["missed_sessions"] == 0
    assert summary["overall_pct"] == 100.0


def test_the_adherence_of_a_plan_with_nothing_due_is_empty(db, plan):
    from app.services.plan_engine import adherence

    future = (TODAY + timedelta(days=5)).isoformat()
    summary = adherence(db, future, future, today=TODAY)
    assert summary["planned_sessions"] == 0
    assert summary["overall_pct"] is None


def test_a_baseline_never_says_missed(db, plan):
    """A plan nobody wrote down cannot have been missed."""
    from app.services.plan_engine import match_sessions

    rows = match_sessions(
        db, plan["past"], plan["future"], today=TODAY, prescribed=False
    )
    verdicts = {row["session"]["date"]: row["verdict"] for row in rows}
    assert verdicts[plan["past"]] in (
        "less_than_usual",
        "more_than_usual",
        "on_a_usual_day",
        "no_usual_session",
    )
    assert "missed" not in verdicts.values()
    assert verdicts[plan["future"]] == "not_yet"


def test_the_verdict_is_the_same_on_both_sides_of_today(db, plan):
    """A session dated today has happened; one dated tomorrow has not."""
    from app.models_phase2 import PlanSession
    from app.services.plan_engine import match_sessions

    today_session = TODAY.isoformat()
    db.add(
        PlanSession(
            id=4,
            block_id=1,
            date=today_session,
            title="Easy",
            session_type="easy",
            target_distance_m=5000.0,
        )
    )
    db.commit()

    rows = match_sessions(db, today_session, today_session, today=TODAY)
    # Today has happened: the session is judged, not excused.
    assert rows[0]["verdict"] != "not_yet"

    tomorrow = (TODAY + timedelta(days=1)).isoformat()
    rows = match_sessions(db, tomorrow, tomorrow, today=TODAY)
    assert rows[0]["verdict"] == "not_yet"


# --------------------------------------------------------------------------
# The route that serves a source
# --------------------------------------------------------------------------


def test_the_sources_route_serves_both_sources(db, plan, monkeypatch):
    from fastapi.testclient import TestClient

    from app import main
    from app.db import get_db

    def database():
        yield db

    previous = main.app.dependency_overrides.copy()
    main.app.dependency_overrides[get_db] = database
    try:
        with TestClient(main.app) as client:
            response = client.get(
                "/api/v1/plan/sources",
                params={"from": plan["past"], "to": plan["future"]},
            )
    finally:
        main.app.dependency_overrides.clear()
        main.app.dependency_overrides.update(previous)

    assert response.status_code == 200
    payload = response.json()
    assert set(payload) == {"inferred", "garmin"}
    # The flag is in the payload, and it decides how a session is judged.
    assert payload["inferred"]["prescribed"] is False
    assert payload["garmin"]["prescribed"] is True


def test_the_sources_route_never_reports_a_baseline_as_missed(db, plan, monkeypatch):
    from fastapi.testclient import TestClient

    from app import main
    from app.db import get_db

    def database():
        yield db

    previous = main.app.dependency_overrides.copy()
    main.app.dependency_overrides[get_db] = database
    try:
        with TestClient(main.app) as client:
            response = client.get(
                "/api/v1/plan/sources",
                params={"from": plan["past"], "to": plan["future"]},
            )
    finally:
        main.app.dependency_overrides.clear()
        main.app.dependency_overrides.update(previous)

    payload = response.json()
    for source in ("inferred", "garmin"):
        for session in payload[source]["sessions"]:
            assert session["verdict"] != "missed" or source == "garmin"


def test_the_source_route_judges_by_the_prescribed_flag(db, plan, monkeypatch):
    """The inferred source produces baseline verdicts; the Garmin one does not."""
    from app.api.plan import _source_with_verdicts

    session = {
        "id": 0,
        "date": plan["past"],
        "session_type": "easy",
        "title": "Your usual run",
        "details": None,
        "target_distance_m": 5000.0,
        "pace_low_s_km": None,
        "pace_high_s_km": None,
    }
    baseline = _source_with_verdicts(
        {"source": "inferred", "prescribed": False, "sessions": [session]}, []
    )
    prescribed = _source_with_verdicts(
        {"source": "garmin", "prescribed": True, "sessions": [session]}, []
    )
    assert baseline["sessions"][0]["verdict"] == "less_than_usual"
    assert baseline["sessions"][0]["message"]
    assert prescribed["sessions"][0]["verdict"] == "missed"
