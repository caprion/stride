//! `adopted_plans` and `adopted_sessions`.
//!
//! The adopted plan: a **dated copy** of a programme from the catalogue,
//! with the sessions that were worked out on the day it was adopted.
//!
//! ## Why the sessions are stored and not derived
//!
//! The plan's paces were worked out **once**, at the adoption, from the
//! threshold that was in force that day. If the sessions were derived from
//! the catalogue on read, a later correction to the catalogue — or a new
//! threshold — would silently re-pace a plan that is already running. The
//! dated copy is what makes that impossible.
//!
//! ## One plan at a time
//!
//! **One plan is in force at a time.** A plan is in force from its first
//! day to its last, and the repository reads the plan in force on a day so
//! that a second plan cannot start while another one is running.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::error::{Result, StoreError};
use crate::model::{AdoptedPlan, AdoptedSession, NewAdoptedPlan, NewAdoptedSession};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `adopted_plans`.
const PLAN_COLUMNS: &str = "id, template_id, template_version, label, method, tier, \
    goal_distance_m, goal_label, race_date, start_date, end_date, adopted_on, threshold_value, \
    threshold_date, threshold_source, threshold_base, rule_value, rule_date, paced, \
    pacing_missing, source, status";

/// The columns of `adopted_sessions`.
const SESSION_COLUMNS: &str = "id, plan_id, date, week, kind, distance_m, pace_low_s_per_km, \
    pace_high_s_per_km, finish_low_s_per_km, finish_high_s_per_km, pace_rule, threshold_value, \
    detail, lt_last_m";

/// Read one row of `adopted_plans`.
fn map_plan(row: &Row<'_>) -> rusqlite::Result<AdoptedPlan> {
    Ok(AdoptedPlan {
        id: row.get("id")?,
        template_id: row.get("template_id")?,
        template_version: row.get("template_version")?,
        label: row.get("label")?,
        method: row.get("method")?,
        tier: row.get("tier")?,
        goal_distance_m: row.get("goal_distance_m")?,
        goal_label: row.get("goal_label")?,
        race_date: row.get("race_date")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        adopted_on: row.get("adopted_on")?,
        threshold_value: row.get("threshold_value")?,
        threshold_date: row.get("threshold_date")?,
        threshold_source: row.get("threshold_source")?,
        threshold_base: row.get("threshold_base")?,
        rule_value: row.get("rule_value")?,
        rule_date: row.get("rule_date")?,
        paced: row.get::<_, i64>("paced")? != 0,
        pacing_missing: row.get("pacing_missing")?,
        source: row.get("source")?,
        status: row.get("status")?,
    })
}

/// Read one row of `adopted_sessions`.
fn map_session(row: &Row<'_>) -> rusqlite::Result<AdoptedSession> {
    Ok(AdoptedSession {
        id: row.get("id")?,
        plan_id: row.get("plan_id")?,
        date: row.get("date")?,
        week: row.get("week")?,
        kind: row.get("kind")?,
        distance_m: row.get("distance_m")?,
        pace_low_s_per_km: row.get("pace_low_s_per_km")?,
        pace_high_s_per_km: row.get("pace_high_s_per_km")?,
        finish_low_s_per_km: row.get("finish_low_s_per_km")?,
        finish_high_s_per_km: row.get("finish_high_s_per_km")?,
        pace_rule: row.get("pace_rule")?,
        threshold_value: row.get("threshold_value")?,
        detail: row.get("detail")?,
        lt_last_m: row.get("lt_last_m")?,
    })
}

