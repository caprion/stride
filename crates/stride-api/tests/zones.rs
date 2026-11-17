//! The heart-rate zones of a run, from the athlete's **stored zone model**.
//!
//! The model is the athlete's own configuration, read from Garmin and
//! stored with the day it was read. The route answers the per-run question
//! — how long was the athlete in each of their zones — from that model, and
//! names it, so a number can say where its input came from and when.
//!
//! A run with no stored model has **no value**: never band 1, never a zero.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{DateTime, Duration, Utc};
use stride_core::{time_in_zone, TimeInZone, ZoneFloors, ZoneSample};
use stride_store::{NewActivity, NewHeartRateZoneModel};

/// The floors of the synthetic model below, in beats per minute.
///
/// The repository is public: no athlete's real floors belong in a test.
const FLOORS: [f64; 5] = [106.0, 121.0, 135.0, 149.0, 163.0];

/// A synthetic zone model, with invented numbers.
fn model(read_at: DateTime<Utc>) -> NewHeartRateZoneModel {
    NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(178.0),
        zone1_floor: FLOORS[0],
        zone2_floor: FLOORS[1],
        zone3_floor: FLOORS[2],
        zone4_floor: FLOORS[3],
        zone5_floor: FLOORS[4],
        read_at,
    }
}

/// The model's floors, as the core takes them.
fn floors() -> ZoneFloors {
    ZoneFloors {
        zone1: FLOORS[0],
        zone2: FLOORS[1],
        zone3: FLOORS[2],
        zone4: FLOORS[3],
        zone5: FLOORS[4],
    }
}

