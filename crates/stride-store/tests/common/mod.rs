//! The helpers the tests share.
//!
//! Every test gets its own scratch directory, and the directory removes
//! itself when the test ends — on the way out of a failure as well as on the
//! way out of a success.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, Result, Store};

/// The key the tests use.
///
/// It is a passphrase, not a raw key, and it is in the test because a test
/// key is not a secret: it opens a database that is thrown away a second
/// later.
pub const TEST_KEY: &str = "the key of a test that throws its database away";

/// A scratch directory for one test.
pub fn scratch(prefix: &str) -> Result<ScratchDir> {
    ScratchDir::new(prefix)
}

/// The test's key.
pub fn key() -> Result<DatabaseKey> {
    DatabaseKey::from_text(TEST_KEY)
}

/// An empty, migrated store in a scratch directory.
pub fn store(prefix: &str) -> Result<(ScratchDir, Store)> {
    let dir = scratch(prefix)?;
    let store = Store::open_and_migrate(&dir.file("stride.db"), &key()?)?;
    Ok((dir, store))
}

/// A fixed instant, so that the tests do not depend on the clock.
pub fn instant() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-05-12T06:30:15Z")
        .map(|value| value.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

/// An activity to write.
pub fn activity(external_id: &str, local_date: &str) -> stride_store::NewActivity {
    stride_store::NewActivity {
        source: "garmin_personal".to_string(),
        external_id: external_id.to_string(),
        name: format!("a run on {local_date}"),
        sport_type: "running".to_string(),
        start_time: instant(),
        local_date: local_date.to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_100.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        fit_blob: format!("a fit file for {external_id}").into_bytes(),
        created_at: instant(),
        updated_at: instant(),
    }
}

/// A day of wellness to write.
pub fn wellness(date: &str) -> stride_store::WellnessDaily {
    stride_store::WellnessDaily {
        date: date.to_string(),
        sleep_s: Some(22_200.0),
        deep_sleep_s: Some(5_400.0),
        rem_s: Some(4_800.0),
        light_sleep_s: Some(12_000.0),
        awake_s: Some(1_800.0),
        hrv_ms: Some(48.0),
        resting_hr: Some(63.0),
        stress_avg: Some(28.0),
        source: "garmin_personal".to_string(),
        updated_at: instant(),
    }
}

/// The path of the contract's schema file.
pub fn contract_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("contract")
        .join("schema.sql")
}

/// Read a file, with the store's error type.
pub fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|error| stride_store::StoreError::io(path.to_path_buf(), error))
}

/// Read a file as text, with the store's error type.
pub fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|error| stride_store::StoreError::io(path.to_path_buf(), error))
}

/// Write a file, with the store's error type.
pub fn write(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text)
        .map_err(|error| stride_store::StoreError::io(path.to_path_buf(), error))
}

/// Remove the comments from a SQL statement.
///
/// The contract's comments carry text that a naive splitter would break on:
/// a comment may hold a semicolon. The comments are removed **before** the
/// statements are split, and the same function is used on both sides of the
/// comparison, so the two are treated the same way.
pub fn strip_comments(sql: &str) -> String {
    let mut out = String::new();
    for line in sql.lines() {
        let without = match line.find("--") {
            Some(at) => &line[..at],
            None => line,
        };
        out.push_str(without);
        out.push('\n');
    }
    out
}

/// Collapse the whitespace of a SQL statement, so that two texts that differ
/// only in their layout compare equal.
///
/// The trailing semicolon is dropped: the contract's statements are split on
/// it, and SQLite does not keep it.
pub fn normalise(sql: &str) -> String {
    let collapsed = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .strip_suffix(';')
        .map(|without| without.to_string())
        .unwrap_or(collapsed)
}

/// The statements of a SQL file, with the comments removed.
pub fn statements(sql: &str) -> Vec<String> {
    let without = strip_comments(sql);
    without
        .split(';')
        .map(normalise)
        .filter(|statement| !statement.is_empty())
        .collect()
}
