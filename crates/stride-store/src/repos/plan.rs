//! `plan_blocks` and `plan_sessions`.
//!
//! ## One active block per source
//!
//! The schema says it and this module keeps it: **exactly one block per plan
//! source is active**. A source — `baseline`, `prescribed`, `typed` — has one
//! block in force; the rest are archived.
//!
//! The old implementation's bug was here: it counted every block it had ever
//! imported, so a plan that had been imported twice looked like two plans.
//! The invariant is enforced on the write, in the same transaction as the
//! write itself, so a database can never hold two active blocks of one
//! source.
//!
//! ## The session is the intent only
//!
//! `plan_sessions` says what was planned. Whether a session was done, not
//! done or is still planned is **computed on read**, from the session's date,
//! the matching run and the block's source. A stored verdict would be a
//! judgement the plan forbids, and a baseline never yields "not done".

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{
    plan_source, plan_status, NewPlanBlock, NewPlanSession, PlanBlock, PlanSession,
};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `plan_blocks`.
const BLOCK_COLUMNS: &str = "id, name, source, start_date, end_date, goal, status, notes, \
    created_at";

/// The columns of `plan_sessions`.
const SESSION_COLUMNS: &str = "id, block_id, date, session_type, title, details, \
    target_distance_m, pace_low_s_km, pace_high_s_km";

/// The sources a plan block may have.
const SOURCES: [&str; 3] = [
    plan_source::BASELINE,
    plan_source::PRESCRIBED,
    plan_source::TYPED,
];

/// The statuses a plan block may have.
const STATUSES: [&str; 2] = [plan_status::ACTIVE, plan_status::ARCHIVED];

/// Read one row of `plan_blocks`.
fn map_block(row: &Row<'_>) -> rusqlite::Result<PlanBlock> {
    Ok(PlanBlock {
        id: row.get("id")?,
        name: row.get("name")?,
        source: row.get("source")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        goal: row.get("goal")?,
        status: row.get("status")?,
        notes: row.get("notes")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
    })
}

/// Read one row of `plan_sessions`.
fn map_session(row: &Row<'_>) -> rusqlite::Result<PlanSession> {
    Ok(PlanSession {
        id: row.get("id")?,
        block_id: row.get("block_id")?,
        date: row.get("date")?,
        session_type: row.get("session_type")?,
        title: row.get("title")?,
        details: row.get("details")?,
        target_distance_m: row.get("target_distance_m")?,
        pace_low_s_km: row.get("pace_low_s_km")?,
        pace_high_s_km: row.get("pace_high_s_km")?,
    })
}

