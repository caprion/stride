//! The sentences come from the core.
//!
//! The direction word, the arithmetic beside it, the direction of travel,
//! the "Rest" line and the pattern sentences are written by `stride-core`.
//! This test holds the answers to that: every sentence in a response is
//! **byte-identical** to what the core renders for the same input, and the
//! example the contract freezes is the example the router produces.
//!
//! There is no second implementation of a sentence here, and there is
//! none to drift.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_core::direction::day_directions;
use stride_core::pattern::{patterns, HistoryDay};
use stride_core::types::{Duration, Hrv, RestingHr, Sleep, Vitals, WellnessDay};
use stride_core::Date;

use common::{NOW, TODAY};

/// The core's history of one test: the days, the sleep, the HRV and the
/// resting heart rate of each.
struct Night {
    date: &'static str,
    sleep_s: f64,
    hrv_ms: f64,
    resting_hr: f64,
    distance_m: Option<f64>,
}

/// The history the contract's example is read from.
///
/// The five nights before the day, and the day itself: a usual of 6 h 40 m
/// of sleep, a usual HRV of 62 ms and a usual resting heart rate of
/// 58 bpm, with the day's own readings on the other side of two of the
/// bands and inside the third.
fn example_nights() -> Vec<Night> {
    vec![
        Night {
            date: "2026-05-07",
            sleep_s: 24_000.0,
            hrv_ms: 54.0,
            resting_hr: 50.0,
            distance_m: None,
        },
        Night {
            date: "2026-05-08",
            sleep_s: 26_400.0,
            hrv_ms: 62.0,
            resting_hr: 58.0,
            distance_m: None,
        },
        Night {
            date: "2026-05-09",
            sleep_s: 27_600.0,
            hrv_ms: 66.0,
            resting_hr: 54.0,
            distance_m: None,
        },
        Night {
            date: "2026-05-10",
            sleep_s: 26_400.0,
            hrv_ms: 62.0,
            resting_hr: 63.0,
            distance_m: None,
        },
        Night {
            date: "2026-05-11",
            sleep_s: 28_800.0,
            hrv_ms: 70.0,
            resting_hr: 58.0,
            distance_m: None,
        },
        Night {
            date: TODAY,
            sleep_s: 22_200.0,
            hrv_ms: 48.0,
            resting_hr: 63.0,
            distance_m: None,
        },
    ]
}

/// The core's reading of a night.
fn core_night(night: &Night) -> Option<WellnessDay> {
    let date = Date::parse(night.date)?;
    let total = Duration::from_seconds(night.sleep_s)?;
    Some(WellnessDay::new(
        date,
        Vitals {
            sleep: Some(Sleep::new(total)),
            hrv: Hrv::from_ms(night.hrv_ms),
            resting_hr: RestingHr::from_bpm(night.resting_hr),
        },
    ))
}

/// Write a history into a harness.
fn seed(harness: &common::Harness, nights: &[Night]) {
    for night in nights {
        harness.write_wellness(common::wellness_with(
            night.date,
            Some(night.sleep_s),
            Some(night.hrv_ms),
            Some(night.resting_hr),
        ));
        if let Some(distance) = night.distance_m {
            harness.write_run(common::run(
                &format!("a-run-on-{}", night.date),
                night.date,
                distance,
                distance / 3.0,
            ));
        }
    }
}

#[test]
fn the_direction_and_the_arithmetic_come_from_the_core() {
    let nights = example_nights();
    let harness = common::Harness::empty("sentences-direction");
    seed(&harness, &nights);

    let body = harness.ok_get("/physiology/vitals");
    let history: Vec<WellnessDay> = nights
        .iter()
        .filter_map(core_night)
        .collect::<Vec<WellnessDay>>();
    let day = match Date::parse(TODAY) {
        Some(day) => day,
        None => panic!("the day could not be read"),
    };
    let directions = day_directions(&history, day);

    for (reading, core) in body["vitals"]
        .as_array()
        .unwrap()
        .iter()
        .zip(directions.vitals.iter())
    {
        assert_eq!(
            reading["direction_text"].as_str(),
            core.direction_text(),
            "the direction word is not the core's"
        );
        assert_eq!(
            reading["deviation_text"].as_str(),
            core.deviation_text().as_deref(),
            "the arithmetic is not the core's"
        );
        assert_eq!(
            reading["travel_text"].as_str(),
            core.travel_text().as_deref(),
            "the direction of travel is not the core's"
        );
        assert_eq!(
            reading["value_text"].as_str(),
            core.value_text().as_deref(),
            "the value is not the core's"
        );
        assert_eq!(
            reading["status_text"].as_str(),
            core.status_text().as_deref(),
            "the status line is not the core's"
        );
        assert_eq!(
            reading["label"].as_str(),
            Some(core.vital.label()),
            "the label is not the catalogue's"
        );
    }
}

