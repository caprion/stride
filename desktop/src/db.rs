//! The database.
//!
//! The desktop opens `backend/data/stride.db` with SQLCipher, through
//! `stride-store`, and it opens the **existing** file: a year of history is
//! in that file and losing it is not an option.
//!
//! ## The database the reference writes is not encrypted
//!
//! The Python app writes a plain SQLite file — `backend/app/db.py` has no
//! cipher in it at all — while `docs/rust-core-spec.md` and the Android
//! workstream both say the database is SQLCipher. The file on this machine
//! starts with `SQLite format 3`, so it is the plain one, and SQLCipher
//! refuses it: `file is not a database`.
//!
//! So the desktop converts it, once, on the first run:
//!
//! * the plain file is read and written into a new, encrypted file, with the
//!   same schema and the same rows;
//! * the plain file is kept as `<name>.plaintext-backup`, so nothing is lost
//!   if the conversion is wrong;
//! * the conversion reports what it found — the tables it copied, the rows
//!   it copied, and any drift between the two schemas.
//!
//! ## The conversion is careful, because it replaces the file
//!
//! The file the desktop converts is a **live** database: the Python app is
//! running and writing to it. So the conversion:
//!
//! * refuses to run while another process has the file open — the copy
//!   would be of a database that is being written under it, and the rename
//!   would take the file away from the writer;
//! * reads the source in **one transaction**, so the rows it copies are a
//!   consistent snapshot and not a mixture of two states;
//! * checks the source's integrity before it starts;
//! * counts the rows it read and the rows it wrote, and refuses to replace
//!   the original if the two do not agree;
//! * writes the backup with SQLite's own backup API, so the backup is a
//!   consistent snapshot too, and not a copy of three files that were
//!   written at three different moments.
//!
//! The key comes from `STRIDE_DB_KEY`, from Azure Key Vault, or — when
//! neither has one — from a local key file next to the database. The last
//! one is a convenience for a machine that has no vault, and it is written
//! with mode 0600.

use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::Connection;
use stride_store::{Store, StoreError};

/// The name of the Key Vault secret that holds the database key.
pub const KEYVAULT_SECRET_ENV: &str = "STRIDE_KEYVAULT_SECRET";

/// The default Key Vault secret name.
pub const KEYVAULT_SECRET_DEFAULT: &str = "stride-db-key";

/// The default Key Vault name.
pub const KEYVAULT_NAME_DEFAULT: &str = "claw-agent";

/// What can go wrong with the database.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// The store could not do it.
    #[error("the database could not be opened: {0}")]
    Store(#[from] StoreError),
    /// A file could not be read or written.
    #[error("could not use the database file at {path}: {source}")]
    Io {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The legacy database could not be read.
    #[error("could not read the legacy database at {path}: {source}")]
    Legacy {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: rusqlite::Error,
    },
    /// The conversion did not work.
    #[error("the conversion failed: {0}")]
    Conversion(String),
    /// Another process is using the database.
    #[error(
        "the database at {path} is open in another process ({pids:?}). \
         Stop it before the desktop converts the file: the conversion replaces it."
    )]
    InUse {
        /// The path.
        path: PathBuf,
        /// The processes that have it open.
        pids: Vec<u32>,
    },
    /// There is no key and none could be made.
    #[error("no database key: {0}")]
    NoKey(String),
}

/// How the database file is encrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// There is no file yet.
    Missing,
    /// A plain SQLite file — what the Python app writes.
    Plaintext,
    /// A SQLCipher file.
    Encrypted,
}

impl Format {
    /// A human-readable name, for the log.
    pub fn name(&self) -> &'static str {
        match self {
            Format::Missing => "missing",
            Format::Plaintext => "plaintext",
            Format::Encrypted => "encrypted",
        }
    }
}

/// What the conversion found.
#[derive(Debug, Clone, Default)]
pub struct ConversionReport {
    /// The tables that were copied, with the number of rows in each.
    pub tables: Vec<(String, i64)>,
    /// The rows that were copied, in total.
    pub rows: i64,
    /// Anything the two schemas disagreed about.
    pub drift: Vec<String>,
}

