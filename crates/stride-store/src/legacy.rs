//! The one-time importer: the previous version's history, carried across.
//!
//! The athlete has two years of running in an app that is being replaced by
//! one with a different schema. The container survives the upgrade — same
//! package, same certificate, higher version code — and so does the file and
//! the Keystore. But the old tables are not the new tables, and nothing in
//! the new code reads the old ones. So without this, the upgrade keeps the
//! container and loses the content.
//!
//! ## What the two schemas are
//!
//! The **old** one is the previous implementation's: seventeen tables —
//! `activity`, `activity_fact`, `activity_lap`, `activity_stream`,
//! `daily_metric`, `daily_physiology`, `gear`, `hr_zone`, `plan_block`,
//! `plan_phase`, `plan_result`, `plan_session`, `plan_week`, `readiness`,
//! `run_fingerprint`, `sync_run`, `athlete`. It never set `user_version`,
//! which is how an old database is told from a new one without believing a
//! number the file could have made up.
//!
//! The **new** one is `docs/contract/schema.sql`: seven tables —
//! `activities`, `activity_samples`, `wellness_daily`, `plan_blocks`,
//! `plan_sessions`, `notes`, `sync_state`.
//!
//! ## The flow
//!
//! ```no_run
//! use std::path::Path;
//! use stride_store::{DatabaseKey, SchemaKind, Store};
//!
//! # fn main() -> Result<(), stride_store::StoreError> {
//! let key = DatabaseKey::from_text("the key the caller holds")?;
//! let store = Store::open(Path::new("stride.db"), &key)?;
//!
//! // 1. What is this file?
//! let kind = store.with_connection(stride_store::detect_schema)?;
//! if kind == SchemaKind::Old {
//!     // 2. The new schema, beside the old one.
//!     store.migrate()?;
//!     // 3. Carry the history across — one transaction, all of it or none.
//!     let report = stride_store::import_legacy(&store)?;
//!     // The app shows the count, in the app's voice.
//!     println!("{} runs carried across", report.runs);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## The rules
//!
//! * **One transaction.** Every row is written in one transaction on the
//!   store's connection: either the whole history is carried across, or
//!   none of it is and the new tables are as empty as they were.
//! * **Nothing is moved and nothing is deleted.** The old tables are still
//!   there afterwards, with their rows, and the import can be run again.
//! * **Running it twice changes nothing.** The rows are matched by their
//!   natural keys — a run by its source and its Garmin id, a day by its
//!   date, a block by its source, name and first day, a session by its
//!   block, date and kind — so the second run finds every row already
//!   there and writes nothing.
//! * **The mapping is a pure function.** [`map_activity`] and its
//!   siblings take the old row and give the new one; they can be read, and
//!   tested, without a database.
//! * **It is honest.** The report says what was carried and what was left
//!   behind, and why.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row, Transaction, TransactionBehavior};

use crate::datetimes::{parse_datetime, DbDateTime};
use crate::error::{Result, StoreError};
use crate::hash;
use crate::key::DatabaseKey;
use crate::model::{NewActivity, NewNote, NewPlanBlock, NewPlanSession, WellnessDaily};
use crate::store::Store;

/// The old table the runs come from.
const OLD_RUNS: &str = "SELECT id, source, external_id, name, sport_type, start_time, \
    local_date, distance_m, moving_time_s, elapsed_time_s, elevation_gain_m, avg_hr, max_hr, \
    description, created_at, updated_at FROM activity ORDER BY id";

/// The old table the wellness comes from.
const OLD_WELLNESS: &str = "SELECT date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s, \
    hrv_ms, resting_hr, stress_avg, source, updated_at FROM daily_physiology ORDER BY date";

/// The old table the plan blocks come from.
const OLD_BLOCKS: &str = "SELECT id, name, source, start_date, end_date, goal, status, notes, \
    created_at FROM plan_block ORDER BY id";

/// The old table the planned sessions come from.
const OLD_SESSIONS: &str = "SELECT id, block_id, date, session_type, title, details, \
    target_distance_m, pace_low_s_km, pace_high_s_km FROM plan_session ORDER BY id";

/// The run, in the new shape.
const INSERT_RUN: &str = "INSERT INTO activities (
        source, external_id, name, sport_type, start_time, local_date, distance_m,
        moving_time_s, elapsed_time_s, elevation_gain_m, avg_hr, max_hr, fit_blob,
        fit_sha256, created_at, updated_at
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
     RETURNING id";

/// The run with this Garmin id.
const FIND_RUN: &str = "SELECT id FROM activities WHERE source = ?1 AND external_id = ?2";

/// The runs a pull has already marked deleted.
///
/// The table belongs to the api thread's tombstones: a run Garmin no longer
/// lists is **marked**, not forgotten, and a later pull that sees it again
/// does not bring it back. The importer never **writes** to it — it reads it
/// once, so that the import does not bring back a run that was already
/// accounted for. On an upgrade the table is empty, because the pull has not
/// run yet, so this is a belt and braces.
const TOMBSTONED_RUNS: &str = "SELECT external_id FROM deleted_garmin_activities";

