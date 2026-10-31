// The test code unwraps and expects: it is the test, and the workspace
// denies those lints in library code only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The plan is the **active block** and nothing else.
//!
//! A plan that was imported twice leaves two blocks with the same dates:
//! the old one archived, the new one active. Reading every session in the
//! range counts each one twice — the week shows fourteen sessions for seven
//! days, the planned distance doubles, and the adherence is halved.
//!
//! The reference filters by the active block; the port has to do the same.

use std::sync::Arc;

use serde_json::Value;
use stride_api::backend::{PlanBlockRow, PlanSessionRow};
use stride_api::{Api, MemBackend};

/// The API over two blocks with the same dates: the archived one and the
/// active one, each with a session for the same day.
fn api() -> Api {
    let backend = MemBackend::new();
    for (id, status) in [(1i64, "archived"), (2, "active")] {
        backend.add_block(PlanBlockRow {
            id,
            name: format!("Block {id}"),
            start_date: "2026-10-26".to_string(),
            end_date: Some("2026-11-01".to_string()),
            status: status.to_string(),
            source: "imported".to_string(),
            ..PlanBlockRow::default()
        });
        backend.add_session(PlanSessionRow {
            id: id * 10,
            block_id: Some(id),
            date: "2026-10-29".to_string(),
            session_type: "easy".to_string(),
            title: Some(format!("Session of block {id}")),
            target_distance_m: Some(3000.0),
            ..PlanSessionRow::default()
        });
    }
    Api::new(Arc::new(stride_api::CoreBackend::new(Arc::new(backend))))
}

/// The week of an active block and an archived one with the same dates
/// shows the active block's sessions, once each.
#[test]
fn a_week_reads_the_active_block_only() {
    let api = api();
    let week = api
        .call("GET", "/api/v1/plan/week?date=2026-10-28", None)
        .expect("the week answers");
    let sessions = week["sessions"].as_array().expect("the sessions");
    assert_eq!(
        sessions.len(),
        1,
        "the archived block's session is counted as well: {week}"
    );
    assert_eq!(
        sessions[0]["title"], "Session of block 2",
        "the wrong block's session is shown: {week}"
    );
}

/// The block the app is showing is the active one.
#[test]
fn the_current_block_is_the_active_one() {
    let api = api();
    let current = api
        .call("GET", "/api/v1/plan/current", None)
        .expect("the current plan answers");
    let block = &current["block"]["block"];
    assert_eq!(block["status"], "active", "{current}");
    assert_eq!(block["id"], 2, "{current}");
}

/// The verdicts of a date range count the active block's sessions once.
#[test]
fn the_results_count_the_active_block_only() {
    let api = api();
    let results = api
        .call(
            "GET",
            "/api/v1/plan/results?from=2026-10-26&to=2026-11-01",
            None,
        )
        .expect("the results answer");
    let items = results["items"].as_array().expect("the items");
    assert_eq!(
        items.len(),
        1,
        "the archived block's session is counted as well: {results}"
    );
}

/// The shape of the answer does not change.
#[test]
fn the_week_answers_the_same_shape() {
    let api = api();
    let week: Value = api
        .call("GET", "/api/v1/plan/week?date=2026-10-28", None)
        .expect("the week answers");
    for key in [
        "actual_km",
        "adherence_pct",
        "completed_count",
        "phase",
        "planned_count",
        "planned_km",
        "sessions",
    ] {
        assert!(week.get(key).is_some(), "the week has no {key}: {week}");
    }
}
