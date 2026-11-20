//! `run_updates` and `update_settings`.
//!
//! **The update is the model's own reading of a run, and it is attributed.**
//! It is not the product's voice: the product's numbers stay the product's,
//! computed in Rust from the stored series, with their sources named. What
//! is stored here is the reading, the two models that wrote and checked it,
//! the hash of the payload it was written from, and the checks it passed.
//!
//! Two rules hold:
//!
//! * **An update that cannot be re-checked is just a sentence.** The
//!   payload hash names what the model was given, so the numbers in the
//!   draft can be looked up in it later, and the checks' findings are
//!   stored beside the draft.
//! * **Only a shipped update is kept.** A draft that did not pass the
//!   checks, and one the reviewer discarded, keep nothing: there is no row
//!   for a run that has no update.
//!
//! The settings are the update's own: which model writes, which model
//! reviews, and whether anything may leave the machine at all. The **key is
//! not here** — the settings name the environment variable it is read from,
//! so the key is never in the database and never in an export.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{NewRunUpdate, RunUpdate, UpdateSettings};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `run_updates`.
const COLUMNS: &str = "id, activity_id, writer, reviewer, payload_hash, draft, checks, \
    verdict, created_at";

/// Read one row of `run_updates`.
fn map(row: &Row<'_>) -> rusqlite::Result<RunUpdate> {
    Ok(RunUpdate {
        id: row.get("id")?,
        activity_id: row.get("activity_id")?,
        writer: row.get("writer")?,
        reviewer: row.get("reviewer")?,
        payload_hash: row.get("payload_hash")?,
        draft: row.get("draft")?,
        checks: row.get("checks")?,
        verdict: row.get("verdict")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
    })
}

impl Store {
    /// The update of a run, when it has one.
    pub fn run_update(&self, activity_id: i64) -> Result<Option<RunUpdate>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM run_updates WHERE activity_id = ?1");
        conn.query_row(&sql, params![activity_id], map)
            .optional()
            .map_err(|source| StoreError::query("the run's update", source))
    }

    /// Every stored update, oldest first.
    pub fn run_updates(&self) -> Result<Vec<RunUpdate>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM run_updates ORDER BY created_at ASC, id ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the run updates", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("the run updates", source))?;
        collect(mapped)
    }

    /// How many updates there are.
    pub fn count_run_updates(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM run_updates", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count the run updates", source))
    }

    /// Write an update, over the one the run already has.
    ///
    /// A run has one update: the one that was shipped. Writing a new one
    /// replaces it, so the run's own history holds the reading that is on
    /// the screen and not every draft that led to it.
    pub fn write_run_update(&self, update: &NewRunUpdate) -> Result<i64> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO run_updates (
                activity_id, writer, reviewer, payload_hash, draft, checks, verdict, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(activity_id) DO UPDATE SET
                writer = excluded.writer,
                reviewer = excluded.reviewer,
                payload_hash = excluded.payload_hash,
                draft = excluded.draft,
                checks = excluded.checks,
                verdict = excluded.verdict,
                created_at = excluded.created_at",
            params![
                update.activity_id,
                update.writer,
                update.reviewer,
                update.payload_hash,
                update.draft,
                update.checks,
                update.verdict,
                DbDateTime(update.created_at),
            ],
        )
        .map_err(|source| StoreError::query("write a run update", source))?;
        Ok(conn.last_insert_rowid())
    }

    /// The update's own settings.
    ///
    /// The defaults when nothing has been written: the feature is off,
    /// nothing may leave the machine, and no model is named. The time is
    /// the caller's — the store has no clock of its own.
    pub fn update_settings(&self, at: chrono::DateTime<chrono::Utc>) -> Result<UpdateSettings> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT enabled, allow_remote, writer_model, writer_family, writer_base_url, \
                 reviewer_model, reviewer_family, reviewer_base_url, key_env, updated_at \
                 FROM update_settings WHERE id = 1",
                [],
                map_settings,
            )
            .optional()
            .map_err(|source| StoreError::query("the update's settings", source))?;
        Ok(row.unwrap_or_else(|| UpdateSettings::defaults(at)))
    }

    /// Write the update's own settings.
    pub fn write_update_settings(&self, settings: &UpdateSettings) -> Result<()> {
        let conn = self.lock()?;
        write_settings(&conn, settings)
    }
}

/// Read one row of `update_settings`.
fn map_settings(row: &Row<'_>) -> rusqlite::Result<UpdateSettings> {
    Ok(UpdateSettings {
        enabled: row.get::<_, i64>("enabled")? != 0,
        allow_remote: row.get::<_, i64>("allow_remote")? != 0,
        writer_model: row.get("writer_model")?,
        writer_family: row.get("writer_family")?,
        writer_base_url: row.get("writer_base_url")?,
        reviewer_model: row.get("reviewer_model")?,
        reviewer_family: row.get("reviewer_family")?,
        reviewer_base_url: row.get("reviewer_base_url")?,
        key_env: row.get("key_env")?,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

/// Write the one row of `update_settings`.
fn write_settings(conn: &Connection, settings: &UpdateSettings) -> Result<()> {
    conn.execute(
        "INSERT INTO update_settings (
            id, enabled, allow_remote, writer_model, writer_family, writer_base_url,
            reviewer_model, reviewer_family, reviewer_base_url, key_env, updated_at
         ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            enabled = excluded.enabled,
            allow_remote = excluded.allow_remote,
            writer_model = excluded.writer_model,
            writer_family = excluded.writer_family,
            writer_base_url = excluded.writer_base_url,
            reviewer_model = excluded.reviewer_model,
            reviewer_family = excluded.reviewer_family,
            reviewer_base_url = excluded.reviewer_base_url,
            key_env = excluded.key_env,
            updated_at = excluded.updated_at",
        params![
            i64::from(settings.enabled),
            i64::from(settings.allow_remote),
            settings.writer_model,
            settings.writer_family,
            settings.writer_base_url,
            settings.reviewer_model,
            settings.reviewer_family,
            settings.reviewer_base_url,
            settings.key_env,
            DbDateTime(settings.updated_at),
        ],
    )
    .map_err(|source| StoreError::query("write the update's settings", source))?;
    Ok(())
}
