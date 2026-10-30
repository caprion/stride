//! The seam between the router and the rest of the system.
//!
//! The router owns HTTP: the path table, the query string, the response
//! shapes, the labels and the error bodies. It does not own the data. That
//! lives in `stride-store` (the rows) and `stride-core` (the derived
//! numbers), and this trait is the boundary between them.
//!
//! **Why the seam is in terms of rows, not store types.** `stride-store`'s
//! row structs are not in the frozen contract, and `stride-core` does not
//! depend on `stride-api`, so the store cannot implement a trait that lives
//! here with its own types without a conversion layer that neither crate
//! owns. The rows below are therefore plain data with the same field names
//! as the SQL columns: the store adapter builds them, the router shapes them.
//!
//! **What the trait does not do.** It does not decide what a response looks
//! like. Every method here returns rows or raw numbers; the shaping — the
//! `m:ss` labels, the `pace_s_per_km` derivations, the `None` that stands for
//! "not measured" — happens in this crate, so the phone and the desktop
//! cannot drift apart.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::BackendError;

/// One run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityRow {
    pub id: i64,
    /// The athlete the run belongs to. `None` is the athlete-less row.
    #[serde(default)]
    pub athlete_id: Option<i64>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub external_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sport_type: String,
    #[serde(default)]
    pub description: Option<String>,
    /// ISO-8601, as the Python app renders `start_time`.
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub local_date: String,
    #[serde(default)]
    pub day_of_week: Option<String>,
    #[serde(default)]
    pub distance_m: f64,
    #[serde(default)]
    pub moving_time_s: f64,
    #[serde(default)]
    pub elapsed_time_s: f64,
    #[serde(default)]
    pub elevation_gain_m: f64,
    #[serde(default)]
    pub avg_hr: Option<f64>,
    #[serde(default)]
    pub max_hr: Option<f64>,
    #[serde(default)]
    pub calories: Option<f64>,
    #[serde(default)]
    pub avg_cadence: Option<f64>,
    #[serde(default)]
    pub max_cadence: Option<f64>,
    #[serde(default)]
    pub avg_speed_ms: Option<f64>,
    #[serde(default)]
    pub max_speed_ms: Option<f64>,
    #[serde(default)]
    pub perceived_exertion: Option<i64>,
    #[serde(default)]
    pub device: Option<String>,
    #[serde(default)]
    pub gear: Option<String>,
    #[serde(default)]
    pub is_trail: bool,
    #[serde(default)]
    pub is_commute: bool,
    #[serde(default)]
    pub running_time_s: Option<f64>,
    #[serde(default)]
    pub walking_time_s: Option<f64>,
    #[serde(default)]
    pub stopped_time_s: Option<f64>,
    #[serde(default)]
    pub running_distance_m: Option<f64>,
    /// The route, as `[lat, lon]` pairs. The Python app keeps the points in a
    /// column and derives both `has_geo` and the thumbnail from it.
    #[serde(default)]
    pub geo_points: Option<Vec<[f64; 2]>>,
    /// How many splits the run has.
    #[serde(default)]
    pub lap_count: i64,
    /// Whether the run has a route, when the row does not carry the route
    /// itself. `None` means "ask the points".
    #[serde(default)]
    pub has_geo: Option<bool>,
}

impl ActivityRow {
    /// Whether the run has a route.
    pub fn has_geo(&self) -> bool {
        self.has_geo.unwrap_or_else(|| {
            self.geo_points
                .as_ref()
                .is_some_and(|points| !points.is_empty())
        })
    }
}

/// One split.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LapRow {
    /// The run the split belongs to.
    #[serde(default)]
    pub activity_id: i64,
    #[serde(default)]
    pub lap_index: i64,
    #[serde(default)]
    pub distance_m: f64,
    #[serde(default)]
    pub duration_s: f64,
    #[serde(default)]
    pub avg_speed_ms: Option<f64>,
    #[serde(default)]
    pub max_speed_ms: Option<f64>,
    #[serde(default)]
    pub avg_hr: Option<f64>,
    #[serde(default)]
    pub max_hr: Option<f64>,
    #[serde(default)]
    pub avg_cadence: Option<f64>,
    #[serde(default)]
    pub max_cadence: Option<f64>,
    #[serde(default)]
    pub stride_length_m: Option<f64>,
    #[serde(default)]
    pub elevation_gain_m: Option<f64>,
    #[serde(default)]
    pub elevation_loss_m: Option<f64>,
    #[serde(default)]
    pub start_elevation_m: Option<f64>,
    #[serde(default)]
    pub end_elevation_m: Option<f64>,
    #[serde(default)]
    pub running_distance_m: Option<f64>,
    #[serde(default)]
    pub elapsed_s: Option<f64>,
}

