//! The catalogue and its verification copy.
//!
//! The canonical copy of the programmes lives inside the crate
//! (`crates/stride-core/src/plan_bases.json`); the exploration copy
//! (`exploration/plan-templates.json`) is the one a reader can check by
//! eye. This test reads the exploration copy with an independent parser and
//! holds every programme, week and day of it against the crate's own
//! catalogue: the data and the code cannot drift.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use serde_json::Value;
use stride_core::date::Weekday;
use stride_core::plan::SessionKind;
use stride_core::plan_bases::{programmes, Catalogue};
use stride_core::training_paces::METRES_PER_MILE;

/// The exploration copy: the file a reader can check by eye.
fn exploration_copy() -> Value {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "..",
        "exploration",
        "plan-templates.json",
    ]
    .iter()
    .collect();
    let text = std::fs::read_to_string(&path).expect("the exploration copy of the catalogue");
    serde_json::from_str(&text).expect("the exploration copy of the catalogue is JSON")
}

/// The crate's own catalogue.
fn catalogue() -> &'static Catalogue {
    stride_core::plan_bases::catalogue()
}

/// A number, in metres, from a number of miles.
fn miles(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(|miles| miles * METRES_PER_MILE)
}

#[test]
fn the_crates_catalogue_and_the_exploration_copy_are_the_same_programmes() {
    let copy = exploration_copy();
    let templates = copy
        .get("templates")
        .and_then(Value::as_array)
        .expect("the exploration copy's programmes");
    let crate_programmes = programmes();
    assert_eq!(
        crate_programmes.len(),
        templates.len(),
        "the catalogue and the exploration copy hold a different number of programmes"
    );

    for (programme, template) in crate_programmes.iter().zip(templates.iter()) {
        assert_eq!(programme.id, template["id"].as_str().unwrap());
        assert_eq!(programme.label, template["label"].as_str().unwrap());
        assert_eq!(programme.method, template["method"].as_str().unwrap());
        assert_eq!(
            programme.distance_m,
            template["distance_m"].as_f64().unwrap()
        );
        assert_eq!(
            programme.weeks,
            template["weeks"].as_f64().unwrap() as usize
        );

        // The entry requirement and the plan's own band.
        for (read, key) in [
            (programme.entry_mileage.as_ref(), "entry_mileage"),
            (programme.plan_mileage.as_ref(), "plan_mileage"),
        ] {
            let range = template.get(key).expect("a mileage band");
            match read {
                Some(read) => {
                    assert!((read.min_m - miles(range, "min_mi").unwrap()).abs() < 0.01);
                    assert!((read.max_m - miles(range, "max_mi").unwrap()).abs() < 0.01);
                }
                None => assert!(range.is_null(), "{key} is in the copy and not in the crate"),
            }
        }

        // The goal race and the distances it may be run at.
        let goal = template.get("goal_race").expect("the goal race");
        assert_eq!(programme.goal_race.label, goal["label"].as_str().unwrap());
        assert_eq!(
            programme.goal_race.distance_m,
            goal["distance_m"].as_f64().unwrap()
        );
        assert_eq!(
            programme.goal_race.distance_label,
            goal["distance_label"].as_str().unwrap()
        );
        let options = goal["options"].as_array().expect("the goal race's options");
        assert_eq!(programme.goal_race.options.len(), options.len());
        for (option, copy) in programme.goal_race.options.iter().zip(options.iter()) {
            assert_eq!(option.label, copy["label"].as_str().unwrap());
            assert_eq!(option.distance_m, copy["distance_m"].as_f64().unwrap());
        }

        // Where it was read from, and the notes beside it.
        assert_eq!(
            programme.source.spine_section,
            template["source"]["spine_section"].as_str().unwrap()
        );
        assert_eq!(
            programme.source.book_chapter,
            template["source"]["book_chapter"].as_str().unwrap()
        );
        let notes: Vec<&str> = template["notes"]
            .as_array()
            .expect("the programme's notes")
            .iter()
            .map(|note| note.as_str().unwrap())
            .collect();
        assert_eq!(programme.notes, notes);

        // The weeks, and every day of them.
        let weeks = template["weeks_detail"].as_array().expect("the weeks");
        assert_eq!(programme.weeks_detail.len(), weeks.len());
        for (week, copy) in programme.weeks_detail.iter().zip(weeks.iter()) {
            assert_eq!(week.week, copy["week"].as_f64().unwrap() as i32);
            assert_eq!(
                week.weeks_to_go,
                copy.get("weeks_to_go")
                    .and_then(Value::as_f64)
                    .map(|number| number as i32)
            );
            assert_eq!(week.label, copy["label"].as_str().unwrap());
            assert_eq!(week.in_plan, copy["in_plan"].as_bool().unwrap());
            assert_eq!(
                week.recovery_week,
                copy.get("recovery_week")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            );
            assert!(
                (week.printed_volume_m.unwrap_or(0.0)
                    - miles(copy, "printed_volume_mi").unwrap_or(0.0))
                .abs()
                    < 0.01
            );
            assert!(
                (week.volume_check.cells_m.unwrap_or(0.0)
                    - miles(&copy["volume_check"], "cells_mi").unwrap_or(0.0))
                .abs()
                    < 0.01
            );
            assert_eq!(
                week.volume_check.reconciles,
                copy["volume_check"]["reconciles"].as_bool()
            );
            assert_eq!(week.source, copy["source"].as_str().unwrap());

            let days = copy["days"].as_array().expect("the week's days");
            assert_eq!(week.days.len(), days.len());
            for (day, copy) in week.days.iter().zip(days.iter()) {
                assert_eq!(day.weekday, weekday_of(copy["day"].as_str().unwrap()));
                assert_eq!(
                    day.kind,
                    SessionKind::from_key(copy["kind"].as_str().unwrap()).unwrap()
                );
                assert_eq!(day.legend, copy["kind_label"].as_str().unwrap());
                assert!(
                    (day.distance_m.unwrap_or(0.0) - miles(copy, "distance_mi").unwrap_or(0.0))
                        .abs()
                        < 0.01
                );
                assert_eq!(
                    day.detail.as_deref(),
                    copy.get("detail").and_then(Value::as_str)
                );
                assert_eq!(
                    day.race_distance_m,
                    copy.get("race_distance_m").and_then(Value::as_f64)
                );
                assert_eq!(
                    day.race_distance_label.as_deref(),
                    copy.get("race_distance_label").and_then(Value::as_str)
                );
                assert!(
                    (day.lt_last_m.unwrap_or(0.0) - miles(copy, "lt_last_mi").unwrap_or(0.0)).abs()
                        < 0.01
                );
                assert_eq!(day.cell, copy["cell"].as_str().unwrap());
            }
        }
    }
}

