//! The plan: the block in force, the day, the week, the count and the
//! results — all of them on the core's engine.
//!
//! Two rules the whole file is about:
//!
//! * **a baseline is not a prescription.** It is inferred from the
//!   athlete's own history, and it never yields "not done".
//! * **an unrecognised session is unclassified**, never given a kind it did
//!   not have.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::Value;

/// The sessions of an answer.
fn sessions<'a>(body: &'a Value, name: &str) -> &'a Vec<Value> {
    match body.get(name).and_then(|value| value.as_array()) {
        Some(items) => items,
        None => panic!("{name} is not an array"),
    }
}

/// A history of runs: enough for the engine to infer a usual week.
///
/// The engine needs eight weeks and twelve runs; a thinner history yields
/// no baseline, and it says so rather than inventing a week.
fn history(harness: &common::Harness) {
    let days = [
        ("2026-03-23", 10_000.0),
        ("2026-03-26", 8_000.0),
        ("2026-03-29", 12_000.0),
        ("2026-04-02", 10_000.0),
        ("2026-04-05", 9_000.0),
        ("2026-04-09", 14_000.0),
        ("2026-04-12", 11_000.0),
        ("2026-04-16", 7_000.0),
        ("2026-04-19", 10_500.0),
        ("2026-04-23", 13_000.0),
        ("2026-04-26", 12_500.0),
        ("2026-04-30", 9_500.0),
        ("2026-05-03", 10_000.0),
        ("2026-05-07", 15_000.0),
        ("2026-05-10", 11_000.0),
    ];
    for (index, (date, distance)) in days.iter().enumerate() {
        harness.write_run(common::run(
            &format!("a-run-{index}"),
            date,
            *distance,
            3_000.0,
        ));
    }
}

#[test]
fn with_no_plan_of_their_own_the_plan_is_the_baseline() {
    let harness = common::Harness::empty("plan-baseline");
    history(&harness);

    let body = harness.ok_get("/plan/current");
    assert_eq!(body["source"].as_str(), Some("baseline"));
    assert_eq!(body["is_baseline"].as_bool(), Some(true));
    assert_eq!(body["baseline_weeks"].as_u64(), Some(8));
    let block = body.get("block").expect("the baseline is a block");
    assert_eq!(block["source"].as_str(), Some("baseline"));
    assert_eq!(block["is_baseline"].as_bool(), Some(true));
    // The baseline is inferred: it is not a block in the store.
    assert!(block["id"].is_null());
}

#[test]
fn a_plan_of_the_athletes_own_is_the_plan_in_force() {
    let harness = common::Harness::empty("plan-own");
    history(&harness);
    harness.write_plan(
        "My plan",
        "prescribed",
        &[
            ("2026-05-12", "easy", Some(10_000.0)),
            ("2026-05-13", "long", Some(20_000.0)),
        ],
    );

    let body = harness.ok_get("/plan/current");
    assert_eq!(body["source"].as_str(), Some("prescribed"));
    assert_eq!(body["is_baseline"].as_bool(), Some(false));
    assert!(body["baseline_weeks"].is_null());
    assert_eq!(body["block"]["name"].as_str(), Some("My plan"));
    assert_eq!(body["block"]["session_count"].as_u64(), Some(2));
}

#[test]
fn the_baseline_never_says_a_session_was_not_done() {
    let harness = common::Harness::empty("plan-baseline-not-done");
    history(&harness);

    let body = harness.ok_get("/plan/week");
    for session in sessions(&body, "sessions").iter() {
        let status = session["status"].as_str().unwrap_or_default();
        assert_ne!(status, "not_done", "a baseline said a session was not done");
    }
}

#[test]
fn the_week_carries_the_core_s_wording_and_whether_a_run_happened() {
    let harness = common::Harness::empty("plan-week");
    harness.write_run(common::run(
        "a-run-on-the-tenth",
        "2026-05-10",
        10_000.0,
        3_000.0,
    ));
    harness.write_plan(
        "My plan",
        "prescribed",
        &[
            ("2026-05-11", "easy", Some(10_000.0)),
            ("2026-05-12", "long", Some(20_000.0)),
        ],
    );

    let body = harness.ok_get("/plan/week");
    assert_eq!(body["source"].as_str(), Some("prescribed"));
    assert_eq!(body["week_start"].as_str(), Some("2026-05-11"));
    assert_eq!(body["week_end"].as_str(), Some("2026-05-17"));
    let sessions = sessions(&body, "sessions");
    assert_eq!(sessions.len(), 2);

    let first = &sessions[0];
    assert_eq!(first["date"].as_str(), Some("2026-05-11"));
    assert_eq!(first["session_type"].as_str(), Some("easy"));
    // The target is the core's own words, and the number beside it.
    assert_eq!(first["target"]["text"].as_str(), Some("10 km"));
    assert_eq!(first["target"]["raw"].as_f64(), Some(10_000.0));
    // The core's wording, and the core's label inside it.
    assert_eq!(
        first["sentence"].as_str(),
        Some("11 May 2026 — 10 km · easy")
    );
    // The eleventh was a Monday, and nothing was run on it.
    assert_eq!(first["status"].as_str(), Some("not_done"));
    assert!(first["actual"].is_null());

    let second = &sessions[1];
    assert_eq!(second["date"].as_str(), Some("2026-05-12"));
    assert_eq!(second["sentence"].as_str(), Some("Today — 20 km · long"));
    assert_eq!(second["status"].as_str(), Some("planned"));

    // The athlete's day is the api's day, and the app highlights it.
    assert_eq!(body["today"].as_str(), Some("2026-05-12"));

    // The counts are by status: the three add up to the week's sessions.
    let counts = body.get("counts").expect("the week carries its counts");
    let total = counts["done"].as_u64().unwrap_or_default()
        + counts["planned"].as_u64().unwrap_or_default()
        + counts["not_done"].as_u64().unwrap_or_default();
    assert_eq!(total, sessions.len() as u64);
    assert_eq!(counts["not_done"].as_u64(), Some(1));
    assert_eq!(
        body["adherence"]["planned"].as_u64(),
        Some(sessions.len() as u64)
    );
}

