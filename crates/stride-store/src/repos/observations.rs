//! `wellness_observations`.
//!
//! The source's own wellness observations: the value, the day the source
//! reported it for, and the day it was read. One row per (day, source,
//! field, item).
//!
//! Two rules hold here, and both are about honesty:
//!
//! * **The missingness is kept.** A field the source never sent has no row
//!   at all; a field it sent as null has a row whose value is null. Nothing
//!   is ever turned into a zero, and nothing is invented to fill a gap.
//! * **A later read never deletes a row.** It replaces the value it carried
//!   and keeps the reading time of the read that carried it, so a failed
//!   read leaves the previous value and its reading time in place. A reading
//!   without its retrieval time cannot be audited, and the time is part of
//!   the row.

use rusqlite::{params, Connection, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{NewWellnessObservation, WellnessObservation};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `wellness_observations`.
const COLUMNS: &str = "id, date, source, endpoint, field, ordinal, value, text_value, unit, \
    observed_at, retrieved_at";

/// Read one row of `wellness_observations`.
fn map(row: &Row<'_>) -> rusqlite::Result<WellnessObservation> {
    Ok(WellnessObservation {
        id: row.get("id")?,
        date: row.get("date")?,
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

/// Write one observation, over the one already stored for the same day,
/// source, field and item.
fn write_one(conn: &Connection, observation: &NewWellnessObservation) -> Result<()> {
    conn.execute(
        "INSERT INTO wellness_observations (
            date, source, endpoint, field, ordinal, value, text_value, unit,
            observed_at, retrieved_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(date, source, field, ordinal) DO UPDATE SET
            endpoint = excluded.endpoint,
            value = excluded.value,
            text_value = excluded.text_value,
            unit = excluded.unit,
            observed_at = excluded.observed_at,
            retrieved_at = excluded.retrieved_at",
        params![
            observation.date,
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
    .map_err(|source| StoreError::query("write a wellness observation", source))?;
    Ok(())
}

/// Write a set of observations into a connection.
///
/// The caller owns the transaction: the observations of a day are written
/// with the day's wellness row and the day's cursor, in one transaction, so
/// the data and the cursor commit together.
pub(crate) fn write_observations(
    conn: &Connection,
    observations: &[NewWellnessObservation],
) -> Result<usize> {
    for observation in observations {
        write_one(conn, observation)?;
    }
    Ok(observations.len())
}

impl Store {
    /// Write a set of observations.
    ///
    /// One transaction: the set is the day's readings, and they land
    /// together or not at all. A field the set does not carry is left
    /// alone — the value already stored, and the day it was read, stay.
    ///
    /// Returns how many rows were written.
    pub fn write_wellness_observations(
        &self,
        observations: &[NewWellnessObservation],
    ) -> Result<usize> {
        if observations.is_empty() {
            return Ok(0);
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction()
            .map_err(|source| StoreError::query("begin the wellness observations", source))?;
        let written = write_observations(&transaction, observations)?;
        transaction
            .commit()
            .map_err(|source| StoreError::query("commit the wellness observations", source))?;
        Ok(written)
    }

    /// The observations of one day, in the order they were written.
    pub fn wellness_observations(&self, date: &str) -> Result<Vec<WellnessObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM wellness_observations
             WHERE date = ?1
             ORDER BY id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the wellness observations", source))?;
        let mapped = statement
            .query_map(params![date], map)
            .map_err(|source| StoreError::query("the wellness observations", source))?;
        collect(mapped)
    }

    /// The observations between two dates, both inclusive, oldest first.
    pub fn wellness_observations_between(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<WellnessObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM wellness_observations
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the wellness observations", source))?;
        let mapped = statement
            .query_map(params![from, to], map)
            .map_err(|source| StoreError::query("the wellness observations", source))?;
        collect(mapped)
    }

    /// How many observations there are.
    pub fn count_wellness_observations(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM wellness_observations", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the wellness observations", source))
    }
}