#[test]
fn the_rest_line_comes_from_the_core() {
    let nights = example_nights();
    let harness = common::Harness::empty("sentences-rest");
    seed(&harness, &nights);

    let body = harness.ok_get("/physiology/vitals");
    let history: Vec<WellnessDay> = nights.iter().filter_map(core_night).collect();
    let day = match Date::parse(TODAY) {
        Some(day) => day,
        None => panic!("the day could not be read"),
    };
    let directions = day_directions(&history, day);

    assert_eq!(
        body["rest"]["line"].as_str(),
        Some(directions.rest.line().as_str()),
        "the Rest line is not the core's"
    );
    assert_eq!(body["rest"]["label"].as_str(), Some("Rest"));
}

#[test]
fn the_patterns_come_from_the_core() {
    // Three nights on the same side of the usual, with runs on them: the
    // pattern engine has something to say.
    let mut nights: Vec<Night> = Vec::new();
    for index in 0..8 {
        nights.push(Night {
            date: Box::leak(format!("2026-05-{:02}", index + 1).into_boxed_str()),
            sleep_s: 20_000.0,
            hrv_ms: 60.0,
            resting_hr: 58.0,
            distance_m: None,
        });
    }
    for (index, distance) in [8_000.0, 9_000.0, 7_000.0].iter().enumerate() {
        nights.push(Night {
            date: Box::leak(format!("2026-05-{:02}", index + 9).into_boxed_str()),
            sleep_s: 30_000.0,
            hrv_ms: 60.0,
            resting_hr: 58.0,
            distance_m: Some(*distance),
        });
    }
    nights.push(Night {
        date: TODAY,
        sleep_s: 30_000.0,
        hrv_ms: 60.0,
        resting_hr: 58.0,
        distance_m: None,
    });

    let harness = common::Harness::empty("sentences-pattern");
    seed(&harness, &nights);

    let body = harness.ok_get("/physiology/vitals");
    let day = match Date::parse(TODAY) {
        Some(day) => day,
        None => panic!("the day could not be read"),
    };

    let mut core_history: Vec<HistoryDay> = Vec::new();
    for night in nights.iter() {
        let date = match Date::parse(night.date) {
            Some(date) => date,
            None => continue,
        };
        let total = match Duration::from_seconds(night.sleep_s) {
            Some(total) => total,
            None => continue,
        };
        core_history.push(HistoryDay::new(
            date,
            Vitals {
                sleep: Some(Sleep::new(total)),
                hrv: Hrv::from_ms(night.hrv_ms),
                resting_hr: RestingHr::from_bpm(night.resting_hr),
            },
            night.distance_m,
        ));
    }
    let found = patterns(&core_history, day);
    assert!(
        !found.is_empty(),
        "the history has a pattern in it, and the core found none"
    );

    let answered = body["patterns"].as_array().unwrap();
    assert_eq!(answered.len(), found.len(), "the patterns differ in number");
    for (reading, core) in answered.iter().zip(found.iter()) {
        assert_eq!(
            reading["text"].as_str(),
            Some(core.text.as_str()),
            "the pattern sentence is not the core's"
        );
        assert_eq!(
            reading["sample_size"].as_u64(),
            Some(core.sample_size as u64),
            "the pattern's sample size is not the core's"
        );
    }
}

#[test]
fn the_contract_s_example_is_the_example_the_router_produces() {
    let nights = example_nights();
    let harness = common::Harness::empty("sentences-example");
    seed(&harness, &nights);

    let body = harness.ok_get("/physiology/vitals");
    let vitals = body["vitals"].as_array().unwrap();

    assert_eq!(vitals[0]["value_text"].as_str(), Some("6 h 10 m"));
    assert_eq!(
        vitals[0]["direction_text"].as_str(),
        Some("could be better")
    );
    assert_eq!(
        vitals[0]["deviation_text"].as_str(),
        Some("1 h 10 m less than your usual")
    );

    assert_eq!(vitals[1]["value_text"].as_str(), Some("48 ms"));
    assert_eq!(
        vitals[1]["direction_text"].as_str(),
        Some("could be better")
    );
    assert_eq!(
        vitals[1]["deviation_text"].as_str(),
        Some("14 below your usual")
    );

    assert_eq!(vitals[2]["value_text"].as_str(), Some("63 bpm"));
    assert_eq!(vitals[2]["direction_text"].as_str(), Some("about usual"));
    assert_eq!(
        vitals[2]["deviation_text"].as_str(),
        Some("5 above your usual")
    );

    assert_eq!(
        body["rest"]["line"].as_str(),
        Some("could be better · 2 of 3 signals below your usual")
    );
}

#[test]
fn the_clock_is_injected_and_the_answer_says_when_it_was_produced() {
    let harness = common::Harness::empty("sentences-clock");
    let body = harness.ok_get("/physiology/vitals");
    assert_eq!(body["generated_at"].as_str(), Some(NOW));
}
