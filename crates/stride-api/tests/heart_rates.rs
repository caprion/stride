// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The athlete's own declared heart rates.
//!
//! **One small fact, entered once, and the app takes care of it.** What is
//! proved here:
//!
//! * the declaration is written with its **source** and its **dates**, and
//!   the pair is **one declaration**;
//! * the bands are **Garmin's published ladder** on the range the
//!   declaration supports — the reserve with both numbers, the share of the
//!   maximum with a maximum alone, and **no bands at all** with a resting
//!   rate alone, where the reason is the core's own;
//! * the **watch's own floors** are stated beside them, and the difference
//!   between the two is stated as a fact;
//! * the **zones route** measures the run against the declared bands and
//!   **names the basis** it used.

mod common;

use common::Harness;
use serde_json::Value;

/// The pair, as the athlete enters it.
const PAIR: &str = r#"{
    "max_heart_rate_bpm": 186,
    "max_source": "test",
    "max_measured_on": "2026-11-08",
    "resting_heart_rate_bpm": 52,
    "resting_source": "doctor_lab",
    "resting_measured_on": "2026-11-12"
}"#;

/// The answer of a write, with its status.
fn post(harness: &Harness, path: &str, body: &str) -> (u16, Value) {
    let answer = harness.router.call("POST", path, Some(body));
    (answer.status, answer.body)
}

#[test]
fn the_pair_is_written_as_one_declaration_with_its_sources_and_its_dates() {
    let harness = Harness::empty("heart-rates-write");
    let (status, body) = post(&harness, "/athlete/heart-rates", PAIR);
    assert_eq!(status, 200, "the declaration was refused: {body}");

    let declaration = &body["declaration"];
    // The two numbers, each with its own source and its own measurement
    // day, and the day the declaration was made.
    assert_eq!(declaration["max"]["value_bpm"], 186.0);
    assert_eq!(declaration["max"]["source"], "test");
    assert_eq!(declaration["max"]["source_label"], "from a test");
    assert_eq!(declaration["max"]["measured_on"], "2026-11-08");
    assert_eq!(declaration["resting"]["value_bpm"], 52.0);
    assert_eq!(declaration["resting"]["source"], "doctor_lab");
    assert_eq!(
        declaration["resting"]["source_label"],
        "from a doctor or a lab"
    );
    assert_eq!(declaration["resting"]["measured_on"], "2026-11-12");
    assert_eq!(declaration["entered_on"], common::TODAY);
    assert_eq!(declaration["updated_on"], common::TODAY);
    assert_eq!(declaration["in_force"], true);

    // And the bands it yields: Garmin's ladder on the reserve.
    let answer = harness.ok_get("/athlete/heart-rates");
    let bands = &answer["bands"];
    assert_eq!(bands["range"], "reserve");
    assert_eq!(bands["floors"]["zone1"]["raw"], 119.0);
    assert_eq!(bands["floors"]["zone2"]["raw"], 132.0);
    assert_eq!(bands["floors"]["zone3"]["raw"], 146.0);
    assert_eq!(bands["floors"]["zone4"]["raw"], 159.0);
    assert_eq!(bands["floors"]["zone5"]["raw"], 173.0);
    assert_eq!(bands["max_heart_rate"]["raw"], 186.0);
    // And the source line names the basis and the day.
    let source = bands["source"].as_str().unwrap_or_default();
    assert!(source.contains("the bands from the heart rates you entered"));
    assert!(source.contains("entered on"));
}

#[test]
fn a_maximum_alone_gives_the_ladder_on_the_share_of_the_maximum() {
    let harness = Harness::empty("heart-rates-maximum");
    let (status, body) = post(
        &harness,
        "/athlete/heart-rates",
        r#"{"max_heart_rate_bpm": 186, "max_source": "watch"}"#,
    );
    assert_eq!(status, 200, "the declaration was refused: {body}");

    let answer = harness.ok_get("/athlete/heart-rates");
    let bands = &answer["bands"];
    assert_eq!(bands["range"], "max_share");
    assert_eq!(bands["floors"]["zone1"]["raw"], 93.0);
    assert_eq!(bands["floors"]["zone2"]["raw"], 112.0);
    assert_eq!(bands["floors"]["zone3"]["raw"], 130.0);
    assert_eq!(bands["floors"]["zone4"]["raw"], 149.0);
    assert_eq!(bands["floors"]["zone5"]["raw"], 167.0);
    // And the reason the other range has no value is the core's own.
    assert!(answer["unavailable"].is_null());
}

