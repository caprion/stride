"""Phase-2 backend regression tests.

Run with:

    cd ~/stride/backend && .venv/bin/python -m pytest -q

All database writes use temporary SQLite; Garmin responses are local fakes.
"""

from __future__ import annotations

import json
import os
import socket
import statistics
import tempfile
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

import pytest

# Configure storage before importing any app module, like test_stride.py.
_TMP = tempfile.TemporaryDirectory(prefix="stride-phase2-")
os.environ["STRIDE_DATA_DIR"] = _TMP.name

from sqlalchemy import create_engine, select  # noqa: E402
from sqlalchemy.orm import Session  # noqa: E402

from app.db import Base, get_db  # noqa: E402
from app.models import Activity, ActivityLap  # noqa: E402
from app.models_phase2 import DailyPhysiology, PlanSession, RunFingerprint  # noqa: E402
from app.ingest.plan_importer import import_plan, parse_distance, parse_pace  # noqa: E402
from app.services import fingerprint, plan_engine, readiness  # noqa: E402


TINY_PLAN = """# Test campaign 2026

| Phase | Weeks | Duration | Weekly Volume |
|-------|-------|----------|---------------|
| Base | 1-1 | Jan 5-11 | 5 km |

| Phase | Easy | Tempo | Intervals | Long Run |
|-------|------|-------|-----------|----------|
| Base | 6:45-7:15 | 5:50 | 5:20-5:35 | 6:45-7:15 |

| Week | Phase | Running (km) | Strength | Key Workout |
|------|-------|--------------|----------|-------------|
| 1 | Base | 5 | 0 | Easy 5km |

#### Week 1 - Base (Jan 5-11) - 5km
| Day | Session | Details |
|-----|---------|---------|
| Tue | Easy 5km | 6:45-7:15/km |
"""


@pytest.fixture(autouse=True)
def offline(monkeypatch):
    """Fail on accidental network access, including inside provider wrappers."""
    attempts = []

    def blocked(*args, **kwargs):
        attempts.append(args)
        raise AssertionError("Network access is forbidden in Phase-2 tests")

    monkeypatch.setattr(socket.socket, "connect", blocked)
    monkeypatch.setattr(socket.socket, "connect_ex", blocked)
    monkeypatch.setattr(socket, "create_connection", blocked)
    yield
    assert attempts == [], "A swallowed provider error attempted network access"


@pytest.fixture()
def db(tmp_path):
    """Give each test a fresh database, including API commits."""
    engine = create_engine(
        f"sqlite:///{tmp_path / 'phase2.db'}",
        connect_args={"check_same_thread": False},
    )
    Base.metadata.create_all(engine)
    with Session(engine, expire_on_commit=False) as session:
        yield session
    engine.dispose()


@pytest.fixture()
def client(db, monkeypatch):
    """Exercise real routers without demo seeding or background sync."""
    from fastapi.testclient import TestClient
    from app import main
    from app.config import state

    def database():
        yield db

    monkeypatch.setattr(main, "_bootstrap", lambda: None)
    monkeypatch.setattr(main, "_start_scheduler", lambda: None)
    monkeypatch.setattr(state, "garmin_connected", False)
    monkeypatch.setattr(state, "garmin_method", None)
    monkeypatch.setattr(state, "demo_visible", False)
    previous = main.app.dependency_overrides.copy()
    main.app.dependency_overrides[get_db] = database
    try:
        with TestClient(main.app) as test_client:
            yield test_client
    finally:
        main.app.dependency_overrides.clear()
        main.app.dependency_overrides.update(previous)


@pytest.fixture(scope="module")
def real_plan():
    """Parse the actual athlete document rather than a substitute fixture."""
    path = Path("/home/sumit/running/seasons/2026-spring-hm-sub2/plan.md")
    return import_plan(path.read_text(), name="HM Sub-2:00 Campaign")


def _activity(day="2026-03-03", distance=5000, pace=350, name="Run"):
    return Activity(
        source="test", external_id=f"{day}-{name}", name=name,
        sport_type="running", local_date=day,
        start_time=datetime.fromisoformat(day).replace(tzinfo=timezone.utc),
        distance_m=distance, moving_time_s=distance / 1000 * pace,
        max_hr=200, avg_hr=150,
    )


