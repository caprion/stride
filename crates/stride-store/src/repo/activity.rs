//! `activity`, `activity_lap`, `activity_stream`, `hr_zone`, `activity_fact`
//! and `run_fingerprint`.

use rusqlite::{named_params, params, Connection, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::Result;
use crate::json::DbJson;
use crate::model::{Activity, ActivityFact, ActivityLap, ActivityStream, HrZone, RunFingerprint};
use crate::repo::helpers::collect;
use crate::Store;

/// The source of the synthetic activities an older installation seeded.
pub const SYNTHETIC_SOURCE: &str = "demo";

/// The tables whose rows hang off an activity.
const ACTIVITY_CHILDREN: [&str; 5] = [
    "activity_lap",
    "activity_stream",
    "hr_zone",
    "activity_fact",
    "run_fingerprint",
];

const ACTIVITY_COLUMNS: &str = "id, athlete_id, source, external_id, name, sport_type, \
    description, start_time, local_date, day_of_week, distance_m, moving_time_s, elapsed_time_s, \
    elevation_gain_m, avg_hr, max_hr, calories, avg_cadence, max_cadence, avg_speed_ms, \
    max_speed_ms, running_time_s, walking_time_s, stopped_time_s, running_distance_m, \
    perceived_exertion, device, gear, is_trail, is_commute, polyline, geo_points, raw, \
    created_at, updated_at";

const LAP_COLUMNS: &str = "id, activity_id, lap_index, distance_m, duration_s, avg_speed_ms, \
    max_speed_ms, avg_hr, max_hr, avg_cadence, max_cadence, stride_length_m, elevation_gain_m, \
    elevation_loss_m, start_elevation_m, end_elevation_m, running_distance_m, elapsed_s";

const STREAM_COLUMNS: &str = "id, activity_id, kind, seconds, \"values\", label, unit";

const ZONE_COLUMNS: &str = "id, activity_id, zone_no, name, low_bpm, high_bpm, time_s";

const FACT_COLUMNS: &str = "activity_id, aerobic_te, anaerobic_te, training_effect_label, vo2max, \
    recovery_time_min, avg_gct_ms, avg_stride_ratio, vertical_ratio, avg_power, \
    normalized_power, avg_grade_adjusted_pace, temperature_c, impact_estimate, gear_uuid, \
    device_name";

const FINGERPRINT_COLUMNS: &str = "activity_id, running_time_s, walking_time_s, stopped_time_s, \
    cadence_avg, cadence_cv, cadence_drift_pct, hr_avg, hr_max, hr_drift_pct, hr_drift_bpm, \
    pace_drift_pct, first_half_hr, second_half_hr, first_half_pace_s_km, second_half_pace_s_km, \
    efficiency_first, efficiency_last, decoupling_pct, pace_cv, negative_split_pct, \
    stride_length_cm, form_score, zone_distribution, notes, computed_at";

/// How the activities are ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityOrder {
    /// Newest first — what the activity list uses.
    Date,
    /// Longest first.
    Distance,
    /// Fastest first: the shortest moving time.
    Pace,
}

/// The filters the activity list understands.
#[derive(Debug, Clone, Default)]
pub struct ActivityFilter {
    /// The first athlete-local date, `YYYY-MM-DD`, inclusive.
    pub from: Option<String>,
    /// The last athlete-local date, `YYYY-MM-DD`, inclusive.
    pub to: Option<String>,
    /// One sport to keep.
    pub sport: Option<String>,
    /// Several sports to keep.
    pub sports: Option<Vec<String>>,
    /// A source to leave out — `demo` is the synthetic one.
    pub exclude_source: Option<String>,
    /// Free text, matched against the name, the description and the date.
    pub search: Option<String>,
    /// How to order the rows.
    pub order: Option<ActivityOrder>,
    /// How many rows to return.
    pub limit: Option<i64>,
    /// How many rows to skip.
    pub offset: Option<i64>,
}

