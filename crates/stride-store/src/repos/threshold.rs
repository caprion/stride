//! `threshold_observations`.
//!
//! The threshold the athlete uses: one dated fact per **choice**, with the
//! value (or the pair of values), the source it came from, the rule that
//! computed it where one applied, the base it was made for, the day the
//! choice was made and the day the row was last changed.
//!
//! The candidates are computed and never stored: this table holds what the
//! athlete picked, and the history of the picks.
//!
//! **One choice is in force at a time.** A later choice does not edit the
//! earlier one — it writes another row, and the earlier row stays in the
//! dated history with its base named. A plan adopted in March is paced by
//! March's number, and the card says so.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::error::{Result, StoreError};
use crate::model::{NewThresholdObservation, ThresholdObservation};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `threshold_observations`.
const COLUMNS: &str = "id, hr_bpm, pace_low_s_per_km, pace_high_s_per_km, source, rule_id, \
    base, origin_date, origin_ref, label, entered_on, updated_on, in_force";

/// Read one row of `threshold_observations`.
fn map(row: &Row<'_>) -> rusqlite::Result<ThresholdObservation> {
    Ok(ThresholdObservation {
        id: row.get("id")?,
        hr_bpm: row.get("hr_bpm")?,
        pace_low_s_per_km: row.get("pace_low_s_per_km")?,
        pace_high_s_per_km: row.get("pace_high_s_per_km")?,
        source: row.get("source")?,
        rule_id: row.get("rule_id")?,
        base: row.get("base")?,
        origin_date: row.get("origin_date")?,
        origin_ref: row.get("origin_ref")?,
        label: row.get("label")?,
        entered_on: row.get("entered_on")?,
        updated_on: row.get("updated_on")?,
        in_force: row.get::<_, i64>("in_force")? != 0,
    })
}

/// Whether a choice carries anything at all.
///
/// A choice with no heart rate and no pace band is not a choice: there is
/// no value for it to hold, and a row that holds nothing would be read as
/// a threshold of no value — which is not the same thing.
fn carries_a_value(choice: &NewThresholdObservation) -> bool {
    choice.hr_bpm.is_some() || choice.pace_low_s_per_km.is_some()
}

impl Store {
    /// Write a threshold choice.
    ///
    /// The choice that is **in force** takes the flag, and the choice that
    /// held it stops: one threshold at a time, in one transaction, so there
    /// is never a moment with two.
    ///
    /// A choice that carries no value at all is refused.
    ///
    /// Returns the choice's primary key.
    pub fn write_threshold_choice(
        &self,
        choice: &NewThresholdObservation,
        in_force: bool,
    ) -> Result<i64> {
        if !carries_a_value(choice) {
            return Err(StoreError::Invariant(
                "the threshold choice carries no value at all".to_string(),
            ));
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let flag = i64::from(in_force);
        if in_force {
            transaction
                .execute(
                    "UPDATE threshold_observations SET in_force = 0 WHERE in_force = 1",
                    [],
                )
                .map_err(|source| StoreError::query("the threshold in force", source))?;
        }

        let id = transaction
            .query_row(
                "INSERT INTO threshold_observations (
                    hr_bpm, pace_low_s_per_km, pace_high_s_per_km, source, rule_id, base,
                    origin_date, origin_ref, label, entered_on, updated_on, in_force
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 RETURNING id",
                params![
                    choice.hr_bpm,
                    choice.pace_low_s_per_km,
                    choice.pace_high_s_per_km,
                    choice.source,
                    choice.rule_id,
                    choice.base,
                    choice.origin_date,
                    choice.origin_ref,
                    choice.label,
                    choice.entered_on,
                    choice.updated_on,
                    flag,
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| StoreError::query("write a threshold choice", source))?;

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// Put a choice in force, and take the flag from the one that held it.
    ///
    /// One transaction: there is never a moment with two choices in force,
    /// or with none.
    ///
    /// Returns whether the choice was found.
    pub fn set_threshold_in_force(&self, id: i64) -> Result<bool> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;
        let exists: i64 = transaction
            .query_row(
                "SELECT count(*) FROM threshold_observations WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::query("the threshold choice", source))?;
        if exists == 0 {
            transaction
                .commit()
                .map_err(|source| StoreError::query("COMMIT", source))?;
            return Ok(false);
        }
        transaction
            .execute(
                "UPDATE threshold_observations SET in_force = 0 WHERE in_force = 1",
                [],
            )
            .map_err(|source| StoreError::query("the threshold in force", source))?;
        transaction
            .execute(
                "UPDATE threshold_observations SET in_force = 1 WHERE id = ?1",
                params![id],
            )
            .map_err(|source| StoreError::query("the threshold in force", source))?;
        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(true)
    }

    /// Every choice, newest first.
    pub fn threshold_choices(&self) -> Result<Vec<ThresholdObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM threshold_observations
             ORDER BY entered_on DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the threshold choices", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("the threshold choices", source))?;
        collect(mapped)
    }

    /// One choice, by its primary key.
    pub fn threshold_choice(&self, id: i64) -> Result<Option<ThresholdObservation>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM threshold_observations WHERE id = ?1");
        conn.query_row(&sql, params![id], map)
            .optional()
            .map_err(|source| StoreError::query("the threshold choice", source))
    }

    /// The choice that is in force.
    pub fn threshold_in_force(&self) -> Result<Option<ThresholdObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM threshold_observations
             WHERE in_force = 1
             ORDER BY id ASC
             LIMIT 1"
        );
        conn.query_row(&sql, [], map)
            .optional()
            .map_err(|source| StoreError::query("the threshold in force", source))
    }

    /// The choices made for one base, newest first.
    pub fn threshold_choices_for_base(&self, base: &str) -> Result<Vec<ThresholdObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM threshold_observations
             WHERE base = ?1
             ORDER BY entered_on DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the threshold choices", source))?;
        let mapped = statement
            .query_map(params![base], map)
            .map_err(|source| StoreError::query("the threshold choices", source))?;
        collect(mapped)
    }

    /// How many choices there are.
    pub fn count_threshold_observations(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM threshold_observations", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the threshold choices", source))
    }

    /// How many choices are in force.
    ///
    /// **One threshold at a time.** The writer takes the flag from the
    /// choice that held it, in one transaction, so this is one — and a
    /// caller that wants to be sure of it can count them.
    pub fn count_threshold_in_force(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT count(*) FROM threshold_observations WHERE in_force = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::query("count the thresholds in force", source))
    }
}
