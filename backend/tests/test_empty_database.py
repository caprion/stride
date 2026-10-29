"""The empty database: the first thing every new installation has.

An athlete who has never connected Garmin opens the app and every endpoint
has to answer. Not with an error, not with a null-dereference, not with a
division by zero — with a well-formed body that says "there is nothing here
yet" and means it.

This file is the empty-database contract. Each test hits one surface and
asserts the shape of the answer, so a regression is a named failure rather
than a screenshot of a stack trace.
"""

from __future__ import annotations

from datetime import date, timedelta

import pytest


@pytest.fixture()
def empty(client):
    """The client, with a database that holds nothing at all."""
    return client


def _body(response):
    assert response.status_code == 200, response.text
    return response.json()


# ------------------------------------------------------------------ health


def test_health_reports_zero_of_everything(empty):
    """The health endpoint counts rows; with no rows every count is zero."""
    body = _body(empty.get("/api/v1/health"))
    assert body["ok"] is True
    assert body["data"]["activities"] == 0
    assert body["data"]["from_garmin"] == 0
    assert body["data"]["physiology_days"] == 0
    assert body["data"]["plan_sessions"] == 0


def test_auth_status_has_no_demo_flag(empty):
    """The auth status payload carries no demo flag any more."""
    body = _body(empty.get("/api/v1/auth/status"))
    assert "demo_visible" not in body
    assert body["garmin_connected"] is False
    assert body["has_personal_api_key"] is False


# --------------------------------------------------------------- activities


def test_the_activity_list_is_empty_and_well_formed(empty):
    """An empty list: no items, a zero total, and a page count that is not an error."""
    body = _body(empty.get("/api/v1/activities"))
    assert body["items"] == []
    assert body["total"] == 0
    assert body["page"] == 1
    assert body["per_page"] == 20
    # One page of nothing: the same convention the desktop core uses.
    assert body["pages"] == 1


def test_the_activity_list_accepts_a_page_past_the_end(empty):
    """Asking for page 5 of nothing is empty, not an error."""
    body = _body(empty.get("/api/v1/activities?page=5&per_page=50"))
    assert body["items"] == []
    assert body["total"] == 0
    assert body["page"] == 5


def test_a_missing_activity_is_a_404(empty):
    """An activity that does not exist is a 404, not a 500."""
    response = empty.get("/api/v1/activities/1")
    assert response.status_code == 404
    assert "detail" in response.json()


def test_the_activity_filters_are_harmless_when_there_is_nothing(empty):
    """Every filter on the list is safe against an empty table."""
    for query in (
        "?sport=running",
        "?from=2026-01-01&to=2026-12-31",
        "?q=morning",
        "?sort=pace",
        "?sort=distance",
        "?sort=elevation",
    ):
        body = _body(empty.get("/api/v1/activities" + query))
        assert body["items"] == []
        assert body["total"] == 0


# -------------------------------------------------------------------- stats


def test_the_summary_reports_zero_totals_and_no_records(empty):
    """The dashboard summary: zeros, no records, no streaks."""
    body = _body(empty.get("/api/v1/stats/summary"))
    assert body["totals"]["activity_count"] == 0
    assert body["totals"]["distance_m"] == 0
    assert body["totals"]["moving_time_s"] == 0
    assert body["totals"]["avg_pace_s_per_km"] is None
    assert body["totals"]["avg_pace_label"] is None
    for bucket in ("this_year", "this_month", "this_week"):
        assert body[bucket]["activity_count"] == 0
        assert body[bucket]["distance_m"] == 0
    assert body["personal_records"] == []
    assert body["streaks"]["current_weeks"] == 0
    assert body["streaks"]["longest_weeks"] == 0
    assert body["streaks"]["total_active_weeks"] == 0
    assert body["sport_breakdown"] == []


def test_the_stats_endpoints_are_empty_lists(empty):
    """Every stats trend is an empty list with its own envelope intact."""
    for path, key in (
        ("/api/v1/stats/weekly", "items"),
        ("/api/v1/stats/monthly", "items"),
        ("/api/v1/stats/pace-trends", "items"),
        ("/api/v1/stats/personal-records", "items"),
        ("/api/v1/stats/elevation", "items"),
    ):
        body = _body(empty.get(path))
        assert body[key] == [], path


def test_the_heatmap_has_a_year_and_no_days(empty):
    """The heatmap names the year it is empty for."""
    body = _body(empty.get("/api/v1/stats/heatmap"))
    assert body["year"] == date.today().year
    assert body["days"] == []


def test_the_heart_rate_zones_are_zeroed_not_missing(empty):
    """The zone model still exists — five zones, all at zero minutes."""
    body = _body(empty.get("/api/v1/stats/hr-zones"))
    assert len(body["zones"]) == 5
    assert all(zone["time_s"] == 0 for zone in body["zones"])
    assert all(zone["percent"] == 0 for zone in body["zones"])
    assert body["total_time_s"] == 0


