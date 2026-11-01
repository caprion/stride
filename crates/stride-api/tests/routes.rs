//! One test per route.
//!
//! Every route of the frozen table answers, and each of them answers with
//! what it is for. The route table is the contract
//! (`docs/contract/api-paths.md`); this file walks it, route by route.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::Value;
use stride_ingest::{ACTIVITIES_PATH, HRV_DAILY_PATH, SLEEP_DAILY_PATH, USER_SUMMARY_PATH};

use common::{TODAY, TOKEN_PATH};

// ------------------------------------------------------------------- the data

#[test]
fn activities_answers_with_the_feed() {
    let harness = common::Harness::seeded("route-activities");
    let body = harness.ok_get("/activities");
    let runs = body["activities"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["local_date"].as_str(), Some("2026-05-10"));
    assert_eq!(runs[0]["distance"]["raw"].as_f64(), Some(10_000.0));
    assert_eq!(runs[0]["distance"]["text"].as_str(), Some("10 km"));
    assert!(body["next_cursor"].is_null());
    assert_eq!(body["generated_at"].as_str(), Some(common::NOW));
}

#[test]
fn activities_pages_with_a_cursor() {
    let harness = common::Harness::empty("route-activities-cursor");
    for index in 0..3 {
        harness.write_run(common::run(
            &format!("a-run-{index}"),
            &format!("2026-05-{:02}", index + 1),
            10_000.0,
            3_000.0,
        ));
    }
    let first = harness.ok_get("/activities?limit=2");
    assert_eq!(first["activities"].as_array().unwrap().len(), 2);
    let cursor = first["next_cursor"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(!cursor.is_empty(), "the feed did not offer a next page");

    let second = harness.ok_get(&format!("/activities?limit=2&cursor={cursor}"));
    assert_eq!(second["activities"].as_array().unwrap().len(), 1);
    assert!(second["next_cursor"].is_null());
}

#[test]
fn an_activity_answers_with_the_run() {
    let harness = common::Harness::seeded("route-activity");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert_eq!(body["id"].as_i64(), Some(id));
    assert_eq!(body["name"].as_str(), Some("a run"));
    assert_eq!(body["sport_type"].as_str(), Some("running"));
    assert_eq!(body["duration"]["text"].as_str(), Some("50 m"));
    assert!(body["splits"].is_array());
    assert!(body["samples"].is_array());
    assert!(body["plan_line"].is_null());
    assert!(body["note"].is_null());
}

#[test]
fn physiology_daily_answers_with_the_days() {
    let harness = common::Harness::seeded("route-physiology-daily");
    let body = harness.ok_get("/physiology/daily");
    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 3);
    assert_eq!(days[0]["date"].as_str(), Some("2026-05-10"));
    assert_eq!(days[0]["sleep"]["text"].as_str(), Some("6 h 10 m"));
    assert_eq!(days[0]["hrv"]["text"].as_str(), Some("48 ms"));
    assert_eq!(days[0]["resting_hr"]["text"].as_str(), Some("63 bpm"));
}

#[test]
fn physiology_sleep_answers_with_the_nights() {
    let harness = common::Harness::seeded("route-physiology-sleep");
    let body = harness.ok_get("/physiology/sleep");
    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 3);
    assert_eq!(days[0]["sleep"]["text"].as_str(), Some("6 h 10 m"));
    assert_eq!(days[0]["deep_sleep"]["text"].as_str(), Some("1 h 30 m"));
}

#[test]
fn physiology_hrv_answers_with_the_mornings() {
    let harness = common::Harness::seeded("route-physiology-hrv");
    let body = harness.ok_get("/physiology/hrv");
    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 3);
    assert_eq!(days[0]["hrv"]["text"].as_str(), Some("48 ms"));
    assert_eq!(days[0]["resting_hr"]["text"].as_str(), Some("63 bpm"));
}

