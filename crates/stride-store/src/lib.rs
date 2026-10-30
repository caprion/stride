//! `stride-store` — the encrypted SQLite database.
//!
//! The schema is the schema the Python app already has, ported exactly: the
//! desktop's snapshot has to open on the phone. `tests/schema_matches_python.rs`
//! proves it, by reading the Python sources at test time.
//!
//! ```no_run
//! use stride_store::Store;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), stride_store::StoreError> {
//! let store = Store::open(Path::new("stride.db"), b"a 32 byte key.............")?;
//! store.migrate()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## The ports
//!
//! | the Python schema | here |
//! |---|---|
//! | `DateTime(timezone=True)` | `chrono::DateTime<Utc>`, stored as the text `YYYY-MM-DD HH:MM:SS.ffffff` |
//! | `JSON` | `serde_json::Value`, stored as text |
//!
//! The datetime format is SQLAlchemy's own storage format, to the microsecond,
//! so a database written by either implementation opens in the other.
//!
//! ## The key
//!
//! The key comes from the caller and from nowhere else: the Android Keystore
//! on the phone, Key Vault on the desktop. A key of exactly 32 bytes is used
//! as a raw key (`PRAGMA key = "x'…'"`); any other length is used as a
//! passphrase. The store never reads a key from the environment, from a file
//! or from a prompt.
//!
//! ## The cipher
//!
//! SQLCipher 4.5.3, vendored: OpenSSL is built from source, so there is no
//! system dependency and the APK carries its own cryptography. Both ends of
//! the project use the same vendored library, so the desktop's snapshot and
//! the phone's database are the same file format.

pub mod datetimes;
pub mod error;
pub mod json;
pub mod model;
pub mod repo;
pub mod schema;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OpenFlags};

pub use error::{Result, StoreError};
pub use model::*;

/// The database.
///
/// One connection, behind a lock: the API reads and the sync writes, and
/// SQLite serialises them anyway.
pub struct Store {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Store {
    /// Open the database at `path`, with `key`.
    ///
    /// The file is created if it is not there. The key must be right: the
    /// store checks it by reading the schema before it returns, so a wrong key
    /// is an error here rather than a mystery later.
    ///
    /// This does not migrate. Call [`Store::migrate`] — it is idempotent — or
    /// use [`Store::open_and_migrate`].
    pub fn open(path: &Path, key: &[u8]) -> Result<Store> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey {
                reason: "the key is empty".to_string(),
            });
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
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
        conn.execute_batch(&key_statement(key)?)
            .map_err(|source| StoreError::Open {
                path: path.to_path_buf(),
                source,
            })?;

        // Prove the key works before anybody reads the schema.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|source| key_error(path, source))?;

        // The same pragmas the Python app sets, in the same order.
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

    /// Open the database and bring it up to the current schema.
    pub fn open_and_migrate(path: &Path, key: &[u8]) -> Result<Store> {
        let store = Store::open(path, key)?;
        store.migrate()?;
        Ok(store)
    }

    /// Bring the database up to the current schema.
    ///
    /// Idempotent: running it twice changes nothing and does not fail.
    ///
    /// It also removes the synthetic activities an older installation left
    /// behind. The demo dataset is gone from the product, but the rows it
    /// seeded are still in some databases, and they were seeded at the
    /// athlete's own coordinates — so they have to go, exactly once, on the
    /// first open of the new build. The reference does the same thing at
    /// every start-up (`services.persistence.purge_synthetic_activities`).
    pub fn migrate(&self) -> Result<()> {
        let conn = self.lock()?;
        schema::migrate(&conn)?;
        let removed = crate::repo::activity::purge_synthetic_activities(&conn)?;
        if removed > 0 {
            eprintln!(
                "stride-store: removed {removed} synthetic activities left over from an older install"
            );
        }
        Ok(())
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
    /// not cover — a `PRAGMA`, a diagnostic query.
    pub fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.lock()?;
        f(&conn)
    }
}

/// The `PRAGMA key` statement for a key.
///
/// A 32-byte key is a raw key — the bytes are used as they are, with no key
/// derivation, which is what a key from the Android Keystore or from Key
/// Vault is. Anything else is a passphrase, and must be text.
fn key_statement(key: &[u8]) -> Result<String> {
    if key.len() == 32 {
        let mut hex = String::with_capacity(64);
        for byte in key {
            hex.push_str(&format!("{byte:02x}"));
        }
        return Ok(format!("PRAGMA key = \"x'{hex}'\";"));
    }

    let text = std::str::from_utf8(key).map_err(|_| StoreError::InvalidKey {
        reason: format!(
            "the key is {} bytes: it is neither a 32-byte raw key nor text",
            key.len()
        ),
    })?;
    let escaped = text.replace('\'', "''");
    Ok(format!("PRAGMA key = '{escaped}';"))
}

/// Turn a SQLite error into a key error when it is one.
fn key_error(path: &Path, source: rusqlite::Error) -> StoreError {
    let message = source.to_string();
    if message.contains("file is not a database") || message.contains("file is encrypted") {
        return StoreError::InvalidKey {
            reason: format!("the key does not open {}: {message}", path.display()),
        };
    }
    StoreError::Open {
        path: path.to_path_buf(),
        source,
    }
}
