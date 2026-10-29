// Integration tests are their own crate, so clippy's allow-in-tests
// configuration does not cover them: the expectations below are allowed on
// purpose. The library itself raises no panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Every tool, against the fixture store.
//!
//! Each test calls one tool and checks the shape of its answer: the three
//! keys every answer has, the fields the tool promises, and the numbers the
//! fixture holds. A tool that answers with the right numbers in the wrong
//! shape is a tool a harness cannot use.

mod common;

use serde_json::{json, Value};

/// The three keys every answer has, and the attribution inside them.
fn assert_envelope(answer: &Value, tool: &str) {
    assert_eq!(answer["tool"], json!(tool));
    assert!(answer["data"].is_object(), "{tool}: data must be an object");
    assert!(
        answer["coverage"].is_object(),
        "{tool}: coverage must be an object"
    );
    assert!(
        answer["limitations"].is_array(),
        "{tool}: limitations must be an array"
    );
    assert!(
        answer["coverage"]["attributed_to"].is_string(),
        "{tool}: the answer must say what it answered from"
    );
}

#[test]
fn the_digest_reports_the_period_the_load_and_the_plan() {
    let tools = common::tools();
    let answer = tools
        .call("training_digest", &json!({ "period": "28d" }))
        .expect("the digest");
    assert_envelope(&answer, "training_digest");

    let data = &answer["data"];
    assert_eq!(data["period"]["name"], json!("28d"));
    assert_eq!(data["period"]["from"], json!("2026-10-01"));
    assert_eq!(data["period"]["to"], json!("2026-10-28"));

    // The four runs of the period: 10 000 m, 8 000 m, 21 000 m and 5 000 m.
    assert_eq!(data["load"]["runs"], json!(4));
    assert_eq!(data["load"]["distance_m"], json!(44_000.0));
    assert_eq!(data["load"]["elevation_gain_m"], json!(506.0));

    // The weekly series covers the period, newest week last.
    let weeks = data["load"]["weekly_series"].as_array().expect("weeks");
    assert!(!weeks.is_empty());
    assert!(weeks.last().expect("week")["week"].is_string());

    // The plan: one session of sixteen kilometres planned, ten kept.
    assert_eq!(data["plan"]["planned_km"], json!(16.0));
    assert_eq!(data["plan"]["actual_km"], json!(10.0));
    assert_eq!(data["plan"]["adherence_pct"], json!(50.0));
    assert_eq!(data["plan"]["misses_found"], json!(1));
    let miss = &data["plan"]["misses"][0];
    assert_eq!(miss["verdict"], json!("missed"));
    assert_eq!(miss["cause"], json!("The session was missed."));
    assert_eq!(miss["cause_confidence"], json!(0.6));

    // The readiness of the period.
    assert_eq!(data["physiology"]["days_with_readiness"], json!(7));
    assert_eq!(data["physiology"]["latest"]["date"], json!("2026-10-28"));
    assert_eq!(data["physiology"]["latest"]["score"], json!(82.0));

    // The change against the period before it.
    assert!(data["change_vs_previous"]["distance_m"].is_number());
    assert_eq!(answer["coverage"]["from"], json!("2026-10-01"));
    assert_eq!(answer["coverage"]["to"], json!("2026-10-28"));
}

#[test]
fn the_search_finds_the_runs_and_pages_them() {
    let tools = common::tools();
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "to": "2026-10-28" }),
        )
        .expect("the search");
    assert_envelope(&answer, "search_activities");

    let items = answer["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 4);
    // Newest first: the morning run of the 27th.
    assert_eq!(items[0]["id"], json!(1));
    assert_eq!(items[0]["name"], json!("Morning run"));
    assert_eq!(items[0]["distance_m"], json!(10_000.0));
    assert_eq!(items[0]["pace_s_per_km"], json!(300.0));
    assert_eq!(answer["data"]["total_estimate"], json!(4));
    assert_eq!(answer["data"]["next_cursor"], Value::Null);

    // The page is bounded by the limit.
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "to": "2026-10-28", "limit": 2 }),
        )
        .expect("the search");
    assert_eq!(answer["data"]["items"].as_array().expect("items").len(), 2);
    assert_eq!(answer["data"]["next_cursor"], json!("p2"));

    // And the next page holds the rest.
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "to": "2026-10-28", "limit": 2, "cursor": "p2" }),
        )
        .expect("the search");
    assert_eq!(answer["data"]["items"].as_array().expect("items").len(), 2);
    assert_eq!(answer["data"]["next_cursor"], Value::Null);
}

