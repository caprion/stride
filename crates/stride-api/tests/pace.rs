//! The pace page's answer: the training week by week, the pace at a heart
//! rate, and where the time went by pace.
//!
//! The bands are a property of the window, so the answer names the window
//! it covers: the same runs over a wider window are a different answer.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::Harness;
use serde_json::Value;

/// The rows of the bands, with their times in seconds.
fn band_rows(bands: &Value) -> Vec<(String, f64)> {
    bands["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    (
                        row["label"].as_str().unwrap_or_default().to_string(),
                        row["time"]["raw"].as_f64().unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_bands_are_the_window_s_and_the_window_is_named() {
    let harness = Harness::seeded("pace-bands");
    // A run inside the window, and one before it.
    harness.write_run(common::run_with_fit(
        "a-run-in-the-window",
        "2026-05-04",
        "hr_gps_run.fit",
    ));
    harness.write_run(common::run_with_fit(
        "a-run-before-the-window",
        "2026-04-13",
        "hr_gps_run.fit",
    ));

    let body = harness.ok_get("/insights/pace?weeks=4");

    // The window is the Progress window: the same calendar weeks, the
    // current partial week included.
    assert_eq!(body["window"]["from"], "2026-04-20");
    assert_eq!(body["window"]["to"], "2026-05-12");
    assert_eq!(body["window"]["weeks"], 4);

    let bands = &body["bands"];
    assert!(bands.is_object(), "the window holds moving samples");
    let rows = band_rows(bands);
    assert!(!rows.is_empty(), "the bands hold the window's time");

    // The times in the bands add up to the window's time: the bands are a
    // distribution of it, not a part of it.
    let total: f64 = rows.iter().map(|(_, time)| time).sum();
    let stated = bands["total"]["raw"].as_f64().unwrap_or_default();
    assert!(
        (total - stated).abs() < 1.0,
        "the bands add up to {total} and the answer says {stated}"
    );
}

#[test]
fn a_narrower_window_does_not_count_the_runs_outside_it() {
    let harness = Harness::seeded("pace-window");
    harness.write_run(common::run_with_fit(
        "a-run-in-the-window",
        "2026-05-04",
        "hr_gps_run.fit",
    ));
    harness.write_run(common::run_with_fit(
        "a-run-before-the-window",
        "2026-04-13",
        "hr_gps_run.fit",
    ));

    let wide = harness.ok_get("/insights/pace?weeks=4");
    let narrow = harness.ok_get("/insights/pace?weeks=1");

    // The narrow window is the last week: the run of April is outside it.
    assert_eq!(narrow["window"]["from"], "2026-05-11");
    assert!(
        narrow["bands"].is_null(),
        "the narrow window holds no moving samples"
    );
    assert!(
        narrow["unavailable"].is_string(),
        "the answer says why there is no value"
    );
    assert!(
        wide["bands"].is_object(),
        "the wider window holds the run's samples"
    );
}

#[test]
fn a_window_with_no_samples_has_no_bands() {
    let harness = Harness::seeded("pace-no-samples");
    // A run with no file: there are no samples, so there is no pace.
    harness.write_run(common::run(
        "a-run-without-a-file",
        "2026-05-04",
        10_000.0,
        3_000.0,
    ));

    let body = harness.ok_get("/insights/pace?weeks=4");

    assert!(body["bands"].is_null(), "no value is not a band of zero");
    assert_eq!(
        body["unavailable"],
        "pace bands unavailable — the window holds no moving samples"
    );
}

#[test]
fn the_training_series_carries_the_week_and_the_long_run_inside_it() {
    let harness = Harness::empty("pace-weeks");
    harness.write_run(common::run("a-short-run", "2026-05-04", 5_000.0, 1_500.0));
    harness.write_run(common::run("a-long-run", "2026-05-06", 16_000.0, 5_000.0));

    let body = harness.ok_get("/insights/pace?weeks=4");

    let weeks = body["weeks"].as_array().expect("the weeks are there");
    let week = weeks
        .iter()
        .find(|week| week["week_start"] == "2026-05-04")
        .expect("the week of the runs is there");

    // The week's distance and its sessions.
    assert_eq!(week["distance"]["raw"], 21_000.0);
    assert_eq!(week["sessions"], 2);
    // And the long run inside it.
    assert_eq!(week["longest"]["distance"]["raw"], 16_000.0);
    assert_eq!(week["longest"]["pace"]["raw"], 312.5);
    // The week's own medians: the pace and the heart rate are separate.
    assert_eq!(week["pace"]["raw"], 306.25);
    assert_eq!(week["heart_rate"]["raw"], 148.0);
}

#[test]
fn the_window_outside_the_bound_is_refused() {
    let harness = Harness::seeded("pace-bound");

    let answer = harness.get("/insights/pace?weeks=105");
    assert_eq!(answer["code"], "bad_request", "the window is refused");
}
