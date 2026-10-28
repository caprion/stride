"""Shared fixtures for the backend test suite.

Each test gets a fresh SQLite database, and every test runs with the network
blocked — nothing here may reach Garmin.
"""

from __future__ import annotations

import os
import socket
import tempfile

import pytest

# Configure storage before importing any app module.
_TMP = tempfile.TemporaryDirectory(prefix="stride-tests-")
os.environ.setdefault("STRIDE_DATA_DIR", _TMP.name)
# Tests must never reach out to Azure Key Vault.
os.environ["STRIDE_KEYVAULT_ENABLED"] = "false"

from sqlalchemy import create_engine  # noqa: E402
from sqlalchemy.orm import Session  # noqa: E402

from app.db import Base, get_db  # noqa: E402

# Import the models so the metadata knows every table before create_all runs.
from app import models, models_phase2  # noqa: E402,F401


@pytest.fixture(autouse=True)
def offline(request, monkeypatch):
    """Fail on accidental network access, including inside provider wrappers.

    A test that genuinely needs the network opts out with
    ``@pytest.mark.network``.
    """
    if request.node.get_closest_marker("network"):
        yield
        return
    attempts = []

    def blocked(*args, **kwargs):
        attempts.append(args)
        raise AssertionError("Network access is forbidden in the test suite")

    monkeypatch.setattr(socket.socket, "connect", blocked)
    monkeypatch.setattr(socket.socket, "connect_ex", blocked)
    yield
    assert not attempts, "a test attempted a network connection"


@pytest.fixture()
def db(tmp_path):
    """Give each test a fresh database, including API commits."""
    engine = create_engine(
        f"sqlite:///{tmp_path / 'test.db'}",
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
    # The state is a module-level singleton, so anything a test writes into it
    # would otherwise leak into the next test.
    monkeypatch.setattr(state, "devices", [])
    monkeypatch.setattr(state, "pairing_code", None)
    monkeypatch.setattr(state, "pairing_expires_at", None)
    previous = main.app.dependency_overrides.copy()
    main.app.dependency_overrides[get_db] = database
    try:
        with TestClient(main.app) as test_client:
            yield test_client
    finally:
        main.app.dependency_overrides.clear()
        main.app.dependency_overrides.update(previous)


def pytest_configure(config):
    """Register the marker that lets a test reach the network."""
    config.addinivalue_line(
        "markers", "network: this test genuinely talks to the network"
    )
