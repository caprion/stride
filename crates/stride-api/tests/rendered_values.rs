//! The values are the core's rendering of them.
//!
//! The contract's §8: *the DTOs carry the rendered value*. A domain value
//! crosses the wire as the text the core rendered **and** the raw number a
//! chart draws — and never as a `format:` hint, because a hint tells the
//! app how to render, which is the hole the section closes.
//!
//! This test holds the answers to that: the text in an answer is
//! byte-identical to what `stride-core` writes for the same number, and
//! the number beside it is the number the chart draws.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_core::types::{Distance, Duration};
use stride_core::{strings, units};

#[test]
fn a_distance_is_the_core_s_rendering_and_the_core_s_number() {
    let harness = common::Harness::empty("rendered-distance");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 8_400.0, 2_520.0));
    let body = harness.ok_get(&format!("/activities/{id}"));

    let distance = match Distance::from_metres(8_400.0) {
        Some(distance) => distance,
        None => panic!("the distance could not be made"),
    };
    assert_eq!(
        body["distance"]["text"].as_str(),
        Some(distance.to_human().as_str()),
        "the distance is not the core's rendering"
    );
    assert_eq!(body["distance"]["raw"].as_f64(), Some(distance.metres()));
}

#[test]
fn a_duration_is_the_core_s_rendering_and_the_core_s_number() {
    let harness = common::Harness::empty("rendered-duration");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 8_400.0, 2_520.0));
    let body = harness.ok_get(&format!("/activities/{id}"));

    let duration = match Duration::from_seconds(2_520.0) {
        Some(duration) => duration,
        None => panic!("the duration could not be made"),
    };
    assert_eq!(
        body["moving_time"]["text"].as_str(),
        Some(duration.to_human().as_str()),
        "the moving time is not the core's rendering"
    );
    assert_eq!(
        body["moving_time"]["raw"].as_f64(),
        Some(duration.seconds())
    );
}

#[test]
fn a_heart_rate_is_the_core_s_catalogue_and_the_core_s_units() {
    let harness = common::Harness::empty("rendered-heart-rate");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 8_400.0, 2_520.0));
    let body = harness.ok_get(&format!("/activities/{id}"));

    let text = match strings::render("vital.resting_hr.value", &[("value", "148")]) {
        Some(text) => text,
        None => panic!("the catalogue has no heart-rate template"),
    };
    assert_eq!(
        body["avg_hr"]["text"].as_str(),
        Some(text.as_str()),
        "the heart rate is not the core's rendering"
    );
    assert_eq!(body["avg_hr"]["raw"].as_f64(), Some(148.0));
}

#[test]
fn a_whole_number_is_the_core_s_units() {
    // The stress average is a whole number the core reads as one.
    let text = units::format_whole(28.4);
    assert_eq!(text, "28");
}

#[test]
fn a_value_the_core_does_not_render_is_absent() {
    // The core does not render a pace, an altitude, a cadence or a
    // difference yet. Those values are absent rather than formatted here:
    // a second implementation of the core's rounding is the disease the
    // contract is about.
    let harness = common::Harness::empty("rendered-absent");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 8_400.0, 2_520.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert!(
        body["pace"].is_null(),
        "the pace was formatted here: {}",
        body["pace"]
    );
    assert!(
        body["elevation_gain"].is_null(),
        "the climb was formatted here: {}",
        body["elevation_gain"]
    );
    assert!(
        body["avg_cadence"].is_null(),
        "the cadence was formatted here: {}",
        body["avg_cadence"]
    );
}

#[test]
fn no_answer_carries_a_format_hint() {
    // A hint that tells the app *how* to render is the hole the contract
    // closes: there is no `format:` field anywhere in an answer.
    let harness = common::Harness::seeded("rendered-no-hint");
    for route in stride_api::ROUTES.iter() {
        let path = route.pattern.replace("{id}", "1");
        let body = harness
            .router
            .call(route.method.as_str(), &path, Some("{}"))
            .body
            .to_string();
        assert!(
            !body.contains("\"format\""),
            "the answer of {path} carries a format hint"
        );
    }
}
