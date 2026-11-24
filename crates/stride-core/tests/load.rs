//! The training load: the series, the missing inputs, and the scenario.
//!
//! Every fixture here is synthetic: synthetic zone floors, a synthetic max
//! heart rate, a synthetic resting heart rate and synthetic dates.
//!
//! The expected numbers are the **reference implementation's own**: the same
//! synthetic series through the reference's arithmetic — the TRIMP of the
//! day over the threshold, the quantised intensity, the seven-day and
//! forty-two-day weighted averages, the seven-day window and the seven-day
//! projection — gives exactly these numbers. They are worked out from the
//! definition in the module, not copied from the implementation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::load::{
    load_report, recovery_scenario, ActivityLoad, DayInput, LoadAnswer, LoadInputs, LoadZoneModel,
    NoLoadReason, BANISTER_K,
};
use stride_core::types::{HeartRate, RestingHr};
use stride_core::zones::ZoneFloors;

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

fn heart_rate(bpm: f64) -> HeartRate {
    HeartRate::from_bpm(bpm).expect("a heart rate in range")
}

fn resting_hr(bpm: f64) -> RestingHr {
    RestingHr::from_bpm(bpm).expect("a resting heart rate in range")
}

/// The synthetic zone model: a max heart rate of 190 and five floors whose
/// zone-5 floor is the threshold of 175.
fn zone_model() -> LoadZoneModel {
    LoadZoneModel {
        max_heart_rate: Some(190.0),
        floors: ZoneFloors {
            zone1: 100.0,
            zone2: 120.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        },
        read_on: date("2026-03-12"),
    }
}

/// A resting heart rate of 50 bpm on every day of March and April 2026.
fn resting_hr_every_day() -> Vec<stride_core::load::RestingHrReading> {
    let mut readings = Vec::new();
    for day in 1..=31 {
        readings.push(stride_core::load::RestingHrReading {
            date: date(&format!("2026-03-{day:02}")),
            resting_hr: resting_hr(50.0),
        });
    }
    for day in 1..=30 {
        readings.push(stride_core::load::RestingHrReading {
            date: date(&format!("2026-04-{day:02}")),
            resting_hr: resting_hr(50.0),
        });
    }
    readings
}

/// One day of the series, with no activity on it.
fn rest_day(text: &str) -> DayInput {
    DayInput {
        date: date(text),
        rest_day: true,
    }
}

/// One day of the series that carries an activity.
fn run_day(text: &str) -> DayInput {
    DayInput {
        date: date(text),
        rest_day: false,
    }
}

/// One activity: `minutes` of moving time at `average_hr`.
fn activity(text: &str, minutes: f64, average_hr: f64) -> ActivityLoad {
    ActivityLoad {
        date: date(text),
        moving_time_s: minutes * 60.0,
        average_hr: Some(heart_rate(average_hr)),
    }
}

/// The report of a series, or a panic when the load is not available.
fn known(inputs: &LoadInputs) -> stride_core::load::LoadReport {
    match load_report(inputs) {
        LoadAnswer::Known(report) => report,
        LoadAnswer::Unavailable(reason) => panic!("the load is not available: {reason:?}"),
    }
}

/// The daily loads of a report, as a list of `(day, load)`.
fn loads(report: &stride_core::load::LoadReport) -> Vec<(String, Option<f64>)> {
    report
        .days
        .iter()
        .map(|day| (day.date.to_iso(), day.load))
        .collect()
}

