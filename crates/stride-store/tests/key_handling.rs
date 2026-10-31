//! The key never comes back out.
//!
//! Three things are checked here, and they are checked together because
//! together they are the rule:
//!
//! 1. **No repository returns the key.** The key is applied to the connection
//!    and is not kept, so there is nothing to return. The rows that come out
//!    of the database are checked, one by one, for the key.
//! 2. **The key is never printed.** Every diagnostic the store emits goes
//!    through its log, and the log is captured here: the whole store is
//!    driven, and every line that was produced is read back.
//! 3. **The key is never written down.** The store writes one file — the
//!    database — and the key is not in it: the file does not open without it.

mod common;

use std::sync::{Arc, Mutex};

use stride_store::logging;
use stride_store::{DatabaseKey, Result, Store};

/// A key that is recognisable if it ever turns up anywhere.
const SECRET: &str = "stride-key-9f2c4a7e1b6d8305-do-not-print-me";

/// A log that keeps what it is given.
fn capture() -> Arc<Mutex<Vec<String>>> {
    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink_lines = Arc::clone(&lines);
    logging::set_sink(Arc::new(move |line: &str| {
        if let Ok(mut guard) = sink_lines.lock() {
            guard.push(line.to_string());
        }
    }));
    lines
}

#[test]
fn the_key_is_not_in_any_row_and_not_in_the_log() -> Result<()> {
    let lines = capture();
    let dir = common::scratch("key-handling")?;
    let key = DatabaseKey::from_text(SECRET)?;
    let store = Store::open_and_migrate(&dir.file("stride.db"), &key)?;

    // Drive the whole store: every repository, every write, every read.
    let activity = common::activity("run-1", "2026-05-10");
    let id = store.upsert_activity(&activity)?;
    store.upsert_activity(&common::activity("run-2", "2026-05-11"))?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;
    store.upsert_plan_block(&stride_store::NewPlanBlock {
        name: "a block".to_string(),
        source: "typed".to_string(),
        start_date: "2026-05-01".to_string(),
        end_date: None,
        goal: None,
        status: "active".to_string(),
        notes: None,
        created_at: common::instant(),
    })?;
    store.upsert_note(&stride_store::NewNote {
        kind: "note".to_string(),
        activity_id: Some(id),
        text: Some("a note about the run".to_string()),
        value: None,
        start_date: None,
        end_date: None,
        created_at: common::instant(),
    })?;

    let mut cursor = store.begin_sync("activities")?;
    cursor.advance("a cursor");
    cursor.commit()?;

    // Everything that comes back out.
    let mut read: Vec<String> = Vec::new();
    if let Some(activity) = store.activity(id)? {
        read.push(format!("{activity:?}"));
    }
    if let Some(activity) = store.activity_by_external("garmin_personal", "run-1")? {
        read.push(format!("{activity:?}"));
    }
    let feed = store.activity_feed(None, 10)?;
    read.push(format!("{:?}", feed.activities));
    read.push(format!("{:?}", feed.next_cursor));
    read.push(format!(
        "{:?}",
        store.activities_between("2026-01-01", "2026-12-31")?
    ));
    read.push(format!("{:?}", store.wellness_daily("2026-05-10")?));
    read.push(format!(
        "{:?}",
        store.wellness_between("2026-01-01", "2026-12-31")?
    ));
    read.push(format!("{:?}", store.plan_blocks()?));
    read.push(format!("{:?}", store.active_plan_block("typed")?));
    read.push(format!(
        "{:?}",
        store.plan_sessions_between("2026-01-01", "2026-12-31")?
    ));
    read.push(format!("{:?}", store.notes()?));
    read.push(format!("{:?}", store.sync_state("activities")?));
    read.push(format!("{:?}", store.all_sync_state()?));
    read.push(format!("{:?}", store.fit_blob(id)?));
    read.push(format!("{:?}", store.fit_sha256(id)?));
    read.push(format!("{:?}", store));
    read.push(format!("{:?}", store.path()));
    read.push(format!("{:?}", store.schema_version()?));

    for text in &read {
        assert!(
            !text.contains(SECRET),
            "the key came back out of the store: {text}"
        );
    }

    // The log, line by line.
    let captured = match lines.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => Vec::new(),
    };
    assert!(!captured.is_empty(), "the store said nothing at all");
    for line in &captured {
        assert!(!line.contains(SECRET), "the key was printed: {line}");
    }

    logging::clear_sink();
    Ok(())
}

#[test]
fn the_key_is_not_in_the_database_file() -> Result<()> {
    let dir = common::scratch("key-file")?;
    let path = dir.file("stride.db");
    let key = DatabaseKey::from_text(SECRET)?;
    {
        let store = Store::open_and_migrate(&path, &key)?;
        store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    }

    // The file on the disk does not hold the key: not in the clear, and not
    // as the passphrase the store was given.
    let bytes =
        std::fs::read(&path).map_err(|error| stride_store::StoreError::io(path.clone(), error))?;
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        !text.contains(SECRET),
        "the key is written down in the database file"
    );
    assert!(
        !text.contains("SQLite format 3"),
        "the database file is not encrypted"
    );
    Ok(())
}

#[test]
fn the_wrong_key_does_not_open_the_database() -> Result<()> {
    let dir = common::scratch("key-wrong")?;
    let path = dir.file("stride.db");
    {
        let store = Store::open_and_migrate(&path, &DatabaseKey::from_text(SECRET)?)?;
        store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    }

    let wrong = DatabaseKey::from_text("a different key entirely")?;
    let error = match Store::open(&path, &wrong) {
        Ok(_) => {
            return Err(stride_store::StoreError::Invariant(
                "the wrong key opened the database".to_string(),
            ))
        }
        Err(error) => error,
    };
    // The error says what happened and does not say what the key was.
    let text = error.to_string();
    assert!(!text.contains(SECRET), "the error holds the key: {text}");
    assert!(
        !text.contains("a different key entirely"),
        "the error holds the key: {text}"
    );
    Ok(())
}

#[test]
fn the_key_type_has_no_way_out() {
    // The type prints a fixed phrase, whatever the key is.
    let key = DatabaseKey::from_text(SECRET);
    let key = match key {
        Ok(key) => key,
        Err(_) => return,
    };
    let printed = format!("{key:?}");
    assert_eq!(printed, "DatabaseKey([redacted])");
    assert!(!printed.contains(SECRET));
}
