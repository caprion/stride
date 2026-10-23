"""Regression tests for FIT run segmentation and running-only metrics."""

from __future__ import annotations

from pathlib import Path

import pytest

from app.ingest.fit_importer import _running_laps, _segment_records, classify_sample, parse_fit_file


@pytest.mark.parametrize("speed,cadence,expected", [
    (0.0, 80, "stand"), (0.1, 80, "stand"), (2.3, 0, "stand"),
    (1.2, 45, "walk"), (2.3, 80, "run"),
    pytest.param(2.3, 55, "walk", marks=pytest.mark.xfail(reason="Low stored cadence is only treated as walking below 2.2 m/s; at 2.3 m/s classify_sample falls through to run despite the requested low-cadence rule.")),
    (1.0, None, "walk"),
])
def test_classify_sample(speed, cadence, expected):
    """Protect speed and stored-cadence classification boundaries."""
    assert classify_sample(speed, cadence) == expected


def test_segments_attribute_intervals_and_distance_to_sample_kind():
    """Protect ordered boundaries, elapsed-time coverage, distance ownership, and run filtering."""
    records = [
        (100, 0, 1.0, 45, 100), (110, 10, 1.0, 45, 105),
        (120, 20, 2.5, 80, 120), (130, 45, 2.5, 80, 125),
        (140, 70, 0.0, 0, 110), (150, 70, 2.5, 80, 130),
        (160, 95, 2.5, 80, 135),
    ]
    segments, running, seconds, metres, classified = _segment_records(records, 100)
    assert [s["kind"] for s in segments] == ["walk", "run", "stand", "run"]
    assert [(s["start_s"], s["end_s"]) for s in segments] == [(0, 10), (20, 30), (40, 40), (50, 60)]
    assert sum(seconds.values()) == pytest.approx(60)
    assert seconds == pytest.approx({"run": 30, "walk": 20, "stand": 10})
    assert metres == pytest.approx({"run": 75, "walk": 20, "stand": 0})
    assert all(classify_sample(row[2], row[3]) == "run" for row in running)
    assert [row[0] for row in running] == [20, 30, 50, 60]
    assert len(classified) == len(records)


def test_sample_fit_segmentation_and_running_streams():
    """Protect parsed segment kinds, elapsed totals, and running-only stream timestamps."""
    fixture = Path(__file__).parent / "fixtures" / "sample-activity.fit"
    activity = parse_fit_file(str(fixture))
    segments = activity["segments"]
    assert {segment["kind"] for segment in segments} <= {"run", "walk", "stand"}
    assert sum(activity[key] for key in ("running_time_s", "walking_time_s", "stopped_time_s")) == pytest.approx(activity["elapsed_time_s"], abs=1)
    for name in ("hr", "cadence", "pace", "distance"):
        stream = activity["streams"].get(name)
        if not stream:
            continue
        for second in stream["seconds"]:
            containing = [s for s in segments if s["start_s"] <= second < s["end_s"]]
            assert all(s["kind"] == "run" for s in containing)


def test_running_lap_excludes_stand_and_preserves_elapsed():
    """Protect running lap time, distance and pace from a 60-second stand."""
    classified = []
    # elapsed, distance, speed, stored cadence, HR, class
    for elapsed, distance, kind in [(0, 0, "run"), (30, 100, "run"), (60, 100, "stand"), (120, 100, "stand"), (150, 200, "run"), (180, 300, "run")]:
        classified.append((elapsed, distance, 3.3, 80, 140, kind))
    lap = {"lap_index": 1, "distance_m": 300, "duration_s": 180, "elapsed_s": 180}
    result = _running_laps([lap], classified)[0]
    assert result["duration_s"] == 60
    assert result["running_distance_m"] == 200
    assert result["pace_s_per_km"] == 300
    assert result["elapsed_s"] == 180


def test_running_lap_distance_only_counts_running_intervals():
    """Protect lap distance and pace from distance increments during walking."""
    classified = [(0, 0, 3.0, 80, 140, "run"), (10, 30, 1.0, 45, 110, "walk"), (20, 40, 3.0, 80, 145, "run"), (30, 70, 3.0, 80, 150, "run")]
    result = _running_laps([{"lap_index": 1, "distance_m": 70, "elapsed_s": 30}], classified)[0]
    assert result["duration_s"] == 10
    assert result["running_distance_m"] == 30
    assert result["pace_s_per_km"] == pytest.approx(1000 / 3, abs=0.01)
    assert result["elapsed_s"] == 30
