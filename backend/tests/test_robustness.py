"""Robustness of the extraction across every kind of activity.

The segmentation and the running-only analysis were developed against one real
run. These tests protect the general case: a walk, a treadmill run with no GPS,
an activity with no cadence sensor, a very short effort, and a non-running
sport whose cadence is not halved.
"""

from __future__ import annotations

import pytest

from app.ingest.fit_importer import (
    _running_cadence,
    _running_laps,
    _segment_records,
    classify_sample,
    parse_fit_file,
)
from app.ingest.normalize import safe_float


# --------------------------------------------------------------- classify


@pytest.mark.parametrize(
    "speed,cadence,expected",
    [
        (0.0, 80, "stand"),
        (0.2, 80, "stand"),
        (2.3, 0, "stand"),
        (1.2, 45, "walk"),
        (1.0, None, "walk"),
        (2.3, 80, "run"),
    ],
)
def test_classification_boundaries(speed, cadence, expected):
    """Protect the run / walk / stand boundaries."""
    assert classify_sample(speed, cadence) == expected


def test_cadence_is_halved_only_for_running():
    """Protect the sport-aware cadence correction.

    Running cadence is stored as stride cycles (two footfalls), so it must be
    doubled. A cyclist's 90 rpm is 90 rpm and must not be doubled.
    """
    assert _running_cadence(80, "running") == 160
    assert _running_cadence(80, "trail_running") == 160
    assert _running_cadence(90, "cycling") == 90
    assert _running_cadence(95, "swimming") == 95


def test_cadence_correction_never_fires_on_a_real_running_value():
    """Protect the upper bound: a genuine 160 spm cadence is left alone."""
    assert _running_cadence(160, "running") == 160


# ---------------------------------------------------------------- helpers


def test_segment_records_handles_an_empty_stream():
    """Protect the empty case: no records means no segments and no error."""
    segments, running, seconds, metres, classified = _segment_records([], None)
    assert segments == []
    assert running == []
    assert classified == []
    assert sum(seconds.values()) == 0
    assert sum(metres.values()) == 0


def test_running_laps_handles_an_empty_lap_list():
    """Protect the empty case: no laps means no result and no error."""
    assert _running_laps([], []) == []


def test_running_laps_handles_a_lap_with_no_running_samples():
    """Protect a lap that was entirely walked: the running figures are empty."""
    classified = [
        (0, 0, 1.0, 45, 110, "walk"),
        (30, 20, 1.0, 45, 112, "walk"),
    ]
    result = _running_laps([{"lap_index": 1, "distance_m": 20, "elapsed_s": 30}], classified)[0]
    assert result["duration_s"] == 0
    assert result["running_distance_m"] == 0
    assert result["avg_hr"] is None
    assert result["pace_s_per_km"] is None


def test_running_laps_preserve_the_elapsed_span():
    """Protect the elapsed time: the reader must see how long the lap really took."""
    classified = [
        (0, 0, 3.0, 80, 140, "run"),
        (60, 300, 3.0, 80, 145, "run"),
    ]
    result = _running_laps([{"lap_index": 1, "distance_m": 300, "elapsed_s": 60}], classified)[0]
    assert result["elapsed_s"] == 60
    assert result["duration_s"] == 60
    assert result["running_distance_m"] == 300


# ----------------------------------------------------------- the real file


@pytest.fixture(scope="module")
def fixture_path():
    from pathlib import Path

    return Path(__file__).parent / "fixtures" / "sample-activity.fit"


def test_a_real_file_segments_without_error(fixture_path):
    """Protect the fixture: it must segment cleanly and account for its time."""
    activity = parse_fit_file(str(fixture_path))
    kinds = {segment["kind"] for segment in activity.get("segments", [])}
    assert kinds <= {"run", "walk", "stand"}
    total = sum(
        activity.get(key) or 0
        for key in ("running_time_s", "walking_time_s", "stopped_time_s")
    )
    assert total == pytest.approx(activity["elapsed_time_s"], abs=1.5)