impl ConversionReport {
    /// A one-line summary, for the log.
    pub fn summary(&self) -> String {
        format!("{} tables, {} rows", self.tables.len(), self.rows)
    }
}

/// A trace of the conversion, for the tests.
///
/// The conversion replaces a file, and a race in it is the difference
/// between a year of history and an empty app. The trace is kept in memory
/// and read when a test fails, so that the failure says what happened rather
/// than only what did not.
pub static TRACE: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Write a line to the trace.
pub fn trace(line: String) {
    if let Ok(mut trace) = TRACE.lock() {
        trace.push(line);
    }
}

/// What is the file?
///
/// A SQLCipher file has a random header, so the only way to tell the two
/// apart is to look for SQLite's own magic at the front of the file.
pub fn detect_format(path: &Path) -> Result<Format, DbError> {
    let mut head = [0u8; 16];
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Format::Missing),
        Err(source) => {
            return Err(DbError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    use std::io::Read;
    let read = file.read(&mut head).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if read < 16 {
        // A file that is too short to be a database: an empty file, or a
        // half-written one. It is not encrypted either, and opening it will
        // fail with a clear error.
        return Ok(Format::Plaintext);
    }
    if &head == b"SQLite format 3\0" {
        Ok(Format::Plaintext)
    } else {
        Ok(Format::Encrypted)
    }
}

/// Open the database, converting it first if it is the plain one.
///
/// The store is migrated after it is opened, exactly as the reference's
/// `init_db` does: the tables are created if they are missing and the
/// additive columns are added.
pub fn open_database(
    path: &Path,
    key: &[u8],
) -> Result<(Store, Option<ConversionReport>), DbError> {
    match detect_format(path)? {
        Format::Missing => {
            let store = Store::open_and_migrate(path, key)?;
            harden(path);
            Ok((store, None))
        }
        Format::Encrypted => {
            let store = Store::open(path, key)?;
            store.migrate()?;
            harden(path);
            Ok((store, None))
        }
        Format::Plaintext => {
            let report = convert_plaintext_to_encrypted(path, key)?;
            let store = Store::open(path, key)?;
            store.migrate()?;
            harden(path);
            Ok((store, Some(report)))
        }
    }
}

/// Make the database and its sidecars owner-only.
///
/// The reference does the same thing in `harden_storage`: a year of heart
/// rate, sleep and GPS routes is not a file other processes on this machine
/// should be able to read.
fn harden(path: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let sidecar = sidecar(path, suffix);
        if sidecar.exists() {
            owner_only(&sidecar);
        }
    }
}

