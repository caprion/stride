"""The wellness pull must not fetch what we already hold.

The pull costs four API calls per day (Personal API) or six or seven (the
Health API), so a 30-day window is 120 to 200 requests. Repeating all of
them on every sync is 120 to 200 requests a day, forever — on a phone over
mobile data that is the difference between a sync and a bill.

The rule is the one the activity pull already uses: an activity that is
already imported reuses its stored data, and the marker says so. Here the
marker is a row that carries a measurement.
"""

from __future__ import annotations

from datetime import date, timedelta

import pytest


@pytest.fixture(scope="session", autouse=True)
def _database():
    """The application's own database, with its tables."""
    from app.db import Base, engine

    Base.metadata.create_all(engine)
    yield


class FakePersonalClient:
    """A Personal API client that counts the calls it is given."""

    def __init__(self) -> None:
        self.calls = 0
        self.seen: list[str] = []

    def _count(self, name: str) -> None:
        self.seen.append(name)
        self.calls += 1

    def daily_summary(self, key: str) -> dict:
        self._count(f"daily_summary:{key}")
        return {"summaryDTO": {"totalSteps": 8000, "restingHeartRate": 48}}

    def sleep(self, key: str) -> dict:
        self._count(f"sleep:{key}")
        return {"sleepTimeSeconds": 27000, "deepSleepSeconds": 5000}

    def hrv(self, key: str) -> dict:
        self._count(f"hrv:{key}")
        return {"hrvSummary": {"lastNightAvg": 42, "status": "BALANCED"}}

    def daily_heart_rate(self, key: str) -> dict:
        self._count(f"heart_rate:{key}")
        return {"heartRateSummary": {"restingHeartRate": 48, "maxHeartRate": 160}}


@pytest.fixture()
def held_day(monkeypatch):
    """A database that already holds yesterday, and a client that counts."""
    from app.db import SessionLocal
    from app.models_phase2 import DailyPhysiology
    from app.services import personal_sync

    yesterday = (date.today() - timedelta(days=1)).isoformat()
    session = SessionLocal()
    try:
        session.query(DailyPhysiology).delete()
        session.add(
            DailyPhysiology(
                date=yesterday,
                source="garmin_personal",
                hrv_ms=42.0,
                sleep_s=27000.0,
                resting_hr=48.0,
            )
        )
        session.commit()
    finally:
        session.close()

    client = FakePersonalClient()
    monkeypatch.setattr(personal_sync, "_client", lambda: client)
    monkeypatch.setattr(personal_sync, "SessionLocal", SessionLocal)
    return yesterday, client


def test_a_day_we_hold_is_not_fetched_again(held_day):
    from app.services.personal_sync import sync_wellness

    yesterday, client = held_day
    result = sync_wellness(days=3)

    assert result["days_skipped"] == 1
    assert result["days"] == 2
    assert not [call for call in client.seen if yesterday in call], client.seen


def test_today_is_always_pulled(held_day):
    """The day is not over: its body battery, stress and steps still move."""
    from app.services.personal_sync import sync_wellness

    _, client = held_day
    today = date.today().isoformat()
    sync_wellness(days=3)
    assert [call for call in client.seen if today in call], client.seen


def test_the_pull_costs_four_calls_a_day(held_day):
    from app.services.personal_sync import sync_wellness

    _, client = held_day
    sync_wellness(days=3)
    # Two days pulled (today and the day before yesterday), four calls each.
    assert client.calls == 8


def test_a_day_with_nothing_in_it_is_fetched(held_day, monkeypatch):
    """An empty row is not a marker: there is nothing to reuse."""
    from app.db import SessionLocal
    from app.models_phase2 import DailyPhysiology
    from app.services.personal_sync import sync_wellness

    yesterday = (date.today() - timedelta(days=1)).isoformat()
    session = SessionLocal()
    try:
        row = session.get(DailyPhysiology, yesterday)
        row.hrv_ms = None
        row.sleep_s = None
        row.resting_hr = None
        session.commit()
    finally:
        session.close()

    _, client = held_day
    result = sync_wellness(days=3)
    assert result["days_skipped"] == 0
    assert [call for call in client.seen if yesterday in call], client.seen