/// How many runs a pull has marked deleted.
const TOMBSTONED_RUNS_COUNT: &str = "SELECT count(*) FROM deleted_garmin_activities";

/// A day of wellness, in the new shape.
const INSERT_WELLNESS: &str = "INSERT INTO wellness_daily (
        date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s, hrv_ms, resting_hr,
        stress_avg, source, updated_at
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
     ON CONFLICT(date) DO NOTHING";

/// The plan block with this source, name and first day.
const FIND_BLOCK: &str = "SELECT id FROM plan_blocks
     WHERE source = ?1 AND name = ?2 AND start_date = ?3 ORDER BY id ASC LIMIT 1";

/// A plan block, in the new shape.
const INSERT_BLOCK: &str = "INSERT INTO plan_blocks (
        name, source, start_date, end_date, goal, status, notes, created_at
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id";

/// The planned session of this block, date and kind.
const FIND_SESSION: &str = "SELECT id FROM plan_sessions
     WHERE block_id = ?1 AND date = ?2 AND session_type = ?3 ORDER BY id ASC LIMIT 1";

/// A planned session, in the new shape.
const INSERT_SESSION: &str = "INSERT INTO plan_sessions (
        block_id, date, session_type, title, details, target_distance_m, pace_low_s_km,
        pace_high_s_km
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id";

/// The note on this run.
const FIND_NOTE: &str = "SELECT id FROM notes
     WHERE kind = ?1 AND activity_id IS ?2 AND text IS ?3
     ORDER BY id ASC LIMIT 1";

/// A note, in the new shape.
const INSERT_NOTE: &str = "INSERT INTO notes (
        kind, activity_id, text, value, start_date, end_date, created_at
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING id";

/// What a database holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaKind {
    /// The previous version's schema.
    Old,
    /// This version's schema.
    New,
    /// Both: an old database that has already been migrated.
    Mixed,
    /// No tables at all.
    Empty,
    /// Something else entirely.
    Unknown,
}

impl SchemaKind {
    /// The kind, in words.
    ///
    /// The app shows this when it cannot decide what to do, and it is worth
    /// saying plainly: "the previous version's database" is a fact the
    /// athlete can act on; "schema kind 3" is not.
    pub fn describe(&self) -> &'static str {
        match self {
            SchemaKind::Old => "the previous version's database",
            SchemaKind::New => "this version's database",
            SchemaKind::Mixed => "the previous version's database, already migrated",
            SchemaKind::Empty => "an empty database",
            SchemaKind::Unknown => "a database this build does not know",
        }
    }
}

impl std::fmt::Display for SchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.describe())
    }
}

/// Something the previous version's database held and the new one does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftBehind {
    /// What it is: an old table's name.
    pub what: String,
    /// How many rows were left behind.
    pub rows: i64,
    /// Why it was not carried.
    pub why: &'static str,
}

/// What the import carried, and what it left.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegacyImportReport {
    /// The runs carried across.
    pub runs: usize,
    /// The days of wellness carried across.
    pub wellness: usize,
    /// The notes carried across.
    pub notes: usize,
    /// The plan blocks carried across.
    pub plan_blocks: usize,
    /// The planned sessions carried across.
    pub plan_sessions: usize,
    /// The rows that were already there and were left alone.
    pub already_there: usize,
    /// The things that were left behind, and why.
    pub left_behind: Vec<LeftBehind>,
}

impl LegacyImportReport {
    /// Everything that was carried across, in one number.
    pub fn carried(&self) -> usize {
        self.runs + self.wellness + self.notes + self.plan_blocks + self.plan_sessions
    }

    /// Whether the import found nothing to do.
    pub fn is_empty(&self) -> bool {
        self.carried() == 0 && self.already_there == 0
    }
}

// ------------------------------------------------------------------- the old

/// One run, as the previous version stored it.
#[derive(Debug, Clone, PartialEq)]
pub struct OldActivity {
    /// The old row's primary key.
    pub id: i64,
    /// Where the run came from: `garmin`, `fit_import`, or `demo`.
    pub source: String,
    /// The vendor's id for the run.
    pub external_id: String,
    /// The run's title.
    pub name: String,
    /// The sport, in the previous version's vocabulary.
    pub sport_type: String,
    /// When the run started, in the previous version's text.
    pub start_time: String,
    /// The athlete's local date.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// The elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The average heart rate.
    pub avg_hr: Option<f64>,
    /// The maximum heart rate.
    pub max_hr: Option<f64>,
    /// The description: the athlete's own words, or Garmin's.
    pub description: Option<String>,
    /// When the row was written, in the previous version's text.
    pub created_at: String,
    /// When the row was last changed, in the previous version's text.
    pub updated_at: String,
}

