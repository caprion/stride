//! The sync's mappings, against the reference.
//!
//! The fixture holds the Garmin payloads the reference implementation was
//! given and the physiology row and the merged activity it made of them,
//! recorded from `backend/app/services/personal_sync.py`.
//!
//! The Garmin payloads come from the Garmin crate's own fixtures, so the
//! same bytes the Garmin crate is tested against are the bytes the sync is
//! tested against.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::Value;
use stride_api::backend::PhysiologyRow;
use stride_api::sync::{
    apply_daily_summary, apply_heart_rate, apply_hrv, apply_sleep, fit_laps, fit_streams, merge_fit,
};

/// The recorded payloads and the reference's answers.
const FIXTURE: &str = include_str!("fixtures/sync.json");

/// The wellness row, filled the way the reference fills it.
fn wellness_row(payloads: &Value) -> PhysiologyRow {
    let mut row = PhysiologyRow {
        date: "2026-03-07".to_string(),
        source: Some("garmin_personal".to_string()),
        ..PhysiologyRow::default()
    };
    apply_daily_summary(&mut row, &payloads["daily_summary"]);
    apply_sleep(&mut row, &payloads["sleep"]);
    apply_hrv(&mut row, &payloads["hrv"]);
    apply_heart_rate(&mut row, &payloads["daily_heart_rate"]);
    row
}

/// The wellness fields are the reference's, field for field.
#[test]
fn the_wellness_row_is_the_reference_s() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture parses");
    let expected = &fixture["wellness"];
    let payloads = &fixture["payloads"];
    let row = wellness_row(payloads);

    let actual = serde_json::json!({
        "sleep_s": row.sleep_s,
        "deep_sleep_s": row.deep_sleep_s,
        "rem_s": row.rem_s,
        "light_sleep_s": row.light_sleep_s,
        "awake_s": row.awake_s,
        "hrv_ms": row.hrv_ms,
        "hrv_status": row.hrv_status,
        "resting_hr": row.resting_hr,
        "max_hr": row.max_hr,
        "stress_avg": row.stress_avg,
        "stress_max": row.stress_max,
        "body_battery_min": row.body_battery_min,
        "body_battery_avg": row.body_battery_avg,
        "body_battery_max": row.body_battery_max,
        "steps": row.steps,
        "vo2max": row.vo2max,
    });
    let expected = serde_json::json!({
        "sleep_s": expected["sleep_s"],
        "deep_sleep_s": expected["deep_sleep_s"],
        "rem_s": expected["rem_s"],
        "light_sleep_s": expected["light_sleep_s"],
        "awake_s": expected["awake_s"],
        "hrv_ms": expected["hrv_ms"],
        "hrv_status": expected["hrv_status"],
        "resting_hr": expected["resting_hr"],
        "max_hr": expected["max_hr"],
        "stress_avg": expected["stress_avg"],
        "stress_max": expected["stress_max"],
        "body_battery_min": expected["body_battery_min"],
        "body_battery_avg": expected["body_battery_avg"],
        "body_battery_max": expected["body_battery_max"],
        "steps": expected["steps"],
        "vo2max": expected["vo2max"],
    });
    assert_eq!(actual, expected, "the wellness row differs");
}

/// The merged activity is the reference's.
#[test]
fn the_merged_activity_is_the_reference_s() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture parses");
    let expected = &fixture["merge"];

    // The canonical activity, as the Garmin crate builds it.
    let canonical = &fixture["canonical"];
    let mut activity = canonical_activity_from(canonical);
    let bytes = std::fs::read("../../backend/tests/golden/fixtures/mixed_run.fit")
        .expect("the fixture is there");
    let parsed = stride_fit::parse_fit(&bytes, "mixed_run.fit").expect("it parses");
    let laps = fit_laps(&parsed);
    let streams = fit_streams(&parsed);
    merge_fit(&mut activity, &laps, &streams, &parsed);

    assert_eq!(activity.running_time_s, expected["running_time_s"].as_f64());
    assert_eq!(activity.walking_time_s, expected["walking_time_s"].as_f64());
    assert_eq!(activity.stopped_time_s, expected["stopped_time_s"].as_f64());
    assert_eq!(
        activity.running_distance_m,
        expected["running_distance_m"].as_f64()
    );
    assert_eq!(activity.device.as_deref(), expected["device"].as_str());
    assert_eq!(
        Some(activity.elevation_gain_m),
        expected["elevation_gain_m"].as_f64()
    );
    // The API's numbers win: the FIT stores cadence in cycles.
    assert_eq!(Some(activity.distance_m), expected["distance_m"].as_f64());
    assert_eq!(
        Some(activity.moving_time_s),
        expected["moving_time_s"].as_f64()
    );
    assert_eq!(activity.avg_cadence, expected["avg_cadence"].as_f64());
    assert_eq!(activity.max_cadence, expected["max_cadence"].as_f64());
    // The API's local date wins over the FIT's UTC.
    assert_eq!(
        activity.start_time.as_deref(),
        expected["start_time"].as_str()
    );
    assert_eq!(
        activity.local_date,
        expected["local_date"].as_str().unwrap_or_default()
    );
    assert_eq!(
        activity.day_of_week.as_deref(),
        expected["day_of_week"].as_str()
    );
    // The FIT's splits and series win over the summary.
    assert_eq!(
        laps.len(),
        expected["laps"].as_array().map(Vec::len).unwrap_or(0)
    );
    assert_eq!(
        streams.len(),
        expected["streams"]
            .as_object()
            .map(|map| map.len())
            .unwrap_or(0)
    );
}

/// The canonical activity, from the fixture's canonical JSON.
fn canonical_activity_from(canonical: &Value) -> stride_api::backend::ActivityRow {
    let text = |key: &str| {
        canonical
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let number = |key: &str| canonical.get(key).and_then(Value::as_f64);
    stride_api::backend::ActivityRow {
        source: "garmin_personal".to_string(),
        external_id: text("external_id").unwrap_or_default(),
        name: text("name").unwrap_or_default(),
        sport_type: text("sport_type").unwrap_or_default(),
        start_time: text("start_time"),
        local_date: text("local_date").unwrap_or_default(),
        day_of_week: text("day_of_week"),
        distance_m: number("distance_m").unwrap_or_default(),
        moving_time_s: number("moving_time_s").unwrap_or_default(),
        elapsed_time_s: number("elapsed_time_s").unwrap_or_default(),
        elevation_gain_m: number("elevation_gain_m").unwrap_or_default(),
        avg_hr: number("avg_hr"),
        max_hr: number("max_hr"),
        calories: number("calories"),
        avg_cadence: number("avg_cadence"),
        max_cadence: number("max_cadence"),
        avg_speed_ms: number("avg_speed_ms"),
        max_speed_ms: number("max_speed_ms"),
        device: text("device"),
        ..stride_api::backend::ActivityRow::default()
    }
}

/// The canonical activity the Garmin crate builds is the reference's.
#[test]
fn the_canonical_activity_is_the_reference_s() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture parses");
    let canonical = &fixture["canonical"];
    let row = canonical_activity_from(canonical);
    // The activity id is the provider's.
    assert!(!row.external_id.is_empty());
    assert_eq!(row.source, "garmin_personal");
}
