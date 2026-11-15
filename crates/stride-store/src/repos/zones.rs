//! `hr_zone_models`.
//!
//! The athlete's heart-rate zone model is a **dated fact about their
//! configuration**, read from Garmin's `/biometric-service/heartRateZones/v2`
//! — never a series derived from a run. One row is written per model change,
//! with the day it was read, so a number can always say where it came from.
//!
//! A read that returns the same model writes nothing; a read that fails
//! writes nothing. Absence is not zero and not "no model" — the previous
//! model, and its date, stay in place.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{HeartRateZoneModel, NewHeartRateZoneModel};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `hr_zone_models`.
const COLUMNS: &str = "id, sport, training_method, max_heart_rate, zone1_floor, zone2_floor, \
    zone3_floor, zone4_floor, zone5_floor, read_at";

/// Read one row of `hr_zone_models`.
fn map(row: &Row<'_>) -> rusqlite::Result<HeartRateZoneModel> {
    Ok(HeartRateZoneModel {
        id: row.get("id")?,
        sport: row.get("sport")?,
        training_method: row.get("training_method")?,
        max_heart_rate: row.get("max_heart_rate")?,
        zone1_floor: row.get("zone1_floor")?,
        zone2_floor: row.get("zone2_floor")?,
        zone3_floor: row.get("zone3_floor")?,
        zone4_floor: row.get("zone4_floor")?,
        zone5_floor: row.get("zone5_floor")?,
        read_at: row.get::<_, DbDateTime>("read_at")?.0,
    })
}

/// Whether a stored row is the same model as the one being written.
fn same_model(stored: &HeartRateZoneModel, incoming: &NewHeartRateZoneModel) -> bool {
    stored.sport == incoming.sport
        && stored.training_method == incoming.training_method
        && stored.max_heart_rate == incoming.max_heart_rate
        && stored.zone1_floor == incoming.zone1_floor
        && stored.zone2_floor == incoming.zone2_floor
        && stored.zone3_floor == incoming.zone3_floor
        && stored.zone4_floor == incoming.zone4_floor
        && stored.zone5_floor == incoming.zone5_floor
}

impl Store {
    /// The current zone model: the one read most recently.
    pub fn current_hr_zone_model(&self) -> Result<Option<HeartRateZoneModel>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM hr_zone_models
             ORDER BY read_at DESC, id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, [], map)
            .optional()
            .map_err(|source| StoreError::query("the current zone model", source))
    }

    /// Write a zone model, only when it is not already the one stored.
    ///
    /// A read that returns the model already stored writes nothing. A read
    /// that fails must simply not call this: the previous model, and its
    /// date, stay in place. Returns whether a row was written.
    pub fn write_hr_zone_model_if_new(&self, model: &NewHeartRateZoneModel) -> Result<bool> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM hr_zone_models
             ORDER BY read_at DESC, id DESC
             LIMIT 1"
        );
        let current = conn
            .query_row(&sql, [], map)
            .optional()
            .map_err(|source| StoreError::query("the current zone model", source))?;
        if current
            .as_ref()
            .is_some_and(|stored| same_model(stored, model))
        {
            return Ok(false);
        }
        write_model(&conn, model)?;
        Ok(true)
    }

    /// Every stored zone model, oldest first.
    pub fn hr_zone_models(&self) -> Result<Vec<HeartRateZoneModel>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM hr_zone_models ORDER BY read_at ASC, id ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the zone models", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("the zone models", source))?;
        collect(mapped)
    }

    /// How many zone models there are.
    pub fn count_hr_zone_models(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM hr_zone_models", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count the zone models", source))
    }
}

/// Insert one row of `hr_zone_models`.
fn write_model(conn: &Connection, model: &NewHeartRateZoneModel) -> Result<()> {
    conn.execute(
        "INSERT INTO hr_zone_models (
            sport, training_method, max_heart_rate,
            zone1_floor, zone2_floor, zone3_floor, zone4_floor, zone5_floor, read_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            model.sport,
            model.training_method,
            model.max_heart_rate,
            model.zone1_floor,
            model.zone2_floor,
            model.zone3_floor,
            model.zone4_floor,
            model.zone5_floor,
            DbDateTime(model.read_at),
        ],
    )
    .map_err(|source| StoreError::query("write a zone model", source))?;
    Ok(())
}