/// Write a run whose FIT file is the corpus file named.
fn write_fit_run(harness: &common::Harness, name: &str) -> i64 {
    let fit = common::corpus_fit(name);
    harness.write_run(NewActivity {
        source: "garmin_personal".to_string(),
        external_id: format!("fit-{name}"),
        name: "a run with heart rate".to_string(),
        sport_type: "running".to_string(),
        start_time: common::instant(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 0.0,
        avg_hr: Some(150.0),
        max_hr: Some(180.0),
        fit_blob: fit,
        created_at: common::instant(),
        updated_at: common::instant(),
    })
}

/// The core's own answer for the run, from the same moving samples.
///
/// The route is held to the core: the numbers it publishes are the core's
/// numbers, not a second computation of them.
fn core_time(harness: &common::Harness, id: i64) -> TimeInZone {
    let blob = harness
        .store
        .fit_blob(id)
        .unwrap()
        .unwrap_or_else(|| panic!("the run has no FIT file"));
    let decoded = stride_ingest::decode_fit(&blob, "activity.fit")
        .unwrap_or_else(|error| panic!("the FIT file could not be decoded: {error}"));
    let samples: Vec<ZoneSample> = decoded
        .samples
        .iter()
        .filter(|sample| sample.kind != stride_ingest::SegmentKind::Stand)
        .map(|sample| ZoneSample {
            elapsed_s: sample.elapsed_s,
            heart_rate_bpm: sample.heart_rate_bpm,
        })
        .collect();
    time_in_zone(floors(), &samples).unwrap()
}

#[test]
fn a_run_with_a_stored_model_answers_with_the_five_times_and_the_models_date() {
    let harness = common::Harness::empty("zones-model");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let read_at = common::instant() - Duration::days(2);
    harness
        .store
        .write_hr_zone_model_if_new(&model(read_at))
        .unwrap();

    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    // The model is named: its sport, its method, its maximum heart rate,
    // its floors and the day it was read.
    let named = &body["model"];
    assert_eq!(named["sport"].as_str(), Some("DEFAULT"));
    assert_eq!(named["training_method"].as_str(), Some("HR_MAX"));
    assert_eq!(named["max_heart_rate"]["text"].as_str(), Some("178 bpm"));
    assert_eq!(named["floors"]["zone1"]["text"].as_str(), Some("106 bpm"));
    assert_eq!(named["floors"]["zone2"]["text"].as_str(), Some("121 bpm"));
    assert_eq!(named["floors"]["zone3"]["text"].as_str(), Some("135 bpm"));
    assert_eq!(named["floors"]["zone4"]["text"].as_str(), Some("149 bpm"));
    assert_eq!(named["floors"]["zone5"]["text"].as_str(), Some("163 bpm"));
    assert_eq!(named["read_at"].as_str(), Some("2026-05-10T06:30:00Z"));
    // The day the model was read, in the athlete's own calendar and in the
    // api's own date rendering — not the core's canonical form.
    assert_eq!(named["read_on"].as_str(), Some("10 May 2026"));
    // And the model's own source line, in the core's words.
    assert_eq!(
        named["source"].as_str(),
        Some("your zone model · HR_MAX · max heart rate 178 bpm · read on 10 May 2026")
    );
    assert!(body["unavailable"].is_null());

    // The five times are the core's own, in seconds.
    let expected = core_time(&harness, id);
    let zones = body["zones"].as_array().unwrap();
    assert_eq!(zones.len(), 5);
    for (index, zone) in zones.iter().enumerate() {
        assert_eq!(
            zone["time"]["raw"].as_f64(),
            Some(expected.per_zone()[index]),
            "zone {} does not carry the core's time",
            index + 1
        );
    }
    assert_eq!(body["outside"]["raw"].as_f64(), Some(expected.outside_s));

    // The bands are the model's own floors — not tenths of this run's
    // maximum heart rate.
    assert_eq!(zones[0]["from_bpm"].as_i64(), Some(106));
    assert_eq!(zones[0]["to_bpm"].as_i64(), Some(121));
    assert_eq!(zones[4]["from_bpm"].as_i64(), Some(163));
    assert_eq!(zones[4]["to_bpm"].as_i64(), Some(178));
    assert_eq!(zones[0]["range"].as_str(), Some("106-121 bpm"));
}

#[test]
fn two_runs_are_measured_on_the_same_scale() {
    let harness = common::Harness::empty("zones-comparable");
    let easy = write_fit_run(&harness, "hr_gps_run.fit");
    let hard = write_fit_run(&harness, "mixed_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();

    let one = harness.ok_get(&format!("/fitness/zones?id={easy}"));
    let two = harness.ok_get(&format!("/fitness/zones?id={hard}"));

    // The same model, the same bands: the two runs are comparable, which is
    // what a per-run scale could never be.
    let edges = |body: &serde_json::Value| -> Vec<(Option<i64>, Option<i64>)> {
        body["zones"]
            .as_array()
            .unwrap()
            .iter()
            .map(|zone| (zone["from_bpm"].as_i64(), zone["to_bpm"].as_i64()))
            .collect()
    };
    assert_eq!(edges(&one), edges(&two));
}

#[test]
fn a_run_with_no_stored_model_has_no_value() {
    let harness = common::Harness::empty("zones-no-model");
    let id = write_fit_run(&harness, "hr_gps_run.fit");

    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    // No value is the absence of the whole result: no bands, no time
    // outside them, and the reason in the core's words.
    assert!(body["zones"].as_array().unwrap().is_empty());
    assert!(body["outside"].is_null());
    assert!(body["model"].is_null());
    assert_eq!(
        body["unavailable"].as_str(),
        Some("zones unavailable — no zone model has been read")
    );
    // Never band 1, and never a zero.
    assert!(body["zones"].as_array().unwrap().is_empty());
    assert!(body["outside"].is_null());
}

#[test]
fn a_model_with_no_maximum_heart_rate_is_named_without_one() {
    let harness = common::Harness::empty("zones-no-max");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let mut stored = model(common::instant());
    // Garmin reported no maximum heart rate for this model.
    stored.max_heart_rate = None;
    harness.store.write_hr_zone_model_if_new(&stored).unwrap();

    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    // The model is still named, and the line names no maximum heart rate
    // that was not reported.
    let named = &body["model"];
    assert!(named["max_heart_rate"].is_null());
    assert_eq!(
        named["source"].as_str(),
        Some("your zone model · HR_MAX · read on 12 May 2026")
    );
    // And the top band has no upper edge to draw.
    let zones = body["zones"].as_array().unwrap();
    assert_eq!(zones[4]["to_bpm"].as_i64(), None);
}

#[test]
fn a_model_whose_floors_are_not_in_order_is_reported_as_unusable() {
    let harness = common::Harness::empty("zones-unusable-model");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let mut stored = model(common::instant());
    // The floors no longer increase: the model cannot be used.
    stored.zone2_floor = 90.0;
    harness.store.write_hr_zone_model_if_new(&stored).unwrap();

    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    // The same way: no value, and the reason says which one it is.
    assert!(body["zones"].as_array().unwrap().is_empty());
    assert!(body["outside"].is_null());
    assert_eq!(
        body["unavailable"].as_str(),
        Some("zones unavailable — the stored zone model's floors are not in order")
    );
}

#[test]
fn a_run_with_no_samples_says_so_in_the_cores_words() {
    let harness = common::Harness::empty("zones-no-samples");
    // A model, and a run with an empty FIT file: there is nothing to read.
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    let body = harness.ok_get(&format!("/fitness/zones?id={id}"));

    assert!(body["zones"].as_array().unwrap().is_empty());
    assert!(body["outside"].is_null());
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
