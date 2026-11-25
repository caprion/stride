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
--
-- This table stays the product's *projection*: the three vitals the screens
-- read. The source's own readings — the sleep window, the stress durations,
-- the body battery, the seven-day average resting heart rate, the respiration
-- — are kept whole, with their provenance, in `wellness_observations` (v4).
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
    kind        VARCHAR(16) NOT NULL,               -- 'note' | 'goal' | 'pause' | 'challenge_week' | 'challenge_run'
    activity_id INTEGER,                            -- set for kind='note'
    text        TEXT,                               -- the note text, or the pause reason
    value       REAL,                               -- the weekly goal in km (kind='goal'), or a challenge's target in km
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

-- Old: (nothing - the old app never learned that a run was deleted).
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

-- Old: (nothing - the old app stored per-run zone bands, never the model
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

-- Old: (nothing - the old app stored a projection of the wellness and
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

-- Old: (nothing - the old app never wrote about a run in words). v5:
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

-- Old: (nothing - the old app had no model to write with). v5: the
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

-- Old: (nothing - the old app never held a race the athlete ran). v6: the
-- athlete's own race: the distance, the time, the day it was run, the day it
-- was entered, the day it was last changed, and the label the athlete gave
-- it.
--
-- A race is an **observation**, not a verdict. The athlete types it, and
-- Stride never invents one and never infers one from the training. The
-- distance is in metres, the contract's own unit (core contract §1), and the
-- label is a closed list that names the distance the race was at.
--
-- The anchor is the race the paces come from. At most one row carries it,
-- and it never moves by itself: entering another race leaves it where the
-- athlete pointed it.
CREATE TABLE race_observations (
    id          INTEGER NOT NULL PRIMARY KEY,
    distance_m  REAL NOT NULL,             -- the race's distance, in metres
    time_s      REAL NOT NULL,             -- the finish time, in seconds
    race_date   VARCHAR(10) NOT NULL,      -- the day of the race, YYYY-MM-DD
    label       VARCHAR(24) NOT NULL,      -- '5K' | '10K' | '15K' | '10-mile' |
                                            -- 'half marathon' | 'marathon'
    name        TEXT,                      -- the athlete's own words for the race
    source      VARCHAR(16) NOT NULL,      -- 'typed' - the only source there is
    entered_on  VARCHAR(10) NOT NULL,      -- the day it was entered
    updated_on  VARCHAR(10) NOT NULL,      -- the day it was last changed
    anchor      INTEGER NOT NULL DEFAULT 0 -- 1 when the paces come from this race
);

-- Old: (nothing - the old app had no threshold of the athlete's own). v6:
-- the threshold the athlete uses: one dated fact per choice, with its
-- source.
--
-- The candidates are **computed and never stored**; this table holds what
-- the athlete picked, and the history of the picks. A choice may carry a
-- heart rate, a pace band (the band's two ends), or **both** - a typed
-- value from a lab test carries the pair. The kind is what the row carries,
-- and a missing kind is a missing kind, never a zero.
--
-- **At most one row carries `in_force = 1`.** One threshold at a time; the
-- earlier choice stays in the dated history with its base named, and a plan
-- adopted in March is paced by March's number.
CREATE TABLE threshold_observations (
    id                 INTEGER NOT NULL PRIMARY KEY,
    hr_bpm             REAL,                -- NULL when the choice carries no heart rate
    pace_low_s_per_km  REAL,                -- NULL when the choice carries no pace
    pace_high_s_per_km REAL,                -- NULL when the choice carries no pace
    source             VARCHAR(24) NOT NULL,-- 'watch_zone_model' | 'race' |
                                            -- 'field_test' | 'typed'
    rule_id            VARCHAR(48),         -- the named rule that computed it
    base               VARCHAR(24),         -- the base it was made for:
                                            -- 'pfitzinger' | 'eighty_twenty'
    origin_date        VARCHAR(10),         -- the day of the underlying fact
    origin_ref         VARCHAR(64),         -- the model / race / run it came from
    label              TEXT,                -- the athlete's own label; 'typed' only
    entered_on         VARCHAR(10) NOT NULL,-- the day the choice was made
    updated_on         VARCHAR(10) NOT NULL,-- the day the row was last changed
    in_force           INTEGER NOT NULL DEFAULT 0 -- 1 = the threshold in use now
);

-- Old: (nothing - the old app stored a projection of the activity and
-- dropped the rest of it). v6: the source's own facts about an activity,
-- kept with their provenance: the value, the day the response carried it,
-- and the day it was read. One row per (activity, source, field, item).
--
-- The missingness is kept as the source gave it: a field the source never
-- sent has no row at all, and a field the source sent as null has a row
-- whose value is null. A later read never deletes a row - it replaces the
-- value it carried and keeps the reading time of the read that carried it -
-- so a failed read leaves the previous value and its reading time in place.
--
-- Nothing here is derived, and nothing here is a verdict. The watch's own
-- zone minutes are the source's own record of the run, over the elapsed
-- time; Garmin's estimates - the VO2 max, the training stress score, the
-- activity training load, the lactate threshold, the intensity factor, the
-- stress and the body battery - are judgements, and they are not stored.
CREATE TABLE activity_observations (
    id           INTEGER NOT NULL PRIMARY KEY,
    activity_id  INTEGER NOT NULL,           -- the run it is about
    source       VARCHAR(24) NOT NULL,       -- 'garmin_personal'
    endpoint     VARCHAR(96) NOT NULL,       -- the API path the value was read from
    field        VARCHAR(48) NOT NULL,       -- the source's own field name
    ordinal      INTEGER NOT NULL DEFAULT 0, -- which item of a repeated field
    value        REAL,                       -- the number, as reported; NULL when the source sent null
    text_value   TEXT,                       -- a value that is not a number, as reported
    unit         VARCHAR(16),                -- the unit the source states for the number
    observed_at  DATETIME,                   -- the instant the item is about, when the source gave one
    retrieved_at DATETIME NOT NULL,          -- when the value was read, ISO-8601 UTC
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE,
    CONSTRAINT uq_activity_observations UNIQUE (activity_id, source, field, ordinal)
);

-- Old: (nothing - the old app had no adoptable programme). v7: the adopted
-- plan: a **dated copy** of a programme from the catalogue, with the sessions
-- that were worked out on the day it was adopted.
--
-- The catalogue is data and it may be corrected later, so the plan keeps the
-- programme's id **and the version** it was adopted from, and the sessions
-- are stored with it. A later correction to the catalogue cannot alter a
-- plan that is running, and a later threshold does not re-pace it either:
-- the paces were worked out once, at the adoption, from the threshold that
-- was in force that day.
--
-- The plan keeps the record of what it was paced with: the threshold's value
-- and the day it was read, the source it came from, and the base it was made
-- for. **One plan is in force at a time** - a second plan cannot start
-- while another one is running.
--
-- A plan adopted before a race was typed carries its sessions without paces,
-- and says why: `paced` is 0 and `pacing_missing` holds the reason.
CREATE TABLE adopted_plans (
    id                INTEGER NOT NULL PRIMARY KEY,
    template_id       VARCHAR(96) NOT NULL,   -- the programme's catalogue id
    template_version  VARCHAR(16) NOT NULL,   -- the catalogue version it was adopted from
    label             VARCHAR(160) NOT NULL,  -- the programme's own name: 'half marathon tier 2'
    method            VARCHAR(64) NOT NULL,   -- whose method it is: 'Pfitzinger'
    tier              VARCHAR(32) NOT NULL,   -- the tier the athlete chose: 'tier 2'
    goal_distance_m   REAL NOT NULL,          -- the goal race's own distance, in metres
    goal_label        VARCHAR(32) NOT NULL,   -- the distance's name: 'half marathon'
    race_date         VARCHAR(10) NOT NULL,   -- the day of the goal race, YYYY-MM-DD
    start_date        VARCHAR(10) NOT NULL,   -- the plan's first day, YYYY-MM-DD
    end_date          VARCHAR(10) NOT NULL,   -- the plan's last day, YYYY-MM-DD
    adopted_on        VARCHAR(10) NOT NULL,   -- the day the plan was adopted
    threshold_value   VARCHAR(96),            -- the threshold it was paced with: '165 bpm'
    threshold_date    VARCHAR(10),            -- the day that threshold was read
    threshold_source  VARCHAR(24),            -- 'watch_zone_model' | 'race' |
                                             -- 'field_test' | 'typed'
    threshold_base    VARCHAR(24),            -- the base it was made for:
                                             -- 'pfitzinger' | 'eighty_twenty'
    rule_value        VARCHAR(64),            -- the race the plan's own rule was read from
    rule_date         VARCHAR(10),            -- the day of that race
    paced             INTEGER NOT NULL DEFAULT 0, -- 1 when the sessions carry paces
    pacing_missing    TEXT,                   -- the reason the sessions have no pace
    source            VARCHAR(24) NOT NULL DEFAULT 'prescribed', -- an adopted plan is a prescription
    status            VARCHAR(16) NOT NULL DEFAULT 'active', -- 'active' | 'archived'
    created_at        DATETIME NOT NULL
);

-- Old: (nothing - the old app never held an adopted programme). v7: the
-- plan's own dated sessions: the day, the kind, the distance, the pace
-- where the session has one, the pacing's source, and the detail the book
-- prints for that day.
--
-- The sessions are **stored, not derived**: the paces were worked out once,
-- at the adoption, and a catalogue correction or a new threshold would
-- silently re-pace a running plan if they were derived.
--
-- A session with no pace is a session with no pace - never a zero pace.
-- The pace's two ends are NULL when the session has none, and the rule and
-- the threshold beside it say where the pace came from.
CREATE TABLE adopted_sessions (
    id                  INTEGER NOT NULL PRIMARY KEY,
    plan_id             INTEGER NOT NULL,
    date                VARCHAR(10) NOT NULL, -- the day the session falls on
    week                INTEGER NOT NULL,     -- the plan's week: 11 down to 1, 0 for the race week
    kind                VARCHAR(32) NOT NULL, -- the method's own key: 'ga' | 'lt' | 'endurance' | ...
    distance_m          REAL,                 -- NULL on a rest day: a day with no distance is not a zero
    pace_low_s_per_km   REAL,                 -- NULL when the session has no pace
    pace_high_s_per_km  REAL,
    finish_low_s_per_km REAL,                 -- the long run's second end
    finish_high_s_per_km REAL,
    pace_rule           VARCHAR(48),          -- the rule the pace came from
    threshold_value     VARCHAR(96),          -- the threshold the pace came from
    detail              TEXT,                 -- the cell's own detail: '3 x 8 min, jog 3'
    lt_last_m           REAL,                 -- the progression run's last miles at LT pace
    FOREIGN KEY (plan_id) REFERENCES adopted_plans (id) ON DELETE CASCADE,
    CONSTRAINT uq_adopted_sessions UNIQUE (plan_id, date, kind)
);

-- Old: (nothing - the old app held no heart rate the athlete gave it).
-- v8: the athlete's own heart rates: the maximum and/or the resting rate,
-- with where each came from, the day it was measured and the day it was
-- entered.
--
-- ONE DECLARATION IS ONE OR TWO ROWS. A maximum and a resting rate entered
-- together are one declaration with two rows, and a later pair is a new
-- declaration: the earlier one stays in the history with its own dates and
-- its own sources.
--
-- The declaration a row belongs to is named by `declaration_id`, so the two
-- rows of one declaration are never read as two declarations. **One
-- declaration is in force at a time** - the most recent one - and the
-- earlier declarations stay.
--
-- A missing value is a missing value: the row is not there, and it is never
-- a zero. The derived bands are NOT stored - they are recomputed on read
-- from the declaration in force, by the rule in
-- exploration/heart-rate-setup.md.
CREATE TABLE heart_rate_declarations (
    id              INTEGER NOT NULL PRIMARY KEY,
    declaration_id  INTEGER NOT NULL,       -- the declaration the row belongs to
    kind            VARCHAR(16) NOT NULL,   -- 'max' | 'resting'
    value_bpm       REAL NOT NULL,          -- the heart rate, in beats per minute
    source          VARCHAR(24) NOT NULL,   -- 'watch' | 'test' | 'doctor_lab' |
                                             -- 'partner_watch' | 'typed'
    measured_on     VARCHAR(10),            -- the day it was measured; NULL when not stated
    entered_on      VARCHAR(10) NOT NULL,   -- the day the declaration was made
    updated_on      VARCHAR(10) NOT NULL    -- the day the row was last changed
);

-- Indexes, created after the tables, in this order.
CREATE INDEX ix_activities_local_date ON activities (local_date);
CREATE INDEX ix_activities_sport      ON activities (sport_type);
CREATE INDEX ix_activities_start      ON activities (start_time);
CREATE INDEX ix_plan_blocks_start_date ON plan_blocks (start_date);
CREATE INDEX ix_plan_sessions_block_date ON plan_sessions (block_id, date);
CREATE INDEX ix_plan_sessions_date     ON plan_sessions (date);
CREATE INDEX ix_notes_activity_id     ON notes (activity_id);
CREATE INDEX ix_wellness_observations_date ON wellness_observations (date);
CREATE INDEX ix_run_updates_activity ON run_updates (activity_id);
CREATE INDEX ix_race_observations_race_date ON race_observations (race_date);
CREATE INDEX ix_threshold_observations_entered_on ON threshold_observations (entered_on);
CREATE INDEX ix_activity_observations_activity ON activity_observations (activity_id);
CREATE INDEX ix_adopted_plans_start_date ON adopted_plans (start_date);
CREATE INDEX ix_adopted_plans_race_date ON adopted_plans (race_date);
CREATE INDEX ix_adopted_sessions_plan_date ON adopted_sessions (plan_id, date);
CREATE INDEX ix_heart_rate_declarations_entered_on ON heart_rate_declarations (entered_on);
