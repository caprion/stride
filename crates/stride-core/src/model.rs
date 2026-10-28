//! The plain data the domain functions run over.
//!
//! These are the *shapes*, not the tables: the store (`stride-store`) reads
//! rows into them, the API (`stride-api`) writes them back out, and the domain
//! functions in this crate only ever look at them. Nothing here knows about
//! SQL.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ordered::OrderedMap;

/// The sports the reference's *model* module counts as running.
///
/// `backend/app/models.py` calls these `RUN_SPORTS`, and the activities API
/// uses that set. It includes walking and hiking.
///
/// **The plan engine does not use this set.** `backend/app/services/analytics.py`
/// defines a narrower one — see [`crate::plan::RUN_SPORTS`] — and the plan
/// engine matches against that. The two differ, and the difference decides
/// whether a walk answers a planned run.
pub const RUN_SPORTS: [&str; 8] = [
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
    "walking",
    "hiking",
];

/// Whether a sport type counts as running.
pub fn is_run_sport(sport_type: &str) -> bool {
    RUN_SPORTS.contains(&sport_type)
}

/// One lap of an activity, as the `activity_lap` table stores it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lap {
    /// The lap's position in the activity.
    #[serde(default)]
    pub lap_index: i64,
    /// The lap's total distance, in metres.
    #[serde(default)]
    pub distance_m: Option<f64>,
    /// The lap's *running* time, in seconds.
    #[serde(default)]
    pub duration_s: Option<f64>,
    /// The lap's average speed, in metres per second.
    #[serde(default)]
    pub avg_speed_ms: Option<f64>,
    /// The lap's maximum speed, in metres per second.
    #[serde(default)]
    pub max_speed_ms: Option<f64>,
    /// The lap's average heart rate, in beats per minute.
    #[serde(default)]
    pub avg_hr: Option<f64>,
    /// The lap's maximum heart rate, in beats per minute.
    #[serde(default)]
    pub max_hr: Option<f64>,
    /// The lap's average cadence, in steps per minute.
    #[serde(default)]
    pub avg_cadence: Option<f64>,
    /// The lap's maximum cadence, in steps per minute.
    #[serde(default)]
    pub max_cadence: Option<f64>,
    /// The lap's stride length, in metres.
    #[serde(default)]
    pub stride_length_m: Option<f64>,
    /// The elevation gained in the lap, in metres.
    #[serde(default)]
    pub elevation_gain_m: Option<f64>,
    /// The elevation lost in the lap, in metres.
    #[serde(default)]
    pub elevation_loss_m: Option<f64>,
    /// The elevation at the start of the lap, in metres.
    #[serde(default)]
    pub start_elevation_m: Option<f64>,
    /// The elevation at the end of the lap, in metres.
    #[serde(default)]
    pub end_elevation_m: Option<f64>,
    /// The distance covered while running, in metres.
    #[serde(default)]
    pub running_distance_m: Option<f64>,
    /// The lap's original cumulative elapsed span, in seconds.
    #[serde(default)]
    pub elapsed_s: Option<f64>,
    /// The lap's pace, in seconds per kilometre.
    ///
    /// Derived, not a table column: the importer computes it and the
    /// fingerprint recomputes it from the running duration and the distance.
    #[serde(default)]
    pub pace_s_per_km: Option<f64>,
}

/// One per-sample series of an activity, as the `activity_stream` table stores
/// it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    /// The sample's offset from the first sample, in seconds.
    #[serde(default)]
    pub seconds: Vec<f64>,
    /// The sample's value. `None` is an absent sample.
    #[serde(default)]
    pub values: Vec<Option<f64>>,
    /// The unit of the values, for display.
    #[serde(default)]
    pub unit: Option<String>,
}

impl Stream {
    /// The values that are actually present.
    pub fn present_values(&self) -> Vec<f64> {
        self.values.iter().filter_map(|value| *value).collect()
    }
}

/// The time spent in one heart-rate zone, as the `hr_zone` table stores it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneTime {
    /// The zone's number.
    #[serde(default)]
    pub zone_no: i64,
    /// The time spent in the zone, in seconds.
    #[serde(default)]
    pub time_s: f64,
}

/// One band of the heart-rate zone model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    /// The zone's number.
    #[serde(default)]
    pub zone_no: i64,
    /// The zone's name.
    #[serde(default)]
    pub name: String,
    /// The zone's lower bound, in beats per minute.
    #[serde(default)]
    pub low_bpm: f64,
    /// The zone's upper bound, in beats per minute. The last zone has none.
    #[serde(default)]
    pub high_bpm: Option<f64>,
}