def _session(day="2026-03-03", kind="easy", distance=5000):
    return PlanSession(
        date=day, session_type=kind, title=kind,
        target_distance_m=distance, pace_low_s_km=340, pace_high_s_km=360,
    )


def _fingerprint(db, paces, cadences=None, distances=None, name="Run"):
    cadences = cadences if cadences is not None else [170] * len(paces)
    distances = distances if distances is not None else [1000] * len(paces)
    activity = _activity(distance=sum(distances), name=name)
    activity.moving_time_s = sum(p * d / 1000 for p, d in zip(paces, distances))
    activity.laps = [
        ActivityLap(
            lap_index=i + 1, distance_m=d, duration_s=p * d / 1000,
            avg_hr=150, max_hr=200, avg_cadence=c,
        )
        for i, (p, c, d) in enumerate(zip(paces, cadences, distances))
    ]
    db.add(activity)
    db.flush()
    return fingerprint.compute_fingerprint(db, activity.id)


# ------------------------------------------------------------ plan importer


def test_real_plan_tree(real_plan):
    """Protect the real campaign's full phase, week and session coverage."""
    assert len(real_plan["phases"]) == 8
    assert len(real_plan["weeks"]) == 20
    assert len(real_plan["sessions"]) >= 60
    assert [w["week_number"] for w in real_plan["weeks"]] == list(range(1, 21))
    block = real_plan["block"]
    assert block["name"] == "HM Sub-2:00 Campaign"
    assert block["start_date"] == "2026-01-05"
    assert block["end_date"] == "2026-05-24"
    assert isinstance(real_plan["warnings"], list)


def test_real_plan_week_nine_weekdays(real_plan):
    """Protect dated Tue/Thu/Sat/Sun sessions and frozen session types."""
    sessions = [s for s in real_plan["sessions"] if "2026-03-02" <= s["date"] <= "2026-03-08"]
    assert [(s["date"], s["session_type"]) for s in sessions] == [
        ("2026-03-03", "easy"), ("2026-03-05", "interval"),
        ("2026-03-07", "long"), ("2026-03-08", "easy"),
    ]
    allowed = {"easy", "long", "tempo", "interval", "race", "recovery", "strength", "rest"}
    assert all(s["session_type"] in allowed for s in real_plan["sessions"])
    assert [date.fromisoformat(s["date"]).weekday() for s in sessions] == [1, 3, 5, 6]


def test_real_plan_units(real_plan):
    """Protect phase/session paces in s/km and long-run distance in metres."""
    base = next(p for p in real_plan["phases"] if p["name"] == "Base")
    assert (base["pace_tempo_low"], base["pace_tempo_high"]) == (350, 360)
    assert (base["pace_long_low"], base["pace_long_high"]) == (405, 435)
    easy = next(s for s in real_plan["sessions"] if s["date"] == "2026-03-03")
    assert (easy["pace_low_s_km"], easy["pace_high_s_km"]) == (405, 435)
    long = next(s for s in real_plan["sessions"] if s["date"] == "2026-03-14")
    assert long["target_distance_m"] == 21000.0


@pytest.mark.parametrize("text,expected", [("5:50", (350, 350)), ("6:45-7:15/km", (405, 435))])
def test_parse_pace_seconds(text, expected):
    """Protect single and ranged clock paces from minute/second confusion."""
    assert parse_pace(text) == expected


def test_parse_distance_metres():
    """Protect km-to-m conversion without dropping the unit multiplier."""
    assert parse_distance("21km") == 21000.0


def test_markdown_single_tempo_pace():
    """Protect a 5:50 tempo band through the complete markdown importer."""
    parsed = import_plan(TINY_PLAN.replace("Easy 5km | 6:45-7:15/km", "Tempo 5km | 5:50/km"))
    session = parsed["sessions"][0]
    assert session["session_type"] == "tempo"
    assert (session["pace_low_s_km"], session["pace_high_s_km"]) == (350, 350)


def test_truncated_plan_warns():
    """Protect graceful partial imports when the dated section is truncated."""
    parsed = import_plan(TINY_PLAN.split("#### Week")[0])
    assert set(parsed) >= {"block", "phases", "weeks", "sessions", "warnings"}
    assert isinstance(parsed["warnings"], list)
    assert parsed["warnings"]