#[test]
fn a_series_carries_its_daily_loads_and_its_two_weighted_series() {
    // A fortnight of training: a run, a rest day, a run, a day the record
    // does not cover, and so on. The expected numbers are worked out from
    // the model: the day's load is the TRIMP of the day over the threshold,
    // and the two series are the seven-day and forty-two-day weighted
    // averages of those loads.
    let days = vec![
        run_day("2026-04-01"),
        run_day("2026-04-02"),
        rest_day("2026-04-03"),
        run_day("2026-04-04"),
        // 2026-04-05 is not covered by the record.
        run_day("2026-04-06"),
        rest_day("2026-04-07"),
        run_day("2026-04-08"),
        run_day("2026-04-09"),
        rest_day("2026-04-10"),
        run_day("2026-04-11"),
        run_day("2026-04-12"),
        rest_day("2026-04-13"),
        run_day("2026-04-14"),
    ];
    let activities = vec![
        activity("2026-04-01", 60.0, 140.0),
        activity("2026-04-02", 45.0, 155.0),
        activity("2026-04-04", 90.0, 150.0),
        activity("2026-04-06", 30.0, 165.0),
        activity("2026-04-08", 75.0, 145.0),
        activity("2026-04-09", 40.0, 160.0),
        activity("2026-04-11", 55.0, 135.0),
        activity("2026-04-12", 50.0, 170.0),
        activity("2026-04-14", 65.0, 150.0),
    ];
    let inputs = LoadInputs {
        days,
        activities,
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);

    // The daily loads, each a whole number.
    assert_eq!(
        loads(&report),
        vec![
            ("2026-04-01".to_string(), Some(44.0)),
            ("2026-04-02".to_string(), Some(48.0)),
            ("2026-04-03".to_string(), Some(0.0)),
            ("2026-04-04".to_string(), Some(84.0)),
            ("2026-04-06".to_string(), Some(40.0)),
            ("2026-04-07".to_string(), Some(0.0)),
            ("2026-04-08".to_string(), Some(63.0)),
            ("2026-04-09".to_string(), Some(48.0)),
            ("2026-04-10".to_string(), Some(0.0)),
            ("2026-04-11".to_string(), Some(36.0)),
            ("2026-04-12".to_string(), Some(75.0)),
            ("2026-04-13".to_string(), Some(0.0)),
            ("2026-04-14".to_string(), Some(61.0)),
        ]
    );

    // The two series: the acute load, the chronic load, the form and the
    // ratio at the last day.
    let last = report.latest_state().expect("a state");
    assert_eq!(last.date, date("2026-04-14"));
    assert_eq!(last.acute, 37.8);
    assert_eq!(last.chronic, 41.7);
    assert_eq!(last.form, 3.8);
    assert_eq!(last.ratio, 0.91);

    // The first day seeds both series with its own load.
    let first = report.states.first().expect("a state");
    assert_eq!(first.acute, 44.0);
    assert_eq!(first.chronic, 44.0);
    assert_eq!(first.form, 0.0);
    assert_eq!(first.ratio, 1.0);

    // The week: the last seven days, 2026-04-08 to 2026-04-14.
    let week = report.week.expect("a week");
    assert_eq!(week.impulse, 283.0);
    assert_eq!(week.monotony, 1.45);
    assert_eq!(week.strain, 409.0);

    // And the same numbers, written the way the load writes them.
    assert_eq!(week.impulse_text(), "283");
    assert_eq!(week.monotony_text(), "1.45");
    assert_eq!(week.strain_text(), "409");
}

