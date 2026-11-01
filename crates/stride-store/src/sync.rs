//! The sync cursor.
//!
//! ## The rule
//!
//! **A cursor advances only after a successful parse — never before.**
//!
//! A failed sync must leave the cursor where it was, so that the next sync
//! picks up the same place and the rows that were not parsed are parsed
//! again. The opposite — advancing first and parsing after — loses data
//! silently, and the loss is invisible until the athlete notices a gap.
//!
//! ## How the rule is kept
//!
//! Not by remembering it. The type makes it hard to break:
//!
//! ```no_run
//! use stride_store::Store;
//!
//! # fn main() -> Result<(), stride_store::StoreError> {
//! # let store = Store::open(std::path::Path::new("stride.db"), &stride_store::DatabaseKey::from_text("k")?)?;
//! let mut cursor = store.begin_sync("activities")?;
//! // … fetch a page, parse it …
//! cursor.advance("the cursor the API gave us"); // in memory only
//! cursor.commit()?;                           // now it is written
//! # Ok(())
//! # }
//! ```
//!
//! [`SyncCursor::advance`] only records the new position in memory.
//! [`SyncCursor::commit`] is what writes it. A sync that fails — a parse
//! error, a dropped connection, a panic — drops the guard instead, and the
//! cursor in the database is exactly where it was.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::store::Store;

/// A sync in progress.
///
/// It borrows the store, so the store cannot be written while the sync is
/// being set up. The cursor it holds is the one in the database until
/// [`SyncCursor::commit`] is called.
pub struct SyncCursor<'a> {
    store: &'a Store,
    kind: String,
    current: Option<String>,
    pending: Option<String>,
}

impl<'a> SyncCursor<'a> {
    /// The kind of data being synced.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The cursor the database holds — the place the sync started from.
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// The cursor the sync would move to, if it commits.
    pub fn pending(&self) -> Option<&str> {
        self.pending.as_deref()
    }

    /// Say where the sync got to.
    ///
    /// Nothing is written: the new position is held in memory until
    /// [`SyncCursor::commit`]. Calling this twice replaces the position, so
    /// the last one wins.
    pub fn advance(&mut self, cursor: impl Into<String>) {
        self.pending = Some(cursor.into());
    }

    /// Write the new position.
    ///
    /// This is the moment the cursor moves. Call it only when the data has
    /// been parsed and stored — never before.
    pub fn commit(self) -> Result<()> {
        self.commit_at(Utc::now())
    }

    /// Write the new position, with the time of the success.
    ///
    /// The same as [`SyncCursor::commit`], with the instant given rather than
    /// read from the clock. A test uses this one.
    pub fn commit_at(self, succeeded_at: DateTime<Utc>) -> Result<()> {
        let cursor = match self.pending {
            Some(cursor) => cursor,
            // Nothing was advanced: there is nothing to write, and the cursor
            // in the database stays where it is.
            None => return Ok(()),
        };
        self.store
            .advance_sync_cursor(&self.kind, &cursor, succeeded_at)
    }
}

impl Store {
    /// Begin a sync of one kind of data.
    ///
    /// The cursor in the database is read; the new one is not written until
    /// the sync commits.
    pub fn begin_sync(&self, kind: &str) -> Result<SyncCursor<'_>> {
        let state = self.sync_state(kind)?;
        let current = state.and_then(|state| state.cursor);
        Ok(SyncCursor {
            store: self,
            kind: kind.to_string(),
            current,
            pending: None,
        })
    }
}