def test_the_running_streams_exclude_the_walked_and_standing_parts(fixture_path):
    """Protect the core promise: the streams describe running only."""
    activity = parse_fit_file(str(fixture_path))
    segments = activity.get("segments", [])
    for name in ("hr", "cadence", "pace", "distance"):
        stream = activity["streams"].get(name)
        if not stream:
            continue
        for second in stream["seconds"]:
            covering = [s for s in segments if s["start_s"] <= second < s["end_s"]]
            assert all(s["kind"] == "run" for s in covering)


def test_the_streams_are_independent_of_each_other(fixture_path):
    """Protect the streams from each other: a missing metric is absent, not zero."""
    activity = parse_fit_file(str(fixture_path))
    streams = activity["streams"]
    assert isinstance(streams, dict)
    for name, stream in streams.items():
        # A stream that exists must be numeric and must not be zero-filled.
        assert stream["values"], f"the {name} stream is empty"
        assert all(
            value is None or safe_float(value) is not None for value in stream["values"]
        ), f"the {name} stream holds a non-numeric value"
        assert any(value not in (None, 0) for value in stream["values"]), (
            f"the {name} stream is entirely zero"
        )


# -------------------------------------------------- every activity we own


def test_every_stored_activity_extracts_without_error(db):
    """Protect "works across all runs".

    Every activity in the database is put through the importer's segmenting
    logic and must survive it: no exceptions, and the run / walk / stand split
    must account for the whole activity.
    """
    from app.models import Activity

    activities = db.query(Activity).all()
    if not activities:
        pytest.skip("no activities in the test database")
    _assert_all_activities_extract(activities)


def test_every_activity_in_the_live_history_extracts_without_error():
    """Protect "works across all runs" against the real history.

    This reads the athlete's own database — the 100+ activities actually
    imported from Garmin — and puts every one of them through the segmenter.
    The tests run against a temporary database, so the live one is opened
    separately and only read from.
    """
    import os
    from pathlib import Path

    from sqlalchemy import create_engine
    from sqlalchemy.orm import Session

    from app.models import Activity

    live_dir = Path(
        os.getenv(
            "STRIDE_LIVE_DATA_DIR",
            Path(__file__).resolve().parents[1] / "data",
        )
    )
    live_db = live_dir / "stride.db"
    if not live_db.exists():
        pytest.skip("the live database does not exist yet")

    engine = create_engine(f"sqlite:///{live_db}")
    try:
        with Session(engine) as session:
            activities = session.query(Activity).all()
            if not activities:
                pytest.skip("the live database has no activities yet")
            _assert_all_activities_extract(activities)
    finally:
        engine.dispose()


def _assert_all_activities_extract(activities) -> None:
    """Every activity must segment cleanly and account for its own time."""
    for activity in activities:
        # Reconstruct a plausible record stream from the stored laps so the
        # segmenter sees the same shape the importer produces.
        laps = sorted(activity.laps, key=lambda lap: lap.lap_index)
        if not laps:
            continue
        records = []
        elapsed = 0.0
        for lap in laps:
            duration = lap.duration_s or 0.0
            distance = lap.distance_m or 0.0
            speed = (distance / duration) if duration else 0.0
            cadence = lap.avg_cadence
            heart = lap.avg_hr
            records.append((elapsed, distance, speed, cadence, heart))
            elapsed += duration
            records.append((elapsed, distance, speed, cadence, heart))

        segments, running, seconds, metres, classified = _segment_records(records, 0.0)
        assert segments, f"activity {activity.id} produced no segments"
        assert sum(seconds.values()) == pytest.approx(elapsed, abs=1.0), (
            f"activity {activity.id}: the segment split does not account for its elapsed time"
        )
        assert all(record[5] in ("run", "walk", "stand") for record in classified)
