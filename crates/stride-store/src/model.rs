//! The rows.
//!
//! One struct per table, plus the "write" structs for the rows that are
//! written. A row that comes out of the database is a plain value: no
//! references into the database, no lazy reads, nothing that has to be closed.
//!
//! ## The units
//!
//! The units are the ones the core contract fixes (`docs/contract/core-contract.md`):
//! distance in **metres**, time in **seconds**, heart rate in **beats per
//! minute**, elevation in **metres**. The local date is the athlete's own
//! date, `YYYY-MM-DD`, and the start time is a UTC instant.
//!
//! ## "No value"
//!
//! A missing measurement is `None`, never `0` and never a sentinel. There is
//! no score, no grade and no verdict anywhere in these structs — the plan
//! drops all three, and a struct that cannot hold a score cannot leak one.

use chrono::{DateTime, Utc};

/// Where a row came from.
pub mod source {
    /// Synced from the Garmin Connect Personal API.
    pub const GARMIN_PERSONAL: &str = "garmin_personal";
    /// Imported from a FIT file — any watch, any vendor.
    pub const FIT_IMPORT: &str = "fit_import";
}

/// Where a plan block came from.
pub mod plan_source {
    /// Inferred from the athlete's own history. A baseline, never a prescription.
    pub const BASELINE: &str = "baseline";
    /// Written down by the athlete — in Garmin Connect, or imported.
    pub const PRESCRIBED: &str = "prescribed";
    /// Typed into the app.
    pub const TYPED: &str = "typed";
}

/// The status of a plan block.
pub mod plan_status {
    /// The block that is in force.
    pub const ACTIVE: &str = "active";
    /// A block that is over, or that was replaced.
    pub const ARCHIVED: &str = "archived";
}

/// The kind of a note.
pub mod note_kind {
    /// A note on a run.
    pub const NOTE: &str = "note";
    /// The weekly goal, in kilometres.
    pub const GOAL: &str = "goal";
    /// A pause in the plan.
    pub const PAUSE: &str = "pause";
}

/// The kinds of data a sync keeps a cursor for.
pub mod sync_kind {
    /// The activity list and the activity details.
    pub const ACTIVITIES: &str = "activities";
    /// The daily wellness: sleep, HRV, resting heart rate.
    pub const WELLNESS: &str = "wellness";
    /// The daily summaries.
    pub const DAILY_SUMMARIES: &str = "daily_summaries";
}

/// One run.
///
/// The FIT file is in here: the blob is the source of truth, and everything
/// else in this row is derived from it and recomputable.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// The row's primary key.
    pub id: i64,
    /// Where the run came from: `garmin_personal` or `fit_import`.
    pub source: String,
    /// The vendor's id for the run.
    pub external_id: String,
    /// The run's title.
    pub name: String,
    /// The normalised sport: `running`, `cycling`, …
    pub sport_type: String,
    /// When the run started, in UTC.
    pub start_time: DateTime<Utc>,
    /// The athlete's local date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// The elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The average heart rate, in beats per minute. `None` = no value.
    pub avg_hr: Option<f64>,
    /// The maximum heart rate, in beats per minute. `None` = no value.
    pub max_hr: Option<f64>,
    /// The original FIT file.
    pub fit_blob: Vec<u8>,
    /// The SHA-256 of the FIT file.
    pub fit_sha256: String,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
    /// When the row was last changed.
    pub updated_at: DateTime<Utc>,
}

/// A run without its FIT file.
///
/// The feed and the date range read this one: a FIT file is ten kilobytes or
/// more, and a list of fifty runs does not need fifty of them. The blob is
/// one call away — [`crate::repos::activity::Store::fit_blob`] — and it is
/// always there.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivitySummary {
    /// The row's primary key.
    pub id: i64,
    /// Where the run came from.
    pub source: String,
    /// The vendor's id for the run.
    pub external_id: String,
    /// The run's title.
    pub name: String,
    /// The normalised sport.
    pub sport_type: String,
    /// When the run started, in UTC.
    pub start_time: DateTime<Utc>,
    /// The athlete's local date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// The elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The average heart rate. `None` = no value.
    pub avg_hr: Option<f64>,
    /// The maximum heart rate. `None` = no value.
    pub max_hr: Option<f64>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
    /// When the row was last changed.
    pub updated_at: DateTime<Utc>,
}