@pytest.mark.parametrize("bad_row", ["| 1 |", "| 1 | Base | unknown |"])
def test_malformed_progression_warns(bad_row):
    """Protect partial plan recovery from short rows and nonnumeric volume."""
    content = "# Plan 2026\n| Week | Phase | Running (km) |\n|---|---|---|\n" + bad_row
    parsed = import_plan(content)
    assert set(parsed) >= {"block", "phases", "weeks", "sessions", "warnings"}
    assert isinstance(parsed["warnings"], list) and parsed["warnings"]


def test_import_csv():
    """Protect CSV session fields, dates and explicit canonical units."""
    content = "date,session_type,title,target_distance_m,pace_low_s_km,pace_high_s_km\n2026-03-03,easy,Easy run,5000,405,435\n"
    parsed = import_plan(content, "csv", "CSV campaign")
    assert parsed["block"]["name"] == "CSV campaign"
    assert len(parsed["sessions"]) == 1
    session = parsed["sessions"][0]
    assert session["date"] == "2026-03-03"
    assert session["session_type"] == "easy"
    assert float(session["target_distance_m"]) == 5000
    assert float(session["pace_low_s_km"]) == 405
    assert float(session["pace_high_s_km"]) == 435
    assert parsed["warnings"] == []


def test_import_json():
    """Protect JSON session normalisation without changing numeric units."""
    session = {"date": "2026-03-03", "session_type": "tempo", "target_distance_m": 5000, "pace_low_s_km": 350, "pace_high_s_km": 350}
    parsed = import_plan(json.dumps({"name": "JSON campaign", "sessions": [session]}), "json")
    assert parsed["block"]["name"] == "JSON campaign"
    assert parsed["sessions"] == [session]
    assert parsed["warnings"] == []


# --------------------------------------------------------------- plan engine


@pytest.mark.parametrize("kind,distance,pace,present,expected", [
    ("easy", 5000, 350, True, "hit"),
    ("easy", 5000, 400, True, "missed_pace"),
    ("easy", 2000, 350, True, "short"),
    ("easy", 0, 350, False, "missed"),
    ("rest", 5000, 350, True, "substituted"),
])
def test_verdict_for(kind, distance, pace, present, expected):
    """Protect distance, pace, absence and rest-day substitution verdicts."""
    activity = _activity(distance=distance, pace=pace) if present else None
    result = plan_engine.verdict_for(_session(kind=kind), activity)
    assert result["verdict"] == expected
    if expected == "hit":
        assert result["distance_hit"] is True
        assert result["pace_band_hit"] is True
        assert result["distance_delta_m"] == 0
        assert result["pace_delta_s_km"] == 0


def test_adherence_totals_and_histogram(db):
    """Protect aggregate totals against independently known rows and verdicts."""
    days = ["2026-03-02", "2026-03-06", "2026-03-10", "2026-03-14", "2026-03-18"]
    db.add_all([_session(day, kind="rest" if i == 4 else "easy", distance=0 if i == 4 else 5000) for i, day in enumerate(days)])
    db.add_all([
        _activity(days[0]), _activity(days[1], pace=400),
        _activity(days[2], distance=2000), _activity(days[4], distance=3000),
    ])
    db.flush()
    rows = plan_engine.match_sessions(db, days[0], days[-1])
    summary = plan_engine.adherence(db, days[0], days[-1])
    assert [r["verdict"] for r in rows] == ["hit", "missed_pace", "short", "missed", "substituted"]
    assert summary["planned_km"] == 20
    assert summary["actual_km"] == 15
    assert summary["planned_sessions"] == 5
    assert summary["completed_sessions"] == 4
    assert summary["missed_sessions"] == 1
    assert summary["by_verdict"] == {"hit": 1, "missed_pace": 1, "short": 1, "missed": 1, "substituted": 1}
    assert sum(summary["by_verdict"].values()) == len(rows)
    for field in ("planned_km", "actual_km", "planned_sessions", "completed_sessions", "missed_sessions"):
        assert sum(row[field] for row in summary["rows"]) == summary[field]


def test_adherence_without_plan_is_unknown(db):
    """Protect no-plan honesty even when an unplanned run exists."""
    db.add(_activity())
    db.flush()
    summary = plan_engine.adherence(db, "2026-03-01", "2026-03-31")
    assert summary["overall_pct"] is None
    assert summary["planned_sessions"] == 0
    assert summary["rows"] == []


