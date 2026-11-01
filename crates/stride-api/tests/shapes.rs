//! The shape of every answer.
//!
//! The response shapes are the contract the app consumes, so they are
//! written down here and checked here: the keys of each answer, and the
//! nullability of each field.
//!
//! Two rules are checked by this file:
//!
//! * **the keys** — an answer carries the fields its shape names, and no
//!   others;
//! * **the nullability** — a field that may be absent is `null` when it
//!   is, and never `0`.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::Value;

/// The keys of an answer, sorted, against the keys the shape names.
fn assert_keys(value: &Value, expected: &[&str], where_: &str) {
    let mut expected: Vec<String> = expected.iter().map(|key| key.to_string()).collect();
    expected.sort();
    assert_eq!(
        common::keys(value),
        expected,
        "the shape of {where_} is not the shape the contract names"
    );
}

/// The first object of an array.
fn first<'a>(value: &'a Value, name: &str) -> &'a Value {
    match value.get(name).and_then(|value| value.as_array()) {
        Some(items) => match items.first() {
            Some(first) => first,
            None => panic!("{name} is empty"),
        },
        None => panic!("{name} is not an array"),
    }
}

#[test]
fn the_feed_carries_the_shape_the_app_reads() {
    let harness = common::Harness::seeded("shape-feed");
    let body = harness.ok_get("/activities");
    assert_keys(
        &body,
        &["activities", "generated_at", "next_cursor"],
        "/activities",
    );
    let run = first(&body, "activities");
    assert_keys(
        run,
        &[
            "distance",
            "duration",
            "elevation_gain",
            "geo_points",
            "group",
            "heart_rate",
            "id",
            "local_date",
            "name",
            "pace",
            "sport_type",
            "start_time",
            "story",
        ],
        "/activities: an activity",
    );
}

#[test]
fn one_run_carries_the_shape_the_app_reads() {
    let harness = common::Harness::seeded("shape-detail");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert_keys(
        &body,
        &[
            "distance",
            "duration",
            "elevation",
            "elevation_gain",
            "geo_points",
            "group",
            "heart_rate",
            "id",
            "local_date",
            "name",
            "note",
            "pace",
            "plan_line",
            "samples",
            "sport_type",
            "splits",
            "start_time",
            "story",
        ],
        "/activities/{id}",
    );
}

#[test]
fn the_wellness_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-wellness");

    let daily = harness.ok_get("/physiology/daily");
    assert_keys(&daily, &["days", "generated_at"], "/physiology/daily");
    assert_keys(
        first(&daily, "days"),
        &[
            "awake",
            "date",
            "deep_sleep",
            "hrv",
            "light_sleep",
            "rem_sleep",
            "resting_hr",
            "sleep",
            "source",
            "stress",
        ],
        "/physiology/daily: a day",
    );

    let sleep = harness.ok_get("/physiology/sleep");
    assert_keys(&sleep, &["days", "generated_at"], "/physiology/sleep");
    assert_keys(
        first(&sleep, "days"),
        &[
            "awake",
            "date",
            "deep_sleep",
            "light_sleep",
            "rem_sleep",
            "sleep",
        ],
        "/physiology/sleep: a night",
    );

    let hrv = harness.ok_get("/physiology/hrv");
    assert_keys(&hrv, &["days", "generated_at"], "/physiology/hrv");
    assert_keys(
        first(&hrv, "days"),
        &["date", "hrv", "resting_hr"],
        "/physiology/hrv: a morning",
    );

    let latest = harness.ok_get("/physiology/latest");
    assert_keys(&latest, &["day", "generated_at"], "/physiology/latest");

    let sources = harness.ok_get("/physiology/sources");
    assert_keys(
        &sources,
        &["generated_at", "sources"],
        "/physiology/sources",
    );
    assert_keys(
        first(&sources, "sources"),
        &["days", "latest", "name"],
        "/physiology/sources: a source",
    );
}

