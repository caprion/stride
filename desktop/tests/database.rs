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
//! The live-data check is opt-in. Routine agent runs use the small synthetic
//! fixtures elsewhere in the suite, not a copy of the athlete's full history.
//!
//! **One conversion, shared.** A conversion copies a database of a few
//! megabytes and encrypts it, and six of them at once is a few hundred
//! megabytes of page cache and a hundred megabytes of disk. On a machine
//! that is busy with other work that is enough to make the *machine* the
//! reason a test fails, and a test that fails for that reason says nothing
//! about the code. One test owns the conversion and runs all the real-data
//! assertions against it. Its temporary files are removed on return or panic.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rusqlite::Connection;
use stride_api::{Api, CoreBackend};
use stride_store::Store;

use stride_desktop::db::{self, ConversionReport, Format};

/// The key the test encrypts with.
const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";

/// The tables the test counts.
const TABLES: [&str; 6] = [
    "activity",
    "activity_lap",
    "activity_stream",
    "daily_physiology",
    "plan_session",
    "run_fingerprint",
];

/// The real database.
fn real_database() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("backend")
        .join("data")
        .join("stride.db")
}

/// One copy of the real database, taken once for the whole run.
///
/// The database is **live**: the reference app is running and writing to
/// it. Copying the files one by one would copy a database that is being
/// written under the copy — the main file at one moment and the write-ahead
/// log at another — and the copy would be a mixture of two states. SQLite's
/// own backup API takes a snapshot instead.
///
/// The snapshot is taken once and shared: two backups of a database that is
/// being written to, at the same time, is two chances to read it while it
/// changes.
fn snapshot_of_the_real_database(directory: &Path) -> PathBuf {
    let source = real_database();
    let target = directory.join("stride.db");
    for attempt in 0..5 {
        match db::take_snapshot(&source, &target) {
            Ok(()) => return target,
            Err(error) if attempt == 4 => panic!("the database snapshot failed: {error}"),
            Err(error) => {
                eprintln!("stride: retrying the database snapshot: {error}");
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
    unreachable!("the final snapshot attempt returns or panics")
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

/// A temporary directory for one attempt.
///
/// The name carries the process and a counter, so **two attempts never share
/// a directory**. A failed attempt that left a directory behind must not be
/// cleaned up by the next one: the next one would delete the files under a
/// conversion that is still running, and the symptom of that is a database
/// that is suddenly not there.
struct FixtureDirectory {
    path: PathBuf,
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            eprintln!(
                "stride: could not remove test fixture {}: {error}",
                self.path.display()
            );
        }
    }
}

fn scratch(name: &str) -> FixtureDirectory {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    loop {
        let serial = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "stride-desktop-db-{name}-{}-{serial}",
            std::process::id()
        ));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => return FixtureDirectory { path },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("could not create fixture {}: {error}", path.display()),
        }
    }
}

/// One conversion of the real database, shared by the tests.
struct Converted {
    /// Where it happened.
    directory: FixtureDirectory,
    /// The encrypted database it produced.
    database: PathBuf,
    /// The plain copy it kept.
    backup: PathBuf,
    /// What the conversion reported.
    report: ConversionReport,
    /// The row counts before the conversion.
    before: Vec<(String, i64)>,
}

/// Convert the real database, once.
fn converted() -> Converted {
    assert!(
        real_database().is_file(),
        "the opt-in real database is missing"
    );
    let directory = scratch("converted");
    let copy = snapshot_of_the_real_database(&directory.path);
    let before = TABLES
        .iter()
        .map(|table| (table.to_string(), count(&copy, table)))
        .collect();
    let report = db::convert_plaintext_to_encrypted(&copy, KEY).expect("the snapshot converts");
    let backup = copy.with_file_name(format!(
        "{}.plaintext-backup",
        copy.file_name().unwrap_or_default().to_string_lossy()
    ));
    Converted {
        database: copy,
        backup,
        directory,
        report,
        before,
    }
}