/// Read one row of `activity`.
fn map_activity(row: &Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id: row.get("id")?,
        athlete_id: row.get("athlete_id")?,
        source: row.get("source")?,
        external_id: row.get("external_id")?,
        name: row.get("name")?,
        sport_type: row.get("sport_type")?,
        description: row.get("description")?,
        start_time: row.get::<_, DbDateTime>("start_time")?.0,
        local_date: row.get("local_date")?,
        day_of_week: row.get("day_of_week")?,
        distance_m: row.get("distance_m")?,
        moving_time_s: row.get("moving_time_s")?,
        elapsed_time_s: row.get("elapsed_time_s")?,
        elevation_gain_m: row.get("elevation_gain_m")?,
        avg_hr: row.get("avg_hr")?,
        max_hr: row.get("max_hr")?,
        calories: row.get("calories")?,
        avg_cadence: row.get("avg_cadence")?,
        max_cadence: row.get("max_cadence")?,
        avg_speed_ms: row.get("avg_speed_ms")?,
        max_speed_ms: row.get("max_speed_ms")?,
        running_time_s: row.get("running_time_s")?,
        walking_time_s: row.get("walking_time_s")?,
        stopped_time_s: row.get("stopped_time_s")?,
        running_distance_m: row.get("running_distance_m")?,
        perceived_exertion: row.get("perceived_exertion")?,
        device: row.get("device")?,
        gear: row.get("gear")?,
        is_trail: row.get("is_trail")?,
        is_commute: row.get("is_commute")?,
        polyline: row.get::<_, Option<DbJson>>("polyline")?.map(|v| v.0),
        geo_points: row.get::<_, Option<DbJson>>("geo_points")?.map(|v| v.0),
        raw: row.get::<_, Option<DbJson>>("raw")?.map(|v| v.0),
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

/// Read one row of `activity_lap`.
fn map_lap(row: &Row<'_>) -> rusqlite::Result<ActivityLap> {
    Ok(ActivityLap {
        id: row.get("id")?,
        activity_id: row.get("activity_id")?,
        lap_index: row.get("lap_index")?,
        distance_m: row.get("distance_m")?,
        duration_s: row.get("duration_s")?,
        avg_speed_ms: row.get("avg_speed_ms")?,
        max_speed_ms: row.get("max_speed_ms")?,
        avg_hr: row.get("avg_hr")?,
        max_hr: row.get("max_hr")?,
        avg_cadence: row.get("avg_cadence")?,
        max_cadence: row.get("max_cadence")?,
        stride_length_m: row.get("stride_length_m")?,
        elevation_gain_m: row.get("elevation_gain_m")?,
        elevation_loss_m: row.get("elevation_loss_m")?,
        start_elevation_m: row.get("start_elevation_m")?,
        end_elevation_m: row.get("end_elevation_m")?,
        running_distance_m: row.get("running_distance_m")?,
        elapsed_s: row.get("elapsed_s")?,
    })
}

/// Read one row of `activity_stream`.
fn map_stream(row: &Row<'_>) -> rusqlite::Result<ActivityStream> {
    Ok(ActivityStream {
        id: row.get("id")?,
        activity_id: row.get("activity_id")?,
        kind: row.get("kind")?,
        seconds: row.get::<_, DbJson>("seconds")?.0,
        values: row.get::<_, DbJson>("values")?.0,
        label: row.get("label")?,
        unit: row.get("unit")?,
    })
}

/// Read one row of `hr_zone`.
fn map_zone(row: &Row<'_>) -> rusqlite::Result<HrZone> {
    Ok(HrZone {
        id: row.get("id")?,
        activity_id: row.get("activity_id")?,
        zone_no: row.get("zone_no")?,
        name: row.get("name")?,
        low_bpm: row.get("low_bpm")?,
        high_bpm: row.get("high_bpm")?,
        time_s: row.get("time_s")?,
    })
}

/// Read one row of `activity_fact`.
fn map_fact(row: &Row<'_>) -> rusqlite::Result<ActivityFact> {
    Ok(ActivityFact {
        activity_id: row.get("activity_id")?,
        aerobic_te: row.get("aerobic_te")?,
        anaerobic_te: row.get("anaerobic_te")?,
        training_effect_label: row.get("training_effect_label")?,
        vo2max: row.get("vo2max")?,
        recovery_time_min: row.get("recovery_time_min")?,
        avg_gct_ms: row.get("avg_gct_ms")?,
        avg_stride_ratio: row.get("avg_stride_ratio")?,
        vertical_ratio: row.get("vertical_ratio")?,
        avg_power: row.get("avg_power")?,
        normalized_power: row.get("normalized_power")?,
        avg_grade_adjusted_pace: row.get("avg_grade_adjusted_pace")?,
        temperature_c: row.get("temperature_c")?,
        impact_estimate: row.get("impact_estimate")?,
        gear_uuid: row.get("gear_uuid")?,
        device_name: row.get("device_name")?,
    })
}

/// Read one row of `run_fingerprint`.
fn map_fingerprint(row: &Row<'_>) -> rusqlite::Result<RunFingerprint> {
    Ok(RunFingerprint {
        activity_id: row.get("activity_id")?,
        running_time_s: row.get("running_time_s")?,
        walking_time_s: row.get("walking_time_s")?,
        stopped_time_s: row.get("stopped_time_s")?,
        cadence_avg: row.get("cadence_avg")?,
        cadence_cv: row.get("cadence_cv")?,
        cadence_drift_pct: row.get("cadence_drift_pct")?,
        hr_avg: row.get("hr_avg")?,
        hr_max: row.get("hr_max")?,
        hr_drift_pct: row.get("hr_drift_pct")?,
        hr_drift_bpm: row.get("hr_drift_bpm")?,
        pace_drift_pct: row.get("pace_drift_pct")?,
        first_half_hr: row.get("first_half_hr")?,
        second_half_hr: row.get("second_half_hr")?,
        first_half_pace_s_km: row.get("first_half_pace_s_km")?,
        second_half_pace_s_km: row.get("second_half_pace_s_km")?,
        efficiency_first: row.get("efficiency_first")?,
        efficiency_last: row.get("efficiency_last")?,
        decoupling_pct: row.get("decoupling_pct")?,
        pace_cv: row.get("pace_cv")?,
        negative_split_pct: row.get("negative_split_pct")?,
        stride_length_cm: row.get("stride_length_cm")?,
        form_score: row.get("form_score")?,
        zone_distribution: row
            .get::<_, Option<DbJson>>("zone_distribution")?
            .map(|v| v.0),
        notes: row.get("notes")?,
        computed_at: row.get::<_, DbDateTime>("computed_at")?.0,
    })
}

/// The `WHERE` clause of an activity query, and the values it binds.
fn activity_where(filter: &ActivityFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(from) = &filter.from {
        clauses.push("local_date >= ?".to_string());
        values.push(Box::new(from.clone()));
    }
    if let Some(to) = &filter.to {
        clauses.push("local_date <= ?".to_string());
        values.push(Box::new(to.clone()));
    }
    if let Some(sport) = &filter.sport {
        clauses.push("sport_type = ?".to_string());
        values.push(Box::new(sport.clone()));
    }
    if let Some(sports) = &filter.sports {
        if !sports.is_empty() {
            let marks = vec!["?"; sports.len()].join(", ");
            clauses.push(format!("sport_type IN ({marks})"));
            for sport in sports {
                values.push(Box::new(sport.clone()));
            }
        }
    }
    if let Some(source) = &filter.exclude_source {
        clauses.push("source != ?".to_string());
        values.push(Box::new(source.clone()));
    }
    if let Some(search) = &filter.search {
        let pattern = format!("%{}%", search.trim());
        clauses.push(
            "(name LIKE ? COLLATE NOCASE OR description LIKE ? COLLATE NOCASE \
             OR local_date LIKE ? COLLATE NOCASE)"
                .to_string(),
        );
        values.push(Box::new(pattern.clone()));
        values.push(Box::new(pattern.clone()));
        values.push(Box::new(pattern));
    }

    let where_clause = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    (where_clause, values)
}

/// The `ORDER BY` clause of an activity query.
fn activity_order(order: Option<ActivityOrder>) -> &'static str {
    match order.unwrap_or(ActivityOrder::Date) {
        ActivityOrder::Date => " ORDER BY start_time DESC, id DESC",
        ActivityOrder::Distance => " ORDER BY distance_m DESC, id DESC",
        ActivityOrder::Pace => " ORDER BY moving_time_s ASC, id DESC",
    }
}