#[test]
fn the_vitals_snapshot_carries_the_shape_the_app_reads() {
    let harness = common::Harness::seeded("shape-vitals");
    let body = harness.ok_get("/physiology/vitals");
    assert_keys(
        &body,
        &[
            "date",
            "generated_at",
            "history",
            "patterns",
            "rest",
            "vitals",
        ],
        "/physiology/vitals",
    );
    assert_keys(
        &body["rest"],
        &["label", "line"],
        "/physiology/vitals: rest",
    );
    assert_keys(
        first(&body, "vitals"),
        &[
            "band",
            "deviation_text",
            "direction_text",
            "key",
            "label",
            "status_text",
            "travel_text",
            "value_text",
        ],
        "/physiology/vitals: a vital",
    );
    assert_keys(
        &body["history"],
        &["hrv", "resting_hr", "sleep"],
        "/physiology/vitals: history",
    );
}

#[test]
fn the_fitness_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-fitness");

    let fingerprint = harness.ok_get("/fitness/fingerprint");
    assert_keys(
        &fingerprint,
        &["generated_at", "items"],
        "/fitness/fingerprint",
    );

    let trends = harness.ok_get("/fitness/trends");
    assert_keys(&trends, &["generated_at", "trends"], "/fitness/trends");
    assert_keys(
        first(&trends, "trends"),
        &["baseline", "label", "metric", "points", "unit"],
        "/fitness/trends: a trend",
    );

    let zones = harness.ok_get("/fitness/zones");
    assert_keys(
        &zones,
        &["generated_at", "total_time_s", "zones"],
        "/fitness/zones",
    );
}

#[test]
fn the_statistics_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-stats");

    let summary = harness.ok_get("/stats/summary");
    assert_keys(
        &summary,
        &[
            "all_time",
            "generated_at",
            "this_month",
            "this_week",
            "this_year",
        ],
        "/stats/summary",
    );
    assert_keys(
        &summary["all_time"],
        &[
            "activity_count",
            "heart_rate",
            "avg_pace",
            "distance",
            "elevation_gain",
            "duration",
        ],
        "/stats/summary: the totals",
    );

    let weekly = harness.ok_get("/stats/weekly");
    assert_keys(&weekly, &["generated_at", "weeks"], "/stats/weekly");
    assert_keys(
        first(&weekly, "weeks"),
        &[
            "activity_count",
            "avg_pace",
            "distance",
            "elevation_gain",
            "duration",
            "week_start",
        ],
        "/stats/weekly: a week",
    );

    let pace = harness.ok_get("/stats/pace-trends");
    assert_keys(&pace, &["generated_at", "weeks"], "/stats/pace-trends");
    assert_keys(
        first(&pace, "weeks"),
        &["best_pace", "median_pace", "run_count", "week_start"],
        "/stats/pace-trends: a week",
    );

    let elevation = harness.ok_get("/stats/elevation");
    assert_keys(&elevation, &["generated_at", "weeks"], "/stats/elevation");
    assert_keys(
        first(&elevation, "weeks"),
        &["elevation_gain", "week_start"],
        "/stats/elevation: a week",
    );

    let records = harness.ok_get("/stats/personal-records");
    assert_keys(
        &records,
        &["generated_at", "records"],
        "/stats/personal-records",
    );
}

#[test]
fn the_plan_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-plan");

    let current = harness.ok_get("/plan/current");
    assert_keys(&current, &["block", "generated_at"], "/plan/current");

    let today = harness.ok_get("/plan/today");
    assert_keys(&today, &["date", "generated_at", "session"], "/plan/today");

    let week = harness.ok_get("/plan/week");
    assert_keys(
        &week,
        &[
            "adherence",
            "block_name",
            "sessions",
            "source",
            "target",
            "week_end",
            "week_start",
        ],
        "/plan/week",
    );

    let blocks = harness.ok_get("/plan/blocks");
    assert_keys(&blocks, &["blocks", "generated_at"], "/plan/blocks");

    let sources = harness.ok_get("/plan/sources");
    assert_keys(&sources, &["generated_at", "sources"], "/plan/sources");
    assert_keys(
        first(&sources, "sources"),
        &["active", "blocks", "name"],
        "/plan/sources: a source",
    );

    let adherence = harness.ok_get("/plan/adherence");
    assert_keys(
        &adherence,
        &["adherence", "generated_at", "weeks"],
        "/plan/adherence",
    );
    assert_keys(
        &adherence["adherence"],
        &["done", "planned"],
        "/plan/adherence: the count",
    );

    let results = harness.ok_get("/plan/results");
    assert_keys(&results, &["generated_at", "results"], "/plan/results");
}

