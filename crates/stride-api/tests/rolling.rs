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
use stride_api::Config;
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

/// A zone model, read on the day named.
///
/// The model in force on the day of a run is the one that was read **on
/// or before** it: the configuration the athlete was wearing when they
/// ran.
fn model_read_on(day: &str, floors: [f64; 5]) -> NewHeartRateZoneModel {
    NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(178.0),
        zone1_floor: floors[0],
        zone2_floor: floors[1],
        zone3_floor: floors[2],
        zone4_floor: floors[3],
        zone5_floor: floors[4],
        read_at: DateTime::parse_from_rfc3339(&format!("{day}T06:30:00Z"))
            .unwrap()
            .with_timezone(&Utc),
    }
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
fn the_window_is_the_last_calendar_weeks_through_today() {
    let harness = Harness::empty("rolling-window");
    // A run today, and one a week and a bit ago: inside the window
    // before it, outside the window.
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    harness.write_run(common::run("last-week", "2026-05-05", 5_000.0, 1_500.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");

    // One calendar week through today: the Monday of this week — the
    // 11th — and the 12th of May. The window's length is its own.
    assert_eq!(text(&answer, &["window", "from"]), "2026-05-11");
    assert_eq!(text(&answer, &["window", "to"]), "2026-05-12");
    assert_eq!(number(&answer, &["window", "days"]), 2.0);
    // The comparison is **the same span, seven days earlier**: the 4th to
    // the 5th of May.
    assert_eq!(text(&answer, &["comparison", "from"]), "2026-05-04");
    assert_eq!(text(&answer, &["comparison", "to"]), "2026-05-05");

    // The window holds the run of today: 10 km in 50 minutes.
    assert_eq!(text(&answer, &["distance", "current", "text"]), "10 km");
    assert_eq!(text(&answer, &["duration", "current", "text"]), "50 m");
    assert_eq!(number(&answer, &["sessions", "current"]), 1.0);
    // The period it is compared against holds the run of the 5th: 5 km.
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
    // Twelve calendar weeks: the Monday of the earliest of them — the
    // 23rd of February — through the 12th of May. The window is
    // seventy-nine days long, not eighty-four: it ends on the day the
    // answer is about, and the current week is the partial one.
    assert_eq!(text(&answer, &["window", "from"]), "2026-02-23");
    assert_eq!(number(&answer, &["window", "days"]), 79.0);
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
        "not measured — your watch's zones were not read, and the watch's own zone minutes are the only zone time here"
    );
}

#[test]
fn the_split_names_the_model_it_was_measured_against() {
    let harness = Harness::empty("rolling-model");
    zone_model(&harness);
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    let answer = harness.ok_get("/insights/rolling?weeks=1");
    // The model is named, with the day it was read, and the source line
    // is **hoisted**: one line for the screen.
    let models = answer["zones"]["models"].as_array().expect("the models");
    assert_eq!(models.len(), 1);
    assert_eq!(text(&models[0], &["read_on"]), "2026-05-10");
    assert_eq!(text(&models[0], &["read_on_label"]), "10 May 2026");
    assert!(text(&models[0], &["source"]).contains("your zones"));
    assert!(text(&answer, &["zones", "source"]).contains("your zones"));
    // The runs carry no heart-rate samples, so the split has no value —
    // and the reason is the core's own words.
    assert!(answer["zones"]["shares"].is_null());
    assert_eq!(
        text(&answer, &["zones", "unavailable"]),
        "No heart rate recorded — none of the runs in this period recorded a heart rate"
    );
}

/// **The rule of stage 1**: a run is measured against the model that was
/// in force on the day of the run — the last one read on or before it.
#[test]
fn each_run_is_measured_against_the_model_that_was_in_force_on_its_day() {
    let harness = Harness::empty("rolling-model-in-force");
    // Two models, read on two days.
    harness
        .store
        .write_hr_zone_model_if_new(&model_read_on("2026-05-01", FLOORS))
        .unwrap();
    harness
        .store
        .write_hr_zone_model_if_new(&model_read_on(
            "2026-05-08",
            [110.0, 125.0, 140.0, 155.0, 170.0],
        ))
        .unwrap();
    // Two runs: one before the second model was read, and one after.
    harness.write_run(common::run("before", "2026-05-05", 10_000.0, 3_000.0));
    harness.write_run(common::run("after", "2026-05-10", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/rolling?weeks=4");

    // Both models are named, oldest first, with the days they were read.
    let models = answer["zones"]["models"].as_array().expect("the models");
    assert_eq!(models.len(), 2);
    assert_eq!(text(&models[0], &["read_on"]), "2026-05-01");
    assert_eq!(text(&models[1], &["read_on"]), "2026-05-08");
    // And the source line is **hoisted**: one line for the screen, naming
    // the models the window was measured against.
    let source = text(&answer, &["zones", "source"]);
    assert!(source.contains("2 readings"), "the source line: {source}");
    assert!(source.contains("1 May 2026"), "the source line: {source}");
    assert!(source.contains("8 May 2026"), "the source line: {source}");
}

/// A run that predates the first stored model is **not calibrated**: it has
/// no model in force, and the answer says so rather than borrowing a
/// later one.
#[test]
fn a_run_that_predates_the_first_model_has_no_model_in_force() {
    let harness = Harness::empty("rolling-model-too-late");
    harness
        .store
        .write_hr_zone_model_if_new(&model_read_on("2026-05-08", FLOORS))
        .unwrap();
    // The only run in the window is before the model was read.
    harness.write_run(common::run("before", "2026-05-05", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/rolling?weeks=4");

    // No model was in force for it, so the split has no value — and the
    // reason says which one it is.
    assert!(answer["zones"]["shares"].is_null());
    assert!(answer["zones"]["models"]
        .as_array()
        .expect("the models")
        .is_empty());
    assert_eq!(
        text(&answer, &["zones", "unavailable"]),
        "not measured — your watch's zones were not read for these runs; the watch's own zone \
         minutes are the only zone time here"
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

    let answer = harness.ok_get("/insights/rolling?weeks=2");
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
fn the_window_may_be_asked_for_in_days() {
    let harness = Harness::empty("rolling-days");
    harness.write_run(common::run("today", TODAY, 10_000.0, 3_000.0));
    // Thirty days: the 13th of April to the 12th of May. A month is not
    // four weeks, and the router answers the days exactly.
    let answer = harness.ok_get("/insights/rolling?days=30");
    assert_eq!(text(&answer, &["window", "from"]), "2026-04-13");
    assert_eq!(text(&answer, &["window", "to"]), "2026-05-12");
    assert_eq!(number(&answer, &["window", "days"]), 30.0);
    // The comparison is the same span, seven days earlier.
    assert_eq!(text(&answer, &["comparison", "from"]), "2026-04-06");
    assert_eq!(number(&answer, &["comparison", "days"]), 30.0);
    // And the volume is a distance a week: ten kilometres in thirty days
    // is 2.3 kilometres a week.
    let profile = answer["profile"].as_array().expect("the facets");
    assert_eq!(text(&profile[0], &["value", "text"]), "2.3 km a week");
    // The total is the window's own: the ten kilometres.
    assert_eq!(text(&profile[0], &["total", "text"]), "10 km");
}

#[test]
fn the_profile_is_seven_values_and_their_movement() {
    let harness = Harness::empty("rolling-profile");
    harness.write_run(common::run("today", TODAY, 20_000.0, 3_000.0));
    // A run in the period the window is compared against: half the
    // distance. The comparison is the same span, seven days earlier — the
    // 4th to the 5th of May — so the run belongs on the 4th.
    harness.write_run(common::run("before", "2026-05-04", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let profile = answer["profile"].as_array().expect("the facets");
    assert_eq!(profile.len(), 7);

    // The volume: the run of today is in the window (20 km), and the
    // earlier one is in the period it is compared against (10 km) — up.
    let volume = &profile[0];
    assert_eq!(text(volume, &["key"]), "volume");
    assert_eq!(text(volume, &["label"]), "Volume");
    // The total is the window's own distance.
    assert_eq!(text(volume, &["total", "text"]), "20 km");
    // And the comparable value is the distance a week, measured against
    // the window's own length. The window is two days long here — Monday
    // and Tuesday — so twenty kilometres over two days is seventy
    // kilometres a week.
    assert_eq!(text(volume, &["value", "text"]), "70 km a week");
    assert_eq!(text(volume, &["previous", "text"]), "35 km a week");
    assert_eq!(text(volume, &["movement"]), "up");

    // The consistency: one day carries a run in each window, and the
    // window is two days long — half of its days.
    assert_eq!(text(&profile[1], &["key"]), "consistency");
    assert_eq!(text(&profile[1], &["total", "text"]), "1");
    assert_eq!(text(&profile[1], &["value", "text"]), "50.0% of the days");
    // The intensity: the median heart rate of the runs.
    assert_eq!(text(&profile[2], &["key"]), "intensity");
    assert_eq!(text(&profile[2], &["value", "text"]), "148 bpm");
    assert_eq!(text(&profile[2], &["total", "text"]), "148 bpm");
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
    // 05:10 and 07:10 are the morning, 13:00 the afternoon, 18:00 the
    // evening and 23:30 the night.
    harness.write_run(run_at("early", TODAY, 5, 10, 8_000.0, 2_400.0));
    harness.write_run(run_at("morning", TODAY, 7, 10, 10_000.0, 3_000.0));
    harness.write_run(run_at("afternoon", TODAY, 13, 0, 5_000.0, 1_500.0));
    harness.write_run(run_at("evening", TODAY, 18, 0, 5_000.0, 1_500.0));
    harness.write_run(run_at("night", TODAY, 23, 30, 5_000.0, 1_500.0));

    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let daytime = answer["daytime"].as_array().expect("the daytime rows");
    assert_eq!(daytime.len(), 4);
    // The label carries the window the bucket covers, and the window is
    // the partition's own.
    assert_eq!(text(&daytime[0], &["label"]), "Morning (04:00–12:00)");
    assert_eq!(number(&daytime[0], &["from_hour"]), 4.0);
    assert_eq!(number(&daytime[0], &["to_hour"]), 12.0);
    // The run at five in the morning is a morning run.
    assert_eq!(number(&daytime[0], &["sessions"]), 2.0);
    assert_eq!(text(&daytime[1], &["label"]), "Afternoon (12:00–17:00)");
    assert_eq!(number(&daytime[1], &["sessions"]), 1.0);
    assert_eq!(text(&daytime[2], &["label"]), "Evening (17:00–21:00)");
    assert_eq!(number(&daytime[2], &["sessions"]), 1.0);
    assert_eq!(text(&daytime[3], &["label"]), "Night (21:00–04:00)");
    assert_eq!(number(&daytime[3], &["from_hour"]), 21.0);
    assert_eq!(number(&daytime[3], &["to_hour"]), 4.0);
    assert_eq!(number(&daytime[3], &["sessions"]), 1.0);
}

#[test]
fn the_part_of_a_day_is_read_in_the_athletes_own_hours() {
    // The run starts at 23:30 UTC, which is **05:00 in a +5:30 zone** —
    // the athlete's own morning, on the day Garmin says it belongs to.
    //
    // The hours are read in the athlete's own zone and the day comes from
    // Garmin: a rig that reads its hours in UTC would call this run a
    // night run, and every early run in the athlete's history would be
    // reported as a night one.
    let run = |harness: &Harness| {
        let mut activity = common::run("early", "2026-05-12", 8_000.0, 2_400.0);
        activity.start_time = Utc.with_ymd_and_hms(2026, 5, 11, 23, 30, 0).unwrap();
        harness.write_run(activity);
    };

    // The athlete's own zone: +5:30.
    let harness = Harness::with_config(
        "rolling-offset-local",
        Config {
            utc_offset_minutes: 330,
            ..Config::default()
        },
    );
    run(&harness);
    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let daytime = answer["daytime"].as_array().expect("the daytime rows");
    assert_eq!(text(&daytime[0], &["label"]), "Morning (04:00–12:00)");
    assert_eq!(number(&daytime[0], &["sessions"]), 1.0);
    assert_eq!(number(&daytime[3], &["sessions"]), 0.0);

    // The same instant, read in UTC: the run is at half past eleven at
    // night, and it is a night run.
    let harness = Harness::empty("rolling-offset-utc");
    run(&harness);
    let answer = harness.ok_get("/insights/rolling?weeks=1");
    let daytime = answer["daytime"].as_array().expect("the daytime rows");
    assert_eq!(number(&daytime[0], &["sessions"]), 0.0);
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
fn the_streaks_answer_the_months_too() {
    let harness = Harness::empty("streaks-months");
    // One run in each of three consecutive months: March, April and May.
    harness.write_run(common::run("march", "2026-03-14", 10_000.0, 3_000.0));
    harness.write_run(common::run("april", "2026-04-12", 10_000.0, 3_000.0));
    harness.write_run(common::run("may", "2026-05-10", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/streaks");
    // The current streak of months: three, from March.
    assert_eq!(number(&answer, &["months", "current", "count"]), 3.0);
    assert_eq!(text(&answer, &["months", "current", "start"]), "2026-03");
    assert_eq!(
        text(&answer, &["months", "current", "start_label"]),
        "March 2026"
    );
    // And the longest one is the same run of months.
    assert_eq!(number(&answer, &["months", "longest", "count"]), 3.0);
    assert_eq!(text(&answer, &["months", "longest", "start"]), "2026-03");
    assert_eq!(text(&answer, &["months", "longest", "end"]), "2026-05");
}

#[test]
fn a_month_with_no_run_breaks_the_month_streak() {
    let harness = Harness::empty("streaks-months-gap");
    // March and May: April has no run in it.
    harness.write_run(common::run("march", "2026-03-14", 10_000.0, 3_000.0));
    harness.write_run(common::run("may", "2026-05-10", 10_000.0, 3_000.0));

    let answer = harness.ok_get("/insights/streaks");
    assert_eq!(number(&answer, &["months", "current", "count"]), 1.0);
    assert_eq!(text(&answer, &["months", "current", "start"]), "2026-05");
    assert_eq!(number(&answer, &["months", "longest", "count"]), 1.0);
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
    assert!(answer["months"]["current"].is_null());
    assert!(answer["months"]["longest"].is_null());
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
    // And a window of a hundred and four weeks is fine: it is the last
    // hundred and four calendar weeks, which through a Tuesday is
    // seven hundred and twenty-three days.
    let answer = harness
        .router
        .call("GET", "/insights/rolling?weeks=104", None);
    assert!(answer.is_ok());
    assert_eq!(number(&answer.body, &["window", "days"]), 723.0);
}
