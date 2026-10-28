//! The rows.
//!
//! One struct per table, one field per column, in the column order of the
//! table. The names are the column names, the types are the column types, and
//! `Option<T>` means "NULL in the column".
//!
//! The defaults mirror the Python-side defaults of the models
//! (`backend/app/models.py`). SQLAlchemy's `default=` is a Python-side
//! default — it never reaches the DDL — so the defaults live here, in the
//! rows, and not in the schema.
//!
//! These are the types the API layer consumes and produces, so they derive
//! `serde::Serialize` / `serde::Deserialize`: the JSON the router emits is
//! built from these structs, not from ad-hoc maps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The athlete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Athlete {
    /// Primary key.
    pub id: Option<i64>,
    /// The athlete's name.
    pub name: String,
    /// The athlete's email, if known.
    pub email: Option<String>,
    /// The athlete's timezone, e.g. `Asia/Kolkata`.
    pub timezone: String,
    /// When the row was created.
    pub created_at: DateTime<Utc>,
}

impl Default for Athlete {
    fn default() -> Self {
        Athlete {
            id: None,
            name: "Athlete".to_string(),
            email: None,
            timezone: "Asia/Kolkata".to_string(),
            created_at: Utc::now(),
        }
    }
}

/// One run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// Primary key.
    pub id: Option<i64>,
    /// The athlete it belongs to, if any.
    pub athlete_id: Option<i64>,
    /// Where it came from: `garmin`, `garmin_fit`, `garmin_account`, …
    pub source: String,
    /// The provider's identifier for it.
    pub external_id: String,
    /// Its name.
    pub name: String,
    /// The sport: `running`, `trail_running`, `cycling`, …
    pub sport_type: String,
    /// Its description, if any.
    pub description: Option<String>,
    /// When it started (UTC).
    pub start_time: DateTime<Utc>,
    /// The athlete-local date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The athlete-local day of the week.
    pub day_of_week: Option<String>,
    /// Distance in metres.
    pub distance_m: f64,
    /// Moving time in seconds.
    pub moving_time_s: f64,
    /// Elapsed time in seconds.
    pub elapsed_time_s: f64,
    /// Elevation gain in metres.
    pub elevation_gain_m: f64,
    /// Average heart rate.
    pub avg_hr: Option<f64>,
    /// Maximum heart rate.
    pub max_hr: Option<f64>,
    /// Calories.
    pub calories: Option<f64>,
    /// Average cadence, in steps per minute.
    pub avg_cadence: Option<f64>,
    /// Maximum cadence, in steps per minute.
    pub max_cadence: Option<f64>,
    /// Average speed in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// Maximum speed in metres per second.
    pub max_speed_ms: Option<f64>,
    /// Seconds spent running.
    pub running_time_s: Option<f64>,
    /// Seconds spent walking.
    pub walking_time_s: Option<f64>,
    /// Seconds spent stopped.
    pub stopped_time_s: Option<f64>,
    /// Distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// Perceived exertion, 1–10.
    pub perceived_exertion: Option<i64>,
    /// The device that recorded it.
    pub device: Option<String>,
    /// The shoes it was run in.
    pub gear: Option<String>,
    /// Whether it was a trail run.
    pub is_trail: bool,
    /// Whether it was a commute.
    pub is_commute: bool,
    /// The encoded polyline.
    pub polyline: Option<Value>,
    /// The decoded points, as `[lat, lon]` pairs.
    pub geo_points: Option<Value>,
    /// The raw provider payload.
    pub raw: Option<Value>,
    /// When the row was created.
    pub created_at: DateTime<Utc>,
    /// When the row was last written.
    pub updated_at: DateTime<Utc>,
}

