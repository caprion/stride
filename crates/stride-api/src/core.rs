//! The core-backed backend.
//!
//! `stride-core` owns the domain: the fingerprint, the readiness, the plan
//! verdicts and the comparison. This file puts them behind the same seam the
//! router reads, so the router never has to know whether the number came
//! from a database or from a pure function.
//!
//! It is a decorator: it wraps any [`Backend`] and answers the derived
//! methods, delegating the rest. `stride-api` therefore depends on
//! `stride-core` and `stride-store`, and neither of them depends on it.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{json, Value};

use stride_core::{FingerprintInput, PlanInputs};

use crate::backend::{
    ActivityRow, Backend, LapRow, PhysiologyRow, PlanSessionRow, SessionWithResult, StreamRow,
    ZoneRow,
};
use crate::error::BackendError;
use crate::num::{duration_label, pace_s_per_km, round_to};

/// The number of days the baselines look back over.
const BASELINE_WINDOW_DAYS: i64 = 30;

/// A backend with the domain logic on top.
pub struct CoreBackend {
    inner: Arc<dyn Backend>,
}

impl CoreBackend {
    /// Wrap a backend.
    pub fn new(inner: Arc<dyn Backend>) -> Self {
        Self { inner }
    }

    /// The backend underneath.
    pub fn inner(&self) -> &Arc<dyn Backend> {
        &self.inner
    }

    /// One run, as `stride-core` wants it.
    fn core_activity(
        row: &ActivityRow,
        laps: &[LapRow],
        streams: &[StreamRow],
        zones: &[ZoneRow],
    ) -> stride_core::Activity {
        let mut core_laps = Vec::with_capacity(laps.len());
        for lap in laps {
            core_laps.push(stride_core::Lap {
                lap_index: lap.lap_index,
                distance_m: Some(lap.distance_m),
                duration_s: Some(lap.duration_s),
                avg_speed_ms: lap.avg_speed_ms,
                max_speed_ms: lap.max_speed_ms,
                avg_hr: lap.avg_hr,
                max_hr: lap.max_hr,
                avg_cadence: lap.avg_cadence,
                max_cadence: lap.max_cadence,
                stride_length_m: lap.stride_length_m,
                elevation_gain_m: lap.elevation_gain_m,
                elevation_loss_m: lap.elevation_loss_m,
                start_elevation_m: lap.start_elevation_m,
                end_elevation_m: lap.end_elevation_m,
                running_distance_m: lap.running_distance_m,
                elapsed_s: lap.elapsed_s,
                pace_s_per_km: None,
            });
        }
        let mut core_zones = Vec::with_capacity(zones.len());
        for zone in zones {
            core_zones.push(stride_core::ZoneTime {
                zone_no: zone.zone_no,
                time_s: zone.time_s,
            });
        }
        let mut core_streams: BTreeMap<String, stride_core::Stream> = BTreeMap::new();
        for stream in streams {
            core_streams.insert(
                stream.kind.clone(),
                stride_core::Stream {
                    seconds: stream.seconds.clone(),
                    values: stream.values.clone(),
                    unit: stream.unit.clone(),
                },
            );
        }
        stride_core::Activity {
            id: row.id,
            name: row.name.clone(),
            local_date: row.local_date.clone(),
            start_date: row.start_time.clone().unwrap_or_default(),
            day_of_week: row.day_of_week.clone(),
            sport_type: row.sport_type.clone(),
            source: row.source.clone(),
            distance_m: row.distance_m,
            moving_time_s: Some(row.moving_time_s),
            elapsed_time_s: Some(row.elapsed_time_s),
            elevation_gain_m: Some(row.elevation_gain_m),
            avg_hr: row.avg_hr,
            max_hr: row.max_hr,
            calories: row.calories,
            running_time_s: row.running_time_s,
            walking_time_s: row.walking_time_s,
            stopped_time_s: row.stopped_time_s,
            laps: core_laps,
            zones: core_zones,
            streams: core_streams,
            fingerprint: None,
        }
    }

