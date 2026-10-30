// The test code unwraps and expects: it is the test, and the workspace
// denies those lints in library code only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The plan engine, measured against the reference.
//!
//! The API golden file records what the **Python reference** answers, and
//! the API golden test feeds those answers to the Rust router: for the plan
//! routes the router shapes an answer it is given, because the domain logic
//! lives in `stride-core` and the reference has the domain logic of its
//! own.
//!
//! That means the recorded plan answers were never compared with anything.
//! A recording that had gone stale — because the definition of adherence
//! changed and the file was not re-recorded — sat in the repository looking
//! like a contract, and the gate was green.
//!
//! This test closes that gap: it builds the **Rust** plan engine over the
//! same rows and asks it the same questions, and it compares the answers
//! with the recorded ones, number for number. If the reference and the
//! port disagree — or if the recording has gone stale — this fails.
//!
//! The routes compared are the ones whose answer is a fact about the data
//! rather than about the day: the adherence roll-up and the verdicts. The
//! route that answers "today" is not among them, because its answer is
//! about the date the recording was made.

use std::sync::Arc;

use serde_json::{json, Value};

use stride_api::backend::{
    ActivityRow, DailyMetricRow, LapRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow, PlanSessionRow,
    PlanWeekRow, StreamRow, ZoneRow,
};
use stride_api::{Api, CoreBackend, MemBackend};

/// The recorded golden file.
const GOLDEN: &str = include_str!("fixtures/api_golden.json");

/// The day the recording was made.
fn recorded_today(golden: &Value) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(
        golden["today"].as_str().unwrap_or("2026-01-01"),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default())
}

/// The backend the recording used, with the same rows.
fn backend(golden: &Value) -> MemBackend {
    let backend = MemBackend::new();
    let rows = &golden["rows"];

    for row in rows["activities"].as_array().cloned().unwrap_or_default() {
        backend.add_activity(serde_json::from_value::<ActivityRow>(row).expect("activity row"));
    }
    for row in rows["laps"].as_array().cloned().unwrap_or_default() {
        backend.add_lap(serde_json::from_value::<LapRow>(row).expect("lap row"));
    }
    for row in rows["streams"].as_array().cloned().unwrap_or_default() {
        backend.add_stream(serde_json::from_value::<StreamRow>(row).expect("stream row"));
    }
    for row in rows["zones"].as_array().cloned().unwrap_or_default() {
        backend.add_zone(serde_json::from_value::<ZoneRow>(row).expect("zone row"));
    }
    for row in rows["physiology"].as_array().cloned().unwrap_or_default() {
        backend
            .add_physiology(serde_json::from_value::<PhysiologyRow>(row).expect("physiology row"));
    }
    for row in rows["daily_metrics"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        backend.add_daily_metric(serde_json::from_value::<DailyMetricRow>(row).expect("daily row"));
    }
    for row in rows["plan_blocks"].as_array().cloned().unwrap_or_default() {
        backend.add_block(serde_json::from_value::<PlanBlockRow>(row).expect("block row"));
    }
    for row in rows["plan_phases"].as_array().cloned().unwrap_or_default() {
        backend.add_phase(serde_json::from_value::<PlanPhaseRow>(row).expect("phase row"));
    }
    for row in rows["plan_weeks"].as_array().cloned().unwrap_or_default() {
        backend.add_week(serde_json::from_value::<PlanWeekRow>(row).expect("week row"));
    }
    for row in rows["plan_sessions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        backend.add_session(serde_json::from_value::<PlanSessionRow>(row).expect("session row"));
    }
    backend
}

/// The recorded answer of one route.
fn recorded(golden: &Value, path: &str) -> Option<Value> {
    golden["cases"]
        .as_array()
        .and_then(|cases| cases.iter().find(|case| case["path"] == path))
        .map(|case| case["response"].clone())
}

/// The Rust answer of one route, computed rather than recorded.
fn computed(golden: &Value, path: &str) -> Value {
    let backend = backend(golden);
    let api = Api::with_today(
        Arc::new(CoreBackend::new(Arc::new(backend))),
        recorded_today(golden),
    );
    api.call("GET", path, None)
        .unwrap_or_else(|error| panic!("{path} answers: {error}"))
}

/// The numbers of two answers, with the keys the two implementations spell
/// differently left out.
fn numbers(value: &Value) -> Value {
    let mut out = serde_json::Map::new();
    if let Some(object) = value.as_object() {
        for (key, item) in object {
            if item.is_number() || item.is_null() || item.is_boolean() {
                out.insert(key.clone(), item.clone());
            }
        }
    }
    Value::Object(out)
}

/// The adherence roll-up is the same number in both implementations.
#[test]
fn the_adherence_the_rust_port_computes_is_the_adherence_the_reference_computed() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let expected = recorded(&golden, "/api/v1/plan/adherence").expect("the recorded adherence");
    let actual = computed(&golden, "/api/v1/plan/adherence");
    assert_eq!(
        numbers(&actual),
        numbers(&expected),
        "the Rust port and the reference disagree about the adherence"
    );
}

/// The week rows are the same in both implementations.
#[test]
fn the_adherence_weeks_are_the_same_in_both_implementations() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let expected = recorded(&golden, "/api/v1/plan/adherence").expect("the recorded adherence");
    let actual = computed(&golden, "/api/v1/plan/adherence");
    let expected_rows = expected["rows"].as_array().cloned().unwrap_or_default();
    let actual_rows = actual["rows"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        actual_rows.len(),
        expected_rows.len(),
        "the two implementations disagree about how many weeks there are"
    );
    for (index, (expected, actual)) in expected_rows.iter().zip(actual_rows.iter()).enumerate() {
        assert_eq!(
            numbers(actual),
            numbers(expected),
            "week {index} differs: {actual} against {expected}"
        );
    }
}

/// The verdicts are the same in both implementations.
#[test]
fn the_plan_verdicts_are_the_same_in_both_implementations() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let path = "/api/v1/plan/results";
    let expected = recorded(&golden, path).expect("the recorded verdicts");
    let actual = computed(&golden, path);
    let expected_items = expected["items"].as_array().cloned().unwrap_or_default();
    let actual_items = actual["items"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        actual_items.len(),
        expected_items.len(),
        "the two implementations disagree about how many verdicts there are"
    );
    for (index, (expected, actual)) in expected_items.iter().zip(actual_items.iter()).enumerate() {
        assert_eq!(
            actual["verdict"],
            expected["verdict"],
            "verdict {index} differs: {actual} against {expected}"
        );
        assert_eq!(
            actual["adherence_score"],
            expected["adherence_score"],
            "the adherence score of verdict {index} differs"
        );
    }
}

/// The recording is not stale: the adherence in the file is the adherence
/// the reference computes today.
///
/// This is the check that would have caught the stale file: the reference's
/// own numbers, re-read from the recorded rows, have to be the numbers in
/// the file.
#[test]
fn the_recorded_adherence_is_the_one_the_reference_computes() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let expected = recorded(&golden, "/api/v1/plan/adherence").expect("the recorded adherence");
    let actual = computed(&golden, "/api/v1/plan/adherence");
    assert_eq!(
        json!(actual["overall_pct"]),
        json!(expected["overall_pct"]),
        "the recorded adherence is stale: the reference computes {} and the file says {}",
        actual["overall_pct"],
        expected["overall_pct"]
    );
}
