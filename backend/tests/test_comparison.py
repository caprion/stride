"""Tests for the run comparison engine.

The comparison view is where the athlete reads two runs against each other, so
the guarantees that matter are: the profile is sampled on a shared distance
axis, a missing metric is missing rather than zero, and a run that cannot be
loaded is reported rather than silently dropped.
"""

from __future__ import annotations

from datetime import date, timedelta

import pytest

from app.models import Activity, ActivityStream
from app.services import comparison


def _make_activity(
    db,
    *,
    activity_id: int = 9001,
    name: str = "Test Run",
    when: str = "2026-01-01",
    distance_m: float = 10000.0,
    moving_time_s: float = 3000.0,
    samples: int = 10,
    with_cadence: bool = True,
    with_pace: bool = True,
    with_elevation: bool = True,
    with_distance: bool = True,
) -> Activity:
    """An activity whose streams share one sample axis, as the importer produces."""
    activity = Activity(
        id=activity_id,
        source="test",
        external_id=str(activity_id),
        name=name,
        sport_type="running",
        local_date=when,
        start_time=date.fromisoformat(when),
        distance_m=distance_m,
        moving_time_s=moving_time_s,
        elapsed_time_s=moving_time_s + 60,
        elevation_gain_m=100.0,
        avg_hr=150.0,
        max_hr=170.0,
    )
    db.add(activity)
    db.flush()

    seconds = [index * 10 for index in range(samples)]
    distance = [distance_m * index / max(1, samples - 1) for index in range(samples)]
    heart_rate = [120.0 + index * 5 for index in range(samples)]

    def add_stream(kind: str, values: list[float]) -> None:
        db.add(
            ActivityStream(
                activity_id=activity.id,
                kind=kind,
                seconds=list(seconds),
                values=list(values),
                unit="test",
            )
        )

    if with_distance:
        add_stream("distance", distance)
    add_stream("hr", heart_rate)
    if with_cadence:
        add_stream("cadence", [160.0] * samples)
    if with_pace:
        add_stream("pace", [300.0] * samples)
    if with_elevation:
        add_stream("elevation", [900.0 + index for index in range(samples)])
    db.flush()
    return activity


# ------------------------------------------------------------- profile_for


def test_profile_has_the_expected_number_of_points(db):
    """Protect the profile length — the overlay chart is drawn from it."""
    activity = _make_activity(db)
    profile = comparison.profile_for(activity)
    assert len(profile) == comparison.PROFILE_POINTS


def test_profile_runs_from_zero_to_one_hundred_percent(db):
    """Protect the normalised axis so runs of different lengths overlay."""
    activity = _make_activity(db)
    profile = comparison.profile_for(activity)
    assert profile[0]["pct"] == 0
    assert profile[-1]["pct"] == 100


def test_profile_spans_the_recorded_distance(db):
    """Protect the distance axis — it is what the runs are compared on."""
    activity = _make_activity(db, distance_m=8000.0)
    profile = comparison.profile_for(activity)
    assert profile[0]["distance_m"] == pytest.approx(0.0, abs=0.5)
    assert profile[-1]["distance_m"] == pytest.approx(8000.0, abs=1.0)


