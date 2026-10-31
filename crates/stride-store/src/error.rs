//! Errors.
//!
//! Every failure says **where** it happened: which table, which column, which
//! step. `StoreError::Query { context: "activities.local_date" }` says where;
//! `StoreError::Query { context: "the database" }` says nothing.
//!
//! No error ever carries the database key. The key is applied to the
//! connection before any of this runs, and the errors it can cause are
//! rewritten into [`StoreError::InvalidKey`] with a fixed reason — see
//! [`crate::key`].

use std::path::PathBuf;

use thiserror::Error;

/// The result type used everywhere in this crate.
pub type Result<T> = std::result::Result<T, StoreError>;

/// Everything that can go wrong in the store.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The database file could not be opened.
    #[error("could not open the database at {path}: {source}")]
    Open {
        /// The path we tried to open.
        path: PathBuf,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },

    /// The key was rejected: the wrong key, or the file is not an encrypted
    /// database. The reason never contains the key.
    #[error("the database key was rejected: {reason}")]
    InvalidKey {
        /// Why the key was rejected. A fixed phrase, never the key.
        reason: String,
    },

    /// A query failed.
    #[error("query failed ({context}): {source}")]
    Query {
        /// What the query was about — a table, a column, a method.
        context: String,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },

    /// A migration step failed.
    #[error("migration failed at step {step}: {source}")]
    Migration {
        /// The migration step that failed.
        step: String,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },

    /// The store's lock was poisoned by a panic in another thread.
    #[error("the database lock was poisoned by a panic in another thread")]
    Poisoned,

    /// A file operation failed.
    #[error("could not use {path}: {source}")]
    Io {
        /// The file we were reading or writing.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The stored blob does not match its hash.
    #[error("the stored blob does not match its hash: {reason}")]
    BlobMismatch {
        /// What did not match.
        reason: String,
    },

    /// An export could not be read, or is not a Stride export.
    #[error("the export at {path} cannot be read: {reason}")]
    Export {
        /// The export's path.
        path: PathBuf,
        /// Why it cannot be read.
        reason: String,
    },

    /// The export's schema version is not the one this build reads.
    ///
    /// The import refuses rather than guessing: a schema that moved under the
    /// reader's feet is how a year of history is lost quietly.
    #[error("the export's schema version is {found}, and this build reads {expected}")]
    SchemaVersion {
        /// The version in the export.
        found: i64,
        /// The version this build reads.
        expected: i64,
    },

    /// A rule the store keeps was broken.
    #[error("{0}")]
    Invariant(String),
}

impl From<rusqlite::Error> for StoreError {
    /// A SQLite error with no further context.
    ///
    /// The context is "the database": enough to say where the error came
    /// from, and not enough to be misleading. The repositories say more.
    fn from(source: rusqlite::Error) -> Self {
        StoreError::Query {
            context: "the database".to_string(),
            source,
        }
    }
}

impl StoreError {
    /// Wrap a SQLite error with the context of the thing that failed.
    pub fn query(context: impl Into<String>, source: rusqlite::Error) -> Self {
        StoreError::Query {
            context: context.into(),
            source,
        }
    }

    /// Wrap a SQLite error with the context of the migration step that failed.
    pub fn migration(step: impl Into<String>, source: rusqlite::Error) -> Self {
        StoreError::Migration {
            step: step.into(),
            source,
        }
    }

    /// Wrap an I/O error with the path it happened on.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        StoreError::Io {
            path: path.into(),
            source,
        }
    }

    /// An export that cannot be read.
    pub fn export(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        StoreError::Export {
            path: path.into(),
            reason: reason.into(),
        }
    }
}