impl Store {
    /// The activities between two athlete-local dates, newest first.
    pub fn activities(&self, from: &str, to: &str) -> Result<Vec<Activity>> {
        self.activities_filtered(&ActivityFilter {
            from: Some(from.to_string()),
            to: Some(to.to_string()),
            ..ActivityFilter::default()
        })
    }

    /// The activities that match a filter.
    pub fn activities_filtered(&self, filter: &ActivityFilter) -> Result<Vec<Activity>> {
        let conn = self.lock()?;
        let (where_clause, values) = activity_where(filter);
        let sql = format!(
            "SELECT {ACTIVITY_COLUMNS} FROM activity{where_clause}{} LIMIT {} OFFSET {}",
            activity_order(filter.order),
            filter.limit.unwrap_or(-1),
            filter.offset.unwrap_or(0)
        );

        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("activity (filtered)".to_string(), source)
        })?;
        let bound: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let rows = statement
            .query_map(bound.as_slice(), map_activity)
            .map_err(|source| {
                crate::error::StoreError::query("activity (filtered)".to_string(), source)
            })?;
        collect(rows, "activity (filtered)")
    }

    /// How many activities match a filter.
    pub fn count_activities(&self, filter: &ActivityFilter) -> Result<i64> {
        let conn = self.lock()?;
        let (where_clause, values) = activity_where(filter);
        let sql = format!("SELECT count(*) FROM activity{where_clause}");

        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("activity count".to_string(), source)
        })?;
        let bound: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let count = statement
            .query_row(bound.as_slice(), |row| row.get::<_, i64>(0))
            .map_err(|source| {
                crate::error::StoreError::query("activity count".to_string(), source)
            })?;
        Ok(count)
    }

    /// One activity, by its primary key.
    pub fn activity(&self, id: i64) -> Result<Option<Activity>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {ACTIVITY_COLUMNS} FROM activity WHERE id = ?");
        conn.query_row(&sql, params![id], map_activity)
            .optional()
            .map_err(|source| crate::error::StoreError::query("activity by id".to_string(), source))
    }

    /// One activity, by the provider's identifier.
    pub fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<Activity>> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {ACTIVITY_COLUMNS} FROM activity WHERE source = ? AND external_id = ?");
        conn.query_row(&sql, params![source, external_id], map_activity)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("activity by external id".to_string(), source)
            })
    }

    /// Write one activity, and return its primary key.
    ///
    /// Idempotent, like the reference: `(source, external_id)` is unique, so
    /// re-syncing an activity updates it in place instead of duplicating it.
    /// `created_at` is kept; `updated_at` is the value the row carries, so
    /// call `touch()` before writing a row that has changed.
    pub fn upsert_activity(&self, activity: &Activity) -> Result<i64> {
        let conn = self.lock()?;
        let sql = "INSERT INTO activity (
                athlete_id, source, external_id, name, sport_type, description, start_time,
                local_date, day_of_week, distance_m, moving_time_s, elapsed_time_s,
                elevation_gain_m, avg_hr, max_hr, calories, avg_cadence, max_cadence, avg_speed_ms,
                max_speed_ms, running_time_s, walking_time_s, stopped_time_s, running_distance_m,
                perceived_exertion, device, gear, is_trail, is_commute, polyline, geo_points, raw,
                created_at, updated_at
             ) VALUES (
                :athlete_id, :source, :external_id, :name, :sport_type, :description,
                :start_time, :local_date, :day_of_week, :distance_m, :moving_time_s,
                :elapsed_time_s, :elevation_gain_m, :avg_hr, :max_hr, :calories, :avg_cadence,
                :max_cadence, :avg_speed_ms, :max_speed_ms, :running_time_s, :walking_time_s,
                :stopped_time_s, :running_distance_m, :perceived_exertion, :device, :gear,
                :is_trail, :is_commute, :polyline, :geo_points, :raw, :created_at, :updated_at
             )
             ON CONFLICT(source, external_id) DO UPDATE SET
                athlete_id = excluded.athlete_id,
                name = excluded.name,
                sport_type = excluded.sport_type,
                description = excluded.description,
                start_time = excluded.start_time,
                local_date = excluded.local_date,
                day_of_week = excluded.day_of_week,
                distance_m = excluded.distance_m,
                moving_time_s = excluded.moving_time_s,
                elapsed_time_s = excluded.elapsed_time_s,
                elevation_gain_m = excluded.elevation_gain_m,
                avg_hr = excluded.avg_hr,
                max_hr = excluded.max_hr,
                calories = excluded.calories,
                avg_cadence = excluded.avg_cadence,
                max_cadence = excluded.max_cadence,
                avg_speed_ms = excluded.avg_speed_ms,
                max_speed_ms = excluded.max_speed_ms,
                running_time_s = excluded.running_time_s,
                walking_time_s = excluded.walking_time_s,
                stopped_time_s = excluded.stopped_time_s,
                running_distance_m = excluded.running_distance_m,
                perceived_exertion = excluded.perceived_exertion,
                device = excluded.device,
                gear = excluded.gear,
                is_trail = excluded.is_trail,
                is_commute = excluded.is_commute,
                polyline = excluded.polyline,
                geo_points = excluded.geo_points,
                raw = excluded.raw,
                updated_at = excluded.updated_at
             RETURNING id";

        let id = conn
            .query_row(
                sql,
                named_params! {
                    ":athlete_id": activity.athlete_id,
                    ":source": activity.source,
                    ":external_id": activity.external_id,
                    ":name": activity.name,
                    ":sport_type": activity.sport_type,
                    ":description": activity.description,
                    ":start_time": DbDateTime(activity.start_time),
                    ":local_date": activity.local_date,
                    ":day_of_week": activity.day_of_week,
                    ":distance_m": activity.distance_m,
                    ":moving_time_s": activity.moving_time_s,
                    ":elapsed_time_s": activity.elapsed_time_s,
                    ":elevation_gain_m": activity.elevation_gain_m,
                    ":avg_hr": activity.avg_hr,
                    ":max_hr": activity.max_hr,
                    ":calories": activity.calories,
                    ":avg_cadence": activity.avg_cadence,
                    ":max_cadence": activity.max_cadence,
                    ":avg_speed_ms": activity.avg_speed_ms,
                    ":max_speed_ms": activity.max_speed_ms,
                    ":running_time_s": activity.running_time_s,
                    ":walking_time_s": activity.walking_time_s,
                    ":stopped_time_s": activity.stopped_time_s,
                    ":running_distance_m": activity.running_distance_m,
                    ":perceived_exertion": activity.perceived_exertion,
                    ":device": activity.device,
                    ":gear": activity.gear,
                    ":is_trail": activity.is_trail,
                    ":is_commute": activity.is_commute,
                    ":polyline": activity.polyline.clone().map(DbJson),
                    ":geo_points": activity.geo_points.clone().map(DbJson),
                    ":raw": activity.raw.clone().map(DbJson),
                    ":created_at": DbDateTime(activity.created_at),
                    ":updated_at": DbDateTime(activity.updated_at),
                },
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| {
                crate::error::StoreError::query("upsert activity".to_string(), source)
            })?;
        Ok(id)
    }

    /// The laps of one activity, in lap order.
    pub fn activity_laps(&self, activity_id: i64) -> Result<Vec<ActivityLap>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {LAP_COLUMNS} FROM activity_lap WHERE activity_id = ? ORDER BY lap_index, id"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("activity_lap".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![activity_id], map_lap)
            .map_err(|source| {
                crate::error::StoreError::query("activity_lap".to_string(), source)
            })?;
        collect(rows, "activity_lap")
    }

    /// Replace the laps of one activity.
    ///
    /// The reference deletes every lap and writes the new ones, so a lap that
    /// has gone is gone.
    pub fn replace_activity_laps(&self, activity_id: i64, laps: &[ActivityLap]) -> Result<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(|source| {
            crate::error::StoreError::query("activity_lap".to_string(), source)
        })?;

        tx.execute(
            "DELETE FROM activity_lap WHERE activity_id = ?",
            params![activity_id],
        )
        .map_err(|source| crate::error::StoreError::query("activity_lap".to_string(), source))?;

        for lap in laps {
            let sql = "INSERT INTO activity_lap (
                activity_id, lap_index, distance_m, duration_s, avg_speed_ms, max_speed_ms,
                avg_hr, max_hr, avg_cadence, max_cadence, stride_length_m, elevation_gain_m,
                elevation_loss_m, start_elevation_m, end_elevation_m, running_distance_m, elapsed_s
             ) VALUES (
                :activity_id, :lap_index, :distance_m, :duration_s, :avg_speed_ms,
                :max_speed_ms, :avg_hr, :max_hr, :avg_cadence, :max_cadence, :stride_length_m,
                :elevation_gain_m, :elevation_loss_m, :start_elevation_m, :end_elevation_m,
                :running_distance_m, :elapsed_s
             )";
            tx.execute(
                sql,
                named_params! {
                    ":activity_id": activity_id,
                    ":lap_index": lap.lap_index,
                    ":distance_m": lap.distance_m,
                    ":duration_s": lap.duration_s,
                    ":avg_speed_ms": lap.avg_speed_ms,
                    ":max_speed_ms": lap.max_speed_ms,
                    ":avg_hr": lap.avg_hr,
                    ":max_hr": lap.max_hr,
                    ":avg_cadence": lap.avg_cadence,
                    ":max_cadence": lap.max_cadence,
                    ":stride_length_m": lap.stride_length_m,
                    ":elevation_gain_m": lap.elevation_gain_m,
                    ":elevation_loss_m": lap.elevation_loss_m,
                    ":start_elevation_m": lap.start_elevation_m,
                    ":end_elevation_m": lap.end_elevation_m,
                    ":running_distance_m": lap.running_distance_m,
                    ":elapsed_s": lap.elapsed_s,
                },
            )
            .map_err(|source| {
                crate::error::StoreError::query("activity_lap".to_string(), source)
            })?;
        }

        tx.commit()
            .map_err(|source| crate::error::StoreError::query("activity_lap".to_string(), source))
    }

    /// The streams of one activity.
    pub fn activity_streams(&self, activity_id: i64) -> Result<Vec<ActivityStream>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {STREAM_COLUMNS} FROM activity_stream WHERE activity_id = ? ORDER BY kind"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("activity_stream".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![activity_id], map_stream)
            .map_err(|source| {
                crate::error::StoreError::query("activity_stream".to_string(), source)
            })?;
        collect(rows, "activity_stream")
    }

    /// Write one stream. `(activity_id, kind)` is unique, so this replaces the
    /// stream of that kind rather than adding a second one.
    pub fn upsert_activity_stream(&self, stream: &ActivityStream) -> Result<i64> {
        let conn = self.lock()?;
        let sql =
            "INSERT INTO activity_stream (activity_id, kind, seconds, \"values\", label, unit)
             VALUES (:activity_id, :kind, :seconds, :values, :label, :unit)
             ON CONFLICT(activity_id, kind) DO UPDATE SET
                seconds = excluded.seconds,
                \"values\" = excluded.\"values\",
                label = excluded.label,
                unit = excluded.unit
             RETURNING id";
        let id = conn
            .query_row(
                sql,
                named_params! {
                    ":activity_id": stream.activity_id,
                    ":kind": stream.kind,
                    ":seconds": DbJson(stream.seconds.clone()),
                    ":values": DbJson(stream.values.clone()),
                    ":label": stream.label,
                    ":unit": stream.unit,
                },
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| {
                crate::error::StoreError::query("upsert activity_stream".to_string(), source)
            })?;
        Ok(id)
    }

    /// Write the streams of one activity.
    ///
    /// Like the reference, a stream with no values is not written, and a kind
    /// that is not in the list is left alone.
    pub fn replace_activity_streams(
        &self,
        activity_id: i64,
        streams: &[ActivityStream],
    ) -> Result<()> {
        for stream in streams {
            let empty = match &stream.values {
                serde_json::Value::Array(values) => values.is_empty(),
                serde_json::Value::Null => true,
                _ => false,
            };
            if empty {
                continue;
            }
            let mut stream = stream.clone();
            stream.activity_id = activity_id;
            self.upsert_activity_stream(&stream)?;
        }
        Ok(())
    }

    /// The heart-rate zones of one activity, in zone order.
    pub fn hr_zones(&self, activity_id: i64) -> Result<Vec<HrZone>> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {ZONE_COLUMNS} FROM hr_zone WHERE activity_id = ? ORDER BY zone_no");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;
        let rows = statement
            .query_map(params![activity_id], map_zone)
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;
        collect(rows, "hr_zone")
    }

    /// The heart-rate zones of every activity between two dates.
    pub fn hr_zones_between(&self, from: &str, to: &str) -> Result<Vec<HrZone>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {ZONE_COLUMNS} FROM hr_zone \
             JOIN activity ON activity.id = hr_zone.activity_id \
             WHERE activity.local_date >= ? AND activity.local_date <= ? \
             ORDER BY activity.local_date, hr_zone.zone_no"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;
        let rows = statement
            .query_map(params![from, to], map_zone)
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;
        collect(rows, "hr_zone")
    }

    /// Replace the heart-rate zones of one activity.
    pub fn replace_hr_zones(&self, activity_id: i64, zones: &[HrZone]) -> Result<()> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction()
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;

        tx.execute(
            "DELETE FROM hr_zone WHERE activity_id = ?",
            params![activity_id],
        )
        .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;

        for zone in zones {
            tx.execute(
                "INSERT INTO hr_zone (activity_id, zone_no, name, low_bpm, high_bpm, time_s)
                 VALUES (:activity_id, :zone_no, :name, :low_bpm, :high_bpm, :time_s)",
                named_params! {
                    ":activity_id": activity_id,
                    ":zone_no": zone.zone_no,
                    ":name": zone.name,
                    ":low_bpm": zone.low_bpm,
                    ":high_bpm": zone.high_bpm,
                    ":time_s": zone.time_s,
                },
            )
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))?;
        }

        tx.commit()
            .map_err(|source| crate::error::StoreError::query("hr_zone".to_string(), source))
    }

    /// The Garmin-only facts of one activity.
    pub fn activity_fact(&self, activity_id: i64) -> Result<Option<ActivityFact>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {FACT_COLUMNS} FROM activity_fact WHERE activity_id = ?");
        conn.query_row(&sql, params![activity_id], map_fact)
            .optional()
            .map_err(|source| crate::error::StoreError::query("activity_fact".to_string(), source))
    }

    /// Write the Garmin-only facts of one activity.
    pub fn upsert_activity_fact(&self, fact: &ActivityFact) -> Result<()> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO activity_fact ({FACT_COLUMNS})
             VALUES (:activity_id, :aerobic_te, :anaerobic_te, :training_effect_label, :vo2max,
                :recovery_time_min, :avg_gct_ms, :avg_stride_ratio, :vertical_ratio, :avg_power,
                :normalized_power, :avg_grade_adjusted_pace, :temperature_c, :impact_estimate,
                :gear_uuid, :device_name)
             ON CONFLICT(activity_id) DO UPDATE SET
                aerobic_te = excluded.aerobic_te,
                anaerobic_te = excluded.anaerobic_te,
                training_effect_label = excluded.training_effect_label,
                vo2max = excluded.vo2max,
                recovery_time_min = excluded.recovery_time_min,
                avg_gct_ms = excluded.avg_gct_ms,
                avg_stride_ratio = excluded.avg_stride_ratio,
                vertical_ratio = excluded.vertical_ratio,
                avg_power = excluded.avg_power,
                normalized_power = excluded.normalized_power,
                avg_grade_adjusted_pace = excluded.avg_grade_adjusted_pace,
                temperature_c = excluded.temperature_c,
                impact_estimate = excluded.impact_estimate,
                gear_uuid = excluded.gear_uuid,
                device_name = excluded.device_name"
        );
        conn.execute(
            &sql,
            named_params! {
                ":activity_id": fact.activity_id,
                ":aerobic_te": fact.aerobic_te,
                ":anaerobic_te": fact.anaerobic_te,
                ":training_effect_label": fact.training_effect_label,
                ":vo2max": fact.vo2max,
                ":recovery_time_min": fact.recovery_time_min,
                ":avg_gct_ms": fact.avg_gct_ms,
                ":avg_stride_ratio": fact.avg_stride_ratio,
                ":vertical_ratio": fact.vertical_ratio,
                ":avg_power": fact.avg_power,
                ":normalized_power": fact.normalized_power,
                ":avg_grade_adjusted_pace": fact.avg_grade_adjusted_pace,
                ":temperature_c": fact.temperature_c,
                ":impact_estimate": fact.impact_estimate,
                ":gear_uuid": fact.gear_uuid,
                ":device_name": fact.device_name,
            },
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert activity_fact".to_string(), source)
        })?;
        Ok(())
    }

    /// The fingerprint of one activity.
    pub fn run_fingerprint(&self, activity_id: i64) -> Result<Option<RunFingerprint>> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {FINGERPRINT_COLUMNS} FROM run_fingerprint WHERE activity_id = ?");
        conn.query_row(&sql, params![activity_id], map_fingerprint)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("run_fingerprint".to_string(), source)
            })
    }

    /// The fingerprints of every activity between two dates.
    pub fn run_fingerprints_between(&self, from: &str, to: &str) -> Result<Vec<RunFingerprint>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {FINGERPRINT_COLUMNS} FROM run_fingerprint \
             JOIN activity ON activity.id = run_fingerprint.activity_id \
             WHERE activity.local_date >= ? AND activity.local_date <= ? \
             ORDER BY activity.local_date"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("run_fingerprint".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![from, to], map_fingerprint)
            .map_err(|source| {
                crate::error::StoreError::query("run_fingerprint".to_string(), source)
            })?;
        collect(rows, "run_fingerprint")
    }

    /// Write the fingerprint of one activity.
    pub fn upsert_run_fingerprint(&self, fingerprint: &RunFingerprint) -> Result<()> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO run_fingerprint ({FINGERPRINT_COLUMNS})
             VALUES (:activity_id, :running_time_s, :walking_time_s, :stopped_time_s, :cadence_avg,
                :cadence_cv, :cadence_drift_pct, :hr_avg, :hr_max, :hr_drift_pct, :hr_drift_bpm,
                :pace_drift_pct, :first_half_hr, :second_half_hr, :first_half_pace_s_km,
                :second_half_pace_s_km, :efficiency_first, :efficiency_last, :decoupling_pct,
                :pace_cv, :negative_split_pct, :stride_length_cm, :form_score,
                :zone_distribution, :notes, :computed_at)
             ON CONFLICT(activity_id) DO UPDATE SET
                running_time_s = excluded.running_time_s,
                walking_time_s = excluded.walking_time_s,
                stopped_time_s = excluded.stopped_time_s,
                cadence_avg = excluded.cadence_avg,
                cadence_cv = excluded.cadence_cv,
                cadence_drift_pct = excluded.cadence_drift_pct,
                hr_avg = excluded.hr_avg,
                hr_max = excluded.hr_max,
                hr_drift_pct = excluded.hr_drift_pct,
                hr_drift_bpm = excluded.hr_drift_bpm,
                pace_drift_pct = excluded.pace_drift_pct,
                first_half_hr = excluded.first_half_hr,
                second_half_hr = excluded.second_half_hr,
                first_half_pace_s_km = excluded.first_half_pace_s_km,
                second_half_pace_s_km = excluded.second_half_pace_s_km,
                efficiency_first = excluded.efficiency_first,
                efficiency_last = excluded.efficiency_last,
                decoupling_pct = excluded.decoupling_pct,
                pace_cv = excluded.pace_cv,
                negative_split_pct = excluded.negative_split_pct,
                stride_length_cm = excluded.stride_length_cm,
                form_score = excluded.form_score,
                zone_distribution = excluded.zone_distribution,
                notes = excluded.notes,
                computed_at = excluded.computed_at"
        );
        conn.execute(
            &sql,
            named_params! {
                ":activity_id": fingerprint.activity_id,
                ":running_time_s": fingerprint.running_time_s,
                ":walking_time_s": fingerprint.walking_time_s,
                ":stopped_time_s": fingerprint.stopped_time_s,
                ":cadence_avg": fingerprint.cadence_avg,
                ":cadence_cv": fingerprint.cadence_cv,
                ":cadence_drift_pct": fingerprint.cadence_drift_pct,
                ":hr_avg": fingerprint.hr_avg,
                ":hr_max": fingerprint.hr_max,
                ":hr_drift_pct": fingerprint.hr_drift_pct,
                ":hr_drift_bpm": fingerprint.hr_drift_bpm,
                ":pace_drift_pct": fingerprint.pace_drift_pct,
                ":first_half_hr": fingerprint.first_half_hr,
                ":second_half_hr": fingerprint.second_half_hr,
                ":first_half_pace_s_km": fingerprint.first_half_pace_s_km,
                ":second_half_pace_s_km": fingerprint.second_half_pace_s_km,
                ":efficiency_first": fingerprint.efficiency_first,
                ":efficiency_last": fingerprint.efficiency_last,
                ":decoupling_pct": fingerprint.decoupling_pct,
                ":pace_cv": fingerprint.pace_cv,
                ":negative_split_pct": fingerprint.negative_split_pct,
                ":stride_length_cm": fingerprint.stride_length_cm,
                ":form_score": fingerprint.form_score,
                ":zone_distribution": fingerprint.zone_distribution.clone().map(DbJson),
                ":notes": fingerprint.notes,
                ":computed_at": DbDateTime(fingerprint.computed_at),
            },
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert run_fingerprint".to_string(), source)
        })?;
        Ok(())
    }
}