#[test]
fn physiology_vitals_answers_with_the_snapshot() {
    let harness = common::Harness::seeded("route-physiology-vitals");
    let body = harness.ok_get("/physiology/vitals");
    assert_eq!(body["date"].as_str(), Some(TODAY));
    let vitals = body["vitals"].as_array().unwrap();
    assert_eq!(vitals.len(), 3);
    assert_eq!(vitals[0]["key"].as_str(), Some("sleep"));
    assert_eq!(vitals[1]["key"].as_str(), Some("hrv"));
    assert_eq!(vitals[2]["key"].as_str(), Some("resting_hr"));
    assert_eq!(body["rest"]["label"].as_str(), Some("Rest"));
    assert!(body["history"]["sleep"].is_array());
}

#[test]
fn physiology_latest_answers_with_the_most_recent_day() {
    let harness = common::Harness::seeded("route-physiology-latest");
    let body = harness.ok_get("/physiology/latest");
    assert_eq!(body["day"]["date"].as_str(), Some(TODAY));
    assert_eq!(body["day"]["hrv"]["text"].as_str(), Some("48 ms"));
}

#[test]
fn physiology_sources_answers_with_the_sources() {
    let harness = common::Harness::seeded("route-physiology-sources");
    let body = harness.ok_get("/physiology/sources");
    let sources = body["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 3);
    assert_eq!(sources[0]["name"].as_str(), Some("sleep"));
    assert_eq!(sources[0]["days"].as_i64(), Some(3));
    assert_eq!(sources[0]["latest"].as_str(), Some(TODAY));
}

#[test]
fn fitness_fingerprint_answers_with_the_fingerprints() {
    let harness = common::Harness::seeded("route-fitness-fingerprint");
    let body = harness.ok_get("/fitness/fingerprint");
    assert!(body["items"].is_array());
    assert_eq!(body["generated_at"].as_str(), Some(common::NOW));
}

#[test]
fn fitness_trends_answers_with_the_trends() {
    let harness = common::Harness::seeded("route-fitness-trends");
    let body = harness.ok_get("/fitness/trends");
    let trends = body["trends"].as_array().unwrap();
    assert_eq!(trends.len(), 2);
    assert_eq!(trends[0]["metric"].as_str(), Some("median_pace"));
    assert_eq!(trends[1]["metric"].as_str(), Some("avg_hr"));
}

#[test]
fn fitness_zones_answers_with_the_zones() {
    let harness = common::Harness::seeded("route-fitness-zones");
    let body = harness.ok_get("/fitness/zones");
    assert!(body["zones"].is_array());
    assert_eq!(body["total_time_s"].as_f64(), Some(0.0));
}

#[test]
fn stats_summary_answers_with_the_totals() {
    let harness = common::Harness::seeded("route-stats-summary");
    harness.write_run(common::run("a-run-today", TODAY, 9_000.0, 2_900.0));
    let body = harness.ok_get("/stats/summary");
    assert_eq!(body["all_time"]["activity_count"].as_i64(), Some(2));
    assert_eq!(body["all_time"]["distance"]["raw"].as_f64(), Some(19_000.0));
    assert_eq!(body["this_week"]["activity_count"].as_i64(), Some(1));
}

#[test]
fn stats_weekly_answers_with_the_weeks() {
    let harness = common::Harness::seeded("route-stats-weekly");
    let body = harness.ok_get("/stats/weekly");
    let weeks = body["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);
    assert_eq!(weeks[0]["week_start"].as_str(), Some("2026-05-04"));
    assert_eq!(weeks[0]["activity_count"].as_i64(), Some(1));
    assert_eq!(weeks[0]["distance"]["raw"].as_f64(), Some(10_000.0));
}

#[test]
fn stats_pace_trends_answers_with_the_pace() {
    let harness = common::Harness::seeded("route-stats-pace");
    let body = harness.ok_get("/stats/pace-trends");
    let weeks = body["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);
    assert_eq!(weeks[0]["run_count"].as_i64(), Some(1));
}

#[test]
fn stats_elevation_answers_with_the_climbing() {
    let harness = common::Harness::seeded("route-stats-elevation");
    let body = harness.ok_get("/stats/elevation");
    let weeks = body["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);
    assert_eq!(weeks[0]["week_start"].as_str(), Some("2026-05-04"));
}

#[test]
fn stats_personal_records_answers_with_the_records() {
    let harness = common::Harness::seeded("route-stats-records");
    let body = harness.ok_get("/stats/personal-records");
    let records = body["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["key"].as_str(), Some("progress.best.longest"));
    assert_eq!(records[0]["value"]["raw"].as_f64(), Some(10_000.0));
}

// ----------------------------------------------------------------- the plan

#[test]
fn plan_current_answers_with_the_block_in_force() {
    let harness = common::Harness::empty("route-plan-current");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-11", "easy", Some(8_000.0))],
    );
    let body = harness.ok_get("/plan/current");
    assert_eq!(body["block"]["name"].as_str(), Some("Base block"));
    assert_eq!(body["block"]["source"].as_str(), Some("prescribed"));
    assert_eq!(body["block"]["session_count"].as_i64(), Some(1));
}

#[test]
fn plan_today_answers_with_the_session_of_the_day() {
    let harness = common::Harness::empty("route-plan-today");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[
            ("2026-05-12", "easy", Some(10_000.0)),
            ("2026-05-13", "rest", None),
        ],
    );
    let body = harness.ok_get("/plan/today");
    assert_eq!(body["date"].as_str(), Some(TODAY));
    assert_eq!(body["session"]["session_type"].as_str(), Some("easy"));
    assert_eq!(body["session"]["status"].as_str(), Some("planned"));
    assert_eq!(
        body["session"]["sentence"].as_str(),
        Some("Today — 10 km · easy")
    );
}

#[test]
fn plan_week_answers_with_the_week() {
    let harness = common::Harness::empty("route-plan-week");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[
            ("2026-05-11", "easy", Some(8_000.0)),
            ("2026-05-12", "easy", Some(10_000.0)),
            ("2026-05-13", "rest", None),
        ],
    );
    let body = harness.ok_get("/plan/week");
    assert_eq!(body["block_name"].as_str(), Some("Base block"));
    assert_eq!(body["week_start"].as_str(), Some("2026-05-11"));
    assert_eq!(body["week_end"].as_str(), Some("2026-05-17"));
    assert_eq!(body["sessions"].as_array().unwrap().len(), 3);
    assert_eq!(body["adherence"]["planned"].as_u64(), Some(3));
    assert_eq!(body["adherence"]["done"].as_u64(), Some(0));
    assert_eq!(body["target"]["raw"].as_f64(), Some(18_000.0));
}

#[test]
fn plan_blocks_answers_with_the_blocks() {
    let harness = common::Harness::empty("route-plan-blocks");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-11", "easy", Some(8_000.0))],
    );
    let body = harness.ok_get("/plan/blocks");
    let blocks = body["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["status"].as_str(), Some("active"));
}

#[test]
fn plan_sources_answers_with_the_sources() {
    let harness = common::Harness::empty("route-plan-sources");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-11", "easy", Some(8_000.0))],
    );
    let body = harness.ok_get("/plan/sources");
    let sources = body["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 3);
    let prescribed = sources
        .iter()
        .find(|source| source["name"].as_str() == Some("prescribed"))
        .unwrap();
    assert_eq!(prescribed["active"].as_bool(), Some(true));
    assert_eq!(prescribed["blocks"].as_u64(), Some(1));
}

#[test]
fn plan_adherence_answers_with_a_count() {
    let harness = common::Harness::empty("route-plan-adherence");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[
            ("2026-05-10", "easy", Some(8_000.0)),
            ("2026-05-12", "easy", Some(10_000.0)),
        ],
    );
    // The run of the tenth answers the first session.
    harness.write_run(common::run("a-run", "2026-05-10", 8_000.0, 2_500.0));
    let body = harness.ok_get("/plan/adherence");
    assert_eq!(body["adherence"]["done"].as_u64(), Some(1));
    assert_eq!(body["adherence"]["planned"].as_u64(), Some(2));
    assert!(body["weeks"].is_array());
}

#[test]
fn plan_results_answers_with_what_happened() {
    let harness = common::Harness::empty("route-plan-results");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[
            ("2026-05-10", "easy", Some(8_000.0)),
            ("2026-05-12", "easy", Some(10_000.0)),
        ],
    );
    harness.write_run(common::run("a-run", "2026-05-10", 8_000.0, 2_500.0));
    let body = harness.ok_get("/plan/results");
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["status"].as_str(), Some("done"));
    assert_eq!(results[0]["actual"]["raw"].as_f64(), Some(8_000.0));
    assert_eq!(results[1]["status"].as_str(), Some("planned"));
}