/// One run: the summary of an activity, its laps, its zones and its streams.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// The activity's row id.
    #[serde(default)]
    pub id: i64,
    /// The athlete the run belongs to. `None` is the athlete-less row the
    /// importer writes.
    #[serde(default)]
    pub athlete_id: Option<i64>,
    /// The athlete's name for the run.
    #[serde(default)]
    pub name: String,
    /// The athlete-local date of the run, `YYYY-MM-DD`.
    #[serde(default)]
    pub local_date: String,
    /// The UTC date of the start, `YYYY-MM-DD`.
    ///
    /// The reference breaks the same-date tie on `start_time.date()`, which is
    /// the UTC date, not the athlete-local one.
    #[serde(default)]
    pub start_date: String,
    /// The weekday name of the run.
    #[serde(default)]
    pub day_of_week: Option<String>,
    /// The normalised sport type, for example `running`.
    #[serde(default)]
    pub sport_type: String,
    /// Where the run came from: `garmin_fit`, `garmin`, `demo`, …
    #[serde(default)]
    pub source: String,
    /// The total distance, in metres.
    #[serde(default)]
    pub distance_m: f64,
    /// The moving time, in seconds.
    #[serde(default)]
    pub moving_time_s: Option<f64>,
    /// The elapsed time, in seconds.
    #[serde(default)]
    pub elapsed_time_s: Option<f64>,
    /// The elevation gain, in metres.
    #[serde(default)]
    pub elevation_gain_m: Option<f64>,
    /// The average heart rate, in beats per minute.
    #[serde(default)]
    pub avg_hr: Option<f64>,
    /// The maximum heart rate, in beats per minute.
    #[serde(default)]
    pub max_hr: Option<f64>,
    /// The calories burned.
    #[serde(default)]
    pub calories: Option<f64>,
    /// The time spent running, in seconds.
    #[serde(default)]
    pub running_time_s: Option<f64>,
    /// The time spent walking, in seconds.
    #[serde(default)]
    pub walking_time_s: Option<f64>,
    /// The time spent standing still, in seconds.
    #[serde(default)]
    pub stopped_time_s: Option<f64>,
    /// The laps, in the order they were recorded.
    #[serde(default)]
    pub laps: Vec<Lap>,
    /// The time spent in each heart-rate zone.
    #[serde(default)]
    pub zones: Vec<ZoneTime>,
    /// The per-sample series, keyed by kind: `hr`, `cadence`, `pace`,
    /// `distance`, `elevation`.
    #[serde(default)]
    pub streams: BTreeMap<String, Stream>,
    /// The run's fingerprint, when it has been computed.
    #[serde(default)]
    pub fingerprint: Option<Fingerprint>,
}

impl Activity {
    /// The UTC date of the start, falling back to the athlete-local date.
    pub fn start_date(&self) -> &str {
        if self.start_date.is_empty() {
            &self.local_date
        } else {
            &self.start_date
        }
    }

    /// The average pace, in seconds per kilometre.
    ///
    /// This is the plan engine's `_pace`: a zero distance or a zero moving
    /// time means there is no pace.
    pub fn pace_s_per_km(&self) -> Option<f64> {
        let moving = self.moving_time_s?;
        if self.distance_m == 0.0 || moving == 0.0 {
            return None;
        }
        Some(moving / (self.distance_m / 1000.0))
    }
}

/// One day of physiology: sleep, HRV, resting heart rate, stress, load.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DailyPhysiology {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The sleep duration, in seconds.
    #[serde(default)]
    pub sleep_s: Option<f64>,
    /// The Garmin sleep score, out of 100.
    #[serde(default)]
    pub sleep_score: Option<f64>,
    /// The deep sleep duration, in seconds.
    #[serde(default)]
    pub deep_sleep_s: Option<f64>,
    /// The REM sleep duration, in seconds.
    #[serde(default)]
    pub rem_s: Option<f64>,
    /// The light sleep duration, in seconds.
    #[serde(default)]
    pub light_sleep_s: Option<f64>,
    /// The awake duration, in seconds.
    #[serde(default)]
    pub awake_s: Option<f64>,
    /// The heart-rate variability, in milliseconds.
    #[serde(default)]
    pub hrv_ms: Option<f64>,
    /// The Garmin HRV status.
    #[serde(default)]
    pub hrv_status: Option<String>,
    /// The resting heart rate, in beats per minute.
    #[serde(default)]
    pub resting_hr: Option<f64>,
    /// The maximum heart rate of the day, in beats per minute.
    #[serde(default)]
    pub max_hr: Option<f64>,
    /// The average stress of the day.
    #[serde(default)]
    pub stress_avg: Option<f64>,
    /// The maximum stress of the day.
    #[serde(default)]
    pub stress_max: Option<f64>,
    /// The lowest body battery of the day.
    #[serde(default)]
    pub body_battery_min: Option<f64>,
    /// The average body battery of the day.
    #[serde(default)]
    pub body_battery_avg: Option<f64>,
    /// The highest body battery of the day.
    #[serde(default)]
    pub body_battery_max: Option<f64>,
    /// The step count.
    #[serde(default)]
    pub steps: Option<f64>,
    /// Garmin's own training readiness.
    #[serde(default)]
    pub training_readiness: Option<f64>,
    /// The acute training load.
    #[serde(default)]
    pub acute_load: Option<f64>,
    /// The chronic training load.
    #[serde(default)]
    pub chronic_load: Option<f64>,
    /// The VO₂ max.
    #[serde(default)]
    pub vo2max: Option<f64>,
    /// The endurance score.
    #[serde(default)]
    pub endurance_score: Option<f64>,
    /// The running tolerance.
    #[serde(default)]
    pub running_tolerance: Option<f64>,
    /// Where the day's data came from.
    #[serde(default)]
    pub source: Option<String>,
}