impl Activity {
    /// The row without its FIT file.
    pub fn summary(&self) -> ActivitySummary {
        ActivitySummary {
            id: self.id,
            source: self.source.clone(),
            external_id: self.external_id.clone(),
            name: self.name.clone(),
            sport_type: self.sport_type.clone(),
            start_time: self.start_time,
            local_date: self.local_date.clone(),
            distance_m: self.distance_m,
            moving_time_s: self.moving_time_s,
            elapsed_time_s: self.elapsed_time_s,
            elevation_gain_m: self.elevation_gain_m,
            avg_hr: self.avg_hr,
            max_hr: self.max_hr,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// A run to write.
#[derive(Debug, Clone, PartialEq)]
pub struct NewActivity {
    /// Where the run came from.
    pub source: String,
    /// The vendor's id for the run.
    pub external_id: String,
    /// The run's title.
    pub name: String,
    /// The normalised sport.
    pub sport_type: String,
    /// When the run started, in UTC.
    pub start_time: DateTime<Utc>,
    /// The athlete's local date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// The elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The average heart rate. `None` = no value.
    pub avg_hr: Option<f64>,
    /// The maximum heart rate. `None` = no value.
    pub max_hr: Option<f64>,
    /// The original FIT file.
    pub fit_blob: Vec<u8>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
    /// When the row was last changed.
    pub updated_at: DateTime<Utc>,
}

/// The derived series of one run.///
/// The series may be **windowed**: it may cover only part of the run, and it
/// may be pruned. The full series is always re-derivable from the FIT blob,
/// which is why the blob is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivitySamples {
    /// The run the series belongs to.
    pub activity_id: i64,
    /// The compressed series.
    pub series: Vec<u8>,
    /// The SHA-256 of the decompressed series.
    pub sha256: String,
    /// The first second of the window, from the start of the run.
    pub first_s: Option<i64>,
    /// The last second of the window, from the start of the run.
    pub last_s: Option<i64>,
}

/// The derived series of one run, to write.
///
/// The run's id is not part of it: the series are written together with the
/// run, and the store knows which run they belong to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewActivitySamples {
    /// The compressed series.
    pub series: Vec<u8>,
    /// The SHA-256 of the decompressed series.
    pub sha256: String,
    /// The first second of the window, from the start of the run.
    pub first_s: Option<i64>,
    /// The last second of the window, from the start of the run.
    pub last_s: Option<i64>,
}

/// One day of wellness.
#[derive(Debug, Clone, PartialEq)]
pub struct WellnessDaily {
    /// The athlete's local date, `YYYY-MM-DD`.
    pub date: String,
    /// The sleep, in seconds. `None` = no value.
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
    /// The average stress. Present only where the watch reports it.
    pub stress_avg: Option<f64>,
    /// Where the row came from.
    pub source: String,
    /// When the row was last changed.
    pub updated_at: DateTime<Utc>,
}

/// One observation the wellness source reported, with its provenance.
///
/// An observation is **the source's own reading**: the value, the day the
/// source reported it for, and the day it was read. Nothing here is derived
/// and nothing here is a verdict.
///
/// The missingness is part of the row: a field the source never sent has no
/// row at all, and a field it sent as null has a row whose value is `None`
/// and whose text is `None` too. A row with a text value is a value that is
/// not a number.
#[derive(Debug, Clone, PartialEq)]
pub struct WellnessObservation {
    /// The row's primary key.
    pub id: i64,
    /// The day the source reported the value for, `YYYY-MM-DD`.
    pub date: String,
    /// Where the reading came from: `garmin_personal`.
    pub source: String,
    /// The API path the value was read from.
    pub endpoint: String,
    /// The source's own field name.
    pub field: String,
    /// Which item of a repeated field this is. Zero for a field that is not
    /// repeated.
    pub ordinal: i64,
    /// The number, as the source reported it. `None` when the source sent
    /// null, or when the value is not a number.
    pub value: Option<f64>,
    /// A value that is not a number, as the source reported it.
    pub text_value: Option<String>,
    /// The unit the source states for the number.
    pub unit: Option<String>,
    /// The instant the item is about, when the source gave one.
    pub observed_at: Option<DateTime<Utc>>,
    /// When the value was read.
    pub retrieved_at: DateTime<Utc>,
}

/// One observation to write.
///
/// The row is written over the one already stored for the same day, source,
/// field and item: a later read replaces the value it carried and keeps the
/// reading time of the read that carried it.
#[derive(Debug, Clone, PartialEq)]
pub struct NewWellnessObservation {
    /// The day the source reported the value for, `YYYY-MM-DD`.
    pub date: String,
    /// Where the reading came from.
    pub source: String,
    /// The API path the value was read from.
    pub endpoint: String,
    /// The source's own field name.
    pub field: String,
    /// Which item of a repeated field this is.
    pub ordinal: i64,
    /// The number, as the source reported it.
    pub value: Option<f64>,
    /// A value that is not a number, as the source reported it.
    pub text_value: Option<String>,
    /// The unit the source states for the number.
    pub unit: Option<String>,
    /// The instant the item is about, when the source gave one.
    pub observed_at: Option<DateTime<Utc>>,
    /// When the value was read.
    pub retrieved_at: DateTime<Utc>,
}

/// The athlete's heart-rate zone model, as Garmin reported it on one day.
///
/// The model is a **dated fact about the athlete's configuration**, not a
/// series derived from a run: it is the watch's own zone floors, read from
/// `/biometric-service/heartRateZones/v2`, and kept with the day it was
/// read. One row is written per model change, so an insight can say where
/// its input came from and when it was read.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartRateZoneModel {
    /// The row's primary key.
    pub id: i64,
    /// The sport Garmin reported the model for, e.g. `DEFAULT`.
    pub sport: String,
    /// Garmin's training method: `HR_MAX`, `LACTATE_THRESHOLD`, `HR_RESERVE`
    /// or `CUSTOM`.
    pub training_method: String,
    /// The maximum heart rate the model was built on. `None` = not reported.
    pub max_heart_rate: Option<f64>,
    /// The lower boundary of zone 1, in beats per minute.
    pub zone1_floor: f64,
    /// The lower boundary of zone 2.
    pub zone2_floor: f64,
    /// The lower boundary of zone 3.
    pub zone3_floor: f64,
    /// The lower boundary of zone 4.
    pub zone4_floor: f64,
    /// The lower boundary of zone 5.
    pub zone5_floor: f64,
    /// When the model was read from Garmin.
    pub read_at: DateTime<Utc>,
}