#[test]
fn the_crates_catalogue_reads_the_same_schema_as_the_exploration_copy() {
    let copy = exploration_copy();
    assert_eq!(
        catalogue().schema,
        copy["schema"].as_str().unwrap(),
        "the catalogue and the exploration copy carry a different schema"
    );
}

#[test]
fn every_kind_the_catalogue_holds_is_one_the_core_names() {
    let copy = exploration_copy();
    let kinds = copy["kinds"].as_object().expect("the catalogue's kinds");
    for name in kinds.keys() {
        assert!(
            SessionKind::from_key(name).is_some(),
            "the catalogue holds a kind the core does not name: {name}"
        );
    }
    // And the core names no kind the catalogue does not hold.
    let held: Vec<&str> = kinds.keys().map(String::as_str).collect();
    for key in [
        "rest_cross",
        "ga",
        "ga_speed",
        "endurance",
        "progression_long_run",
        "lt",
        "vo2max",
        "speed_repetitions",
        "recovery",
        "race_pace",
        "race_pace_speed",
        "tune_up_race",
        "goal_race",
    ] {
        assert!(
            held.contains(&key),
            "the core names a kind the catalogue does not: {key}"
        );
    }
}

/// The weekday the catalogue's own key names.
fn weekday_of(text: &str) -> Weekday {
    match text {
        "mon" => Weekday::Monday,
        "tue" => Weekday::Tuesday,
        "wed" => Weekday::Wednesday,
        "thu" => Weekday::Thursday,
        "fri" => Weekday::Friday,
        "sat" => Weekday::Saturday,
        "sun" => Weekday::Sunday,
        other => panic!("the catalogue holds a day it does not name: {other}"),
    }
}
