//! The schema: the DDL, the indexes, and the migrations.
//!
//! This is a **port**, not a redesign. The tables, the columns, the types, the
//! nullability, the defaults, the indexes, the foreign keys and the uniqueness
//! constraints are exactly those the Python implementation creates, because the
//! desktop's snapshot has to open on the phone.
//!
//! The authoritative definition is `backend/app/models.py`,
//! `backend/app/models_phase2.py` and the DDL in `backend/app/db.py`.
//! `tests/schema_matches_python.rs` proves that this module still agrees with
//! them; it reads the Python sources at test time and fails on any drift.
//!
//! Two facts about the reference are worth knowing before reading the DDL:
//!
//! * SQLAlchemy's `default=` is a **Python-side** default. It does not become
//!   a SQL `DEFAULT` clause. So the schema has no `DEFAULT` clauses at all,
//!   and neither does ours — the defaults live in the Rust row structs.
//! * The tables are created in one pass (`Base.metadata.create_all`) and the
//!   additive column migrations run afterwards, in `db.py::_migrate`. We do
//!   the same thing in the same order.

use rusqlite::Connection;

use crate::error::{Result, StoreError};

/// The tables, in creation order. Parents come before their children, so the
/// foreign keys resolve as the tables are created.
pub const TABLES: &[&str] = &[
    r#"
CREATE TABLE IF NOT EXISTS athlete (
    id INTEGER NOT NULL,
    name VARCHAR(120) NOT NULL,
    email VARCHAR(255),
    timezone VARCHAR(64) NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (id)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS activity (
    id INTEGER NOT NULL,
    athlete_id INTEGER,
    source VARCHAR(32) NOT NULL,
    external_id VARCHAR(64) NOT NULL,
    name VARCHAR(255) NOT NULL,
    sport_type VARCHAR(40) NOT NULL,
    description TEXT,
    start_time DATETIME NOT NULL,
    local_date VARCHAR(10) NOT NULL,
    day_of_week VARCHAR(16),
    distance_m FLOAT NOT NULL,
    moving_time_s FLOAT NOT NULL,
    elapsed_time_s FLOAT NOT NULL,
    elevation_gain_m FLOAT NOT NULL,
    avg_hr FLOAT,
    max_hr FLOAT,
    calories FLOAT,
    avg_cadence FLOAT,
    max_cadence FLOAT,
    avg_speed_ms FLOAT,
    max_speed_ms FLOAT,
    running_time_s FLOAT,
    walking_time_s FLOAT,
    stopped_time_s FLOAT,
    running_distance_m FLOAT,
    perceived_exertion INTEGER,
    device VARCHAR(120),
    gear VARCHAR(120),
    is_trail BOOLEAN NOT NULL,
    is_commute BOOLEAN NOT NULL,
    polyline JSON,
    geo_points JSON,
    raw JSON,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT uq_activity_source_external UNIQUE (source, external_id),
    FOREIGN KEY(athlete_id) REFERENCES athlete (id)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS activity_fact (
    activity_id INTEGER NOT NULL,
    aerobic_te FLOAT,
    anaerobic_te FLOAT,
    training_effect_label VARCHAR(64),
    vo2max FLOAT,
    recovery_time_min FLOAT,
    avg_gct_ms FLOAT,
    avg_stride_ratio FLOAT,
    vertical_ratio FLOAT,
    avg_power FLOAT,
    normalized_power FLOAT,
    avg_grade_adjusted_pace FLOAT,
    temperature_c FLOAT,
    impact_estimate FLOAT,
    gear_uuid VARCHAR(64),
    device_name VARCHAR(120),
    PRIMARY KEY (activity_id),
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS activity_lap (
    id INTEGER NOT NULL,
    activity_id INTEGER NOT NULL,
    lap_index INTEGER NOT NULL,
    distance_m FLOAT NOT NULL,
    duration_s FLOAT NOT NULL,
    avg_speed_ms FLOAT,
    max_speed_ms FLOAT,
    avg_hr FLOAT,
    max_hr FLOAT,
    avg_cadence FLOAT,
    max_cadence FLOAT,
    stride_length_m FLOAT,
    elevation_gain_m FLOAT,
    elevation_loss_m FLOAT,
    start_elevation_m FLOAT,
    end_elevation_m FLOAT,
    running_distance_m FLOAT,
    elapsed_s FLOAT,
    PRIMARY KEY (id),
    CONSTRAINT uq_lap UNIQUE (activity_id, lap_index),
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS activity_stream (
    id INTEGER NOT NULL,
    activity_id INTEGER NOT NULL,
    kind VARCHAR(24) NOT NULL,
    seconds JSON NOT NULL,
    "values" JSON NOT NULL,
    label VARCHAR(64),
    unit VARCHAR(16),
    PRIMARY KEY (id),
    CONSTRAINT uq_stream UNIQUE (activity_id, kind),
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS daily_metric (
    date VARCHAR(10) NOT NULL,
    resting_hr FLOAT,
    max_hr FLOAT,
    sleep_s FLOAT,
    deep_sleep_s FLOAT,
    hrv_ms FLOAT,
    stress FLOAT,
    steps FLOAT,
    body_battery FLOAT,
    readiness FLOAT,
    distance_m FLOAT NOT NULL,
    PRIMARY KEY (date)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS daily_physiology (
    date VARCHAR(10) NOT NULL,
    sleep_s FLOAT,
    sleep_score FLOAT,
    deep_sleep_s FLOAT,
    rem_s FLOAT,
    light_sleep_s FLOAT,
    awake_s FLOAT,
    hrv_ms FLOAT,
    hrv_status VARCHAR(24),
    resting_hr FLOAT,
    max_hr FLOAT,
    stress_avg FLOAT,
    stress_max FLOAT,
    body_battery_min FLOAT,
    body_battery_avg FLOAT,
    body_battery_max FLOAT,
    steps FLOAT,
    training_readiness FLOAT,
    acute_load FLOAT,
    chronic_load FLOAT,
    vo2max FLOAT,
    endurance_score FLOAT,
    running_tolerance FLOAT,
    source VARCHAR(24) NOT NULL,
    updated_at DATETIME NOT NULL,
    PRIMARY KEY (date)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS gear (
    id INTEGER NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    name VARCHAR(160) NOT NULL,
    brand VARCHAR(80),
    model VARCHAR(80),
    status VARCHAR(24) NOT NULL,
    distance_m FLOAT NOT NULL,
    activity_count INTEGER NOT NULL,
    first_used VARCHAR(10),
    last_used VARCHAR(10),
    retire_at_m FLOAT NOT NULL,
    PRIMARY KEY (id)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS hr_zone (
    id INTEGER NOT NULL,
    activity_id INTEGER NOT NULL,
    zone_no INTEGER NOT NULL,
    name VARCHAR(64) NOT NULL,
    low_bpm FLOAT,
    high_bpm FLOAT,
    time_s FLOAT NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT uq_hrzone UNIQUE (activity_id, zone_no),
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS plan_block (
    id INTEGER NOT NULL,
    name VARCHAR(160) NOT NULL,
    start_date VARCHAR(10) NOT NULL,
    end_date VARCHAR(10),
    goal VARCHAR(255),
    status VARCHAR(24) NOT NULL,
    source VARCHAR(24) NOT NULL,
    notes TEXT,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (id)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS plan_phase (
    id INTEGER NOT NULL,
    block_id INTEGER NOT NULL,
    name VARCHAR(80) NOT NULL,
    start_date VARCHAR(10),
    end_date VARCHAR(10),
    target_weekly_km_min FLOAT,
    target_weekly_km_max FLOAT,
    target_sessions INTEGER,
    pace_easy_low FLOAT,
    pace_easy_high FLOAT,
    pace_tempo_low FLOAT,
    pace_tempo_high FLOAT,
    pace_interval_low FLOAT,
    pace_interval_high FLOAT,
    pace_long_low FLOAT,
    pace_long_high FLOAT,
    hr_zone_target INTEGER,
    notes TEXT,
    PRIMARY KEY (id),
    FOREIGN KEY(block_id) REFERENCES plan_block (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS plan_session (
    id INTEGER NOT NULL,
    week_id INTEGER,
    block_id INTEGER,
    date VARCHAR(10) NOT NULL,
    session_type VARCHAR(24) NOT NULL,
    title VARCHAR(160),
    details TEXT,
    target_distance_m FLOAT,
    pace_low_s_km FLOAT,
    pace_high_s_km FLOAT,
    hr_zone_target INTEGER,
    status VARCHAR(24) NOT NULL,
    source VARCHAR(24) NOT NULL,
    notes TEXT,
    PRIMARY KEY (id),
    FOREIGN KEY(week_id) REFERENCES plan_week (id) ON DELETE CASCADE,
    FOREIGN KEY(block_id) REFERENCES plan_block (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS plan_week (
    id INTEGER NOT NULL,
    phase_id INTEGER NOT NULL,
    week_number INTEGER NOT NULL,
    start_date VARCHAR(10),
    end_date VARCHAR(10),
    target_km FLOAT,
    key_workout VARCHAR(255),
    notes TEXT,
    PRIMARY KEY (id),
    FOREIGN KEY(phase_id) REFERENCES plan_phase (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS plan_result (
    id INTEGER NOT NULL,
    plan_session_id INTEGER NOT NULL,
    activity_id INTEGER,
    verdict VARCHAR(24) NOT NULL,
    distance_delta_m FLOAT,
    pace_delta_s_km FLOAT,
    pace_band_hit BOOLEAN,
    distance_hit BOOLEAN,
    adherence_score FLOAT,
    cause TEXT,
    cause_confidence FLOAT,
    note TEXT,
    computed_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(plan_session_id) REFERENCES plan_session (id) ON DELETE CASCADE,
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE SET NULL
)"#,
    r#"
CREATE TABLE IF NOT EXISTS readiness (
    date VARCHAR(10) NOT NULL,
    score FLOAT NOT NULL,
    grade VARCHAR(16) NOT NULL,
    drivers JSON,
    recommendation TEXT,
    rationale TEXT,
    computed_at DATETIME NOT NULL,
    PRIMARY KEY (date)
)"#,
    r#"
CREATE TABLE IF NOT EXISTS run_fingerprint (
    activity_id INTEGER NOT NULL,
    running_time_s FLOAT,
    walking_time_s FLOAT,
    stopped_time_s FLOAT,
    cadence_avg FLOAT,
    cadence_cv FLOAT,
    cadence_drift_pct FLOAT,
    hr_avg FLOAT,
    hr_max FLOAT,
    hr_drift_pct FLOAT,
    hr_drift_bpm FLOAT,
    pace_drift_pct FLOAT,
    first_half_hr FLOAT,
    second_half_hr FLOAT,
    first_half_pace_s_km FLOAT,
    second_half_pace_s_km FLOAT,
    efficiency_first FLOAT,
    efficiency_last FLOAT,
    decoupling_pct FLOAT,
    pace_cv FLOAT,
    negative_split_pct FLOAT,
    stride_length_cm FLOAT,
    form_score FLOAT,
    zone_distribution JSON,
    notes TEXT,
    computed_at DATETIME NOT NULL,
    PRIMARY KEY (activity_id),
    FOREIGN KEY(activity_id) REFERENCES activity (id) ON DELETE CASCADE
)"#,
    r#"
CREATE TABLE IF NOT EXISTS sync_run (
    id INTEGER NOT NULL,
    source VARCHAR(32) NOT NULL,
    started_at DATETIME NOT NULL,
    finished_at DATETIME,
    status VARCHAR(24) NOT NULL,
    days INTEGER NOT NULL,
    activities_found INTEGER NOT NULL,
    activities_new INTEGER NOT NULL,
    activities_updated INTEGER NOT NULL,
    message TEXT,
    error TEXT,
    PRIMARY KEY (id)
)"#,
];

/// The indexes, in the order SQLAlchemy creates them.
///
/// The names are SQLAlchemy's: `ix_<table>_<column>` for `index=True`, and
/// the explicit `Index(...)` names for the composite ones.
pub const INDEXES: &[&str] = &[
    "CREATE UNIQUE INDEX IF NOT EXISTS ix_gear_uuid ON gear (uuid)",
    "CREATE INDEX IF NOT EXISTS ix_plan_block_start_date ON plan_block (start_date)",
    "CREATE INDEX IF NOT EXISTS ix_activity_local_date ON activity (local_date)",
    "CREATE INDEX IF NOT EXISTS ix_activity_sport ON activity (sport_type)",
    "CREATE INDEX IF NOT EXISTS ix_activity_start ON activity (start_time)",
    "CREATE INDEX IF NOT EXISTS ix_plan_phase_block_id ON plan_phase (block_id)",
    "CREATE INDEX IF NOT EXISTS ix_activity_lap_activity_id ON activity_lap (activity_id)",
    "CREATE INDEX IF NOT EXISTS ix_activity_stream_activity_id ON activity_stream (activity_id)",
    "CREATE INDEX IF NOT EXISTS ix_hr_zone_activity_id ON hr_zone (activity_id)",
    "CREATE INDEX IF NOT EXISTS ix_plan_week_phase_id ON plan_week (phase_id)",
    "CREATE INDEX IF NOT EXISTS ix_plan_session_block_date ON plan_session (block_id, date)",
    "CREATE INDEX IF NOT EXISTS ix_plan_session_date ON plan_session (date)",
    "CREATE INDEX IF NOT EXISTS ix_plan_session_week_id ON plan_session (week_id)",
    "CREATE INDEX IF NOT EXISTS ix_plan_result_activity_id ON plan_result (activity_id)",
    "CREATE INDEX IF NOT EXISTS ix_plan_result_plan_session_id ON plan_result (plan_session_id)",
];

/// Columns added to the schema after the first release.
///
/// This is the same list as `backend/app/db.py::_migrate` — a column added to
/// a model later has to be applied by hand, because `create_all` never alters
/// an existing table. Every step is idempotent: if the column is already
/// there, nothing happens.
pub const ADDITIVE_COLUMNS: &[(&str, &str, &str)] = &[
    ("daily_physiology", "max_hr", "REAL"),
    ("activity", "running_time_s", "REAL"),
    ("activity", "walking_time_s", "REAL"),
    ("activity", "stopped_time_s", "REAL"),
    ("activity", "running_distance_m", "REAL"),
    ("activity_lap", "running_distance_m", "REAL"),
    ("activity_lap", "elapsed_s", "REAL"),
    ("run_fingerprint", "running_time_s", "REAL"),
    ("run_fingerprint", "walking_time_s", "REAL"),
    ("run_fingerprint", "stopped_time_s", "REAL"),
    ("run_fingerprint", "hr_drift_bpm", "REAL"),
    ("run_fingerprint", "pace_drift_pct", "REAL"),
    ("run_fingerprint", "first_half_hr", "REAL"),
    ("run_fingerprint", "second_half_hr", "REAL"),
    ("run_fingerprint", "first_half_pace_s_km", "REAL"),
    ("run_fingerprint", "second_half_pace_s_km", "REAL"),
];

/// Columns removed from the models but still present in older databases.
///
/// `activity.visibility` was left behind `NOT NULL` with no default when the
/// social layer was removed, which broke every INSERT that did not supply it.
/// SQLite can only drop a column from 3.35 onwards; when it cannot, the
/// reference logs a warning and carries on, and so do we.
pub const REMOVED_COLUMNS: &[(&str, &str)] = &[("activity", "visibility")];

/// Bring a database up to the current schema.
///
/// Idempotent: running it twice changes nothing and does not fail.
pub fn migrate(conn: &Connection) -> Result<()> {
    for statement in TABLES {
        conn.execute_batch(statement)
            .map_err(|source| StoreError::migration(*statement, source))?;
    }

    apply_additive_columns(conn)?;
    apply_removals(conn)?;

    for statement in INDEXES {
        conn.execute_batch(statement)
            .map_err(|source| StoreError::migration(*statement, source))?;
    }

    Ok(())
}

/// Add the columns that a later model introduced, if the table is missing them.
fn apply_additive_columns(conn: &Connection) -> Result<()> {
    for (table, column, kind) in ADDITIVE_COLUMNS {
        let present = column_names(conn, table)?;
        if present.iter().any(|name| name == column) {
            continue;
        }
        let statement = format!("ALTER TABLE {table} ADD COLUMN {column} {kind}");
        conn.execute_batch(&statement)
            .map_err(|source| StoreError::migration(statement.clone(), source))?;
    }
    Ok(())
}

/// Drop the columns that a later model removed, if they are still there.
fn apply_removals(conn: &Connection) -> Result<()> {
    for (table, column) in REMOVED_COLUMNS {
        let present = column_names(conn, table)?;
        if !present.iter().any(|name| name == column) {
            continue;
        }
        let statement = format!("ALTER TABLE {table} DROP COLUMN {column}");
        // A database that cannot drop the column still works — it is how the
        // reference behaves — but the leftover column has to be reported.
        if let Err(source) = conn.execute_batch(&statement) {
            eprintln!(
                "stride-store: could not drop {table}.{column} ({source}); rebuild the table"
            );
        }
    }
    Ok(())
}

/// The column names of a table, in declaration order. Empty if the table is
/// not there.
fn column_names(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|source| StoreError::query(format!("PRAGMA table_info({table})"), source))?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|source| StoreError::query(format!("PRAGMA table_info({table})"), source))?;

    let mut out = Vec::new();
    for name in names {
        out.push(
            name.map_err(|source| {
                StoreError::query(format!("PRAGMA table_info({table})"), source)
            })?,
        );
    }
    Ok(out)
}
