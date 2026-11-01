//! "No value" is `null`, never `0`.
//!
//! The contract's §2: `null` is the only representation of "absent". There
//! is no sentinel value in any stored or returned structure: a missing
//! heart rate is `null`, never `0`.
//!
//! This test walks the routes that carry measurements and holds every one
//! of them to that rule.

mod common;

use serde_json::Value;

/// The value of a field, or the marker for a field that is not there.
fn value_of(body: &Value, path: &[&str]) -> Value {
    let mut current = body;
    for step in path {
        current = match current.get(step) {
            Some(value) => value,
            None => return Value::String("<the field is not there>".to_string()),
        };
    }
    current.clone()
}

#[test]
fn a_run_with_no_heart_rate_carries_null() {
    let harness = common::Harness::empty("no-value-run");
    let id = harness.write_run(common::run_without_readings(
        "a-run-without-readings",
        "2026-05-10",
        10_000.0,
        3_000.0,
    ));

    let body = harness.ok_get(&format!("/activities/{id}"));
    assert!(
        value_of(&body, &["avg_hr"]).is_null(),
        "the average heart rate is not null: {}",
        value_of(&body, &["avg_hr"])
    );
    assert!(
        value_of(&body, &["max_hr"]).is_null(),
        "the maximum heart rate is not null: {}",
        value_of(&body, &["max_hr"])
    );
    assert!(
        value_of(&body, &["avg_cadence"]).is_null(),
        "the cadence is not null: {}",
        value_of(&body, &["avg_cadence"])
    );
    assert!(
        value_of(&body, &["story"]).is_null(),
        "the story is not null: {}",
        value_of(&body, &["story"])
    );

    let feed = harness.ok_get("/activities");
    let run = &feed["activities"][0];
    assert!(run["avg_hr"].is_null(), "the feed's heart rate is not null");
    assert!(run["max_hr"].is_null(), "the feed's heart rate is not null");
}

#[test]
fn a_run_with_no_distance_has_no_pace() {
    let harness = common::Harness::empty("no-value-pace");
    let id = harness.write_run(common::run_without_readings(
        "a-run-with-no-distance",
        "2026-05-10",
        0.0,
        0.0,
    ));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert!(
        value_of(&body, &["pace_s_per_km"]).is_null(),
        "the pace of a run with no distance is not null: {}",
        value_of(&body, &["pace_s_per_km"])
    );
}

#[test]
fn a_day_with_no_readings_carries_null() {
    let harness = common::Harness::empty("no-value-wellness");
    harness.write_wellness(common::wellness_without_readings("2026-05-10"));

    let body = harness.ok_get("/physiology/daily");
    let day = &body["days"][0];
    for field in [
        "sleep_s",
        "deep_sleep_s",
        "rem_sleep_s",
        "light_sleep_s",
        "awake_s",
        "hrv_ms",
        "resting_hr",
        "stress_avg",
    ] {
        assert!(
            day[field].is_null(),
            "the {field} is not null: {}",
            day[field]
        );
    }

    let sleep = harness.ok_get("/physiology/sleep");
    assert!(sleep["days"][0]["sleep_s"].is_null());

    let hrv = harness.ok_get("/physiology/hrv");
    assert!(hrv["days"][0]["hrv_ms"].is_null());
    assert!(hrv["days"][0]["resting_hr"].is_null());
}

#[test]
fn a_day_without_a_comparison_says_so_and_shows_the_raw_value() {
    let harness = common::Harness::empty("no-value-comparison");
    // One night is not enough for a band.
    for date in ["2026-05-11", common::TODAY] {
        harness.write_wellness(common::wellness_with(
            date,
            Some(22_200.0),
            Some(48.0),
            Some(63.0),
        ));
    }

    let body = harness.ok_get("/physiology/vitals");
    let vitals = body["vitals"].as_array().unwrap();
    assert_eq!(vitals[0]["value_text"].as_str(), Some("6 h 10 m"));
    assert!(vitals[0]["direction_text"].is_null());
    assert!(vitals[0]["deviation_text"].is_null());
    assert!(vitals[0]["travel_text"].is_null());
    assert_eq!(
        vitals[0]["status_text"].as_str(),
        Some("not enough data for a comparison yet")
    );
    assert!(vitals[0]["band"].is_null());
}

#[test]
fn a_run_with_no_readings_still_carries_the_numbers_it_has() {
    let harness = common::Harness::empty("no-value-numbers");
    let id = harness.write_run(common::run_without_readings(
        "a-run-without-readings",
        "2026-05-10",
        10_000.0,
        3_000.0,
    ));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert_eq!(body["distance_m"].as_f64(), Some(10_000.0));
    assert_eq!(body["moving_time_s"].as_f64(), Some(3_000.0));
    // A flat run has no climbing: that is a measurement of zero, not an
    // absent measurement.
    assert_eq!(body["elevation_gain_m"].as_f64(), Some(0.0));
}

#[test]
fn an_empty_database_answers_with_empty_lists_and_not_with_zeros() {
    let harness = common::Harness::empty("no-value-empty");
    let feed = harness.ok_get("/activities");
    assert!(feed["activities"].as_array().unwrap().is_empty());
    assert!(feed["next_cursor"].is_null());

    let vitals = harness.ok_get("/physiology/vitals");
    assert!(vitals["vitals"].as_array().unwrap().len() == 3);
    assert!(vitals["patterns"].as_array().unwrap().is_empty());
}