/// Make a file owner-only, where the filesystem agrees.
fn owner_only(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// The processes that have the database open.
///
/// The conversion replaces the file, so it must not do that while somebody
/// else is writing to it: the writer would carry on into the old inode and
/// its rows would go nowhere. Linux says who has a file open in `/proc`.
pub fn users_of(path: &Path) -> Vec<u32> {
    let Ok(canonical) = path.canonicalize() else {
        return Vec::new();
    };
    let mut users = Vec::new();
    let Ok(processes) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    for process in processes.flatten() {
        let name = process.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.is_empty() || !name.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if pid == std::process::id() {
            continue;
        }
        let descriptors = process.path().join("fd");
        let Ok(descriptors) = std::fs::read_dir(descriptors) else {
            continue;
        };
        for descriptor in descriptors.flatten() {
            if let Ok(target) = std::fs::read_link(descriptor.path()) {
                if target == canonical {
                    users.push(pid);
                    break;
                }
            }
        }
    }
    users.sort_unstable();
    users
}

/// Convert the plain database at `path` into an encrypted one, in place.
///
/// The plain file is kept as `<name>.plaintext-backup`.
pub fn convert_plaintext_to_encrypted(
    path: &Path,
    key: &[u8],
) -> Result<ConversionReport, DbError> {
    // Another process writing to the file right now would make the rename
    // take the file away from it: its rows would go into the old inode and
    // nowhere else. So this is refused, and said so.
    let users = users_of(path);
    if !users.is_empty() {
        return Err(DbError::InUse {
            path: path.to_path_buf(),
            pids: users,
        });
    }

    let staged = staged_path(path);
    let backup = backup_path(path);

    // A stale file from an interrupted conversion would be read as the source
    // of truth, so they go first.
    for stale in [
        &staged,
        &backup,
        &sidecar(&staged, "-wal"),
        &sidecar(&staged, "-shm"),
        &sidecar(&backup, "-wal"),
        &sidecar(&backup, "-shm"),
    ] {
        let _ = std::fs::remove_file(stale);
    }

    // The source: the plain database the reference app wrote. It is read
    // inside one transaction below, which is a consistent snapshot of it —
    // in the write-ahead mode a reader sees one moment of the database, even
    // while the reference app is syncing into it.
    let source = Connection::open(path).map_err(|error| DbError::Legacy {
        path: path.to_path_buf(),
        source: error,
    })?;
    quick_check(&source, path)?;

    // The plain file is kept, because the conversion is the one operation
    // here that cannot be undone. It is SQLite's own copy — one file, with
    // the data in it and no write-ahead log next to it — and it is checked
    // against the database it came from.
    take_snapshot(path, &backup)?;
    owner_only(&backup);

    // The encrypted copy.
    let store = Store::open_and_migrate(&staged, key)?;
    let report = store
        .with_connection(|conn| {
            // One read transaction: the rows that are copied are the rows of
            // one moment, and not a mixture of two states.
            source
                .execute_batch("BEGIN;")
                .map_err(|error| StoreError::query("BEGIN".to_string(), error))?;
            let outcome = copy_tables(conn, &source);
            // The transaction is a read: it is rolled back, not committed.
            let _ = source.execute_batch("ROLLBACK;");
            let report = outcome?;
            // Fold the write-ahead log back into the file and turn the log
            // off, so that the file that is renamed into place is complete
            // on its own: one file, with the rows in it. The connection that
            // wrote the rows is the one that folds them back — it holds the
            // lock, and a second connection would be told the database is
            // locked.
            trace(format!(
                "the staged file: {} bytes, a log: {}, files in the directory: {}",
                std::fs::metadata(&staged).map(|m| m.len()).unwrap_or(0),
                sidecar(&staged, "-wal").exists(),
                std::fs::read_dir(staged.parent().unwrap_or(Path::new("/")))
                    .map(|d| d.count())
                    .unwrap_or(0)
            ));
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|source| StoreError::query("wal_checkpoint".to_string(), source))?;
            conn.execute_batch("PRAGMA journal_mode=DELETE;")
                .map_err(|source| {
                    trace(format!(
                        "the staged file is {} {:?}; the directory is {:?}; the log is there: {}",
                        staged.display(),
                        std::fs::metadata(&staged).map(|m| {
                            use std::os::unix::fs::PermissionsExt;
                            format!("{:o}", m.permissions().mode())
                        }),
                        std::fs::metadata(staged.parent().unwrap_or(Path::new("/"))).map(|m| {
                            use std::os::unix::fs::PermissionsExt;
                            format!("{:o}", m.permissions().mode())
                        }),
                        sidecar(&staged, "-wal").exists(),
                    ));
                    StoreError::query("journal_mode=DELETE".to_string(), source)
                })?;
            Ok(report)
        })
        .map_err(|error| DbError::Conversion(error.to_string()))?;
    drop(store);

    // The write-ahead log is gone if the connection closed. If it is still
    // there, the rows are in it and it is not deleted: the file is opened
    // again and the log is folded back in.
    if sidecar(&staged, "-wal").exists() {
        let store = Store::open(&staged, key)?;
        store
            .with_connection(|conn| {
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
                    .map_err(|source| StoreError::query("wal_checkpoint".to_string(), source))
            })
            .map_err(|error| DbError::Conversion(error.to_string()))?;
    }
    let _ = std::fs::remove_file(sidecar(&staged, "-wal"));
    let _ = std::fs::remove_file(sidecar(&staged, "-shm"));

    // The plain file's write-ahead log belongs to the plain file. Leaving it
    // next to an encrypted database would make SQLite try to apply it.
    let _ = std::fs::remove_file(sidecar(path, "-wal"));
    let _ = std::fs::remove_file(sidecar(path, "-shm"));

    std::fs::rename(&staged, path).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    // The database is owner-only, like the file it replaced.
    harden(path);

    // Read it back. The file that is now in place is opened with the same
    // key and its rows are counted; if it does not answer, the plain file
    // goes back where it was, because a desktop that cannot open the
    // database is a nuisance and an empty database is a disaster.
    if let Err(error) = verify_converted(path, key, &report) {
        trace(format!("the converted file did not answer: {error}"));
        let _ = std::fs::remove_file(path);
        let _ = std::fs::copy(&backup, path);
        return Err(DbError::Conversion(format!(
            "the converted database did not answer, and the plain file was put back: {error}"
        )));
    }

    Ok(report)
}