impl Store {
    /// Write one plan block.
    ///
    /// A block is identified by its source, its name and its first day: a plan
    /// that is imported twice updates the block that is already there instead
    /// of adding a second one.
    ///
    /// If the block is **active**, every other active block of the same
    /// source is archived in the same transaction — so there is never more
    /// than one active block per source.
    ///
    /// Returns the block's primary key.
    pub fn upsert_plan_block(&self, block: &NewPlanBlock) -> Result<i64> {
        if !SOURCES.contains(&block.source.as_str()) {
            return Err(StoreError::Invariant(format!(
                "the plan source {:?} is not one of {}",
                block.source,
                SOURCES.join(", ")
            )));
        }
        if !STATUSES.contains(&block.status.as_str()) {
            return Err(StoreError::Invariant(format!(
                "the plan status {:?} is not one of {}",
                block.status,
                STATUSES.join(", ")
            )));
        }

        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let existing = transaction
            .query_row(
                "SELECT id FROM plan_blocks
                 WHERE source = ?1 AND name = ?2 AND start_date = ?3
                 ORDER BY id ASC
                 LIMIT 1",
                params![block.source, block.name, block.start_date],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|source| StoreError::query("find the plan block", source))?;

        if block.status == plan_status::ACTIVE {
            // The other active blocks of this source go to the archive. The
            // one being written is not among them.
            transaction
                .execute(
                    "UPDATE plan_blocks SET status = ?3
                     WHERE source = ?1 AND status = ?2 AND id IS NOT ?4",
                    params![
                        block.source,
                        plan_status::ACTIVE,
                        plan_status::ARCHIVED,
                        existing,
                    ],
                )
                .map_err(|source| StoreError::query("archive the other plan blocks", source))?;
        }

        let id = match existing {
            Some(id) => {
                transaction
                    .execute(
                        "UPDATE plan_blocks
                         SET name = ?2, source = ?3, start_date = ?4, end_date = ?5,
                             goal = ?6, status = ?7, notes = ?8
                         WHERE id = ?1",
                        params![
                            id,
                            block.name,
                            block.source,
                            block.start_date,
                            block.end_date,
                            block.goal,
                            block.status,
                            block.notes,
                        ],
                    )
                    .map_err(|source| StoreError::query("update the plan block", source))?;
                id
            }
            None => transaction
                .query_row(
                    "INSERT INTO plan_blocks (
                        name, source, start_date, end_date, goal, status, notes, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     RETURNING id",
                    params![
                        block.name,
                        block.source,
                        block.start_date,
                        block.end_date,
                        block.goal,
                        block.status,
                        block.notes,
                        DbDateTime(block.created_at),
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|source| StoreError::query("insert the plan block", source))?,
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// Every plan block, oldest first.
    pub fn plan_blocks(&self) -> Result<Vec<PlanBlock>> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {BLOCK_COLUMNS} FROM plan_blocks ORDER BY start_date ASC, id ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("plan_blocks", source))?;
        let mapped = statement
            .query_map([], map_block)
            .map_err(|source| StoreError::query("plan_blocks", source))?;
        collect(mapped)
    }

    /// Every plan block of one source, oldest first.
    ///
    /// This is the list of blocks a source has ever had — **active and
    /// archived**. The one that is in force is [`Store::active_plan_block`].
    pub fn plan_blocks_of_source(&self, source: &str) -> Result<Vec<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {BLOCK_COLUMNS} FROM plan_blocks
             WHERE source = ?1
             ORDER BY start_date ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("plan_blocks of a source", source))?;
        let mapped = statement
            .query_map(params![source], map_block)
            .map_err(|source| StoreError::query("plan_blocks of a source", source))?;
        collect(mapped)
    }

    /// The one active block of a source.
    ///
    /// There is at most one, because the repository keeps it that way. The
    /// newest of them wins if a database was written by something else.
    pub fn active_plan_block(&self, source: &str) -> Result<Option<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {BLOCK_COLUMNS} FROM plan_blocks
             WHERE source = ?1 AND status = ?2
             ORDER BY start_date DESC, id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![source, plan_status::ACTIVE], map_block)
            .optional()
            .map_err(|source| StoreError::query("active plan block", source))
    }

    /// One plan block.
    pub fn plan_block(&self, id: i64) -> Result<Option<PlanBlock>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {BLOCK_COLUMNS} FROM plan_blocks WHERE id = ?1");
        conn.query_row(&sql, params![id], map_block)
            .optional()
            .map_err(|source| StoreError::query("plan block by id", source))
    }

    /// How many active blocks a source has.
    ///
    /// It is one, and it stays one. The count is here so that a test can say
    /// so out loud.
    pub fn count_active_plan_blocks(&self, source: &str) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT count(*) FROM plan_blocks WHERE source = ?1 AND status = ?2",
            params![source, plan_status::ACTIVE],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::query("count active plan blocks", source))
    }

    /// Archive every active block of a source.
    ///
    /// Used when a source is turned off — the block stops being in force, and
    /// the archive keeps the history.
    pub fn archive_active_plan_blocks(&self, source: &str) -> Result<usize> {
        let conn = self.lock()?;
        let archived = conn
            .execute(
                "UPDATE plan_blocks SET status = ?2 WHERE source = ?1 AND status = ?3",
                params![source, plan_status::ARCHIVED, plan_status::ACTIVE],
            )
            .map_err(|source| StoreError::query("archive the plan blocks", source))?;
        Ok(archived)
    }

    /// Write one planned session.
    ///
    /// A session is identified by its block, its date and its kind: importing
    /// the same plan twice updates the session that is already there.
    ///
    /// Returns the session's primary key.
    pub fn upsert_plan_session(&self, planned: &NewPlanSession) -> Result<i64> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let existing = transaction
            .query_row(
                "SELECT id FROM plan_sessions
                 WHERE block_id = ?1 AND date = ?2 AND session_type = ?3
                 ORDER BY id ASC
                 LIMIT 1",
                params![planned.block_id, planned.date, planned.session_type],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|source| StoreError::query("find the plan session", source))?;

        let id = match existing {
            Some(id) => {
                transaction
                    .execute(
                        "UPDATE plan_sessions
                         SET title = ?2, details = ?3, target_distance_m = ?4,
                             pace_low_s_km = ?5, pace_high_s_km = ?6
                         WHERE id = ?1",
                        params![
                            id,
                            planned.title,
                            planned.details,
                            planned.target_distance_m,
                            planned.pace_low_s_km,
                            planned.pace_high_s_km,
                        ],
                    )
                    .map_err(|source| StoreError::query("update the plan session", source))?;
                id
            }
            None => transaction
                .query_row(
                    "INSERT INTO plan_sessions (
                        block_id, date, session_type, title, details,
                        target_distance_m, pace_low_s_km, pace_high_s_km
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     RETURNING id",
                    params![
                        planned.block_id,
                        planned.date,
                        planned.session_type,
                        planned.title,
                        planned.details,
                        planned.target_distance_m,
                        planned.pace_low_s_km,
                        planned.pace_high_s_km,
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|source| StoreError::query("insert the plan session", source))?,
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// One planned session.
    pub fn plan_session(&self, id: i64) -> Result<Option<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {SESSION_COLUMNS} FROM plan_sessions WHERE id = ?1");
        conn.query_row(&sql, params![id], map_session)
            .optional()
            .map_err(|source| StoreError::query("plan session by id", source))
    }

    /// The sessions between two dates, both inclusive, oldest first.
    pub fn plan_sessions_between(&self, from: &str, to: &str) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM plan_sessions
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("plan sessions between", source))?;
        let mapped = statement
            .query_map(params![from, to], map_session)
            .map_err(|source| StoreError::query("plan sessions between", source))?;
        collect(mapped)
    }

    /// The sessions of one block, oldest first.
    pub fn plan_sessions_for_block(&self, block_id: i64) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM plan_sessions
             WHERE block_id = ?1
             ORDER BY date ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("plan sessions of a block", source))?;
        let mapped = statement
            .query_map(params![block_id], map_session)
            .map_err(|source| StoreError::query("plan sessions of a block", source))?;
        collect(mapped)
    }

    /// The sessions of one day, oldest first.
    pub fn plan_sessions_for_day(&self, date: &str) -> Result<Vec<PlanSession>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM plan_sessions
             WHERE date = ?1
             ORDER BY id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("plan sessions of a day", source))?;
        let mapped = statement
            .query_map(params![date], map_session)
            .map_err(|source| StoreError::query("plan sessions of a day", source))?;
        collect(mapped)
    }

    /// How many planned sessions there are.
    pub fn count_plan_sessions(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM plan_sessions", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count plan_sessions", source))
    }
}