/// One day of wellness, as the previous version stored it.
#[derive(Debug, Clone, PartialEq)]
pub struct OldWellness {
    /// The athlete's local date.
    pub date: String,
    /// The sleep, in seconds.
    pub sleep_s: Option<f64>,
    /// The deep sleep, in seconds.
    pub deep_sleep_s: Option<f64>,
    /// The REM sleep, in seconds.
    pub rem_s: Option<f64>,
    /// The light sleep, in seconds.
    pub light_sleep_s: Option<f64>,
    /// The time awake, in seconds.
    pub awake_s: Option<f64>,
    /// The heart-rate variability, in milliseconds.
    pub hrv_ms: Option<f64>,
    /// The resting heart rate, in beats per minute.
    pub resting_hr: Option<f64>,
    /// The average stress.
    pub stress_avg: Option<f64>,
    /// Where the row came from.
    pub source: String,
    /// When the row was last changed, in the previous version's text.
    pub updated_at: String,
}

/// One plan block, as the previous version stored it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldPlanBlock {
    /// The old row's primary key.
    pub id: i64,
    /// The block's name.
    pub name: String,
    /// Where the block came from: `garmin`, `manual` or `inferred`.
    pub source: String,
    /// The first day of the block.
    pub start_date: String,
    /// The last day of the block.
    pub end_date: Option<String>,
    /// The block's goal.
    pub goal: Option<String>,
    /// `active` or `archived`.
    pub status: String,
    /// The block's notes.
    pub notes: Option<String>,
    /// When the row was written, in the previous version's text.
    pub created_at: String,
}

/// One planned session, as the previous version stored it.
#[derive(Debug, Clone, PartialEq)]
pub struct OldPlanSession {
    /// The old row's primary key.
    pub id: i64,
    /// The block it belongs to. `None` in the previous version.
    pub block_id: Option<i64>,
    /// The day of the session.
    pub date: String,
    /// The kind of session.
    pub session_type: String,
    /// The session's title.
    pub title: Option<String>,
    /// The session's details.
    pub details: Option<String>,
    /// The target distance, in metres.
    pub target_distance_m: Option<f64>,
    /// The lower bound of the pace band, in seconds per kilometre.
    pub pace_low_s_km: Option<f64>,
    /// The upper bound of the pace band, in seconds per kilometre.
    pub pace_high_s_km: Option<f64>,
}

// --------------------------------------------------------------- the mapping

/// The new name for an old source.
///
/// The previous version called the Garmin Personal API `garmin`; this version
/// calls it `garmin_personal`, so that a run imported from a file cannot be
/// confused with one that came from Garmin. The old `demo` — the rows an
/// older installation seeded with synthetic data — has no new name: those
/// rows are not the athlete's history, and the previous version threw them
/// away at every start-up.
pub fn map_source(old: &str) -> Option<&'static str> {
    match old.trim().to_lowercase().as_str() {
        "garmin" | "garmin_personal" => Some("garmin_personal"),
        "fit_import" => Some("fit_import"),
        _ => None,
    }
}

/// The new name for an old plan block's source.
///
/// `garmin` was a plan the athlete wrote down in Garmin Connect, so it is a
/// **prescription**. `manual` was a plan the athlete typed in, so it is
/// **typed**. `inferred` was a plan the previous version inferred from the
/// athlete's own history, so it is a **baseline** — and a baseline is never
/// a prescription, which is the distinction the whole plan engine rests on.
pub fn map_plan_source(old: &str) -> Option<&'static str> {
    match old.trim().to_lowercase().as_str() {
        "garmin" | "prescribed" => Some("prescribed"),
        "manual" | "typed" => Some("typed"),
        "inferred" | "baseline" => Some("baseline"),
        _ => None,
    }
}

/// The new kind for an old session's kind.
///
/// The previous version knew `recovery` and `unknown`; this version does
/// not. A session of a kind it does not know becomes **unclassified** — the
/// session is kept, and the fact that its kind was lost is in the report.
pub fn map_session_type(old: &str) -> &'static str {
    match old.trim().to_lowercase().as_str() {
        "easy" => "easy",
        "tempo" => "tempo",
        "long" => "long",
        "interval" => "interval",
        "race" => "race",
        "rest" => "rest",
        "strength" => "strength",
        _ => "unclassified",
    }
}

/// The new kind for an old block's status.
///
/// A status this build does not know is **archived**: a block that is not
/// known to be in force is not in force.
pub fn map_block_status(old: &str) -> &'static str {
    match old.trim().to_lowercase().as_str() {
        "active" => "active",
        _ => "archived",
    }
}