/// Open the converted database and count its rows.
///
/// The counts are the ones the copy reported, so this proves that what is
/// on the disk is what was copied.
fn verify_converted(path: &Path, key: &[u8], report: &ConversionReport) -> Result<(), DbError> {
    let store = Store::open(path, key)?;
    let counts = store
        .with_connection(|conn| {
            let mut counts = Vec::new();
            for (table, rows) in &report.tables {
                let count = count_rows(conn, table)?;
                counts.push((table.clone(), count, *rows));
            }
            Ok(counts)
        })
        .map_err(|error| DbError::Conversion(error.to_string()))?;
    for (table, count, expected) in counts {
        if count != expected {
            return Err(DbError::Conversion(format!(
                "the table {table} has {count} rows and {expected} were copied"
            )));
        }
    }
    Ok(())
}

/// Take a snapshot of a database that may be being written to.
///
/// `VACUUM INTO` is SQLite's own answer to "copy this database while
/// somebody is writing to it": it reads the database in one transaction and
/// writes a **new** file, in the rollback-journal mode, with no write-ahead
/// log next to it. The snapshot is one self-contained file, so everything
/// that reads it afterwards reads a database that is not changing.
///
/// The copy the backup API makes is not used here on purpose: the
/// destination inherits the source's write-ahead mode, and a file whose
/// header says "write-ahead log" but which has no log is a file that other
/// tools read differently.
pub fn take_snapshot(source: &Path, target: &Path) -> Result<(), DbError> {
    // The database may be written to while it is snapshotted, so the
    // snapshot is checked against the source and taken again if they do not
    // agree. Three tries, and then it is an error rather than a snapshot
    // that is half of two states.
    let mut last = String::new();
    for attempt in 1..=3 {
        let _ = std::fs::remove_file(target);
        let origin = Connection::open(source).map_err(|error| DbError::Legacy {
            path: source.to_path_buf(),
            source: error,
        })?;
        let quoted = target.to_string_lossy().replace('\'', "''");
        origin
            .execute_batch(&format!("VACUUM INTO '{quoted}'"))
            .map_err(|error| DbError::Legacy {
                path: target.to_path_buf(),
                source: error,
            })?;
        if !target.is_file() {
            return Err(DbError::Conversion(format!(
                "the snapshot at {} was not written",
                target.display()
            )));
        }
        match snapshot_matches(source, target) {
            Ok(true) => {
                trace(format!(
                    "the snapshot of {} matches the database (attempt {attempt})",
                    source.display()
                ));
                return Ok(());
            }
            Ok(false) => {
                last = format!("the snapshot of {} does not match it", source.display());
                trace(format!("{last} (attempt {attempt})"));
            }
            Err(error) => return Err(error),
        }
    }
    Err(DbError::Conversion(last))
}

/// Do the snapshot and the database hold the same rows?
///
/// The snapshot is a copy of a database that may have been written to while
/// it was taken. The check is the row count of every table, which is what a
/// torn copy changes first.
fn snapshot_matches(source: &Path, snapshot: &Path) -> Result<bool, DbError> {
    let origin = table_counts(source)?;
    let copy = table_counts(snapshot)?;
    if origin.is_empty() {
        // A database with no tables is not the athlete's history, and it is
        // not a copy of anything either. Two empty lists are not a match.
        return Err(DbError::Conversion(format!(
            "the database at {} has no tables in it",
            source.display()
        )));
    }
    Ok(origin == copy)
}