/// One planned session.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSession {
    /// The session's row id.
    pub id: i64,
    /// The day the session is planned for, `YYYY-MM-DD`.
    pub date: String,
    /// The session's type: `easy`, `tempo`, `interval`, `long`, `race`,
    /// `recovery`, `strength`, `rest`.
    pub session_type: String,
    /// The session's title.
    #[serde(default)]
    pub title: Option<String>,
    /// The session's free-text details.
    #[serde(default)]
    pub details: Option<String>,
    /// The target distance, in metres.
    #[serde(default)]
    pub target_distance_m: Option<f64>,
    /// The lower bound of the target pace band, in seconds per kilometre.
    #[serde(default)]
    pub pace_low_s_km: Option<f64>,
    /// The upper bound of the target pace band, in seconds per kilometre.
    #[serde(default)]
    pub pace_high_s_km: Option<f64>,
}

/// The activity a session was matched to, as the plan engine reports it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchedActivity {
    /// The activity's row id.
    pub id: i64,
    /// The athlete-local date of the run.
    pub date: String,
    /// The athlete's name for the run.
    pub name: String,
    /// The normalised sport type.
    pub sport_type: String,
    /// The distance, in metres.
    #[serde(default)]
    pub distance_m: Option<f64>,
    /// The average pace, in seconds per kilometre.
    #[serde(default)]
    pub pace_s_km: Option<f64>,
}

/// The derived mechanics of one run.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fingerprint {
    /// The mean of the laps' average cadence, in steps per minute.
    #[serde(default)]
    pub cadence_avg: Option<f64>,
    /// The coefficient of variation of the laps' average cadence, in percent.
    #[serde(default)]
    pub cadence_cv: Option<f64>,
    /// The cadence drift between the last and the first third, in percent.
    #[serde(default)]
    pub cadence_drift_pct: Option<f64>,
    /// The average heart rate, in beats per minute.
    #[serde(default)]
    pub hr_avg: Option<f64>,
    /// The maximum heart rate, in beats per minute.
    #[serde(default)]
    pub hr_max: Option<f64>,
    /// The heart-rate drift between the last and the first third, in percent.
    #[serde(default)]
    pub hr_drift_pct: Option<f64>,
    /// The efficiency factor of the first third.
    #[serde(default)]
    pub efficiency_first: Option<f64>,
    /// The efficiency factor of the last third.
    #[serde(default)]
    pub efficiency_last: Option<f64>,
    /// The aerobic decoupling, in percent.
    #[serde(default)]
    pub decoupling_pct: Option<f64>,
    /// The coefficient of variation of the laps' pace, in percent.
    #[serde(default)]
    pub pace_cv: Option<f64>,
    /// 100 when the last third was faster than the first, otherwise 0.
    #[serde(default)]
    pub negative_split_pct: Option<f64>,
    /// The mean stride length, in centimetres.
    #[serde(default)]
    pub stride_length_cm: Option<f64>,
    /// The form score, out of 100.
    #[serde(default)]
    pub form_score: Option<f64>,
    /// The seconds spent in each heart-rate zone.
    #[serde(default)]
    pub zone_distribution: Option<OrderedMap<f64>>,
    /// The time spent running, in seconds.
    #[serde(default)]
    pub running_time_s: Option<f64>,
    /// The time spent walking, in seconds.
    #[serde(default)]
    pub walking_time_s: Option<f64>,
    /// The time spent standing still, in seconds.
    #[serde(default)]
    pub stopped_time_s: Option<f64>,
    /// The heart-rate drift between the two halves, in beats per minute.
    #[serde(default)]
    pub hr_drift_bpm: Option<f64>,
    /// The pace drift between the two halves, in percent.
    #[serde(default)]
    pub pace_drift_pct: Option<f64>,
    /// The mean heart rate of the first half.
    #[serde(default)]
    pub first_half_hr: Option<f64>,
    /// The mean heart rate of the second half.
    #[serde(default)]
    pub second_half_hr: Option<f64>,
    /// The mean pace of the first half, in seconds per kilometre.
    #[serde(default)]
    pub first_half_pace_s_km: Option<f64>,
    /// The mean pace of the second half, in seconds per kilometre.
    #[serde(default)]
    pub second_half_pace_s_km: Option<f64>,
    /// A note, for demo activities.
    #[serde(default)]
    pub notes: Option<String>,
}