/// The sport, in this version's vocabulary.
///
/// The core contract fixes the vocabulary (`docs/contract/core-contract.md`
/// §1): every flavour of running is `running`, and the rest are themselves.
/// The previous version kept `trail_running`, `treadmill_running` and
/// `virtualrun` apart, so the old value is normalised again here.
pub fn normalise_sport(old: &str) -> String {
    let key = old.trim().to_lowercase().replace(' ', "_");
    match key.as_str() {
        "running" | "trail_running" | "treadmill_running" | "virtualrun" | "street_running"
        | "track_running" => "running".to_string(),
        "walking" | "hiking" | "cycling" | "swimming" | "strength_training" => key,
        _ => {
            if key.is_empty() || key.ends_with("running") {
                "running".to_string()
            } else {
                key
            }
        }
    }
}

/// The run, in the new shape.
///
/// The FIT file is empty: the previous version's database did not keep one.
/// The run keeps its Garmin id, so the sync's dedupe still recognises it and
/// does not download it again; what the app shows of the run is the numbers
/// the old database already had.
pub fn map_activity(old: &OldActivity) -> Result<NewActivity> {
    let source = map_source(&old.source).ok_or_else(|| {
        StoreError::Invariant(format!(
            "the run {} came from {:?}, which this build does not read",
            old.external_id, old.source
        ))
    })?;
    Ok(NewActivity {
        source: source.to_string(),
        external_id: old.external_id.clone(),
        name: old.name.clone(),
        sport_type: normalise_sport(&old.sport_type),
        start_time: parse_datetime(&old.start_time)?,
        local_date: old.local_date.clone(),
        distance_m: old.distance_m,
        moving_time_s: old.moving_time_s,
        elapsed_time_s: old.elapsed_time_s,
        elevation_gain_m: old.elevation_gain_m,
        avg_hr: old.avg_hr,
        max_hr: old.max_hr,
        fit_blob: Vec::new(),
        created_at: parse_datetime(&old.created_at)?,
        updated_at: parse_datetime(&old.updated_at)?,
    })
}

/// The note the run's description became.
///
/// The previous version had no notes table: the only free text on a run was
/// its description, and the athlete's own words are worth carrying across.
/// A run with no description has no note. The run the note belongs to is
/// filled in by the importer, because the new run has its own id.
pub fn map_description(old: &OldActivity) -> Result<Option<NewNote>> {
    let text = match old.description.as_ref() {
        Some(text) => text.trim(),
        None => return Ok(None),
    };
    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(NewNote {
        kind: "note".to_string(),
        activity_id: None,
        text: Some(text.to_string()),
        value: None,
        start_date: None,
        end_date: None,
        created_at: parse_datetime(&old.created_at)?,
    }))
}

/// The day of wellness, in the new shape.
pub fn map_wellness(old: &OldWellness) -> Result<WellnessDaily> {
    let source = map_source(&old.source).unwrap_or("garmin_personal");
    Ok(WellnessDaily {
        date: old.date.clone(),
        sleep_s: old.sleep_s,
        deep_sleep_s: old.deep_sleep_s,
        rem_s: old.rem_s,
        light_sleep_s: old.light_sleep_s,
        awake_s: old.awake_s,
        hrv_ms: old.hrv_ms,
        resting_hr: old.resting_hr,
        stress_avg: old.stress_avg,
        source: source.to_string(),
        updated_at: parse_datetime(&old.updated_at)?,
    })
}

/// The plan block, in the new shape.
pub fn map_plan_block(old: &OldPlanBlock) -> Result<NewPlanBlock> {
    let source = map_plan_source(&old.source).ok_or_else(|| {
        StoreError::Invariant(format!(
            "the plan block {:?} came from {:?}, which this build does not read",
            old.name, old.source
        ))
    })?;
    Ok(NewPlanBlock {
        name: old.name.clone(),
        source: source.to_string(),
        start_date: old.start_date.clone(),
        end_date: old.end_date.clone(),
        goal: old.goal.clone(),
        status: map_block_status(&old.status).to_string(),
        notes: old.notes.clone(),
        created_at: parse_datetime(&old.created_at)?,
    })
}

/// The planned session, in the new shape.
///
/// The session needs a block: the new schema says so, and a session without
/// one has nothing to belong to. Such a session is not carried, and the
/// report says how many.
pub fn map_plan_session(old: &OldPlanSession, block_id: i64) -> Result<NewPlanSession> {
    Ok(NewPlanSession {
        block_id,
        date: old.date.clone(),
        session_type: map_session_type(&old.session_type).to_string(),
        title: old.title.clone(),
        details: old.details.clone(),
        target_distance_m: old.target_distance_m,
        pace_low_s_km: old.pace_low_s_km,
        pace_high_s_km: old.pace_high_s_km,
    })
}

// ----------------------------------------------------------------- detection

/// The tables a database holds.
fn table_names(conn: &Connection) -> Result<Vec<String>> {
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY rowid")
        .map_err(|source| StoreError::query("sqlite_master", source))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| StoreError::query("sqlite_master", source))?;
    let mut names = Vec::new();
    for row in rows {
        names.push(row.map_err(|source| StoreError::query("sqlite_master", source))?);
    }
    Ok(names)
}