def test_hit_has_no_cause():
    """Protect successful sessions from speculative fatigue explanations."""
    physiology = {"sleep_s": 10000, "sleep_baseline_s": 28800}
    assert plan_engine.explain_cause(_session(), _activity(), physiology, "hit") == (None, None)


def test_pace_miss_without_physiology_is_unexplained():
    """Protect honesty when a logged run misses pace without health data."""
    cause, confidence = plan_engine.explain_cause(_session(), _activity(pace=400), None, "missed_pace")
    assert "physiology" in cause.lower() and "unavailable" in cause.lower()
    assert confidence == 0


def test_missed_session_states_the_activity_is_absent():
    """Protect the honest cause for a session that was never run.

    Physiology is irrelevant here: you do not need sleep or HRV to know the
    run did not happen, so the cause says exactly that.
    """
    cause, confidence = plan_engine.explain_cause(_session(), None, None, "missed")
    assert "no activity" in cause.lower()
    assert confidence == 0


# --------------------------------------------------------------- fingerprint


def test_fingerprint_cadence_math(db):
    """Protect arithmetic mean and sample-stdev cadence CV with known laps."""
    cadences = [160, 165, 170, 175, 180, 170]
    result = _fingerprint(db, [350] * 6, cadences)
    assert result["cadence_avg"] == pytest.approx(statistics.mean(cadences))
    assert result["cadence_cv"] == pytest.approx(100 * statistics.stdev(cadences) / statistics.mean(cadences))
    assert db.get(RunFingerprint, result["activity_id"]).cadence_avg == result["cadence_avg"]


@pytest.mark.parametrize("last_pace,sign", [(400, 1), (300, -1)])
def test_fingerprint_decoupling_direction(db, last_pace, sign):
    """Protect positive fade and negative improvement at unchanged HR."""
    result = _fingerprint(db, [350, 350, 350, 350, last_pace, last_pace])
    expected = (1 - 350 / last_pace) * 100
    assert result["decoupling_pct"] == pytest.approx(expected)
    assert result["decoupling_pct"] * sign > 0


def test_fingerprint_speed_efficiency_not_inverted(db):
    """Regression guard: faster at the same HR must yield higher Minetti EF."""
    slow = _fingerprint(db, [400] * 6, name="Slow")
    fast = _fingerprint(db, [300] * 6, name="Fast")
    for key in ("efficiency_first", "efficiency_last"):
        assert fast[key] == pytest.approx((1000 / 300) / (150 / 200))
        assert slow[key] == pytest.approx((1000 / 400) / (150 / 200))
        assert fast[key] > slow[key]


def test_fingerprint_form_rewards_stability(db):
    """Protect bounded form scores that reward stable rather than erratic cadence."""
    stable = _fingerprint(db, [350] * 6, [170] * 6, name="Stable")
    erratic = _fingerprint(db, [350] * 6, [130, 200, 140, 195, 135, 210], name="Erratic")
    assert 0 <= erratic["form_score"] < stable["form_score"] <= 100
    assert stable["decoupling_pct"] == pytest.approx(0)
    assert stable["form_score"] == pytest.approx(100)


@pytest.mark.parametrize("trailing_distance,excluded", [(500, True), (900, False), (1000, False)])
def test_fingerprint_pace_cv_trailing_lap(db, trailing_distance, excluded):
    """Protect the strict 15%-distance threshold, including its exact boundary."""
    paces = [330, 340, 350, 360, 370, 800]
    result = _fingerprint(db, paces, distances=[1020] * 5 + [trailing_distance])
    expected_paces = paces[:-1] if excluded else paces
    expected = 100 * statistics.stdev(expected_paces) / statistics.mean(expected_paces)
    assert result["pace_cv"] == pytest.approx(expected)


def test_fingerprint_drift_and_form_on_running_laps(db):
    """Protect positive HR drift and decoupling for a genuinely fading run."""
    def make(name, hrs, paces):
        activity = _activity(name=name)
        activity.distance_m = 6000
        activity.laps = [ActivityLap(lap_index=i + 1, distance_m=1000, duration_s=pace, avg_hr=hr, max_hr=hr + 5, avg_cadence=170) for i, (hr, pace) in enumerate(zip(hrs, paces))]
        db.add(activity)
        db.flush()
        return fingerprint.compute_fingerprint(db, activity.id)

    drifting = make("Drifting", [140, 140, 140, 165, 165, 165], [330, 330, 330, 390, 390, 390])
    stable = make("Stable front to back", [150] * 6, [360] * 6)
    assert drifting["hr_drift_pct"] > 0
    assert drifting["decoupling_pct"] > 0
    assert stable["hr_drift_pct"] == pytest.approx(0)
    assert stable["decoupling_pct"] == pytest.approx(0)
    assert stable["form_score"] > drifting["form_score"]