#[test]
fn the_day_carries_the_core_s_label_and_wording() {
    let harness = common::Harness::empty("plan-today");
    harness.write_plan(
        "My plan",
        "prescribed",
        &[("2026-05-12", "easy", Some(10_000.0))],
    );

    let body = harness.ok_get("/plan/today");
    assert_eq!(body["date"].as_str(), Some("2026-05-12"));
    let planned = body.get("session").expect("the day has a session");
    // The core's wording, and the core's label inside it.
    assert_eq!(planned["sentence"].as_str(), Some("Today — 10 km · easy"));
    assert_eq!(planned["source"].as_str(), Some("prescribed"));
    assert_eq!(planned["status"].as_str(), Some("planned"));
}

#[test]
fn the_sources_are_named_and_one_of_them_is_in_force() {
    let harness = common::Harness::empty("plan-sources");
    harness.write_plan(
        "My plan",
        "typed",
        &[("2026-05-12", "easy", Some(10_000.0))],
    );

    let body = harness.ok_get("/plan/sources");
    let sources = sessions(&body, "sources");
    assert_eq!(sources.len(), 3);
    let names: Vec<&str> = sources
        .iter()
        .map(|source| source["name"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(names, vec!["prescribed", "typed", "baseline"]);
    // The source in force is named, and it is the typed one.
    assert_eq!(body["active_source"].as_str(), Some("typed"));
    for source in sources.iter() {
        assert!(source["name"].is_string());
    }
}

#[test]
fn the_adherence_is_a_count_and_not_a_score() {
    let harness = common::Harness::empty("plan-adherence");
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    harness.write_plan(
        "My plan",
        "prescribed",
        &[
            ("2026-05-10", "easy", Some(10_000.0)),
            ("2026-05-11", "easy", Some(8_000.0)),
            ("2026-05-13", "easy", Some(12_000.0)),
        ],
    );

    let body = harness.ok_get("/plan/adherence");
    let count = body.get("adherence").expect("there is a count");
    assert_eq!(count["planned"].as_u64(), Some(3));
    assert_eq!(count["done"].as_u64(), Some(1));
    // The count is a count: there is no score and no percentage anywhere in
    // the answer.
    assert!(count.get("score").is_none());
    assert!(count.get("percentage").is_none());
}

#[test]
fn the_results_state_the_intent_the_actual_and_the_difference() {
    let harness = common::Harness::empty("plan-results");
    harness.write_run(common::run("a-run", "2026-05-10", 12_400.0, 3_500.0));
    harness.write_plan(
        "My plan",
        "prescribed",
        &[("2026-05-10", "easy", Some(10_000.0))],
    );

    let body = harness.ok_get("/plan/results");
    let results = sessions(&body, "results");
    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result["date"].as_str(), Some("2026-05-10"));
    assert_eq!(result["target"]["text"].as_str(), Some("10 km"));
    assert_eq!(result["actual"]["text"].as_str(), Some("12.4 km"));
    // The difference is the core's statement, and it says nothing that is
    // good or bad.
    assert_eq!(
        result["difference"].as_str(),
        Some("2.4 km more than the 10 km target")
    );
}

#[test]
fn a_difference_within_ten_per_cent_is_not_stated() {
    let harness = common::Harness::empty("plan-results-small-gap");
    harness.write_run(common::run("a-run", "2026-05-10", 10_200.0, 3_000.0));
    harness.write_plan(
        "My plan",
        "prescribed",
        &[("2026-05-10", "easy", Some(10_000.0))],
    );

    let body = harness.ok_get("/plan/results");
    let results = sessions(&body, "results");
    assert!(results[0]["difference"].is_null());
}

#[test]
fn a_plan_the_athlete_typed_in_is_recorded_as_typed() {
    let harness = common::Harness::empty("plan-import-typed");
    let body = harness.post(
        "/plan/import",
        "{\"name\":\"My plan\",\"sessions\":[{\"date\":\"2026-05-12\",\"session_type\":\"easy\",\"target_distance_m\":10000}]}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(1));
    assert!(body["block_id"].as_i64().is_some());

    // The source is recorded on the block.
    let current = harness.ok_get("/plan/current");
    assert_eq!(current["source"].as_str(), Some("typed"));
}

#[test]
fn a_plan_the_athlete_imported_is_recorded_as_prescribed() {
    let harness = common::Harness::empty("plan-import-prescribed");
    let body = harness.post(
        "/plan/import",
        "{\"name\":\"A coach's plan\",\"format\":\"csv\",\"content\":\"date,type,distance\\n2026-05-12,easy,10 km\\n2026-05-13,long,20 km\\n\"}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(2));

    // The source is recorded on the block.
    let current = harness.ok_get("/plan/current");
    assert_eq!(current["source"].as_str(), Some("prescribed"));
    assert_eq!(current["block"]["name"].as_str(), Some("A coach's plan"));
}

#[test]
fn a_markdown_plan_is_read() {
    let harness = common::Harness::empty("plan-import-markdown");
    let body = harness.post(
        "/plan/import",
        "{\"format\":\"markdown\",\"content\":\"- 2026-05-12: easy 10 km\\n- 2026-05-13: long 20 km\\n\"}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(2));

    let week = harness.ok_get("/plan/week");
    let sessions = sessions(&week, "sessions");
    assert_eq!(sessions[0]["session_type"].as_str(), Some("easy"));
    assert_eq!(sessions[0]["target"]["text"].as_str(), Some("10 km"));
    assert_eq!(sessions[1]["session_type"].as_str(), Some("long"));
}

#[test]
fn a_garmin_workout_schedule_is_read() {
    let harness = common::Harness::empty("plan-import-garmin");
    let body = harness.post(
        "/plan/import",
        "{\"format\":\"garmin_csv\",\"content\":\"Workout Name,Date,Distance\\nEasy Run 5K,2026-05-12,5\\nLong Run,2026-05-14,18\\n\"}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(2));

    let week = harness.ok_get("/plan/week");
    let sessions = sessions(&week, "sessions");
    assert_eq!(sessions[0]["session_type"].as_str(), Some("easy"));
    assert_eq!(sessions[0]["target"]["text"].as_str(), Some("5 km"));
}

#[test]
fn a_json_plan_is_read() {
    let harness = common::Harness::empty("plan-import-json");
    let body = harness.post(
        "/plan/import",
        "{\"format\":\"json\",\"content\":\"{\\\"name\\\":\\\"A JSON plan\\\",\\\"sessions\\\":[{\\\"date\\\":\\\"2026-05-12\\\",\\\"type\\\":\\\"tempo\\\",\\\"distance_km\\\":8}]}\"}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["sessions"].as_u64(), Some(1));

    let current = harness.ok_get("/plan/current");
    assert_eq!(current["block"]["name"].as_str(), Some("A JSON plan"));
}

#[test]
fn a_workout_the_vocabulary_does_not_know_is_unclassified() {
    let harness = common::Harness::empty("plan-import-unclassified");
    let body = harness.post(
        "/plan/import",
        "{\"format\":\"csv\",\"content\":\"date,type,distance\\n2026-05-12,Hill repeats,10\\n\"}",
    );
    assert_eq!(body["imported"].as_bool(), Some(true));

    let week = harness.ok_get("/plan/week");
    let sessions = sessions(&week, "sessions");
    // Never "easy": the import does not guess. The session keeps the
    // athlete's own words as its title.
    assert_eq!(sessions[0]["session_type"].as_str(), Some("unclassified"));
    assert_eq!(sessions[0]["title"].as_str(), Some("Hill repeats"));
}

#[test]
fn a_plan_the_import_cannot_read_is_refused_and_nothing_is_written() {
    let harness = common::Harness::empty("plan-import-refused");
    let answer = harness.router.call(
        "POST",
        "/plan/import",
        Some("{\"format\":\"csv\",\"content\":\"date,type\\n,easy\\n\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
    // The refusal says which line it could not read.
    let detail = answer.body["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("line 2"), "the detail was {detail}");

    // Nothing was written.
    let blocks = harness.ok_get("/plan/blocks");
    assert!(blocks["blocks"].as_array().unwrap().is_empty());
    let current = harness.ok_get("/plan/current");
    assert!(current["block"].is_null());
}

#[test]
fn a_plan_in_a_format_the_import_does_not_know_is_refused() {
    let harness = common::Harness::empty("plan-import-unknown-format");
    let answer = harness.router.call(
        "POST",
        "/plan/import",
        Some("{\"format\":\"pdf\",\"content\":\"whatever\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}