/// What a database holds.
///
/// The old schema is told from the new one by its **tables**, not by
/// `user_version`: the previous version never set it, so a number in that
/// field says nothing at all. The old schema has `activity` and
/// `activity_fact`; the new one has `activities` and `sync_state`.
pub fn detect_schema(conn: &Connection) -> Result<SchemaKind> {
    let tables = table_names(conn)?;
    let has = |name: &str| tables.iter().any(|table| table == name);
    let old = has("activity") || has("activity_fact");
    let new = has("activities");
    Ok(match (old, new) {
        (true, true) => SchemaKind::Mixed,
        (true, false) => SchemaKind::Old,
        (false, true) => SchemaKind::New,
        (false, false) => {
            if tables.is_empty() {
                SchemaKind::Empty
            } else {
                SchemaKind::Unknown
            }
        }
    })
}

/// What the database at this path holds.
///
/// The file is opened with the key the caller has — the previous version's
/// phone database was encrypted too — and read. A file that cannot be
/// opened is an error, not an empty database.
pub fn detect_schema_file(path: &Path, key: &DatabaseKey) -> Result<SchemaKind> {
    let store = Store::open(path, key)?;
    store.with_connection(detect_schema)
}

// ----------------------------------------------------------------- the import

/// The old tables this version has no place for: the table's name, the
/// statement that counts its rows, and why it was left behind.
const LEFT_BEHIND: [(&str, &str, &str); 13] = [
    (
        "activity_stream",
        "SELECT count(*) FROM activity_stream",
        "the previous version's series were one row per kind, in JSON; the new table \
         holds one compressed series per run, and its encoding is not in the contract. \
         The app re-derives what it can from the FIT.",
    ),
    (
        "activity_lap",
        "SELECT count(*) FROM activity_lap",
        "the laps are derived from the FIT, and the new schema keeps no lap table.",
    ),
    (
        "hr_zone",
        "SELECT count(*) FROM hr_zone",
        "the heart-rate zones are derived from the FIT, and the new schema keeps no \
         zone table.",
    ),
    (
        "run_fingerprint",
        "SELECT count(*) FROM run_fingerprint",
        "the run fingerprint is derived from the FIT, and the new schema keeps no \
         fingerprint table.",
    ),
    (
        "activity_fact",
        "SELECT count(*) FROM activity_fact",
        "the training facts are derived from the FIT, and the new schema keeps no \
         facts table.",
    ),
    (
        "gear",
        "SELECT count(*) FROM gear",
        "the new schema has no gear table: a piece of kit is not a fact about a run.",
    ),
    (
        "readiness",
        "SELECT count(*) FROM readiness",
        "the product shows no device score, so no score is stored.",
    ),
    (
        "sync_run",
        "SELECT count(*) FROM sync_run",
        "the new sync state is one row per kind of data, and the new app keeps its own.",
    ),
    (
        "athlete",
        "SELECT count(*) FROM athlete",
        "there is one athlete; the new schema has no athlete table.",
    ),
    (
        "daily_metric",
        "SELECT count(*) FROM daily_metric",
        "the old phase-1 mirror of the wellness table; the wellness itself was carried \
         across.",
    ),
    (
        "plan_phase",
        "SELECT count(*) FROM plan_phase",
        "the new schema keeps the block and the session; the phase is the block's \
         business.",
    ),
    (
        "plan_week",
        "SELECT count(*) FROM plan_week",
        "the new schema keeps the block and the session; the week is the session's \
         business.",
    ),
    (
        "plan_result",
        "SELECT count(*) FROM plan_result",
        "the plan's done and not-done is computed on read; the new schema stores no \
         verdict.",
    ),
];