#[test]
fn the_search_filters_by_distance_and_says_that_it_did() {
    let tools = common::tools();
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-09-01", "to": "2026-10-28", "min_distance_m": 10_000.0 }),
        )
        .expect("the search");
    let items = answer["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 3);
    assert!(!answer["limitations"]
        .as_array()
        .expect("limitations")
        .is_empty());
}

#[test]
fn the_activity_detail_holds_the_splits_and_the_zones() {
    let tools = common::tools();
    let answer = tools
        .call("get_activity", &json!({ "activity_id": 1 }))
        .expect("the activity");
    assert_envelope(&answer, "get_activity");

    assert_eq!(answer["data"]["activity"]["id"], json!(1));
    assert_eq!(answer["data"]["activity"]["distance_m"], json!(10_000.0));
    assert_eq!(answer["data"]["activity"]["pace_s_per_km"], json!(300.0));

    let laps = answer["data"]["laps"].as_array().expect("laps");
    assert_eq!(laps.len(), 4);
    assert_eq!(laps[0]["lap_index"], json!(1));
    assert_eq!(laps[0]["distance_m"], json!(2_500.0));

    let zones = answer["data"]["zones"].as_array().expect("zones");
    assert_eq!(zones.len(), 5);
    assert_eq!(zones[0]["zone_no"], json!(1));

    // The streams and the route are not in the answer unless they are asked
    // for: the route is the athlete's location.
    assert!(answer["data"].get("streams").is_none());
    assert!(answer["data"].get("route").is_none());
    assert_eq!(answer["coverage"]["has_route"], json!(true));
}

#[test]
fn the_activity_detail_gives_the_streams_and_the_route_when_they_are_asked_for() {
    let tools = common::tools();
    let answer = tools
        .call(
            "get_activity",
            &json!({ "activity_id": 1, "include": ["streams", "route"] }),
        )
        .expect("the activity");
    let streams = answer["data"]["streams"].as_object().expect("streams");
    assert!(streams.contains_key("hr"));
    let samples = streams["hr"]["samples"].as_array().expect("samples");
    // Three hundred samples, downsampled to at most 240.
    assert!(samples.len() <= 241);
    assert!(samples.len() > 100);
    assert_eq!(samples[0]["t"].as_f64(), Some(0.0));
    assert!(samples[0]["value"].is_number());

    let route = answer["data"]["route"].as_object().expect("route");
    assert_eq!(route["point_count"], json!(4));
    assert_eq!(route["points"].as_array().expect("points").len(), 4);
}

#[test]
fn the_analysis_splits_the_run_and_says_what_it_found() {
    let tools = common::tools();
    let answer = tools
        .call("activity_analysis", &json!({ "activity_id": 1 }))
        .expect("the analysis");
    assert_envelope(&answer, "activity_analysis");

    let segments = answer["data"]["segments"].as_array().expect("segments");
    assert_eq!(segments.len(), 4);
    assert_eq!(segments[0]["distance_m"], json!(2_500.0));
    assert_eq!(segments[0]["pace_s_per_km"], json!(300.3));

    // The splits are described, never explained: the second split of the
    // fixture's run climbs, the last one fades.
    assert_eq!(segments[0]["notes"], json!("the fastest split of the run"));
    assert_eq!(segments[1]["grade_pct"], json!(4.8));
    assert_eq!(segments[1]["notes"], json!("a climb"));
    assert_eq!(segments[2]["notes"], Value::Null);
    assert_eq!(
        segments[3]["notes"],
        json!("the slowest split of the run, slower than the first split")
    );

    let signals = answer["data"]["signals"].as_array().expect("signals");
    assert!(!signals.is_empty());
    for signal in signals {
        assert!(signal["finding"].is_string());
        assert!(signal["evidence"].is_object());
        assert!(signal["confidence"].is_string());
    }

    // The comparison was not asked for, so there is none.
    assert!(answer["data"].get("comparison").is_none());
}

#[test]
fn the_analysis_compares_with_a_similar_run() {
    let tools = common::tools();
    let answer = tools
        .call(
            "activity_analysis",
            &json!({ "activity_id": 1, "compare_with": "similar_recent" }),
        )
        .expect("the analysis");
    let comparison = answer["data"]["comparison"]
        .as_object()
        .expect("comparison");
    assert_eq!(comparison["basis"], json!("similar_recent"));
    let candidates = comparison["candidates"].as_array().expect("candidates");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["activity_id"], json!(5));
    // The race was 15 s/km faster.
    assert_eq!(candidates[0]["pace_delta_s_per_km"], json!(-15.0));
    assert_eq!(candidates[0]["hr_delta_bpm"], json!(15.0));
}

