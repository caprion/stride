"""The FIT import: any watch, and never the same run twice.

Two things are under test here.

**A file from a watch that is not a Garmin watch imports like any other
file.** The distance, the time and the heart rate are the file's, and the
watch the run is attributed to is the one the file names — not the one the
code was written against. The three files used here are the golden
fixtures: a Strava export, a COROS export and a Wahoo export.

**A run that is already in the history is not written twice.** The same
file imported twice is the same row, updated in place; and the run that was
synced from Garmin and the same run exported from the watch are one run,
because they are the same run under two identities.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from app.ingest.normalize import parse_iso

FIXTURES = Path(__file__).resolve().parent / "golden" / "fixtures"


def fixture(name: str) -> bytes:
    """The bytes of one golden fixture."""
    return (FIXTURES / name).read_bytes()


def import_file(client, name: str) -> dict[str, Any]:
    """Import one fixture through the route, and return its answer."""
    response = client.post(
        "/api/v1/auth/garmin/fit-import",
        files=[("files", (name, fixture(name), "application/octet-stream"))],
    )
    assert response.status_code == 200, response.text
    return response.json()


def rows(db) -> list[Any]:
    """Every activity in the database, oldest first."""
    from app.models import Activity

    return list(db.query(Activity).order_by(Activity.id).all())


def parsed(name: str) -> dict[str, Any]:
    """The file, as the reference importer reads it.

    The reference converts FIT timestamps through the *host* timezone, so
    the start time of a file is not the same number on every machine (the
    contract says so). The tests below need the run's own start time, so
    they ask the importer for it rather than writing one down.
    """
    from app.ingest.fit_importer import parse_fit

    return parse_fit(fixture(name), filename=name)


# ------------------------------------------------------- a watch, not Garmin


def test_a_strava_export_imports(client, db):
    """A Strava export is a run file like any other.

    Strava re-encodes the file it serves: its own manufacturer is in the
    ``file_id`` message and the watch that recorded the run is not in the
    file at all. So the run has no watch to name — and it must not borrow
    the name of a watch that is not in the file.
    """
    body = import_file(client, "vendor_strava_export.fit")
    assert body["imported"] == 1
    assert body["errors"] == []
    assert body["duplicates"] == []

    activity = rows(db)[0]
    assert activity.distance_m == 2040.0
    assert activity.moving_time_s == 600.0
    assert activity.avg_hr == 142
    assert activity.max_hr == 147
    assert activity.sport_type == "running"
    # The file names no watch, so the run names no watch.
    assert activity.device is None


def test_a_coros_export_names_its_own_watch(client, db):
    """A COROS export is attributed to the watch that wrote it."""
    body = import_file(client, "vendor_coros_export.fit")
    assert body["imported"] == 1

    activity = rows(db)[0]
    assert activity.distance_m == 1266.0
    assert activity.moving_time_s == 420.0
    assert activity.avg_hr == 135
    assert activity.max_hr == 141
    # The walking warm-up is a walk, and it is not running time.
    assert activity.walking_time_s == 60.0
    assert activity.running_time_s == 359.0
    # The watch is COROS's, read from the file.
    assert activity.device == "COROS DURA"
    assert "garmin" not in (activity.device or "").lower()


def test_a_wahoo_export_names_its_own_watch(client, db):
    """A Wahoo export is attributed to the watch that wrote it.

    Wahoo writes the manufacturer and no product name, so the watch is
    named by its manufacturer alone.
    """
    body = import_file(client, "vendor_wahoo_export.fit")
    assert body["imported"] == 1

    activity = rows(db)[0]
    assert activity.distance_m == 1920.0
    assert activity.moving_time_s == 600.0
    assert activity.avg_hr == 147
    assert activity.max_hr == 152
    assert activity.device == "wahoo_fitness"
    assert "garmin" not in (activity.device or "").lower()


# ------------------------------------------------------------- the same run


def test_the_same_file_twice_is_one_run(client, db):
    """The same file imported twice is the same row, updated in place."""
    first = import_file(client, "mixed_run.fit")
    assert first["imported"] == 1
    assert first["activities"][0]["created"] is True

    second = import_file(client, "mixed_run.fit")
    assert second["imported"] == 1
    assert second["activities"][0]["id"] == first["activities"][0]["id"]
    assert second["activities"][0]["created"] is False
    # It is not a second run, so it is not a duplicate either.
    assert second["duplicates"] == []

    assert len(rows(db)) == 1


def test_a_synced_run_and_its_file_are_one_run(client, db):
    """The run Garmin synced and the file exported from the watch are one run.

    They are the same run under two identities: the Garmin activity has
    Garmin's id, the file has the watch's. The history holds it once, and
    the import says the run is already there.
    """
    from app.services.persistence import store_activity

    activity = parsed("mixed_run.fit")
    synced, _ = store_activity(
        db,
        {
            "source": "garmin",
            "external_id": "16786616153",
            "name": "mixed run",
            "sport_type": "running",
            "start_time": activity["start_time"],
            "local_date": activity["local_date"],
            "day_of_week": activity["day_of_week"],
            "distance_m": activity["distance_m"],
            "moving_time_s": activity["moving_time_s"],
            "elapsed_time_s": activity["elapsed_time_s"],
            "avg_hr": activity["avg_hr"],
            "max_hr": activity["max_hr"],
            "running_time_s": activity["running_time_s"],
            "walking_time_s": activity["walking_time_s"],
            "stopped_time_s": activity["stopped_time_s"],
            "running_distance_m": activity["running_distance_m"],
            "device": "Forerunner 265",
        },
    )
    db.commit()

    body = import_file(client, "mixed_run.fit")
    assert body["imported"] == 0
    assert body["activities"] == []
    assert len(body["duplicates"]) == 1
    duplicate = body["duplicates"][0]
    assert duplicate["filename"] == "mixed_run.fit"
    assert duplicate["id"] == synced.id
    assert duplicate["name"] == "mixed run"
    assert duplicate["date"] == activity["local_date"]

    # One run in the history: the one Garmin delivered.
    assert len(rows(db)) == 1
    assert rows(db)[0].id == synced.id
    assert rows(db)[0].source == "garmin"


def test_a_different_run_is_still_imported(client, db):
    """Two runs of the same week are two runs."""
    first = import_file(client, "mixed_run.fit")
    second = import_file(client, "hr_gps_run.fit")
    assert first["imported"] == 1
    assert second["imported"] == 1
    assert second["duplicates"] == []
    assert second["activities"][0]["id"] != first["activities"][0]["id"]

    stored = rows(db)
    assert len(stored) == 2
    assert stored[0].distance_m == 443.0
    assert stored[1].distance_m == 1980.0


def test_a_run_of_another_day_is_not_a_duplicate(client, db):
    """The window is two minutes; a run an hour earlier is another run."""
    from datetime import timedelta

    from app.services.persistence import store_activity

    activity = parsed("mixed_run.fit")
    start = parse_iso(activity["start_time"])
    store_activity(
        db,
        {
            "source": "garmin",
            "external_id": "16786616154",
            "name": "an earlier run",
            "sport_type": "running",
            "start_time": (start - timedelta(hours=1)).isoformat(),
            "local_date": activity["local_date"],
            "distance_m": activity["distance_m"],
            "moving_time_s": activity["moving_time_s"],
        },
    )
    db.commit()

    body = import_file(client, "mixed_run.fit")
    assert body["imported"] == 1
    assert body["duplicates"] == []
    assert len(rows(db)) == 2
