//! The heart-rate zones of a run.
//!
//! The zones are computed from the samples that carry a heart rate, and from
//! nothing else. The time with no heart rate is counted separately, the basis
//! is named, the maximum heart rate the bands came from is shown, and a run
//! with nothing to compute from says so in the core's words.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::NewActivity;

fn instant() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(common::NOW)
        .map(|value| value.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

/// Write a run whose FIT file is the corpus file named.
fn write_fit_run(harness: &common::Harness, name: &str) -> i64 {
    let fit = common::corpus_fit(name);
    harness.write_run(NewActivity {
        source: "garmin_personal".to_string(),
        external_id: format!("fit-{name}"),
        name: "a run with heart rate".to_string(),
        sport_type: "running".to_string(),
        start_time: instant(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 0.0,
        avg_hr: Some(150.0),
        max_hr: Some(180.0),
        fit_blob: fit,
        created_at: instant(),
        updated_at: instant(),
    })
}

#[test]
fn the_zones_are_computed_from_the_samples_with_heart_rate() {
    let harness = common::Harness::empty("zones-computed");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    // The basis is named, and the maximum heart rate the bands came from is
    // shown, rendered by the core.
    assert_eq!(body["basis"].as_str(), Some("moving"));
    assert!(body["max_heart_rate"].is_object());
    assert!(body["max_heart_rate"]["text"].as_str().is_some());
    assert!(body["unavailable"].is_null());

    // Five bands, each with a name, a range and a time.
    let zones = body["zones"].as_array().unwrap();
    assert_eq!(zones.len(), 5);
    for (index, zone) in zones.iter().enumerate() {
        assert_eq!(
            zone["label"].as_str(),
            Some(format!("Z{}", index + 1).as_str())
        );
        assert!(zone["range"].as_str().is_some());
        assert!(zone["time"].is_object());
    }

    // The time with no heart rate is counted separately.
    assert!(body["without_heart_rate"].is_object());
}

#[test]
fn the_bands_are_tenths_of_the_maximum_heart_rate() {
    let harness = common::Harness::empty("zones-bands");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    let max = body["max_heart_rate"]["raw"].as_f64().unwrap();
    let zones = body["zones"].as_array().unwrap();
    // The first band runs from half of the maximum to six tenths of it.
    let first = &zones[0];
    let expected = stride_core::units::render_heart_rate_range(Some(0.5 * max), Some(0.6 * max));
    assert_eq!(first["range"].as_str(), expected.as_deref());
    // The edges the words were rendered from, for the chart to draw.
    let (lower, upper) =
        stride_core::units::render_heart_rate_range_edges(Some(0.5 * max), Some(0.6 * max))
            .unwrap_or((0, 0));
    assert_eq!(first["from_bpm"].as_i64(), Some(lower));
    assert_eq!(first["to_bpm"].as_i64(), Some(upper));
    // The last band ends at the maximum itself.
    let last = &zones[4];
    let expected = stride_core::units::render_heart_rate_range(Some(0.9 * max), Some(max));
    assert_eq!(last["range"].as_str(), expected.as_deref());
    let (lower, upper) =
        stride_core::units::render_heart_rate_range_edges(Some(0.9 * max), Some(max))
            .unwrap_or((0, 0));
    assert_eq!(last["from_bpm"].as_i64(), Some(lower));
    assert_eq!(last["to_bpm"].as_i64(), Some(upper));
}

#[test]
fn a_run_with_no_heart_rate_says_so_in_the_cores_words() {
    let harness = common::Harness::empty("zones-unavailable");
    // A run with an empty FIT file has nothing to compute from.
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    assert!(body["zones"].as_array().unwrap().is_empty());
    assert!(body["max_heart_rate"].is_null());
    assert_eq!(body["basis"].as_str(), None);
    // The reason is the core's sentence, not this crate's.
    assert_eq!(
        body["unavailable"].as_str(),
        Some("zones unavailable — the run has no heart-rate samples")
    );
}

#[test]
fn zones_need_a_run() {
    let harness = common::Harness::empty("zones-need-run");
    let answer = harness.router.call("GET", "/fitness/zones", None);
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));

    // And a run that is not there is a not-found.
    let answer = harness.router.call("GET", "/fitness/zones?id=9999", None);
    assert_eq!(answer.status, 404);
}