#[test]
fn plan_import_answers_with_what_it_wrote() {
    let harness = common::Harness::empty("route-plan-import");
    let body = harness.post(
        "/plan/import",
        r#"{"name":"Base block","source":"typed","start_date":"2026-05-11",
            "sessions":[{"date":"2026-05-12","session_type":"easy","target_distance_m":10000}]}"#,
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(1));
    assert!(body["block_id"].as_i64().unwrap_or_default() > 0);

    // The plan is there afterwards.
    let week = harness.ok_get("/plan/week");
    assert_eq!(week["sessions"].as_array().unwrap().len(), 1);
}

#[test]
fn plan_import_refuses_a_plan_it_cannot_read() {
    let harness = common::Harness::empty("route-plan-import-bad");
    let answer = harness.router.call("POST", "/plan/import", Some("{}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

// -------------------------------------------------------------- the insights

#[test]
fn insights_today_answers_with_the_readout() {
    let harness = common::Harness::seeded("route-insights-today");
    harness.write_run(common::run("a-run-today", TODAY, 9_000.0, 2_900.0));
    harness.write_plan(
        "Base block",
        "prescribed",
        &[
            ("2026-05-12", "easy", Some(10_000.0)),
            ("2026-05-13", "rest", None),
            ("2026-05-14", "easy", Some(5_000.0)),
        ],
    );
    let body = harness.ok_get("/insights/today");
    assert_eq!(body["date"].as_str(), Some(TODAY));
    assert_eq!(body["planned"]["session_type"].as_str(), Some("easy"));
    // The rest day is not an alternative to run.
    let options = body["options"].as_array().unwrap();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0]["date"].as_str(), Some("2026-05-14"));
    assert!(body["last_run"].is_object());
    assert_eq!(body["week"]["sessions"].as_u64(), Some(1));
    assert_eq!(body["week"]["distance"]["raw"].as_f64(), Some(9_000.0));
}

#[test]
fn insights_today_states_the_distance_comparison_only_beyond_ten_per_cent() {
    let harness = common::Harness::empty("route-insights-comparison");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-12", "easy", Some(10_000.0))],
    );
    // 10.2 km against a 10 km target is a 2% gap: not news.
    harness.write_run(common::run("a-run", "2026-05-10", 10_200.0, 3_300.0));
    let body = harness.ok_get("/insights/today");
    assert!(
        body["distance_comparison"].is_null(),
        "a 2% gap was stated: {}",
        body["distance_comparison"]
    );

    // 12.4 km against a 10 km target is 24% over: news.
    harness.write_run(common::run("b-run", "2026-05-10", 12_400.0, 4_020.0));
    let body = harness.ok_get("/insights/today");
    assert_eq!(
        body["distance_comparison"].as_str(),
        Some("2.4 km more than the 10 km target")
    );
}

