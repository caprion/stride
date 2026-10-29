#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The database the athlete actually has.
//!
//! This is the test that matters most in this workstream: the file on disk
//! is a year of history, and the desktop has to open it and read it. If the
//! schema drifted, the desktop loses the history — silently, because an
//! empty app looks like a new season.
//!
//! So the test takes the **real** `backend/data/stride.db`, works on a copy
//! of it, and reads the rows out of the copy with the same store the
//! desktop uses. The count it asserts is the count in the original file, so
//! a conversion that drops rows fails the test.
//!
//! The file is not in the repository — `backend/data/` is ignored — so the
//! test says so and does nothing when it is not there.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use rusqlite::Connection;
use stride_api::{Api, CoreBackend};
use stride_desktop::db::{self, Format};
use stride_store::Store;

/// The key the test encrypts with.
const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";

/// The conversions run one at a time.
///
/// A conversion copies a database of a few megabytes and encrypts it, and
/// six of them at once is a few hundred megabytes of page cache and a
/// hundred megabytes of disk. On a machine that is busy with other work
/// that is enough to make the *machine* the reason a test fails, and a test
/// that fails for that reason says nothing about the code.
fn serialise<T>(body: impl FnOnce() -> T) -> T {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let answer = body();
    drop(guard);
    answer
}

/// The real database.
fn real_database() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("backend")
        .join("data")
        .join("stride.db")
}

/// One snapshot of the real database, taken once for the whole test run.
///
/// The database is **live**: the reference app is running and writing to it.
/// Copying the files one by one would copy a database that is being written
/// under the copy — the main file at one moment and the write-ahead log at
/// another — and the copy would be a mixture of two states. SQLite's own
/// backup API takes a snapshot instead.
///
/// The snapshot is taken once and shared: two backups of a database that is
/// being written to, at the same time, is two chances to read it while it
/// changes.
fn snapshot_of_the_real_database() -> Option<PathBuf> {
    static SNAPSHOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            let source = real_database();
            if !source.is_file() {
                return None;
            }
            let directory = std::env::temp_dir().join("stride-desktop-db-snapshot");
            let _ = std::fs::remove_dir_all(&directory);
            std::fs::create_dir_all(&directory).ok()?;
            let target = directory.join("stride.db");
            // The reference app writes to the database while it syncs, so a
            // snapshot can fail. A few tries, and then the test does
            // nothing.
            for _ in 0..5 {
                if stride_desktop::db::take_snapshot(&source, &target).is_ok() {
                    return Some(target);
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            None
        })
        .clone()
}

/// A copy of the snapshot of the real database.
///
/// The snapshot is a static file, so copying it is copying a file and not a
/// database that is being written to.
fn copy_of_the_real_database(directory: &Path) -> Option<PathBuf> {
    let source = snapshot_of_the_real_database()?;
    std::fs::create_dir_all(directory).ok()?;
    let target = directory.join("stride.db");
    let _ = std::fs::remove_file(&target);
    std::fs::copy(&source, &target).ok()?;
    Some(target)
}

/// The number of rows in a table of a plain database.
fn count(path: &Path, table: &str) -> i64 {
    let connection = Connection::open(path).expect("the database opens");
    connection
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("could not count {table}: {error}"))
}

/// A temporary directory that cleans itself up.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stride-desktop-db-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the directory");
    dir
}

/// The real database opens, and every row is still there.
#[test]
fn the_real_database_opens_and_keeps_its_rows() {
    let directory = scratch("real");
    let Some(copy) = copy_of_the_real_database(&directory) else {
        eprintln!(
            "stride: {} is not there; the test does nothing",
            real_database().display()
        );
        return;
    };

    // The file the Python app writes is not encrypted. That is the first
    // thing the desktop has to deal with.
    assert_eq!(
        db::detect_format(&copy).expect("the format"),
        Format::Plaintext,
        "the database on disk is expected to be the plain one"
    );

    let before: Vec<(String, i64)> = [
        "activity",
        "activity_lap",
        "activity_stream",
        "daily_physiology",
        "plan_session",
        "run_fingerprint",
    ]
    .iter()
    .map(|table| (table.to_string(), count(&copy, table)))
    .collect();
    let activities = before
        .iter()
        .find(|(table, _)| table == "activity")
        .map(|(_, rows)| *rows)
        .unwrap_or_default();
    assert!(activities > 0, "the real database has no activities at all");

    // Convert it, the way the desktop does on the first run.
    let report = db::convert_plaintext_to_encrypted(&copy, KEY).expect("the conversion");
    for (table, rows) in &before {
        let copied = report
            .tables
            .iter()
            .find(|(name, _)| name == table)
            .map(|(_, rows)| *rows)
            .unwrap_or_default();
        assert_eq!(copied, *rows, "the table {table} lost rows");
    }
    assert!(
        report.drift.is_empty(),
        "the schema drifted: {:?}",
        report.drift
    );

    // It is encrypted now, and it opens.
    assert_eq!(
        db::detect_format(&copy).expect("the format"),
        Format::Encrypted
    );
    let store = Store::open(&copy, KEY).expect("the store opens the converted database");
    let after = store
        .with_connection(|connection| {
            let mut rows = Vec::new();
            for table in [
                "activity",
                "activity_lap",
                "activity_stream",
                "daily_physiology",
                "plan_session",
                "run_fingerprint",
            ] {
                let mut statement = connection
                    .prepare(&format!("SELECT count(*) FROM {table}"))
                    .map_err(|source| stride_store::StoreError::query(table.to_string(), source))?;
                let count = statement
                    .query_row([], |row| row.get::<_, i64>(0))
                    .map_err(|source| stride_store::StoreError::query(table.to_string(), source))?;
                rows.push((table.to_string(), count));
            }
            Ok(rows)
        })
        .expect("the counts");
    assert_eq!(after, before, "the conversion changed the number of rows");

    eprintln!(
        "stride: the real database has {activities} activities, {} laps, {} streams",
        before[1].1, before[2].1
    );
}

