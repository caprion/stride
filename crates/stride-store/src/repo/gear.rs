//! `gear` and `athlete` — the shoes and the person wearing them.

use rusqlite::{named_params, params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::Result;
use crate::model::{Athlete, Gear};
use crate::repo::helpers::collect;
use crate::Store;

const GEAR_COLUMNS: &str = "id, uuid, name, brand, model, status, distance_m, activity_count, \
    first_used, last_used, retire_at_m";

/// The columns of `gear` that are written. The primary key is not one of
/// them: SQLite assigns it.
const GEAR_INSERT_COLUMNS: &str = "uuid, name, brand, model, status, distance_m, activity_count, \
    first_used, last_used, retire_at_m";

const ATHLETE_COLUMNS: &str = "id, name, email, timezone, created_at";

fn map_gear(row: &Row<'_>) -> rusqlite::Result<Gear> {
    Ok(Gear {
        id: row.get("id")?,
        uuid: row.get("uuid")?,
        name: row.get("name")?,
        brand: row.get("brand")?,
        model: row.get("model")?,
        status: row.get("status")?,
        distance_m: row.get("distance_m")?,
        activity_count: row.get("activity_count")?,
        first_used: row.get("first_used")?,
        last_used: row.get("last_used")?,
        retire_at_m: row.get("retire_at_m")?,
    })
}

fn map_athlete(row: &Row<'_>) -> rusqlite::Result<Athlete> {
    Ok(Athlete {
        id: row.get("id")?,
        name: row.get("name")?,
        email: row.get("email")?,
        timezone: row.get("timezone")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
    })
}

impl Store {
    /// Every pair of shoes, in the order they were first used.
    pub fn gear(&self) -> Result<Vec<Gear>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {GEAR_COLUMNS} FROM gear ORDER BY id");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("gear".to_string(), source))?;
        let rows = statement
            .query_map([], map_gear)
            .map_err(|source| crate::error::StoreError::query("gear".to_string(), source))?;
        collect(rows, "gear")
    }

    /// One pair of shoes.
    pub fn gear_by_uuid(&self, uuid: &str) -> Result<Option<Gear>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {GEAR_COLUMNS} FROM gear WHERE uuid = ?");
        conn.query_row(&sql, params![uuid], map_gear)
            .optional()
            .map_err(|source| crate::error::StoreError::query("gear by uuid".to_string(), source))
    }

    /// Write one pair of shoes. The UUID is unique, so this replaces it.
    pub fn upsert_gear(&self, gear: &Gear) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO gear ({GEAR_INSERT_COLUMNS})
             VALUES (:uuid, :name, :brand, :model, :status, :distance_m, :activity_count,
                :first_used, :last_used, :retire_at_m)
             ON CONFLICT(uuid) DO UPDATE SET
                name = excluded.name,
                brand = excluded.brand,
                model = excluded.model,
                status = excluded.status,
                distance_m = excluded.distance_m,
                activity_count = excluded.activity_count,
                first_used = excluded.first_used,
                last_used = excluded.last_used,
                retire_at_m = excluded.retire_at_m
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":uuid": gear.uuid,
                ":name": gear.name,
                ":brand": gear.brand,
                ":model": gear.model,
                ":status": gear.status,
                ":distance_m": gear.distance_m,
                ":activity_count": gear.activity_count,
                ":first_used": gear.first_used,
                ":last_used": gear.last_used,
                ":retire_at_m": gear.retire_at_m,
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert gear".to_string(), source))
    }

    /// Every athlete.
    pub fn athletes(&self) -> Result<Vec<Athlete>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {ATHLETE_COLUMNS} FROM athlete ORDER BY id");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("athlete".to_string(), source))?;
        let rows = statement
            .query_map([], map_athlete)
            .map_err(|source| crate::error::StoreError::query("athlete".to_string(), source))?;
        collect(rows, "athlete")
    }

    /// One athlete.
    pub fn athlete(&self, id: i64) -> Result<Option<Athlete>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {ATHLETE_COLUMNS} FROM athlete WHERE id = ?");
        conn.query_row(&sql, params![id], map_athlete)
            .optional()
            .map_err(|source| crate::error::StoreError::query("athlete by id".to_string(), source))
    }

    /// Write one athlete, and return its primary key.
    pub fn upsert_athlete(&self, athlete: &Athlete) -> Result<i64> {
        let conn = self.lock()?;
        let sql = "INSERT INTO athlete (id, name, email, timezone, created_at)
             VALUES (:id, :name, :email, :timezone, :created_at)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                email = excluded.email,
                timezone = excluded.timezone
             RETURNING id";
        conn.query_row(
            sql,
            named_params! {
                ":id": athlete.id,
                ":name": athlete.name,
                ":email": athlete.email,
                ":timezone": athlete.timezone,
                ":created_at": DbDateTime(athlete.created_at),
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert athlete".to_string(), source))
    }
}