/// One per-second series.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamRow {
    /// The run the series belongs to.
    #[serde(default)]
    pub activity_id: i64,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub seconds: Vec<f64>,
    #[serde(default)]
    pub values: Vec<Option<f64>>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
}

/// One heart-rate zone of one run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneRow {
    /// The run the zone belongs to.
    #[serde(default)]
    pub activity_id: i64,
    #[serde(default)]
    pub zone_no: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub low_bpm: Option<f64>,
    #[serde(default)]
    pub high_bpm: Option<f64>,
    #[serde(default)]
    pub time_s: f64,
}

/// One day of physiology.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysiologyRow {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub sleep_s: Option<f64>,
    #[serde(default)]
    pub sleep_score: Option<f64>,
    #[serde(default)]
    pub deep_sleep_s: Option<f64>,
    #[serde(default)]
    pub rem_s: Option<f64>,
    #[serde(default)]
    pub light_sleep_s: Option<f64>,
    #[serde(default)]
    pub awake_s: Option<f64>,
    #[serde(default)]
    pub hrv_ms: Option<f64>,
    #[serde(default)]
    pub hrv_status: Option<String>,
    #[serde(default)]
    pub resting_hr: Option<f64>,
    #[serde(default)]
    pub max_hr: Option<f64>,
    #[serde(default)]
    pub stress_avg: Option<f64>,
    #[serde(default)]
    pub stress_max: Option<f64>,
    #[serde(default)]
    pub body_battery_min: Option<f64>,
    #[serde(default)]
    pub body_battery_avg: Option<f64>,
    #[serde(default)]
    pub body_battery_max: Option<f64>,
    #[serde(default)]
    pub steps: Option<f64>,
    #[serde(default)]
    pub training_readiness: Option<f64>,
    #[serde(default)]
    pub acute_load: Option<f64>,
    #[serde(default)]
    pub chronic_load: Option<f64>,
    #[serde(default)]
    pub vo2max: Option<f64>,
    #[serde(default)]
    pub endurance_score: Option<f64>,
    #[serde(default)]
    pub running_tolerance: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// One day of the daily-metric table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyMetricRow {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub resting_hr: Option<f64>,
    #[serde(default)]
    pub max_hr: Option<f64>,
    #[serde(default)]
    pub sleep_s: Option<f64>,
    #[serde(default)]
    pub deep_sleep_s: Option<f64>,
    #[serde(default)]
    pub hrv_ms: Option<f64>,
    #[serde(default)]
    pub stress: Option<f64>,
    #[serde(default)]
    pub steps: Option<f64>,
    #[serde(default)]
    pub body_battery: Option<f64>,
    #[serde(default)]
    pub readiness: Option<f64>,
    #[serde(default)]
    pub distance_m: f64,
}

/// A training block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanBlockRow {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub start_date: String,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub notes: Option<String>,
}