impl Store {
    /// Write an adopted plan and its own sessions.
    ///
    /// **One transaction.** The plan and the sessions that belong to it go
    /// in together or not at all: a plan whose sessions are half written
    /// is a plan the app cannot read.
    ///
    /// Returns the plan's primary key.
    pub fn write_adopted_plan(
        &self,
        plan: &NewAdoptedPlan,
        sessions: &[NewAdoptedSession],
    ) -> Result<i64> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let id = transaction
            .query_row(
                "INSERT INTO adopted_plans (
                    template_id, template_version, label, method, tier, goal_distance_m,
                    goal_label, race_date, start_date, end_date, adopted_on, threshold_value,
                    threshold_date, threshold_source, threshold_base, rule_value, rule_date,
                    paced, pacing_missing, source, status, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, 'prescribed', 'active', ?20)
                 RETURNING id",
                params![
                    plan.template_id,
                    plan.template_version,
                    plan.label,
                    plan.method,
                    plan.tier,
                    plan.goal_distance_m,
                    plan.goal_label,
                    plan.race_date,
                    plan.start_date,
                    plan.end_date,
                    plan.adopted_on,
                    plan.threshold_value,
                    plan.threshold_date,
                    plan.threshold_source,
                    plan.threshold_base,
                    plan.rule_value,
                    plan.rule_date,
                    i64::from(plan.paced),
                    plan.pacing_missing,
                    crate::datetimes::DbDateTime::from(plan.created_at),
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| StoreError::query("write an adopted plan", source))?;

        for session in sessions.iter() {
            transaction
                .execute(
                    "INSERT INTO adopted_sessions (
                        plan_id, date, week, kind, distance_m, pace_low_s_per_km,
                        pace_high_s_per_km, finish_low_s_per_km, finish_high_s_per_km,
                        pace_rule, threshold_value, detail, lt_last_m
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        id,
                        session.date,
                        session.week,
                        session.kind,
                        session.distance_m,
                        session.pace_low_s_per_km,
                        session.pace_high_s_per_km,
                        session.finish_low_s_per_km,
                        session.finish_high_s_per_km,
                        session.pace_rule,
                        session.threshold_value,
                        session.detail,
                        session.lt_last_m,
                    ],
                )
                .map_err(|source| StoreError::query("write an adopted session", source))?;
        }

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// Every adopted plan, newest first.
    pub fn adopted_plans(&self) -> Result<Vec<AdoptedPlan>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {PLAN_COLUMNS} FROM adopted_plans
             ORDER BY start_date DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the adopted plans", source))?;
        let mapped = statement
            .query_map([], map_plan)
            .map_err(|source| StoreError::query("the adopted plans", source))?;
        collect(mapped)
    }

    /// One adopted plan, by its primary key.
    pub fn adopted_plan(&self, id: i64) -> Result<Option<AdoptedPlan>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {PLAN_COLUMNS} FROM adopted_plans WHERE id = ?1");
        conn.query_row(&sql, params![id], map_plan)
            .optional()
            .map_err(|source| StoreError::query("the adopted plan", source))
    }

    /// The plans that are in force — the active ones, newest first.
    pub fn active_adopted_plans(&self) -> Result<Vec<AdoptedPlan>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {PLAN_COLUMNS} FROM adopted_plans
             WHERE status = 'active'
             ORDER BY start_date DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the adopted plans in force", source))?;
        let mapped = statement
            .query_map([], map_plan)
            .map_err(|source| StoreError::query("the adopted plans in force", source))?;
        collect(mapped)
    }

    /// The plan in force on a day.
    ///
    /// A plan is in force from its first day to its last. `None` when no
    /// plan covers the day.
    pub fn adopted_plan_in_force(&self, date: &str) -> Result<Option<AdoptedPlan>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {PLAN_COLUMNS} FROM adopted_plans
             WHERE status = 'active' AND start_date <= ?1 AND end_date >= ?1
             ORDER BY start_date DESC, id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![date], map_plan)
            .optional()
            .map_err(|source| StoreError::query("the plan in force", source))
    }

    /// The sessions of one plan, in date order.
    pub fn adopted_sessions(&self, plan_id: i64) -> Result<Vec<AdoptedSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM adopted_sessions
             WHERE plan_id = ?1
             ORDER BY date ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the adopted sessions", source))?;
        let mapped = statement
            .query_map(params![plan_id], map_session)
            .map_err(|source| StoreError::query("the adopted sessions", source))?;
        collect(mapped)
    }

    /// How many adopted plans there are.
    pub fn count_adopted_plans(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM adopted_plans", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count the adopted plans", source))
    }

    /// How many sessions the adopted plans hold.
    pub fn count_adopted_sessions(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM adopted_sessions", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the adopted sessions", source))
    }
}