/// Carry the previous version's history into the new tables.
///
/// **One transaction.** Either every row is written, or none of them is and
/// the new tables are as empty as they were. The old tables are read, never
/// written: they are still there afterwards, with their rows.
///
/// Running it again finds every row already there — the rows are matched by
/// their natural keys — and writes nothing.
///
/// The store must already hold the new schema: call [`crate::Store::migrate`]
/// first.
pub fn import_legacy(store: &Store) -> Result<LegacyImportReport> {
    let mut report = LegacyImportReport::default();
    let mut conn = store.lock()?;
    let transaction = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| StoreError::query("BEGIN", source))?;

    let kind = detect_schema(&transaction)?;
    if !matches!(kind, SchemaKind::Old | SchemaKind::Mixed) {
        // Nothing of the previous version is here. Not an error: the app asks
        // for this on every start-up, and most start-ups are not an upgrade.
        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        return Ok(report);
    }

    import_runs(&transaction, &mut report)?;
    let blocks = import_blocks(&transaction, &mut report)?;
    import_sessions(&transaction, &blocks, &mut report)?;
    import_wellness(&transaction, &mut report)?;

    // What was left behind: the old tables that held rows and that this
    // version has no place for.
    for (table, count, why) in LEFT_BEHIND {
        let rows = transaction
            .query_row(count, [], |row| row.get::<_, i64>(0))
            .optional()
            .map_err(|source| StoreError::query("count the old rows", source))?;
        if let Some(rows) = rows {
            if rows > 0 {
                report.left_behind.push(LeftBehind {
                    what: table.to_string(),
                    rows,
                    why,
                });
            }
        }
    }
    // The columns of a run that the new row has no place for. The
    // description was not left behind: it became a note on the run.
    if report.runs > 0 {
        report.left_behind.push(LeftBehind {
            what: "the run's calories, cadence, speed, running and walking time, the \
                   device, the gear, the perceived exertion and the route"
                .to_string(),
            rows: report.runs as i64,
            why: "the new run row has no place for them. They are derived from the \
                  FIT, or they are the device's opinion of the run.",
        });
    }

    // The runs a pull had already marked deleted. The importer never writes
    // to the tombstones; it only leaves those runs alone.
    let marked = marked_count(&transaction)?;
    if marked > 0 {
        report.left_behind.push(LeftBehind {
            what: "the runs a pull had already marked deleted".to_string(),
            rows: marked,
            why: "a run Garmin no longer lists is marked, not forgotten. The import \
                  does not bring it back, and it does not write to the tombstones.",
        });
    }

    transaction
        .commit()
        .map_err(|source| StoreError::query("COMMIT", source))?;
    Ok(report)
}

/// How many runs a pull has marked deleted.
fn marked_count(transaction: &Transaction<'_>) -> Result<i64> {
    if !has_tombstones(transaction)? {
        return Ok(0);
    }
    transaction
        .query_row(TOMBSTONED_RUNS_COUNT, [], |row| row.get::<_, i64>(0))
        .map_err(|source| StoreError::query("count the tombstones", source))
}

/// Whether the database has the tombstone table.
///
/// The table arrived with the second version of the schema. A database from
/// before it has no tombstones, and no tombstones is not an error.
fn has_tombstones(transaction: &Transaction<'_>) -> Result<bool> {
    let count: i64 = transaction
        .query_row(
            "SELECT count(*) FROM sqlite_master
             WHERE type = 'table' AND name = 'deleted_garmin_activities'",
            [],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::query("sqlite_master", source))?;
    Ok(count > 0)
}

/// Carry the runs across, and the notes that were on them.
fn import_runs(transaction: &Transaction<'_>, report: &mut LegacyImportReport) -> Result<()> {
    // The runs a pull has already marked deleted: the import leaves them
    // alone, so that it does not bring back what was accounted for. A
    // database from before the tombstones existed has none, and that is
    // simply an empty set.
    let mut marked: std::collections::HashSet<String> = std::collections::HashSet::new();
    if has_tombstones(transaction)? {
        let mut statement = transaction
            .prepare(TOMBSTONED_RUNS)
            .map_err(|source| StoreError::query("the tombstones", source))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|source| StoreError::query("the tombstones", source))?;
        for row in rows {
            let external_id = row.map_err(|source| StoreError::query("the tombstones", source))?;
            marked.insert(external_id);
        }
    }

    let mut statement = transaction
        .prepare(OLD_RUNS)
        .map_err(|source| StoreError::query("the old activity table", source))?;
    let rows = statement
        .query_map([], map_old_activity)
        .map_err(|source| StoreError::query("the old activity table", source))?;

    for row in rows {
        let old = row.map_err(|source| StoreError::query("the old activity table", source))?;
        // The synthetic rows an older installation seeded are not the
        // athlete's history. The previous version threw them away at every
        // start-up; they are left here and not carried.
        if map_source(&old.source).is_none() {
            continue;
        }
        let new = map_activity(&old)?;

        // A run a pull has already marked deleted is not brought back.
        if marked.contains(&new.external_id) {
            continue;
        }

        let existing = transaction
            .query_row(FIND_RUN, params![new.source, new.external_id], |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(|source| StoreError::query("find the run", source))?;

        let id = match existing {
            Some(id) => {
                report.already_there += 1;
                id
            }
            None => {
                let id = transaction
                    .query_row(
                        INSERT_RUN,
                        params![
                            new.source,
                            new.external_id,
                            new.name,
                            new.sport_type,
                            DbDateTime(new.start_time),
                            new.local_date,
                            new.distance_m,
                            new.moving_time_s,
                            new.elapsed_time_s,
                            new.elevation_gain_m,
                            new.avg_hr,
                            new.max_hr,
                            new.fit_blob,
                            hash::sha256_hex(&new.fit_blob),
                            DbDateTime(new.created_at),
                            DbDateTime(new.updated_at),
                        ],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|source| StoreError::query("write the run", source))?;
                report.runs += 1;
                id
            }
        };

        // The description was the athlete's own text: it became a note.
        if let Some(mut note) = map_description(&old)? {
            note.activity_id = Some(id);
            let existing = transaction
                .query_row(
                    FIND_NOTE,
                    params![note.kind, note.activity_id, note.text],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|source| StoreError::query("find the note", source))?;
            match existing {
                Some(_) => report.already_there += 1,
                None => {
                    transaction
                        .query_row(
                            INSERT_NOTE,
                            params![
                                note.kind,
                                note.activity_id,
                                note.text,
                                note.value,
                                note.start_date,
                                note.end_date,
                                DbDateTime(note.created_at),
                            ],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|source| StoreError::query("write the note", source))?;
                    report.notes += 1;
                }
            }
        }
    }
    Ok(())
}

