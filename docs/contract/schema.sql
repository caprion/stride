-- Stride v2 — the frozen database schema.
--
-- FROZEN. This file is part of the contract freeze (Slice 0). Changing it is a
-- plan change and must travel with a PLAN.md edit in the same pull request.
-- See docs/contract/README.md for what "frozen" means.
--
-- The shape is the plan's (§6.1), not the old repo's. The old implementation
-- (backend/app/models.py + crates/stride-store/src/schema.rs) is the source for
-- the column types below, but it is a one-athlete/one-vendor ledger with
-- score columns, a gear ledger and a per-run sync log. v2 drops those: the FIT
-- file is the source of truth, derived series are recomputed on read, and no
-- score is ever stored. The mapping from old tables to v2 tables is in the
-- header of each CREATE TABLE.
--
-- The database is SQLCipher-encrypted at the file level; the key comes from the
-- caller and is never stored, returned, or logged (PLAN.md §6). The DDL itself
-- is plain SQLite and is what both hosts (the phone in-process via FFI, the
-- desktop over loopback) run.
--
-- Creation order is the migration order: parents before children, so foreign
-- keys resolve as the tables are created. Indexes are created after the tables.

-- Old: activity (+ activity_fact, activity_lap, activity_stream, run_fingerprint,
-- hr_zone, readiness). v2: one row per run; the FIT blob is the source of truth,
-- and every derived series (samples, laps, zones, fingerprint features) is
-- recomputed on read from it — never stored here.
CREATE TABLE activities (
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

-- Old: activity_stream. v2: one compressed per-second stream per activity.
-- "Windowed" = it may cover only a sub-range (first_s..last_s) and be pruned;
-- the full stream is always re-derivable from activities.fit_blob.
CREATE TABLE activity_samples (
    activity_id   INTEGER NOT NULL PRIMARY KEY,
    series        BLOB NOT NULL,                   -- compressed per-second series
    sha256        TEXT NOT NULL,                   -- hash of the decompressed series
    first_s       INTEGER,                         -- window start, seconds from run start
    last_s        INTEGER,                         -- window end, seconds from run start
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE
);

-- Old: daily_metric + daily_physiology + readiness. v2: the wellness inputs only.
-- Device scores and judgements are dropped (sleep_score, hrv_status,
-- body_battery, training_readiness, vo2max, endurance_score, running_tolerance,
-- acute/chronic load, steps) — the product never shows a device score, so none
-- is stored.
CREATE TABLE wellness_daily (
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

-- Old: plan_block (+ plan_phase, plan_week). v2: the plan block only. Exactly
-- one block per plan source is 'active'; the invariant is enforced by the
-- repository, not a partial index.
CREATE TABLE plan_blocks (
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

-- Old: plan_session (+ plan_result). v2: the intent only. done / not done /
-- planned is computed on read (from the session date, the matching activity and
-- the block's source), never stored — a baseline never yields "not done", and a
-- stored verdict would be a judgement the plan forbids. The source lives on the
-- block; the session inherits it.
CREATE TABLE plan_sessions (
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

-- Old: none (new). The only free text the app stores: a note on a run, and the
-- goal/pause settings — the three inputs that change what the app says.
CREATE TABLE notes (
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

-- Old: sync_run. v2: one row per data kind. A cursor advances only after a
-- successful parse — never before (PLAN.md §3.3).
CREATE TABLE sync_state (
    kind              VARCHAR(24) NOT NULL PRIMARY KEY, -- 'activities' | 'wellness' | 'daily_summaries'
    cursor            TEXT,                             -- the opaque resume cursor for that kind
    last_success_at   DATETIME,
    token_expires_at  DATETIME,                        -- the disposable 24 h JWT
    updated_at        DATETIME NOT NULL
);

-- Indexes, created after the tables, in this order.
CREATE INDEX ix_activities_local_date ON activities (local_date);
CREATE INDEX ix_activities_sport      ON activities (sport_type);
CREATE INDEX ix_activities_start      ON activities (start_time);
CREATE INDEX ix_plan_blocks_start_date ON plan_blocks (start_date);
CREATE INDEX ix_plan_sessions_block_date ON plan_sessions (block_id, date);
CREATE INDEX ix_plan_sessions_date     ON plan_sessions (date);
CREATE INDEX ix_notes_activity_id     ON notes (activity_id);
