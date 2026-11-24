//! The run film.
//!
//! The film is one run, playable: the run's own series, the zone each
//! sample sits in, the ground, and the splits — and the zone model the
//! zones were measured against, named and dated.
//!
//! The tests hold three things:
//!
//! * the series is the **run's own** — decoded from its FIT file on read —
//!   and each sample carries the zone the **core's** classifier gives it;
//! * the model is **named and dated**, so a number can say where its input
//!   came from;
//! * **no value is the absence of the whole result** — a run with no file
//!   has no series, and says why in the core's words, never a zero.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::Duration;
use serde_json::Value;

/// The zone model the tests store: the watch's own configuration, with the
/// day it was read.
fn model() -> stride_store::NewHeartRateZoneModel {
    stride_store::NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(190.0),
        zone1_floor: 100.0,
        zone2_floor: 120.0,
        zone3_floor: 140.0,
        zone4_floor: 160.0,
        zone5_floor: 175.0,
        // The model is read **before** the runs: the model in force on the
        // day of a run is the one that was read on or before it.
        read_at: common::instant() - Duration::days(3),
    }
}

/// The film of a run, as the router answers it.
fn film(harness: &common::Harness, id: i64) -> Value {
    harness.ok_get(&format!("/activities/{id}/film"))
}

#[test]
fn the_film_carries_the_runs_own_series() {
    let harness = common::Harness::seeded("film-series");
    let id = harness.write_run(common::run_with_fit(
        "a-run-with-a-file",
        "2026-05-10",
        "hr_gps_run.fit",
    ));
    harness
        .store
        .write_hr_zone_model_if_new(&model())
        .expect("the zone model could be written");

    let body = film(&harness, id);

    assert_eq!(body["activity_id"], id);
    assert_eq!(body["local_date"], "2026-05-10");
    assert_eq!(body["basis"], "samples");
    assert_eq!(body["unavailable"], Value::Null);
    assert_eq!(body["zones_unavailable"], Value::Null);

    let samples = body["samples"].as_array().expect("the film has samples");
    assert!(!samples.is_empty(), "the film has no samples");

    // Every sample carries the core's words for the time, and the number a
    // chart draws.
    for sample in samples {
        let elapsed = &sample["elapsed"];
        assert!(elapsed["text"].is_string(), "a sample has no elapsed time");
        assert!(elapsed["raw"].is_number(), "a sample has no elapsed number");
        assert!(
            sample["pace"].is_null() || sample["pace"]["text"].is_string(),
            "a sample's pace is not a rendered value"
        );
        assert!(
            sample["heart_rate"].is_null() || sample["heart_rate"]["text"].is_string(),
            "a sample's heart rate is not a rendered value"
        );
    }

    // The ground is on the run's own time axis, and the splits are places
    // on it: each one carries the time it began.
    let ground = body["ground"].as_array().expect("the film has ground");
    assert!(!ground.is_empty(), "the film has no ground");
    for point in ground {
        assert!(point["offset_s"].is_number());
        assert!(point["elevation"]["text"].is_string());
    }
    let splits = body["splits"].as_array().expect("the film has splits");
    assert!(!splits.is_empty(), "the film has no splits");
    let mut previous = -1.0;
    for split in splits {
        let offset = split["offset_s"].as_f64().expect("a split has an offset");
        assert!(offset > previous, "the splits do not move along the run");
        previous = offset;
    }
}

