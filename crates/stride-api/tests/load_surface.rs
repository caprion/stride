//! The training load, on the contract.
//!
//! The route answers with the load over a window: the daily series, the
//! two weighted series, the form, the ratio, the week's impulse, monotony
//! and strain, and the recovery scenario with its assumption.
//!
//! Three things are held here:
//!
//! * the **numbers are the core's** — the route reads them through the
//!   seam and writes them as they are;
//! * **no value is the absence of the value** — a store with no zone model
//!   answers with a reason and no numbers, never a zero and never a
//!   guessed maximum heart rate;
//! * the **window is the Progress window**, and the series is warmed with
//!   the history before it, so the numbers at the start of a window are not
//!   the numbers of a series that began there.
//!
//! Every fixture here is synthetic: invented zone floors, an invented max
//! heart rate, an invented resting heart rate and invented dates.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::NewHeartRateZoneModel;

/// The day the harness reads the clock on.
const TODAY: &str = common::TODAY;

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

/// A measured resting heart rate on every day of a month.
fn resting_hr_every_day(harness: &common::Harness, month: u32, days: u32) {
    for day in 1..=days {
        let date = format!("2026-{month:02}-{day:02}");
        harness.write_wellness(common::wellness_with(
            &date,
            Some(22_200.0),
            Some(48.0),
            Some(50.0),
        ));
    }
}

/// A run every other day of a month, on the odd days.
fn runs_every_other_day(harness: &common::Harness, month: u32, days: u32) {
    for day in (1..=days).step_by(2) {
        let date = format!("2026-{month:02}-{day:02}");
        harness.write_run(common::run(
            &format!("a-run-on-the-{month}-{day}"),
            &date,
            10_000.0,
            3_600.0,
        ));
    }
}

/// Two months of synthetic history: a measured resting heart rate every
/// day and a run every other day.
fn two_months(harness: &common::Harness) {
    resting_hr_every_day(harness, 3, 31);
    resting_hr_every_day(harness, 4, 30);
    resting_hr_every_day(harness, 5, 12);
    runs_every_other_day(harness, 3, 31);
    runs_every_other_day(harness, 4, 30);
    runs_every_other_day(harness, 5, 12);
}

#[test]
fn the_route_answers_with_the_series_the_core_computed() {
    let harness = common::Harness::empty("load-route");
    write_zone_model(&harness);
    two_months(&harness);

    let body = harness.ok_get("/insights/load?weeks=2");

    // The window: the same calendar weeks the Progress route shows, and
    // the warm-up the series was seeded with.
    assert_eq!(body["window"]["from"], "2026-05-04");
    assert_eq!(body["window"]["to"], TODAY);
    assert_eq!(body["window"]["weeks"], 2);
    assert_eq!(body["window"]["warmup_days"], 210);
    // The dates are the api's own rendering.
    assert_eq!(body["window"]["from_label"], "4 May 2026");
    assert_eq!(body["window"]["to_label"], "12 May 2026");

    // The load is there, and it is the core's.
    let load = &body["load"];
    assert!(load.is_object(), "the route answered no load: {body}");
    assert_eq!(body["unavailable"], serde_json::Value::Null);

    // The daily series: one day per calendar day the record covers, with
    // the day's load as the core renders it.
    let days = load["days"].as_array().expect("the days");
    assert!(!days.is_empty());
    let first = &days[0];
    assert_eq!(first["date"], "2026-05-04");
    assert_eq!(first["label"], "4 May 2026");
    assert!(first["load"]["raw"].is_number());
    assert!(first["load"]["text"].is_string());
    assert_eq!(first["reason"], serde_json::Value::Null);

    // The two weighted series, the form and the ratio, at every day.
    let states = load["states"].as_array().expect("the states");
    assert_eq!(states.len(), days.len());
    let last = states.last().expect("a state");
    assert_eq!(last["date"], TODAY);
    assert_eq!(last["label"], "12 May 2026");
    assert!(last["acute"]["raw"].is_number());
    assert!(last["chronic"]["raw"].is_number());
    assert!(last["form"]["raw"].is_number());
    assert!(last["ratio"]["raw"].is_number());

    // The week: the impulse, the monotony and the strain.
    let week = &load["week"];
    assert!(week["impulse"]["raw"].is_number());
    assert!(week["monotony"]["raw"].is_number());
    assert!(week["strain"]["raw"].is_number());

    // The assumption, in the core's own words: the coefficient, where the
    // maximum heart rate came from and when the model was read, and the
    // two windows.
    let assumption = &load["assumptions"];
    assert_eq!(assumption["banister_k"], 1.92);
    assert_eq!(assumption["max_heart_rate_bpm"], 190.0);
    assert_eq!(assumption["threshold_hr_bpm"], 175.0);
    assert_eq!(assumption["zone_model_read_on"], TODAY);
    assert_eq!(assumption["zone_model_read_on_label"], "12 May 2026");
    assert_eq!(assumption["acute_window_days"], 7);
    assert_eq!(assumption["chronic_window_days"], 42);
    let statement = assumption["statement"].as_str().expect("the statement");
    assert!(statement.contains("Banister k = 1.92"));
    assert!(statement.contains("read on 12 May 2026"));
    assert!(statement.contains("chronic window 42 days"));
}

