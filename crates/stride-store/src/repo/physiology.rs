//! `daily_physiology`, `daily_metric` and `readiness`.

use rusqlite::{named_params, params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::Result;
use crate::json::DbJson;
use crate::model::{DailyMetric, DailyPhysiology, Readiness};
use crate::repo::helpers::collect;
use crate::Store;

const PHYSIOLOGY_COLUMNS: &str = "date, sleep_s, sleep_score, deep_sleep_s, rem_s, light_sleep_s, \
    awake_s, hrv_ms, hrv_status, resting_hr, max_hr, stress_avg, stress_max, body_battery_min, \
    body_battery_avg, body_battery_max, steps, training_readiness, acute_load, chronic_load, \
    vo2max, endurance_score, running_tolerance, source, updated_at";

const METRIC_COLUMNS: &str = "date, resting_hr, max_hr, sleep_s, deep_sleep_s, hrv_ms, stress, \
    steps, body_battery, readiness, distance_m";

const READINESS_COLUMNS: &str =
    "date, score, grade, drivers, recommendation, rationale, computed_at";

fn map_physiology(row: &Row<'_>) -> rusqlite::Result<DailyPhysiology> {
    Ok(DailyPhysiology {
        date: row.get("date")?,
        sleep_s: row.get("sleep_s")?,
        sleep_score: row.get("sleep_score")?,
        deep_sleep_s: row.get("deep_sleep_s")?,
        rem_s: row.get("rem_s")?,
        light_sleep_s: row.get("light_sleep_s")?,
        awake_s: row.get("awake_s")?,
        hrv_ms: row.get("hrv_ms")?,
        hrv_status: row.get("hrv_status")?,
        resting_hr: row.get("resting_hr")?,
        max_hr: row.get("max_hr")?,
        stress_avg: row.get("stress_avg")?,
        stress_max: row.get("stress_max")?,
        body_battery_min: row.get("body_battery_min")?,
        body_battery_avg: row.get("body_battery_avg")?,
        body_battery_max: row.get("body_battery_max")?,
        steps: row.get("steps")?,
        training_readiness: row.get("training_readiness")?,
        acute_load: row.get("acute_load")?,
        chronic_load: row.get("chronic_load")?,
        vo2max: row.get("vo2max")?,
        endurance_score: row.get("endurance_score")?,
        running_tolerance: row.get("running_tolerance")?,
        source: row.get("source")?,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

fn map_metric(row: &Row<'_>) -> rusqlite::Result<DailyMetric> {
    Ok(DailyMetric {
        date: row.get("date")?,
        resting_hr: row.get("resting_hr")?,
        max_hr: row.get("max_hr")?,
        sleep_s: row.get("sleep_s")?,
        deep_sleep_s: row.get("deep_sleep_s")?,
        hrv_ms: row.get("hrv_ms")?,
        stress: row.get("stress")?,
        steps: row.get("steps")?,
        body_battery: row.get("body_battery")?,
        readiness: row.get("readiness")?,
        distance_m: row.get("distance_m")?,
    })
}

fn map_readiness(row: &Row<'_>) -> rusqlite::Result<Readiness> {
    Ok(Readiness {
        date: row.get("date")?,
        score: row.get("score")?,
        grade: row.get("grade")?,
        drivers: row.get::<_, Option<DbJson>>("drivers")?.map(|v| v.0),
        recommendation: row.get("recommendation")?,
        rationale: row.get("rationale")?,
        computed_at: row.get::<_, DbDateTime>("computed_at")?.0,
    })
}

impl Store {
    /// The physiology of the days between two dates, oldest first.
    pub fn physiology(&self, from: &str, to: &str) -> Result<Vec<DailyPhysiology>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {PHYSIOLOGY_COLUMNS} FROM daily_physiology \
             WHERE date >= ? AND date <= ? ORDER BY date"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("daily_physiology".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![from, to], map_physiology)
            .map_err(|source| {
                crate::error::StoreError::query("daily_physiology".to_string(), source)
            })?;
        collect(rows, "daily_physiology")
    }

    /// The physiology of one day.
    pub fn physiology_day(&self, day: &str) -> Result<Option<DailyPhysiology>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {PHYSIOLOGY_COLUMNS} FROM daily_physiology WHERE date = ?");
        conn.query_row(&sql, params![day], map_physiology)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("daily_physiology by date".to_string(), source)
            })
    }

    /// The most recent day of physiology there is.
    pub fn latest_physiology(&self) -> Result<Option<DailyPhysiology>> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {PHYSIOLOGY_COLUMNS} FROM daily_physiology ORDER BY date DESC LIMIT 1");
        conn.query_row(&sql, [], map_physiology)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("latest daily_physiology".to_string(), source)
            })
    }

    /// Write the physiology of one day. The day is the key, so this replaces it.
    pub fn upsert_physiology(&self, physiology: &DailyPhysiology) -> Result<()> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO daily_physiology ({PHYSIOLOGY_COLUMNS})
             VALUES (:date, :sleep_s, :sleep_score, :deep_sleep_s, :rem_s, :light_sleep_s, :awake_s,
                :hrv_ms, :hrv_status, :resting_hr, :max_hr, :stress_avg, :stress_max,
                :body_battery_min, :body_battery_avg, :body_battery_max, :steps,
                :training_readiness, :acute_load, :chronic_load, :vo2max, :endurance_score,
                :running_tolerance, :source, :updated_at)
             ON CONFLICT(date) DO UPDATE SET
                sleep_s = excluded.sleep_s,
                sleep_score = excluded.sleep_score,
                deep_sleep_s = excluded.deep_sleep_s,
                rem_s = excluded.rem_s,
                light_sleep_s = excluded.light_sleep_s,
                awake_s = excluded.awake_s,
                hrv_ms = excluded.hrv_ms,
                hrv_status = excluded.hrv_status,
                resting_hr = excluded.resting_hr,
                max_hr = excluded.max_hr,
                stress_avg = excluded.stress_avg,
                stress_max = excluded.stress_max,
                body_battery_min = excluded.body_battery_min,
                body_battery_avg = excluded.body_battery_avg,
                body_battery_max = excluded.body_battery_max,
                steps = excluded.steps,
                training_readiness = excluded.training_readiness,
                acute_load = excluded.acute_load,
                chronic_load = excluded.chronic_load,
                vo2max = excluded.vo2max,
                endurance_score = excluded.endurance_score,
                running_tolerance = excluded.running_tolerance,
                source = excluded.source,
                updated_at = excluded.updated_at"
        );
        conn.execute(
            &sql,
            named_params! {
                ":date": physiology.date,
                ":sleep_s": physiology.sleep_s,
                ":sleep_score": physiology.sleep_score,
                ":deep_sleep_s": physiology.deep_sleep_s,
                ":rem_s": physiology.rem_s,
                ":light_sleep_s": physiology.light_sleep_s,
                ":awake_s": physiology.awake_s,
                ":hrv_ms": physiology.hrv_ms,
                ":hrv_status": physiology.hrv_status,
                ":resting_hr": physiology.resting_hr,
                ":max_hr": physiology.max_hr,
                ":stress_avg": physiology.stress_avg,
                ":stress_max": physiology.stress_max,
                ":body_battery_min": physiology.body_battery_min,
                ":body_battery_avg": physiology.body_battery_avg,
                ":body_battery_max": physiology.body_battery_max,
                ":steps": physiology.steps,
                ":training_readiness": physiology.training_readiness,
                ":acute_load": physiology.acute_load,
                ":chronic_load": physiology.chronic_load,
                ":vo2max": physiology.vo2max,
                ":endurance_score": physiology.endurance_score,
                ":running_tolerance": physiology.running_tolerance,
                ":source": physiology.source,
                ":updated_at": DbDateTime(physiology.updated_at),
            },
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert daily_physiology".to_string(), source)
        })?;
        Ok(())
    }

    /// The daily metrics of the days between two dates, oldest first.
    pub fn daily_metrics(&self, from: &str, to: &str) -> Result<Vec<DailyMetric>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {METRIC_COLUMNS} FROM daily_metric WHERE date >= ? AND date <= ? ORDER BY date"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("daily_metric".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![from, to], map_metric)
            .map_err(|source| {
                crate::error::StoreError::query("daily_metric".to_string(), source)
            })?;
        collect(rows, "daily_metric")
    }

    /// Write the metrics of one day. The day is the key, so this replaces it.
    pub fn upsert_daily_metric(&self, metric: &DailyMetric) -> Result<()> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO daily_metric ({METRIC_COLUMNS})
             VALUES (:date, :resting_hr, :max_hr, :sleep_s, :deep_sleep_s, :hrv_ms, :stress, :steps,
                :body_battery, :readiness, :distance_m)
             ON CONFLICT(date) DO UPDATE SET
                resting_hr = excluded.resting_hr,
                max_hr = excluded.max_hr,
                sleep_s = excluded.sleep_s,
                deep_sleep_s = excluded.deep_sleep_s,
                hrv_ms = excluded.hrv_ms,
                stress = excluded.stress,
                steps = excluded.steps,
                body_battery = excluded.body_battery,
                readiness = excluded.readiness,
                distance_m = excluded.distance_m"
        );
        conn.execute(
            &sql,
            named_params! {
                ":date": metric.date,
                ":resting_hr": metric.resting_hr,
                ":max_hr": metric.max_hr,
                ":sleep_s": metric.sleep_s,
                ":deep_sleep_s": metric.deep_sleep_s,
                ":hrv_ms": metric.hrv_ms,
                ":stress": metric.stress,
                ":steps": metric.steps,
                ":body_battery": metric.body_battery,
                ":readiness": metric.readiness,
                ":distance_m": metric.distance_m,
            },
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert daily_metric".to_string(), source)
        })?;
        Ok(())
    }

    /// The readiness of the days between two dates, oldest first.
    pub fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Readiness>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {READINESS_COLUMNS} FROM readiness WHERE date >= ? AND date <= ? ORDER BY date"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("readiness".to_string(), source))?;
        let rows = statement
            .query_map(params![from, to], map_readiness)
            .map_err(|source| crate::error::StoreError::query("readiness".to_string(), source))?;
        collect(rows, "readiness")
    }

    /// The readiness of one day.
    pub fn readiness(&self, day: &str) -> Result<Option<Readiness>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {READINESS_COLUMNS} FROM readiness WHERE date = ?");
        conn.query_row(&sql, params![day], map_readiness)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("readiness by date".to_string(), source)
            })
    }

    /// The most recent readiness there is.
    pub fn latest_readiness(&self) -> Result<Option<Readiness>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {READINESS_COLUMNS} FROM readiness ORDER BY date DESC LIMIT 1");
        conn.query_row(&sql, [], map_readiness)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("latest readiness".to_string(), source)
            })
    }

    /// Write the readiness of one day. The day is the key, so this replaces it.
    pub fn upsert_readiness(&self, readiness: &Readiness) -> Result<()> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO readiness ({READINESS_COLUMNS})
             VALUES (:date, :score, :grade, :drivers, :recommendation, :rationale, :computed_at)
             ON CONFLICT(date) DO UPDATE SET
                score = excluded.score,
                grade = excluded.grade,
                drivers = excluded.drivers,
                recommendation = excluded.recommendation,
                rationale = excluded.rationale,
                computed_at = excluded.computed_at"
        );
        conn.execute(
            &sql,
            named_params! {
                ":date": readiness.date,
                ":score": readiness.score,
                ":grade": readiness.grade,
                ":drivers": readiness.drivers.clone().map(DbJson),
                ":recommendation": readiness.recommendation,
                ":rationale": readiness.rationale,
                ":computed_at": DbDateTime(readiness.computed_at),
            },
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert readiness".to_string(), source)
        })?;
        Ok(())
    }
}
