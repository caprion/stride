//! The year: the months, the calendar of the days, and what changed.
//!
//! Every number in this answer is the core's, and every expectation is
//! worked out by hand here — the window's days, the months' totals, the
//! day's mix and the sentences of the diff.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{DateTime, Utc};
use serde_json::Value;
use stride_store::{NewActivity, NewHeartRateZoneModel};

use common::Harness;

/// The day the harness's clock reads: 2026-05-12.
const TODAY: &str = "2026-05-12";

/// Write a run whose FIT file is the corpus file named.
///
/// The file is the run's own source of truth: the time in the zones is
/// read from it on read.
fn write_fit_run(harness: &Harness, name: &str, local_date: &str) -> i64 {
    harness.write_run(NewActivity {
        source: "garmin_personal".to_string(),
        external_id: format!("fit-{name}-{local_date}"),
        name: "a run with heart rate".to_string(),
        sport_type: "running".to_string(),
        start_time: common::instant(),
        local_date: local_date.to_string(),
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

/// The athlete's own zone model, as the watch reported it.
fn zone_model(harness: &Harness) {
    harness
        .store
        .write_hr_zone_model_if_new(&NewHeartRateZoneModel {
            sport: "DEFAULT".to_string(),
            training_method: "HR_MAX".to_string(),
            max_heart_rate: Some(178.0),
            zone1_floor: 106.0,
            zone2_floor: 121.0,
            zone3_floor: 135.0,
            zone4_floor: 149.0,
            zone5_floor: 163.0,
            read_at: DateTime::parse_from_rfc3339("2026-05-10T06:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        })
        .unwrap();
}

/// The text of a field of the answer.
fn text(answer: &Value, path: &[&str]) -> String {
    let mut current = answer;
    for step in path {
        current = current
            .get(*step)
            .unwrap_or_else(|| panic!("the answer has no {step}: {answer}"));
    }
    current
        .as_str()
        .unwrap_or_else(|| panic!("the field is not a string: {current}"))
        .to_string()
}

/// The number of a field of the answer.
fn number(answer: &Value, path: &[&str]) -> f64 {
    let mut current = answer;
    for step in path {
        current = current
            .get(*step)
            .unwrap_or_else(|| panic!("the answer has no {step}: {answer}"));
    }
    current
        .as_f64()
        .unwrap_or_else(|| panic!("the field is not a number: {current}"))
}

/// The rows of a list of the answer.
fn rows<'a>(answer: &'a Value, name: &str) -> &'a Vec<Value> {
    answer
        .get(name)
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("the answer has no {name}"))
}

/// The row of a list with the given key.
fn row<'a>(rows: &'a [Value], key: &str) -> &'a Value {
    rows.iter()
        .find(|row| row.get("key").and_then(|value| value.as_str()) == Some(key))
        .unwrap_or_else(|| panic!("there is no row {key}"))
}

/// The row of the calendar with the given date.
fn day<'a>(rows: &'a [Value], date: &str) -> &'a Value {
    rows.iter()
        .find(|row| row.get("date").and_then(|value| value.as_str()) == Some(date))
        .unwrap_or_else(|| panic!("there is no day {date}"))
}

/// The year names its window, and the period it is compared against.
#[test]
fn the_year_names_the_window_and_the_period_before_it() {
    let harness = Harness::empty("year-window");
    let answer = harness.ok_get("/insights/year");

    // The last twelve calendar months, ending on the day the answer is
    // about.
    assert_eq!(text(&answer, &["window", "from"]), "2025-06-01");
    assert_eq!(text(&answer, &["window", "to"]), TODAY);
    assert_eq!(text(&answer, &["window", "from_label"]), "1 Jun 2025");
    assert_eq!(text(&answer, &["window", "to_label"]), "12 May 2026");
    assert_eq!(number(&answer, &["window", "days"]), 346.0);

    // The period before it: the same number of days, ending a day
    // earlier. The two never share a day.
    assert_eq!(text(&answer, &["comparison", "to"]), "2025-05-31");
    assert_eq!(number(&answer, &["comparison", "days"]), 346.0);
}

