//! `wellness_daily`.
//!
//! The wellness inputs and nothing else: sleep, HRV, resting heart rate, and
//! the stress average where the watch reports one. There is **no score** here
//! — no sleep score, no readiness, no body battery — because the product
//! never shows a device's opinion of the athlete.
//!
//! A missing measurement is `NULL`, never `0`. A night with no HRV reading
//! has no HRV, and a resting heart rate of zero is not a heart rate.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{SyncState, WellnessDaily};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `wellness_daily`.
const COLUMNS: &str = "date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s, hrv_ms, \
    resting_hr, stress_avg, source, updated_at";

/// Read one row of `wellness_daily`.
fn map_wellness(row: &Row<'_>) -> rusqlite::Result<WellnessDaily> {
    Ok(WellnessDaily {
        date: row.get("date")?,
        sleep_s: row.get("sleep_s")?,
        deep_sleep_s: row.get("deep_sleep_s")?,
        rem_s: row.get("rem_s")?,
        light_sleep_s: row.get("light_sleep_s")?,
        awake_s: row.get("awake_s")?,
        hrv_ms: row.get("hrv_ms")?,
        resting_hr: row.get("resting_hr")?,
        stress_avg: row.get("stress_avg")?,
        source: row.get("source")?,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

fn write_wellness(
    conn: &Connection,
    wellness: &WellnessDaily,
    preserve_missing: bool,
) -> Result<()> {
    conn.execute(
            "INSERT INTO wellness_daily (
                date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s,
                hrv_ms, resting_hr, stress_avg, source, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(date) DO UPDATE SET
                sleep_s = CASE WHEN ?12 THEN COALESCE(excluded.sleep_s, wellness_daily.sleep_s) ELSE excluded.sleep_s END,
                deep_sleep_s = CASE WHEN ?12 THEN COALESCE(excluded.deep_sleep_s, wellness_daily.deep_sleep_s) ELSE excluded.deep_sleep_s END,
                rem_s = CASE WHEN ?12 THEN COALESCE(excluded.rem_s, wellness_daily.rem_s) ELSE excluded.rem_s END,
                light_sleep_s = CASE WHEN ?12 THEN COALESCE(excluded.light_sleep_s, wellness_daily.light_sleep_s) ELSE excluded.light_sleep_s END,
                awake_s = CASE WHEN ?12 THEN COALESCE(excluded.awake_s, wellness_daily.awake_s) ELSE excluded.awake_s END,
                hrv_ms = CASE WHEN ?12 THEN COALESCE(excluded.hrv_ms, wellness_daily.hrv_ms) ELSE excluded.hrv_ms END,
                resting_hr = CASE WHEN ?12 THEN COALESCE(excluded.resting_hr, wellness_daily.resting_hr) ELSE excluded.resting_hr END,
                stress_avg = CASE WHEN ?12 THEN COALESCE(excluded.stress_avg, wellness_daily.stress_avg) ELSE excluded.stress_avg END,
                source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                wellness.date,
                wellness.sleep_s,
                wellness.deep_sleep_s,
                wellness.rem_s,
                wellness.light_sleep_s,
                wellness.awake_s,
                wellness.hrv_ms,
                wellness.resting_hr,
                wellness.stress_avg,
                wellness.source,
                DbDateTime(wellness.updated_at),
                preserve_missing,
            ],
        )
        .map_err(|source| StoreError::query("upsert wellness_daily", source))?;
    Ok(())
}

impl Store {
    /// Write one day of wellness, replacing nullable fields as supplied.
    pub fn upsert_wellness_daily(&self, wellness: &WellnessDaily) -> Result<()> {
        let conn = self.lock()?;
        write_wellness(&conn, wellness, false)
    }

    /// Commit a provider day and its checked-date cursor together.
    ///
    /// Missing provider fields preserve stored observations. `None` records a
    /// successful no-data response without inventing a wellness row.
    pub fn write_wellness_checkpoint(
        &self,
        wellness: Option<&WellnessDaily>,
        state: &SyncState,
    ) -> Result<()> {
        if state.kind != crate::model::sync_kind::WELLNESS {
            return Err(StoreError::Invariant(
                "a wellness checkpoint has the wrong kind".to_string(),
            ));
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction()
            .map_err(|source| StoreError::query("begin wellness checkpoint", source))?;
        if let Some(wellness) = wellness {
            write_wellness(&transaction, wellness, true)?;
        }
        super::sync::write_state(&transaction, state)?;
        transaction
            .commit()
            .map_err(|source| StoreError::query("commit wellness checkpoint", source))
    }

    /// One day of wellness.
    pub fn wellness_daily(&self, date: &str) -> Result<Option<WellnessDaily>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM wellness_daily WHERE date = ?1");
        conn.query_row(&sql, params![date], map_wellness)
            .optional()
            .map_err(|source| StoreError::query("wellness_daily by date", source))
    }

    /// The wellness between two dates, both inclusive, oldest first.
    ///
    /// This is the window the direction band is computed over: the median of
    /// the last thirty nights, the day itself excluded.
    pub fn wellness_between(&self, from: &str, to: &str) -> Result<Vec<WellnessDaily>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM wellness_daily
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("wellness between", source))?;
        let mapped = statement
            .query_map(params![from, to], map_wellness)
            .map_err(|source| StoreError::query("wellness between", source))?;
        collect(mapped)
    }

    /// The wellness of the last `days` days before `before`, oldest first.
    ///
    /// The day itself is excluded: "usual" is the median of the nights
    /// before, never of the night being judged.
    pub fn wellness_before(&self, before: &str, days: i64) -> Result<Vec<WellnessDaily>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM wellness_daily
             WHERE date < ?1
             ORDER BY date DESC
             LIMIT ?2"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("wellness before", source))?;
        let mapped = statement
            .query_map(params![before, days], map_wellness)
            .map_err(|source| StoreError::query("wellness before", source))?;
        let mut rows = collect(mapped)?;
        rows.reverse();
        Ok(rows)
    }

    /// The most recent day of wellness that is not after `before`.
    pub fn latest_wellness(&self, before: &str) -> Result<Option<WellnessDaily>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM wellness_daily
             WHERE date <= ?1
             ORDER BY date DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![before], map_wellness)
            .optional()
            .map_err(|source| StoreError::query("latest wellness_daily", source))
    }

    /// How many days of wellness there are.
    pub fn count_wellness_daily(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM wellness_daily", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count wellness_daily", source))
    }
}