    /// One day of physiology, as `stride-core` wants it.
    fn core_physiology(row: &PhysiologyRow) -> stride_core::DailyPhysiology {
        stride_core::DailyPhysiology {
            date: row.date.clone(),
            sleep_s: row.sleep_s,
            sleep_score: row.sleep_score,
            deep_sleep_s: row.deep_sleep_s,
            rem_s: row.rem_s,
            light_sleep_s: row.light_sleep_s,
            awake_s: row.awake_s,
            hrv_ms: row.hrv_ms,
            hrv_status: row.hrv_status.clone(),
            resting_hr: row.resting_hr,
            max_hr: row.max_hr,
            stress_avg: row.stress_avg,
            stress_max: row.stress_max,
            body_battery_min: row.body_battery_min,
            body_battery_avg: row.body_battery_avg,
            body_battery_max: row.body_battery_max,
            steps: row.steps,
            training_readiness: row.training_readiness,
            acute_load: row.acute_load,
            chronic_load: row.chronic_load,
            vo2max: row.vo2max,
            endurance_score: row.endurance_score,
            running_tolerance: row.running_tolerance,
            source: row.source.clone(),
        }
    }

    /// One planned session, as `stride-core` wants it.
    fn core_session(row: &PlanSessionRow) -> stride_core::PlanSession {
        stride_core::PlanSession {
            id: row.id,
            date: row.date.clone(),
            session_type: row.session_type.clone(),
            title: row.title.clone(),
            details: row.details.clone(),
            target_distance_m: row.target_distance_m,
            pace_low_s_km: row.pace_low_s_km,
            pace_high_s_km: row.pace_high_s_km,
        }
    }

    /// Every run, with its splits, series and zones.
    fn core_activities(&self) -> Result<Vec<stride_core::Activity>, BackendError> {
        let rows = self.inner.activities(&crate::backend::ActivityFilter {
            all_sports: true,
            ..crate::backend::ActivityFilter::default()
        })?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let laps = self.inner.activity_laps(row.id)?;
            let streams = self.inner.activity_streams(row.id)?;
            let zones = self.inner.activity_zones(row.id)?;
            out.push(Self::core_activity(row, &laps, &streams, &zones));
        }
        Ok(out)
    }

    /// The days of a range, with the trailing window the baselines need.
    fn core_physiology_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<stride_core::DailyPhysiology>, BackendError> {
        let start = shift_days(from, -BASELINE_WINDOW_DAYS);
        let rows = self.inner.physiology(&start, to)?;
        Ok(rows.iter().map(Self::core_physiology).collect())
    }

    /// The maximum heart rate the athlete's history establishes.
    fn reference(&self) -> Result<f64, BackendError> {
        let activities = self.core_activities()?;
        Ok(stride_core::max_hr_reference(&activities))
    }

    /// The fingerprint of one run, as the API renders it.
    fn fingerprint_of(&self, row: &ActivityRow) -> Result<Value, BackendError> {
        let laps = self.inner.activity_laps(row.id)?;
        let streams = self.inner.activity_streams(row.id)?;
        let zones = self.inner.activity_zones(row.id)?;
        let activity = Self::core_activity(row, &laps, &streams, &zones);
        let reference = self.reference()?;
        let model = stride_core::fingerprint::default_zone_model(reference);
        let fingerprint = stride_core::fingerprint(&FingerprintInput {
            max_hr_reference: reference,
            zone_model: &model,
            activity: &activity,
        });
        let mut payload = serde_json::to_value(&fingerprint).map_err(|err| {
            BackendError::Malformed(format!("the fingerprint did not render: {err}"))
        })?;
        let pace = if row.distance_m > 0.0 && row.moving_time_s > 0.0 {
            Some(row.moving_time_s * 1000.0 / row.distance_m)
        } else {
            None
        };
        if let Some(object) = payload.as_object_mut() {
            object.insert("activity_id".into(), json!(row.id));
            object.insert("activity_name".into(), json!(row.name));
            object.insert("local_date".into(), json!(row.local_date));
            object.insert("distance_m".into(), json!(row.distance_m));
            object.insert("pace_s_per_km".into(), json!(pace));
            object.insert("source".into(), json!(row.source));
            object.insert("is_demo".into(), json!(row.source == "demo"));
        }
        Ok(payload)
    }
}

/// A date, moved by a number of days.
fn shift_days(day: &str, days: i64) -> String {
    match chrono::NaiveDate::parse_from_str(day.get(0..10).unwrap_or(day), "%Y-%m-%d") {
        Ok(date) => (date + chrono::Duration::days(days))
            .format("%Y-%m-%d")
            .to_string(),
        Err(_) => day.to_string(),
    }
}