def test_profile_interpolates_the_streams(db):
    """Protect the interpolation: a value halfway along must be halfway too."""
    activity = _make_activity(db, samples=11, distance_m=10000.0)
    profile = comparison.profile_for(activity)
    # Heart rate runs 120 -> 170 across the run; the midpoint must be near 145.
    middle = profile[len(profile) // 2]
    assert middle["hr"] == pytest.approx(145.0, abs=4.0)


def test_profile_leaves_an_absent_metric_as_none(db):
    """Protect a missing metric: null, never zero — zero would draw a false line."""
    activity = _make_activity(db, with_cadence=False)
    profile = comparison.profile_for(activity)
    assert profile
    assert all(point["cadence"] is None for point in profile)
    assert all(point["hr"] is not None for point in profile)


def test_profile_is_empty_without_a_distance_stream(db):
    """Protect the degenerate case: no distance stream means no profile, not an error."""
    activity = _make_activity(db, with_distance=False)
    assert comparison.profile_for(activity) == []


def test_profile_is_empty_with_a_single_distance_sample(db):
    """Protect the single-sample case: one point cannot define an axis."""
    activity = _make_activity(db, samples=1)
    assert comparison.profile_for(activity) == []


def test_profile_is_empty_without_streams(db):
    """Protect an activity that carries no streams at all."""
    activity = _make_activity(db)
    db.query(ActivityStream).filter(ActivityStream.activity_id == activity.id).delete()
    db.flush()
    db.refresh(activity)
    assert comparison.profile_for(activity) == []


# ------------------------------------------------------------------ compare


def test_compare_returns_one_entry_per_run(db):
    """Protect the run list: one entry per requested activity."""
    first = _make_activity(db, activity_id=9101, when="2026-02-01")
    second = _make_activity(db, activity_id=9102, when="2026-01-01")
    result = comparison.compare(db, [first.id, second.id])
    assert len(result["runs"]) == 2
    # Sorted oldest first, so the comparison reads chronologically.
    assert result["runs"][0]["local_date"] == "2026-01-01"


def test_compare_reports_missing_ids(db):
    """Protect the missing list: an id that cannot be loaded must be reported."""
    activity = _make_activity(db, activity_id=9103)
    result = comparison.compare(db, [activity.id, 999999])
    assert len(result["runs"]) == 1
    assert result["missing"] == [999999]


def test_compare_values_cover_every_parameter(db):
    """Protect the matrix: every declared parameter has a value slot."""
    activity = _make_activity(db, activity_id=9104)
    result = comparison.compare(db, [activity.id])
    values = result["runs"][0]["values"]
    for parameter in result["parameters"]:
        assert parameter["key"] in values


def test_compare_without_a_fingerprint_leaves_those_values_null(db):
    """Protect the no-fingerprint case: the matrix still shows the run."""
    activity = _make_activity(db, activity_id=9105)
    result = comparison.compare(db, [activity.id])
    values = result["runs"][0]["values"]
    assert values["form_score"] is None
    assert values["decoupling_pct"] is None


def test_candidates_lists_the_activity(db):
    """Protect the picker's source list."""
    activity = _make_activity(db, activity_id=9106)
    ids = [item["id"] for item in comparison.candidates(db)]
    assert activity.id in ids


# --------------------------------------------------------------------- api


def test_compare_candidates_endpoint(client):
    """Protect the candidates endpoint's shape."""
    response = client.get("/api/v1/compare/candidates")
    assert response.status_code == 200
    assert "items" in response.json()


def test_compare_endpoint_returns_runs_and_parameters(client):
    """Protect the comparison endpoint's shape."""
    response = client.get("/api/v1/compare?ids=1,2")
    assert response.status_code == 200
    payload = response.json()
    assert "runs" in payload
    assert "parameters" in payload
    assert "missing" in payload


def test_compare_endpoint_tolerates_empty_ids(client):
    """Protect the lenient id parsing: no ids is an empty comparison, not a 422."""
    response = client.get("/api/v1/compare?ids=")
    assert response.status_code == 200
    assert response.json()["runs"] == []


def test_compare_endpoint_tolerates_non_numeric_ids(client):
    """Protect the lenient id parsing: rubbish ids are ignored, not fatal."""
    response = client.get("/api/v1/compare?ids=abc,def")
    assert response.status_code == 200
    assert response.json()["runs"] == []


def test_compare_endpoint_caps_the_number_of_runs(client):
    """Protect the cap: a comparison of twenty runs would be unreadable."""
    ids = ",".join(str(index) for index in range(1, 21))
    response = client.get(f"/api/v1/compare?ids={ids}")
    assert response.status_code == 200
    assert len(response.json()["runs"]) <= 8


# ------------------------------------------------------------------- basis


def test_the_basis_of_a_comparison_follows_the_plan_intent():
    """Protect the basis: like for like, different jobs, or distance."""
    assert comparison.comparison_basis(["easy", "easy"]) == {
        "kind": "plan_session",
        "session_type": "easy",
    }
    assert comparison.comparison_basis(["easy", "tempo"]) == {"kind": "different_jobs"}
    assert comparison.comparison_basis(["tempo", None]) == {"kind": "distance"}
    assert comparison.comparison_basis([None, None]) == {"kind": "distance"}
    assert comparison.comparison_basis([]) == {"kind": "distance"}


def test_a_rest_or_a_strength_session_gives_a_run_no_intent():
    """Protect the intent: only a run's session says what the run was for."""
    assert comparison.plan_intent("easy") == "easy"
    assert comparison.plan_intent("Intervals") == "interval"
    assert comparison.plan_intent("long_run") == "long"
    assert comparison.plan_intent("threshold") == "tempo"
    assert comparison.plan_intent("rest") is None
    assert comparison.plan_intent("strength") is None
    assert comparison.plan_intent(None) is None


def test_two_runs_planned_as_the_same_session_are_compared_like_for_like(db):
    """Protect the plan-matched case: the basis is the session, not the distance."""
    from app.models_phase2 import PlanBlock, PlanSession

    first = _make_activity(db, activity_id=9201, when="2026-02-01")
    second = _make_activity(db, activity_id=9202, when="2026-02-03")
    db.add(
        PlanBlock(
            id=920,
            name="Block",
            status="active",
            start_date="2026-02-01",
            end_date="2026-02-28",
        )
    )
    for index, when in enumerate(("2026-02-01", "2026-02-03"), start=1):
        db.add(
            PlanSession(
                id=920 + index,
                block_id=920,
                date=when,
                title="Easy",
                session_type="easy",
                target_distance_m=10000.0,
            )
        )
    db.flush()
    # The plan engine reads the start time as a timestamp, so the rows are
    # read back rather than taken from the objects just built.
    db.refresh(first)
    db.refresh(second)

    result = comparison.compare(db, [first.id, second.id])
    assert result["basis"] == {"kind": "plan_session", "session_type": "easy"}
    assert [run["plan_session_type"] for run in result["runs"]] == ["easy", "easy"]


def test_two_runs_planned_as_different_sessions_are_not_compared(db):
    """Protect the different-jobs case: nothing may be marked as strongest."""
    from app.models_phase2 import PlanBlock, PlanSession

    first = _make_activity(db, activity_id=9203, when="2026-02-01")
    second = _make_activity(db, activity_id=9204, when="2026-02-03")
    db.add(
        PlanBlock(
            id=921,
            name="Block",
            status="active",
            start_date="2026-02-01",
            end_date="2026-02-28",
        )
    )
    db.add(
        PlanSession(
            id=921,
            block_id=921,
            date="2026-02-01",
            title="Easy",
            session_type="easy",
            target_distance_m=10000.0,
        )
    )
    db.add(
        PlanSession(
            id=922,
            block_id=921,
            date="2026-02-03",
            title="Tempo",
            session_type="tempo",
            target_distance_m=10000.0,
        )
    )
    db.flush()
    db.refresh(first)
    db.refresh(second)

    result = comparison.compare(db, [first.id, second.id])
    assert result["basis"] == {"kind": "different_jobs"}


def test_a_run_without_a_plan_session_falls_back_to_distance(db):
    """Protect the fallback: no plan intent means distance is the proxy."""
    activity = _make_activity(db, activity_id=9205)
    result = comparison.compare(db, [activity.id])
    assert result["basis"] == {"kind": "distance"}
    assert result["runs"][0]["plan_session_type"] is None