impl Default for Activity {
    fn default() -> Self {
        let now = Utc::now();
        Activity {
            id: None,
            athlete_id: None,
            source: "garmin".to_string(),
            external_id: String::new(),
            name: "Run".to_string(),
            sport_type: "running".to_string(),
            description: None,
            start_time: now,
            local_date: now.format("%Y-%m-%d").to_string(),
            day_of_week: None,
            distance_m: 0.0,
            moving_time_s: 0.0,
            elapsed_time_s: 0.0,
            elevation_gain_m: 0.0,
            avg_hr: None,
            max_hr: None,
            calories: None,
            avg_cadence: None,
            max_cadence: None,
            avg_speed_ms: None,
            max_speed_ms: None,
            running_time_s: None,
            walking_time_s: None,
            stopped_time_s: None,
            running_distance_m: None,
            perceived_exertion: None,
            device: None,
            gear: None,
            is_trail: false,
            is_commute: false,
            polyline: None,
            geo_points: None,
            raw: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// One lap of an activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityLap {
    /// Primary key.
    pub id: Option<i64>,
    /// The activity it belongs to.
    pub activity_id: i64,
    /// The lap number, from 1.
    pub lap_index: i64,
    /// Distance in metres.
    pub distance_m: f64,
    /// Duration in seconds.
    pub duration_s: f64,
    /// Average speed in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// Maximum speed in metres per second.
    pub max_speed_ms: Option<f64>,
    /// Average heart rate.
    pub avg_hr: Option<f64>,
    /// Maximum heart rate.
    pub max_hr: Option<f64>,
    /// Average cadence.
    pub avg_cadence: Option<f64>,
    /// Maximum cadence.
    pub max_cadence: Option<f64>,
    /// Stride length in metres.
    pub stride_length_m: Option<f64>,
    /// Elevation gain in metres.
    pub elevation_gain_m: Option<f64>,
    /// Elevation loss in metres.
    pub elevation_loss_m: Option<f64>,
    /// Elevation at the start of the lap, in metres.
    pub start_elevation_m: Option<f64>,
    /// Elevation at the end of the lap, in metres.
    pub end_elevation_m: Option<f64>,
    /// Distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// Elapsed span of the lap, in seconds.
    pub elapsed_s: Option<f64>,
}

impl Default for ActivityLap {
    fn default() -> Self {
        ActivityLap {
            id: None,
            activity_id: 0,
            lap_index: 1,
            distance_m: 0.0,
            duration_s: 0.0,
            avg_speed_ms: None,
            max_speed_ms: None,
            avg_hr: None,
            max_hr: None,
            avg_cadence: None,
            max_cadence: None,
            stride_length_m: None,
            elevation_gain_m: None,
            elevation_loss_m: None,
            start_elevation_m: None,
            end_elevation_m: None,
            running_distance_m: None,
            elapsed_s: None,
        }
    }
}

/// One per-second series of an activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityStream {
    /// Primary key.
    pub id: Option<i64>,
    /// The activity it belongs to.
    pub activity_id: i64,
    /// The series: `hr`, `pace`, `elevation`, `distance`, `cadence`.
    pub kind: String,
    /// The sample times, in seconds from the start.
    pub seconds: Value,
    /// The sample values.
    pub values: Value,
    /// A label for the UI.
    pub label: Option<String>,
    /// The unit of the values.
    pub unit: Option<String>,
}

impl Default for ActivityStream {
    fn default() -> Self {
        ActivityStream {
            id: None,
            activity_id: 0,
            kind: String::new(),
            seconds: Value::Array(Vec::new()),
            values: Value::Array(Vec::new()),
            label: None,
            unit: None,
        }
    }
}

/// The time spent in one heart-rate zone of an activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HrZone {
    /// Primary key.
    pub id: Option<i64>,
    /// The activity it belongs to.
    pub activity_id: i64,
    /// The zone number, from 1.
    pub zone_no: i64,
    /// The zone name.
    pub name: String,
    /// The lower bound in beats per minute.
    pub low_bpm: Option<f64>,
    /// The upper bound in beats per minute.
    pub high_bpm: Option<f64>,
    /// The time in the zone, in seconds.
    pub time_s: f64,
}

impl Default for HrZone {
    fn default() -> Self {
        HrZone {
            id: None,
            activity_id: 0,
            zone_no: 1,
            name: "Zone 1".to_string(),
            low_bpm: None,
            high_bpm: None,
            time_s: 0.0,
        }
    }
}

/// One day of the coarse daily metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyMetric {
    /// The day, `YYYY-MM-DD`. Primary key.
    pub date: String,
    /// Resting heart rate.
    pub resting_hr: Option<f64>,
    /// Maximum heart rate.
    pub max_hr: Option<f64>,
    /// Sleep in seconds.
    pub sleep_s: Option<f64>,
    /// Deep sleep in seconds.
    pub deep_sleep_s: Option<f64>,
    /// Heart-rate variability in milliseconds.
    pub hrv_ms: Option<f64>,
    /// Average stress.
    pub stress: Option<f64>,
    /// Steps.
    pub steps: Option<f64>,
    /// Body battery.
    pub body_battery: Option<f64>,
    /// Readiness.
    pub readiness: Option<f64>,
    /// Distance run that day, in metres.
    pub distance_m: f64,
}

