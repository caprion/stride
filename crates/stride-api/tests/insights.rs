// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The four insight routes: the ladder, the curve, the threshold and the
//! ledger.
//!
//! **Facts, never verdicts.** The answers carry values, periods, models and
//! sources, and a missing value is `null` with a reason — never a zero and
//! never an invented number.

mod common;

use common::{run_with_fit, Harness};
use serde_json::Value;
use stride_store::{NewHeartRateZoneModel, NewRaceObservation, NewThresholdObservation};

/// The zone model the athlete's watch was set to.
fn zone_model() -> NewHeartRateZoneModel {
    NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(183.0),
        zone1_floor: 115.0,
        zone2_floor: 128.0,
        zone3_floor: 143.0,
        zone4_floor: 159.0,
        zone5_floor: 165.0,
        read_at: common::instant(),
    }
}

/// The athlete's own race: a 10K in 41:36 on 12 October 2026.
fn race() -> NewRaceObservation {
    NewRaceObservation {
        distance_m: 10_000.0,
        time_s: 2_496.0,
        race_date: "2026-10-12".to_string(),
        label: "10K".to_string(),
        name: Some("the autumn 10K".to_string()),
        source: "typed".to_string(),
        entered_on: "2026-10-12".to_string(),
        updated_on: "2026-10-12".to_string(),
    }
}

/// A harness with the athlete's own facts in it.
fn athlete(prefix: &str) -> Harness {
    let harness = Harness::empty(prefix);
    harness.write_zone_model(zone_model());
    harness.write_race(race());
    harness
}