#[test]
fn a_resting_rate_alone_gives_no_bands_and_the_reason_why() {
    let harness = Harness::empty("heart-rates-resting");
    let (status, body) = post(
        &harness,
        "/athlete/heart-rates",
        r#"{"resting_heart_rate_bpm": 52, "resting_source": "doctor_lab"}"#,
    );
    assert_eq!(status, 200, "the declaration was refused: {body}");

    let answer = harness.ok_get("/athlete/heart-rates");
    // **Absence is not zero**: no bands at all, and the reason.
    assert!(answer["bands"].is_null());
    let reason = answer["unavailable"].as_str().unwrap_or_default();
    assert!(reason.contains("no maximum heart rate"), "{reason}");
}

#[test]
fn a_declaration_with_no_value_is_refused() {
    let harness = Harness::empty("heart-rates-empty");
    let (status, body) = post(&harness, "/athlete/heart-rates", "{}");
    assert_eq!(status, 400, "the declaration was written: {body}");
    assert_eq!(body["code"], "bad_request");
}

#[test]
fn a_source_the_app_does_not_know_is_refused() {
    let harness = Harness::empty("heart-rates-source");
    let (status, body) = post(
        &harness,
        "/athlete/heart-rates",
        r#"{"max_heart_rate_bpm": 186, "max_source": "a rumour"}"#,
    );
    assert_eq!(status, 400, "the declaration was written: {body}");
    assert_eq!(body["code"], "bad_request");
}

#[test]
fn a_heart_rate_needs_the_source_it_came_from() {
    let harness = Harness::empty("heart-rates-no-source");
    let (status, body) = post(
        &harness,
        "/athlete/heart-rates",
        r#"{"max_heart_rate_bpm": 186}"#,
    );
    assert_eq!(status, 400, "the declaration was written: {body}");
}

#[test]
fn a_change_carries_the_other_value_forward_and_leaves_the_earlier_one() {
    let harness = Harness::empty("heart-rates-history");
    let (status, _) = post(&harness, "/athlete/heart-rates", PAIR);
    assert_eq!(status, 200);

    // The resting rate changes; the maximum carries forward, so the
    // declaration stays one coherent statement.
    let (status, body) = post(
        &harness,
        "/athlete/heart-rates",
        r#"{"resting_heart_rate_bpm": 50, "resting_source": "watch"}"#,
    );
    assert_eq!(status, 200, "the declaration was refused: {body}");
    assert_eq!(body["declaration"]["max"]["value_bpm"], 186.0);
    assert_eq!(body["declaration"]["resting"]["value_bpm"], 50.0);

    // And the earlier declaration stays in the history, with its own dates.
    let answer = harness.ok_get("/athlete/heart-rates");
    let history = answer["history"].as_array().cloned().unwrap_or_default();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["in_force"], true);
    assert_eq!(history[1]["in_force"], false);
}

#[test]
fn the_watch_s_own_floors_are_stated_beside_the_declared_ones() {
    let harness = Harness::empty("heart-rates-compare");
    harness.write_zone_model(common::zone_model());
    let (status, _) = post(&harness, "/athlete/heart-rates", PAIR);
    assert_eq!(status, 200);

    let answer = harness.ok_get("/athlete/heart-rates");
    // The watch's own floors: the other fact.
    assert_eq!(answer["watch"]["floors"]["zone1"]["raw"], 115.0);
    assert_eq!(answer["watch"]["read_on"], common::TODAY);
    let watch_source = answer["watch"]["source"].as_str().unwrap_or_default();
    // The watch's own model, and the day it was read: the wording is the
    // core's, and this holds whichever form it takes.
    assert!(watch_source.contains("your zones"), "{watch_source}");
    assert!(watch_source.contains("183 bpm"), "{watch_source}");

    // And the difference, boundary by boundary: a fact, and not a
    // correction.
    let difference = answer["difference"].as_array().cloned().unwrap_or_default();
    assert_eq!(difference.len(), 5);
    assert_eq!(difference[0]["declared_bpm"], 119.0);
    assert_eq!(difference[0]["watch_bpm"], 115.0);
    assert_eq!(difference[0]["difference_bpm"], 4.0);
    assert_eq!(difference[3]["difference_bpm"], 0.0);
    assert_eq!(difference[4]["difference_bpm"], 8.0);
}