impl Default for DailyMetric {
    fn default() -> Self {
        DailyMetric {
            date: String::new(),
            resting_hr: None,
            max_hr: None,
            sleep_s: None,
            deep_sleep_s: None,
            hrv_ms: None,
            stress: None,
            steps: None,
            body_battery: None,
            readiness: None,
            distance_m: 0.0,
        }
    }
}

/// One day of physiology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyPhysiology {
    /// The day, `YYYY-MM-DD`. Primary key.
    pub date: String,
    /// Sleep in seconds.
    pub sleep_s: Option<f64>,
    /// The sleep score.
    pub sleep_score: Option<f64>,
    /// Deep sleep in seconds.
    pub deep_sleep_s: Option<f64>,
    /// REM sleep in seconds.
    pub rem_s: Option<f64>,
    /// Light sleep in seconds.
    pub light_sleep_s: Option<f64>,
    /// Awake time in seconds.
    pub awake_s: Option<f64>,
    /// Heart-rate variability in milliseconds.
    pub hrv_ms: Option<f64>,
    /// The HRV status: `balanced`, `unbalanced`, …
    pub hrv_status: Option<String>,
    /// Resting heart rate.
    pub resting_hr: Option<f64>,
    /// Maximum heart rate.
    pub max_hr: Option<f64>,
    /// Average stress.
    pub stress_avg: Option<f64>,
    /// Maximum stress.
    pub stress_max: Option<f64>,
    /// The lowest body battery.
    pub body_battery_min: Option<f64>,
    /// The average body battery.
    pub body_battery_avg: Option<f64>,
    /// The highest body battery.
    pub body_battery_max: Option<f64>,
    /// Steps.
    pub steps: Option<f64>,
    /// Garmin's training readiness.
    pub training_readiness: Option<f64>,
    /// The acute load.
    pub acute_load: Option<f64>,
    /// The chronic load.
    pub chronic_load: Option<f64>,
    /// VO2 max.
    pub vo2max: Option<f64>,
    /// The endurance score.
    pub endurance_score: Option<f64>,
    /// The running tolerance.
    pub running_tolerance: Option<f64>,
    /// Where the row came from.
    pub source: String,
    /// When the row was last written.
    pub updated_at: DateTime<Utc>,
}

impl Default for DailyPhysiology {
    fn default() -> Self {
        DailyPhysiology {
            date: String::new(),
            sleep_s: None,
            sleep_score: None,
            deep_sleep_s: None,
            rem_s: None,
            light_sleep_s: None,
            awake_s: None,
            hrv_ms: None,
            hrv_status: None,
            resting_hr: None,
            max_hr: None,
            stress_avg: None,
            stress_max: None,
            body_battery_min: None,
            body_battery_avg: None,
            body_battery_max: None,
            steps: None,
            training_readiness: None,
            acute_load: None,
            chronic_load: None,
            vo2max: None,
            endurance_score: None,
            running_tolerance: None,
            source: "garmin".to_string(),
            updated_at: Utc::now(),
        }
    }
}

/// The extra per-activity detail only Garmin knows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityFact {
    /// The activity it belongs to. Primary key.
    pub activity_id: i64,
    /// Aerobic training effect.
    pub aerobic_te: Option<f64>,
    /// Anaerobic training effect.
    pub anaerobic_te: Option<f64>,
    /// The training effect label.
    pub training_effect_label: Option<String>,
    /// VO2 max.
    pub vo2max: Option<f64>,
    /// Recovery time in minutes.
    pub recovery_time_min: Option<f64>,
    /// Average ground contact time in milliseconds.
    pub avg_gct_ms: Option<f64>,
    /// Average stride ratio.
    pub avg_stride_ratio: Option<f64>,
    /// Vertical ratio.
    pub vertical_ratio: Option<f64>,
    /// Average power.
    pub avg_power: Option<f64>,
    /// Normalised power.
    pub normalized_power: Option<f64>,
    /// Average grade-adjusted pace.
    pub avg_grade_adjusted_pace: Option<f64>,
    /// Temperature in degrees Celsius.
    pub temperature_c: Option<f64>,
    /// The impact estimate.
    pub impact_estimate: Option<f64>,
    /// The gear's UUID.
    pub gear_uuid: Option<String>,
    /// The device name.
    pub device_name: Option<String>,
}