#[test]
fn a_day_the_record_does_not_cover_is_not_a_rest_day() {
    // A day with no activity and no rest-day mark has **no load**: absence
    // is not zero.
    let inputs = LoadInputs {
        days: vec![DayInput {
            date: date("2026-04-01"),
            rest_day: false,
        }],
        activities: Vec::new(),
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-01".to_string(), None)]);
    assert_eq!(report.days[0].reason, Some(NoLoadReason::NothingWeighable));

    // A day that **is** a known rest day carries zero.
    let inputs = LoadInputs {
        days: vec![rest_day("2026-04-01")],
        activities: Vec::new(),
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-01".to_string(), Some(0.0))]);
    assert_eq!(report.days[0].reason, None);
}

#[test]
fn a_gap_in_the_record_decays_the_two_series_across_it() {
    // Two days, three days apart. The averages are over calendar days, so
    // the day the record does not cover is a day that passes: the series
    // decay across the gap.
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01"), run_day("2026-04-04")],
        activities: vec![
            activity("2026-04-01", 60.0, 140.0),
            activity("2026-04-04", 60.0, 140.0),
        ],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    let last = report.latest_state().expect("a state");
    // 44 after the first day, decayed over the three days between the two
    // days of the record and then weighted with the new day's load.
    assert_eq!(last.acute, 34.5);
    assert_eq!(last.chronic, 42.0);
    assert_eq!(last.form, 7.5);
    assert_eq!(last.ratio, 0.82);
}

#[test]
fn an_empty_history_has_no_value() {
    let inputs = LoadInputs {
        days: Vec::new(),
        activities: Vec::new(),
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    assert_eq!(
        load_report(&inputs),
        LoadAnswer::Unavailable(NoLoadReason::NoHistory)
    );
}

#[test]
fn a_single_activity_is_one_day_of_load() {
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01")],
        activities: vec![activity("2026-04-01", 60.0, 140.0)],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-01".to_string(), Some(44.0))]);
    let state = report.latest_state().expect("a state");
    assert_eq!(state.acute, 44.0);
    assert_eq!(state.chronic, 44.0);
    assert_eq!(state.form, 0.0);
    assert_eq!(state.ratio, 1.0);
    // A week needs seven days: one day is not a week.
    assert_eq!(report.week, None);
}

#[test]
fn no_zone_model_is_no_value_and_never_a_default() {
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01")],
        activities: vec![activity("2026-04-01", 60.0, 140.0)],
        resting_hr: resting_hr_every_day(),
        zone_model: None,
    };
    assert_eq!(
        load_report(&inputs),
        LoadAnswer::Unavailable(NoLoadReason::NoZoneModel)
    );
}

#[test]
fn a_stored_model_without_a_max_heart_rate_is_no_value() {
    let mut model = zone_model();
    model.max_heart_rate = None;
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01")],
        activities: vec![activity("2026-04-01", 60.0, 140.0)],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(model),
    };
    assert_eq!(
        load_report(&inputs),
        LoadAnswer::Unavailable(NoLoadReason::NoMaxHeartRate)
    );
}

#[test]
fn a_model_whose_floors_do_not_increase_is_no_value() {
    let mut model = zone_model();
    model.floors.zone4 = 130.0;
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01")],
        activities: vec![activity("2026-04-01", 60.0, 140.0)],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(model),
    };
    assert_eq!(
        load_report(&inputs),
        LoadAnswer::Unavailable(NoLoadReason::UnusableZoneModel)
    );
}

#[test]
fn the_resting_heart_rate_falls_back_to_the_last_reading_on_or_before() {
    // The watch did not read a resting heart rate on the day of the run, so
    // the most recent reading on or before it is used: the assumption is
    // that the resting heart rate has not moved since it was measured.
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-02")],
        activities: vec![activity("2026-04-02", 60.0, 140.0)],
        resting_hr: vec![stride_core::load::RestingHrReading {
            date: date("2026-04-01"),
            resting_hr: resting_hr(50.0),
        }],
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-02".to_string(), Some(44.0))]);
}

#[test]
fn a_day_with_no_resting_heart_rate_at_all_has_no_load() {
    // No reading on or before the day: the day's load is "no value" — never
    // zero and never a guessed number.
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-02")],
        activities: vec![activity("2026-04-02", 60.0, 140.0)],
        resting_hr: vec![stride_core::load::RestingHrReading {
            date: date("2026-04-03"),
            resting_hr: resting_hr(50.0),
        }],
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-02".to_string(), None)]);
    assert_eq!(report.days[0].reason, Some(NoLoadReason::NoRestingHr));
}

#[test]
fn a_ride_without_a_heart_rate_carries_no_load() {
    // v2 has no FTP model, so a ride without a heart rate cannot be
    // weighed. It contributes nothing, and the day it is on has no load:
    // it was not a rest day, and nothing on it could be weighed.
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01")],
        activities: vec![ActivityLoad {
            date: date("2026-04-01"),
            moving_time_s: 3600.0,
            average_hr: None,
        }],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(loads(&report), vec![("2026-04-01".to_string(), None)]);
    assert_eq!(report.days[0].reason, Some(NoLoadReason::NothingWeighable));
}