#[test]
fn every_sample_carries_the_zone_the_core_gives_it() {
    let harness = common::Harness::seeded("film-zones");
    let id = harness.write_run(common::run_with_fit(
        "a-run-with-a-file",
        "2026-05-10",
        "hr_gps_run.fit",
    ));
    harness
        .store
        .write_hr_zone_model_if_new(&model())
        .expect("the zone model could be written");

    let body = film(&harness, id);
    let samples = body["samples"].as_array().expect("the film has samples");
    let zones = body["zones"].as_array().expect("the film has zones");
    assert_eq!(zones.len(), 5, "the athlete has five zones");

    // The zone of a sample is the zone its own heart rate falls into,
    // measured against the stored model's floors — the core's rule, the
    // same one the time-in-zone computation uses.
    let floors = [100.0, 120.0, 140.0, 160.0, 175.0];
    let mut checked = 0;
    for sample in samples {
        let rate = match &sample["heart_rate"]["raw"] {
            Value::Null => continue,
            value => value.as_f64().expect("a heart rate is a number"),
        };
        let zone = sample["zone"].as_i64();
        let expected = if rate <= floors[0] {
            None
        } else if rate <= floors[1] {
            Some(0)
        } else if rate <= floors[2] {
            Some(1)
        } else if rate <= floors[3] {
            Some(2)
        } else if rate <= floors[4] {
            Some(3)
        } else {
            Some(4)
        };
        assert_eq!(
            zone, expected,
            "the zone of a reading of {rate} bpm is wrong"
        );
        checked += 1;
    }
    assert!(checked > 0, "the run carried no heart rate to check");
}

#[test]
fn the_model_is_named_and_dated() {
    let harness = common::Harness::seeded("film-model");
    let id = harness.write_run(common::run_with_fit(
        "a-run-with-a-file",
        "2026-05-10",
        "hr_gps_run.fit",
    ));
    harness
        .store
        .write_hr_zone_model_if_new(&model())
        .expect("the zone model could be written");

    let body = film(&harness, id);
    let model = &body["model"];
    assert_eq!(model["sport"], "DEFAULT");
    assert_eq!(model["training_method"], "HR_MAX");
    assert_eq!(model["max_heart_rate"]["text"], "190 bpm");
    // The day the model was read, in the api's own rendering, and the
    // model's own source line.
    assert_eq!(model["read_on"], "9 May 2026");
    let source = model["source"]
        .as_str()
        .expect("the model has a source line");
    assert!(
        source.contains("HR_MAX"),
        "the source line names the method"
    );
    assert!(
        source.contains("9 May 2026"),
        "the source line names the day"
    );
}

#[test]
fn a_run_with_no_file_has_no_film() {
    let harness = common::Harness::seeded("film-no-file");
    let id = harness.write_run(common::run(
        "a-run-without-a-file",
        "2026-05-10",
        10_000.0,
        3_000.0,
    ));
    harness
        .store
        .write_hr_zone_model_if_new(&model())
        .expect("the zone model could be written");

    let body = film(&harness, id);

    // The honest answer: no series, and the reason in the core's words.
    assert_eq!(
        body["unavailable"], "the run has no file to read",
        "the reason is not the core's own"
    );
    assert_eq!(body["samples"].as_array().map(|s| s.len()), Some(0));
    assert_eq!(body["ground"].as_array().map(|s| s.len()), Some(0));
    // And the model is still named: the athlete's configuration is a fact
    // whatever the run has or has not.
    assert_eq!(body["model"]["sport"], "DEFAULT");
}

#[test]
fn a_run_with_no_zone_model_has_no_zones_but_still_has_its_series() {
    let harness = common::Harness::seeded("film-no-model");
    let id = harness.write_run(common::run_with_fit(
        "a-run-with-a-file",
        "2026-05-10",
        "hr_gps_run.fit",
    ));

    let body = film(&harness, id);

    assert_eq!(body["unavailable"], Value::Null, "the film itself is there");
    assert!(!body["samples"].as_array().expect("samples").is_empty());
    // The zones have no value, and the core says why.
    assert_eq!(body["model"], Value::Null);
    assert_eq!(body["zones"].as_array().map(|z| z.len()), Some(0));
    assert_eq!(
        body["zones_unavailable"],
        "zones unavailable — no zone model has been read"
    );
    // And no sample claims to be in a zone.
    for sample in body["samples"].as_array().expect("samples") {
        assert_eq!(sample["zone"], Value::Null);
    }
}

#[test]
fn a_run_that_is_not_there_is_not_found() {
    let harness = common::Harness::empty("film-missing");
    let answer = harness.router.call("GET", "/activities/9999/film", None);
    assert_eq!(answer.status, 404);
    assert_eq!(answer.body["code"], "not_found");
}
