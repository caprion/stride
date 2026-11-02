//! The FIT import: the file comes in, it is stored with the run, and a
//! duplicate is reported and adds no row.
//!
//! The rule is the schema's own: a FIT import dedupes by start ±2 minutes
//! and distance ±2 %. A file that is the same run as one that is already
//! there adds no row, and the answer says which run it already has.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn a_fit_file_is_stored_with_the_run() {
    let harness = common::Harness::empty("fit-import-stored");
    let file = common::corpus_fit("real_garmin_run.fit");
    let body = harness.post(
        "/auth/garmin/fit-import",
        &format!("{{\"fit_base64\":\"{}\"}}", common::base64(&file)),
    );
    assert_eq!(body["imported"].as_bool(), Some(true));
    assert_eq!(body["duplicate"].as_bool(), Some(false));
    let id = body["activity_id"].as_i64().unwrap_or_default();
    assert!(id > 0);

    // The run is there, and the original file is with it: the FIT file is
    // the source of truth.
    let run = harness.ok_get(&format!("/activities/{id}"));
    assert_eq!(run["sport_type"].as_str(), Some("running"));
    let blob = harness
        .store
        .fit_blob(id)
        .unwrap_or(None)
        .unwrap_or_default();
    assert_eq!(blob.len(), file.len());
}

#[test]
fn a_duplicate_adds_no_row_and_the_answer_says_which_run() {
    let harness = common::Harness::empty("fit-import-duplicate");
    let file = common::corpus_fit("real_garmin_run.fit");
    let first = harness.post(
        "/auth/garmin/fit-import",
        &format!("{{\"fit_base64\":\"{}\"}}", common::base64(&file)),
    );
    let id = first["activity_id"].as_i64().unwrap_or_default();
    let run = harness.ok_get(&format!("/activities/{id}"));
    let date = run["local_date"].as_str().unwrap_or_default().to_string();
    let distance = run["distance"]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    // The same file again: the same run. No row is added, and the answer
    // says which run is already in the log.
    let second = harness.post(
        "/auth/garmin/fit-import",
        &format!("{{\"fit_base64\":\"{}\"}}", common::base64(&file)),
    );
    assert_eq!(second["imported"].as_bool(), Some(false));
    assert_eq!(second["duplicate"].as_bool(), Some(true));
    assert_eq!(second["activity_id"].as_i64(), Some(id));
    assert_eq!(second["date"].as_str(), Some(date.as_str()));
    assert_eq!(second["distance"]["text"].as_str(), Some(distance.as_str()));

    // And no second row was added.
    let feed = harness.ok_get("/activities");
    assert_eq!(feed["activities"].as_array().unwrap().len(), 1);
}

#[test]
fn a_file_that_is_not_a_fit_file_is_refused() {
    let harness = common::Harness::empty("fit-import-bad");
    let answer = harness.router.call(
        "POST",
        "/auth/garmin/fit-import",
        Some("{\"fit_base64\":\"bm90IGEgZml0IGZpbGU=\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}