/// The row count of every table of a database.
fn table_counts(path: &Path) -> Result<Vec<(String, i64)>, DbError> {
    let connection = Connection::open(path).map_err(|error| DbError::Legacy {
        path: path.to_path_buf(),
        source: error,
    })?;
    let mut statement = connection
        .prepare(
            "SELECT name FROM main.sqlite_master WHERE type='table' \
             AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .map_err(|error| DbError::Legacy {
            path: path.to_path_buf(),
            source: error,
        })?;
    let mut tables = Vec::new();
    for table in statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| DbError::Legacy {
            path: path.to_path_buf(),
            source: error,
        })?
    {
        // A row that cannot be read is an error, not a table that is not
        // there: swallowing it would make an empty database look like a
        // database with tables in it.
        tables.push(table.map_err(|error| DbError::Legacy {
            path: path.to_path_buf(),
            source: error,
        })?);
    }
    let mut counts = Vec::with_capacity(tables.len());
    for table in tables {
        let count: i64 = connection
            .query_row(
                &format!("SELECT count(*) FROM main.\"{table}\""),
                [],
                |row| row.get(0),
            )
            .map_err(|error| DbError::Legacy {
                path: path.to_path_buf(),
                source: error,
            })?;
        counts.push((table, count));
    }
    Ok(counts)
}

/// Check that the source is a database that can be read.
fn quick_check(source: &Connection, path: &Path) -> Result<(), DbError> {
    let mut statement = source
        .prepare("PRAGMA quick_check")
        .map_err(|error| DbError::Legacy {
            path: path.to_path_buf(),
            source: error,
        })?;
    let verdict = statement
        .query_row([], |row| row.get::<_, String>(0))
        .map_err(|error| DbError::Legacy {
            path: path.to_path_buf(),
            source: error,
        })?;
    if verdict != "ok" {
        return Err(DbError::Conversion(format!(
            "the database at {} is not sound: {verdict}",
            path.display()
        )));
    }
    Ok(())
}

/// Copy every table of the legacy database into the encrypted one.
///
/// The columns are matched by name, not by position: the two schemas have
/// the same columns in a different order in three tables, and a copy by
/// position would put a timestamp in a distance column.
///
/// The two databases are read and written by two connections, because
/// SQLCipher will not attach a plain database to an encrypted one — it reads
/// the plain header as an encrypted page and gives up. So the rows are read
/// from the plain connection and written to the encrypted one, one row at a
/// time, with the values passed through as they are.
///
/// The read happens inside one transaction, so the rows are a snapshot of
/// one moment and not a mixture of two: the database may be being written
/// to while this runs.
fn copy_tables(
    conn: &rusqlite::Connection,
    source: &rusqlite::Connection,
) -> Result<ConversionReport, stride_store::StoreError> {
    let mut report = ConversionReport::default();

    // The foreign keys are off while the tables are copied: they arrive in
    // alphabetical order, so a child arrives before its parent. They are
    // checked afterwards, with `foreign_key_check`.
    conn.execute_batch("PRAGMA foreign_keys=OFF;")
        .map_err(|source| StoreError::query("PRAGMA foreign_keys=OFF".to_string(), source))?;

    copy_tables_in_transaction(conn, source, &mut report)?;

    // The rows that were written are counted again, in the file they were
    // written to. If the two counts disagree, the file is not replaced.
    for (table, rows) in &report.tables {
        let written = count_rows(conn, table)?;
        if written != *rows {
            return Err(StoreError::query(
                format!("the table {table}"),
                rusqlite::Error::InvalidParameterName(format!(
                    "{rows} rows were read and {written} were written"
                )),
            ));
        }
    }

    // The foreign keys are checked after the copy, not during it: the tables
    // arrive in alphabetical order and the children arrive before their
    // parents.
    let violations = conn
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut statement| {
            let mut rows = statement.query([])?;
            let mut count = 0i64;
            while rows.next()?.is_some() {
                count += 1;
            }
            Ok(count)
        })
        .unwrap_or(0);
    if violations > 0 {
        report
            .drift
            .push(format!("{violations} rows break a foreign key"));
    }

    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|source| StoreError::query("PRAGMA foreign_keys=ON".to_string(), source))?;

    Ok(report)
}

