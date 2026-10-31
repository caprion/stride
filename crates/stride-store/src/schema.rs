//! The schema: the DDL, the indexes, and the migrations.
//!
//! The statements below are a **verbatim copy** of `docs/contract/schema.sql`
//! — the frozen contract of Slice 0. They are not a redesign, and they are
//! not the old implementation's tables: the old schema was a
//! one-athlete/one-vendor ledger with score columns, a gear ledger and a
//! per-run sync log, and v2 drops all three. The FIT file is the source of
//! truth here, every derived series is recomputed on read, and no score is
//! ever stored.
//!
//! `tests/schema_contract.rs` proves the two agree: it builds a reference
//! database from the contract file itself and compares SQLite's own
//! `sqlite_master` output with the one this module produces. A change to
//! either side without the other fails the build — a golden assertion, not a
//! comment.
//!
//! ## The migration order
//!
//! The creation order **is** the migration order: parents before children,
//! so the foreign keys resolve as the tables are created. The indexes come
//! after the tables, in the order the contract lists them.
//!
//! The version of the schema is SQLite's own `user_version`, so a database
//! knows where it is without a table of its own. Running the migration twice
//! changes nothing and does not fail.

use rusqlite::Connection;

use crate::error::{Result, StoreError};

/// The version of the schema this build reads and writes.
pub const SCHEMA_VERSION: i64 = 1;

/// The statements that create the tables, in creation order.
///
/// The text is the contract's own text, comments and all. SQLite keeps the
/// comments in `sqlite_master`, so the contract's own wording is part of the
/// schema this crate creates.
pub const TABLE_STATEMENTS: &[&str] = &[
    r#"CREATE TABLE activities (
    id                INTEGER NOT NULL PRIMARY KEY,
    source            VARCHAR(32) NOT NULL,        -- 'garmin_personal' | 'fit_import'
    external_id       VARCHAR(64) NOT NULL,        -- FIT file_id.serial_number; fit_import
                                                   -- dedupes by start ±2 min / distance ±2 %
    name              VARCHAR(255) NOT NULL,       -- the run title, from the file name
    sport_type        VARCHAR(40) NOT NULL,        -- normalised FIT sport
    start_time        DATETIME NOT NULL,           -- ISO-8601 UTC
    local_date        VARCHAR(10) NOT NULL,        -- YYYY-MM-DD, athlete-local
    distance_m        REAL NOT NULL,
    moving_time_s     REAL NOT NULL,
    elapsed_time_s    REAL NOT NULL,
    elevation_gain_m  REAL NOT NULL,
    avg_hr            REAL,                        -- NULL = no value, never 0
    max_hr            REAL,
    fit_blob          BLOB NOT NULL,               -- the original FIT file, stored
    fit_sha256        TEXT NOT NULL,               -- content hash, for dedupe/verification
    created_at        DATETIME NOT NULL,
    updated_at        DATETIME NOT NULL,
    CONSTRAINT uq_activities_source_external UNIQUE (source, external_id)
);
"#,
    r#"CREATE TABLE activity_samples (
    activity_id   INTEGER NOT NULL PRIMARY KEY,
    series        BLOB NOT NULL,                   -- compressed per-second series
    sha256        TEXT NOT NULL,                   -- hash of the decompressed series
    first_s       INTEGER,                         -- window start, seconds from run start
    last_s        INTEGER,                         -- window end, seconds from run start
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE
);
"#,
    r#"CREATE TABLE wellness_daily (
    date          VARCHAR(10) NOT NULL PRIMARY KEY, -- YYYY-MM-DD
    sleep_s       REAL,                             -- NULL = no value
    deep_sleep_s  REAL,
    rem_s         REAL,
    light_sleep_s REAL,
    awake_s       REAL,
    hrv_ms        REAL,
    resting_hr    REAL,
    stress_avg    REAL,                             -- present only where the watch reports it
    source        VARCHAR(24) NOT NULL,             -- 'garmin_personal'
    updated_at    DATETIME NOT NULL
);
"#,
    r#"CREATE TABLE plan_blocks (
    id         INTEGER NOT NULL PRIMARY KEY,
    name       VARCHAR(160) NOT NULL,
    source     VARCHAR(24) NOT NULL,                -- 'baseline' | 'prescribed' | 'typed'
    start_date VARCHAR(10) NOT NULL,
    end_date   VARCHAR(10),                         -- NULL = open-ended
    goal       VARCHAR(255),
    status     VARCHAR(24) NOT NULL,                -- 'active' | 'archived'
    notes      TEXT,
    created_at DATETIME NOT NULL
);
"#,
    r#"CREATE TABLE plan_sessions (
    id                INTEGER NOT NULL PRIMARY KEY,
    block_id          INTEGER NOT NULL,
    date              VARCHAR(10) NOT NULL,         -- YYYY-MM-DD
    session_type      VARCHAR(24) NOT NULL,         -- 'easy' | 'tempo' | 'long' | 'interval' |
                                                    -- 'race' | 'rest' | 'strength' | 'unclassified'
    title             VARCHAR(160),
    details           TEXT,
    target_distance_m REAL,
    pace_low_s_km     REAL,                         -- NULL = unbounded
    pace_high_s_km    REAL,
    FOREIGN KEY (block_id) REFERENCES plan_blocks (id) ON DELETE CASCADE
);
"#,
    r#"CREATE TABLE notes (
    id          INTEGER NOT NULL PRIMARY KEY,
    kind        VARCHAR(16) NOT NULL,               -- 'note' | 'goal' | 'pause'
    activity_id INTEGER,                            -- set for kind='note'
    text        TEXT,                               -- the note text, or the pause reason
    value       REAL,                               -- the weekly goal in km, for kind='goal'
    start_date  VARCHAR(10),                        -- pause start
    end_date    VARCHAR(10),                        -- pause end
    created_at  DATETIME NOT NULL,
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE
);
"#,
    r#"CREATE TABLE sync_state (
    kind              VARCHAR(24) NOT NULL PRIMARY KEY, -- 'activities' | 'wellness' | 'daily_summaries'
    cursor            TEXT,                             -- the opaque resume cursor for that kind
    last_success_at   DATETIME,
    token_expires_at  DATETIME,                        -- the disposable 24 h JWT
    updated_at        DATETIME NOT NULL
);
"#,
];

