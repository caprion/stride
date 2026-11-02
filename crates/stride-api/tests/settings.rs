//! The settings: the goal, the pause, the storage, and the note on a run.
//!
//! Four rules the whole file is about:
//!
//! * **a blank goal is no goal** — never a zero that reads as a failure;
//! * **a pause suspends the adherence**, and the answer says so;
//! * **a note is the only free text the product stores**, and a run
//!   carries one;
//! * **the delete carries its own confirmation** — the word has to travel
//!   with the request.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn with_no_goal_the_settings_carry_no_goal() {
    let harness = common::Harness::empty("settings-no-goal");
    let body = harness.ok_get("/settings");
    // A blank goal is no goal: it is null, never a zero that reads as a
    // failure.
    assert!(body["goal"]["value"].is_null());
    assert!(body["goal"]["km"].is_null());
    assert!(body["pause"].is_null());
}

#[test]
fn a_goal_is_a_weekly_distance() {
    let harness = common::Harness::empty("settings-goal");
    let body = harness.post("/settings", "{\"goal_km\":40}");
    assert_eq!(body["goal"]["value"]["text"].as_str(), Some("40 km"));
    assert_eq!(body["goal"]["value"]["raw"].as_f64(), Some(40_000.0));
    // The number the field holds, in kilometres.
    assert_eq!(body["goal"]["km"].as_f64(), Some(40.0));

    // And it is still there when the settings are read again.
    let body = harness.ok_get("/settings");
    assert_eq!(body["goal"]["value"]["text"].as_str(), Some("40 km"));
}

#[test]
fn a_blank_goal_takes_the_goal_away() {
    let harness = common::Harness::empty("settings-goal-cleared");
    harness.post("/settings", "{\"goal_km\":40}");
    let body = harness.post("/settings", "{\"goal_km\":null}");
    assert!(body["goal"]["value"].is_null());
    assert!(body["goal"]["km"].is_null());

    let body = harness.ok_get("/settings");
    assert!(body["goal"]["value"].is_null());
}

