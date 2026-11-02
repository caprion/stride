//! `deleted_garmin_activities`.
//!
//! A run Garmin no longer lists is **marked**, not forgotten and not
//! resurrected. A pull that cannot find a stored run in Garmin's list writes
//! a tombstone here; a later pull that sees the run again does not bring it
//! back — Garmin's list is not proof that a run came back, it is proof that
//! the run is still there, and the tombstone says the pull already accounted
//! for it.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::store::Store;

/// One tombstone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedActivity {
    /// The Garmin activity id.
    pub external_id: String,
    /// The run's day, `YYYY-MM-DD`.
    pub local_date: String,
    /// When the pull marked it.
    pub marked_at: DateTime<Utc>,
}

fn map(row: &Row<'_>) -> rusqlite::Result<DeletedActivity> {
    Ok(DeletedActivity {
        external_id: row.get("external_id")?,
        local_date: row.get("local_date")?,
        marked_at: row.get::<_, DbDateTime>("marked_at")?.0,
    })
}

impl Store {
    /// Mark a Garmin activity as deleted, by its id.
    ///
    /// A tombstone is written once and kept: a later pull that sees the run
    /// again does not bring it back.
    pub fn mark_activity_deleted(
        &self,
        external_id: &str,
        local_date: &str,
        marked_at: DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO deleted_garmin_activities (external_id, local_date, marked_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(external_id) DO NOTHING",
            params![external_id, local_date, DbDateTime(marked_at)],
        )
        .map_err(|source| StoreError::query("mark an activity deleted", source))?;
        Ok(())
    }

    /// Whether a Garmin activity id is marked deleted.
    pub fn activity_is_deleted(&self, external_id: &str) -> Result<bool> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT 1 FROM deleted_garmin_activities WHERE external_id = ?1",
            params![external_id],
            |_| Ok(()),
        )
        .optional()
        .map(|found| found.is_some())
        .map_err(|source| StoreError::query("is the activity deleted", source))
    }

    /// Every tombstone, oldest first.
    pub fn deleted_activities(&self) -> Result<Vec<DeletedActivity>> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT external_id, local_date, marked_at
                 FROM deleted_garmin_activities
                 ORDER BY marked_at ASC, external_id ASC",
            )
            .map_err(|source| StoreError::query("deleted activities", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("deleted activities", source))?;
        crate::repos::collect(mapped)
    }

    /// How many activities are marked deleted.
    pub fn count_deleted_activities(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT count(*) FROM deleted_garmin_activities",
            [],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::query("count deleted activities", source))
    }
}
