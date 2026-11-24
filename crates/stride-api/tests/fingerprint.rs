//! The fingerprint of one run: the observations, the zone shares, and the
//! "no value" answers.
//!
//! Every fixture is synthetic: the corpus files are the repository's own
//! sanitised ones, and the zone model carries invented floors.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{DateTime, Duration, Utc};
use stride_store::{NewActivity, NewHeartRateZoneModel};

/// The floors of the synthetic model below, in beats per minute.
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

/// The observation with this key.
fn observation<'a>(body: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    body["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|observation| observation["key"] == key)
        .unwrap_or_else(|| panic!("the answer carries no {key}"))
}

#[test]
fn a_run_answers_with_its_observations() {
    let harness = common::Harness::empty("fingerprint-observations");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();

    let body = harness.ok_get(&format!("/fitness/fingerprint?id={id}"));

    // The run is named, and the basis is named too.
    assert_eq!(body["activity_id"].as_i64(), Some(id));
    assert_eq!(body["local_date"].as_str(), Some("2026-05-10"));
    assert_eq!(body["basis"].as_str(), Some("laps and moving samples"));
    assert!(body["unavailable"].is_null());

    // Every observation is a value or a reason, and the answer carries no
    // score: a weighted mean of a run's numbers would be a grade.
    let observations = body["observations"].as_array().unwrap();
    assert!(!observations.is_empty());
    for observation in observations {
        assert!(
            observation["value"].is_object() || observation["reason"].is_string(),
            "the observation carries neither a value nor a reason: {observation}"
        );
        assert!(!observation["label"].as_str().unwrap_or_default().is_empty());
    }
    let keys: Vec<&str> = observations
        .iter()
        .map(|observation| observation["key"].as_str().unwrap_or_default())
        .collect();
    assert!(keys.contains(&"cadence"));
    assert!(keys.contains(&"decoupling"));
    assert!(!keys.contains(&"form_score"));
    assert!(body.get("score").is_none());
    assert!(body.get("form_score").is_none());

    // The shape of an observation: a key, a label, a value and a reason.
    let mut shape: Vec<&str> = observations[0]
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str())
        .collect();
    shape.sort();
    assert_eq!(shape, vec!["key", "label", "reason", "value"]);
}

#[test]
fn the_zone_shares_are_the_stored_models_own() {
    let harness = common::Harness::empty("fingerprint-zones");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant() - Duration::days(2)))
        .unwrap();

    let body = harness.ok_get(&format!("/fitness/fingerprint?id={id}"));

    // The model is named and dated: the shares say which model they were
    // measured against.
    assert_eq!(body["zones"]["model"]["sport"].as_str(), Some("DEFAULT"));
    assert_eq!(
        body["zones"]["model"]["max_heart_rate"]["text"].as_str(),
        Some("178 bpm")
    );
    assert_eq!(
        body["zones"]["model"]["read_at"].as_str(),
        Some("2026-05-10T06:30:00Z")
    );
    assert_eq!(
        body["zones"]["source"].as_str(),
        Some("your zones, taken from your watch on 2026-05-10")
    );

    // The five shares are there, and they add up to a hundred per cent of
    // the time in the zones.
    let shares = body["zones"]["shares"].as_array().unwrap();
    assert_eq!(shares.len(), 5);
    assert_eq!(shares[0]["label"].as_str(), Some("Z1"));
    assert_eq!(shares[4]["label"].as_str(), Some("Z5"));
    let total: f64 = shares
        .iter()
        .map(|share| share["share"]["raw"].as_f64().unwrap_or(0.0))
        .sum();
    assert!((total - 100.0).abs() < 1e-6, "the shares add up to {total}");
    assert!(body["zones"]["unavailable"].is_null());
}

#[test]
fn a_run_with_no_stored_model_has_no_shares() {
    let harness = common::Harness::empty("fingerprint-no-model");
    let id = write_fit_run(&harness, "hr_gps_run.fit");

    let body = harness.ok_get(&format!("/fitness/fingerprint?id={id}"));

    // No value is the absence of the whole result: no shares, and the
    // reason in the core's words. Never a zero.
    assert!(body["zones"]["shares"].as_array().unwrap().is_empty());
    assert!(body["zones"]["model"].is_null());
    assert_eq!(
        body["zones"]["unavailable"].as_str(),
        Some("No zones yet — your watch's heart-rate zones have not come through")
    );
    // The observations are still there: the model is only the zones' input.
    assert!(!body["observations"].as_array().unwrap().is_empty());
}

#[test]
fn a_run_with_no_file_has_no_value() {
    let harness = common::Harness::empty("fingerprint-no-file");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    let body = harness.ok_get(&format!("/fitness/fingerprint?id={id}"));

    assert!(body["observations"].as_array().unwrap().is_empty());
    assert!(body["basis"].is_null());
    assert_eq!(
        body["unavailable"].as_str(),
        Some("the run has no file to read")
    );
}

#[test]
fn a_run_with_no_cadence_says_so() {
    let harness = common::Harness::empty("fingerprint-no-cadence");
    let id = write_fit_run(&harness, "no_cadence.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();

    let body = harness.ok_get(&format!("/fitness/fingerprint?id={id}"));

    let cadence = observation(&body, "cadence");
    assert!(cadence["value"].is_null());
    assert_eq!(
        cadence["reason"].as_str(),
        Some("the run has no cadence readings")
    );
}

#[test]
fn the_fingerprint_needs_a_run() {
    let harness = common::Harness::empty("fingerprint-needs-run");
    let answer = harness.router.call("GET", "/fitness/fingerprint", None);
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));

    // And a run that is not there is a not-found.
    let answer = harness
        .router
        .call("GET", "/fitness/fingerprint?id=9999", None);
    assert_eq!(answer.status, 404);
}