@pytest.mark.xfail(reason="Fingerprint currently consumes every persisted lap; it has no segment/kind signal to exclude walking laps. Persist running-only laps before fingerprint computation.")
def test_fingerprint_excludes_slow_walking_lap(db):
    """Protect cadence averages from a walking lap included in the activity."""
    activity = _activity(name="Run with cooldown walk")
    activity.distance_m = 5000
    activity.laps = [
        ActivityLap(lap_index=1, distance_m=2000, duration_s=700, avg_hr=150, max_hr=160, avg_cadence=170),
        ActivityLap(lap_index=2, distance_m=1000, duration_s=900, avg_hr=110, max_hr=120, avg_cadence=90),
        ActivityLap(lap_index=3, distance_m=2000, duration_s=700, avg_hr=155, max_hr=165, avg_cadence=180),
    ]
    db.add(activity)
    db.flush()
    result = fingerprint.compute_fingerprint(db, activity.id)
    assert result["cadence_avg"] == pytest.approx(175)


@pytest.mark.xfail(reason="The fingerprint API does not currently expose hr_drift_bpm or pace_drift_pct; only hr_drift_pct and decoupling_pct are available.")
def test_fingerprint_exposes_absolute_and_pace_drift_fields(db):
    """Protect the proposed absolute HR and pace drift outputs."""
    result = _fingerprint(db, [330, 330, 330, 390, 390, 390])
    assert result["hr_drift_bpm"] > 0
    assert result["pace_drift_pct"] > 0


# ---------------------------------------------------------------- readiness


def test_readiness_missing_is_unknown(db):
    """Protect missing health data from appearing as a zero recovery score."""
    result = readiness.compute_readiness(db, "2026-03-01")
    assert result["score"] is None
    assert result["grade"] == "unknown"
    assert result["drivers"] == []


def test_readiness_sleep_only_renormalises(db):
    """Protect available-weight renormalisation without HRV/RHR/stress/battery."""
    db.add_all([DailyPhysiology(date="2026-02-28", sleep_s=28800), DailyPhysiology(date="2026-03-01", sleep_s=21600)])
    db.flush()
    result = readiness.compute_readiness(db, "2026-03-01")
    assert result["score"] == pytest.approx(75)
    assert result["score"] != 25
    assert [d["key"] for d in result["drivers"]] == ["sleep"]
    assert result["drivers"][0]["contribution"] == pytest.approx(75)


def test_readiness_baseline_preceding_thirty_days(db):
    """Protect trailing-window baselines from old, same-day and future rows."""
    first = date(2026, 1, 1)
    rows = [DailyPhysiology(date=(first + timedelta(days=i)).isoformat(), sleep_s=28800 if i < 30 else 14400, hrv_ms=60 if i < 30 else 30, resting_hr=50 if i < 30 else 70) for i in range(60)]
    db.add_all(rows)
    db.flush()
    assert readiness.baselines(rows, "2026-01-31") == {"sleep_s": 28800, "hrv_ms": 60, "resting_hr": 50}
    start = (date(2026, 2, 20) - timedelta(days=30)).isoformat()
    history = [r for r in rows if start <= r.date < "2026-02-20"]
    expected = {"sleep_s": statistics.median(r.sleep_s for r in history), "hrv_ms": statistics.mean(r.hrv_ms for r in history), "resting_hr": statistics.mean(r.resting_hr for r in history)}
    assert readiness.baselines(rows, "2026-02-20") == expected
    result = readiness.compute_readiness(db, "2026-01-31")
    drivers = {d["key"]: d for d in result["drivers"]}
    assert drivers["sleep"]["baseline"] == 8
    assert drivers["hrv"]["baseline"] == 60
    assert drivers["resting_hr"]["baseline"] == 50
    later = readiness.compute_readiness(db, "2026-02-20")
    later_drivers = {d["key"]: d for d in later["drivers"]}
    assert later_drivers["sleep"]["baseline"] == expected["sleep_s"] / 3600
    assert later_drivers["hrv"]["baseline"] == expected["hrv_ms"]
    assert later_drivers["resting_hr"]["baseline"] == expected["resting_hr"]