/// The derived mechanics of one activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFingerprint {
    /// The activity it belongs to. Primary key.
    pub activity_id: i64,
    /// Seconds spent running.
    pub running_time_s: Option<f64>,
    /// Seconds spent walking.
    pub walking_time_s: Option<f64>,
    /// Seconds spent stopped.
    pub stopped_time_s: Option<f64>,
    /// Average cadence.
    pub cadence_avg: Option<f64>,
    /// The coefficient of variation of the cadence.
    pub cadence_cv: Option<f64>,
    /// The cadence drift, in per cent.
    pub cadence_drift_pct: Option<f64>,
    /// Average heart rate.
    pub hr_avg: Option<f64>,
    /// Maximum heart rate.
    pub hr_max: Option<f64>,
    /// The heart-rate drift, in per cent.
    pub hr_drift_pct: Option<f64>,
    /// The heart-rate drift, in beats per minute.
    pub hr_drift_bpm: Option<f64>,
    /// The pace drift, in per cent.
    pub pace_drift_pct: Option<f64>,
    /// The average heart rate of the first half.
    pub first_half_hr: Option<f64>,
    /// The average heart rate of the second half.
    pub second_half_hr: Option<f64>,
    /// The average pace of the first half, in seconds per kilometre.
    pub first_half_pace_s_km: Option<f64>,
    /// The average pace of the second half, in seconds per kilometre.
    pub second_half_pace_s_km: Option<f64>,
    /// The efficiency of the first half.
    pub efficiency_first: Option<f64>,
    /// The efficiency of the last part.
    pub efficiency_last: Option<f64>,
    /// The decoupling, in per cent.
    pub decoupling_pct: Option<f64>,
    /// The coefficient of variation of the pace.
    pub pace_cv: Option<f64>,
    /// The negative split, in per cent.
    pub negative_split_pct: Option<f64>,
    /// The stride length, in centimetres.
    pub stride_length_cm: Option<f64>,
    /// The form score.
    pub form_score: Option<f64>,
    /// The time in each heart-rate zone.
    pub zone_distribution: Option<Value>,
    /// Free-text notes.
    pub notes: Option<String>,
    /// When the fingerprint was computed.
    pub computed_at: DateTime<Utc>,
}

impl Default for RunFingerprint {
    fn default() -> Self {
        RunFingerprint {
            activity_id: 0,
            running_time_s: None,
            walking_time_s: None,
            stopped_time_s: None,
            cadence_avg: None,
            cadence_cv: None,
            cadence_drift_pct: None,
            hr_avg: None,
            hr_max: None,
            hr_drift_pct: None,
            hr_drift_bpm: None,
            pace_drift_pct: None,
            first_half_hr: None,
            second_half_hr: None,
            first_half_pace_s_km: None,
            second_half_pace_s_km: None,
            efficiency_first: None,
            efficiency_last: None,
            decoupling_pct: None,
            pace_cv: None,
            negative_split_pct: None,
            stride_length_cm: None,
            form_score: None,
            zone_distribution: None,
            notes: None,
            computed_at: Utc::now(),
        }
    }
}

/// The readiness score of one day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Readiness {
    /// The day, `YYYY-MM-DD`. Primary key.
    pub date: String,
    /// The score.
    pub score: f64,
    /// The grade: `peak`, `good`, `fair`, `low`.
    pub grade: String,
    /// What drove the score.
    pub drivers: Option<Value>,
    /// What to do about it.
    pub recommendation: Option<String>,
    /// Why the score is what it is.
    pub rationale: Option<String>,
    /// When the score was computed.
    pub computed_at: DateTime<Utc>,
}

impl Default for Readiness {
    fn default() -> Self {
        Readiness {
            date: String::new(),
            score: 0.0,
            grade: "unknown".to_string(),
            drivers: None,
            recommendation: None,
            rationale: None,
            computed_at: Utc::now(),
        }
    }
}

