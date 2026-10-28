//! Errors.
//!
//! Every failure carries the context needed to find it: which table, which
//! column, which statement. `StoreError::Query { context: "activity.local_date" }`
//! says where; `StoreError("bad file")` says nothing.

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

    /// The key was rejected: wrong key, or the file is not an encrypted database.
    #[error("the database key was rejected: {reason}")]
    InvalidKey {
        /// Why the key was rejected.
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
    #[error("migration failed at {step}: {source}")]
    Migration {
        /// The migration step that failed.
        step: String,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },

    /// A JSON column could not be decoded.
    #[error("could not decode the {column} column: {source}")]
    Json {
        /// The column that could not be decoded.
        column: String,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },

    /// A datetime column could not be decoded.
    #[error("could not decode the {column} column: {value:?} is not a datetime")]
    DateTime {
        /// The column that could not be decoded.
        column: String,
        /// The text that was in the column.
        value: String,
    },

    /// The store's lock was poisoned by a panic in another thread.
    #[error("the database lock was poisoned by a panic in another thread")]
    Poisoned,

    /// A file operation failed.
    #[error("could not read {path}: {source}")]
    Io {
        /// The file we were reading.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
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
}
