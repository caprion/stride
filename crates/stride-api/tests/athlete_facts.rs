// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The athlete's own facts: the races they typed, and the threshold they
//! chose.
//!
//! **These facts are never synced.** They are entered in the app and
//! stored locally, and every write carries the value, the day it was
//! entered, the day it last changed and the source.

mod common;

use common::Harness;
use serde_json::Value;

/// The answer of a write, with its status.
fn post(harness: &Harness, path: &str, body: &str) -> (u16, Value) {
    let answer = harness.router.call("POST", path, Some(body));
    (answer.status, answer.body)
}

#[test]
fn a_race_is_written_with_the_day_it_was_entered_and_its_source() {
    let harness = Harness::empty("athlete-races");
    let (status, body) = post(
        &harness,
        "/athlete/races",
        r#"{"label": "10K", "time_s": 2496, "date": "2026-10-12", "name": "the autumn 10K"}"#,
    );
    assert_eq!(status, 200, "the race was refused: {body}");

    let race = &body["race"];
    assert_eq!(race["label"], "10K");
    assert_eq!(race["time"]["text"], "41:36");
    assert_eq!(race["distance"]["raw"], 10000);
    assert_eq!(race["date"], "2026-10-12");
    assert_eq!(race["name"], "the autumn 10K");
    // The discipline: the day it was entered, the day it last changed, and
    // the source.
    assert_eq!(race["entered_on"], common::TODAY);
    assert_eq!(race["updated_on"], common::TODAY);
    assert_eq!(race["source"], "typed");
}

#[test]
fn a_race_with_a_label_the_app_does_not_know_is_refused() {
    let harness = Harness::empty("athlete-races-label");
    let (status, body) = post(
        &harness,
        "/athlete/races",
        r#"{"label": "a marathon", "time_s": 2496, "date": "2026-10-12"}"#,
    );
    assert_eq!(status, 400, "the race was written: {body}");
    assert_eq!(body["code"], "bad_request");
}

#[test]
fn a_race_without_a_time_is_refused() {
    let harness = Harness::empty("athlete-races-time");
    let (status, body) = post(
        &harness,
        "/athlete/races",
        r#"{"label": "10K", "date": "2026-10-12"}"#,
    );
    assert_eq!(status, 400, "the race was written: {body}");
}

#[test]
fn the_races_the_athlete_typed_are_read_back() {
    let harness = Harness::empty("athlete-races-list");
    post(
        &harness,
        "/athlete/races",
        r#"{"label": "10K", "time_s": 2496, "date": "2026-10-12"}"#,
    );
    let body = harness.ok_get("/athlete/races");
    let races = body["races"].as_array().expect("the races");
    assert_eq!(races.len(), 1);
    assert_eq!(races[0]["label"], "10K");
}

#[test]
fn a_typed_choice_carries_its_value_its_label_and_its_base() {
    let harness = Harness::empty("athlete-threshold");
    let (status, body) = post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "typed", "heart_rate_bpm": 168, "base": "pfitzinger", "label": "coach"}"#,
    );
    assert_eq!(status, 200, "the choice was refused: {body}");

    let choice = &body["choice"];
    assert_eq!(choice["source"], "typed");
    assert_eq!(choice["heart_rate_bpm"], 168.0);
    assert_eq!(choice["base"], "pfitzinger");
    assert_eq!(choice["typed_label"], "a coach");
    assert_eq!(choice["entered_on"], common::TODAY);
    assert_eq!(choice["updated_on"], common::TODAY);
    assert_eq!(choice["in_force"], true);
}

#[test]
fn a_choice_with_no_value_is_refused() {
    let harness = Harness::empty("athlete-threshold-empty");
    let (status, body) = post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "typed", "base": "pfitzinger", "label": "coach"}"#,
    );
    assert_eq!(status, 400, "the choice was written: {body}");
    assert_eq!(body["code"], "bad_request");
}

#[test]
fn a_choice_of_a_candidate_takes_the_candidate_s_own_value() {
    let harness = Harness::empty("athlete-threshold-watch");
    harness.write_zone_model(common::zone_model());
    let (status, body) = post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "watch_zone_model", "base": "eighty_twenty"}"#,
    );
    assert_eq!(status, 200, "the choice was refused: {body}");
    // The floor of the fifth zone.
    assert_eq!(body["choice"]["heart_rate_bpm"], 165.0);
    assert_eq!(body["choice"]["base"], "eighty_twenty");
}

#[test]
fn one_choice_is_in_force_at_a_time_and_the_older_one_stays_in_the_history() {
    let harness = Harness::empty("athlete-threshold-history");
    post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "typed", "heart_rate_bpm": 168, "base": "pfitzinger", "label": "coach"}"#,
    );
    post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "typed", "heart_rate_bpm": 172, "base": "pfitzinger", "label": "lab_test"}"#,
    );

    // Exactly one choice is in force.
    let in_force = harness.store.count_threshold_in_force().unwrap_or_default();
    assert_eq!(in_force, 1, "two thresholds were in force at once");

    // And the older choice is still there, in the history.
    let body = harness.ok_get("/insights/threshold");
    let history = body["history"].as_array().expect("the history");
    assert_eq!(history.len(), 2);
    assert_eq!(body["in_force"]["heart_rate_bpm"], 172.0);
    assert_eq!(history[1]["value"], "168 bpm");
}

#[test]
fn the_base_is_a_closed_list() {
    let harness = Harness::empty("athlete-threshold-base");
    let (status, body) = post(
        &harness,
        "/athlete/threshold",
        r#"{"source": "typed", "heart_rate_bpm": 168, "base": "something_else", "label": "coach"}"#,
    );
    assert_eq!(status, 400, "the choice was written: {body}");
}

#[test]
fn the_programmes_are_served_with_their_entry_requirement() {
    let harness = Harness::empty("plan-bases");
    let body = harness.ok_get("/plan/bases");
    // The catalogue holds the four programmes that were read cell by cell
    // from the book's own tables.
    let bases = body["bases"].as_array().expect("the programmes");
    assert_eq!(bases.len(), 4);
    let ids: Vec<&str> = bases
        .iter()
        .map(|base| base["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![
            "pfitzinger-10k-tier-1",
            "pfitzinger-10k-tier-2",
            "pfitzinger-half-marathon-tier-1",
            "pfitzinger-half-marathon-tier-2",
        ]
    );
    for base in bases {
        assert_eq!(base["method"], "Pfitzinger");
        assert_eq!(base["weeks"], 12);
        // The entry requirement is stated, in the core's own unit.
        assert!(base["entry_requirement"].is_string());
        assert!(base["entry_from_m"].as_f64().unwrap() > 0.0);
        assert!(base["entry_to_m"].as_f64().unwrap() > base["entry_from_m"].as_f64().unwrap());
    }
    // And there is nothing to be unavailable about.
    assert!(body["unavailable"].is_null());
}