#[test]
fn the_insight_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-insight");

    let today = harness.ok_get("/insights/today");
    assert_keys(
        &today,
        &[
            "date",
            "distance_comparison",
            "generated_at",
            "last_run",
            "options",
            "planned",
            "week",
        ],
        "/insights/today",
    );
    assert_keys(
        &today["week"],
        &["distance", "sessions"],
        "/insights/today: week",
    );

    let progress = harness.ok_get("/insights/progress");
    assert_keys(
        &progress,
        &[
            "adherence",
            "bests",
            "consistency",
            "generated_at",
            "headline",
            "headline_note",
            "pace_at_hr",
            "sessions_per_week",
            "typical_week",
            "weekly_volume",
        ],
        "/insights/progress",
    );

    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let chips = harness.ok_get(&format!("/insights/chips?id={id}"));
    assert_keys(
        &chips,
        &["activity_id", "chips", "generated_at"],
        "/insights/chips",
    );
    assert_keys(
        first(&chips, "chips"),
        &["key", "label", "value"],
        "/insights/chips: a chip",
    );
}

#[test]
fn the_compare_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-compare");
    let first_id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let second_id = harness.write_run(common::run("b-run", "2026-05-03", 10_200.0, 3_320.0));

    let compare = harness.ok_get(&format!("/compare?a={first_id}&b={second_id}"));
    assert_keys(
        &compare,
        &["a", "b", "basis_text", "metrics", "series"],
        "/compare",
    );
    assert_keys(
        first(&compare, "metrics"),
        &["a", "b", "delta", "key", "label_key"],
        "/compare: a metric",
    );
    assert_keys(
        &compare["a"],
        &["date", "distance", "id", "label", "story"],
        "/compare: a run",
    );

    let candidates = harness.ok_get("/compare/candidates");
    assert!(
        candidates.is_array(),
        "the candidates are the list the app reads"
    );
    assert_keys(
        &candidates[0],
        &["date", "distance", "id", "label"],
        "/compare/candidates: a candidate",
    );
}

#[test]
fn the_app_routes_carry_their_shapes() {
    let harness = common::Harness::seeded("shape-app");

    let health = harness.ok_get("/health");
    assert_keys(
        &health,
        &[
            "activities",
            "garmin_connected",
            "generated_at",
            "ok",
            "plan_sessions",
            "service",
            "version",
            "wellness_days",
        ],
        "/health",
    );

    let daily = harness.ok_get("/health/daily");
    assert_keys(&daily, &["date", "generated_at", "stale"], "/health/daily");

    let status = harness.ok_get("/sync/status");
    assert_keys(
        &status,
        &["activity_count", "last_error", "last_sync_at"],
        "/sync/status",
    );

    let runs = harness.ok_get("/sync/runs");
    assert_keys(&runs, &["generated_at", "runs"], "/sync/runs");

    let history = harness.ok_get("/sync/history");
    assert_keys(&history, &["entries", "generated_at"], "/sync/history");

    let settings = harness.ok_get("/sync/settings");
    assert_keys(
        &settings,
        &[
            "generated_at",
            "health_history_days",
            "history_months",
            "sync_days",
        ],
        "/sync/settings",
    );

    let auth = harness.ok_get("/auth/status");
    assert_keys(
        &auth,
        &[
            "connected",
            "generated_at",
            "key_source",
            "last_sync_at",
            "method",
        ],
        "/auth/status",
    );

    let calendar = harness.ok_get("/calendar?year=2026&month=5");
    assert_keys(
        &calendar,
        &["days", "generated_at", "month", "year"],
        "/calendar",
    );
    assert_keys(
        first(&calendar, "days"),
        &["activities", "date", "day", "distance", "duration"],
        "/calendar: a day",
    );
}