/// The statements that create the indexes, in creation order.
pub const INDEX_STATEMENTS: &[&str] = &[
    r#"CREATE INDEX ix_activities_local_date ON activities (local_date);
"#,
    r#"CREATE INDEX ix_activities_sport      ON activities (sport_type);
"#,
    r#"CREATE INDEX ix_activities_start      ON activities (start_time);
"#,
    r#"CREATE INDEX ix_plan_blocks_start_date ON plan_blocks (start_date);
"#,
    r#"CREATE INDEX ix_plan_sessions_block_date ON plan_sessions (block_id, date);
"#,
    r#"CREATE INDEX ix_plan_sessions_date     ON plan_sessions (date);
"#,
    r#"CREATE INDEX ix_notes_activity_id     ON notes (activity_id);
"#,
];

/// Every statement of the schema, in order: the tables first, then the
/// indexes.
pub fn all_statements() -> Vec<&'static str> {
    TABLE_STATEMENTS
        .iter()
        .chain(INDEX_STATEMENTS.iter())
        .copied()
        .collect()
}

/// One step of the migration ladder.
pub struct Migration {
    /// The schema version the database is at once this step has run.
    pub version: i64,
    /// The statements the step runs, in order.
    pub statements: Vec<&'static str>,
}

/// The migration ladder, in order.
///
/// There is one step today: the schema of the contract. It is written as a
/// ladder rather than a single call so that the next version of the schema
/// has somewhere to go.
pub fn migrations() -> Vec<Migration> {
    vec![Migration {
        version: SCHEMA_VERSION,
        statements: all_statements(),
    }]
}

/// The schema version of a database.
pub fn schema_version(conn: &Connection) -> Result<i64> {
    conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .map_err(|source| StoreError::query("PRAGMA user_version", source))
}

/// Bring a database up to the current schema.
///
/// Idempotent: a database that is already at the current version changes
/// nothing and does not fail. A database at an older version gets the missing
/// steps, each in its own transaction.
pub fn migrate(conn: &Connection) -> Result<()> {
    let current = schema_version(conn)?;
    for migration in migrations() {
        if current >= migration.version {
            continue;
        }
        apply(conn, &migration)?;
    }
    Ok(())
}

/// Run one migration, in a transaction.
fn apply(conn: &Connection, migration: &Migration) -> Result<()> {
    let transaction = conn
        .unchecked_transaction()
        .map_err(|source| StoreError::migration("BEGIN", source))?;

    for statement in &migration.statements {
        transaction
            .execute_batch(statement)
            .map_err(|source| StoreError::migration(label(statement), source))?;
    }

    transaction
        .pragma_update(None, "user_version", migration.version)
        .map_err(|source| StoreError::migration("PRAGMA user_version", source))?;

    transaction
        .commit()
        .map_err(|source| StoreError::migration("COMMIT", source))?;
    Ok(())
}

/// A short name for a statement, for the error message.
///
/// It is the kind of object the statement makes and its name — "table
/// activities" — and never the statement itself, which would be a page of
/// text in every error.
pub fn label(statement: &str) -> String {
    let trimmed = statement.trim_start();
    let (kind, rest) = if let Some(rest) = trimmed.strip_prefix("CREATE TABLE") {
        ("table", rest)
    } else if let Some(rest) = trimmed.strip_prefix("CREATE INDEX") {
        ("index", rest)
    } else if let Some(rest) = trimmed.strip_prefix("CREATE UNIQUE INDEX") {
        ("unique index", rest)
    } else {
        ("statement", trimmed)
    };
    let name = rest
        .trim_start()
        .split(|character: char| character.is_whitespace() || character == '(')
        .next()
        .unwrap_or("")
        .trim();
    format!("{kind} {name}")
}

/// The names of the tables in a database, in creation order.
pub fn table_names(conn: &Connection) -> Result<Vec<String>> {
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

/// The names of the indexes in a database, in creation order.
///
/// The indexes SQLite makes for itself — the ones behind a `UNIQUE`
/// constraint — are not named by the schema and are not listed here.
pub fn index_names(conn: &Connection) -> Result<Vec<String>> {
    let mut statement = conn
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'index' AND name NOT LIKE 'sqlite_%'
             ORDER BY rowid",
        )
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