/// A phase of a block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanPhaseRow {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub block_id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub target_weekly_km_min: Option<f64>,
    #[serde(default)]
    pub target_weekly_km_max: Option<f64>,
    #[serde(default)]
    pub target_sessions: Option<i64>,
    #[serde(default)]
    pub pace_easy_low: Option<f64>,
    #[serde(default)]
    pub pace_easy_high: Option<f64>,
    #[serde(default)]
    pub pace_tempo_low: Option<f64>,
    #[serde(default)]
    pub pace_tempo_high: Option<f64>,
    #[serde(default)]
    pub pace_interval_low: Option<f64>,
    #[serde(default)]
    pub pace_interval_high: Option<f64>,
    #[serde(default)]
    pub pace_long_low: Option<f64>,
    #[serde(default)]
    pub pace_long_high: Option<f64>,
    #[serde(default)]
    pub hr_zone_target: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// A week of a phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanWeekRow {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub phase_id: i64,
    #[serde(default)]
    pub week_number: i64,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub target_km: Option<f64>,
    #[serde(default)]
    pub key_workout: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// A planned session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanSessionRow {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub week_id: Option<i64>,
    #[serde(default)]
    pub block_id: Option<i64>,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub session_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub target_distance_m: Option<f64>,
    #[serde(default)]
    pub pace_low_s_km: Option<f64>,
    #[serde(default)]
    pub pace_high_s_km: Option<f64>,
    #[serde(default)]
    pub hr_zone_target: Option<i64>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub notes: Option<String>,
}

/// The mutable application state, as the browser is allowed to see it.
///
/// The reference is `config.AppState.public_status`. The secret fields are
/// deliberately absent: the browser learns *that* a key exists, never the key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusView {
    #[serde(default)]
    pub garmin_connected: bool,
    #[serde(default)]
    pub garmin_method: Option<String>,
    #[serde(default)]
    pub athlete: Value,
    #[serde(default)]
    pub has_developer_key: bool,
    #[serde(default)]
    pub has_account_credentials: bool,
    #[serde(default)]
    pub has_personal_api_key: bool,
    #[serde(default)]
    pub pending_authorization: bool,
    #[serde(default)]
    pub network_route: String,
    #[serde(default)]
    pub exit_node: Option<String>,
    #[serde(default)]
    pub sync_days: i64,
    #[serde(default)]
    pub history_months: Option<i64>,
    #[serde(default)]
    pub health_history_days: Option<i64>,
    #[serde(default)]
    pub auto_sync: bool,
    #[serde(default)]
    pub sync_interval_minutes: i64,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub demo_visible: bool,
}

/// The mutable settings a `PATCH /sync/settings` may change.
#[derive(Debug, Clone, Default)]
pub struct SettingsPatch {
    pub sync_days: Option<i64>,
    pub auto_sync: Option<bool>,
    pub sync_interval_minutes: Option<i64>,
    pub network_route: Option<String>,
    pub exit_node: Option<String>,
    pub demo_visible: Option<bool>,
}

/// The fields a `PATCH /activities/{id}` may change.
#[derive(Debug, Clone, Default)]
pub struct ActivityPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub perceived_exertion: Option<i64>,
    pub is_trail: Option<bool>,
    pub is_commute: Option<bool>,
}

/// The counts the health route reports.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counts {
    pub activities: i64,
    pub from_garmin: i64,
    pub physiology_days: i64,
    pub plan_sessions: i64,
}

/// A map tile.
#[derive(Debug, Clone)]
pub struct Tile {
    pub bytes: Vec<u8>,
    pub source: &'static str,
}

/// What a plan document import asks for.
#[derive(Debug, Clone, Default)]
pub struct ImportRequest {
    pub format: String,
    pub content: String,
    pub name: Option<String>,
    pub anchor: String,
}

/// One run of the sync log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncRunRow {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub days: i64,
    #[serde(default)]
    pub activities_found: i64,
    #[serde(default)]
    pub activities_new: i64,
    #[serde(default)]
    pub activities_updated: i64,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Everything the router needs from the rest of the system.
///
/// Implementations: `stride_store::Store` (through an adapter, proposed in the
/// workstream report) and the in-memory backend in this crate, which the
/// tests use.
pub trait Backend: Send + Sync {
    // ------------------------------------------------------------ activities

