// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The rolling view: the window, the splits and the streaks.
//!
//! Every number in these answers is the core's, and every expectation is
//! worked out by hand here — the window's days, the totals, the difference
//! and the streak's counts.

mod common;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use stride_store::{NewActivity, NewHeartRateZoneModel};

use common::Harness;

/// The floors of the synthetic model below, in beats per minute.
///
/// The repository is public: no athlete's real floors belong in a test.
const FLOORS: [f64; 5] = [106.0, 121.0, 135.0, 149.0, 163.0];

/// Write a run whose FIT file is the corpus file named.
fn write_fit_run(harness: &Harness, name: &str, local_date: &str) -> i64 {
    let fit = common::corpus_fit(name);
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
        fit_blob: fit,
        created_at: common::instant(),
        updated_at: common::instant(),
    })
}

/// The day the harness's clock reads: 2026-05-12.
const TODAY: &str = "2026-05-12";

/// A run on a day, at a given hour of the athlete's own local time.
fn run_at(
    external_id: &str,
    local_date: &str,
    hour: u32,
    minute: u32,
    distance_m: f64,
    moving_time_s: f64,
) -> NewActivity {
    let mut activity = common::run(external_id, local_date, distance_m, moving_time_s);
    activity.start_time = Utc
        .with_ymd_and_hms(
            local_date[..4].parse().unwrap_or(2026),
            local_date[5..7].parse().unwrap_or(5),
            local_date[8..10].parse().unwrap_or(12),
            hour,
            minute,
            0,
        )
        .unwrap();
    activity
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

/// The rolling view over a window of days.
#[test]
fn the_window_is_the_last_days_ending_today() {
    let harness = Harness::empty("rolling-window");
    // A run today, and one a week and a bit ago: inside the window
    // before it, outside the window.
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    harness.write_run(common::run("last-week", "2026-05-05", 5_000.0, 1_500.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");

    // Seven days ending today: the 6th to the 12th of May.
    assert_eq!(text(&answer, &["window", "from"]), "2026-05-06");
    assert_eq!(text(&answer, &["window", "to"]), "2026-05-12");
    assert_eq!(number(&answer, &["window", "days"]), 7.0);
    // The comparison is the seven days **before** the window: the 29th of
    // April to the 5th of May. The two windows share no day.
    assert_eq!(text(&answer, &["comparison", "from"]), "2026-04-29");
    assert_eq!(text(&answer, &["comparison", "to"]), "2026-05-05");

    // The window holds the run of today: 10 km in 50 minutes.
    assert_eq!(text(&answer, &["distance", "current", "text"]), "10 km");
    assert_eq!(text(&answer, &["duration", "current", "text"]), "50 m");
    assert_eq!(number(&answer, &["sessions", "current"]), 1.0);
    // The window before it holds the run of the 5th: 5 km.
    assert_eq!(text(&answer, &["distance", "previous", "text"]), "5 km");
    // And the difference between them: five kilometres more.
    assert_eq!(text(&answer, &["distance", "difference", "text"]), "+5 km");
    // One run in each window: the same count. The difference is the
    // core's rendering of it, sign and all.
    assert_eq!(text(&answer, &["sessions", "difference", "text"]), "0");
}

#[test]
fn the_window_follows_the_range_it_is_asked_for() {
    let harness = Harness::empty("rolling-weeks");
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    let answer = harness.ok_get("/insights/rolling?weeks=12");
    // Twelve weeks are eighty-four days: the 18th of February to the
    // 12th of May.
    assert_eq!(text(&answer, &["window", "from"]), "2026-02-18");
    assert_eq!(number(&answer, &["window", "days"]), 84.0);
}

#[test]
fn a_window_with_no_runs_holds_no_distance_and_no_shares() {
    let harness = Harness::empty("rolling-empty");
    let answer = harness.ok_get("/insights/rolling?weeks=4");
    // The window's totals are facts about the window: there was nothing
    // in it.
    assert_eq!(text(&answer, &["distance", "current", "text"]), "0 km");
    assert_eq!(number(&answer, &["sessions", "current"]), 0.0);
    // The split has no value at all: no model has been read.
    assert!(answer["zones"]["shares"].is_null());
    assert_eq!(
        text(&answer, &["zones", "unavailable"]),
        "zones unavailable — no zone model has been read"
    );
}

#[test]
fn the_split_names_the_model_it_was_measured_against() {
    let harness = Harness::empty("rolling-model");
    zone_model(&harness);
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    let answer = harness.ok_get("/insights/rolling?weeks=1");
    // The model is named, with the day it was read.
    assert_eq!(text(&answer, &["zones", "model", "read_on"]), "2026-05-10");
    assert_eq!(
        text(&answer, &["zones", "model", "read_on_label"]),
        "10 May 2026"
    );
    assert!(text(&answer, &["zones", "model", "source"]).contains("your zone model"));
    // The runs carry no heart-rate samples, so the split has no value —
    // and the reason is the core's own words.
    assert!(answer["zones"]["shares"].is_null());
    assert_eq!(
        text(&answer, &["zones", "unavailable"]),
        "the split has no value — the runs in the window have no heart-rate samples"
    );
}

#[test]
fn the_zone_distribution_measures_the_time_in_each_zone() {
    let harness = Harness::empty("rolling-distribution");
    // A run with heart-rate samples, and the athlete's stored model.
    write_fit_run(&harness, "hr_gps_run.fit", TODAY);
    harness
        .store
        .write_hr_zone_model_if_new(&NewHeartRateZoneModel {
            sport: "DEFAULT".to_string(),
            training_method: "HR_MAX".to_string(),
            max_heart_rate: Some(178.0),
            zone1_floor: FLOORS[0],
            zone2_floor: FLOORS[1],
            zone3_floor: FLOORS[2],
            zone4_floor: FLOORS[3],
            zone5_floor: FLOORS[4],
            read_at: DateTime::parse_from_rfc3339("2026-05-10T06:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        })
        .unwrap();

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let distribution = answer["zones"]["distribution"]
        .as_array()
        .expect("the distribution of the zones");

    // Five zones, each with the time in it and its share.
    assert_eq!(distribution.len(), 5);
    assert_eq!(text(&distribution[0], &["zone"]), "Z1");
    assert_eq!(text(&distribution[0], &["range"]), "106-121 bpm");
    // The last zone ends at the maximum heart rate the model was built on.
    assert_eq!(text(&distribution[4], &["range"]), "163-178 bpm");
    // The times are the core's own.
    assert!(number(&distribution[0], &["time", "raw"]) >= 0.0);
    // And the shares add up to the time in the zones.
    let total: f64 = distribution
        .iter()
        .map(|zone| number(zone, &["share", "raw"]))
        .sum();
    assert!((total - 100.0).abs() < 0.05, "the shares add up to {total}");
    // The bands are there as well.
    let shares = answer["zones"]["shares"].as_array().expect("the bands");
    assert_eq!(shares.len(), 3);
}

#[test]
fn the_distance_breakdown_buckets_the_runs() {
    let harness = Harness::empty("rolling-breakdown");
    // Three runs: 3 km, 7 km and 21 km. The longest one sets the scale.
    harness.write_run(common::run("short", "2026-05-10", 3_000.0, 900.0));
    harness.write_run(common::run("middle", "2026-05-11", 7_000.0, 2_100.0));
    harness.write_run(common::run("long", TODAY, 21_000.0, 6_300.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let buckets = answer["distances"].as_array().expect("the buckets");
    // The longest run is 21 km: the buckets are 5 km wide, five of them,
    // and the longest run lands in the last one.
    assert_eq!(buckets.len(), 5);
    assert_eq!(text(&buckets[0], &["label"]), "0-5 km");
    assert_eq!(text(&buckets[4], &["label"]), "20-25 km");
    // The run of 3 km is in the first bucket, the run of 7 km in the
    // second, the run of 21 km in the last.
    assert_eq!(number(&buckets[0], &["sessions"]), 1.0);
    assert_eq!(text(&buckets[0], &["distance", "text"]), "3 km");
    assert_eq!(number(&buckets[1], &["sessions"]), 1.0);
    assert_eq!(text(&buckets[1], &["distance", "text"]), "7 km");
    assert_eq!(number(&buckets[4], &["sessions"]), 1.0);
    assert_eq!(text(&buckets[4], &["distance", "text"]), "21 km");
}

#[test]
fn the_profile_is_seven_values_and_their_movement() {
    let harness = Harness::empty("rolling-profile");
    harness.write_run(common::run("today", TODAY, 20_000.0, 3_000.0));
    // A run in the window **before** it: half the distance. The window
    // before the window is the 29th of April to the 5th of May, so the run
    // belongs on the 4th.
    harness.write_run(common::run("before", "2026-05-04", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let profile = answer["profile"].as_array().expect("the facets");
    assert_eq!(profile.len(), 7);

    // The volume: the run of today is in the window (20 km), and the
    // earlier one is in the window before it (10 km) — up.
    let volume = &profile[0];
    assert_eq!(text(volume, &["key"]), "volume");
    assert_eq!(text(volume, &["label"]), "Volume");
    assert_eq!(text(volume, &["value", "text"]), "20 km");
    assert_eq!(text(volume, &["previous", "text"]), "10 km");
    assert_eq!(text(volume, &["movement"]), "up");

    // The consistency: one day carries a run in each window.
    assert_eq!(text(&profile[1], &["key"]), "consistency");
    assert_eq!(text(&profile[1], &["value", "text"]), "1");
    // The intensity: the median heart rate of the runs.
    assert_eq!(text(&profile[2], &["key"]), "intensity");
    assert_eq!(text(&profile[2], &["value", "text"]), "148 bpm");
    // And the rest of the facets are there, with their names.
    let keys: Vec<String> = profile.iter().map(|facet| text(facet, &["key"])).collect();
    assert_eq!(
        keys,
        vec![
            "volume",
            "consistency",
            "intensity",
            "duration",
            "density",
            "variety",
            "week_days"
        ]
    );
}

#[test]
fn a_window_with_no_run_has_no_profile() {
    let harness = Harness::empty("rolling-profile-empty");
    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let profile = answer["profile"].as_array().expect("the facets");
    // The facets are there, and every one of them has no value: a
    // window with nothing in it has no shape.
    assert_eq!(profile.len(), 7);
    assert!(profile.iter().all(|facet| facet["value"].is_null()));
    assert!(profile.iter().all(|facet| facet["movement"].is_null()));
}

#[test]
fn the_weekday_split_places_the_runs_on_their_day() {
    let harness = Harness::empty("rolling-weekday");
    // The 11th of May 2026 is a Monday, the 12th a Tuesday.
    harness.write_run(common::run("monday", "2026-05-11", 10_000.0, 3_000.0));
    harness.write_run(common::run("tuesday", TODAY, 5_000.0, 1_500.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let weekday = answer["weekday"].as_array().expect("the weekday rows");
    assert_eq!(weekday.len(), 7);
    assert_eq!(text(&weekday[0], &["weekday"]), "Monday");
    assert_eq!(number(&weekday[0], &["sessions"]), 1.0);
    assert_eq!(text(&weekday[0], &["distance", "text"]), "10 km");
    assert_eq!(text(&weekday[1], &["weekday"]), "Tuesday");
    assert_eq!(number(&weekday[1], &["sessions"]), 1.0);
    // Tuesday has half the moving time of the week.
    assert_eq!(text(&weekday[1], &["share", "text"]), "33.33%");
    // Wednesday and the rest of the week: nothing.
    assert_eq!(number(&weekday[2], &["sessions"]), 0.0);
}

#[test]
fn the_daytime_split_uses_the_local_hour() {
    let harness = Harness::empty("rolling-daytime");
    // 07:10 is the morning, 13:00 the afternoon, 18:00 the evening and
    // 23:30 the night.
    harness.write_run(run_at("morning", TODAY, 7, 10, 10_000.0, 3_000.0));
    harness.write_run(run_at("afternoon", TODAY, 13, 0, 5_000.0, 1_500.0));
    harness.write_run(run_at("evening", TODAY, 18, 0, 5_000.0, 1_500.0));
    harness.write_run(run_at("night", TODAY, 23, 30, 5_000.0, 1_500.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let daytime = answer["daytime"].as_array().expect("the daytime rows");
    assert_eq!(daytime.len(), 4);
    assert_eq!(text(&daytime[0], &["label"]), "Morning");
    assert_eq!(number(&daytime[0], &["from_hour"]), 6.0);
    assert_eq!(number(&daytime[0], &["to_hour"]), 12.0);
    assert_eq!(number(&daytime[0], &["sessions"]), 1.0);
    assert_eq!(text(&daytime[1], &["label"]), "Afternoon");
    assert_eq!(number(&daytime[1], &["sessions"]), 1.0);
    assert_eq!(text(&daytime[2], &["label"]), "Evening");
    assert_eq!(number(&daytime[2], &["sessions"]), 1.0);
    assert_eq!(text(&daytime[3], &["label"]), "Night");
    assert_eq!(number(&daytime[3], &["from_hour"]), 23.0);
    assert_eq!(number(&daytime[3], &["to_hour"]), 6.0);
    assert_eq!(number(&daytime[3], &["sessions"]), 1.0);
}

#[test]
fn the_monthly_and_yearly_tables_hold_the_periods_that_have_runs() {
    let harness = Harness::empty("rolling-periods");
    harness.write_run(common::run("march", "2026-03-02", 10_000.0, 3_000.0));
    harness.write_run(common::run("april", "2026-04-02", 12_000.0, 3_600.0));
    harness.write_run(common::run("april-again", "2026-04-04", 8_000.0, 2_400.0));

    let answer = harness.ok_get("/insights/rolling?weeks=12");
    let months = answer["months"].as_array().expect("the months");
    // Two months: April and March. February is not there at all.
    assert_eq!(months.len(), 2);
    assert_eq!(text(&months[0], &["key"]), "2026-04");
    assert_eq!(text(&months[0], &["label"]), "April 2026");
    assert_eq!(number(&months[0], &["sessions"]), 2.0);
    assert_eq!(text(&months[0], &["distance", "text"]), "20 km");
    assert_eq!(text(&months[1], &["key"]), "2026-03");

    let years = answer["years"].as_array().expect("the years");
    assert_eq!(years.len(), 1);
    assert_eq!(text(&years[0], &["key"]), "2026");
    assert_eq!(number(&years[0], &["sessions"]), 3.0);
}

#[test]
fn the_streaks_answer_the_current_and_the_longest() {
    let harness = Harness::empty("streaks");
    // Three consecutive days: the 10th, the 11th and the 12th of May.
    harness.write_run(common::run("first", "2026-05-10", 10_000.0, 3_000.0));
    harness.write_run(common::run("second", "2026-05-11", 10_000.0, 3_000.0));
    harness.write_run(common::run("third", TODAY, 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/streaks");
    assert_eq!(text(&answer, &["date"]), TODAY);
    // The current streak of days: three, from the 10th of May.
    assert_eq!(number(&answer, &["days", "current", "count"]), 3.0);
    assert_eq!(text(&answer, &["days", "current", "start"]), "2026-05-10");
    assert_eq!(
        text(&answer, &["days", "current", "start_label"]),
        "10 May 2026"
    );
    // The longest one is the same run of days.
    assert_eq!(number(&answer, &["days", "longest", "count"]), 3.0);
    assert_eq!(text(&answer, &["days", "longest", "start"]), "2026-05-10");
    assert_eq!(text(&answer, &["days", "longest", "end"]), "2026-05-12");
    // The weeks: the 10th is in the week of the 4th, the 11th and the
    // 12th in the week of the 11th. Two weeks in a row.
    assert_eq!(number(&answer, &["weeks", "current", "count"]), 2.0);
    assert_eq!(text(&answer, &["weeks", "current", "start"]), "2026-05-10");
}

#[test]
fn a_streak_that_ended_is_not_a_current_streak() {
    let harness = Harness::empty("streaks-old");
    // The last run was on the 1st of May: eleven days ago.
    harness.write_run(common::run("old", "2026-05-01", 10_000.0, 3_000.0));
    harness.write_run(common::run("older", "2026-04-30", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/streaks");
    // No current streak: a streak of zero days is no streak.
    assert!(answer["days"]["current"].is_null());
    // The longest one is still a fact: two days, on the 30th and the 1st.
    assert_eq!(number(&answer, &["days", "longest", "count"]), 2.0);
    assert_eq!(text(&answer, &["days", "longest", "start"]), "2026-04-30");
    assert_eq!(text(&answer, &["days", "longest", "end"]), "2026-05-01");
}

#[test]
fn a_history_with_no_runs_has_no_streak() {
    let harness = Harness::empty("streaks-empty");
    let answer = harness.ok_get("/insights/streaks");
    assert!(answer["days"]["current"].is_null());
    assert!(answer["days"]["longest"].is_null());
    assert!(answer["weeks"]["current"].is_null());
    assert!(answer["weeks"]["longest"].is_null());
}

#[test]
fn the_window_is_refused_outside_its_bound() {
    let harness = Harness::empty("rolling-bound");
    let answer = harness
        .router
        .call("GET", "/insights/rolling?weeks=0", None);
    assert!(!answer.is_ok());
    let answer = harness
        .router
        .call("GET", "/insights/rolling?weeks=105", None);
    assert!(!answer.is_ok());
    // And a window of a hundred and four weeks is fine.
    let answer = harness
        .router
        .call("GET", "/insights/rolling?weeks=104", None);
    assert!(answer.is_ok());
    assert_eq!(number(&answer.body, &["window", "days"]), 728.0);
}