/// One pair of shoes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gear {
    /// Primary key.
    pub id: Option<i64>,
    /// The provider's UUID for the shoes.
    pub uuid: String,
    /// Their name.
    pub name: String,
    /// Their brand.
    pub brand: Option<String>,
    /// Their model.
    pub model: Option<String>,
    /// `active` or `retired`.
    pub status: String,
    /// The distance run in them, in metres.
    pub distance_m: f64,
    /// How many activities they have been used in.
    pub activity_count: i64,
    /// The first day they were used, `YYYY-MM-DD`.
    pub first_used: Option<String>,
    /// The last day they were used, `YYYY-MM-DD`.
    pub last_used: Option<String>,
    /// The distance at which they retire, in metres.
    pub retire_at_m: f64,
}

impl Default for Gear {
    fn default() -> Self {
        Gear {
            id: None,
            uuid: String::new(),
            name: "Shoes".to_string(),
            brand: None,
            model: None,
            status: "active".to_string(),
            distance_m: 0.0,
            activity_count: 0,
            first_used: None,
            last_used: None,
            retire_at_m: 800_000.0,
        }
    }
}

/// A training cycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanBlock {
    /// Primary key.
    pub id: Option<i64>,
    /// The block's name.
    pub name: String,
    /// The first day, `YYYY-MM-DD`.
    pub start_date: String,
    /// The last day, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// The goal.
    pub goal: Option<String>,
    /// `active`, `completed` or `archived`.
    pub status: String,
    /// `manual`, `imported` or `garmin`.
    pub source: String,
    /// Free-text notes.
    pub notes: Option<String>,
    /// When the row was created.
    pub created_at: DateTime<Utc>,
}

impl Default for PlanBlock {
    fn default() -> Self {
        PlanBlock {
            id: None,
            name: "Training block".to_string(),
            start_date: String::new(),
            end_date: None,
            goal: None,
            status: "active".to_string(),
            source: "manual".to_string(),
            notes: None,
            created_at: Utc::now(),
        }
    }
}

/// A phase of a training block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanPhase {
    /// Primary key.
    pub id: Option<i64>,
    /// The block it belongs to.
    pub block_id: i64,
    /// The phase's name.
    pub name: String,
    /// The first day, `YYYY-MM-DD`.
    pub start_date: Option<String>,
    /// The last day, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// The target weekly volume, lower bound, in kilometres.
    pub target_weekly_km_min: Option<f64>,
    /// The target weekly volume, upper bound, in kilometres.
    pub target_weekly_km_max: Option<f64>,
    /// The target number of sessions.
    pub target_sessions: Option<i64>,
    /// The easy pace band, lower bound, in seconds per kilometre.
    pub pace_easy_low: Option<f64>,
    /// The easy pace band, upper bound, in seconds per kilometre.
    pub pace_easy_high: Option<f64>,
    /// The tempo pace band, lower bound, in seconds per kilometre.
    pub pace_tempo_low: Option<f64>,
    /// The tempo pace band, upper bound, in seconds per kilometre.
    pub pace_tempo_high: Option<f64>,
    /// The interval pace band, lower bound, in seconds per kilometre.
    pub pace_interval_low: Option<f64>,
    /// The interval pace band, upper bound, in seconds per kilometre.
    pub pace_interval_high: Option<f64>,
    /// The long-run pace band, lower bound, in seconds per kilometre.
    pub pace_long_low: Option<f64>,
    /// The long-run pace band, upper bound, in seconds per kilometre.
    pub pace_long_high: Option<f64>,
    /// The target heart-rate zone.
    pub hr_zone_target: Option<i64>,
    /// Free-text notes.
    pub notes: Option<String>,
}

impl Default for PlanPhase {
    fn default() -> Self {
        PlanPhase {
            id: None,
            block_id: 0,
            name: "Phase".to_string(),
            start_date: None,
            end_date: None,
            target_weekly_km_min: None,
            target_weekly_km_max: None,
            target_sessions: None,
            pace_easy_low: None,
            pace_easy_high: None,
            pace_tempo_low: None,
            pace_tempo_high: None,
            pace_interval_low: None,
            pace_interval_high: None,
            pace_long_low: None,
            pace_long_high: None,
            hr_zone_target: None,
            notes: None,
        }
    }
}

/// A week of a phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanWeek {
    /// Primary key.
    pub id: Option<i64>,
    /// The phase it belongs to.
    pub phase_id: i64,
    /// The week number, from 1.
    pub week_number: i64,
    /// The first day, `YYYY-MM-DD`.
    pub start_date: Option<String>,
    /// The last day, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// The target distance, in kilometres.
    pub target_km: Option<f64>,
    /// The key workout of the week.
    pub key_workout: Option<String>,
    /// Free-text notes.
    pub notes: Option<String>,
}

