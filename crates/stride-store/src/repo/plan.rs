//! `plan_block`, `plan_phase`, `plan_week`, `plan_session` and `plan_result`.

use rusqlite::{named_params, params, OptionalExtension, Row};

use crate::datetimes::DbDateTime;
use crate::error::Result;
use crate::model::{PlanBlock, PlanPhase, PlanResult, PlanSession, PlanWeek};
use crate::repo::helpers::collect;
use crate::Store;

const BLOCK_COLUMNS: &str =
    "id, name, start_date, end_date, goal, status, source, notes, created_at";

const PHASE_COLUMNS: &str = "id, block_id, name, start_date, end_date, target_weekly_km_min, \
    target_weekly_km_max, target_sessions, pace_easy_low, pace_easy_high, pace_tempo_low, \
    pace_tempo_high, pace_interval_low, pace_interval_high, pace_long_low, pace_long_high, \
    hr_zone_target, notes";

const WEEK_COLUMNS: &str =
    "id, phase_id, week_number, start_date, end_date, target_km, key_workout, notes";

const SESSION_COLUMNS: &str = "id, week_id, block_id, date, session_type, title, details, \
    target_distance_m, pace_low_s_km, pace_high_s_km, hr_zone_target, status, source, notes";

const RESULT_COLUMNS: &str = "id, plan_session_id, activity_id, verdict, distance_delta_m, \
    pace_delta_s_km, pace_band_hit, distance_hit, adherence_score, cause, cause_confidence, note, \
    computed_at";

fn map_block(row: &Row<'_>) -> rusqlite::Result<PlanBlock> {
    Ok(PlanBlock {
        id: row.get("id")?,
        name: row.get("name")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        goal: row.get("goal")?,
        status: row.get("status")?,
        source: row.get("source")?,
        notes: row.get("notes")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
    })
}

fn map_phase(row: &Row<'_>) -> rusqlite::Result<PlanPhase> {
    Ok(PlanPhase {
        id: row.get("id")?,
        block_id: row.get("block_id")?,
        name: row.get("name")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        target_weekly_km_min: row.get("target_weekly_km_min")?,
        target_weekly_km_max: row.get("target_weekly_km_max")?,
        target_sessions: row.get("target_sessions")?,
        pace_easy_low: row.get("pace_easy_low")?,
        pace_easy_high: row.get("pace_easy_high")?,
        pace_tempo_low: row.get("pace_tempo_low")?,
        pace_tempo_high: row.get("pace_tempo_high")?,
        pace_interval_low: row.get("pace_interval_low")?,
        pace_interval_high: row.get("pace_interval_high")?,
        pace_long_low: row.get("pace_long_low")?,
        pace_long_high: row.get("pace_long_high")?,
        hr_zone_target: row.get("hr_zone_target")?,
        notes: row.get("notes")?,
    })
}

fn map_week(row: &Row<'_>) -> rusqlite::Result<PlanWeek> {
    Ok(PlanWeek {
        id: row.get("id")?,
        phase_id: row.get("phase_id")?,
        week_number: row.get("week_number")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        target_km: row.get("target_km")?,
        key_workout: row.get("key_workout")?,
        notes: row.get("notes")?,
    })
}

fn map_session(row: &Row<'_>) -> rusqlite::Result<PlanSession> {
    Ok(PlanSession {
        id: row.get("id")?,
        week_id: row.get("week_id")?,
        block_id: row.get("block_id")?,
        date: row.get("date")?,
        session_type: row.get("session_type")?,
        title: row.get("title")?,
        details: row.get("details")?,
        target_distance_m: row.get("target_distance_m")?,
        pace_low_s_km: row.get("pace_low_s_km")?,
        pace_high_s_km: row.get("pace_high_s_km")?,
        hr_zone_target: row.get("hr_zone_target")?,
        status: row.get("status")?,
        source: row.get("source")?,
        notes: row.get("notes")?,
    })
}

