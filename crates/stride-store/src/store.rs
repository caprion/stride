//! The database.
//!
//! One encrypted SQLite file, behind one lock. The key comes from the caller
//! and goes nowhere else — see [`crate::key`].
//!
//! ```no_run
//! use std::path::Path;
//! use stride_store::{DatabaseKey, Store};
//!
//! # fn main() -> Result<(), stride_store::StoreError> {
//! let key = DatabaseKey::from_text("the key the caller holds")?;
//! let store = Store::open_and_migrate(Path::new("stride.db"), &key)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## The connection
//!
//! One connection, behind a `Mutex`: the API reads and the sync writes, and
//! SQLite serialises them anyway. The lock is held for the length of one
//! operation, so a long read does not block a short write for longer than it
//! has to.
//!
//! ## The pragmas
//!
//! The same pragmas the old implementation set, in the same order:
//! write-ahead logging, `synchronous=NORMAL`, foreign keys **on**, and a
//! thirty-second busy timeout. The foreign keys matter: the schema declares
//! them, and SQLite ignores them unless they are switched on.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OpenFlags, TransactionBehavior};

use crate::error::{Result, StoreError};
use crate::key::{key_failure, DatabaseKey};
use crate::logging;
use crate::schema;

/// The database.
///
/// The key is applied when the connection is opened and is not kept: there is
/// no field for it, so it cannot be read back out of a `Store`.
pub struct Store {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The path, and nothing else. There is no key to print.
        f.debug_struct("Store")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Store {
    /// Open the database at `path`, with `key`.
    ///
    /// The file is created if it is not there. The key must be right: the
    /// store checks it by reading the schema before it returns, so a wrong
    /// key is an error here rather than a mystery later.
    ///
    /// This does not migrate. Call [`Store::migrate`] — it is idempotent — or
    /// use [`Store::open_and_migrate`].
    pub fn open(path: &Path, key: &DatabaseKey) -> Result<Store> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|source| StoreError::io(parent.to_path_buf(), source))?;
            }
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|source| StoreError::Open {
            path: path.to_path_buf(),
            source,
        })?;

        // The key has to be the first thing the connection sees: SQLCipher
        // decrypts the file from here on.
        key.apply(&conn)?;

        // Prove the key works before anybody reads the schema. A file that
        // does not open is a wrong key or a file that is not a database, and
        // both are the caller's problem to hear about now.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|source| key_failure(&source))?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=30000;",
        )
        .map_err(|source| StoreError::Open {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(Store {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
        })
    }

    /// Open the database with a key given as bytes.
    ///
    /// The same as [`Store::open`], for a caller that holds the key as bytes.
    /// The bytes are copied into a [`DatabaseKey`] and are not kept by the
    /// store.
    pub fn open_with_bytes(path: &Path, key: &[u8]) -> Result<Store> {
        let key = DatabaseKey::from_bytes(key.to_vec())?;
        Store::open(path, &key)
    }

    /// Open the database and bring it up to the current schema.
    pub fn open_and_migrate(path: &Path, key: &DatabaseKey) -> Result<Store> {
        let store = Store::open(path, key)?;
        store.migrate()?;
        Ok(store)
    }

    /// Open the database with a key given as bytes, and bring it up to the
    /// current schema.
    pub fn open_and_migrate_with_bytes(path: &Path, key: &[u8]) -> Result<Store> {
        let key = DatabaseKey::from_bytes(key.to_vec())?;
        Store::open_and_migrate(path, &key)
    }

    /// Bring the database up to the current schema.
    ///
    /// Idempotent: running it twice changes nothing and does not fail.
    pub fn migrate(&self) -> Result<()> {
        let conn = self.lock()?;
        schema::migrate(&conn)?;
        drop(conn);
        self.log("the schema is up to date");
        Ok(())
    }

    /// The version of the schema in the database.
    pub fn schema_version(&self) -> Result<i64> {
        let conn = self.lock()?;
        schema::schema_version(&conn)
    }

    /// The path of the database file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The lock on the connection.
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }

    /// Run something against the connection.
    ///
    /// The connection is held for the duration, so the caller cannot use the
    /// store at the same time. It is here for the things the repositories do
    /// not cover — a `PRAGMA`, a diagnostic query, a transaction that spans
    /// more than one repository.
    pub fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.lock()?;
        f(&conn)
    }

    /// Delete everything: every run, every wellness day, the plan and every
    /// note.
    ///
    /// **One transaction.** The data goes or it does not: a delete that
    /// stopped halfway would leave the log in a state no athlete had ever
    /// been in.
    ///
    /// The sync cursors go with it. They point at runs that are no longer
    /// there, and a cursor that outlives its data would ask Garmin for
    /// nothing and leave the log empty.
    ///
    /// Returns how many rows were deleted.
    pub fn delete_everything(&self) -> Result<i64> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let mut deleted = 0i64;
        // The children first: the schema's foreign keys point at the
        // parents. Every statement is written out — the tables are the
        // frozen ones, and a statement that is built at runtime is a
        // statement nobody read.
        let removed = transaction
            .execute("DELETE FROM activity_samples", [])
            .map_err(|source| StoreError::query("activity_samples", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM activities", [])
            .map_err(|source| StoreError::query("activities", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM wellness_daily", [])
            .map_err(|source| StoreError::query("wellness_daily", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM plan_sessions", [])
            .map_err(|source| StoreError::query("plan_sessions", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM plan_blocks", [])
            .map_err(|source| StoreError::query("plan_blocks", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM notes", [])
            .map_err(|source| StoreError::query("notes", source))?;
        deleted += removed as i64;
        let removed = transaction
            .execute("DELETE FROM sync_state", [])
            .map_err(|source| StoreError::query("sync_state", source))?;
        deleted += removed as i64;

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(deleted)
    }

    /// The size of the database file, in bytes.
    ///
    /// `None` when the file cannot be read: a size that is not there is
    /// not a zero.
    pub fn size_bytes(&self) -> Option<u64> {
        std::fs::metadata(&self.path)
            .ok()
            .map(|metadata| metadata.len())
    }

    /// Say something in the store's log.
    pub(crate) fn log(&self, message: &str) {
        logging::log(&format!("{}: {message}", self.path.display()));
    }
}