#[test]
fn the_statistics_cover_the_range() {
    let tools = common::tools();
    let answer = tools
        .call(
            "training_statistics",
            &json!({ "from": "2026-10-01", "to": "2026-10-28" }),
        )
        .expect("the statistics");
    assert_envelope(&answer, "training_statistics");

    assert_eq!(answer["data"]["totals"]["runs"], json!(4));
    assert_eq!(answer["data"]["totals"]["distance_m"], json!(44_000.0));
    assert_eq!(answer["data"]["totals"]["elevation_gain_m"], json!(506.0));

    let series = answer["data"]["series"].as_array().expect("series");
    assert!(!series.is_empty());
    assert!(series[0]["period"].is_string());
    assert!(series[0]["run_count"].is_number());

    // The zones say how they were worked out.
    let zones = answer["data"]["hr_zones"].as_object().expect("zones");
    assert_eq!(zones["method"], json!("estimated"));
    assert_eq!(zones["zones"].as_array().expect("zones").len(), 5);

    // The records are the best of the whole history.
    assert!(!answer["data"]["records"]
        .as_array()
        .expect("records")
        .is_empty());
}

#[test]
fn the_plan_context_holds_the_week_and_the_verdicts() {
    let tools = common::tools();
    let answer = tools
        .call(
            "plan_context",
            &json!({ "view": "week", "date": "2026-10-28" }),
        )
        .expect("the plan");
    assert_envelope(&answer, "plan_context");

    let week = answer["data"]["week"].as_object().expect("week");
    assert_eq!(week["id"], json!(1));
    assert_eq!(week["week_number"], json!(4));
    assert_eq!(week["start_date"], json!("2026-10-26"));
    assert_eq!(week["end_date"], json!("2026-11-01"));
    assert_eq!(week["planned_km"], json!(16.0));
    assert_eq!(week["actual_km"], json!(10.0));

    let sessions = week["sessions"].as_array().expect("sessions");
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0]["verdict"], json!("hit"));
    assert_eq!(sessions[0]["activity"]["id"], json!(1));
    assert_eq!(sessions[1]["verdict"], json!("missed"));
    assert_eq!(sessions[1]["cause"], json!("The session was missed."));
    assert_eq!(sessions[1]["cause_confidence"], json!(0.6));

    // The adherence of the range.
    assert_eq!(answer["data"]["adherence"]["overall_pct"], json!(50.0));
}

#[test]
fn the_plan_context_reports_the_day_as_a_suggestion() {
    let tools = common::tools();
    let answer = tools
        .call("plan_context", &json!({ "view": "current" }))
        .expect("the plan");
    assert_envelope(&answer, "plan_context");
    assert_eq!(answer["data"]["has_plan"], json!(true));
    assert_eq!(answer["data"]["block"]["name"], json!("Autumn base"));
    assert_eq!(answer["data"]["today"]["date"], json!("2026-10-28"));
    assert!(answer["data"]["today"]["suggestion"].is_string());
    assert!(answer["data"]["today"]["evidence"].is_string());
    // The recommendation is a suggestion, not a command.
    assert!(answer["data"]["today"].get("command").is_none());
}

#[test]
fn the_physiology_context_reads_the_days() {
    let tools = common::tools();
    let answer = tools
        .call(
            "physiology_context",
            &json!({ "from": "2026-10-15", "to": "2026-10-28" }),
        )
        .expect("the physiology");
    assert_envelope(&answer, "physiology_context");

    let days = answer["data"]["days"].as_array().expect("days");
    assert_eq!(days.len(), 7);
    assert_eq!(days[0]["date"], json!("2026-10-15"));
    assert_eq!(days[0]["hrv"]["value"], json!(54.0));
    assert_eq!(days[0]["hrv"]["status"], json!("BALANCED"));
    assert_eq!(days[0]["sleep"]["sleep_s"], json!(25_200.0));
    assert_eq!(days[0]["resting_hr"]["value"], json!(48.0));
    assert_eq!(days[0]["readiness"]["score"], json!(68.0));
    assert_eq!(days[0]["readiness"]["grade"], json!("good"));

    // The drivers are the three that matter most.
    let drivers = days[0]["readiness"]["drivers"].as_array().expect("drivers");
    assert_eq!(drivers.len(), 3);
    assert_eq!(drivers[0]["key"], json!("sleep"));

    // The days are in order.
    let dates: Vec<&str> = days.iter().filter_map(|day| day["date"].as_str()).collect();
    let mut sorted = dates.clone();
    sorted.sort_unstable();
    assert_eq!(dates, sorted);
}

