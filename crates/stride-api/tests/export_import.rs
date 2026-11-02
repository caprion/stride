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
fn an_export_is_written_and_says_how_big_it_is() {
    let root = root("export-written");
    let harness = state("export-written", root.path());
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    let body = harness.post("/settings/export", "{}");
    assert_eq!(body["exported"].as_bool(), Some(true));
    // The size is the host's rendering of the file: "2.4 MB", "64 KB".
    assert!(body["size"]["text"].as_str().is_some());
    assert!(body["size"]["raw"].as_u64().unwrap_or_default() > 0);

    // The directory holds the manifest and the database.
    let directories: Vec<String> = std::fs::read_dir(root.path())
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(directories.len(), 1, "the export directory was not written");
    let written = root.path().join(&directories[0]);
    assert!(written.join("manifest.json").exists());
    assert!(written.join("stride.db").exists());
}

#[test]
fn an_export_is_read_back() {
    let root = root("export-read-back");
    let harness = state("export-read-back", root.path());
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    harness.post("/settings/export", "{}");

    // The rows come back into a second database.
    let other = state("export-read-back-other", root.path());
    let directory = export_directory(root.path());
    let body = other.post(
        "/settings/import",
        &format!("{{\"directory\":\"{directory}\"}}"),
    );
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
        "/settings/import",
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
    harness.post("/settings/export", "{}");

    // The file is changed after it was written.
    let database = root
        .path()
        .join(export_directory(root.path()))
        .join("stride.db");
    let mut bytes = std::fs::read(&database).unwrap_or_default();
    bytes.push(0);
    let _ = std::fs::write(&database, &bytes);

    let answer = harness.router.call(
        "POST",
        "/settings/import",
        Some(&format!(
            "{{\"directory\":\"{}\"}}",
            export_directory(root.path())
        )),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("bad_request"));
}

#[test]
fn an_export_that_is_not_there_is_refused() {
    let root = root("export-missing");
    let harness = state("export-missing", root.path());
    let answer = harness.router.call(
        "POST",
        "/settings/import",
        Some("{\"directory\":\"nothing-here\"}"),
    );
    assert!(!answer.is_ok());
    assert_eq!(answer.body["code"].as_str(), Some("not_found"));
}

#[test]
fn a_device_with_no_export_directory_says_so() {
    let harness = common::Harness::empty("export-no-directory");
    let body = harness.post("/settings/export", "{}");
    assert_eq!(body["exported"].as_bool(), Some(false));
    assert!(body["size"].is_null());
}

/// The name of the one export directory in a directory.
fn export_directory(root: &Path) -> String {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => panic!("the export directory could not be read"),
    };
    match entries.flatten().next() {
        Some(entry) => entry.file_name().to_string_lossy().to_string(),
        None => panic!("there is no export directory"),
    }
}