/// The months carry their own character.
#[test]
fn the_months_carry_their_distance_their_runs_and_their_longest_run() {
    let harness = Harness::empty("year-months");
    harness.write_run(common::run(
        "a-run-in-march",
        "2026-03-04",
        12_000.0,
        3_600.0,
    ));
    harness.write_run(common::run(
        "a-longer-run-in-march",
        "2026-03-08",
        21_000.0,
        6_000.0,
    ));
    harness.write_run(common::run("a-run-in-may", "2026-05-10", 8_000.0, 2_400.0));

    let answer = harness.ok_get("/insights/year");
    let months = rows(&answer, "months");
    // Twelve calendar months, oldest first.
    assert_eq!(months.len(), 12);
    assert_eq!(text(&months[0], &["key"]), "2025-06");
    assert_eq!(text(&months[11], &["key"]), "2026-05");

    // March: two runs, 33 km, and the longer one is the month's longest.
    let march = row(months, "2026-03");
    assert_eq!(number(march, &["sessions"]), 2.0);
    assert_eq!(text(march, &["distance", "text"]), "33 km");
    assert_eq!(text(march, &["longest", "text"]), "21 km");
    assert_eq!(text(march, &["label"]), "March 2026");

    // The shape of the weeks the month touches: the 1st of March 2026 is
    // a Saturday, so March holds six weeks.
    let weeks = march
        .get("weeks")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("the month has no weeks"));
    assert_eq!(weeks.len(), 6);
    assert_eq!(text(&weeks[0], &["start"]), "2026-02-23");
    assert_eq!(text(&weeks[0], &["start_label"]), "23 Feb 2026");
    assert_eq!(text(&weeks[1], &["distance", "text"]), "33 km");
    assert_eq!(number(&weeks[1], &["sessions"]), 2.0);
    // A week with nothing in it keeps its place and holds no distance:
    // the shape of the month keeps the gap.
    assert!(weeks[2].get("distance").unwrap().is_null());

    // May: one run of 8 km, and it is the month so far.
    let may = row(months, "2026-05");
    assert_eq!(number(may, &["sessions"]), 1.0);
    assert_eq!(text(may, &["distance", "text"]), "8 km");
    assert_eq!(text(may, &["longest", "text"]), "8 km");
}

/// A month with no run has no numbers at all.
#[test]
fn a_month_with_no_run_has_no_numbers() {
    let harness = Harness::empty("year-empty-month");
    harness.write_run(common::run("a-run-in-may", "2026-05-10", 8_000.0, 2_400.0));

    let answer = harness.ok_get("/insights/year");
    let months = rows(&answer, "months");
    let january = row(months, "2026-01");
    assert_eq!(number(january, &["sessions"]), 0.0);
    assert!(january.get("distance").unwrap().is_null());
    assert!(january.get("longest").unwrap().is_null());
    assert!(january.get("form").unwrap().is_null());
    // And no mix: there was no time in the zones.
    assert_eq!(
        january
            .get("mix")
            .and_then(|value| value.as_array())
            .map(|mix| mix.len()),
        Some(0)
    );
    assert!(january.get("high").unwrap().is_null());
}

/// The calendar is every day of the window, coloured by the zones.
#[test]
fn the_calendar_carries_every_day_of_the_window() {
    let harness = Harness::empty("year-calendar");
    zone_model(&harness);
    // A run whose file carries heart rate: the day has a mix.
    write_fit_run(&harness, "hr_gps_run.fit", "2026-05-10");
    // A run with no file: the day has a distance and no mix.
    harness.write_run(common::run(
        "a-run-without-a-file",
        "2026-05-08",
        5_000.0,
        1_500.0,
    ));

    let answer = harness.ok_get("/insights/year");
    let days = rows(&answer, "days");
    assert_eq!(days.len(), 346);
    assert_eq!(text(&days[0], &["date"]), "2025-06-01");
    assert_eq!(text(&days[345], &["date"]), TODAY);

    // The day with the run that has no file: a distance, and no mix at
    // all. That is no value, never a share of zero.
    let without = day(days, "2026-05-08");
    assert_eq!(number(without, &["sessions"]), 1.0);
    assert_eq!(text(without, &["distance", "text"]), "5 km");
    assert!(without.get("high").unwrap().is_null());

    // The day with the run whose file carries heart rate: the mix is
    // there, measured against the stored model.
    let with = day(days, "2026-05-10");
    assert_eq!(number(with, &["sessions"]), 1.0);
    assert_eq!(text(with, &["distance", "text"]), "10 km");
    let high = with
        .get("high")
        .unwrap_or_else(|| panic!("the day has no high share"));
    assert!(!high.is_null(), "the day has no mix: {with}");
    assert!(
        number(with, &["high", "raw"]) >= 0.0,
        "the high share is not a share"
    );

    // A day with no run: no distance at all.
    let empty = day(days, "2026-05-09");
    assert_eq!(number(empty, &["sessions"]), 0.0);
    assert!(empty.get("distance").unwrap().is_null());
    assert!(empty.get("high").unwrap().is_null());
}