/// The plain file is kept, so nothing is lost if the conversion is wrong.
#[test]
fn the_conversion_keeps_the_plain_file() {
    let directory = scratch("backup");
    let Some(copy) = copy_of_the_real_database(&directory) else {
        eprintln!("stride: the real database is not there; the test does nothing");
        return;
    };
    let before = count(&copy, "activity");
    serialise(|| db::convert_plaintext_to_encrypted(&copy, KEY)).expect("the conversion");
    let backup = copy.with_file_name(format!(
        "{}.plaintext-backup",
        copy.file_name().unwrap_or_default().to_string_lossy()
    ));
    let listing: Vec<String> = std::fs::read_dir(directory)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    format!("{name} ({size})")
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        backup.is_file(),
        "the plain file was not kept; the directory holds {listing:?}; the trace holds {:?}",
        stride_desktop::db::TRACE
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default()
    );
    assert_eq!(count(&backup, "activity"), before, "the backup lost rows");
}

/// The wrong key does not open the database.
#[test]
fn the_wrong_key_is_refused() {
    let directory = scratch("wrongkey");
    let Some(copy) = copy_of_the_real_database(&directory) else {
        eprintln!("stride: the real database is not there; the test does nothing");
        return;
    };
    serialise(|| db::convert_plaintext_to_encrypted(&copy, KEY)).expect("the conversion");
    let wrong = b"ffffffffffffffffffffffffffffffff";
    let error = Store::open(&copy, wrong).expect_err("the wrong key opens the database");
    assert!(
        error.to_string().contains("key"),
        "the error does not say the key is wrong: {error}"
    );
}

/// The database and the plain file it keeps are owner-only.
///
/// The reference hardens its files the same way (`harden_storage`): a year
/// of heart rate, sleep and GPS routes is not a file other processes on this
/// machine should be able to read.
#[cfg(unix)]
#[test]
fn the_database_and_the_plain_file_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let directory = scratch("modes");
    let Some(copy) = copy_of_the_real_database(&directory) else {
        eprintln!("stride: the real database is not there; the test does nothing");
        return;
    };
    serialise(|| db::convert_plaintext_to_encrypted(&copy, KEY)).expect("the conversion");

    for name in ["stride.db", "stride.db.plaintext-backup"] {
        let path = directory.join(name);
        let mode = std::fs::metadata(&path)
            .expect("the file")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "{} is {mode:o}", path.display());
    }
}

/// A database that is not there is created, encrypted, with the schema.
#[test]
fn a_new_database_is_created_encrypted() {
    let directory = scratch("new");
    let path = directory.join("stride.db");
    assert_eq!(
        db::detect_format(&path).expect("the format"),
        Format::Missing
    );
    let (store, conversion) =
        serialise(|| db::open_database(&path, KEY)).expect("the database opens");
    assert!(conversion.is_none(), "nothing had to be converted");
    assert_eq!(
        db::detect_format(&path).expect("the format"),
        Format::Encrypted
    );
    let tables = store
        .with_connection(|connection| {
            let mut statement = connection
                .prepare("SELECT count(*) FROM sqlite_master WHERE type='table'")
                .map_err(|source| {
                    stride_store::StoreError::query("sqlite_master".to_string(), source)
                })?;
            statement
                .query_row([], |row| row.get::<_, i64>(0))
                .map_err(|source| {
                    stride_store::StoreError::query("sqlite_master".to_string(), source)
                })
        })
        .expect("the tables");
    assert!(tables >= 15, "the schema has only {tables} tables");
}

/// The router reads the converted database: the whole path, end to end.
#[test]
fn the_router_reads_the_converted_database() {
    let directory = scratch("router");
    let Some(copy) = copy_of_the_real_database(&directory) else {
        eprintln!("stride: the real database is not there; the test does nothing");
        return;
    };
    serialise(|| db::convert_plaintext_to_encrypted(&copy, KEY)).expect("the conversion");
    let store = Arc::new(Store::open(&copy, KEY).expect("the store opens"));
    let backend = stride_api::store::backend(Arc::clone(&store));
    let api = Api::new(Arc::new(CoreBackend::new(backend)));

    let list = api
        .call("GET", "/api/v1/activities?per_page=5", None)
        .expect("the list answers");
    let total = list["total"].as_i64().expect("the total");
    assert!(total > 0, "the router read no activities");
    let items = list["items"].as_array().expect("the items");
    assert!(!items.is_empty(), "the router returned no activities");
    eprintln!("stride: the router read {total} activities");

    // One activity, with its splits.
    let id = items[0]["id"].as_i64().expect("the id");
    let activity = api
        .call("GET", &format!("/api/v1/activities/{id}"), None)
        .expect("the activity answers");
    assert_eq!(activity["id"].as_i64(), Some(id));
    assert!(activity["distance_m"].as_f64().is_some());
}
