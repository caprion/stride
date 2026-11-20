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
//!
//! A step adds the tables **and** the indexes that go with them: an index
//! added to an old step would never be created on a database that already
//! ran that step.

use rusqlite::Connection;

use crate::error::{Result, StoreError};

/// The version of the schema this build reads and writes.
pub const SCHEMA_VERSION: i64 = 5;

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
    // The table added in version 2: tombstones for the runs Garmin no
    // longer lists.
    r#"-- Old: (nothing - the old app never learned that a run was deleted).
-- v2: a run Garmin no longer lists is **marked**, not forgotten and not
-- resurrected. A pull that cannot find a stored run in Garmin's list writes a
-- tombstone here; a later pull that sees the run again does not bring it
-- back, because Garmin's list is not proof that a run came back - it is proof
-- that the run is still there, and the tombstone says we already accounted
-- for it.
CREATE TABLE deleted_garmin_activities (
    external_id  VARCHAR(64) NOT NULL PRIMARY KEY,  -- the Garmin activity id
    local_date   VARCHAR(10) NOT NULL,               -- the run's day
    marked_at    DATETIME NOT NULL                  -- when the pull marked it
);
"#,
    // The table added in version 3: the athlete's heart-rate zone model.
    // It is the watch's own configuration, read from Garmin — not a series
    // derived from a run — so it is stored as a dated fact, one row per
    // model change.
    r#"-- Old: (nothing - the old app stored per-run zone bands, never the model
-- they came from). v3: the zone model is the watch's own configuration, read
-- from Garmin at /biometric-service/heartRateZones/v2 - a dated fact about
-- the athlete's settings, not a series derived from a run. One row per model
-- change; read_at is the day the model was read, so an insight can say where
-- its input came from and when it was read.
CREATE TABLE hr_zone_models (
    id               INTEGER NOT NULL PRIMARY KEY,
    sport            VARCHAR(24) NOT NULL,          -- as Garmin reported it ('DEFAULT')
    training_method  VARCHAR(24) NOT NULL,          -- HR_MAX | LACTATE_THRESHOLD | HR_RESERVE | CUSTOM
    max_heart_rate   REAL,                          -- maxHeartRateUsed; NULL = not reported
    zone1_floor      REAL NOT NULL,                 -- inclusive lower bounds, in bpm
    zone2_floor      REAL NOT NULL,
    zone3_floor      REAL NOT NULL,
    zone4_floor      REAL NOT NULL,
    zone5_floor      REAL NOT NULL,
    read_at          DATETIME NOT NULL              -- when the model was read, ISO-8601 UTC
);
"#,
    // The table added in version 4: the source's own wellness observations.
    //
    // The old app kept a projection of the wellness and dropped the rest:
    // the sleep window, the stress durations, the body battery, the seven-day
    // average resting heart rate, the respiration. This table keeps what the
    // source actually serves, with its provenance - the value, the day the
    // source reported it for, and the day it was read.
    r#"-- Old: (nothing - the old app stored a projection of the wellness and
-- dropped the rest of it). v4: the source's own wellness observations, kept
-- with their provenance: the value, the day the source reported it for, and
-- the day it was read. One row per (day, source, field, item).
--
-- The missingness is kept as the source gave it: a field the source never
-- sent has no row at all, and a field the source sent as null has a row
-- whose value is null. A later read never deletes a row - it replaces the
-- value it carried and keeps the reading time of the read that carried it -
-- so a failed read leaves the previous value and its reading time in place.
--
-- Nothing here is derived: every value is one the source reported, in the
-- source's own unit. The device's own verdicts - the sleep scores, the HRV
-- status, the VO2 max - are judgements, not observations, and are not
-- stored.
CREATE TABLE wellness_observations (
    id           INTEGER NOT NULL PRIMARY KEY,
    date         VARCHAR(10) NOT NULL,        -- the day the source reported the value for
    source       VARCHAR(24) NOT NULL,        -- 'garmin_personal'
    endpoint     VARCHAR(96) NOT NULL,        -- the API path the value was read from
    field        VARCHAR(48) NOT NULL,        -- the source's own field name
    ordinal      INTEGER NOT NULL DEFAULT 0,  -- which item of a repeated field this is
    value        REAL,                        -- the number, as reported; NULL when the source sent null
    text_value   TEXT,                        -- a value that is not a number, as reported
    unit         VARCHAR(16),                 -- the unit the source states for the number
    observed_at  DATETIME,                    -- the instant the item is about, when the source gave one
    retrieved_at DATETIME NOT NULL,           -- when the value was read, ISO-8601 UTC
    CONSTRAINT uq_wellness_observations UNIQUE (date, source, field, ordinal)
);
"#,
    // The table added in version 5: the run update.
    //
    // The update is a model's own reading of one run: four to six
    // sentences, written from the run's numbers and checked against them.
    // It is not the product's voice, and it is not a fact of the run: it
    // is **attributed**, and what it was checked against is stored beside
    // it.
    //
    // The payload hash is the hash of the payload the model was given. A
    // draft whose numbers are not in that payload was not written from it,
    // and the hash is what lets anybody check that later.
    r#"-- Old: (nothing - the old app never wrote about a run in words). v5:
-- the run update: a model's own reading of one run, checked against the
-- numbers it was given.
--
-- The update is not the product's own voice. It is attributed to the
-- writer and to the reviewer, the checks it passed are stored beside it,
-- and the hash of the payload names what it was written from. An update
-- that cannot be re-checked is just a sentence.
--
-- Only a shipped update is kept: a draft that did not pass the checks,
-- and one the reviewer discarded, keep nothing.
CREATE TABLE run_updates (
    id            INTEGER NOT NULL PRIMARY KEY,
    activity_id   INTEGER NOT NULL,          -- the run it is about
    writer        VARCHAR(96) NOT NULL,      -- the writer's model, by name
    reviewer      VARCHAR(96) NOT NULL,      -- the reviewer's model, by name
    payload_hash  TEXT NOT NULL,             -- the SHA-256 of the payload the model was given
    draft         TEXT NOT NULL,             -- the model's own words
    checks        TEXT NOT NULL,             -- the checks and their findings, as JSON
    verdict       VARCHAR(16) NOT NULL,      -- 'ship'
    created_at    DATETIME NOT NULL,         -- when it was made, ISO-8601 UTC
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE,
    CONSTRAINT uq_run_updates_activity UNIQUE (activity_id)
);
"#,
    // The table added in version 5: the update's own settings.
    //
    // Which model writes, which model reviews, and whether anything may
    // leave the machine at all. The **key is not here**: the settings
    // name the environment variable the key is read from, so the key
    // itself is never in the database and never in an export.
    r#"-- Old: (nothing - the old app had no model to write with). v5: the
-- update's own settings: the writer, the reviewer, and whether anything
-- may leave the machine at all.
--
-- The feature is off until the athlete turns it on, and the two models
-- are named here because they are a choice and not a default. The key
-- itself is never stored: the settings name the environment variable it
-- is read from.
CREATE TABLE update_settings (
    id                INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
    enabled           INTEGER NOT NULL DEFAULT 0,  -- off until the athlete turns it on
    allow_remote      INTEGER NOT NULL DEFAULT 0,  -- off: nothing leaves the machine
    writer_model      VARCHAR(96) NOT NULL DEFAULT '',
    writer_family     VARCHAR(48) NOT NULL DEFAULT '',
    writer_base_url   VARCHAR(256) NOT NULL DEFAULT '',
    reviewer_model    VARCHAR(96) NOT NULL DEFAULT '',
    reviewer_family   VARCHAR(48) NOT NULL DEFAULT '',
    reviewer_base_url VARCHAR(256) NOT NULL DEFAULT '',
    key_env           VARCHAR(64) NOT NULL DEFAULT 'STRIDE_MODEL_KEY',
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

/// The indexes that version 4 adds, in creation order.
///
/// They are separate from [`INDEX_STATEMENTS`] because an index added to an
/// older step would never be created on a database that already ran it.
pub const V4_INDEX_STATEMENTS: &[&str] = &[
    r#"CREATE INDEX ix_wellness_observations_date ON wellness_observations (date);
"#,
];

/// The indexes that version 5 adds, in creation order.
pub const V5_INDEX_STATEMENTS: &[&str] = &[
    r#"CREATE INDEX ix_run_updates_activity ON run_updates (activity_id);
"#,
];

/// Every statement of the schema, in order: the tables first, then the
/// indexes.
pub fn all_statements() -> Vec<&'static str> {
    TABLE_STATEMENTS
        .iter()
        .chain(INDEX_STATEMENTS.iter())
        .chain(V4_INDEX_STATEMENTS.iter())
        .chain(V5_INDEX_STATEMENTS.iter())
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
/// Version 1 is the schema of the original freeze. Version 2 adds the
/// tombstone table for the runs Garmin no longer lists. Version 3 adds the
/// heart-rate zone model table. Version 4 adds the wellness observations
/// table. Version 5 adds the run update's table and the update's own
/// settings. A database at an older version gets the missing steps, each in
/// its own transaction.
///
/// The first seven tables of [`TABLE_STATEMENTS`] are version 1; the eighth
/// — the tombstone table — is version 2; the ninth — the zone model table —
/// is version 3; the tenth — the wellness observations table — is version 4;
/// the eleventh and the twelfth — the run update and its settings — are
/// version 5.
pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            statements: TABLE_STATEMENTS[..7]
                .iter()
                .chain(INDEX_STATEMENTS.iter())
                .copied()
                .collect(),
        },
        Migration {
            version: 2,
            statements: TABLE_STATEMENTS[7..8].to_vec(),
        },
        Migration {
            version: 3,
            statements: TABLE_STATEMENTS[8..9].to_vec(),
        },
        Migration {
            version: 4,
            statements: TABLE_STATEMENTS[9..10]
                .iter()
                .chain(V4_INDEX_STATEMENTS.iter())
                .copied()
                .collect(),
        },
        Migration {
            version: SCHEMA_VERSION,
            statements: TABLE_STATEMENTS[10..]
                .iter()
                .chain(V5_INDEX_STATEMENTS.iter())
                .copied()
                .collect(),
        },
    ]
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