@pytest.mark.parametrize("stress,battery", [(0, 100), (100, 0), (40, 70)])
def test_readiness_bounds_and_drivers(db, stress, battery):
    """Protect score bounds and contributions that explain the returned score."""
    db.add(DailyPhysiology(date="2026-03-01", stress_avg=stress, body_battery_avg=battery))
    db.flush()
    result = readiness.compute_readiness(db, "2026-03-01")
    assert 0 <= result["score"] <= 100
    assert result["score"] == pytest.approx(((100 - stress) + battery) / 2)
    assert {d["key"] for d in result["drivers"]} == {"stress", "body_battery"}
    assert all(d["detail"] for d in result["drivers"])
    assert sum(d["contribution"] for d in result["drivers"]) == pytest.approx(result["score"], abs=0.02)


# ---------------------------------------------------------------- API shape


@pytest.mark.parametrize("path,keys", [
    ("/plan/blocks", {"items"}),
    ("/plan/adherence", {"overall_pct", "by_verdict", "rows"}),
    ("/plan/today", {"has_plan", "has_session"}),
    ("/physiology/sources", {"available", "endpoints"}),
    ("/fitness/trends", {"metric", "points", "baseline", "label", "unit", "better"}),
    ("/gear", {"items"}),
])
def test_api_shape_without_parameters(client, path, keys):
    """Protect public response keys and optional query parameters from 422s."""
    response = client.get("/api/v1" + path)
    assert response.status_code == 200, response.text
    payload = response.json()
    assert set(payload) >= keys
    for key in keys & {"items", "rows", "points", "endpoints"}:
        assert isinstance(payload[key], list)
    if path == "/plan/adherence":
        assert payload["overall_pct"] is None
    if path == "/plan/today":
        assert payload["has_plan"] is False and payload["has_session"] is False
    if path == "/physiology/sources":
        assert payload["available"] is False


def test_api_import_markdown_persists(client, db):
    """Protect successful markdown imports and verify the persisted public block."""
    response = client.post("/api/v1/plan/import", json={"format": "markdown", "content": TINY_PLAN, "name": "Tiny campaign", "anchor": "none"})
    assert response.status_code == 200, response.text
    payload = response.json()
    assert payload["ok"] is True
    assert payload["sessions"] > 0
    sessions = list(db.scalars(select(PlanSession).where(PlanSession.block_id == payload["block_id"])))
    assert len(sessions) == payload["sessions"]
    assert sessions[0].target_distance_m == 5000
    listing = client.get("/api/v1/plan/blocks").json()["items"]
    block = next(b for b in listing if b["id"] == payload["block_id"])
    assert block["session_count"] == payload["sessions"]
    assert block["name"] == "Tiny campaign"


def test_api_import_session_count_contract(client):
    """Protect the import response's session count, separate from persistence checks."""
    response = client.post("/api/v1/plan/import", json={"content": TINY_PLAN, "anchor": "none"})
    assert response.status_code == 200
    payload = response.json()
    assert payload["ok"] is True
    assert payload["sessions"] > 0


# ----------------------------------------------------------- Garmin clients


class FakeGarmin:
    """Only canned endpoint replies; the counter detects hidden provider calls."""

    def __init__(self, sleep=None, workout=None):
        self.sleep = sleep
        self.workout = workout
        self.calls = []

    def get_sleep_data(self, day):
        self.calls.append(("get_sleep_data", day))
        return self.sleep

    def get_workout_by_id(self, workout_id):
        self.calls.append(("get_workout_by_id", workout_id))
        return self.workout


@pytest.fixture()
def sleep_payload():
    return {"dailySleepDTO": {
        "sleepTimeSeconds": 27000, "deepSleepSeconds": 5400,
        "remSleepSeconds": 6300, "lightSleepSeconds": 15300,
        "awakeSleepSeconds": 900,
        "sleepScores": {"overall": {"value": 82, "qualifierKey": "GOOD"}},
    }}