impl Default for PlanWeek {
    fn default() -> Self {
        PlanWeek {
            id: None,
            phase_id: 0,
            week_number: 1,
            start_date: None,
            end_date: None,
            target_km: None,
            key_workout: None,
            notes: None,
        }
    }
}

/// A planned session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanSession {
    /// Primary key.
    pub id: Option<i64>,
    /// The week it belongs to.
    pub week_id: Option<i64>,
    /// The block it belongs to.
    pub block_id: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The session type: `easy`, `long`, `tempo`, …
    pub session_type: String,
    /// The title.
    pub title: Option<String>,
    /// The details.
    pub details: Option<String>,
    /// The target distance, in metres.
    pub target_distance_m: Option<f64>,
    /// The pace band, lower bound, in seconds per kilometre.
    pub pace_low_s_km: Option<f64>,
    /// The pace band, upper bound, in seconds per kilometre.
    pub pace_high_s_km: Option<f64>,
    /// The target heart-rate zone.
    pub hr_zone_target: Option<i64>,
    /// `planned`, `done`, `skipped`, …
    pub status: String,
    /// `manual`, `imported` or `garmin`.
    pub source: String,
    /// Free-text notes.
    pub notes: Option<String>,
}

impl Default for PlanSession {
    fn default() -> Self {
        PlanSession {
            id: None,
            week_id: None,
            block_id: None,
            date: String::new(),
            session_type: "easy".to_string(),
            title: None,
            details: None,
            target_distance_m: None,
            pace_low_s_km: None,
            pace_high_s_km: None,
            hr_zone_target: None,
            status: "planned".to_string(),
            source: "manual".to_string(),
            notes: None,
        }
    }
}

/// How a planned session compared with what was actually done.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanResult {
    /// Primary key.
    pub id: Option<i64>,
    /// The session it scores.
    pub plan_session_id: i64,
    /// The activity that satisfied it, if any.
    pub activity_id: Option<i64>,
    /// The verdict: `hit`, `missed_pace`, `short`, `missed`, …
    pub verdict: String,
    /// How far off the distance was, in metres.
    pub distance_delta_m: Option<f64>,
    /// How far off the pace was, in seconds per kilometre.
    pub pace_delta_s_km: Option<f64>,
    /// Whether the pace band was hit.
    pub pace_band_hit: Option<bool>,
    /// Whether the distance was hit.
    pub distance_hit: Option<bool>,
    /// The adherence score.
    pub adherence_score: Option<f64>,
    /// Why the session went the way it did.
    pub cause: Option<String>,
    /// How sure we are of the cause.
    pub cause_confidence: Option<f64>,
    /// A note.
    pub note: Option<String>,
    /// When the verdict was computed.
    pub computed_at: DateTime<Utc>,
}

impl Default for PlanResult {
    fn default() -> Self {
        PlanResult {
            id: None,
            plan_session_id: 0,
            activity_id: None,
            verdict: "missed".to_string(),
            distance_delta_m: None,
            pace_delta_s_km: None,
            pace_band_hit: None,
            distance_hit: None,
            adherence_score: None,
            cause: None,
            cause_confidence: None,
            note: None,
            computed_at: Utc::now(),
        }
    }
}

/// One run of the sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncRun {
    /// Primary key.
    pub id: Option<i64>,
    /// The source that was synced.
    pub source: String,
    /// When the sync started.
    pub started_at: DateTime<Utc>,
    /// When the sync finished.
    pub finished_at: Option<DateTime<Utc>>,
    /// `running`, `ok` or `error`.
    pub status: String,
    /// How many days back the sync went.
    pub days: i64,
    /// How many activities the provider returned.
    pub activities_found: i64,
    /// How many activities were new.
    pub activities_new: i64,
    /// How many activities were updated.
    pub activities_updated: i64,
    /// A message about the run.
    pub message: Option<String>,
    /// The error, if it failed.
    pub error: Option<String>,
}

impl Default for SyncRun {
    fn default() -> Self {
        SyncRun {
            id: None,
            source: "garmin".to_string(),
            started_at: Utc::now(),
            finished_at: None,
            status: "running".to_string(),
            days: 90,
            activities_found: 0,
            activities_new: 0,
            activities_updated: 0,
            message: None,
            error: None,
        }
    }
}