/// The rungs of the ladder, by kind.
fn rungs(body: &Value) -> Vec<(String, Value)> {
    body["rungs"]
        .as_array()
        .map(|rungs| {
            rungs
                .iter()
                .map(|rung| {
                    (
                        rung["kind"].as_str().unwrap_or_default().to_string(),
                        rung.clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_ladder_carries_the_rungs_their_rules_and_their_citations() {
    let harness = athlete("insights-paces");
    let body = harness.ok_get("/insights/paces");

    // The window is named.
    assert_eq!(body["window"]["weeks"], 12);
    assert_eq!(body["window"]["to"], common::TODAY);

    // The race the paces come from.
    assert_eq!(body["race"]["label"], "10K");
    assert_eq!(body["race"]["source"], "typed");
    assert_eq!(body["race"]["entered_on"], "2026-10-12");
    assert_eq!(body["race"]["time"]["text"], "41:36");

    // The rungs: the threshold rung carries its rule, its citation and
    // its pace.
    let rungs = rungs(&body);
    assert_eq!(rungs.len(), 5);
    let threshold = rungs
        .iter()
        .find(|(kind, _)| kind == "lactate_threshold")
        .map(|(_, rung)| rung.clone())
        .expect("the threshold rung");
    assert_eq!(threshold["rule"]["id"], "book.lt_pace_band");
    assert_eq!(threshold["rule"]["citation"], "ch. 1, ch. 7");
    assert!(threshold["rule"]["line"].is_string());
    assert_eq!(threshold["anchor"], "threshold_band");
    assert!(threshold["pace"]["value"].is_string());
    // The band is the athlete's own: the model's maximum heart rate.
    assert_eq!(threshold["heart_rate"]["value"], "146–167 bpm");
    assert_eq!(threshold["heart_rate"]["max_heart_rate"], 183.0);

    // The two anchors are named and never mixed.
    let goal = rungs
        .iter()
        .find(|(kind, _)| kind == "goal_race")
        .map(|(_, rung)| rung.clone())
        .expect("the goal-race rung");
    assert_eq!(goal["anchor"], "goal_race");
    assert_ne!(goal["pace"]["value"], threshold["pace"]["value"]);

    // The source line is hoisted, and the lines are behind the click.
    assert!(body["source"].is_string());
    assert!(!body["sources"].as_array().unwrap().is_empty());
}

#[test]
fn the_ladder_without_a_race_says_why_it_has_no_value() {
    let harness = Harness::empty("insights-paces-empty");
    let body = harness.ok_get("/insights/paces");

    assert!(body["race"].is_null());
    // The rungs are still there — a rung with no pace is no value with a
    // reason, not a rung that is not there.
    let rungs = rungs(&body);
    assert_eq!(rungs.len(), 5);
    for (_, rung) in &rungs {
        assert!(rung["pace"]["value"].is_null());
        assert!(rung["pace"]["missing"].is_string());
    }
    let reason = body["unavailable"].as_str().unwrap_or_default();
    assert!(
        reason.contains("race"),
        "the reason was {reason:?}, and it does not name the race"
    );
}

#[test]
fn the_curve_carries_the_relation_the_races_and_the_method_s_sessions() {
    let harness = athlete("insights-curve");
    let body = harness.ok_get("/insights/pacing-curve");

    // The curve itself.
    let curve = body["curve"].as_array().expect("the curve");
    assert!(!curve.is_empty());
    let first = &curve[0];
    assert!(first["percent_max"].as_f64().unwrap_or_default() > 0.0);
    assert!(first["pace"]["text"].is_string());

    // The athlete's own races, as points.
    let races = body["races"].as_array().expect("the races");
    assert_eq!(races.len(), 1);
    assert_eq!(races[0]["label"], "10K");
    assert_eq!(races[0]["time"], "41:36");
    assert!((races[0]["minutes"].as_f64().unwrap_or_default() - 41.6).abs() < 0.1);

    // The method's own sessions: the threshold session and the recovery
    // run, and the long run, whose length comes with the plan's weeks.
    let sessions = body["sessions"].as_array().expect("the sessions");
    let kinds: Vec<String> = sessions
        .iter()
        .map(|session| session["kind"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(kinds, vec!["threshold", "recovery", "long_run"]);
    let threshold = sessions
        .iter()
        .find(|session| session["kind"] == "threshold")
        .expect("the threshold session");
    assert_eq!(threshold["from_minutes"], 20.0);
    assert_eq!(threshold["to_minutes"], 40.0);
    let long_run = sessions
        .iter()
        .find(|session| session["kind"] == "long_run")
        .expect("the long run");
    assert!(long_run["from_minutes"].is_null());
    assert!(long_run["missing"].is_string());

    // The field test's marker: forty-five minutes.
    assert_eq!(body["field_test"]["minutes"], 45.0);

    // The equivalents.
    assert!(!body["equivalents"].as_array().unwrap().is_empty());
}

#[test]
fn the_curve_without_a_race_has_no_value() {
    let harness = Harness::empty("insights-curve-empty");
    let body = harness.ok_get("/insights/pacing-curve");
    assert!(body["curve"].as_array().unwrap().is_empty());
    assert!(body["unavailable"].is_string());
}

#[test]
fn the_threshold_carries_the_four_candidates_the_choice_and_the_history() {
    let harness = athlete("insights-threshold");
    harness.write_threshold_choice(
        &NewThresholdObservation {
            hr_bpm: Some(168.0),
            pace_low_s_per_km: None,
            pace_high_s_per_km: None,
            source: "typed".to_string(),
            rule_id: None,
            base: Some("pfitzinger".to_string()),
            origin_date: None,
            origin_ref: None,
            label: Some("coach".to_string()),
            entered_on: "2026-11-03".to_string(),
            updated_on: "2026-11-03".to_string(),
        },
        true,
    );
    let body = harness.ok_get("/insights/threshold");

    // The four candidates, in the product's own order.
    let candidates = body["candidates"].as_array().expect("the candidates");
    assert_eq!(candidates.len(), 4);
    let sources: Vec<String> = candidates
        .iter()
        .map(|candidate| candidate["source"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        sources,
        vec![
            "watch_zone_model".to_string(),
            "race".to_string(),
            "field_test".to_string(),
            "typed".to_string()
        ]
    );

    // The watch's own candidate is the floor of the fifth zone.
    assert_eq!(candidates[0]["value"], "165 bpm");
    assert_eq!(candidates[0]["date"], common::TODAY);
    assert!(candidates[0]["note"].is_string());

    // The race's candidate is the hour-pace band.
    assert!(candidates[1]["value"].is_string());
    assert!(candidates[1]["rule"]["citation"].is_string());

    // The field test's candidate has no run to read: it says so.
    assert!(candidates[2]["value"].is_null());
    assert!(candidates[2]["missing"].is_string());

    // The typed candidate carries the athlete's own number.
    assert_eq!(candidates[3]["value"], "168 bpm");

    // The choice in force, and the history it is in.
    assert_eq!(body["in_force"]["heart_rate_bpm"], 168.0);
    assert_eq!(body["in_force"]["base"], "pfitzinger");
    assert_eq!(body["in_force"]["typed_label"], "a coach");
    assert_eq!(body["in_force"]["in_force"], true);
    let history = body["history"].as_array().expect("the history");
    assert_eq!(history.len(), 1);
    assert!(history[0]["line"]
        .as_str()
        .unwrap_or_default()
        .contains("168 bpm"));
}

#[test]
fn the_ledger_carries_the_kinds_the_weeks_the_coverage_and_the_two_bases() {
    let harness = Harness::empty("insights-ledger");
    let id = harness.write_run(run_with_fit("a-run", common::TODAY, "hr_gps_run.fit"));
    harness.write_zone_model(zone_model());
    let body = harness.ok_get("/insights/stimulus");

    // The window is named.
    assert_eq!(body["window"]["weeks"], 12);

    // The kinds: the athlete's own bands.
    let kinds = body["kinds"].as_array().expect("the kinds");
    assert_eq!(kinds.len(), 3);
    assert_eq!(kinds[0]["key"], "low");
    assert_eq!(kinds[0]["label"], "Z1–Z2 low");

    // The coverage.
    assert_eq!(body["coverage"]["runs"], 1);

    // The weeks matrix: the band each week held.
    let weeks = body["weeks"].as_array().expect("the weeks");
    assert!(!weeks.is_empty());
    assert_eq!(weeks[0]["bands"].as_array().unwrap().len(), 3);

    // The repeated work: no value with a reason.
    assert!(body["repeated"].is_null());
    assert!(body["repeated_missing"].is_string());

    // The overlap: there is none, because the watch recorded no zone
    // minutes for this run.
    assert!(body["overlap"].is_null());
    let _ = id;
}

#[test]
fn the_ledger_with_no_runs_says_why_it_has_no_value() {
    let harness = Harness::empty("insights-ledger-empty");
    let body = harness.ok_get("/insights/stimulus");
    assert!(body["kinds"].as_array().unwrap().is_empty());
    assert!(body["unavailable"].is_string());
    assert_eq!(body["coverage"]["runs"], 0);
}

#[test]
fn the_ledger_reads_each_run_once_and_serves_the_second_read_from_the_cache() {
    let harness = Harness::empty("insights-ledger-cache");
    harness.write_run(run_with_fit("a-run", common::TODAY, "hr_gps_run.fit"));
    harness.write_zone_model(zone_model());

    harness.ok_get("/insights/stimulus");
    let first = harness.api.decoded_cache().stats();
    assert_eq!(first.misses, 1);
    assert_eq!(first.hits, 0);

    harness.ok_get("/insights/stimulus");
    let second = harness.api.decoded_cache().stats();
    assert_eq!(second.misses, 1, "the second read decoded the file again");
    assert_eq!(
        second.hits, 1,
        "the second read was not served from the cache"
    );
}