# ----------------------------------------------------------------- calendar


def test_the_calendar_has_every_day_and_no_activities(empty):
    """The month is laid out, day by day, with nothing on it.

    The day count is not asserted exactly: the reference's day loop is
    off by one (it emits one day fewer than the month has, and 29 February
    on a year that has no 29 February). The Rust port reproduces that
    deliberately and the golden fixture agrees with it, so fixing it here
    alone would make the two hosts disagree. It is reported instead.
    """
    today = date.today()
    body = _body(empty.get("/api/v1/calendar"))
    assert body["year"] == today.year
    assert body["month"] == today.month
    assert body["activity_count"] == 0
    assert body["total_distance_m"] == 0
    assert len(body["days"]) >= 28
    for day in body["days"]:
        assert day["activities"] == []
        assert day["distance_m"] == 0
        assert day["moving_time_s"] == 0
    assert [day["day"] for day in body["days"]] == list(
        range(1, len(body["days"]) + 1)
    )


# --------------------------------------------------------------- physiology


def test_the_physiology_daily_series_are_empty(empty):
    """A year of wellness with nothing in it is an empty list, not an error."""
    for path in ("/api/v1/physiology/daily", "/api/v1/physiology/sleep"):
        body = _body(empty.get(path))
        assert body["items"] == [], path


def test_the_physiology_hrv_series_is_empty_and_has_no_baseline(empty):
    """HRV with no rows has no baseline — and does not invent one."""
    body = _body(empty.get("/api/v1/physiology/hrv"))
    assert body["items"] == []
    assert body["baseline"] is None


def test_the_latest_physiology_day_is_none(empty):
    """There is no latest day, and saying so is a null, not a crash."""
    body = _body(empty.get("/api/v1/physiology/latest"))
    assert body["day"] is None
    assert body["readiness"] is None


def test_the_readiness_series_invents_no_score(empty):
    """Readiness may report the missing days; it must not score them."""
    body = _body(empty.get("/api/v1/physiology/readiness"))
    assert isinstance(body["items"], list)
    for item in body["items"]:
        assert item["score"] is None
        assert "date" in item


def test_the_physiology_sources_say_nothing_is_available(empty):
    """The sources endpoint says what is missing and why."""
    body = _body(empty.get("/api/v1/physiology/sources"))
    assert body["available"] is False
    assert body["days_available"] == 0
    assert body["last_synced_at"] is None
    assert body["reason"]
    assert all(endpoint["available"] is False for endpoint in body["endpoints"])


# --------------------------------------------------------------------- plan


def test_there_is_no_plan_and_the_plan_endpoints_say_so(empty):
    """No plan: the blocks list is empty and the current block is null."""
    assert _body(empty.get("/api/v1/plan/blocks"))["items"] == []
    current = _body(empty.get("/api/v1/plan/current"))
    assert current["block"] is None
    assert current["week"] is None


def test_the_plan_today_has_no_session_and_says_there_is_no_plan(empty):
    """Today's plan on an empty database: no session, no plan, no advice."""
    body = _body(empty.get("/api/v1/plan/today"))
    assert body["session"] is None
    assert body["week"] is None
    assert body["has_plan"] is False
    assert body["has_session"] is False
    assert body["recommendation"]
    assert body["rationale"]


def test_the_plan_week_is_empty_and_has_no_adherence(empty):
    """A week with no plan: nulls, zero counts, and no division by zero."""
    today = date.today().isoformat()
    body = _body(empty.get(f"/api/v1/plan/week?date={today}"))
    assert body["week"] is None
    assert body["phase"] is None
    assert body["sessions"] == []
    assert body["planned_km"] == 0
    assert body["actual_km"] == 0
    assert body["planned_count"] == 0
    assert body["completed_count"] == 0
    assert body["adherence_pct"] is None


def test_the_plan_results_are_empty(empty):
    """No sessions, so no results."""
    assert _body(empty.get("/api/v1/plan/results"))["items"] == []


def test_the_plan_adherence_is_null_not_zero(empty):
    """A missing plan is never reported as zero adherence."""
    body = _body(empty.get("/api/v1/plan/adherence"))
    assert body["overall_pct"] is None
    assert body["planned_sessions"] == 0
    assert body["completed_sessions"] == 0
    assert body["hit_sessions"] == 0
    assert body["missed_sessions"] == 0
    assert body["extra_sessions"] == 0
    assert set(body["by_verdict"].values()) == {0}