/// The copy itself, inside the transaction.
fn copy_tables_in_transaction(
    conn: &rusqlite::Connection,
    source: &rusqlite::Connection,
    report: &mut ConversionReport,
) -> Result<(), stride_store::StoreError> {
    let tables = strings(
        source,
        "SELECT name FROM main.sqlite_master WHERE type='table' \
         AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;

    for table in tables {
        let target_columns = columns(conn, &table)?;
        let source_columns = columns(source, &table)?;
        if target_columns.is_empty() {
            report
                .drift
                .push(format!("the table {table} is not in the current schema"));
            continue;
        }
        let shared: Vec<String> = target_columns
            .iter()
            .filter(|column| source_columns.contains(column))
            .cloned()
            .collect();
        for column in &source_columns {
            if !target_columns.contains(column) {
                report.drift.push(format!(
                    "the column {table}.{column} is not in the current schema"
                ));
            }
        }
        for column in &target_columns {
            if !source_columns.contains(column) {
                report.drift.push(format!(
                    "the column {table}.{column} is not in the old database"
                ));
            }
        }
        if shared.is_empty() {
            report
                .drift
                .push(format!("the table {table} has no column in common"));
            continue;
        }

        let list = shared
            .iter()
            .map(|column| format!("\"{column}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let select = format!("SELECT {list} FROM main.\"{table}\"");
        let insert = format!("INSERT INTO main.\"{table}\" ({list}) VALUES ({})", {
            let mut placeholders = Vec::with_capacity(shared.len());
            for index in 1..=shared.len() {
                placeholders.push(format!("?{index}"));
            }
            placeholders.join(", ")
        });

        let mut statement = source
            .prepare(&select)
            .map_err(|error| StoreError::query(select.clone(), error))?;
        let mut rows = statement
            .query([])
            .map_err(|error| StoreError::query(select.clone(), error))?;
        let mut prepared = conn
            .prepare(&insert)
            .map_err(|error| StoreError::query(insert.clone(), error))?;

        let mut copied = 0i64;
        while let Some(row) = rows
            .next()
            .map_err(|error| StoreError::query(select.clone(), error))?
        {
            let mut values = Vec::with_capacity(shared.len());
            for index in 0..shared.len() {
                let value = row
                    .get::<_, rusqlite::types::Value>(index)
                    .map_err(|error| StoreError::query(select.clone(), error))?;
                values.push(value);
            }
            prepared
                .execute(rusqlite::params_from_iter(values.iter()))
                .map_err(|error| StoreError::query(insert.clone(), error))?;
            copied += 1;
        }
        report.rows += copied;
        report.tables.push((table, copied));
    }
    Ok(())
}

/// The number of rows in a table.
fn count_rows(conn: &rusqlite::Connection, table: &str) -> Result<i64, stride_store::StoreError> {
    let statement = format!("SELECT count(*) FROM main.\"{table}\"");
    conn.query_row(&statement, [], |row| row.get::<_, i64>(0))
        .map_err(|error| StoreError::query(statement.clone(), error))
}

/// The columns of a table.
fn columns(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<Vec<String>, stride_store::StoreError> {
    let statement = format!("PRAGMA main.table_info(\"{table}\")");
    let mut prepared = conn
        .prepare(&statement)
        .map_err(|source| StoreError::query(statement.clone(), source))?;
    let rows = prepared
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|source| StoreError::query(statement.clone(), source))?;
    let mut columns = Vec::new();
    for row in rows {
        match row {
            Ok(column) => columns.push(column),
            Err(error) => {
                return Err(StoreError::query(statement.clone(), error));
            }
        }
    }
    Ok(columns)
}

/// The strings of a query that returns one column.
fn strings(
    conn: &rusqlite::Connection,
    sql: &str,
) -> Result<Vec<String>, stride_store::StoreError> {
    let mut prepared = conn
        .prepare(sql)
        .map_err(|source| StoreError::query(sql.to_string(), source))?;
    let rows = prepared
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| StoreError::query(sql.to_string(), source))?;
    let mut values = Vec::new();
    for row in rows {
        match row {
            Ok(value) => values.push(value),
            Err(error) => return Err(StoreError::query(sql.to_string(), error)),
        }
    }
    if values.is_empty() {
        return Err(StoreError::query(
            sql.to_string(),
            rusqlite::Error::InvalidQuery,
        ));
    }
    Ok(values)
}

/// The path the encrypted database is written to before it is put in place.
fn staged_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".encrypted");
    path.with_file_name(name)
}

