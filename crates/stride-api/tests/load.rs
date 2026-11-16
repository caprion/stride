//! The training load, read from the store.
//!
//! The seam reads three facts — the athlete's stored zone model, the
//! measured resting heart rate of the daily summary and the stored
//! activities — and hands them to the core's engine. Every fixture here is
//! synthetic: invented zone floors, an invented max heart rate, an invented
//! resting heart rate and invented dates.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_api::load::{load_inputs, load_report};
use stride_core::date::Date;
use stride_core::load::{LoadAnswer, NoLoadReason};
use stride_store::NewHeartRateZoneModel;

/// The synthetic zone model: a max heart rate of 190 and five invented
/// floors, read on the day the tests read the clock at.
fn write_zone_model(harness: &common::Harness) {
    let model = NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(190.0),
        zone1_floor: 100.0,
        zone2_floor: 120.0,
        zone3_floor: 140.0,
        zone4_floor: 160.0,
        zone5_floor: 175.0,
        read_at: common::instant(),
    };
    harness
        .store
        .write_hr_zone_model_if_new(&model)
        .expect("the zone model could be written");
}

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

#[test]
fn the_load_is_read_from_the_stored_zone_model_and_the_measured_heart_rate() {
    let harness = common::Harness::empty("load-seam");
    write_zone_model(&harness);
    // A run on the tenth, a resting heart rate on the tenth and on the two
    // days after it, and a day the record does not cover at all.
    harness.write_run(common::run(
        "a-run-on-the-tenth",
        "2026-05-10",
        10_000.0,
        3_600.0,
    ));
    for day in ["2026-05-10", "2026-05-11", "2026-05-12"] {
        harness.write_wellness(common::wellness_with(
            day,
            Some(22_200.0),
            Some(48.0),
            Some(50.0),
        ));
    }

    let inputs = load_inputs(&harness.api, date("2026-05-09"), date("2026-05-12")).unwrap();

    // The days of the series are the days the record covers: the run's day
    // and the two rest days. The ninth is not covered, so it is not in the
    // series at all — it is missing, not a rest day.
    let days: Vec<(String, bool)> = inputs
        .days
        .iter()
        .map(|day| (day.date.to_iso(), day.rest_day))
        .collect();
    assert_eq!(
        days,
        vec![
            ("2026-05-10".to_string(), false),
            ("2026-05-11".to_string(), true),
            ("2026-05-12".to_string(), true),
        ]
    );

    // The zone model is the stored one, with the day it was read.
    let model = inputs.zone_model.expect("the stored zone model");
    assert_eq!(model.max_heart_rate, Some(190.0));
    assert_eq!(model.floors.zone5, 175.0);
    assert_eq!(model.read_on, date("2026-05-12"));

    // The measured resting heart rate is the store's, and the readings before
    // the window are read too — the core falls back to the most recent one.
    assert!(inputs
        .resting_hr
        .iter()
        .any(|reading| reading.date == date("2026-05-12") && reading.resting_hr.bpm() == 50.0));

    // The run's day carries a load; the rest days carry zero.
    let answer = load_report(&harness.api, date("2026-05-09"), date("2026-05-12")).unwrap();
    let report = match answer {
        LoadAnswer::Known(report) => report,
        LoadAnswer::Unavailable(reason) => panic!("the load is not available: {reason:?}"),
    };
    let loads: Vec<(String, Option<f64>)> = report
        .days
        .iter()
        .map(|day| (day.date.to_iso(), day.load))
        .collect();
    assert_eq!(
        loads,
        vec![
            ("2026-05-10".to_string(), Some(54.0)),
            ("2026-05-11".to_string(), Some(0.0)),
            ("2026-05-12".to_string(), Some(0.0)),
        ]
    );
}

#[test]
fn without_a_stored_zone_model_the_load_has_no_value() {
    let harness = common::Harness::empty("load-without-a-model");
    harness.write_run(common::run(
        "a-run-on-the-tenth",
        "2026-05-10",
        10_000.0,
        3_600.0,
    ));
    harness.write_wellness(common::wellness_with(
        "2026-05-10",
        Some(22_200.0),
        Some(48.0),
        Some(50.0),
    ));

    let answer = load_report(&harness.api, date("2026-05-09"), date("2026-05-12")).unwrap();
    assert_eq!(answer, LoadAnswer::Unavailable(NoLoadReason::NoZoneModel));
}