#[test]
fn the_scenario_is_the_core_s_scenario_with_its_assumption() {
    let harness = common::Harness::empty("load-scenario");
    write_zone_model(&harness);
    two_months(&harness);

    let body = harness.ok_get("/insights/load?weeks=2");
    let scenario = &body["scenario"];
    assert!(scenario.is_object(), "there is no scenario: {body}");

    // The band is a range, and it is named as one.
    assert_eq!(scenario["band"], "0.8 to 1.3");

    // The scenario's own words: how many rest days the ratio needs to be
    // back inside the band, and how many until the form turns positive.
    let statement = scenario["statement"].as_str().expect("the statement");
    assert!(
        statement.contains("acute-to-chronic ratio"),
        "the scenario does not name the ratio: {statement}"
    );
    assert!(
        statement.contains("0.8 to 1.3"),
        "the scenario does not name the band: {statement}"
    );
    assert!(
        statement.contains("the form"),
        "the scenario does not name the form: {statement}"
    );

    // The assumption travels with it.
    let assumption = scenario["assumption"].as_str().expect("the assumption");
    assert!(assumption.contains("Banister k = 1.92"));
    assert!(assumption.contains("no training for 7 days"));
    assert!(assumption.contains("chronic window 42 days"));

    // The projection: seven days of no training.
    let projection = scenario["projection"].as_array().expect("the projection");
    assert_eq!(projection.len(), 7);
    assert_eq!(projection[0]["day"], 1);
    assert_eq!(projection[6]["day"], 7);
    assert!(projection[0]["form"]["raw"].is_number());
    assert!(projection[0]["ratio"]["raw"].is_number());
}

#[test]
fn a_store_with_no_zone_model_has_no_load_and_a_reason() {
    let harness = common::Harness::empty("load-no-model");
    two_months(&harness);

    let body = harness.ok_get("/insights/load?weeks=2");

    // No value is the absence of the value: there is no load, and the
    // reason says why.
    assert_eq!(body["load"], serde_json::Value::Null);
    assert_eq!(body["unavailable"], "no_zone_model");
    assert_eq!(body["scenario"], serde_json::Value::Null);
    // And not a zero anywhere: the window still names itself.
    assert_eq!(body["window"]["weeks"], 2);
}

#[test]
fn a_window_with_no_recorded_days_has_no_load_and_a_reason() {
    let harness = common::Harness::empty("load-empty-window");
    write_zone_model(&harness);
    // A fortnight in January: the two-week window of May holds nothing.
    resting_hr_every_day(&harness, 1, 14);

    let body = harness.ok_get("/insights/load?weeks=2");
    assert_eq!(body["load"], serde_json::Value::Null);
    assert_eq!(body["unavailable"], "no_history");
    assert_eq!(body["scenario"], serde_json::Value::Null);
}

#[test]
fn the_window_follows_the_range() {
    let harness = common::Harness::empty("load-window");
    write_zone_model(&harness);
    two_months(&harness);

    let two = harness.ok_get("/insights/load?weeks=2");
    let four = harness.ok_get("/insights/load?weeks=4");

    assert_eq!(two["window"]["from"], "2026-05-04");
    assert_eq!(four["window"]["from"], "2026-04-20");
    assert_eq!(four["window"]["weeks"], 4);
    // The window's days are the window's days: the longer window holds
    // more of them.
    let short_days = two["load"]["days"].as_array().expect("the days").len();
    let long_days = four["load"]["days"].as_array().expect("the days").len();
    assert!(long_days > short_days);
}

#[test]
fn a_window_outside_the_bound_is_refused() {
    let harness = common::Harness::empty("load-window-bound");
    let answer = harness.router.call("GET", "/insights/load?weeks=0", None);
    assert_eq!(answer.status, 400);
    let answer = harness.router.call("GET", "/insights/load?weeks=105", None);
    assert_eq!(answer.status, 400);
}

#[test]
fn the_series_is_warmed_with_the_history_before_the_window() {
    let harness = common::Harness::empty("load-warm-up");
    write_zone_model(&harness);
    // A resting heart rate and a run a hundred days before the window,
    // and a measured resting heart rate every day of April and May.
    resting_hr_every_day(&harness, 1, 31);
    runs_every_other_day(&harness, 1, 31);
    resting_hr_every_day(&harness, 4, 30);
    resting_hr_every_day(&harness, 5, 12);

    let body = harness.ok_get("/insights/load?weeks=4");
    let states = body["load"]["states"].as_array().expect("the states");
    let first = &states[0];

    // The window's first day is the first day the record covers inside
    // it.
    assert_eq!(first["date"], "2026-04-20");
    // The chronic load there is not zero: the series was seeded with the
    // history before the window. A series that began at the window would
    // carry nothing into it.
    assert!(
        first["chronic"]["raw"].as_f64().expect("the chronic load") > 0.0,
        "the chronic load at the window's first day is zero: {first}"
    );
}

#[test]
fn a_day_with_nothing_to_weigh_carries_its_reason() {
    let harness = common::Harness::empty("load-day-reason");
    write_zone_model(&harness);
    // A ride with no heart rate: nothing on the day could be weighed.
    harness.write_run(common::run_without_readings(
        "a-ride-with-no-heart-rate",
        "2026-05-11",
        20_000.0,
        3_600.0,
    ));
    resting_hr_every_day(&harness, 5, 12);

    let body = harness.ok_get("/insights/load?weeks=2");
    let days = body["load"]["days"].as_array().expect("the days");
    let day = days
        .iter()
        .find(|day| day["date"] == "2026-05-11")
        .expect("the ride's day");
    // No value, and the reason that says so: never a zero.
    assert_eq!(day["load"], serde_json::Value::Null);
    assert_eq!(day["reason"], "nothing_weighable");
}
