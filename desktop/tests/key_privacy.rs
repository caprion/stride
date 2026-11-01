//! The Garmin key never appears.
//!
//! One key, one promise: **it is never logged, never returned by an
//! endpoint, and never written into the database.**
//!
//! The test puts a distinctive key in play, drives the whole rig, and then
//! reads back everything the rig produced: every response body, every line
//! of the log, and the database file itself. Nothing in any of them may
//! carry the key.

mod common;

use std::sync::{Arc, Mutex};

use stride_desktop::error::Result;
use stride_desktop::key::{GarminKey, KeyStore};
use stride_desktop::trust::Trust;

use common::{body_of, elsewhere, loopback, request, request_with_body, server};

/// A key that is recognisable if it ever turns up anywhere.
///
/// It is not credential-shaped on purpose: this is a test fixture, and the
/// repository's secret scanner would (rightly) refuse a string that looked
/// like a real Garmin key.
const THE_GARMIN_KEY: &str = "stride-garmin-key-4f2c9a7e1b6d8305-do-not-print";

/// A log that keeps what it is given.
fn capture() -> Arc<Mutex<Vec<String>>> {
    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink_lines = Arc::clone(&lines);
    let collect = move |line: &str| {
        if let Ok(mut guard) = sink_lines.lock() {
            guard.push(line.to_string());
        }
    };
    stride_desktop::log::set_sink(Arc::new(collect));
    let store_lines = Arc::clone(&lines);
    stride_store::logging::set_sink(Arc::new(move |line: &str| {
        if let Ok(mut guard) = store_lines.lock() {
            guard.push(line.to_string());
        }
    }));
    lines
}

/// Every line that was logged.
fn lines_of(lines: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    match lines.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => Vec::new(),
    }
}

#[test]
fn the_key_is_in_no_response_and_no_log_line() -> Result<()> {
    let lines = capture();
    let server = server(Trust::loopback_only());

    let keys = KeyStore::empty();
    keys.set(THE_GARMIN_KEY)?;

    // Every kind of request the rig answers.
    let responses = [
        server.handle(&request("GET", "/api/v1/activities"), loopback()),
        server.handle(&request("GET", "/api/v1/auth/status"), loopback()),
        server.handle(
            &request_with_body(
                "POST",
                "/api/v1/auth/garmin/personal-key",
                &format!("{{\"key\":\"{THE_GARMIN_KEY}\"}}"),
            ),
            loopback(),
        ),
        server.handle(
            &request("GET", &format!("/api/v1/auth/status?key={THE_GARMIN_KEY}")),
            loopback(),
        ),
        server.handle(&request("GET", "/"), loopback()),
        server.handle(&request("GET", "/api/v1/activities"), elsewhere()),
    ];

    for response in &responses {
        let body = body_of(response);
        assert!(
            !body.contains(THE_GARMIN_KEY),
            "a response carried the key: {body}"
        );
    }

    for line in lines_of(&lines) {
        assert!(
            !line.contains(THE_GARMIN_KEY),
            "a log line carried the key: {line}"
        );
    }

    stride_desktop::log::clear_sink();
    stride_store::logging::clear_sink();
    Ok(())
}

#[test]
fn the_key_is_not_in_the_debug_output() -> Result<()> {
    let key = GarminKey::new(THE_GARMIN_KEY)?;
    let printed = format!("{key:?}");
    assert_eq!(printed, "GarminKey([redacted])");
    assert!(!printed.contains(THE_GARMIN_KEY));
    Ok(())
}

#[test]
fn the_key_is_not_in_the_database() -> Result<()> {
    let lines = capture();
    let directory = std::env::temp_dir().join("stride-key-privacy-test");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;

    // A database, opened with a key of its own.
    let database_key = stride_desktop::dbkey::DatabaseSecret::generate()?;
    let database = directory.join("stride.db");
    let store = stride_store::Store::open_and_migrate(&database, &database_key.store_key()?)?;
    store.upsert_activity(&stride_store::NewActivity {
        source: "garmin_personal".to_string(),
        external_id: "test-1".to_string(),
        name: "a run".to_string(),
        sport_type: "running".to_string(),
        start_time: chrono::Utc::now(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_100.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        fit_blob: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })?;
    drop(store);

    // The key was never anywhere near it.
    let bytes = std::fs::read(&database)?;
    let as_text = String::from_utf8_lossy(&bytes).into_owned();
    assert!(
        !as_text.contains(THE_GARMIN_KEY),
        "the database carried the Garmin key"
    );

    for line in lines_of(&lines) {
        assert!(
            !line.contains(THE_GARMIN_KEY),
            "a log line carried the key: {line}"
        );
    }

    let _ = std::fs::remove_dir_all(&directory);
    stride_desktop::log::clear_sink();
    stride_store::logging::clear_sink();
    Ok(())
}

#[test]
fn a_key_in_a_file_is_only_read_from_a_private_file() -> Result<()> {
    let directory = std::env::temp_dir().join("stride-key-file-test");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("garmin.key");

    std::fs::write(&path, THE_GARMIN_KEY)?;

    // A file other people can read is refused.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path)?.permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&path, permissions)?;
        assert!(
            KeyStore::from_file(&path).is_err(),
            "a key in a file other people can read was read anyway"
        );
    }

    // A file only its owner can read is read.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path)?.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&path, permissions)?;
    }
    let store = KeyStore::from_file(&path)?;
    assert!(store.is_set());
    let seen = store.with_secret(|secret| secret.to_string());
    assert_eq!(seen.as_deref(), Some(THE_GARMIN_KEY));

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn an_empty_key_is_refused() -> Result<()> {
    assert!(GarminKey::new("").is_err());
    assert!(KeyStore::empty().set("   ").is_err());
    Ok(())
}