#[test]
fn insights_progress_answers_with_the_progress() {
    let harness = common::Harness::seeded("route-insights-progress");
    let body = harness.ok_get("/insights/progress");
    assert!(body["headline"].is_string());
    assert!(body["weekly_volume"].is_array());
    assert!(body["consistency"].is_array());
    assert!(body["bests"].is_array());
}

#[test]
fn insights_chips_answers_with_the_chips_of_a_run() {
    let harness = common::Harness::seeded("route-insights-chips");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get(&format!("/insights/chips?id={id}"));
    assert_eq!(body["activity_id"].as_i64(), Some(id));
    let chips = body["chips"].as_array().unwrap();
    assert_eq!(chips.len(), 4);
    assert_eq!(chips[0]["key"].as_str(), Some("distance"));
    assert_eq!(chips[0]["value"]["text"].as_str(), Some("10 km"));
}

#[test]
fn compare_answers_with_the_diff() {
    let harness = common::Harness::empty("route-compare");
    let first = harness.write_run(common::run("a-run", "2026-05-03", 10_000.0, 3_300.0));
    let second = harness.write_run(common::run("b-run", "2026-05-10", 10_200.0, 3_240.0));
    let body = harness.ok_get(&format!("/compare?a={first}&b={second}"));
    assert!(body["basis_text"].is_string());
    let metrics = body["metrics"].as_array().unwrap();
    assert_eq!(metrics.len(), 4);
    assert_eq!(metrics[0]["key"].as_str(), Some("distance"));
    assert_eq!(metrics[0]["a"]["raw"].as_f64(), Some(10_000.0));
    assert_eq!(metrics[0]["b"]["raw"].as_f64(), Some(10_200.0));
    assert_eq!(body["a"]["id"].as_i64(), Some(first));
    assert_eq!(body["b"]["id"].as_i64(), Some(second));
}