    /// The runs, newest first, before paging. The filter is applied by the
    /// backend because the store can do it in SQL.
    fn activities(&self, filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError>;

    /// The total number of runs matching the filter, ignoring paging.
    fn activity_count(&self, filter: &ActivityFilter) -> Result<i64, BackendError>;

    /// One run.
    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError>;

    /// One run, by the provider's identifier for it.
    ///
    /// The sync asks this to tell a new run from one it has seen before.
    ///
    /// The default is "no such run": a backend that keeps its rows somewhere
    /// this trait cannot reach says so rather than pretending.
    fn activity_by_external(
        &self,
        _source: &str,
        _external_id: &str,
    ) -> Result<Option<ActivityRow>, BackendError> {
        Ok(None)
    }

    /// The splits of one run.
    fn activity_laps(&self, id: i64) -> Result<Vec<LapRow>, BackendError>;

    /// The series of one run.
    fn activity_streams(&self, id: i64) -> Result<Vec<StreamRow>, BackendError>;

    /// The heart-rate zones of one run.
    fn activity_zones(&self, id: i64) -> Result<Vec<ZoneRow>, BackendError>;

    /// The route of one run, as `[lat, lon]` pairs.
    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError>;

    /// Change the editable fields of a run.
    fn patch_activity(
        &self,
        id: i64,
        patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError>;

    /// The counts the health route reports.
    fn counts(&self) -> Result<Counts, BackendError>;

    /// Write a run the sync brought in, with its splits and its series.
    ///
    /// A backend that cannot be written to says so; the default is the
    /// answer, so that a new method does not break every implementor.
    fn upsert_synced_activity(
        &self,
        _activity: &ActivityRow,
        _laps: &[LapRow],
        _streams: &[StreamRow],
    ) -> Result<i64, BackendError> {
        Err(BackendError::Unsupported(
            "this backend cannot be written to".to_string(),
        ))
    }

    /// Write one day of physiology.
    fn upsert_physiology(&self, _row: &PhysiologyRow) -> Result<(), BackendError> {
        Err(BackendError::Unsupported(
            "this backend cannot be written to".to_string(),
        ))
    }

    /// Write one day of the daily-metric table.
    fn upsert_daily_metric(&self, _row: &DailyMetricRow) -> Result<(), BackendError> {
        Err(BackendError::Unsupported(
            "this backend cannot be written to".to_string(),
        ))
    }

    /// Write one run of the sync log.
    fn upsert_sync_run(&self, _run: &SyncRunRow) -> Result<i64, BackendError> {
        Err(BackendError::Unsupported(
            "this backend cannot be written to".to_string(),
        ))
    }

    /// The Garmin Personal API key, if there is one.
    ///
    /// A secret: it never reaches a response. The sync needs it and nothing
    /// else does, so it is a seam and not a route.
    fn garmin_personal_key(&self) -> Result<Option<String>, BackendError> {
        Ok(None)
    }

    // ----------------------------------------------------------- physiology

    /// The physiology rows of a date range, inclusive, oldest first.
    fn physiology(&self, from: &str, to: &str) -> Result<Vec<PhysiologyRow>, BackendError>;

    /// The most recent physiology row.
    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError>;

    /// The daily-metric rows of a date range.
    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyMetricRow>, BackendError>;

    // ----------------------------------------------------------------- plan

    /// The blocks, newest first.
    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError>;

    /// One block.
    fn plan_block(&self, id: i64) -> Result<Option<PlanBlockRow>, BackendError>;

    /// The phases of a block.
    fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError>;

    /// The weeks of a phase.
    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError>;

    /// The sessions of a week.
    fn plan_sessions(&self, week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError>;

    /// The sessions of a date range, oldest first.
    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError>;

    /// Create a block from the fields of the body.
    fn create_plan_block(&self, fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError>;

    /// Add a phase to a block.
    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError>;

    /// Add a week to a phase.
    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError>;

    /// Add a session to a week.
    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError>;

    /// Import a plan document as a new block.
    fn import_plan(&self, request: &ImportRequest) -> Result<Value, BackendError>;

    // -------------------------------------------------- derived (the core)

    /// The fingerprint of one run, already shaped as the API renders it.
    fn fingerprint(&self, activity_id: i64) -> Result<Value, BackendError>;

    /// The fingerprints of a date range.
    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError>;

    /// The readiness of one day.
    fn readiness(&self, day: &str) -> Result<Value, BackendError>;

    /// The readiness of a date range, including the days with no data.
    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError>;

    /// The baseline numbers of the trailing window.
    fn baselines(&self, day: &str) -> Result<Value, BackendError>;

    /// A comparison of two or more runs.
    fn compare(&self, ids: &[i64]) -> Result<Value, BackendError>;

    /// The runs that can be compared.
    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, BackendError>;

    /// The plan results of a date range.
    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError>;

    /// The adherence summary of a date range.
    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, BackendError>;

    /// The planned sessions of a date range, each with its result and the
    /// activity that fulfilled it.
    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError>;

    /// The athlete's heart-rate zone model, as the fitness surface uses it.
    fn athlete_zone_model(&self, activity_id: i64) -> Result<Vec<Value>, BackendError>;

    /// The maximum heart rate the athlete's history establishes.
    fn max_hr_reference(&self, activity_id: i64) -> Result<f64, BackendError>;

    // ------------------------------------------------------------- the state

    /// The state the browser is allowed to see.
    fn public_status(&self) -> Result<StatusView, BackendError>;

    /// The network route status of the machine.
    fn network_status(&self) -> Result<Value, BackendError>;

    /// The public identity of the machine, for the diagnostics panel.
    fn public_identity(&self) -> Result<Value, BackendError>;

    /// The exit nodes the tailnet offers.
    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError>;

    /// Change the network route.
    fn set_network_route(&self, route: &str) -> Result<Value, BackendError>;

    /// The state of the last sync.
    fn sync_status(&self) -> Result<Value, BackendError>;

    /// The sync log.
    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError>;

    /// Start a sync.
    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, BackendError>;

    /// Load a chosen span of history.
    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, BackendError>;

    /// Change the sync settings.
    fn update_settings(&self, patch: &SettingsPatch) -> Result<StatusView, BackendError>;

    // ----------------------------------------------------------------- misc

    /// The shoes.
    fn gear(&self) -> Result<Vec<Value>, BackendError>;

    /// Mint a pairing code.
    fn pair_code(&self) -> Result<Value, BackendError>;

    /// Whether this host may mint a pairing code for the caller that is
    /// asking.
    ///
    /// A pairing code is the operator's console secret: it is minted **on
    /// the machine that runs Stride** and typed into the other device. The
    /// host is the only layer that knows who is asking — it has the peer
    /// address and the `Host` header, and the router has neither — so the
    /// host answers the question.
    ///
    /// **The default is no.** A host that has not said otherwise does not
    /// mint a pairing code, because the alternative is handing a credential
    /// to whoever asks for it: the reference answers `403` to anybody who is
    /// not on the machine, and the security gate checks exactly that.
    ///
    /// A host that *can* tell a caller on the machine from a caller on the
    /// network — the desktop can, with the peer address and the `Host`
    /// header — says so and mints the code for the local caller only.
    fn may_mint_pairing_code(&self, _caller: &Caller) -> bool {
        false
    }

    /// Is this a live paired-device token?
    ///
    /// A paired device holds its own token, not the master capability, so
    /// revoking one phone does not lock the athlete out of their laptop.
    /// The state owns the device list and the hashes, so it is the state
    /// that answers.
    ///
    /// **The default is no.** A backend that has not said otherwise
    /// recognises no device token at all.
    fn device_token_is_live(&self, _token: &str) -> bool {
        false
    }

    /// Exchange a pairing code for a device credential.
    ///
    /// `None` means "no body": the desktop puts the credential in a cookie
    /// and answers 204, exactly as the Python route does. The phone has no
    /// cookie jar, so its implementation answers with the credential.
    fn pair(&self, code: &str) -> Result<Option<Value>, BackendError>;

    /// The paired devices.
    fn devices(&self) -> Result<Vec<Value>, BackendError>;

    /// Revoke a device.
    fn revoke_device(&self, device_id: &str) -> Result<Value, BackendError>;

    /// The raw provider payload of an activity, when it has one.
    ///
    /// The `fit_imported` marker lives in here: it says the activity has
    /// already been rebuilt from its original FIT file, so nothing about the
    /// file can have changed and it does not have to be fetched again.
    ///
    /// The default is `None`. A backend that keeps the payload elsewhere —
    /// the phone's preferences, for instance — has no marker to offer, and
    /// the sync falls back to the resolution of the series.
    fn activity_raw(&self, _id: i64) -> Result<Option<Value>, BackendError> {
        Ok(None)
    }

    /// Store a Garmin Personal API key after validating it.
    fn connect_personal_key(&self, api_key: &str) -> Result<Value, BackendError>;

    /// Validate a Garmin Personal API key without storing it.
    fn test_personal_key(&self, api_key: &str) -> Result<Value, BackendError>;

    /// Where the map tiles come from.
    fn tile_policy(&self) -> Result<Value, BackendError>;

    /// Fetch one map tile.
    fn tile(&self, z: i64, x: i64, y: i64) -> Result<Tile, BackendError>;

    /// One of the Garmin account flows the desktop supports: `developer_key`,
    /// `exchange`, `login`, `mfa` or `logout`.
    ///
    /// The phone build has no OAuth client and answers 501.
    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, BackendError>;

    /// Parse one FIT file and store it.
    ///
    /// The answer is the shape `api/auth.fit_import` puts in `activities`:
    /// `{"id", "name", "date", "distance_m", "created"}`.
    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, BackendError>;
}

/// Who is asking.
///
/// The router does not know the caller's address, and it does not know the
/// name the caller used: only the host does. So the host is the one that
/// answers "is this caller on this machine?", and it answers with what it
/// knows about the call.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Caller {
    /// The address the connection came from.
    pub peer: Option<std::net::IpAddr>,
    /// The `Host` header the request carried.
    pub host: Option<String>,
}

/// The filter the activity list applies.
#[derive(Debug, Clone, Default)]
pub struct ActivityFilter {
    pub sport: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub query: Option<String>,
    pub sort: String,
    pub page: i64,
    pub per_page: i64,
    /// Whether synthetic demo runs are visible.
    pub demo_visible: bool,
    /// Whether the filter should keep the sports that are not runs.
    ///
    /// The activity list shows runs; the statistics break the history down
    /// by sport, and a ride is part of that breakdown.
    pub all_sports: bool,
}

/// The fields of a new plan block.
#[derive(Debug, Clone, Default)]
pub struct PlanBlockFields {
    pub name: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub goal: Option<String>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// The fields of a new plan phase.
#[derive(Debug, Clone, Default)]
pub struct PlanPhaseFields {
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub target_weekly_km_min: Option<f64>,
    pub target_weekly_km_max: Option<f64>,
    pub target_sessions: Option<i64>,
    pub pace_easy_low: Option<f64>,
    pub pace_easy_high: Option<f64>,
    pub pace_tempo_low: Option<f64>,
    pub pace_tempo_high: Option<f64>,
    pub pace_interval_low: Option<f64>,
    pub pace_interval_high: Option<f64>,
    pub pace_long_low: Option<f64>,
    pub pace_long_high: Option<f64>,
    pub hr_zone_target: Option<i64>,
    pub notes: Option<String>,
}

/// The fields of a new plan week.
#[derive(Debug, Clone, Default)]
pub struct PlanWeekFields {
    pub week_number: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub target_km: Option<f64>,
    pub key_workout: Option<String>,
    pub notes: Option<String>,
}

/// The fields of a new plan session.
#[derive(Debug, Clone, Default)]
pub struct PlanSessionFields {
    pub date: String,
    pub session_type: String,
    pub title: Option<String>,
    pub details: Option<String>,
    pub target_distance_m: Option<f64>,
    pub pace_low_s_km: Option<f64>,
    pub pace_high_s_km: Option<f64>,
    pub hr_zone_target: Option<i64>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// A planned session, the run that fulfilled it and the verdict.
///
/// The plan engine decides the verdict; the router shapes the session and the
/// activity around it.
#[derive(Debug, Clone, Default)]
pub struct SessionWithResult {
    pub session: PlanSessionRow,
    pub activity: Option<ActivityRow>,
    /// The verdict, in the shape `api/plan._result` produces.
    pub result: Value,
}

/// The sports the analytics treat as runs.
///
/// The reference: `services.analytics.RUN_SPORTS`. Note that this list and
/// the one in `models.py` differ — the analytics list is the one the API uses.
pub const RUN_SPORTS: [&str; 6] = [
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
];

/// Whether a sport is a run for the analytics.
pub fn is_run_sport(sport: &str) -> bool {
    RUN_SPORTS.contains(&sport)
}