/// A heart-rate zone model to write.
#[derive(Debug, Clone, PartialEq)]
pub struct NewHeartRateZoneModel {
    /// The sport Garmin reported the model for.
    pub sport: String,
    /// Garmin's training method.
    pub training_method: String,
    /// The maximum heart rate the model was built on. `None` = not reported.
    pub max_heart_rate: Option<f64>,
    /// The lower boundary of zone 1, in beats per minute.
    pub zone1_floor: f64,
    /// The lower boundary of zone 2.
    pub zone2_floor: f64,
    /// The lower boundary of zone 3.
    pub zone3_floor: f64,
    /// The lower boundary of zone 4.
    pub zone4_floor: f64,
    /// The lower boundary of zone 5.
    pub zone5_floor: f64,
    /// When the model was read from Garmin.
    pub read_at: DateTime<Utc>,
}

/// A plan block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanBlock {
    /// The row's primary key.
    pub id: i64,
    /// The block's name.
    pub name: String,
    /// Where the block came from: `baseline`, `prescribed` or `typed`.
    pub source: String,
    /// The first day of the block, `YYYY-MM-DD`.
    pub start_date: String,
    /// The last day of the block. `None` = open-ended.
    pub end_date: Option<String>,
    /// The block's goal.
    pub goal: Option<String>,
    /// `active` or `archived`.
    pub status: String,
    /// The block's notes.
    pub notes: Option<String>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
}

