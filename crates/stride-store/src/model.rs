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