#[test]
fn compare_candidates_answers_with_the_runs_to_compare_against() {
    let harness = common::Harness::seeded("route-compare-candidates");
    let body = harness.ok_get("/compare/candidates");
    let candidates = body.as_array().unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["date"].as_str(), Some("2026-05-10"));
    assert_eq!(candidates[0]["distance"]["raw"].as_f64(), Some(10_000.0));
}

// -------------------------------------------------------------------- the app

#[test]
fn health_answers_with_the_liveness() {
    let harness = common::Harness::seeded("route-health");
    let body = harness.ok_get("/health");
    assert_eq!(body["ok"].as_bool(), Some(true));
    assert_eq!(body["service"].as_str(), Some("stride"));
    assert_eq!(body["activities"].as_i64(), Some(1));
    assert_eq!(body["wellness_days"].as_i64(), Some(3));
    assert_eq!(body["garmin_connected"].as_bool(), Some(false));
}

#[test]
fn health_daily_answers_with_the_staleness() {
    let harness = common::Harness::seeded("route-health-daily");
    let body = harness.ok_get("/health/daily");
    assert_eq!(body["date"].as_str(), Some(TODAY));
    // Nothing has been synced yet, so the data is stale.
    assert_eq!(body["stale"].as_bool(), Some(true));
}

#[test]
fn sync_status_answers_with_the_state_of_the_last_sync() {
    let harness = common::Harness::seeded("route-sync-status");
    let body = harness.ok_get("/sync/status");
    assert!(body["last_sync_at"].is_null());
    assert!(body["last_error"].is_null());
    assert_eq!(body["activity_count"].as_i64(), Some(1));
}