/// A plan block to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlanBlock {
    /// The block's name.
    pub name: String,
    /// Where the block comes from.
    pub source: String,
    /// The first day of the block, `YYYY-MM-DD`.
    pub start_date: String,
    /// The last day of the block. `None` = open-ended.
    pub end_date: Option<String>,
    /// The block's goal.
    pub goal: Option<String>,
    /// `active` or `archived`.
    pub status: String,
    /// The block's notes.
    pub notes: Option<String>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
}

/// One planned session.
///
/// The session is the **intent** only. Whether it was done, not done or is
/// still planned is computed on read — a stored verdict would be a judgement
/// the plan forbids, and a baseline never yields "not done".
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSession {
    /// The row's primary key.
    pub id: i64,
    /// The block the session belongs to.
    pub block_id: i64,
    /// The day of the session, `YYYY-MM-DD`.
    pub date: String,
    /// The kind of planned session — `easy`, `tempo`, `long`, `interval`, `race`,
    /// `rest`, `strength` or `unclassified`.
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

/// A planned session to write.
#[derive(Debug, Clone, PartialEq)]
pub struct NewPlanSession {
    /// The block the session belongs to.
    pub block_id: i64,
    /// The day of the session, `YYYY-MM-DD`.
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

/// One note.
///
/// The only free text the app stores: a note on a run, and the goal and pause
/// settings — the three inputs that change what the app says.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// The row's primary key.
    pub id: i64,
    /// `note`, `goal` or `pause`.
    pub kind: String,
    /// The run the note is about. Set for `kind = 'note'`.
    pub activity_id: Option<i64>,
    /// The note's text, or the pause's reason.
    pub text: Option<String>,
    /// The weekly goal, in kilometres. For `kind = 'goal'`.
    pub value: Option<f64>,
    /// The first day of the pause, `YYYY-MM-DD`.
    pub start_date: Option<String>,
    /// The last day of the pause, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
}

/// A note to write.
#[derive(Debug, Clone, PartialEq)]
pub struct NewNote {
    /// `note`, `goal` or `pause`.
    pub kind: String,
    /// The run the note is about. Set for `kind = 'note'`.
    pub activity_id: Option<i64>,
    /// The note's text, or the pause's reason.
    pub text: Option<String>,
    /// The weekly goal, in kilometres. For `kind = 'goal'`.
    pub value: Option<f64>,
    /// The first day of the pause, `YYYY-MM-DD`.
    pub start_date: Option<String>,
    /// The last day of the pause, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// When the row was written.
    pub created_at: DateTime<Utc>,
}

/// The state of one kind of sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncState {
    /// Which kind of data: `activities`, `wellness` or `daily_summaries`.
    pub kind: String,
    /// The opaque resume cursor.
    pub cursor: Option<String>,
    /// When the last sync of this kind succeeded.
    pub last_success_at: Option<DateTime<Utc>>,
    /// When the disposable 24-hour token expires.
    pub token_expires_at: Option<DateTime<Utc>>,
    /// When the row was last changed.
    pub updated_at: DateTime<Utc>,
}

/// The verdict an update may carry.
///
/// The reviewer's three answers: ship, repair, discard. **Only a shipped
/// update is kept** — a repair is a second pass, and a discard keeps
/// nothing.
pub mod verdict {
    /// The draft is true to the payload and written in the register.
    pub const SHIP: &str = "ship";
    /// The draft can be corrected, and the reviewer named what to correct.
    pub const REPAIR: &str = "repair";
    /// The draft cannot be corrected, and nothing is kept.
    pub const DISCARD: &str = "discard";
}

/// One run's update.
///
/// The update is a model's own reading of the run: it is **attributed**,
/// and what it was checked against is stored beside it. The payload hash
/// names the payload the model was given, so the update can be re-checked
/// instead of believed.
#[derive(Debug, Clone, PartialEq)]
pub struct RunUpdate {
    /// The row's primary key.
    pub id: i64,
    /// The run it is about.
    pub activity_id: i64,
    /// The writer's model, by name.
    pub writer: String,
    /// The reviewer's model, by name.
    pub reviewer: String,
    /// The SHA-256 of the payload the model was given.
    pub payload_hash: String,
    /// The model's own words.
    pub draft: String,
    /// The checks and their findings, as JSON.
    pub checks: String,
    /// The verdict: `ship`.
    pub verdict: String,
    /// When the update was made.
    pub created_at: DateTime<Utc>,
}

