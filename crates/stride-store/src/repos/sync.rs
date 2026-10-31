//! `sync_state`.
//!
//! One row per kind of data: `activities`, `wellness`, `daily_summaries`.
//! Each row holds the cursor to resume from, when the last sync of that kind
//! succeeded, and when the disposable 24-hour token expires.
//!
//! The cursor itself is opaque — it is whatever the Garmin API gives us —
//! and the store only moves it when the caller says the parse worked. See
//! [`crate::sync`] for the rule.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::SyncState;
use crate::repos::collect;
use crate::store::Store;

/// The columns of `sync_state`.
const COLUMNS: &str = "kind, cursor, last_success_at, token_expires_at, updated_at";

/// Read one row of `sync_state`.
fn map_state(row: &Row<'_>) -> rusqlite::Result<SyncState> {
    Ok(SyncState {
        kind: row.get("kind")?,
        cursor: row.get("cursor")?,
        last_success_at: row
            .get::<_, Option<DbDateTime>>("last_success_at")?
            .map(|value| value.0),
        token_expires_at: row
            .get::<_, Option<DbDateTime>>("token_expires_at")?
            .map(|value| value.0),
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

impl Store {
    /// The state of one kind of sync.
    pub fn sync_state(&self, kind: &str) -> Result<Option<SyncState>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM sync_state WHERE kind = ?1");
        conn.query_row(&sql, params![kind], map_state)
            .optional()
            .map_err(|source| StoreError::query("sync_state by kind", source))
    }

    /// The state of every kind of sync, by kind.
    pub fn all_sync_state(&self) -> Result<Vec<SyncState>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM sync_state ORDER BY kind ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("every sync_state", source))?;
        let mapped = statement
            .query_map([], map_state)
            .map_err(|source| StoreError::query("every sync_state", source))?;
        collect(mapped)
    }

    /// Write the state of one kind of sync.
    pub fn write_sync_state(&self, state: &SyncState) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO sync_state (kind, cursor, last_success_at, token_expires_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(kind) DO UPDATE SET
                cursor = excluded.cursor,
                last_success_at = excluded.last_success_at,
                token_expires_at = excluded.token_expires_at,
                updated_at = excluded.updated_at",
            params![
                state.kind,
                state.cursor,
                state.last_success_at.map(DbDateTime),
                state.token_expires_at.map(DbDateTime),
                DbDateTime(state.updated_at),
            ],
        )
        .map_err(|source| StoreError::query("upsert sync_state", source))?;
        Ok(())
    }

    /// Move the cursor of one kind of sync.
    ///
    /// This is the *only* way the cursor moves, and the caller is expected to
    /// call it after a successful parse — never before. [`crate::sync`] wraps
    /// it so that the rule cannot be forgotten.
    pub fn advance_sync_cursor(
        &self,
        kind: &str,
        cursor: &str,
        succeeded_at: DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO sync_state (kind, cursor, last_success_at, token_expires_at, updated_at)
             VALUES (?1, ?2, ?3, NULL, ?4)
             ON CONFLICT(kind) DO UPDATE SET
                cursor = excluded.cursor,
                last_success_at = excluded.last_success_at,
                updated_at = excluded.updated_at",
            params![
                kind,
                cursor,
                DbDateTime(succeeded_at),
                DbDateTime(succeeded_at)
            ],
        )
        .map_err(|source| StoreError::query("advance the sync cursor", source))?;
        Ok(())
    }

    /// Say when the disposable token expires.
    ///
    /// The token is disposable: a 401 triggers exactly one re-exchange and
    /// one retry. This only records when the current one runs out.
    pub fn record_token_expiry(
        &self,
        kind: &str,
        expires_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO sync_state (kind, cursor, last_success_at, token_expires_at, updated_at)
             VALUES (?1, NULL, NULL, ?2, ?3)
             ON CONFLICT(kind) DO UPDATE SET
                token_expires_at = excluded.token_expires_at,
                updated_at = excluded.updated_at",
            params![kind, DbDateTime(expires_at), DbDateTime(updated_at)],
        )
        .map_err(|source| StoreError::query("record the token expiry", source))?;
        Ok(())
    }

    /// Forget the cursor of one kind of sync, so the next sync starts again
    /// from the beginning.
    pub fn clear_sync_cursor(&self, kind: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE sync_state SET cursor = NULL, updated_at = ?2 WHERE kind = ?1",
            params![kind, DbDateTime(Utc::now())],
        )
        .map_err(|source| StoreError::query("clear the sync cursor", source))?;
        Ok(())
    }
}
