//! `activity_observations`.
//!
//! The source's own facts about a run: the value, the day the response
//! carried it, the day it was read, and the endpoint it was read from. One
//! row per (activity, source, field, item).
//!
//! Two rules hold here, and both are about honesty:
//!
//! * **The missingness is kept.** A field the source never sent has no row
//!   at all; a field it sent as null has a row whose value is null. Nothing
//!   is ever turned into a zero, and nothing is invented to fill a gap.
//! * **A later read never deletes a row.** It replaces the value it carried
//!   and keeps the reading time of the read that carried it, so a failed
//!   read leaves the previous value and its reading time in place.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{ActivityObservation, NewActivityObservation};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `activity_observations`.
const COLUMNS: &str = "id, activity_id, source, endpoint, field, ordinal, value, text_value, \
    unit, observed_at, retrieved_at";

/// Read one row of `activity_observations`.
fn map(row: &Row<'_>) -> rusqlite::Result<ActivityObservation> {
    Ok(ActivityObservation {
        id: row.get("id")?,
        activity_id: row.get("activity_id")?,
        source: row.get("source")?,
        endpoint: row.get("endpoint")?,
        field: row.get("field")?,
        ordinal: row.get("ordinal")?,
        value: row.get("value")?,
        text_value: row.get("text_value")?,
        unit: row.get("unit")?,
        observed_at: row
            .get::<_, Option<DbDateTime>>("observed_at")?
            .map(|value| value.0),
        retrieved_at: row.get::<_, DbDateTime>("retrieved_at")?.0,
    })
}

/// Write one observation, over the one already stored for the same run,
/// source, field and item.
fn write_one(conn: &Connection, observation: &NewActivityObservation) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_observations (
            activity_id, source, endpoint, field, ordinal, value, text_value, unit,
            observed_at, retrieved_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(activity_id, source, field, ordinal) DO UPDATE SET
            endpoint = excluded.endpoint,
            value = excluded.value,
            text_value = excluded.text_value,
            unit = excluded.unit,
            observed_at = excluded.observed_at,
            retrieved_at = excluded.retrieved_at",
        params![
            observation.activity_id,
            observation.source,
            observation.endpoint,
            observation.field,
            observation.ordinal,
            observation.value,
            observation.text_value,
            observation.unit,
            observation.observed_at.map(DbDateTime),
            DbDateTime(observation.retrieved_at),
        ],
    )
    .map_err(|source| StoreError::query("write an activity observation", source))?;
    Ok(())
}

impl Store {
    /// Write a set of observations about runs.
    ///
    /// One transaction: the set is one response's readings, and they land
    /// together or not at all. A field the set does not carry is left
    /// alone — the value already stored, and the day it was read, stay.
    ///
    /// Returns how many rows were written.
    pub fn write_activity_observations(
        &self,
        observations: &[NewActivityObservation],
    ) -> Result<usize> {
        if observations.is_empty() {
            return Ok(0);
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction()
            .map_err(|source| StoreError::query("begin the activity observations", source))?;
        for observation in observations {
            write_one(&transaction, observation)?;
        }
        transaction
            .commit()
            .map_err(|source| StoreError::query("commit the activity observations", source))?;
        Ok(observations.len())
    }

    /// The observations of one run, in the order they were written.
    pub fn activity_observations(&self, activity_id: i64) -> Result<Vec<ActivityObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM activity_observations
             WHERE activity_id = ?1
             ORDER BY id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the activity observations", source))?;
        let mapped = statement
            .query_map(params![activity_id], map)
            .map_err(|source| StoreError::query("the activity observations", source))?;
        collect(mapped)
    }

    /// The observations of the runs between two dates, both inclusive, oldest
    /// first.
    pub fn activity_observations_between(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<ActivityObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM activity_observations
             WHERE activity_id IN (
                 SELECT id FROM activities WHERE local_date >= ?1 AND local_date <= ?2
             )
             ORDER BY activity_id ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the activity observations", source))?;
        let mapped = statement
            .query_map(params![from, to], map)
            .map_err(|source| StoreError::query("the activity observations", source))?;
        collect(mapped)
    }

    /// One observation, by its run, source, field and item.
    pub fn activity_observation(
        &self,
        activity_id: i64,
        source: &str,
        field: &str,
        ordinal: i64,
    ) -> Result<Option<ActivityObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM activity_observations
             WHERE activity_id = ?1 AND source = ?2 AND field = ?3 AND ordinal = ?4"
        );
        conn.query_row(&sql, params![activity_id, source, field, ordinal], map)
            .optional()
            .map_err(|source| StoreError::query("the activity observation", source))
    }

    /// How many observations there are.
    pub fn count_activity_observations(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM activity_observations", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the activity observations", source))
    }
}
