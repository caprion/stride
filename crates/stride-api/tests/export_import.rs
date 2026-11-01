//! The export and the import of the database.
//!
//! Two rules the whole file is about:
//!
//! * **the path is confined** — an export is named by a plain name inside
//!   the directory the host chose, and nothing else is reachable;
//! * **the digest is verified** — an export that was changed after it was
//!   written is refused, not read.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;

use stride_api::Config;
use stride_store::DatabaseKey;

/// The state, with an export directory the host chose and a key for it.
fn state(prefix: &str, root: &Path) -> common::Harness {
    let harness = common::Harness::with_config(
        prefix,
        Config {
            export_root: Some(root.to_path_buf()),
            ..Config::default()
        },
    );
    let key = DatabaseKey::from_text("the database key of a test")
        .unwrap_or_else(|_| panic!("the database key could not be made"));
    harness.router.api().set_export_key(key);
    harness
}

/// A scratch directory for the exports, alive for the length of the test.
fn root(prefix: &str) -> stride_store::scratch::ScratchDir {
    match stride_store::scratch::ScratchDir::new(prefix) {
        Ok(dir) => dir,
        Err(_) => panic!("the scratch directory could not be made"),
    }
}

#[test]
fn an_export_is_written_and_says_what_it_holds() {
    let root = root("export-written");
    let harness = state("export-written", root.path());
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    let body = harness.post("/data/export", "{\"directory\":\"a-backup\"}");
    assert_eq!(body["directory"].as_str(), Some("a-backup"));
    assert_eq!(body["rows"].as_i64(), Some(1));
    assert_eq!(body["tables"]["activities"].as_i64(), Some(1));
    assert!(!body["database_sha256"]
        .as_str()
        .unwrap_or_default()
        .is_empty());

    // The directory holds the manifest and the database.
    assert!(root.path().join("a-backup").join("manifest.json").exists());
    assert!(root.path().join("a-backup").join("stride.db").exists());
}

#[test]
fn an_export_is_read_back() {
    let root = root("export-read-back");
    let harness = state("export-read-back", root.path());
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    harness.post("/data/export", "{\"directory\":\"a-backup\"}");

    // The rows come back into a second database.
    let other = state("export-read-back-other", root.path());
    let body = other.post("/data/import", "{\"directory\":\"a-backup\"}");
    assert_eq!(body["activities"].as_u64(), Some(1));
    assert_eq!(body["rows"].as_u64(), Some(1));

    // And the run is there.
    let feed = other.ok_get("/activities");
    assert_eq!(feed["activities"].as_array().unwrap().len(), 1);
}

#[test]
fn a_name_outside_the_export_directory_is_refused() {
    let root = root("export-confined");
    let harness = state("export-confined", root.path());
    let answer = harness.router.call(
        "POST",
        "/data/import",
        Some("{\"directory\":\"../elsewhere\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));

    let answer = harness.router.call(
        "POST",
        "/data/export",
        Some("{\"directory\":\"../elsewhere\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn an_export_that_was_changed_is_refused() {
    let root = root("export-digest");
    let harness = state("export-digest", root.path());
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    harness.post("/data/export", "{\"directory\":\"a-backup\"}");

    // The file is changed after it was written.
    let database = root.path().join("a-backup").join("stride.db");
    let mut bytes = std::fs::read(&database).unwrap_or_default();
    bytes.push(0);
    let _ = std::fs::write(&database, &bytes);

    let answer = harness
        .router
        .call("POST", "/data/import", Some("{\"directory\":\"a-backup\"}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn an_export_that_is_not_there_is_refused() {
    let root = root("export-missing");
    let harness = state("export-missing", root.path());
    let answer = harness.router.call(
        "POST",
        "/data/import",
        Some("{\"directory\":\"nothing-here\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("not_found"));
}

#[test]
fn a_device_with_no_export_directory_says_so() {
    let harness = common::Harness::empty("export-no-directory");
    let answer = harness.router.call("POST", "/data/export", Some("{}"));
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}