def test_garmin_sleep_normalises_realistic_dto(sleep_payload):
    """Protect Garmin dailySleepDTO aggregate duration, score and stage splits."""
    from app.ingest.garmin_health import GarminHealthClient

    fake = FakeGarmin(sleep=sleep_payload)
    result = GarminHealthClient(fake).sleep_for("2026-03-01")
    assert result["duration_s"] == 27000
    assert result["score"] == 82
    assert {k: result[k] for k in ("deep_s", "rem_s", "light_s", "awake_s")} == {"deep_s": 5400, "rem_s": 6300, "light_s": 15300, "awake_s": 900}
    assert fake.calls == [("get_sleep_data", "2026-03-01")]


def test_garmin_sleep_level_arrays():
    """Protect supported named sleep-level arrays and summing repeated stages."""
    from app.ingest.garmin_health import GarminHealthClient

    payload = {"dailySleepDTO": {"sleepTimeSeconds": 27000, "sleepScores": {"overall": {"value": 82}}}, "sleepLevels": [
        {"level": "deep", "seconds": 2700}, {"level": "deep", "seconds": 2700},
        {"level": "rem", "seconds": 6300}, {"level": "light", "seconds": 15300},
        {"level": "awake", "seconds": 900},
    ]}
    assert GarminHealthClient(FakeGarmin(sleep=payload)).sleep_for("2026-03-01") == {"duration_s": 27000, "score": 82, "deep_s": 5400, "rem_s": 6300, "light_s": 15300, "awake_s": 900}


@pytest.mark.parametrize("payload", [None, {}, {"dailySleepDTO": {}}])
def test_garmin_no_sleep_is_empty(payload):
    """Protect missing sleep from exceptions and fabricated zero measurements."""
    from app.ingest.garmin_health import GarminHealthClient

    assert GarminHealthClient(FakeGarmin(sleep=payload)).sleep_for("2026-03-01") == {}


def test_garmin_health_sources_are_local(sleep_payload):
    """Protect capability reporting from hidden network probes before/after pulls."""
    from app.ingest.garmin_health import GarminHealthClient

    fake = FakeGarmin(sleep=sleep_payload)
    health = GarminHealthClient(fake)
    assert health.health_sources()["sleep"] is False
    assert fake.calls == []
    health.sleep_for("2026-03-01")
    calls = list(fake.calls)
    assert health.health_sources()["sleep"] is True
    assert fake.calls == calls


@pytest.fixture()
def workout_payload():
    return {"workoutName": "Track session", "workoutSteps": [
        {"stepType": "repeat", "numberOfIterations": 4, "stepList": [
            {"stepType": "interval", "targetType": "pace", "targetValueLow": 0.3, "targetValueHigh": 0.36},
        ]},
    ]}


@pytest.mark.parametrize("low,high", [(0.3, 0.36), (300, 360), ("5:00", "6:00")])
def test_garmin_repeat_workout_pace_units(workout_payload, low, high):
    """Protect nested repeat classification and physically sane s/km pace bands."""
    from app.ingest.garmin_workouts import GarminWorkoutClient

    step = workout_payload["workoutSteps"][0]["stepList"][0]
    step.update(targetValueLow=low, targetValueHigh=high)
    fake = FakeGarmin(workout=workout_payload)
    result = GarminWorkoutClient(fake).workout_detail(42)
    assert result["session_type"] == "interval"
    assert result["target_pace_low_s_km"] == pytest.approx(300)
    assert result["target_pace_high_s_km"] == pytest.approx(360)
    assert 150 <= result["target_pace_low_s_km"] <= result["target_pace_high_s_km"] <= 600
    assert fake.calls == [("get_workout_by_id", 42)]


def test_garmin_workout_distance_is_not_a_pace(workout_payload):
    """Protect distance targetValue from contaminating a pace target band."""
    from app.ingest.garmin_workouts import GarminWorkoutClient

    step = workout_payload["workoutSteps"][0]["stepList"][0]
    step.update(durationType="distance", targetValue=1000)
    result = GarminWorkoutClient(FakeGarmin(workout=workout_payload)).workout_detail(42)
    assert result["target_pace_low_s_km"] == 300
    assert result["target_pace_high_s_km"] == 360


def test_garmin_workout_sources_are_local():
    """Protect the requested workout sources interface from network probing."""
    from app.ingest.garmin_workouts import GarminWorkoutClient

    fake = FakeGarmin()
    result = GarminWorkoutClient(fake).sources()
    assert isinstance(result, dict)
    assert fake.calls == []