/// The calendar says **when**, and never **where**.
#[test]
fn the_calendar_carries_no_position() {
    let harness = Harness::empty("year-privacy");
    zone_model(&harness);
    write_fit_run(&harness, "hr_gps_run.fit", "2026-05-10");

    let answer = harness.ok_get("/insights/year");
    let body = answer.to_string();
    for word in [
        "geo_point",
        "latitude",
        "longitude",
        "south",
        "north",
        "west",
        "east",
        "place",
        "address",
        "street",
    ] {
        assert!(!body.contains(word), "the year carries {word}");
    }
}

/// What changed: the diff against the athlete's own past, in sentences.
#[test]
fn what_changed_says_the_facts_about_the_two_periods() {
    let harness = Harness::empty("year-changed");
    // The window: 30 km in two runs.
    harness.write_run(common::run(
        "a-run-in-april",
        "2026-04-04",
        10_000.0,
        3_000.0,
    ));
    harness.write_run(common::run(
        "a-longer-run-in-april",
        "2026-04-08",
        20_000.0,
        6_000.0,
    ));
    // The period before it: 5 km in one run, in March of last year.
    harness.write_run(common::run(
        "a-run-in-march-last-year",
        "2025-03-08",
        5_000.0,
        1_500.0,
    ));

    let answer = harness.ok_get("/insights/year");
    let changed = rows(&answer, "changed");
    let lines: Vec<String> = changed
        .iter()
        .map(|line| line.as_str().unwrap_or_default().to_string())
        .collect();

    // The distance, the runs and the longest run: the numbers of the two
    // periods, side by side.
    assert_eq!(
        lines[0],
        "+25 km against the period before (30 km against 5 km)"
    );
    assert_eq!(lines[1], "+1 recorded sessions (2 against 1)");
    assert_eq!(
        lines[2],
        "the longest run grew by 15 km (20 km against 5 km)"
    );
    // The pace: the window's moving time over its distance.
    assert_eq!(lines[3], "the average pace was 5:00 /km against 5:00 /km");

    // And nothing in the diff is a verdict or an instruction.
    for line in lines {
        let text = line.to_lowercase();
        for word in [
            "should", "must", "good", "bad", "better", "worse", "poor", "missed",
        ] {
            assert!(!text.contains(word), "\"{line}\" carries \"{word}\"");
        }
    }
}

/// A store with nothing in it answers with the honest year.
#[test]
fn a_store_with_nothing_in_it_has_an_empty_year() {
    let harness = Harness::empty("year-nothing");
    let answer = harness.ok_get("/insights/year");

    // The months are the twelve calendar months, and none of them holds
    // anything.
    let months = rows(&answer, "months");
    assert_eq!(months.len(), 12);
    for month in months {
        assert_eq!(number(month, &["sessions"]), 0.0);
        assert!(month.get("distance").unwrap().is_null());
    }

    // The calendar is still every day of the window, and every day has no
    // distance.
    let days = rows(&answer, "days");
    assert_eq!(days.len(), 346);
    assert!(days
        .iter()
        .all(|day| day.get("distance").unwrap().is_null()));

    // The diff says the distance and the runs, and has no pace and no
    // form to say.
    let changed = rows(&answer, "changed");
    assert_eq!(changed.len(), 3);
}