def test_the_plan_sources_are_present_but_empty(empty):
    """The inferred plan exists with no sessions; Garmin's is unavailable."""
    body = _body(empty.get("/api/v1/plan/sources"))
    assert body["inferred"]["sessions"] == []
    assert body["inferred"]["prescribed"] is False
    assert body["garmin"]["available"] is False
    assert body["garmin"]["sessions"] == []
    assert body["garmin"]["reason"]


# ----------------------------------------------------------------- fitness


def test_the_fingerprints_are_empty(empty):
    """The running fingerprint of nothing is nothing."""
    body = _body(empty.get("/api/v1/fitness/fingerprint"))
    assert body["items"] == []


def test_the_fitness_trends_have_no_points_and_no_baseline(empty):
    """No decoupling to measure, and no baseline to invent."""
    body = _body(empty.get("/api/v1/fitness/trends?metric=decoupling_pct"))
    assert body["points"] == []
    assert body["baseline"] is None
    assert body["trend_pct"] is None


def test_the_fitness_zones_are_empty(empty):
    """The athlete's zone model has no time in it."""
    body = _body(empty.get("/api/v1/fitness/zones"))
    assert body["zones"] == []
    assert body["total_time_s"] == 0


def test_the_gear_list_is_empty(empty):
    """No shoes have been logged."""
    assert _body(empty.get("/api/v1/gear"))["items"] == []


def test_the_comparison_candidates_are_empty(empty):
    """There is nothing to compare."""
    assert _body(empty.get("/api/v1/compare/candidates"))["items"] == []


def test_comparing_nothing_is_an_empty_comparison(empty):
    """A comparison of no ids is empty, and names no parameters it cannot fill."""
    body = _body(empty.get("/api/v1/compare?ids="))
    assert body["runs"] == []
    assert body["missing"] == []


# -------------------------------------------------------------------- sync


def test_the_sync_history_is_empty(empty):
    """No sync has run, so there is no history."""
    body = _body(empty.get("/api/v1/sync/runs"))
    assert body["items"] == []


def test_the_sync_status_says_nothing_has_run(empty):
    """The sync status is well-formed with no sync behind it."""
    body = _body(empty.get("/api/v1/sync/status"))
    assert body["running"] is False
    assert body["last_sync_at"] is None
    assert body["connected"] is False


def test_the_settings_can_still_be_written_on_an_empty_instance(empty):
    """The settings endpoint answers, and the demo flag is not one of its fields."""
    response = empty.patch(
        "/api/v1/sync/settings",
        json={"sync_days": 30, "demo_visible": False},
    )
    body = _body(response)
    assert body["sync_days"] == 30
    assert "demo_visible" not in body


# -------------------------------------------------------------- regressions


def test_no_endpoint_raises_on_an_empty_database(empty):
    """Every read endpoint answers 200 (or a documented 404/422) with no rows.

    This is the sweep: the named tests above say what each body contains,
    and this one says none of them falls over.
    """
    today = date.today().isoformat()
    paths = [
        "/api/v1/health",
        "/api/v1/auth/status",
        "/api/v1/activities",
        "/api/v1/calendar",
        "/api/v1/compare/candidates",
        "/api/v1/fitness/fingerprint",
        "/api/v1/fitness/trends",
        "/api/v1/fitness/zones",
        "/api/v1/gear",
        "/api/v1/health/daily",
        "/api/v1/physiology/daily",
        "/api/v1/physiology/hrv",
        "/api/v1/physiology/latest",
        "/api/v1/physiology/readiness",
        "/api/v1/physiology/sleep",
        "/api/v1/physiology/sources",
        "/api/v1/plan/adherence",
        "/api/v1/plan/blocks",
        "/api/v1/plan/current",
        "/api/v1/plan/results",
        "/api/v1/plan/sources",
        "/api/v1/plan/today",
        f"/api/v1/plan/week?date={today}",
        "/api/v1/stats/elevation",
        "/api/v1/stats/heatmap",
        "/api/v1/stats/hr-zones",
        "/api/v1/stats/monthly",
        "/api/v1/stats/pace-trends",
        "/api/v1/stats/personal-records",
        "/api/v1/stats/summary",
        "/api/v1/stats/weekly",
        "/api/v1/sync/history",
        "/api/v1/sync/runs",
        "/api/v1/sync/status",
    ]
    for path in paths:
        response = empty.get(path)
        assert response.status_code == 200, f"{path} answered {response.status_code}"
        assert response.json() is not None, path


def test_a_year_of_readiness_on_an_empty_database_does_not_invent_a_day(empty):
    """The readiness series may enumerate the missing days; it may not score them."""
    start = (date.today() - timedelta(days=30)).isoformat()
    end = date.today().isoformat()
    body = _body(empty.get(f"/api/v1/physiology/readiness?from={start}&to={end}"))
    for item in body["items"]:
        assert item["score"] is None
        assert item["drivers"] == []
