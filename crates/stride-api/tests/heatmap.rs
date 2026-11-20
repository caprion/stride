//! The heatmap: a grid over the stored positions, computed on the device.
//!
//! The route reads the athlete's own runs and their stored FIT files, and
//! answers with a count of runs per cell. It fetches nothing and sends
//! nothing: the tests below prove it, with the fake Garmin that records
//! every request the router makes.
//!
//! The runs the tests use carry the corpus's own files, whose positions sit
//! on the synthetic anchor in the South Pacific.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;

use common::Harness;
use serde_json::Value;
use stride_store::NewActivity;

/// The corpus file whose positions sit on the synthetic anchor.
const FIT: &str = "real_garmin_run.fit";

/// Write a run whose FIT file is the corpus file named.
fn write_fit_run(harness: &Harness, name: &str, date: &str) -> i64 {
    let fit = common::corpus_fit(name);
    harness.write_run(NewActivity {
        source: "garmin_personal".to_string(),
        external_id: format!("fit-{name}-{date}"),
        name: "a run".to_string(),
        sport_type: "running".to_string(),
        start_time: common::instant(),
        local_date: date.to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        fit_blob: fit,
        created_at: common::instant(),
        updated_at: common::instant(),
    })
}

/// The cells of an answer.
fn cells(body: &Value) -> &Vec<Value> {
    body["cells"].as_array().expect("the answer carries cells")
}

#[test]
fn a_store_with_no_runs_has_no_heatmap() {
    let harness = Harness::empty("heatmap-empty");

    let body = harness.ok_get("/insights/heatmap");

    // No value is the absence of the whole result: there is no grid, and
    // the reason says why.
    assert!(cells(&body).is_empty(), "there are no cells to draw");
    assert_eq!(body["unavailable"], "no_positions");
    assert_eq!(body["runs"], 0);
    assert!(body["extent"].is_null(), "there is no box to draw them in");
    assert!(body["cell_size"].is_null(), "there is no cell to size");
}

#[test]
fn a_run_with_a_route_is_a_cell() {
    let harness = Harness::empty("heatmap-one-run");
    write_fit_run(&harness, FIT, "2026-05-10");

    let body = harness.ok_get("/insights/heatmap");

    assert_eq!(body["runs"], 1, "one run was counted");
    assert_eq!(cells(&body).len(), 1, "the run touched one cell");
    assert_eq!(cells(&body)[0]["runs"], 1, "the cell counts the run once");
    assert!(body["unavailable"].is_null(), "there is a heatmap");
    // The cell's own place in the grid, and its corners.
    assert!(cells(&body)[0]["x"].is_number());
    assert!(cells(&body)[0]["y"].is_number());
    for corner in ["south", "west", "north", "east"] {
        assert!(
            cells(&body)[0][corner].is_number(),
            "the cell carries its {corner}"
        );
    }
    // And the size of a cell, as the core renders it.
    assert_eq!(body["cell_size"]["text"], "2 km");
}

#[test]
fn a_cell_counts_every_run_that_went_through_it() {
    let harness = Harness::empty("heatmap-two-runs");
    write_fit_run(&harness, FIT, "2026-05-10");
    write_fit_run(&harness, FIT, "2026-05-11");

    let body = harness.ok_get("/insights/heatmap");

    assert_eq!(body["runs"], 2, "two runs were counted");
    assert_eq!(cells(&body).len(), 1, "both runs went through one cell");
    assert_eq!(cells(&body)[0]["runs"], 2, "the cell counts both runs");
}

#[test]
fn the_heatmap_asks_nobody_for_anything() {
    let harness = Harness::empty("heatmap-local");
    write_fit_run(&harness, FIT, "2026-05-10");

    let body = harness.ok_get("/insights/heatmap");

    assert_eq!(body["runs"], 1, "the answer is a real one");
    assert_eq!(
        harness.garmin.request_count(),
        0,
        "the heatmap asked for something over the network"
    );
}

#[test]
fn the_heatmap_route_carries_no_client_at_all() {
    // The route's own source: there is no HTTP client, no transport and
    // no Garmin in the path that draws the grid. The grid is made of the
    // files the runs were stored with.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("routes")
        .join("heatmap.rs");
    let source = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => panic!("the route's source could not be read: {error}"),
    };
    for forbidden in [
        "GarminClient",
        "HttpTransport",
        "transport",
        "reqwest",
        "http",
    ] {
        assert!(
            !source.contains(forbidden),
            "the heatmap route mentions {forbidden}"
        );
    }
}

#[test]
fn the_heatmap_leaves_the_store_as_it_was() {
    let harness = Harness::empty("heatmap-read-only");
    write_fit_run(&harness, FIT, "2026-05-10");
    let before = harness.store.count_activities().unwrap_or(-1);

    let body = harness.ok_get("/insights/heatmap");

    assert_eq!(body["runs"], 1);
    assert_eq!(
        harness.store.count_activities().unwrap_or(-1),
        before,
        "the heatmap wrote to the store"
    );
}

#[test]
fn the_same_read_twice_answers_the_same_way() {
    let harness = Harness::empty("heatmap-twice");
    write_fit_run(&harness, FIT, "2026-05-10");

    let first = harness.ok_get("/insights/heatmap");
    let second = harness.ok_get("/insights/heatmap");

    assert_eq!(first, second, "the heatmap answered differently");
}
