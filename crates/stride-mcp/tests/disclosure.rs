//! The disclosure rules, as tests.
//!
//! A tool's answer is a disclosure: it ends up in a model's context. These
//! tests are the rules, enforced:
//!
//! * no path, no credential, no Garmin identifier, no device name and no
//!   note the athlete wrote;
//! * every answer inside the size bound, even when the history is large.

mod common;

use std::sync::Arc;

use serde_json::{json, Value};
use stride_api::backend::{ActivityRow, LapRow, StreamRow};
use stride_api::mem::MemBackend;
use stride_mcp::{StrideApi, Tools, MAX_OUTPUT_BYTES};

/// The calls every tool is asked for.
fn calls() -> Vec<(&'static str, Value)> {
    vec![
        ("training_digest", json!({ "period": "28d" })),
        (
            "search_activities",
            json!({ "from": "2026-10-01", "to": "2026-10-28" }),
        ),
        (
            "get_activity",
            json!({ "activity_id": 1, "include": ["laps", "streams", "zones", "route"] }),
        ),
        (
            "activity_analysis",
            json!({ "activity_id": 1, "compare_with": "similar_recent" }),
        ),
        (
            "training_statistics",
            json!({ "from": "2026-10-01", "to": "2026-10-28" }),
        ),
        ("plan_context", json!({ "view": "current" })),
        (
            "physiology_context",
            json!({ "from": "2026-10-15", "to": "2026-10-28" }),
        ),
        (
            "fingerprint_trends",
            json!({ "from": "2026-07-07", "to": "2026-10-28" }),
        ),
        ("compare_runs", json!({ "activity_ids": [1, 5] })),
    ]
}

/// Every string in a value.
fn strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                strings(item, out);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                strings(item, out);
            }
        }
        _ => {}
    }
}

/// The strings that must never reach a model's context.
fn forbidden() -> Vec<&'static str> {
    vec![
        // A path, or the shape of one.
        "/home/",
        "/Users/",
        "/api/v1",
        "/var/",
        "C:\\",
        ".sqlite",
        ".db",
        ".fit",
        // A credential.
        "api_key",
        "apikey",
        "personal_key",
        "authorization",
        "bearer",
        "secret",
        "password",
        "token",
        "credential",
        // The athlete's Garmin account.
        "connect.garmin.com",
        "14823947261",
        // The device and the note the athlete wrote.
        "Forerunner",
        "Felt strong after the rest week",
    ]
}

#[test]
fn no_tool_leaks_a_path_or_a_credential() {
    let tools = common::tools();
    for (name, arguments) in calls() {
        let answer = tools.call(name, &arguments).expect("the tool");
        let mut texts = Vec::new();
        strings(&answer, &mut texts);
        for text in &texts {
            for needle in forbidden() {
                assert!(
                    !text.to_lowercase().contains(&needle.to_lowercase()),
                    "{name} leaked {needle:?}: {text:?}"
                );
            }
        }
    }
}

#[test]
fn every_answer_is_inside_the_size_bound() {
    let tools = common::tools();
    for (name, arguments) in calls() {
        let answer = tools.call(name, &arguments).expect("the tool");
        let size = serde_json::to_string(&answer).expect("json").len();
        assert!(
            size <= MAX_OUTPUT_BYTES,
            "{name} answered {size} bytes, over the {MAX_OUTPUT_BYTES} bound"
        );
    }
}