/// The real database opens, and every row is still there.
#[test]
#[ignore = "reads a live-data snapshot; run explicitly with --ignored"]
fn the_real_database_opens_and_keeps_its_rows() {
    let converted = converted();

    let activities = converted
        .before
        .iter()
        .find(|(table, _)| table == "activity")
        .map(|(_, rows)| *rows)
        .unwrap_or_default();
    assert!(activities > 0, "the real database has no activities at all");

    // Every row that was there is there now.
    for (table, rows) in &converted.before {
        let copied = converted
            .report
            .tables
            .iter()
            .find(|(name, _)| name == table)
            .map(|(_, rows)| *rows)
            .unwrap_or_default();
        assert_eq!(copied, *rows, "the table {table} lost rows");
    }
    assert!(
        converted.report.drift.is_empty(),
        "the schema drifted: {:?}",
        converted.report.drift
    );

    // It is encrypted now, and it opens.
    assert_eq!(
        db::detect_format(&converted.database).expect("the format"),
        Format::Encrypted
    );
    let store = Store::open(&converted.database, KEY).expect("the store opens");
    let after = store
        .with_connection(|connection| {
            let mut rows = Vec::new();
            for table in TABLES {
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
    assert_eq!(after, converted.before, "the conversion changed the rows");

    eprintln!(
        "stride: the real database has {activities} activities, {} laps, {} streams",
        converted.before[1].1, converted.before[2].1
    );

    the_database_on_disk_is_the_plain_one(&converted);
    the_conversion_keeps_the_plain_file(&converted);
    the_wrong_key_is_refused(&converted);
    #[cfg(unix)]
    the_database_and_the_plain_file_are_owner_only(&converted);
    the_router_reads_the_converted_database(&converted);
}

/// The file the Python app writes is not encrypted, and the desktop deals
/// with it.
fn the_database_on_disk_is_the_plain_one(converted: &Converted) {
    let source = real_database();
    if !source.is_file() {
        return;
    }
    assert_eq!(
        db::detect_format(&source).expect("the format"),
        Format::Plaintext,
        "the database on disk is expected to be the plain one"
    );
    assert_eq!(
        db::detect_format(&converted.database).expect("the format"),
        Format::Encrypted,
        "the converted database is expected to be the encrypted one"
    );
}

/// The plain file is kept, so nothing is lost if the conversion is wrong.
fn the_conversion_keeps_the_plain_file(converted: &Converted) {
    let listing: Vec<String> = std::fs::read_dir(&converted.directory.path)
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
        converted.backup.is_file(),
        "the plain file was not kept; the directory holds {listing:?}"
    );
    let before = converted
        .before
        .iter()
        .find(|(table, _)| table == "activity")
        .map(|(_, rows)| *rows)
        .unwrap_or_default();
    assert_eq!(
        count(&converted.backup, "activity"),
        before,
        "the backup lost rows"
    );
}

/// The wrong key does not open the database.
fn the_wrong_key_is_refused(converted: &Converted) {
    let wrong = b"ffffffffffffffffffffffffffffffff";
    let error =
        Store::open(&converted.database, wrong).expect_err("the wrong key opens the database");
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
fn the_database_and_the_plain_file_are_owner_only(converted: &Converted) {
    use std::os::unix::fs::PermissionsExt;

    for path in [&converted.database, &converted.backup] {
        let mode = std::fs::metadata(path)
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
    let path = directory.path.join("stride.db");
    assert_eq!(
        db::detect_format(&path).expect("the format"),
        Format::Missing
    );
    let (store, conversion) = db::open_database(&path, KEY).expect("the database opens");
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
fn the_router_reads_the_converted_database(converted: &Converted) {
    let store = Arc::new(Store::open(&converted.database, KEY).expect("the store opens"));
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

#[test]
fn temporary_database_files_are_removed_after_success() {
    let path = {
        let directory = scratch("cleanup");
        std::fs::write(directory.path.join("stride.db"), b"fixture").expect("the fixture");
        directory.path.clone()
    };
    assert!(!path.exists(), "the test fixture was left behind");
}

#[test]
fn temporary_database_files_are_removed_after_panic() {
    let directory = scratch("cleanup-panic");
    let path = directory.path.clone();
    let result = std::panic::catch_unwind(move || {
        let _directory = directory;
        panic!("a failed database test");
    });
    assert!(result.is_err());
    assert!(!path.exists(), "the failed test left its fixture behind");
}