/// The path the plain database is kept at.
fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".plaintext-backup");
    path.with_file_name(name)
}

/// The write-ahead log of a database.
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// Find the key the database is encrypted with.
///
/// The order is the order the constitution gives: the environment, then Azure
/// Key Vault, then — only because there was nothing else — a local key file
/// beside the database.
pub fn resolve_key(
    explicit: Option<Vec<u8>>,
    data_dir: &Path,
) -> Result<(Vec<u8>, crate::config::KeySource), DbError> {
    if let Some(key) = explicit {
        if !key.is_empty() {
            return Ok((key, crate::config::KeySource::Environment));
        }
    }
    if let Some(key) = key_from_vault()? {
        return Ok((key, crate::config::KeySource::KeyVault));
    }
    let path = data_dir.join("stride-desktop.key");
    let key = read_or_create_key(&path)?;
    Ok((key, crate::config::KeySource::LocalFile))
}

/// Read the key from Azure Key Vault, with the Azure CLI.
///
/// The CLI is invoked with an argument list and without a shell, so a vault
/// or a secret name cannot be interpolated into a command.
pub fn key_from_vault() -> Result<Option<Vec<u8>>, DbError> {
    let vault = std::env::var("STRIDE_KEYVAULT_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| KEYVAULT_NAME_DEFAULT.to_string());
    let secret = std::env::var(KEYVAULT_SECRET_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| KEYVAULT_SECRET_DEFAULT.to_string());

    let output = Command::new("az")
        .args([
            "keyvault",
            "secret",
            "show",
            "--vault-name",
            &vault,
            "--name",
            &secret,
            "--query",
            "value",
            "-o",
            "tsv",
        ])
        .output();
    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() || value.contains('\n') {
        return Ok(None);
    }
    Ok(Some(value.into_bytes()))
}

/// The local key file: read it, or make it.
fn read_or_create_key(path: &Path) -> Result<Vec<u8>, DbError> {
    if let Ok(raw) = std::fs::read_to_string(path) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.as_bytes().to_vec());
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DbError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let key = crate::state::random_token();
    std::fs::write(path, &key).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(key.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_missing() {
        let path = std::env::temp_dir().join("stride-desktop-absent.db");
        let _ = std::fs::remove_file(&path);
        assert_eq!(detect_format(&path).expect("the format"), Format::Missing);
    }

    #[test]
    fn a_plain_file_is_plaintext() {
        let path = std::env::temp_dir().join("stride-desktop-plain.db");
        std::fs::write(&path, b"SQLite format 3\0 and then some").expect("the file");
        assert_eq!(detect_format(&path).expect("the format"), Format::Plaintext);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_encrypted_file_is_not_plaintext() {
        let path = std::env::temp_dir().join("stride-desktop-encrypted.db");
        std::fs::write(&path, [0x7fu8; 32]).expect("the file");
        assert_eq!(detect_format(&path).expect("the format"), Format::Encrypted);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_this_process_has_open_is_not_a_user() {
        let path = std::env::temp_dir().join("stride-desktop-open.db");
        std::fs::write(&path, b"SQLite format 3\0").expect("the file");
        let file = std::fs::File::open(&path).expect("the file");
        assert!(
            !users_of(&path).contains(&std::process::id()),
            "the process is reported as a user of its own file"
        );
        drop(file);
        let _ = std::fs::remove_file(&path);
    }
}