/// A store with a history far larger than the fixture's.
fn big_store() -> MemBackend {
    let store = MemBackend::new();
    for index in 0..400_i64 {
        let id = index + 1;
        store.add_activity(ActivityRow {
            id,
            source: "garmin_personal".into(),
            external_id: format!("9000000000{index:02}"),
            name: format!("Run {index}"),
            sport_type: "running".into(),
            start_time: Some(format!("2026-10-{:02}T06:00:00+00:00", (index % 28) + 1)),
            local_date: format!("2026-10-{:02}", (index % 28) + 1),
            day_of_week: Some("Tuesday".into()),
            distance_m: 8_000.0 + (index % 13) as f64 * 500.0,
            moving_time_s: 2_700.0 + (index % 7) as f64 * 60.0,
            elapsed_time_s: 2_800.0,
            elevation_gain_m: 40.0 + (index % 11) as f64,
            avg_hr: Some(148.0),
            max_hr: Some(172.0),
            avg_cadence: Some(84.0),
            device: Some("Forerunner 965".into()),
            lap_count: 500,
            ..Default::default()
        });
        for lap in 0..500_i64 {
            store.add_lap(LapRow {
                activity_id: id,
                lap_index: lap + 1,
                distance_m: 20.0,
                duration_s: 60.0,
                avg_speed_ms: Some(3.33),
                avg_hr: Some(150.0),
                avg_cadence: Some(85.0),
                elevation_gain_m: Some(1.0),
                start_elevation_m: Some(100.0),
                end_elevation_m: Some(101.0),
                ..Default::default()
            });
        }
        let count = 5_000usize;
        store.add_stream(StreamRow {
            activity_id: id,
            kind: "hr".into(),
            seconds: (0..count).map(|index| index as f64).collect(),
            values: (0..count)
                .map(|index| Some(140.0 + (index % 30) as f64))
                .collect(),
            unit: Some("bpm".into()),
            ..Default::default()
        });
    }
    store
}

fn big_tools() -> Tools {
    let api: Arc<dyn StrideApi> = Arc::new(stride_api::Api::with_today(
        Arc::new(big_store()),
        chrono::NaiveDate::parse_from_str(common::TODAY, "%Y-%m-%d").expect("the day"),
    ));
    Tools::with_today(
        api,
        chrono::NaiveDate::parse_from_str(common::TODAY, "%Y-%m-%d").expect("the day"),
    )
}

#[test]
fn a_large_history_still_answers_inside_the_bound() {
    let tools = big_tools();

    // The search is capped by the limit.
    let answer = tools
        .call(
            "search_activities",
            &json!({ "from": "2026-10-01", "to": "2026-10-28", "limit": 100 }),
        )
        .expect("the search");
    assert_eq!(
        answer["data"]["items"].as_array().expect("items").len(),
        100
    );
    assert!(serde_json::to_string(&answer).expect("json").len() <= MAX_OUTPUT_BYTES);

    // The detail of a run with five hundred splits and five thousand samples
    // is still bounded, and says that it was cut.
    let answer = tools
        .call(
            "get_activity",
            &json!({ "activity_id": 1, "include": ["laps", "streams"] }),
        )
        .expect("the activity");
    assert_eq!(answer["data"]["laps"].as_array().expect("laps").len(), 100);
    let samples = answer["data"]["streams"]["hr"]["samples"]
        .as_array()
        .expect("samples");
    assert!(samples.len() <= 241);
    let limitations = answer["limitations"].as_array().expect("limitations");
    assert!(!limitations.is_empty());
    assert!(serde_json::to_string(&answer).expect("json").len() <= MAX_OUTPUT_BYTES);

    // And the digest, which reads the whole history, is bounded too.
    let answer = tools
        .call("training_digest", &json!({ "period": "28d" }))
        .expect("the digest");
    assert!(serde_json::to_string(&answer).expect("json").len() <= MAX_OUTPUT_BYTES);
}

#[test]
fn the_tool_that_answers_a_year_of_history_is_still_a_page() {
    let tools = big_tools();
    let answer = tools
        .call(
            "training_statistics",
            &json!({ "from": "2025-10-01", "to": "2026-10-28", "group_by": "month" }),
        )
        .expect("the statistics");
    let series = answer["data"]["series"].as_array().expect("series");
    assert!(series.len() <= 13);
    assert!(serde_json::to_string(&answer).expect("json").len() <= MAX_OUTPUT_BYTES);
}