#[test]
fn a_goal_that_is_not_a_distance_is_refused() {
    let harness = common::Harness::empty("settings-goal-bad");
    let answer = harness
        .router
        .call("POST", "/settings", Some("{\"goal_km\":\"lots\"}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn a_pause_has_a_start_an_end_and_a_reason() {
    let harness = common::Harness::empty("settings-pause");
    let body = harness.post(
        "/settings",
        "{\"pause\":{\"start_date\":\"2026-05-10\",\"end_date\":\"2026-05-14\",\"reason\":\"travel\"}}",
    );
    assert_eq!(body["pause"]["start_date"].as_str(), Some("2026-05-10"));
    assert_eq!(body["pause"]["end_date"].as_str(), Some("2026-05-14"));
    assert_eq!(body["pause"]["reason"].as_str(), Some("travel"));
}

#[test]
fn a_pause_may_have_no_reason() {
    let harness = common::Harness::empty("settings-pause-no-reason");
    let body = harness.post(
        "/settings",
        "{\"pause\":{\"start_date\":\"2026-05-10\",\"end_date\":\"2026-05-14\",\"reason\":null}}",
    );
    assert_eq!(body["pause"]["start_date"].as_str(), Some("2026-05-10"));
    assert!(body["pause"]["reason"].is_null());
}

#[test]
fn a_reason_outside_the_three_is_refused() {
    let harness = common::Harness::empty("settings-pause-reason");
    let answer = harness.router.call(
        "POST",
        "/settings",
        Some("{\"pause\":{\"start_date\":\"2026-05-10\",\"end_date\":\"2026-05-14\",\"reason\":\"weather\"}}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn a_pause_suspends_the_adherence_and_the_answer_says_so() {
    let harness = common::Harness::empty("settings-pause-adherence");
    harness.write_plan(
        "My plan",
        "prescribed",
        &[
            ("2026-05-11", "easy", Some(10_000.0)),
            ("2026-05-12", "easy", Some(8_000.0)),
        ],
    );
    harness.post(
        "/settings",
        "{\"pause\":{\"start_date\":\"2026-05-10\",\"end_date\":\"2026-05-14\",\"reason\":\"sick\"}}",
    );

    let body = harness.ok_get("/plan/adherence");
    assert_eq!(body["suspended"].as_bool(), Some(true));
    assert_eq!(body["suspension_reason"].as_str(), Some("sick"));
}

#[test]
fn with_no_pause_the_adherence_is_not_suspended() {
    let harness = common::Harness::empty("settings-no-pause-adherence");
    harness.write_plan(
        "My plan",
        "prescribed",
        &[("2026-05-12", "easy", Some(8_000.0))],
    );
    let body = harness.ok_get("/plan/adherence");
    assert_eq!(body["suspended"].as_bool(), Some(false));
    assert!(body["suspension_reason"].is_null());
}

#[test]
fn the_settings_carry_what_is_on_the_device() {
    let harness = common::Harness::seeded("settings-storage");
    let body = harness.ok_get("/settings");
    assert_eq!(body["storage"]["activity_count"].as_i64(), Some(1));
    assert_eq!(body["storage"]["wellness_day_count"].as_i64(), Some(3));
    // The size is the host's rendering of the file.
    assert!(body["storage"]["size"]["text"].as_str().is_some());
}

#[test]
fn a_note_is_written_on_a_run_and_read_back() {
    let harness = common::Harness::empty("settings-note");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.post(
        &format!("/activities/{id}/note"),
        "{\"text\":\"a note about the run\"}",
    );
    assert_eq!(body["note"].as_str(), Some("a note about the run"));

    // The note is on the run, and the run carries it.
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert_eq!(body["note"].as_str(), Some("a note about the run"));
}

#[test]
fn a_run_carries_one_note() {
    let harness = common::Harness::empty("settings-note-one");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    harness.post(
        &format!("/activities/{id}/note"),
        "{\"text\":\"the first note\"}",
    );
    harness.post(
        &format!("/activities/{id}/note"),
        "{\"text\":\"the second note\"}",
    );

    let body = harness.post(
        &format!("/activities/{id}/note"),
        "{\"text\":\"the second note\"}",
    );
    assert_eq!(body["note"].as_str(), Some("the second note"));
    let stored = harness.store.notes_for_activity(id).unwrap_or_default();
    assert_eq!(stored.len(), 1);
}

#[test]
fn a_note_on_a_run_that_is_not_there_is_refused() {
    let harness = common::Harness::empty("settings-note-missing");
    let answer = harness.router.call(
        "POST",
        "/activities/9999/note",
        Some("{\"text\":\"a note\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("not_found"));
}

#[test]
fn a_settings_write_with_nothing_in_it_is_refused() {
    let harness = common::Harness::empty("settings-empty");
    let answer = harness.router.call("POST", "/settings", Some("{}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn a_delete_without_its_word_is_refused() {
    let harness = common::Harness::seeded("settings-delete-refused");
    let answer = harness
        .router
        .call("POST", "/settings/delete", Some("{\"confirm\":\"delete\"}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));

    // And the data is still there.
    let feed = harness.ok_get("/activities");
    assert_eq!(feed["activities"].as_array().unwrap().len(), 1);
}

#[test]
fn a_delete_with_its_word_removes_everything() {
    let harness = common::Harness::seeded("settings-delete");
    harness.write_plan(
        "My plan",
        "prescribed",
        &[("2026-05-12", "easy", Some(8_000.0))],
    );
    let body = harness.post("/settings/delete", "{\"confirm\":\"DELETE\"}");
    assert_eq!(body["deleted"].as_bool(), Some(true));

    // The runs, the wellness, the plan and the notes are gone.
    let feed = harness.ok_get("/activities");
    assert!(feed["activities"].as_array().unwrap().is_empty());
    let wellness = harness.ok_get("/physiology/daily");
    assert!(wellness["days"].as_array().unwrap().is_empty());
    let blocks = harness.ok_get("/plan/blocks");
    assert!(blocks["blocks"].as_array().unwrap().is_empty());
    let settings = harness.ok_get("/settings");
    assert_eq!(settings["storage"]["activity_count"].as_i64(), Some(0));
    assert_eq!(settings["storage"]["note_count"].as_i64(), Some(0));
}