/// Carry the plan blocks across.
///
/// Returns the old block id of every block carried, so that the planned
/// sessions can find the block they belong to.
fn import_blocks(
    transaction: &Transaction<'_>,
    report: &mut LegacyImportReport,
) -> Result<Vec<(i64, i64)>> {
    let mut statement = transaction
        .prepare(OLD_BLOCKS)
        .map_err(|source| StoreError::query("the old plan_block table", source))?;
    let rows = statement
        .query_map([], map_old_block)
        .map_err(|source| StoreError::query("the old plan_block table", source))?;

    let mut carried = Vec::new();
    for row in rows {
        let old = row.map_err(|source| StoreError::query("the old plan_block table", source))?;
        let new = match map_plan_block(&old) {
            Ok(new) => new,
            // A block whose source this build does not know is left behind:
            // guessing what kind of plan it was would be a judgement.
            Err(_) => continue,
        };

        let existing = transaction
            .query_row(
                FIND_BLOCK,
                params![new.source, new.name, new.start_date],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|source| StoreError::query("find the plan block", source))?;

        let id = match existing {
            Some(id) => {
                report.already_there += 1;
                id
            }
            None => {
                let id = transaction
                    .query_row(
                        INSERT_BLOCK,
                        params![
                            new.name,
                            new.source,
                            new.start_date,
                            new.end_date,
                            new.goal,
                            new.status,
                            new.notes,
                            DbDateTime(new.created_at),
                        ],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|source| StoreError::query("write the plan block", source))?;
                report.plan_blocks += 1;
                id
            }
        };
        carried.push((old.id, id));
    }
    Ok(carried)
}

/// Carry the planned sessions across.
fn import_sessions(
    transaction: &Transaction<'_>,
    blocks: &[(i64, i64)],
    report: &mut LegacyImportReport,
) -> Result<()> {
    let mut statement = transaction
        .prepare(OLD_SESSIONS)
        .map_err(|source| StoreError::query("the old plan_session table", source))?;
    let rows = statement
        .query_map([], map_old_session)
        .map_err(|source| StoreError::query("the old plan_session table", source))?;

    for row in rows {
        let old = row.map_err(|source| StoreError::query("the old plan_session table", source))?;
        let old_block = match old.block_id {
            Some(id) => id,
            // A session with no block has nothing to belong to in the new
            // schema. It is left behind, and the report says so.
            None => continue,
        };
        let block_id = match blocks.iter().find(|(from, _)| *from == old_block) {
            Some((_, to)) => *to,
            // The block itself was not carried, so the session cannot be.
            None => continue,
        };
        let new = map_plan_session(&old, block_id)?;

        let existing = transaction
            .query_row(
                FIND_SESSION,
                params![new.block_id, new.date, new.session_type],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|source| StoreError::query("find the plan session", source))?;

        match existing {
            Some(_) => report.already_there += 1,
            None => {
                transaction
                    .query_row(
                        INSERT_SESSION,
                        params![
                            new.block_id,
                            new.date,
                            new.session_type,
                            new.title,
                            new.details,
                            new.target_distance_m,
                            new.pace_low_s_km,
                            new.pace_high_s_km,
                        ],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|source| StoreError::query("write the plan session", source))?;
                report.plan_sessions += 1;
            }
        }
    }
    Ok(())
}