#[test]
fn the_fingerprint_trends_group_the_runs() {
    let tools = common::tools();
    let answer = tools
        .call(
            "fingerprint_trends",
            &json!({ "from": "2026-07-07", "to": "2026-10-28", "group_by": "run" }),
        )
        .expect("the fingerprint");
    assert_envelope(&answer, "fingerprint_trends");

    let series = answer["data"]["series"].as_array().expect("series");
    assert_eq!(series.len(), 4);
    assert_eq!(series[0]["runs"], json!(1));
    assert_eq!(series[0]["period"], json!("2026-10-15"));
    let metrics = series[0]["metrics"].as_object().expect("metrics");
    assert_eq!(metrics["cadence_cv"]["median"], json!(1.8));
    assert_eq!(metrics["cadence_cv"]["runs"], json!(1));

    // The trend of a metric over the whole range.
    let trends = answer["data"]["trends"].as_object().expect("trends");
    assert!(trends.contains_key("decoupling_pct"));
}

#[test]
fn the_fingerprint_trends_can_be_grouped_by_week() {
    let tools = common::tools();
    let answer = tools
        .call(
            "fingerprint_trends",
            &json!({ "from": "2026-07-07", "to": "2026-10-28", "group_by": "week" }),
        )
        .expect("the fingerprint");
    let series = answer["data"]["series"].as_array().expect("series");
    assert!(!series.is_empty());
    // The 15th, the 20th, the 24th and the 27th are three weeks.
    let weeks: Vec<&str> = series
        .iter()
        .filter_map(|item| item["period"].as_str())
        .collect();
    assert!(weeks.iter().all(|week| week.contains("-W")));
}

#[test]
fn the_fingerprint_trends_can_report_the_zones() {
    let tools = common::tools();
    let answer = tools
        .call(
            "fingerprint_trends",
            &json!({
                "from": "2026-07-07",
                "to": "2026-10-28",
                "metrics": ["zones"],
                "group_by": "week",
            }),
        )
        .expect("the fingerprint");
    let zones = answer["data"]["zones"].as_object().expect("zones");
    assert_eq!(zones["zones"].as_array().expect("zones").len(), 5);
    assert!(zones["total_time_s"].as_f64().expect("total") > 0.0);
}

#[test]
fn the_search_can_ask_for_every_sport() {
    let tools = common::tools();
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "to": "2026-10-28", "sport": "all" }),
        )
        .expect("the search");
    let items = answer["data"]["items"].as_array().expect("items");
    // The four runs of the period and the ride: "all" is the athlete's own
    // sports, not a guess about what sports exist.
    assert_eq!(items.len(), 5);
    assert!(items
        .iter()
        .any(|item| item["sport_type"] == json!("cycling")));
    let limitations = answer["limitations"].as_array().expect("limitations");
    assert!(limitations
        .iter()
        .any(|text| text.as_str().expect("text").contains("1 other sport")));
}

#[test]
fn the_comparison_puts_two_runs_on_one_axis() {
    let tools = common::tools();
    let answer = tools
        .call("compare_runs", &json!({ "activity_ids": [1, 5] }))
        .expect("the comparison");
    assert_envelope(&answer, "compare_runs");

    let runs = answer["data"]["runs"].as_array().expect("runs");
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0]["id"], json!(1));
    assert_eq!(runs[1]["id"], json!(5));
    assert_eq!(runs[0]["values"]["decoupling_pct"], json!(4.1));

    // The profile has the default number of points, on a percentage axis.
    let profile = runs[0]["profile"].as_array().expect("profile");
    assert_eq!(profile.len(), 120);
    assert_eq!(profile[0]["pct"], json!(0.0));
    assert_eq!(profile[119]["pct"], json!(100.0));
    // At the router's own resolution the profile is handed back untouched:
    // the first and the last point are exactly the router's.
    assert_eq!(profile[0]["hr"], json!(145.0));
    assert_eq!(profile[119]["hr"], json!(157.0));
    let middle = profile[60]["hr"].as_f64().expect("hr");
    assert!(
        (middle - 151.05042).abs() < 0.001,
        "the router's own value, unrounded: {middle}"
    );
}

