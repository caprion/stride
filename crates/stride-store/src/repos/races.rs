//! `race_observations`.
//!
//! The athlete's own races: the distance, the finish time, the day of the
//! race, the day it was entered, the day it was last changed, and the
//! label the athlete gave it.
//!
//! A race is an **observation**, not a verdict. The athlete types it, and
//! the app never invents one and never infers one from the training: a
//! store that holds no race holds no paces, and the reason is the app's to
//! state, not the store's to paper over.
//!
//! **The anchor** is the race the paces come from, and at most one race
//! carries it. The first race the athlete enters takes it; entering
//! another race never moves it by itself — the athlete points it at a race,
//! and the app says which race it is.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::error::{Result, StoreError};
use crate::model::{NewRaceObservation, RaceObservation};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `race_observations`.
const COLUMNS: &str = "id, distance_m, time_s, race_date, label, name, source, entered_on, \
    updated_on, anchor";

/// Read one row of `race_observations`.
fn map(row: &Row<'_>) -> rusqlite::Result<RaceObservation> {
    Ok(RaceObservation {
        id: row.get("id")?,
        distance_m: row.get("distance_m")?,
        time_s: row.get("time_s")?,
        race_date: row.get("race_date")?,
        label: row.get("label")?,
        name: row.get("name")?,
        source: row.get("source")?,
        entered_on: row.get("entered_on")?,
        updated_on: row.get("updated_on")?,
        anchor: row.get::<_, i64>("anchor")? != 0,
    })
}

/// Whether a label is one the app knows.
///
/// A label the app does not know names no distance, and a race with such a
/// label cannot be read by anything that asks what distance it was. The
/// store refuses it rather than keeping a row nobody can use.
fn known_label(label: &str) -> bool {
    crate::model::race_label::is_known(label)
}

impl Store {
    /// Write a race.
    ///
    /// The **first** race the athlete enters takes the anchor, because the
    /// paces have to come from somewhere; a later race does not move it,
    /// and the athlete points it at a race with
    /// [`Store::set_anchor_race`].
    ///
    /// Returns the race's primary key.
    pub fn write_race(&self, race: &NewRaceObservation) -> Result<i64> {
        if !known_label(&race.label) {
            return Err(StoreError::Invariant(format!(
                "the race's label {:?} is not one the app knows",
                race.label
            )));
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let anchored: i64 = transaction
            .query_row(
                "SELECT count(*) FROM race_observations WHERE anchor = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::query("the anchored race", source))?;
        let anchor = if anchored == 0 { 1 } else { 0 };

        let id = transaction
            .query_row(
                "INSERT INTO race_observations (
                    distance_m, time_s, race_date, label, name, source,
                    entered_on, updated_on, anchor
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 RETURNING id",
                params![
                    race.distance_m,
                    race.time_s,
                    race.race_date,
                    race.label,
                    race.name,
                    race.source,
                    race.entered_on,
                    race.updated_on,
                    anchor,
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| StoreError::query("write a race", source))?;

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// Change a race, in place.
    ///
    /// The race keeps its identity and its place in the history: the
    /// values change, and the day the row was last changed is the caller's.
    /// The anchor does not move.
    pub fn update_race(&self, id: i64, race: &NewRaceObservation) -> Result<usize> {
        if !known_label(&race.label) {
            return Err(StoreError::Invariant(format!(
                "the race's label {:?} is not one the app knows",
                race.label
            )));
        }
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE race_observations SET
                    distance_m = ?2,
                    time_s = ?3,
                    race_date = ?4,
                    label = ?5,
                    name = ?6,
                    source = ?7,
                    entered_on = ?8,
                    updated_on = ?9
                 WHERE id = ?1",
                params![
                    id,
                    race.distance_m,
                    race.time_s,
                    race.race_date,
                    race.label,
                    race.name,
                    race.source,
                    race.entered_on,
                    race.updated_on,
                ],
            )
            .map_err(|source| StoreError::query("update a race", source))?;
        Ok(changed)
    }

    /// Every race, newest first.
    pub fn races(&self) -> Result<Vec<RaceObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM race_observations
             ORDER BY race_date DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the races", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("the races", source))?;
        collect(mapped)
    }

    /// One race, by its primary key.
    pub fn race(&self, id: i64) -> Result<Option<RaceObservation>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM race_observations WHERE id = ?1");
        conn.query_row(&sql, params![id], map)
            .optional()
            .map_err(|source| StoreError::query("the race", source))
    }

    /// The race the paces come from.
    pub fn anchor_race(&self) -> Result<Option<RaceObservation>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM race_observations
             WHERE anchor = 1
             ORDER BY id ASC
             LIMIT 1"
        );
        conn.query_row(&sql, [], map)
            .optional()
            .map_err(|source| StoreError::query("the anchored race", source))
    }

    /// Point the anchor at a race.
    ///
    /// One transaction: the anchor moves to the race the athlete named, and
    /// the race it was on stops carrying it. There is never a moment with
    /// two anchored races or with none.
    ///
    /// Returns whether the race was found.
    pub fn set_anchor_race(&self, id: i64) -> Result<bool> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;
        let exists: i64 = transaction
            .query_row(
                "SELECT count(*) FROM race_observations WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::query("the race", source))?;
        if exists == 0 {
            transaction
                .commit()
                .map_err(|source| StoreError::query("COMMIT", source))?;
            return Ok(false);
        }
        transaction
            .execute(
                "UPDATE race_observations SET anchor = 0 WHERE anchor = 1",
                [],
            )
            .map_err(|source| StoreError::query("the anchored race", source))?;
        transaction
            .execute(
                "UPDATE race_observations SET anchor = 1 WHERE id = ?1",
                params![id],
            )
            .map_err(|source| StoreError::query("the anchored race", source))?;
        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(true)
    }

    /// Remove a race.
    pub fn delete_race(&self, id: i64) -> Result<usize> {
        let conn = self.lock()?;
        let removed = conn
            .execute("DELETE FROM race_observations WHERE id = ?1", params![id])
            .map_err(|source| StoreError::query("the race", source))?;
        Ok(removed)
    }

    /// How many races there are.
    pub fn count_race_observations(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM race_observations", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the races", source))
    }
}