/// Remove the synthetic activities an older installation still holds.
///
/// The demo dataset is gone — it is not seeded, not filtered and not behind
/// a flag — but an installation that was seeded before it was removed still
/// has those rows in its database. They were seeded at the athlete's **own**
/// coordinates and their own city, and they look exactly like the athlete's
/// own runs, so they are removed once, on every open, and never come back.
///
/// The children go with them: the laps, the streams, the heart-rate zones,
/// the facts and the fingerprint are all rows keyed on the activity. A plan
/// result that pointed at one of them keeps its row and loses the pointer,
/// which is what the schema's `ON DELETE SET NULL` says.
///
/// The reference is `backend/app/services/persistence.py::purge_synthetic_activities`.
///
/// It returns the number of activities it removed, so a caller can say so.
pub fn purge_synthetic_activities(conn: &Connection) -> Result<i64> {
    let ids = {
        let mut statement = conn
            .prepare("SELECT id FROM activity WHERE source = ?")
            .map_err(|source| {
                crate::error::StoreError::query("the synthetic activities".to_string(), source)
            })?;
        let rows = statement
            .query_map(params![SYNTHETIC_SOURCE], |row| row.get::<_, i64>(0))
            .map_err(|source| {
                crate::error::StoreError::query("the synthetic activities".to_string(), source)
            })?;
        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|source| {
                crate::error::StoreError::query("the synthetic activities".to_string(), source)
            })?);
        }
        ids
    };
    if ids.is_empty() {
        return Ok(0);
    }

    // The children go first, so the purge does not depend on the foreign
    // keys being enforced: a database opened with the pragma off would
    // otherwise keep the laps of a run that is not there.
    let marks = vec!["?"; ids.len()].join(", ");
    for table in ACTIVITY_CHILDREN {
        let sql = format!("DELETE FROM {table} WHERE activity_id IN ({marks})");
        conn.execute(&sql, rusqlite::params_from_iter(ids.iter()))
            .map_err(|source| {
                crate::error::StoreError::query(format!("the synthetic {table}"), source)
            })?;
    }
    let removed = conn
        .execute(
            "DELETE FROM activity WHERE source = ?",
            params![SYNTHETIC_SOURCE],
        )
        .map_err(|source| {
            crate::error::StoreError::query("the synthetic activities".to_string(), source)
        })?;
    Ok(removed as i64)
}