/// Write a run whose FIT file is the corpus file named.
///
/// The run carries heart rate, so the zones can be measured on it.
fn write_fit_run(harness: &Harness, name: &str) -> i64 {
    harness.write_run(stride_store::NewActivity {
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
        fit_blob: common::corpus_fit(name),
        created_at: common::instant(),
        updated_at: common::instant(),
    })
}

#[test]
fn the_zones_of_a_run_are_measured_against_the_declared_bands() {
    let harness = Harness::empty("heart-rates-zones");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    // The watch's own model, read before the run.
    let read_at = common::instant() - chrono::Duration::days(2);
    harness.write_zone_model(stride_store::NewHeartRateZoneModel {
        read_at,
        ..common::zone_model()
    });
    let (status, _) = post(&harness, "/athlete/heart-rates", PAIR);
    assert_eq!(status, 200);

    let answer = harness.ok_get(&format!("/fitness/zones?id={id}"));
    // The basis is named, and it is the declared one.
    assert_eq!(answer["bands_basis"]["kind"], "declared");
    assert_eq!(
        answer["bands_basis"]["label"],
        "the bands from the heart rates you entered"
    );
    // And the bands are the declared ones.
    assert_eq!(answer["bands"]["floors"]["zone1"]["raw"], 119.0);
    assert_eq!(answer["bands"]["range"], "reserve");
    // The watch's own model is the other fact, and it is stated beside it.
    assert_eq!(answer["model"]["floors"]["zone1"]["raw"], 115.0);
}

#[test]
fn a_run_s_own_zones_say_when_the_zones_begin_to_apply() {
    let harness = Harness::empty("heart-rates-apply-from");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let read_at = common::instant() - chrono::Duration::days(2);
    harness.write_zone_model(stride_store::NewHeartRateZoneModel {
        read_at,
        ..common::zone_model()
    });

    let answer = harness.ok_get(&format!("/fitness/zones?id={id}"));
    // The day the athlete's own zones begin to apply, in the core's own
    // words — the run's own page says it.
    let apply_from = answer["apply_from"].as_str().unwrap_or_default();
    assert!(apply_from.contains("your watch's zones apply to the runs from"));
    assert!(apply_from.contains("10 May 2026"));
}

#[test]
fn without_a_model_there_is_no_day_the_zones_begin_to_apply() {
    let harness = Harness::empty("heart-rates-no-apply-from");
    let id = write_fit_run(&harness, "hr_gps_run.fit");

    let answer = harness.ok_get(&format!("/fitness/zones?id={id}"));
    // No model has ever been read: there is no such day, and the reason
    // stands as it is.
    assert!(answer["apply_from"].is_null());
    assert!(answer["unavailable"]
        .as_str()
        .unwrap_or_default()
        .contains("not measured"));
}

#[test]
fn without_a_declaration_the_zones_are_measured_against_the_watch_s_model() {
    let harness = Harness::empty("heart-rates-no-declaration");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    let read_at = common::instant() - chrono::Duration::days(2);
    harness.write_zone_model(stride_store::NewHeartRateZoneModel {
        read_at,
        ..common::zone_model()
    });

    let answer = harness.ok_get(&format!("/fitness/zones?id={id}"));
    assert_eq!(answer["bands_basis"]["kind"], "watch");
    assert_eq!(answer["bands_basis"]["label"], "your watch's zones");
    assert!(answer["bands"].is_null());
}
