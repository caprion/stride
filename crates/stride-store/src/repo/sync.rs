//! `sync_run` — the sync log.

use rusqlite::{named_params, params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::Result;
use crate::model::SyncRun;
use crate::repo::helpers::collect;
use crate::Store;

const COLUMNS: &str = "id, source, started_at, finished_at, status, days, activities_found, \
    activities_new, activities_updated, message, error";

fn map_run(row: &Row<'_>) -> rusqlite::Result<SyncRun> {
    Ok(SyncRun {
        id: row.get("id")?,
        source: row.get("source")?,
        started_at: row.get::<_, DbDateTime>("started_at")?.0,
        finished_at: row
            .get::<_, Option<DbDateTime>>("finished_at")?
            .map(|v| v.0),
        status: row.get("status")?,
        days: row.get("days")?,
        activities_found: row.get("activities_found")?,
        activities_new: row.get("activities_new")?,
        activities_updated: row.get("activities_updated")?,
        message: row.get("message")?,
        error: row.get("error")?,
    })
}

impl Store {
    /// The sync log, newest first. `limit` of zero means every run.
    pub fn sync_runs(&self, limit: i64) -> Result<Vec<SyncRun>> {
        let conn = self.lock()?;
        let sql = if limit > 0 {
            format!(
                "SELECT {COLUMNS} FROM sync_run ORDER BY started_at DESC, id DESC LIMIT {limit}"
            )
        } else {
            format!("SELECT {COLUMNS} FROM sync_run ORDER BY started_at DESC, id DESC")
        };
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("sync_run".to_string(), source))?;
        let rows = statement
            .query_map([], map_run)
            .map_err(|source| crate::error::StoreError::query("sync_run".to_string(), source))?;
        collect(rows, "sync_run")
    }

    /// One sync run.
    pub fn sync_run(&self, id: i64) -> Result<Option<SyncRun>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM sync_run WHERE id = ?");
        conn.query_row(&sql, params![id], map_run)
            .optional()
            .map_err(|source| crate::error::StoreError::query("sync_run by id".to_string(), source))
    }

    /// The sync that is running right now, if there is one.
    pub fn running_sync(&self) -> Result<Option<SyncRun>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM sync_run WHERE status = 'running' ORDER BY started_at DESC LIMIT 1"
        );
        conn.query_row(&sql, [], map_run)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("running sync_run".to_string(), source)
            })
    }

    /// Write one sync run, and return its primary key.
    pub fn upsert_sync_run(&self, run: &SyncRun) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO sync_run ({COLUMNS})
             VALUES (:id, :source, :started_at, :finished_at, :status, :days, :activities_found,
                :activities_new, :activities_updated, :message, :error)
             ON CONFLICT(id) DO UPDATE SET
                source = excluded.source,
                started_at = excluded.started_at,
                finished_at = excluded.finished_at,
                status = excluded.status,
                days = excluded.days,
                activities_found = excluded.activities_found,
                activities_new = excluded.activities_new,
                activities_updated = excluded.activities_updated,
                message = excluded.message,
                error = excluded.error
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": run.id,
                ":source": run.source,
                ":started_at": DbDateTime(run.started_at),
                ":finished_at": run.finished_at.map(DbDateTime),
                ":status": run.status,
                ":days": run.days,
                ":activities_found": run.activities_found,
                ":activities_new": run.activities_new,
                ":activities_updated": run.activities_updated,
                ":message": run.message,
                ":error": run.error,
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert sync_run".to_string(), source))
    }
}