def test_force_re_pulls_everything(held_day):
    from app.services.personal_sync import sync_wellness

    yesterday, client = held_day
    result = sync_wellness(days=3, force=True)
    assert result["days_skipped"] == 0
    assert result["days"] == 3
    assert [call for call in client.seen if yesterday in call], client.seen


def test_the_pull_reports_its_own_cost(held_day):
    from app.services.personal_sync import sync_wellness

    _, client = held_day
    result = sync_wellness(days=3)
    assert result["requests"] == client.calls
    assert result["requests"] == 8


def test_the_health_api_pull_skips_the_days_it_holds(monkeypatch):
    """The account path has the same rule and the same reason."""
    from app.db import SessionLocal
    from app.models_phase2 import DailyPhysiology
    from app.services import health_sync

    yesterday = (date.today() - timedelta(days=1)).isoformat()
    session = SessionLocal()
    try:
        session.query(DailyPhysiology).delete()
        session.add(
            DailyPhysiology(date=yesterday, source="garmin", hrv_ms=42.0, resting_hr=48.0)
        )
        session.commit()
    finally:
        session.close()

    calls: list[str] = []

    class FakeHealth:
        def __init__(self, client=None) -> None:
            self.calls = 0

        def _count(self, name: str) -> None:
            calls.append(name)
            self.calls += 1

        def sleep_range(self, start, end):
            self._count("sleep_range")
            return []

        def hrv_range(self, start, end):
            self._count("hrv_range")
            return {}

        def sleep_for(self, day):
            self._count(f"sleep_for:{day}")
            return {"duration_s": 27000}

        def hrv(self, day):
            self._count(f"hrv:{day}")
            return {"value_ms": 42}

        def body_battery(self, day):
            self._count(f"body_battery:{day}")
            return {"min": 20.0, "avg": 50.0, "max": 80.0}

        def stress(self, day):
            self._count(f"stress:{day}")
            return {"avg": 30.0, "max": 90.0}

        def resting_hr(self, day):
            self._count(f"resting_hr:{day}")
            return {"resting_hr": 48.0}

        def daily_summary(self, day):
            self._count(f"daily_summary:{day}")
            return {"steps": 8000}

        def training_status(self, day):
            self._count(f"training_status:{day}")
            return {"acute_load": 300.0}

    class FakeAccount:
        _client = object()

    class FakeWorkouts:
        def gear(self):
            return []

    monkeypatch.setattr(health_sync, "_account", lambda: FakeAccount())
    monkeypatch.setattr(health_sync, "GarminHealthClient", FakeHealth)
    monkeypatch.setattr(health_sync, "GarminWorkoutClient", lambda client: FakeWorkouts())
    monkeypatch.setattr(health_sync, "SessionLocal", SessionLocal)

    result = health_sync.sync_health(days=3)

    assert result["days_skipped"] == 1
    assert result["days_written"] == 2
    assert not [call for call in calls if yesterday in call], calls


def test_the_marker_is_a_measurement_not_a_row():
    """The rule is the same one the activity pull uses: a marker, not a guess."""
    from app.db import SessionLocal
    from app.models_phase2 import DailyPhysiology
    from app.services.persistence import physiology_day_is_held

    session = SessionLocal()
    try:
        empty = DailyPhysiology(date="1999-01-01", source="garmin")
        session.add(empty)
        session.commit()
        assert physiology_day_is_held(empty) is False

        empty.hrv_ms = 42.0
        session.commit()
        assert physiology_day_is_held(empty) is True
        assert physiology_day_is_held(None) is False
    finally:
        session.query(DailyPhysiology).filter_by(date="1999-01-01").delete()
        session.commit()
        session.close()