/// The fingerprint of one run.
impl Backend for CoreBackend {
    fn fingerprint(&self, activity_id: i64) -> Result<Value, BackendError> {
        let Some(row) = self.inner.activity(activity_id)? else {
            return Ok(Value::Null);
        };
        self.fingerprint_of(&row)
    }

    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let rows = self.inner.activities(&crate::backend::ActivityFilter {
            date_from: Some(from.to_string()),
            date_to: Some(to.to_string()),
            all_sports: true,
            ..crate::backend::ActivityFilter::default()
        })?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            out.push(self.fingerprint_of(row)?);
        }
        Ok(out)
    }

    fn readiness(&self, day: &str) -> Result<Value, BackendError> {
        let rows = self.core_physiology_range(day, day)?;
        let baselines = stride_core::baselines(&rows, day);
        let row = rows.iter().find(|row| row.date == day);
        let readiness = stride_core::readiness(day, row, &baselines);
        serde_json::to_value(readiness)
            .map_err(|err| BackendError::Malformed(format!("the readiness did not render: {err}")))
    }

    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let rows = self.core_physiology_range(from, to)?;
        let first = chrono::NaiveDate::parse_from_str(from.get(0..10).unwrap_or(from), "%Y-%m-%d");
        let last = chrono::NaiveDate::parse_from_str(to.get(0..10).unwrap_or(to), "%Y-%m-%d");
        let (Ok(first), Ok(last)) = (first, last) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for offset in 0..=(last - first).num_days() {
            let day = (first + chrono::Duration::days(offset))
                .format("%Y-%m-%d")
                .to_string();
            let baselines = stride_core::baselines(&rows, &day);
            let row = rows.iter().find(|row| row.date == day);
            let readiness = stride_core::readiness(&day, row, &baselines);
            out.push(serde_json::to_value(readiness).map_err(|err| {
                BackendError::Malformed(format!("the readiness did not render: {err}"))
            })?);
        }
        Ok(out)
    }

    fn baselines(&self, day: &str) -> Result<Value, BackendError> {
        let rows = self.core_physiology_range(day, day)?;
        let baselines = stride_core::baselines(&rows, day);
        Ok(json!({
            "sleep_s": baselines.sleep_s,
            "hrv_ms": baselines.hrv_ms,
            "resting_hr": baselines.resting_hr,
        }))
    }

    fn compare(&self, ids: &[i64]) -> Result<Value, BackendError> {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            match self.inner.activity(*id)? {
                Some(row) => found.push(row),
                None => missing.push(*id),
            }
        }
        found.sort_by(|a, b| a.local_date.cmp(&b.local_date).then(a.id.cmp(&b.id)));

        let mut runs = Vec::with_capacity(found.len());
        for row in &found {
            let laps = self.inner.activity_laps(row.id)?;
            let streams = self.inner.activity_streams(row.id)?;
            let zones = self.inner.activity_zones(row.id)?;
            let activity = Self::core_activity(row, &laps, &streams, &zones);
            let fingerprint = self.fingerprint_of(row)?;
            let values = comparison_values(row, &fingerprint);
            let profile = stride_core::profile_for(&activity, stride_core::PROFILE_POINTS);
            runs.push(json!({
                "id": row.id,
                "name": row.name,
                "local_date": row.local_date,
                "day_of_week": row.day_of_week,
                "sport_type": row.sport_type,
                "source": row.source,
                "distance_m": row.distance_m,
                "moving_time_s": row.moving_time_s,
                "moving_time_label": duration_label(Some(row.moving_time_s)),
                "elevation_gain_m": row.elevation_gain_m,
                "avg_hr": row.avg_hr,
                "max_hr": row.max_hr,
                "calories": row.calories,
                "pace_s_per_km": pace_s_per_km(row.distance_m, row.moving_time_s),
                "has_geo": row.has_geo(),
                "values": values,
                "profile": profile,
            }));
        }
        Ok(json!({ "runs": runs, "missing": missing }))
    }

    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        let rows = self.inner.activities(&crate::backend::ActivityFilter {
            all_sports: true,
            ..crate::backend::ActivityFilter::default()
        })?;
        let mut out = Vec::new();
        for row in rows
            .iter()
            .filter(|row| {
                matches!(
                    row.sport_type.as_str(),
                    "running" | "trail_running" | "treadmill_running" | "virtualrun"
                )
            })
            .take(limit)
        {
            let streams = self.inner.activity_streams(row.id)?;
            out.push(json!({
                "id": row.id,
                "name": row.name,
                "local_date": row.local_date,
                "distance_m": row.distance_m,
                "pace_s_per_km": pace_s_per_km(row.distance_m, row.moving_time_s),
                "avg_hr": row.avg_hr,
                "elevation_gain_m": row.elevation_gain_m,
                "has_fingerprint": !streams.is_empty() || !self.inner.activity_laps(row.id)?.is_empty(),
                "has_profile": !streams.is_empty(),
                "source": row.source,
            }));
        }
        Ok(out)
    }

    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        Ok(self
            .plan_matches(from, to)?
            .iter()
            .map(|(session, activity, verdict)| {
                json!({
                    "id": Value::Null,
                    "plan_session_id": session.id,
                    "activity_id": activity.as_ref().map(|activity| activity.id),
                    "verdict": verdict.verdict,
                    "distance_delta_m": verdict.distance_delta_m,
                    "pace_delta_s_km": verdict.pace_delta_s_km,
                    "pace_band_hit": verdict.pace_band_hit,
                    "distance_hit": verdict.distance_hit,
                    "adherence_score": verdict.adherence_score,
                    "cause": verdict.cause,
                    "cause_confidence": verdict.cause_confidence,
                    "note": Value::Null,
                })
            })
            .collect())
    }

    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, BackendError> {
        let data = self.plan_inputs(from, to)?;
        let inputs = PlanInputs {
            from,
            to,
            sessions: &data.sessions,
            activities: &data.activities,
            physiology: &data.physiology,
        };
        let adherence = stride_core::plan::adherence_for(&inputs);
        serde_json::to_value(adherence)
            .map_err(|err| BackendError::Malformed(format!("the adherence did not render: {err}")))
    }

    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError> {
        let matches = self.plan_matches(from, to)?;
        let mut out = Vec::with_capacity(matches.len());
        for (session, activity, verdict) in matches {
            let row = self
                .inner
                .plan_sessions_in_range(from, to)?
                .into_iter()
                .find(|row| row.id == session.id);
            let result = json!({
                "id": Value::Null,
                "plan_session_id": session.id,
                "activity_id": activity.as_ref().map(|activity| activity.id),
                "verdict": verdict.verdict,
                "distance_delta_m": verdict.distance_delta_m,
                "pace_delta_s_km": verdict.pace_delta_s_km,
                "pace_band_hit": verdict.pace_band_hit,
                "distance_hit": verdict.distance_hit,
                "adherence_score": verdict.adherence_score,
                "cause": verdict.cause,
                "cause_confidence": verdict.cause_confidence,
                "note": Value::Null,
            });
            out.push(SessionWithResult {
                session: row.unwrap_or_else(|| session_row_from(&session)),
                activity,
                result,
            });
        }
        Ok(out)
    }

    fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
        let reference = self.reference()?;
        Ok(stride_core::fingerprint::default_zone_model(reference)
            .into_iter()
            .map(|zone| {
                json!({
                    "zone_no": zone.zone_no,
                    "name": zone.name,
                    "low_bpm": zone.low_bpm,
                    "high_bpm": zone.high_bpm,
                })
            })
            .collect())
    }

    fn max_hr_reference(&self, _activity_id: i64) -> Result<f64, BackendError> {
        self.reference()
    }

    fn activities(
        &self,
        filter: &crate::backend::ActivityFilter,
    ) -> Result<Vec<ActivityRow>, BackendError> {
        self.inner.activities(filter)
    }

    fn activity_count(&self, filter: &crate::backend::ActivityFilter) -> Result<i64, BackendError> {
        self.inner.activity_count(filter)
    }

    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError> {
        self.inner.activity(id)
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<LapRow>, BackendError> {
        self.inner.activity_laps(id)
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<StreamRow>, BackendError> {
        self.inner.activity_streams(id)
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<ZoneRow>, BackendError> {
        self.inner.activity_zones(id)
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
        self.inner.activity_geo(id)
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &crate::backend::ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        self.inner.patch_activity(id, patch)
    }

    fn counts(&self) -> Result<crate::backend::Counts, BackendError> {
        self.inner.counts()
    }

    fn physiology(&self, from: &str, to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
        self.inner.physiology(from, to)
    }

    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
        self.inner.physiology_latest()
    }

    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<crate::backend::DailyMetricRow>, BackendError> {
        self.inner.daily_metrics(from, to)
    }

    fn plan_blocks(&self) -> Result<Vec<crate::backend::PlanBlockRow>, BackendError> {
        self.inner.plan_blocks()
    }

    fn plan_block(&self, id: i64) -> Result<Option<crate::backend::PlanBlockRow>, BackendError> {
        self.inner.plan_block(id)
    }

    fn plan_phases(
        &self,
        block_id: i64,
    ) -> Result<Vec<crate::backend::PlanPhaseRow>, BackendError> {
        self.inner.plan_phases(block_id)
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<crate::backend::PlanWeekRow>, BackendError> {
        self.inner.plan_weeks(phase_id)
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
        self.inner.plan_sessions(week_id)
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError> {
        self.inner.plan_sessions_in_range(from, to)
    }

    fn create_plan_block(
        &self,
        fields: &crate::backend::PlanBlockFields,
    ) -> Result<crate::backend::PlanBlockRow, BackendError> {
        self.inner.create_plan_block(fields)
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &crate::backend::PlanPhaseFields,
    ) -> Result<crate::backend::PlanPhaseRow, BackendError> {
        self.inner.create_plan_phase(block_id, fields)
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &crate::backend::PlanWeekFields,
    ) -> Result<crate::backend::PlanWeekRow, BackendError> {
        self.inner.create_plan_week(phase_id, fields)
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &crate::backend::PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        self.inner.create_plan_session(week_id, fields)
    }

    fn import_plan(&self, request: &crate::backend::ImportRequest) -> Result<Value, BackendError> {
        self.inner.import_plan(request)
    }

    fn public_status(&self) -> Result<crate::backend::StatusView, BackendError> {
        self.inner.public_status()
    }

    fn network_status(&self) -> Result<Value, BackendError> {
        self.inner.network_status()
    }

    fn public_identity(&self) -> Result<Value, BackendError> {
        self.inner.public_identity()
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        self.inner.exit_nodes()
    }

    fn set_network_route(&self, route: &str) -> Result<Value, BackendError> {
        self.inner.set_network_route(route)
    }

    fn sync_status(&self) -> Result<Value, BackendError> {
        self.inner.sync_status()
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        self.inner.sync_runs(limit)
    }

    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, BackendError> {
        self.inner.run_sync(days, background)
    }

    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, BackendError> {
        self.inner.load_history(activities, health_days, background)
    }

    fn update_settings(
        &self,
        patch: &crate::backend::SettingsPatch,
    ) -> Result<crate::backend::StatusView, BackendError> {
        self.inner.update_settings(patch)
    }

    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        self.inner.gear()
    }

    fn pair_code(&self) -> Result<Value, BackendError> {
        self.inner.pair_code()
    }

    fn pair(&self, code: &str) -> Result<Option<Value>, BackendError> {
        self.inner.pair(code)
    }

    fn devices(&self) -> Result<Vec<Value>, BackendError> {
        self.inner.devices()
    }

    fn revoke_device(&self, device_id: &str) -> Result<Value, BackendError> {
        self.inner.revoke_device(device_id)
    }

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        self.inner.connect_personal_key(api_key)
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        self.inner.test_personal_key(api_key)
    }

    fn tile_policy(&self) -> Result<Value, BackendError> {
        self.inner.tile_policy()
    }

    fn tile(&self, z: i64, x: i64, y: i64) -> Result<crate::backend::Tile, BackendError> {
        self.inner.tile(z, x, y)
    }

    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, BackendError> {
        self.inner.garmin_oauth(action, body)
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, BackendError> {
        self.inner.import_fit(filename, bytes)
    }
}

impl CoreBackend {
    /// The inputs of the plan engine, as owned data.
    fn plan_inputs(&self, from: &str, to: &str) -> Result<PlanData, BackendError> {
        let sessions: Vec<stride_core::PlanSession> = self
            .inner
            .plan_sessions_in_range(from, to)?
            .iter()
            .map(Self::core_session)
            .collect();
        let activities = self.core_activities()?;
        let physiology = self.core_physiology_range(from, to)?;
        Ok(PlanData {
            sessions,
            activities,
            physiology,
        })
    }

    /// The matches of the plan engine, with the activity each session found.
    fn plan_matches(
        &self,
        from: &str,
        to: &str,
    ) -> Result<
        Vec<(
            stride_core::PlanSession,
            Option<ActivityRow>,
            stride_core::Verdict,
        )>,
        BackendError,
    > {
        let data = self.plan_inputs(from, to)?;
        let inputs = PlanInputs {
            from,
            to,
            sessions: &data.sessions,
            activities: &data.activities,
            physiology: &data.physiology,
        };
        let matches = stride_core::match_sessions(&inputs);
        let mut out = Vec::with_capacity(matches.len());
        for matched in matches {
            let activity = match &matched.activity {
                Some(activity) => self.inner.activity(activity.id)?,
                None => None,
            };
            out.push((matched.session, activity, matched.verdict));
        }
        Ok(out)
    }
}

/// The data the plan engine needs, owned.
struct PlanData {
    sessions: Vec<stride_core::PlanSession>,
    activities: Vec<stride_core::Activity>,
    physiology: Vec<stride_core::DailyPhysiology>,
}

/// The values of the comparison matrix, for one run.
fn comparison_values(row: &ActivityRow, fingerprint: &Value) -> Value {
    let mut values = serde_json::Map::new();
    for (key, _, _, _, _) in crate::routes::PARAMETERS {
        let value = match key {
            "avg_hr" => fingerprint.get("hr_avg").cloned().unwrap_or(Value::Null),
            "hr_drift_bpm" => fingerprint
                .get("hr_drift_bpm")
                .cloned()
                .unwrap_or(Value::Null),
            "pace_drift_pct" => fingerprint
                .get("pace_drift_pct")
                .cloned()
                .unwrap_or(Value::Null),
            "decoupling_pct" => fingerprint
                .get("decoupling_pct")
                .cloned()
                .unwrap_or(Value::Null),
            "cadence_avg" => fingerprint
                .get("cadence_avg")
                .cloned()
                .unwrap_or(Value::Null),
            "cadence_cv" => fingerprint
                .get("cadence_cv")
                .cloned()
                .unwrap_or(Value::Null),
            "efficiency_first" => fingerprint
                .get("efficiency_first")
                .cloned()
                .unwrap_or(Value::Null),
            "form_score" => fingerprint
                .get("form_score")
                .cloned()
                .unwrap_or(Value::Null),
            "running_time_s" => fingerprint
                .get("running_time_s")
                .cloned()
                .unwrap_or(Value::Null),
            "walking_time_s" => fingerprint
                .get("walking_time_s")
                .cloned()
                .unwrap_or(Value::Null),
            "stopped_time_s" => fingerprint
                .get("stopped_time_s")
                .cloned()
                .unwrap_or(Value::Null),
            "pace_s_per_km" => json!(pace_s_per_km(row.distance_m, row.moving_time_s)),
            "distance_m" => json!(row.distance_m),
            "moving_time_s" => json!(row.moving_time_s),
            "elevation_gain_m" => json!(row.elevation_gain_m),
            _ => Value::Null,
        };
        values.insert(key.to_string(), value);
    }
    Value::Object(values)
}

/// A session row, from the core's session.
fn session_row_from(session: &stride_core::PlanSession) -> PlanSessionRow {
    PlanSessionRow {
        id: session.id,
        week_id: None,
        block_id: None,
        date: session.date.clone(),
        session_type: session.session_type.clone(),
        title: session.title.clone(),
        details: session.details.clone(),
        target_distance_m: session.target_distance_m,
        pace_low_s_km: session.pace_low_s_km,
        pace_high_s_km: session.pace_high_s_km,
        hr_zone_target: None,
        status: "planned".to_string(),
        source: "manual".to_string(),
        notes: None,
    }
}

/// Round a number the way the reference does, for the callers that need it.
pub fn round(value: Option<f64>, digits: usize) -> Option<f64> {
    value.map(|value| round_to(value, digits))
}