#[test]
fn sync_run_answers_with_what_the_pull_brought() {
    let harness = common::Harness::empty("route-sync-run");
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    harness.garmin.with_json(TOKEN_PATH, "\"the-access-value\"");
    harness
        .garmin
        .with_json(ACTIVITIES_PATH, "{\"activityList\": []}");
    harness.garmin.with_json(
        SLEEP_DAILY_PATH,
        "{\"dailySleep\": {\"sleepTimeSeconds\": 27900.0}}",
    );
    harness
        .garmin
        .with_json(HRV_DAILY_PATH, "{\"hrvSummary\": {\"lastNightAvg\": 58.0}}");
    harness.garmin.with_json(
        USER_SUMMARY_PATH,
        "{\"summaryDTO\": {\"calendarDate\": \"2026-05-12\", \"restingHeartRate\": 47.0}}",
    );

    let body = harness.post("/sync/run", "{\"days\": 1}");
    assert_eq!(body["activities"].as_u64(), Some(0));
    assert_eq!(body["wellness_days"].as_u64(), Some(1));
    assert_eq!(body["generated_at"].as_str(), Some(common::NOW));

    // The wellness the pull brought is in the store.
    let days = harness.ok_get("/physiology/daily");
    assert!(!days["days"].as_array().unwrap().is_empty());
}

