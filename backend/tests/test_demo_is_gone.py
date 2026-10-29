"""The demo dataset is gone, and it cannot come back.

Not behind a flag, not seeded on first boot, not filtered in a query. The
demo was a confession built into the product — *"you are browsing sample
data"*, forever, seeded at the athlete's own coordinates — and it hid the
real first-run experience, which is an empty app that says what to do.

These tests are the tripwire. They assert the absence of the machinery, so
the next person who finds an empty screen does not reach for a seed script.
"""

from __future__ import annotations

import importlib
import json
from pathlib import Path

import pytest

APP_DIR = Path(__file__).resolve().parent.parent / "app"

# The ``client`` fixture replaces ``main._bootstrap`` with a no-op, so the
# original is captured here, at import time, before any fixture runs.
from app import main as _main  # noqa: E402

REAL_BOOTSTRAP = _main._bootstrap


def _python_sources() -> list[Path]:
    return sorted(APP_DIR.rglob("*.py"))


def test_the_demo_ingest_module_is_gone():
    """``app.ingest.demo`` does not exist, and cannot be imported."""
    assert not (APP_DIR / "ingest" / "demo.py").exists()
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("app.ingest.demo")


def test_no_module_can_seed_or_hide_the_demo():
    """No source file mentions the seeder, the filter or the flag."""
    forbidden = ("bootstrap_demo", "demo_filter", "demo_visible", "STRIDE_SEED_DEMO")
    offenders: list[str] = []
    for path in _python_sources():
        text = path.read_text(encoding="utf-8")
        for needle in forbidden:
            if needle in text:
                offenders.append(f"{path.relative_to(APP_DIR)}: {needle}")
    assert offenders == [], offenders


def test_nothing_writes_the_demo_source():
    """No source file can produce a row with ``source == "demo"``.

    Two files may still name it, and both are allowed for one reason each:
    the persistence layer's purge of the rows an older install left behind,
    and the fingerprint payload's ``is_demo`` flag, which is part of the
    response shape and is always false now.
    """
    allowed = {
        "services/persistence.py": "the purge of rows an older install left behind",
        "services/fingerprint.py": "the ``is_demo`` flag in the response shape",
    }
    offenders: dict[str, list[str]] = {}
    for path in _python_sources():
        relative = str(path.relative_to(APP_DIR))
        for line in path.read_text(encoding="utf-8").splitlines():
            if '"demo"' in line or "'demo'" in line:
                offenders.setdefault(relative, []).append(line.strip())
    unexpected = sorted(set(offenders) - set(allowed))
    assert unexpected == [], unexpected
    # Every file that names it is one of the two, and both do.
    assert set(offenders) == set(allowed), sorted(offenders)


def test_the_state_has_no_demo_flag():
    """The flag is not a field of the state any more."""
    from app.config import AppState, state

    assert "demo_visible" not in AppState.__dataclass_fields__
    assert not hasattr(state, "demo_visible")
    assert "demo_visible" not in state.public_status()


def test_the_state_file_does_not_carry_the_demo_flag():
    """The persisted state does not carry it either — it is not written."""
    from app.config import STATE_PATH, state

    state.save()
    payload = json.loads(STATE_PATH.read_text(encoding="utf-8"))
    assert "demo_visible" not in payload


def test_an_old_state_file_with_the_flag_loads_without_it():
    """A state file written by the previous version still loads, and the
    unknown key is dropped rather than resurrected."""
    from app.config import AppState, STATE_PATH

    original = STATE_PATH.read_text(encoding="utf-8") if STATE_PATH.exists() else None
    payload = json.loads(original) if original else {"schema_version": 1}
    payload["demo_visible"] = False
    STATE_PATH.write_text(json.dumps(payload), encoding="utf-8")
    try:
        loaded = AppState.load()
        assert not hasattr(loaded, "demo_visible")
        assert "demo_visible" not in loaded.public_status()
    finally:
        if original is None:
            STATE_PATH.unlink(missing_ok=True)
        else:
            STATE_PATH.write_text(original, encoding="utf-8")


def test_a_fresh_bootstrap_seeds_nothing(client, db):
    """Starting the app on an empty database leaves it empty."""
    from app.db import SessionLocal, init_db
    from app.models import Activity

    init_db()
    REAL_BOOTSTRAP()
    session = SessionLocal()
    try:
        assert session.query(Activity).count() == 0
    finally:
        session.close()


def test_a_bootstrap_removes_the_synthetic_activities_of_an_older_install(db):
    """The purge takes the rows and their children with it."""
    from app.models import Activity, ActivityLap
    from app.services.persistence import purge_synthetic_activities

    activity = Activity(
        source="demo",
        external_id="demo-1",
        name="Sample run",
        sport_type="running",
        local_date="2026-01-01",
    )
    db.add(activity)
    db.flush()
    db.add(ActivityLap(activity_id=activity.id, lap_index=1, distance_m=1000.0, duration_s=300.0))
    db.commit()

    assert purge_synthetic_activities(db) == 1
    assert db.query(Activity).count() == 0
    assert db.query(ActivityLap).count() == 0


def test_the_bootstrap_purges_on_every_start(client, db, monkeypatch):
    """The purge runs at startup, not once, so it cannot be forgotten."""
    from app.services import persistence

    calls: list[object] = []

    def fake(session):
        calls.append(session)
        return 0

    monkeypatch.setattr(persistence, "purge_synthetic_activities", fake)
    REAL_BOOTSTRAP()
    assert len(calls) == 1


def test_the_health_endpoint_counts_no_seeded_activities(client, db):
    """After a bootstrap the activity count is the count in the database."""
    from app import main
    from app.models import Activity

    main._bootstrap()
    response = client.get("/api/v1/health")
    assert response.status_code == 200
    assert response.json()["data"]["activities"] == db.query(Activity).count() == 0