/// Carry the wellness across.
fn import_wellness(transaction: &Transaction<'_>, report: &mut LegacyImportReport) -> Result<()> {
    let mut statement = transaction
        .prepare(OLD_WELLNESS)
        .map_err(|source| StoreError::query("the old daily_physiology table", source))?;
    let rows = statement
        .query_map([], map_old_wellness)
        .map_err(|source| StoreError::query("the old daily_physiology table", source))?;

    for row in rows {
        let old =
            row.map_err(|source| StoreError::query("the old daily_physiology table", source))?;
        let new = map_wellness(&old)?;
        let written = transaction
            .execute(
                INSERT_WELLNESS,
                params![
                    new.date,
                    new.sleep_s,
                    new.deep_sleep_s,
                    new.rem_s,
                    new.light_sleep_s,
                    new.awake_s,
                    new.hrv_ms,
                    new.resting_hr,
                    new.stress_avg,
                    new.source,
                    DbDateTime(new.updated_at),
                ],
            )
            .map_err(|source| StoreError::query("write the wellness", source))?;
        if written == 0 {
            // The row was already there: ON CONFLICT did nothing.
            report.already_there += 1;
        } else {
            report.wellness += 1;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ the reads

/// Read one row of the old `activity` table.
fn map_old_activity(row: &Row<'_>) -> rusqlite::Result<OldActivity> {
    Ok(OldActivity {
        id: row.get("id")?,
        source: row.get("source")?,
        external_id: row.get("external_id")?,
        name: row.get("name")?,
        sport_type: row.get("sport_type")?,
        start_time: row.get("start_time")?,
        local_date: row.get("local_date")?,
        distance_m: row.get("distance_m")?,
        moving_time_s: row.get("moving_time_s")?,
        elapsed_time_s: row.get("elapsed_time_s")?,
        elevation_gain_m: row.get("elevation_gain_m")?,
        avg_hr: row.get("avg_hr")?,
        max_hr: row.get("max_hr")?,
        description: row.get("description")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Read one row of the old `daily_physiology` table.
fn map_old_wellness(row: &Row<'_>) -> rusqlite::Result<OldWellness> {
    Ok(OldWellness {
        date: row.get("date")?,
        sleep_s: row.get("sleep_s")?,
        deep_sleep_s: row.get("deep_sleep_s")?,
        rem_s: row.get("rem_s")?,
        light_sleep_s: row.get("light_sleep_s")?,
        awake_s: row.get("awake_s")?,
        hrv_ms: row.get("hrv_ms")?,
        resting_hr: row.get("resting_hr")?,
        stress_avg: row.get("stress_avg")?,
        source: row.get("source")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Read one row of the old `plan_block` table.
fn map_old_block(row: &Row<'_>) -> rusqlite::Result<OldPlanBlock> {
    Ok(OldPlanBlock {
        id: row.get("id")?,
        name: row.get("name")?,
        source: row.get("source")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        goal: row.get("goal")?,
        status: row.get("status")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
    })
}

/// Read one row of the old `plan_session` table.
fn map_old_session(row: &Row<'_>) -> rusqlite::Result<OldPlanSession> {
    Ok(OldPlanSession {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_source_becomes_the_new_one() {
        assert_eq!(map_source("garmin"), Some("garmin_personal"));
        assert_eq!(map_source("GARMIN"), Some("garmin_personal"));
        assert_eq!(map_source("fit_import"), Some("fit_import"));
        assert_eq!(map_source("demo"), None);
        assert_eq!(map_source(""), None);
    }

    #[test]
    fn the_old_plan_source_becomes_the_new_one() {
        assert_eq!(map_plan_source("garmin"), Some("prescribed"));
        assert_eq!(map_plan_source("manual"), Some("typed"));
        assert_eq!(map_plan_source("inferred"), Some("baseline"));
        assert_eq!(map_plan_source("something else"), None);
    }

    #[test]
    fn the_old_session_kind_becomes_the_new_one() {
        assert_eq!(map_session_type("easy"), "easy");
        assert_eq!(map_session_type("tempo"), "tempo");
        assert_eq!(map_session_type("long"), "long");
        assert_eq!(map_session_type("interval"), "interval");
        assert_eq!(map_session_type("race"), "race");
        assert_eq!(map_session_type("rest"), "rest");
        assert_eq!(map_session_type("strength"), "strength");
        assert_eq!(map_session_type("recovery"), "unclassified");
        assert_eq!(map_session_type("unknown"), "unclassified");
        assert_eq!(map_session_type(""), "unclassified");
    }

    #[test]
    fn the_sport_is_normalised_as_the_core_contract_says() {
        assert_eq!(normalise_sport("running"), "running");
        assert_eq!(normalise_sport("trail_running"), "running");
        assert_eq!(normalise_sport("treadmill_running"), "running");
        assert_eq!(normalise_sport("virtualrun"), "running");
        assert_eq!(normalise_sport("street_running"), "running");
        assert_eq!(normalise_sport("track_running"), "running");
        assert_eq!(normalise_sport("Running"), "running");
        assert_eq!(normalise_sport("cycling"), "cycling");
        assert_eq!(normalise_sport("strength_training"), "strength_training");
        assert_eq!(
            normalise_sport("Open Water Swimming"),
            "open_water_swimming"
        );
        assert_eq!(normalise_sport(""), "running");
    }

    #[test]
    fn the_hash_of_an_empty_file_is_the_hash_of_an_empty_file() {
        // The hash a carried run carries: it has no FIT file, and this is
        // the hash of nothing.
        assert_eq!(
            hash::sha256_hex(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
