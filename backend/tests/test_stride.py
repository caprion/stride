"""Backend test suite.

Run with:

    cd ~/stride/backend && .venv/bin/python -m pytest -q

The tests use a temporary SQLite database so they never touch the real one.
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

import pytest

# Point the app at a throwaway database before anything imports it.
_TMP = tempfile.mkdtemp(prefix="stride-tests-")
os.environ["STRIDE_DATA_DIR"] = _TMP

from app.config import DB_PATH  # noqa: E402
from app.db import Base, SessionLocal, engine  # noqa: E402


@pytest.fixture(scope="session", autouse=True)
def _database():
    Base.metadata.create_all(engine)
    yield
    Base.metadata.drop_all(engine)


@pytest.fixture()
def client():
    from fastapi.testclient import TestClient

    from app.main import app

    with TestClient(app) as test_client:
        yield test_client


@pytest.fixture()
def db():
    session = SessionLocal()
    try:
        yield session
    finally:
        session.close()


# ------------------------------------------------------------------ helpers


def test_polyline_roundtrip():
    """A known encoded polyline decodes to the expected coordinates."""
    from app.ingest.normalize import decode_polyline

    # "_p~iF~ps|U_ulLnnqC_mqNvxq`@" is the canonical Google example.
    points = decode_polyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@")
    assert len(points) == 3
    assert points[0] == [38.5, -120.2]
    assert points[1] == [40.7, -120.95]
    assert points[2] == [43.252, -126.453]


def test_format_pace():
    from app.ingest.normalize import format_pace

    assert format_pace(379.4) == "6:19"
    assert format_pace(60) == "1:00"
    assert format_pace(None) is None
    assert format_pace(0) is None


def test_normalise_sport():
    from app.ingest.normalize import normalise_sport

    assert normalise_sport("running") == "running"
    assert normalise_sport("trail_running") == "trail_running"
    assert normalise_sport("street_running") == "running"
    assert normalise_sport(None) == "running"


def test_parse_iso_handles_garmin_formats():
    from app.ingest.normalize import parse_iso

    assert parse_iso("2026-01-04 07:22:33") is not None
    assert parse_iso("2026-01-04T07:22:33.0") is not None
    assert parse_iso("2026-01-04") is not None
    assert parse_iso("") is None
    assert parse_iso(None) is None


def test_duration_to_seconds_handles_clock_strings():
    from app.ingest.normalize import duration_to_seconds

    assert duration_to_seconds("00:45:30") == 2730
    assert duration_to_seconds(120.5) == 120.5
    assert duration_to_seconds(None) == 0.0


def test_week_key():
    from app.ingest.normalize import week_key

    assert week_key("2026-01-04") == "2026-W01"
    assert week_key("nonsense") == ""


# --------------------------------------------------------------------- fit


def test_fit_import_extracts_run_data():
    """A real Garmin FIT file decodes into a canonical activity."""
    from app.ingest.fit_importer import parse_fit_file

    fixture = Path(__file__).parent / "fixtures" / "sample-activity.fit"
    activity = parse_fit_file(str(fixture))

    assert activity["source"] == "garmin_fit"
    assert activity["sport_type"] == "running"
    assert activity["distance_m"] > 0
    assert activity["moving_time_s"] > 0
    assert activity["local_date"]
    # The fixture carries a GPS track.
    assert activity["geo_points"]
    assert len(activity["geo_points"]) > 2


def test_fit_import_rejects_garbage():
    from app.ingest.fit_importer import parse_fit

    with pytest.raises(ValueError):
        parse_fit(b"this is not a fit file", filename="bad.fit")


# -------------------------------------------------------------------- api


def test_health(client):
    response = client.get("/api/v1/health")
    assert response.status_code == 200
    assert response.json()["ok"] is True


def test_auth_status(client):
    response = client.get("/api/v1/auth/status")
    assert response.status_code == 200
    payload = response.json()
    assert "garmin_connected" in payload
    assert "network_route" in payload


def test_developer_key_rejects_short_input(client):
    response = client.post(
        "/api/v1/auth/garmin/developer-key",
        json={"consumer_key": "abc", "consumer_secret": "xyz"},
    )
    assert response.status_code == 422


def test_developer_key_reports_invalid_key(client):
    """A bogus key must produce a clear 400, not a stack trace."""
    response = client.post(
        "/api/v1/auth/garmin/developer-key",
        json={"consumer_key": "00000000-0000-0000-0000-000000000000", "consumer_secret": "x" * 20},
    )
    assert response.status_code == 400
    assert "consumer key" in response.json()["detail"].lower()


def test_settings_patch_persists(client):
    response = client.patch("/api/v1/sync/settings", json={"sync_days": 120})
    assert response.status_code == 200
    assert response.json()["sync_days"] == 120
    client.patch("/api/v1/sync/settings", json={"sync_days": 90})


def test_activity_listing_shape(client):
    response = client.get("/api/v1/activities", params={"per_page": 5})
    assert response.status_code == 200
    payload = response.json()
    assert set(payload) >= {"items", "total", "page", "per_page", "pages"}
    for item in payload["items"]:
        assert set(item) >= {
            "id",
            "name",
            "sport_type",
            "local_date",
            "distance_m",
            "moving_time_s",
            "pace_label",
        }


def test_activity_detail_shape(client):
    listing = client.get("/api/v1/activities", params={"per_page": 1}).json()
    if not listing["items"]:
        pytest.skip("no activities in the test database")
    activity_id = listing["items"][0]["id"]

    response = client.get(f"/api/v1/activities/{activity_id}")
    assert response.status_code == 200
    detail = response.json()
    assert "laps" in detail
    assert "streams" in detail
    assert "geo_points" in detail


def test_activity_detail_404(client):
    assert client.get("/api/v1/activities/99999999").status_code == 404


def test_stats_summary_shape(client):
    response = client.get("/api/v1/stats/summary")
    assert response.status_code == 200
    payload = response.json()
    assert set(payload) >= {
        "totals",
        "this_year",
        "this_month",
        "this_week",
        "personal_records",
        "streaks",
        "sport_breakdown",
    }


def test_calendar_shape(client):
    response = client.get("/api/v1/calendar", params={"year": 2026, "month": 1})
    assert response.status_code == 200
    payload = response.json()
    assert payload["year"] == 2026
    assert len(payload["days"]) in (28, 29, 30, 31)


def test_network_status_shape(client):
    response = client.get("/api/v1/network/status")
    assert response.status_code == 200
    assert "tailscale_available" in response.json()