#[test]
fn the_comparison_resamples_to_the_number_of_points_asked_for() {
    let tools = common::tools();
    let answer = tools
        .call(
            "compare_runs",
            &json!({ "activity_ids": [1, 5], "points": 20 }),
        )
        .expect("the comparison");
    let runs = answer["data"]["runs"].as_array().expect("runs");
    let profile = runs[0]["profile"].as_array().expect("profile");
    assert_eq!(profile.len(), 20);
    assert_eq!(profile[0]["pct"], json!(0.0));
    assert_eq!(profile[19]["pct"], json!(100.0));
    // The ends are the router's own points; the middle is interpolated.
    assert_eq!(profile[0]["hr"], json!(145.0));
    assert_eq!(profile[19]["hr"], json!(157.0));
    let middle = profile[10]["hr"].as_f64().expect("hr");
    assert!(
        middle > 145.0 && middle < 157.0,
        "the middle is interpolated: {middle}"
    );
    // And the answer says that the profile was resampled, and from what.
    assert_eq!(answer["coverage"]["source_profile_points"], json!(120));
    assert_eq!(answer["coverage"]["profile_points"], json!(20));
    let limitations = answer["limitations"].as_array().expect("limitations");
    assert!(limitations
        .iter()
        .any(|text| text.as_str().expect("text").contains("resampled")));
}

#[test]
fn the_comparison_refuses_a_duplicated_run() {
    let tools = common::tools();
    let error = tools
        .call("compare_runs", &json!({ "activity_ids": [1, 1] }))
        .expect_err("the comparison");
    assert_eq!(error.tool(), "compare_runs");
    assert!(error.to_string().contains("twice"));
}

#[test]
fn every_tool_is_announced_with_a_strict_schema() {
    let tools = common::tools();
    let listed = tools.list();
    assert_eq!(listed.len(), 9);
    for tool in listed {
        assert_eq!(tool.input_schema["type"], json!("object"));
        assert_eq!(tool.input_schema["additionalProperties"], json!(false));
        assert!(!tool.description.is_empty());
    }
}

#[test]
fn a_run_that_does_not_exist_is_an_error_and_not_a_panic() {
    let tools = common::tools();
    let error = tools
        .call("get_activity", &json!({ "activity_id": 999_999 }))
        .expect_err("the activity");
    assert_eq!(error.tool(), "get_activity");
    assert_eq!(error.code(), "api_error");
    assert!(error.to_string().contains("404"));
}

#[test]
fn an_unknown_argument_is_refused_and_named() {
    let tools = common::tools();
    let error = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "colour": "red" }),
        )
        .expect_err("the search");
    assert_eq!(error.code(), "invalid_params");
    assert!(error.to_string().contains("colour"));
}

#[test]
fn a_date_that_is_not_a_date_is_refused() {
    let tools = common::tools();
    let error = tools
        .call("search_activities", &json!({ "from": "01/10/2026" }))
        .expect_err("the search");
    assert_eq!(error.code(), "invalid_params");
    assert!(error.to_string().contains("YYYY-MM-DD"));
}

#[test]
fn the_router_and_the_tools_count_from_the_same_day() {
    use std::sync::Arc;

    // The fixture's plan week is the week of 2026-10-26.
    let store = common::store();
    let day = chrono::NaiveDate::parse_from_str("2026-10-28", "%Y-%m-%d").expect("the day");
    let tools = stride_mcp::Tools::over(Arc::new(store), day);

    // The tools count from that day: the digest's period ends on it.
    let answer = tools
        .call("training_digest", &json!({ "period": "7d" }))
        .expect("the digest");
    assert_eq!(answer["coverage"]["to"], json!("2026-10-28"));

    // And the router counts from it too: the plan's current week is the one
    // that contains it.
    let answer = tools
        .call("plan_context", &json!({ "view": "current" }))
        .expect("the plan");
    assert_eq!(answer["data"]["week"]["week_number"], json!(4));
    assert_eq!(answer["data"]["week"]["start_date"], json!("2026-10-26"));
}

#[test]
fn an_unknown_tool_is_an_error_and_not_a_panic() {
    let tools = common::tools();
    let error = tools
        .call("delete_everything", &json!({}))
        .expect_err("the tool");
    assert_eq!(error.code(), "unknown_tool");
}