#[test]
fn a_week_with_a_missing_day_has_no_impulse_and_no_monotony() {
    // Six days with a load and a seventh the record does not cover: the
    // week is not a week, and absence is not zero.
    let mut days: Vec<DayInput> = (1..=7)
        .map(|day| run_day(&format!("2026-04-{day:02}")))
        .collect();
    let activities: Vec<ActivityLoad> = (1..=6)
        .map(|day| activity(&format!("2026-04-{day:02}"), 60.0, 140.0))
        .collect();
    // The seventh day is in the record but carries nothing that can be
    // weighed: its load is missing, so the week is not a week.
    days.push(DayInput {
        date: date("2026-04-07"),
        rest_day: false,
    });
    let inputs = LoadInputs {
        days,
        activities,
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    assert_eq!(report.week, None);
}

#[test]
fn the_scenario_projects_seven_days_of_no_training() {
    // Twelve alternating days and then two hard ones: the acute load is
    // above the chronic one, the ratio is above its band and the form is
    // negative. The projection decays both series with zero load.
    let mut days: Vec<DayInput> = Vec::new();
    let mut activities: Vec<ActivityLoad> = Vec::new();
    for day in 1..=12 {
        let text = format!("2026-04-{day:02}");
        if day % 2 == 1 {
            days.push(run_day(&text));
            activities.push(activity(&text, 40.0, 130.0));
        } else {
            days.push(rest_day(&text));
        }
    }
    days.push(run_day("2026-04-13"));
    days.push(run_day("2026-04-14"));
    activities.push(activity("2026-04-13", 95.0, 172.0));
    activities.push(activity("2026-04-14", 95.0, 172.0));

    let inputs = LoadInputs {
        days,
        activities,
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    let last = report.latest_state().expect("a state");
    assert_eq!(last.acute, 46.5);
    assert_eq!(last.chronic, 26.1);
    assert_eq!(last.form, -20.4);
    assert_eq!(last.ratio, 1.78);

    let scenario = recovery_scenario(&report).expect("a scenario");
    // The ratio is back inside its band after three days of no training, and
    // the form turns positive after five.
    assert_eq!(scenario.days_to_ratio_band, Some(3));
    assert_eq!(scenario.days_to_positive_form, Some(5));

    // The projection itself: seven days, each one the decay of the last.
    let projection: Vec<(u32, f64, f64)> = scenario
        .projection
        .iter()
        .map(|day| (day.day, day.form, day.ratio))
        .collect();
    assert_eq!(
        projection,
        vec![
            (1, -14.8, 1.58),
            (2, -10.1, 1.4),
            (3, -6.0, 1.25),
            (4, -2.5, 1.11),
            (5, 0.4, 0.98),
            (6, 2.9, 0.87),
            (7, 5.0, 0.77),
        ]
    );
}

#[test]
fn the_scenario_names_its_assumption() {
    let mut days: Vec<DayInput> = Vec::new();
    let mut activities: Vec<ActivityLoad> = Vec::new();
    for day in 1..=12 {
        let text = format!("2026-04-{day:02}");
        if day % 2 == 1 {
            days.push(run_day(&text));
            activities.push(activity(&text, 40.0, 130.0));
        } else {
            days.push(rest_day(&text));
        }
    }
    days.push(run_day("2026-04-13"));
    days.push(run_day("2026-04-14"));
    activities.push(activity("2026-04-13", 95.0, 172.0));
    activities.push(activity("2026-04-14", 95.0, 172.0));

    let inputs = LoadInputs {
        days,
        activities,
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    let scenario = recovery_scenario(&report).expect("a scenario");
    // The day the model was read is the caller's: the core puts the day it
    // is given into its own sentence. The api writes dates the way the
    // product shows them, and the core does not know how.
    let statement = scenario.statement("12 Mar 2026");

    // The wording is stable, and it names what it assumed: the coefficient,
    // where the max heart rate came from and when the model was read, the
    // two windows, and the assumption of no training.
    assert_eq!(
        statement,
        "the last 7 days are expected back inside 0.8 to 1.3 times the last 6 weeks in 3 days \
         · the difference turns positive in 5 days \
         · no training for 7 days \
         · Banister k = 1.92 \
         · max heart rate 190 bpm from your zones, taken from your watch on 12 Mar 2026 \
         · the last 7 days · the last 6 weeks"
    );
    assert!(statement.contains("1.92"));
    assert!(statement.contains("12 Mar 2026"));
    assert!(statement.contains("no training"));

    // The scenario on its own, without the assumption attached: the same
    // two sentences, and no assumption to read them without.
    let scenario_only = scenario.scenario_text();
    assert_eq!(
        scenario_only,
        "the last 7 days are expected back inside 0.8 to 1.3 times the last 6 weeks in 3 days \
         · the difference turns positive in 5 days"
    );
    assert!(!scenario_only.contains("Banister"));
}

#[test]
fn a_scenario_that_is_already_inside_its_band_says_so() {
    // A run and a rest day: the acute load has fallen below the chronic one,
    // so the ratio is inside its band and the form is positive. The scenario
    // says "now" and not "in 3 days".
    let inputs = LoadInputs {
        days: vec![run_day("2026-04-01"), rest_day("2026-04-02")],
        activities: vec![activity("2026-04-01", 60.0, 140.0)],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);
    let scenario = recovery_scenario(&report).expect("a scenario");
    assert_eq!(scenario.days_to_ratio_band, Some(0));
    assert_eq!(scenario.days_to_positive_form, Some(0));
    assert!(scenario
        .statement("12 Mar 2026")
        .contains("are already inside 0.8 to 1.3 times the last 6 weeks"));
    assert!(scenario
        .statement("12 Mar 2026")
        .contains("the difference is positive now"));
}

#[test]
fn the_coefficient_is_the_named_one() {
    // The one modelling constant, named and sourced. If it ever needs to
    // move, it moves here.
    assert_eq!(BANISTER_K, 1.92);
}

#[test]
fn the_numbers_are_rendered_at_the_load_s_own_precision() {
    // The load is dimensionless — there is no unit to write — so the
    // number is the value, and the precision is the load's own: whole
    // numbers for the day's load, the impulse and the strain, one decimal
    // for the acute load, the chronic load and the form, two for the ratio
    // and the monotony.
    let inputs = LoadInputs {
        days: vec![
            run_day("2026-04-01"),
            rest_day("2026-04-02"),
            run_day("2026-04-03"),
            rest_day("2026-04-04"),
            run_day("2026-04-05"),
            rest_day("2026-04-06"),
            run_day("2026-04-07"),
        ],
        activities: vec![
            activity("2026-04-01", 60.0, 140.0),
            activity("2026-04-03", 45.0, 155.0),
            activity("2026-04-05", 90.0, 150.0),
            activity("2026-04-07", 55.0, 135.0),
        ],
        resting_hr: resting_hr_every_day(),
        zone_model: Some(zone_model()),
    };
    let report = known(&inputs);

    // The day's load: a whole number.
    assert_eq!(report.days[0].load_text(), Some("44".to_string()));
    assert_eq!(report.days[1].load_text(), Some("0".to_string()));

    // The state: one decimal for the three series, two for the ratio.
    let state = report.latest_state().expect("a state");
    assert_eq!(state.acute_text(), "35.5");
    assert_eq!(state.chronic_text(), "41.9");
    assert_eq!(state.form_text(), "6.4");
    assert_eq!(state.ratio_text(), "0.85");

    // The week: whole numbers for the impulse and the strain, two for the
    // monotony.
    let week = report.week.expect("a week");
    assert_eq!(week.impulse_text(), "212");
    assert_eq!(week.monotony_text(), "1.02");
    assert_eq!(week.strain_text(), "216");
}