#[test]
fn sync_run_surfaces_a_refused_key_as_a_typed_failure() {
    let harness = common::Harness::empty("route-sync-refused");
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    harness
        .garmin
        .with_status(TOKEN_PATH, 401, "the key was refused");
    let answer = harness.router.call("POST", "/sync/run", Some("{}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("garmin_rejected"));
    assert_eq!(answer.status, 401);
}

#[test]
fn sync_run_surfaces_an_ended_session_as_a_typed_failure() {
    let harness = common::Harness::empty("route-sync-reconnect");
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    // The exchange works and the session is refused: the client answers
    // `needs_reconnect`, and the router does not retry it.
    harness.garmin.with_json(TOKEN_PATH, "\"the-access-value\"");
    harness
        .garmin
        .with_status(ACTIVITIES_PATH, 401, "the session ended");
    harness
        .garmin
        .with_json(TOKEN_PATH, "\"the-second-access-value\"");
    harness
        .garmin
        .with_status(ACTIVITIES_PATH, 401, "the session ended again");

    let answer = harness.router.call("POST", "/sync/run", Some("{}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("needs_reconnect"));
    assert_eq!(answer.status, 401);
    assert_eq!(answer.status, 401);
}

#[test]
fn sync_runs_answers_with_the_runs_that_came_from_garmin() {
    let harness = common::Harness::seeded("route-sync-runs");
    let body = harness.ok_get("/sync/runs");
    let runs = body["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["local_date"].as_str(), Some("2026-05-10"));
}

#[test]
fn sync_history_answers_with_the_state_of_each_kind() {
    let harness = common::Harness::empty("route-sync-history");
    let body = harness.ok_get("/sync/history");
    assert!(body["entries"].is_array());
}

#[test]
fn sync_settings_answers_with_the_windows() {
    let harness = common::Harness::empty("route-sync-settings");
    let body = harness.ok_get("/sync/settings");
    assert_eq!(body["sync_days"].as_i64(), Some(30));
    assert_eq!(body["history_months"].as_i64(), Some(12));
    assert_eq!(body["health_history_days"].as_i64(), Some(90));
}

#[test]
fn auth_status_answers_with_the_connection() {
    let harness = common::Harness::empty("route-auth-status");
    let body = harness.ok_get("/auth/status");
    assert_eq!(body["connected"].as_bool(), Some(false));
    assert_eq!(body["key_source"].as_str(), Some("none"));
    assert!(body["method"].is_null());
}

#[test]
fn auth_personal_key_holds_the_key_and_never_returns_it() {
    let harness = common::Harness::empty("route-auth-key");
    let body = harness.post(
        "/auth/garmin/personal-key",
        &format!("{{\"key\":\"{}\"}}", common::GARMIN_KEY),
    );
    assert_eq!(body["connected"].as_bool(), Some(true));
    assert_eq!(body["method"].as_str(), Some("personal_key"));
    assert!(
        !body.to_string().contains(common::GARMIN_KEY),
        "the key came back: {body}"
    );
    assert!(harness.router.api().garmin_connected());
}

#[test]
fn auth_personal_key_test_answers_with_whether_garmin_took_the_key() {
    let harness = common::Harness::empty("route-auth-key-test");
    harness.garmin.with_json(TOKEN_PATH, "\"the-access-value\"");
    harness.garmin.with_json(
        "/userprofile-service/userprofile/settings",
        "{\"displayName\": \"ATHLETE\"}",
    );
    let body = harness.post(
        "/auth/garmin/personal-key/test",
        &format!("{{\"key\":\"{}\"}}", common::GARMIN_KEY),
    );
    assert_eq!(body["ok"].as_bool(), Some(true));
    assert!(
        !body.to_string().contains(common::GARMIN_KEY),
        "the key came back: {body}"
    );
}

#[test]
fn auth_personal_key_test_refuses_a_key_garmin_will_not_take() {
    let harness = common::Harness::empty("route-auth-key-refused");
    harness
        .garmin
        .with_status(TOKEN_PATH, 401, "the key was refused");
    let answer = harness.router.call(
        "POST",
        "/auth/garmin/personal-key/test",
        Some(&format!("{{\"key\":\"{}\"}}", common::GARMIN_KEY)),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("garmin_rejected"));
}

#[test]
fn auth_fit_import_answers_with_the_run_the_file_became() {
    let harness = common::Harness::empty("route-auth-fit-import");
    let file = common::corpus_fit("real_garmin_run.fit");
    let body = harness.post(
        "/auth/garmin/fit-import",
        &format!("{{\"fit_base64\":\"{}\"}}", common::base64(&file)),
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    let id = body["activity_id"].as_i64().unwrap_or_default();
    assert!(id > 0);

    // The run is there, with the series the file carries.
    let run = harness.ok_get(&format!("/activities/{id}"));
    assert_eq!(run["sport_type"].as_str(), Some("running"));
    assert!(!run["splits"].as_array().unwrap().is_empty());
    assert!(!run["samples"].as_array().unwrap().is_empty());
}

#[test]
fn auth_fit_import_refuses_a_file_that_is_not_a_fit_file() {
    let harness = common::Harness::empty("route-auth-fit-bad");
    let answer = harness.router.call(
        "POST",
        "/auth/garmin/fit-import",
        Some("{\"fit_base64\":\"bm90IGEgZml0IGZpbGU=\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn auth_logout_forgets_the_key() {
    let harness = common::Harness::empty("route-auth-logout");
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    let body = harness.post("/auth/garmin/logout", "{}");
    assert_eq!(body["connected"].as_bool(), Some(false));
    assert!(!harness.router.api().garmin_connected());
}

#[test]
fn calendar_answers_with_the_month() {
    let harness = common::Harness::seeded("route-calendar");
    let body = harness.ok_get("/calendar?year=2026&month=5");
    assert_eq!(body["year"].as_i64(), Some(2026));
    assert_eq!(body["month"].as_u64(), Some(5));
    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 31);
    let tenth = days
        .iter()
        .find(|day| day["date"].as_str() == Some("2026-05-10"))
        .unwrap();
    assert_eq!(tenth["day"].as_u64(), Some(10));
    assert_eq!(tenth["activities"].as_array().unwrap().len(), 1);
    assert_eq!(tenth["distance"]["raw"].as_f64(), Some(10_000.0));
}

#[test]
fn the_compare_delta_carries_its_sign() {
    let harness = common::Harness::empty("route-compare-delta");
    let first = harness.write_run(common::run("a-run", "2026-05-03", 10_000.0, 3_300.0));
    let second = harness.write_run(common::run("b-run", "2026-05-10", 10_200.0, 3_240.0));

    let body = harness.ok_get(&format!("/compare?a={first}&b={second}"));
    let metrics = body["metrics"].as_array().unwrap();
    let distance = metrics
        .iter()
        .find(|metric| metric["key"] == "distance")
        .unwrap();
    // The core renders the difference, sign and all.
    assert_eq!(distance["delta"]["text"].as_str(), Some("+0.2 km"));
    assert_eq!(distance["delta"]["raw"].as_f64(), Some(200.0));

    let time = metrics
        .iter()
        .find(|metric| metric["key"] == "time")
        .unwrap();
    assert_eq!(time["delta"]["text"].as_str(), Some("\u{2212}1 m"));

    // Swapping A and B flips the sign of every difference: the question is
    // a different one, and the app asks the API again.
    let swapped = harness.ok_get(&format!("/compare?a={second}&b={first}"));
    let metrics = swapped["metrics"].as_array().unwrap();
    let distance = metrics
        .iter()
        .find(|metric| metric["key"] == "distance")
        .unwrap();
    assert_eq!(distance["delta"]["text"].as_str(), Some("\u{2212}0.2 km"));
    assert_eq!(distance["delta"]["raw"].as_f64(), Some(-200.0));
}

#[test]
fn the_feed_groups_the_runs_and_carries_the_group_totals() {
    let harness = common::Harness::empty("route-feed-group");
    // Two runs in the same week, one in another.
    harness.write_run(common::run("a-run", "2026-05-04", 10_000.0, 3_000.0));
    harness.write_run(common::run("b-run", "2026-05-06", 5_000.0, 1_500.0));
    harness.write_run(common::run("c-run", "2026-05-11", 8_000.0, 2_400.0));

    let body = harness.ok_get("/activities");
    let runs = body["activities"].as_array().unwrap();
    assert_eq!(runs.len(), 3);
    // The group is the week: the key is the day the week starts on.
    assert_eq!(runs[0]["group"]["key"].as_str(), Some("2026-05-11"));
    assert_eq!(runs[0]["group"]["distance"]["raw"].as_f64(), Some(8_000.0));
    assert_eq!(runs[1]["group"]["key"].as_str(), Some("2026-05-04"));
    // The group's totals are the week's, not the page's.
    assert_eq!(runs[1]["group"]["distance"]["raw"].as_f64(), Some(15_000.0));
    assert_eq!(
        runs[1]["group"]["duration"]["text"].as_str(),
        Some("1 h 15 m")
    );
}

#[test]
fn the_plan_line_is_the_core_s_statement() {
    let harness = common::Harness::empty("route-plan-line");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-10", "easy", Some(10_000.0))],
    );
    let id = harness.write_run(common::run("a-run", "2026-05-10", 12_400.0, 4_020.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    // The line against the plan is the core's statement: the distance and
    // the comparison, in the core's words.
    assert_eq!(
        body["plan_line"]["text"].as_str(),
        Some("12.4 km \u{b7} 2.4 km more than the 10 km target")
    );
}

#[test]
fn the_plan_line_is_absent_when_the_gap_is_not_news() {
    let harness = common::Harness::empty("route-plan-line-small");
    harness.write_plan(
        "Base block",
        "prescribed",
        &[("2026-05-10", "easy", Some(10_000.0))],
    );
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_200.0, 3_300.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert!(
        body["plan_line"].is_null(),
        "a 2% gap was stated: {}",
        body["plan_line"]
    );
}

/// The answers of every route, read once: nothing in them is a secret and
/// nothing in them is a credential.
#[test]
fn no_answer_carries_a_credential() {
    let harness = common::Harness::seeded("route-no-credential");
    for route in stride_api::ROUTES.iter() {
        let path = route.pattern.replace("{id}", "1");
        let body: Value = harness
            .router
            .call(route.method.as_str(), &path, Some("{}"))
            .body;
        let text = body.to_string();
        assert!(
            !text.contains(common::GARMIN_KEY),
            "the key is in the answer of {path}"
        );
        assert!(
            !text.to_lowercase().contains("apikey"),
            "something that looks like a key is in the answer of {path}"
        );
    }
}