/// A run update to write.
#[derive(Debug, Clone, PartialEq)]
pub struct NewRunUpdate {
    /// The run it is about.
    pub activity_id: i64,
    /// The writer's model, by name.
    pub writer: String,
    /// The reviewer's model, by name.
    pub reviewer: String,
    /// The SHA-256 of the payload the model was given.
    pub payload_hash: String,
    /// The model's own words.
    pub draft: String,
    /// The checks and their findings, as JSON.
    pub checks: String,
    /// The verdict: `ship`.
    pub verdict: String,
    /// When the update was made.
    pub created_at: DateTime<Utc>,
}

/// The update's own settings.
///
/// Which model writes, which model reviews, and whether anything may leave
/// the machine at all. The **key is not here**: the settings name the
/// environment variable it is read from, so the key is never in the
/// database and never in an export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSettings {
    /// Is the feature on? It is off until the athlete turns it on.
    pub enabled: bool,
    /// May anything leave the machine? It may not, until the athlete says so.
    pub allow_remote: bool,
    /// The writer's model, by name.
    pub writer_model: String,
    /// The writer's family: the models of one family read alike, and a
    /// reviewer of the same family repeats the writer's misreadings.
    pub writer_family: String,
    /// The base URL the writer is called at.
    pub writer_base_url: String,
    /// The reviewer's model, by name.
    pub reviewer_model: String,
    /// The reviewer's family. It has to differ from the writer's.
    pub reviewer_family: String,
    /// The base URL the reviewer is called at.
    pub reviewer_base_url: String,
    /// The environment variable the key is read from.
    pub key_env: String,
    /// When the settings were last written.
    pub updated_at: DateTime<Utc>,
}

impl UpdateSettings {
    /// The settings as they are before anything is written.
    ///
    /// The feature is off and nothing may leave the machine. The **models
    /// are named**, though: a season of runs was measured against them, and
    /// the choice that came out of it is the pair the app starts from —
    /// the writer that was clean on 105 of 107 runs and the reviewer that
    /// fixes a sign error, which is a different family on purpose.
    ///
    /// The athlete can change every one of them; these are the ones the
    /// evidence chose, and the settings screen shows them.
    pub fn defaults(at: DateTime<Utc>) -> UpdateSettings {
        UpdateSettings {
            enabled: false,
            allow_remote: false,
            writer_model: Self::DEFAULT_WRITER_MODEL.to_string(),
            writer_family: Self::DEFAULT_WRITER_FAMILY.to_string(),
            writer_base_url: Self::DEFAULT_WRITER_BASE_URL.to_string(),
            reviewer_model: Self::DEFAULT_REVIEWER_MODEL.to_string(),
            reviewer_family: Self::DEFAULT_REVIEWER_FAMILY.to_string(),
            reviewer_base_url: Self::DEFAULT_REVIEWER_BASE_URL.to_string(),
            key_env: Self::DEFAULT_KEY_ENV.to_string(),
            updated_at: at,
        }
    }

    /// The environment variable the key is read from, when there is none.
    pub const DEFAULT_KEY_ENV: &'static str = "STRIDE_MODEL_KEY";

    /// The writer the season measured: the fastest clean writer, and the
    /// cheapest one that reaches the bar.
    pub const DEFAULT_WRITER_MODEL: &str = "deepseek-flash";
    /// The writer's family.
    pub const DEFAULT_WRITER_FAMILY: &str = "deepseek";
    /// Where the writer is called: DeepSeek's own OpenAI-compatible route.
    pub const DEFAULT_WRITER_BASE_URL: &str = "https://api.deepseek.com/v1";

    /// The reviewer the season measured: the only one of them that fixes a
    /// signed direction, and a different family from the writer.
    pub const DEFAULT_REVIEWER_MODEL: &str = "gemini-3.1-flash-lite";
    /// The reviewer's family.
    pub const DEFAULT_REVIEWER_FAMILY: &str = "gemini";
    /// Where the reviewer is called: Google's OpenAI-compatible route.
    pub const DEFAULT_REVIEWER_BASE_URL: &str =
        "https://generativelanguage.googleapis.com/v1beta/openai";
}