fn map_result(row: &Row<'_>) -> rusqlite::Result<PlanResult> {
    Ok(PlanResult {
        id: row.get("id")?,
        plan_session_id: row.get("plan_session_id")?,
        activity_id: row.get("activity_id")?,
        verdict: row.get("verdict")?,
        distance_delta_m: row.get("distance_delta_m")?,
        pace_delta_s_km: row.get("pace_delta_s_km")?,
        pace_band_hit: row.get("pace_band_hit")?,
        distance_hit: row.get("distance_hit")?,
        adherence_score: row.get("adherence_score")?,
        cause: row.get("cause")?,
        cause_confidence: row.get("cause_confidence")?,
        note: row.get("note")?,
        computed_at: row.get::<_, DbDateTime>("computed_at")?.0,
    })
}

impl Store {
    /// Every training block, oldest first.
    pub fn plan_blocks(&self) -> Result<Vec<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {BLOCK_COLUMNS} FROM plan_block ORDER BY start_date, id");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("plan_block".to_string(), source))?;
        let rows = statement
            .query_map([], map_block)
            .map_err(|source| crate::error::StoreError::query("plan_block".to_string(), source))?;
        collect(rows, "plan_block")
    }

    /// One training block.
    pub fn plan_block(&self, id: i64) -> Result<Option<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {BLOCK_COLUMNS} FROM plan_block WHERE id = ?");
        conn.query_row(&sql, params![id], map_block)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("plan_block by id".to_string(), source)
            })
    }

    /// The block that is active today, if there is one.
    pub fn current_plan_block(&self, day: &str) -> Result<Option<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {BLOCK_COLUMNS} FROM plan_block \
             WHERE status = 'active' AND start_date <= ? AND (end_date IS NULL OR end_date >= ?) \
             ORDER BY start_date DESC, id DESC LIMIT 1"
        );
        conn.query_row(&sql, params![day, day], map_block)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("current plan_block".to_string(), source)
            })
    }

    /// Write one training block, and return its primary key.
    pub fn upsert_plan_block(&self, block: &PlanBlock) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO plan_block ({BLOCK_COLUMNS})
             VALUES (:id, :name, :start_date, :end_date, :goal, :status, :source, :notes, :created_at)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                goal = excluded.goal,
                status = excluded.status,
                source = excluded.source,
                notes = excluded.notes
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": block.id,
                ":name": block.name,
                ":start_date": block.start_date,
                ":end_date": block.end_date,
                ":goal": block.goal,
                ":status": block.status,
                ":source": block.source,
                ":notes": block.notes,
                ":created_at": DbDateTime(block.created_at),
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert plan_block".to_string(), source))
    }

    /// The phases of one block.
    pub fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhase>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {PHASE_COLUMNS} FROM plan_phase WHERE block_id = ? ORDER BY start_date, id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("plan_phase".to_string(), source))?;
        let rows = statement
            .query_map(params![block_id], map_phase)
            .map_err(|source| crate::error::StoreError::query("plan_phase".to_string(), source))?;
        collect(rows, "plan_phase")
    }

    /// Write one phase, and return its primary key.
    pub fn upsert_plan_phase(&self, phase: &PlanPhase) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO plan_phase ({PHASE_COLUMNS})
             VALUES (:id, :block_id, :name, :start_date, :end_date, :target_weekly_km_min,
                :target_weekly_km_max, :target_sessions, :pace_easy_low, :pace_easy_high,
                :pace_tempo_low, :pace_tempo_high, :pace_interval_low, :pace_interval_high,
                :pace_long_low, :pace_long_high, :hr_zone_target, :notes)
             ON CONFLICT(id) DO UPDATE SET
                block_id = excluded.block_id,
                name = excluded.name,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                target_weekly_km_min = excluded.target_weekly_km_min,
                target_weekly_km_max = excluded.target_weekly_km_max,
                target_sessions = excluded.target_sessions,
                pace_easy_low = excluded.pace_easy_low,
                pace_easy_high = excluded.pace_easy_high,
                pace_tempo_low = excluded.pace_tempo_low,
                pace_tempo_high = excluded.pace_tempo_high,
                pace_interval_low = excluded.pace_interval_low,
                pace_interval_high = excluded.pace_interval_high,
                pace_long_low = excluded.pace_long_low,
                pace_long_high = excluded.pace_long_high,
                hr_zone_target = excluded.hr_zone_target,
                notes = excluded.notes
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": phase.id,
                ":block_id": phase.block_id,
                ":name": phase.name,
                ":start_date": phase.start_date,
                ":end_date": phase.end_date,
                ":target_weekly_km_min": phase.target_weekly_km_min,
                ":target_weekly_km_max": phase.target_weekly_km_max,
                ":target_sessions": phase.target_sessions,
                ":pace_easy_low": phase.pace_easy_low,
                ":pace_easy_high": phase.pace_easy_high,
                ":pace_tempo_low": phase.pace_tempo_low,
                ":pace_tempo_high": phase.pace_tempo_high,
                ":pace_interval_low": phase.pace_interval_low,
                ":pace_interval_high": phase.pace_interval_high,
                ":pace_long_low": phase.pace_long_low,
                ":pace_long_high": phase.pace_long_high,
                ":hr_zone_target": phase.hr_zone_target,
                ":notes": phase.notes,
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert plan_phase".to_string(), source))
    }

    /// The weeks of one phase.
    pub fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeek>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {WEEK_COLUMNS} FROM plan_week WHERE phase_id = ? ORDER BY week_number, id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("plan_week".to_string(), source))?;
        let rows = statement
            .query_map(params![phase_id], map_week)
            .map_err(|source| crate::error::StoreError::query("plan_week".to_string(), source))?;
        collect(rows, "plan_week")
    }

    /// Write one week, and return its primary key.
    pub fn upsert_plan_week(&self, week: &PlanWeek) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO plan_week ({WEEK_COLUMNS})
             VALUES (:id, :phase_id, :week_number, :start_date, :end_date, :target_km,
                :key_workout, :notes)
             ON CONFLICT(id) DO UPDATE SET
                phase_id = excluded.phase_id,
                week_number = excluded.week_number,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                target_km = excluded.target_km,
                key_workout = excluded.key_workout,
                notes = excluded.notes
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": week.id,
                ":phase_id": week.phase_id,
                ":week_number": week.week_number,
                ":start_date": week.start_date,
                ":end_date": week.end_date,
                ":target_km": week.target_km,
                ":key_workout": week.key_workout,
                ":notes": week.notes,
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert plan_week".to_string(), source))
    }

    /// The sessions between two dates.
    pub fn plan_sessions(&self, from: &str, to: &str) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM plan_session WHERE date >= ? AND date <= ? ORDER BY date, id"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("plan_session".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![from, to], map_session)
            .map_err(|source| {
                crate::error::StoreError::query("plan_session".to_string(), source)
            })?;
        collect(rows, "plan_session")
    }

    /// The sessions of one block.
    pub fn plan_sessions_for_block(&self, block_id: i64) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM plan_session WHERE block_id = ? ORDER BY date, id"
        );
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("plan_session by block".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![block_id], map_session)
            .map_err(|source| {
                crate::error::StoreError::query("plan_session by block".to_string(), source)
            })?;
        collect(rows, "plan_session by block")
    }

    /// The sessions of one day.
    pub fn plan_sessions_for_day(&self, day: &str) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {SESSION_COLUMNS} FROM plan_session WHERE date = ? ORDER BY id");
        let mut statement = conn.prepare(&sql).map_err(|source| {
            crate::error::StoreError::query("plan_session by date".to_string(), source)
        })?;
        let rows = statement
            .query_map(params![day], map_session)
            .map_err(|source| {
                crate::error::StoreError::query("plan_session by date".to_string(), source)
            })?;
        collect(rows, "plan_session by date")
    }

    /// One planned session.
    pub fn plan_session(&self, id: i64) -> Result<Option<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {SESSION_COLUMNS} FROM plan_session WHERE id = ?");
        conn.query_row(&sql, params![id], map_session)
            .optional()
            .map_err(|source| {
                crate::error::StoreError::query("plan_session by id".to_string(), source)
            })
    }

    /// Write one planned session, and return its primary key.
    pub fn upsert_plan_session(&self, session: &PlanSession) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO plan_session ({SESSION_COLUMNS})
             VALUES (:id, :week_id, :block_id, :date, :session_type, :title, :details,
                :target_distance_m, :pace_low_s_km, :pace_high_s_km, :hr_zone_target, :status,
                :source, :notes)
             ON CONFLICT(id) DO UPDATE SET
                week_id = excluded.week_id,
                block_id = excluded.block_id,
                date = excluded.date,
                session_type = excluded.session_type,
                title = excluded.title,
                details = excluded.details,
                target_distance_m = excluded.target_distance_m,
                pace_low_s_km = excluded.pace_low_s_km,
                pace_high_s_km = excluded.pace_high_s_km,
                hr_zone_target = excluded.hr_zone_target,
                status = excluded.status,
                source = excluded.source,
                notes = excluded.notes
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": session.id,
                ":week_id": session.week_id,
                ":block_id": session.block_id,
                ":date": session.date,
                ":session_type": session.session_type,
                ":title": session.title,
                ":details": session.details,
                ":target_distance_m": session.target_distance_m,
                ":pace_low_s_km": session.pace_low_s_km,
                ":pace_high_s_km": session.pace_high_s_km,
                ":hr_zone_target": session.hr_zone_target,
                ":status": session.status,
                ":source": session.source,
                ":notes": session.notes,
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| {
            crate::error::StoreError::query("upsert plan_session".to_string(), source)
        })
    }

    /// The results of one planned session.
    pub fn plan_results(&self, plan_session_id: i64) -> Result<Vec<PlanResult>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {RESULT_COLUMNS} FROM plan_result WHERE plan_session_id = ? ORDER BY id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("plan_result".to_string(), source))?;
        let rows = statement
            .query_map(params![plan_session_id], map_result)
            .map_err(|source| crate::error::StoreError::query("plan_result".to_string(), source))?;
        collect(rows, "plan_result")
    }

    /// The results of every session between two dates.
    pub fn plan_results_between(&self, from: &str, to: &str) -> Result<Vec<PlanResult>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {RESULT_COLUMNS} FROM plan_result \
             WHERE plan_session_id IN (SELECT id FROM plan_session WHERE date >= ? AND date <= ?) \
             ORDER BY id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| crate::error::StoreError::query("plan_result".to_string(), source))?;
        let rows = statement
            .query_map(params![from, to], map_result)
            .map_err(|source| crate::error::StoreError::query("plan_result".to_string(), source))?;
        collect(rows, "plan_result")
    }

    /// Write one result, and return its primary key.
    pub fn upsert_plan_result(&self, result: &PlanResult) -> Result<i64> {
        let conn = self.lock()?;
        let sql = format!(
            "INSERT INTO plan_result ({RESULT_COLUMNS})
             VALUES (:id, :plan_session_id, :activity_id, :verdict, :distance_delta_m,
                :pace_delta_s_km, :pace_band_hit, :distance_hit, :adherence_score, :cause,
                :cause_confidence, :note, :computed_at)
             ON CONFLICT(id) DO UPDATE SET
                plan_session_id = excluded.plan_session_id,
                activity_id = excluded.activity_id,
                verdict = excluded.verdict,
                distance_delta_m = excluded.distance_delta_m,
                pace_delta_s_km = excluded.pace_delta_s_km,
                pace_band_hit = excluded.pace_band_hit,
                distance_hit = excluded.distance_hit,
                adherence_score = excluded.adherence_score,
                cause = excluded.cause,
                cause_confidence = excluded.cause_confidence,
                note = excluded.note,
                computed_at = excluded.computed_at
             RETURNING id"
        );
        conn.query_row(
            &sql,
            named_params! {
                ":id": result.id,
                ":plan_session_id": result.plan_session_id,
                ":activity_id": result.activity_id,
                ":verdict": result.verdict,
                ":distance_delta_m": result.distance_delta_m,
                ":pace_delta_s_km": result.pace_delta_s_km,
                ":pace_band_hit": result.pace_band_hit,
                ":distance_hit": result.distance_hit,
                ":adherence_score": result.adherence_score,
                ":cause": result.cause,
                ":cause_confidence": result.cause_confidence,
                ":note": result.note,
                ":computed_at": DbDateTime(result.computed_at),
            },
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| crate::error::StoreError::query("upsert plan_result".to_string(), source))
    }
}
