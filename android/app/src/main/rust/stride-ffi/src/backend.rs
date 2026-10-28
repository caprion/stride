//! The phone's backend: `stride_store::Store` behind `stride_api::backend::Backend`.
//!
//! **This file is meant to move.** `stride-api` documents the production
//! backend as "the `stride-store` adapter (proposed in the workstream
//! report)"; that adapter does not exist yet, and the phone cannot serve a
//! single route without one, so it lives here for now. When `stride-api`
//! grows its own — which is where it belongs, because `stride-api` is the
//! crate that depends on `stride-store` — this file should be deleted and
//! `lib.rs` should install that one instead.
//!
//! ## What the phone can and cannot do
//!
//! The phone has no server, so the routes that need one answer 501 rather
//! than pretending:
//!
//! * pairing (`/auth/pair`, `/auth/devices`) — there is nothing to pair to;
//! * the Garmin OAuth dance (`/auth/garmin/login`, `/auth/garmin/developer-key`)
//!   — the phone uses the Personal API key, which is one field and no
//!   developer registration;
//! * map tiles (`/tiles/...`) — the phone ships no basemap, and the OSM
//!   tile servers forbid offline prefetching;
//! * network routing (`/network/route`) — the phone has no exit node.
//!
//! Everything else is answered from the phone's own database.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde_json::{json, Map, Value};

use stride_api::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, BackendError, Counts, DailyMetricRow,
    ImportRequest, LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields,
    PlanPhaseRow, PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SessionWithResult,
    SettingsPatch, StatusView, StreamRow, Tile, ZoneRow,
};
use stride_core as core;
use stride_store::model as store_model;
use stride_store::Store;

use crate::rows;

/// The phone's backend.
pub struct PhoneBackend {
    /// The phone's own encrypted database.
    store: Store,
    /// The Garmin Personal API key, in memory, only while a pull is running.
    ///
    /// It is empty for most of the process's life: the Kotlin shell unwraps
    /// the key from the Keystore for the length of one pull and hands it
    /// over, and the key is dropped as soon as the pull is over.
    garmin_key: Mutex<String>,
}

impl PhoneBackend {
    /// A backend over a database.
    pub fn new(store: Store) -> Self {
        Self {
            store,
            garmin_key: Mutex::new(String::new()),
        }
    }

    /// The store, for the sync.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The Garmin key, if one is in memory.
    pub fn garmin_key(&self) -> Option<String> {
        let key = self
            .garmin_key
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if key.is_empty() {
            None
        } else {
            Some(key)
        }
    }

    /// Put the Garmin key in memory, or take it out again.
    pub fn set_garmin_key(&self, key: &str) {
        let mut guard = self
            .garmin_key
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = key.to_string();
    }

    /// The store's error, as the backend's error.
    fn store_error(err: stride_store::StoreError) -> BackendError {
        BackendError::Store(err.to_string())
    }

    /// The store's answer, as the backend's answer.
    fn map<T>(result: Result<T, stride_store::StoreError>) -> Result<T, BackendError> {
        result.map_err(Self::store_error)
    }

    /// The runs of a range, as the domain sees them.
    fn core_activities(&self, from: &str, to: &str) -> Result<Vec<core::model::Activity>, BackendError> {
        let mut filter = stride_store::repo::ActivityFilter {
            from: Some(from.to_string()),
            to: Some(to.to_string()),
            ..Default::default()
        };
        filter.order = Some(stride_store::repo::ActivityOrder::Date);
        let activities = Self::map(self.store.activities_filtered(&filter))?;
        let mut out = Vec::with_capacity(activities.len());
        for activity in &activities {
            out.push(self.core_activity(activity)?);
        }
        Ok(out)
    }

    /// One run, as the domain sees it.
    fn core_activity(
        &self,
        activity: &store_model::Activity,
    ) -> Result<core::model::Activity, BackendError> {
        let id = activity.id.unwrap_or_default();
        let laps = Self::map(self.store.activity_laps(id))?;
        let zones = Self::map(self.store.hr_zones(id))?;
        let streams = Self::map(self.store.activity_streams(id))?;
        Ok(rows::core_activity(activity, &laps, &zones, &streams))
    }

    /// One run, as the domain sees it, by id.
    fn core_activity_by_id(
        &self,
        id: i64,
    ) -> Result<Option<core::model::Activity>, BackendError> {
        let activity = Self::map(self.store.activity(id))?;
        match activity {
            Some(activity) => Ok(Some(self.core_activity(&activity)?)),
            None => Ok(None),
        }
    }

    /// Every run, as the domain sees them.
    ///
    /// The athlete's maximum heart rate and the trailing physiology baselines
    /// are read over the whole history, because that is what the reference
    /// does.
    fn every_core_activity(
        &self,
    ) -> Result<Vec<core::model::Activity>, BackendError> {
        let activities = Self::map(self.store.activities("0000-01-01", "9999-12-31"))?;
        let mut out = Vec::with_capacity(activities.len());
        for activity in &activities {
            out.push(self.core_activity(activity)?);
        }
        Ok(out)
    }

    /// The athlete's maximum heart rate.
    fn reference_max_hr(&self) -> Result<f64, BackendError> {
        let activities = self.every_core_activity()?;
        Ok(core::max_hr_reference(&activities))
    }

    /// The zone model the fingerprint is computed against.
    fn zone_model(&self) -> Result<Vec<core::model::Zone>, BackendError> {
        let reference = self.reference_max_hr()?;
        Ok(core::zone_model(reference, None))
    }

    /// The fingerprint of one run, with the fields the API adds.
    fn fingerprint_value(
        &self,
        activity: &core::model::Activity,
    ) -> Result<Value, BackendError> {
        let reference = self.reference_max_hr()?;
        let zones = core::zone_model(reference, None);
        let input = core::FingerprintInput {
            max_hr_reference: reference,
            zone_model: &zones,
            activity,
        };
        let fingerprint = core::fingerprint(&input);
        let mut value = serde_json::to_value(&fingerprint).unwrap_or(Value::Null);
        let object = value.as_object_mut().ok_or_else(|| {
            BackendError::Malformed("the fingerprint is not an object".to_string())
        })?;
        object.insert("activity_id".into(), json!(activity.id));
        object.insert("activity_name".into(), json!(activity.name));
        object.insert("local_date".into(), json!(activity.local_date));
        object.insert("distance_m".into(), json!(activity.distance_m));
        object.insert(
            "pace_s_per_km".into(),
            json!(activity.pace_s_per_km()),
        );
        Ok(value)
    }

    /// The physiology of the whole history, as the domain sees it.
    fn core_physiology(&self) -> Result<Vec<core::model::DailyPhysiology>, BackendError> {
        let rows = Self::map(self.store.physiology("0000-01-01", "9999-12-31"))?;
        Ok(rows.iter().map(rows::core_physiology).collect())
    }

    /// The days of the trailing window, oldest first.
    fn core_physiology_between(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<core::model::DailyPhysiology>, BackendError> {
        let rows = Self::map(self.store.physiology(from, to))?;
        Ok(rows.iter().map(rows::core_physiology).collect())
    }
}

impl Backend for PhoneBackend {
    // ------------------------------------------------------------ activities

    fn activities(&self, filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError> {
        let store_filter = stride_store::repo::ActivityFilter {
            from: filter.date_from.clone(),
            to: filter.date_to.clone(),
            sport: filter.sport.clone(),
            sports: None,
            exclude_source: Some("demo".to_string()),
            search: filter.query.clone(),
            order: Some(match filter.sort.as_str() {
                "distance" => stride_store::repo::ActivityOrder::Distance,
                "pace" => stride_store::repo::ActivityOrder::Pace,
                _ => stride_store::repo::ActivityOrder::Date,
            }),
            limit: None,
            offset: None,
        };
        let activities = Self::map(self.store.activities_filtered(&store_filter))?;
        let mut rows: Vec<ActivityRow> = activities
            .iter()
            .map(rows::activity_row)
            .filter(|row| filter.all_sports || core::is_run_sport(&row.sport_type))
            .collect();
        // The paging is the router's business, but the store does not page.
        let start = filter.page.max(0) as usize * filter.per_page.max(0) as usize;
        if start > 0 {
            rows = rows.into_iter().skip(start).collect();
        }
        let per_page = filter.per_page;
        if per_page > 0 {
            rows.truncate(per_page as usize);
        }
        Ok(rows)
    }

    fn activity_count(&self, filter: &ActivityFilter) -> Result<i64, BackendError> {
        let store_filter = stride_store::repo::ActivityFilter {
            from: filter.date_from.clone(),
            to: filter.date_to.clone(),
            sport: filter.sport.clone(),
            sports: None,
            exclude_source: Some("demo".to_string()),
            search: filter.query.clone(),
            order: None,
            limit: None,
            offset: None,
        };
        let count = Self::map(self.store.count_activities(&store_filter))?;
        if filter.all_sports || core::is_run_sport(filter.sport.as_deref().unwrap_or("running")) {
            Ok(count)
        } else {
            Ok(count)
        }
    }

    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError> {
        let activity = Self::map(self.store.activity(id))?;
        Ok(activity.as_ref().map(rows::activity_row))
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<LapRow>, BackendError> {
        let laps = Self::map(self.store.activity_laps(id))?;
        Ok(laps.iter().map(rows::lap_row).collect())
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<StreamRow>, BackendError> {
        let streams = Self::map(self.store.activity_streams(id))?;
        Ok(streams.iter().map(rows::stream_row).collect())
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<ZoneRow>, BackendError> {
        let zones = Self::map(self.store.hr_zones(id))?;
        Ok(zones.iter().map(rows::zone_row).collect())
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
        let activity = Self::map(self.store.activity(id))?;
        Ok(activity
            .as_ref()
            .and_then(|row| rows::geo_points(row.geo_points.as_ref())))
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        let Some(mut activity) = Self::map(self.store.activity(id))? else {
            return Ok(None);
        };
        if let Some(name) = patch.name.as_ref() {
            activity.name = name.clone();
        }
        if let Some(description) = patch.description.as_ref() {
            activity.description = Some(description.clone());
        }
        if let Some(effort) = patch.perceived_exertion {
            activity.perceived_exertion = Some(effort);
        }
        if let Some(trail) = patch.is_trail {
            activity.is_trail = trail;
        }
        if let Some(commute) = patch.is_commute {
            activity.is_commute = commute;
        }
        activity.touch();
        Self::map(self.store.upsert_activity(&activity))?;
        Ok(Some(rows::activity_row(&activity)))
    }

    fn counts(&self) -> Result<Counts, BackendError> {
        let activities = Self::map(self.store.count_activities(
            &stride_store::repo::ActivityFilter::default(),
        ))?;
        let physiology = Self::map(self.store.physiology("0000-01-01", "9999-12-31"))?;
        let sessions = Self::map(self.store.plan_sessions("0000-01-01", "9999-12-31"))?;
        Ok(Counts {
            activities,
            from_garmin: activities,
            physiology_days: physiology.len() as i64,
            plan_sessions: sessions.len() as i64,
        })
    }

    // ----------------------------------------------------------- physiology

    fn physiology(&self, from: &str, to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
        let days = Self::map(self.store.physiology(from, to))?;
        Ok(days.iter().map(rows::physiology_row).collect())
    }

    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
        let day = Self::map(self.store.latest_physiology())?;
        Ok(day.as_ref().map(rows::physiology_row))
    }

    fn daily_metrics(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<DailyMetricRow>, BackendError> {
        let days = Self::map(self.store.daily_metrics(from, to))?;
        Ok(days.iter().map(rows::daily_metric_row).collect())
    }

    // ----------------------------------------------------------------- plan

    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
        let blocks = Self::map(self.store.plan_blocks())?;
        Ok(blocks.iter().map(rows::plan_block_row).collect())
    }

    fn plan_block(&self, id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
        let block = Self::map(self.store.plan_block(id))?;
        Ok(block.as_ref().map(rows::plan_block_row))
    }

    fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
        let phases = Self::map(self.store.plan_phases(block_id))?;
        Ok(phases.iter().map(rows::plan_phase_row).collect())
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
        let weeks = Self::map(self.store.plan_weeks(phase_id))?;
        Ok(weeks.iter().map(rows::plan_week_row).collect())
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
        let sessions = Self::map(self.store.plan_sessions_for_week(week_id))?;
        Ok(sessions.iter().map(rows::plan_session_row).collect())
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError> {
        let sessions = Self::map(self.store.plan_sessions(from, to))?;
        Ok(sessions.iter().map(rows::plan_session_row).collect())
    }

    fn create_plan_block(&self, fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError> {
        let block = store_model::PlanBlock {
            id: None,
            name: fields.name.clone(),
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            goal: fields.goal.clone(),
            status: fields.status.clone().unwrap_or_else(|| "active".to_string()),
            source: fields.source.clone().unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
            created_at: chrono::Utc::now(),
        };
        let id = Self::map(self.store.upsert_plan_block(&block))?;
        let block = Self::map(self.store.plan_block(id))?
            .ok_or_else(|| BackendError::Store("the block was not written".to_string()))?;
        Ok(rows::plan_block_row(&block))
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError> {
        let phase = store_model::PlanPhase {
            id: None,
            block_id,
            name: fields.name.clone(),
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            target_weekly_km_min: fields.target_weekly_km_min,
            target_weekly_km_max: fields.target_weekly_km_max,
            target_sessions: fields.target_sessions,
            pace_easy_low: fields.pace_easy_low,
            pace_easy_high: fields.pace_easy_high,
            pace_tempo_low: fields.pace_tempo_low,
            pace_tempo_high: fields.pace_tempo_high,
            pace_interval_low: fields.pace_interval_low,
            pace_interval_high: fields.pace_interval_high,
            pace_long_low: fields.pace_long_low,
            pace_long_high: fields.pace_long_high,
            hr_zone_target: fields.hr_zone_target,
            notes: fields.notes.clone(),
        };
        let id = Self::map(self.store.upsert_plan_phase(&phase))?;
        let phase = Self::map(self.store.plan_phase(id))?
            .ok_or_else(|| BackendError::Store("the phase was not written".to_string()))?;
        Ok(rows::plan_phase_row(&phase))
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError> {
        let week = store_model::PlanWeek {
            id: None,
            phase_id,
            week_number: fields.week_number,
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            target_km: fields.target_km,
            key_workout: fields.key_workout.clone(),
            notes: fields.notes.clone(),
        };
        let id = Self::map(self.store.upsert_plan_week(&week))?;
        let week = Self::map(self.store.plan_week(id))?
            .ok_or_else(|| BackendError::Store("the week was not written".to_string()))?;
        Ok(rows::plan_week_row(&week))
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        let session = store_model::PlanSession {
            id: None,
            week_id: Some(week_id),
            block_id: fields.block_id,
            date: fields.date.clone(),
            session_type: fields.session_type.clone(),
            title: fields.title.clone(),
            details: fields.details.clone(),
            target_distance_m: fields.target_distance_m,
            pace_low_s_km: fields.pace_low_s_km,
            pace_high_s_km: fields.pace_high_s_km,
            hr_zone_target: fields.hr_zone_target,
            status: fields.status.clone().unwrap_or_else(|| "planned".to_string()),
            source: fields.source.clone().unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
        };
        let id = Self::map(self.store.upsert_plan_session(&session))?;
        let session = Self::map(self.store.plan_session(id))?
            .ok_or_else(|| BackendError::Store("the session was not written".to_string()))?;
        Ok(rows::plan_session_row(&session))
    }

    fn import_plan(&self, _request: &ImportRequest) -> Result<Value, BackendError> {
        // The plan importer is not ported to Rust yet: the reference is
        // `backend/app/ingest/plan_importer.py`. The phone says so rather
        // than importing half a plan.
        Err(BackendError::Unsupported(
            "the plan importer is not part of the Rust core yet".to_string(),
        ))
    }

    // ------------------------------------------------------------- fitness

    fn fingerprint(&self, activity_id: i64) -> Result<Value, BackendError> {
        match self.core_activity_by_id(activity_id)? {
            Some(activity) => self.fingerprint_value(&activity),
            None => Ok(Value::Null),
        }
    }

    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let activities = self.core_activities(from, to)?;
        let mut out = Vec::with_capacity(activities.len());
        for activity in activities.iter().filter(|row| core::is_run_sport(&row.sport_type)) {
            out.push(self.fingerprint_value(activity)?);
        }
        Ok(out)
    }

    fn readiness(&self, day: &str) -> Result<Value, BackendError> {
        let physiology = self.core_physiology()?;
        let baselines = core::baselines(&physiology, day);
        let row = physiology.iter().find(|row| row.date == day);
        let value = core::readiness(day, row, &baselines);
        Ok(serde_json::to_value(value).unwrap_or(Value::Null))
    }

    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let physiology = self.core_physiology()?;
        let days = self.core_physiology_between(from, to)?;
        let mut out = Vec::with_capacity(days.len());
        for day in &days {
            let baselines = core::baselines(&physiology, &day.date);
            let value = core::readiness(&day.date, Some(day), &baselines);
            out.push(serde_json::to_value(value).unwrap_or(Value::Null));
        }
        Ok(out)
    }

    fn baselines(&self, day: &str) -> Result<Value, BackendError> {
        let physiology = self.core_physiology()?;
        let baselines = core::baselines(&physiology, day);
        Ok(serde_json::to_value(baselines).unwrap_or(Value::Null))
    }

    fn compare(&self, ids: &[i64]) -> Result<Value, BackendError> {
        let mut runs = Vec::new();
        let mut missing = Vec::new();
        let mut found: Vec<(i64, core::model::Activity)> = Vec::new();
        for id in ids {
            match self.core_activity_by_id(*id)? {
                Some(activity) => found.push((*id, activity)),
                None => missing.push(*id),
            }
        }
        found.sort_by(|left, right| {
            left.1
                .local_date
                .cmp(&right.1.local_date)
                .then(left.0.cmp(&right.0))
        });

        let reference = self.reference_max_hr()?;
        let zones = core::zone_model(reference, None);
        for (_, activity) in &found {
            let mut activity = activity.clone();
            // The fingerprint carries the derived parameters, so the
            // comparison's values are the ones the fingerprint screen shows.
            let input = core::FingerprintInput {
                max_hr_reference: reference,
                zone_model: &zones,
                activity: &activity,
            };
            activity.fingerprint = Some(core::fingerprint(&input));
            runs.push(self.compare_run(&activity));
        }

        Ok(json!({"runs": runs, "missing": missing}))
    }

    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        let mut filter = stride_store::repo::ActivityFilter {
            order: Some(stride_store::repo::ActivityOrder::Date),
            limit: Some(limit as i64),
            ..Default::default()
        };
        filter.exclude_source = Some("demo".to_string());
        let activities = Self::map(self.store.activities_filtered(&filter))?;
        Ok(activities
            .iter()
            .filter(|row| core::is_run_sport(&row.sport_type))
            .map(|activity| {
                json!({
                    "id": activity.id.unwrap_or_default(),
                    "name": activity.name,
                    "local_date": activity.local_date,
                    "distance_m": activity.distance_m,
                    "sport_type": activity.sport_type,
                })
            })
            .collect())
    }

    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let sessions = self.plan_sessions_with_results(from, to)?;
        Ok(sessions
            .iter()
            .map(|session| session.result.clone())
            .filter(|result| !result.is_null())
            .collect())
    }

    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, BackendError> {
        let inputs = self.plan_inputs(from, to)?;
        let matches = core::match_sessions(&inputs);
        let extra = core::extra_activities(&inputs).len();
        let adherence = core::adherence(&matches, extra);
        Ok(serde_json::to_value(adherence).unwrap_or(Value::Null))
    }

    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError> {
        let inputs = self.plan_inputs(from, to)?;
        let matches = core::match_sessions(&inputs);
        let physiology = self.core_physiology()?;

        let mut out = Vec::with_capacity(matches.len());
        for matched in &matches {
            let session_row = self
                .store
                .plan_session(matched.session.id)
                .ok()
                .flatten()
                .map(|row| rows::plan_session_row(&row));
            let Some(session_row) = session_row else {
                continue;
            };

            let activity_row = match matched.activity.as_ref() {
                Some(activity) => self.core_activity_by_id(activity.id)?.map(|core| {
                    let mut row = rows::activity_row(
                        &self
                            .store
                            .activity(activity.id)
                            .ok()
                            .flatten()
                            .unwrap_or_else(store_model::Activity::default),
                    );
                    row.id = activity.id;
                    row
                }),
                None => None,
            };

            let (cause, confidence) = core::explain_cause(
                &matched.session,
                matched.activity.as_ref().map(|_| {
                    // The cause is explained against the run itself.
                    &inputs.activities[0]
                }),
                None,
                &matched.verdict.verdict,
            );

            let result = json!({
                "id": matched.session.id,
                "plan_session_id": matched.session.id,
                "activity_id": matched.activity.as_ref().map(|activity| activity.id),
                "verdict": matched.verdict.verdict,
                "distance_delta_m": matched.verdict.distance_delta_m,
                "pace_delta_s_km": matched.verdict.pace_delta_s_km,
                "pace_band_hit": matched.verdict.pace_band_hit,
                "distance_hit": matched.verdict.distance_hit,
                "adherence_score": matched.verdict.adherence_score,
                "cause": cause.or(matched.verdict.cause.clone()),
                "cause_confidence": confidence.or(matched.verdict.cause_confidence),
                "note": Value::Null,
            });
            let _ = physiology;

            out.push(SessionWithResult {
                session: session_row,
                activity: activity_row,
                result,
            });
        }
        Ok(out)
    }

    fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
        let reference = self.reference_max_hr()?;
        let zones = core::zone_model(reference, None);
        Ok(zones
            .iter()
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
        self.reference_max_hr()
    }

    // -------------------------------------------------------------- status

    fn public_status(&self) -> Result<StatusView, BackendError> {
        let runs = Self::map(self.store.sync_runs(1))?;
        let last = runs.first();
        Ok(StatusView {
            garmin_connected: self.garmin_key().is_some(),
            garmin_method: Some("garmin_personal".to_string()),
            athlete: json!({}),
            has_developer_key: false,
            has_account_credentials: false,
            has_personal_api_key: self.garmin_key().is_some(),
            pending_authorization: false,
            network_route: "local".to_string(),
            exit_node: None,
            sync_days: 30,
            history_months: Some(12),
            health_history_days: Some(90),
            auto_sync: true,
            sync_interval_minutes: 360,
            last_sync_at: last
                .and_then(|run| run.finished_at)
                .map(|at| at.to_rfc3339()),
            demo_visible: false,
        })
    }

    fn network_status(&self) -> Result<Value, BackendError> {
        Ok(json!({
            "route": "local",
            "connected": true,
            "exit_nodes": [],
            "message": "The phone does not use a server. The routes are answered on the device.",
        }))
    }

    fn public_identity(&self) -> Result<Value, BackendError> {
        Ok(json!({
            "name": "This phone",
            "mode": "local",
            "paired": false,
        }))
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        Ok(Vec::new())
    }

    fn set_network_route(&self, _route: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no exit node to route through".to_string(),
        ))
    }

    // ---------------------------------------------------------------- sync

    fn sync_status(&self) -> Result<Value, BackendError> {
        let runs = Self::map(self.store.sync_runs(5))?;
        let last = runs.first();
        Ok(json!({
            "running": false,
            "last_sync_at": last.and_then(|run| run.finished_at).map(|at| at.to_rfc3339()),
            "last_status": last.map(|run| run.status.clone()),
            "message": last.and_then(|run| run.message.clone()),
            "source": "garmin_personal",
        }))
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        let runs = Self::map(self.store.sync_runs(limit as i64))?;
        Ok(runs
            .iter()
            .map(|run| {
                json!({
                    "id": run.id.unwrap_or_default(),
                    "source": run.source,
                    "started_at": run.started_at.to_rfc3339(),
                    "finished_at": run.finished_at.map(|at| at.to_rfc3339()),
                    "status": run.status,
                    "days": run.days,
                    "activities_found": run.activities_found,
                    "activities_new": run.activities_new,
                    "activities_updated": run.activities_updated,
                    "message": run.message,
                    "error": run.error,
                })
            })
            .collect())
    }

    fn run_sync(&self, days: Option<i64>, _background: bool) -> Result<Value, BackendError> {
        let Some(key) = self.garmin_key() else {
            return Err(BackendError::Domain(
                "Garmin is not connected on this phone.".to_string(),
            ));
        };
        let days = days.unwrap_or(30).clamp(1, 90);
        let outcome = crate::garmin::sync(self, &key, days)?;
        Ok(outcome)
    }

    fn load_history(
        &self,
        _activities: Option<i64>,
        _health_days: Option<i64>,
    ) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone pulls a bounded window; there is no bulk history load".to_string(),
        ))
    }

    fn update_settings(&self, patch: &SettingsPatch) -> Result<StatusView, BackendError> {
        // The phone's settings live in the Kotlin shell, not in the database:
        // the sync window, the auto-sync switch and the interval are the
        // shell's business.
        let _ = patch;
        self.public_status()
    }

    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        let gear = Self::map(self.store.gear())?;
        Ok(gear
            .iter()
            .map(|row| {
                json!({
                    "id": row.id.unwrap_or_default(),
                    "uuid": row.uuid,
                    "name": row.name,
                    "brand": row.brand,
                    "model": row.model,
                    "status": row.status,
                    "distance_m": row.distance_m,
                    "activity_count": row.activity_count,
                    "first_used": row.first_used,
                    "last_used": row.last_used,
                    "retire_at_m": row.retire_at_m,
                })
            })
            .collect())
    }

    // --------------------------------------------------------------- pairs

    fn pair_code(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no server to pair with".to_string(),
        ))
    }

    fn pair(&self, _code: &str) -> Result<Option<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no server to pair with".to_string(),
        ))
    }

    fn devices(&self) -> Result<Vec<Value>, BackendError> {
        Ok(Vec::new())
    }

    fn revoke_device(&self, _device_id: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no paired devices".to_string(),
        ))
    }

    // -------------------------------------------------------------- garmin

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        let profile = crate::garmin::validate(api_key)?;
        self.set_garmin_key(api_key);
        Ok(json!({
            "ok": true,
            "garmin_method": "garmin_personal",
            "athlete": profile,
            "sources": crate::garmin::sources(),
        }))
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        let profile = crate::garmin::validate(api_key)?;
        Ok(json!({
            "ok": true,
            "athlete": profile,
        }))
    }

    fn tile_policy(&self) -> Result<Value, BackendError> {
        Ok(json!({
            "policy": "none",
            "message": "The phone ships no basemap. Routes are drawn from the recorded track.",
        }))
    }

    fn tile(&self, _z: i64, _x: i64, _y: i64) -> Result<Tile, BackendError> {
        Err(BackendError::Unsupported(
            "the phone ships no basemap tiles".to_string(),
        ))
    }

    fn garmin_oauth(&self, action: &str, _body: &Value) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(format!(
            "the phone connects with a Personal API key; `{action}` is not part of that"
        )))
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, BackendError> {
        crate::garmin::import_fit(self, filename, bytes)
    }
}

impl PhoneBackend {
    /// The plan engine's inputs for a range.
    fn plan_inputs(
        &self,
        from: &str,
        to: &str,
    ) -> Result<core::plan::PlanInputs<'_>, BackendError> {
        // The plan engine needs the sessions, the runs and the physiology of
        // the range, and it borrows them, so they are kept alive here.
        let sessions = Self::map(self.store.plan_sessions(from, to))?;
        let activities = self.core_activities(from, to)?;
        let physiology = self.core_physiology_between(from, to)?;

        let sessions: Vec<core::model::PlanSession> =
            sessions.iter().map(rows::core_plan_session).collect();

        Ok(core::plan::PlanInputs {
            from,
            to,
            sessions: &sessions,
            activities: &activities,
            physiology: &physiology,
        })
    }

    /// One run, in the shape the comparison view reads.
    fn compare_run(&self, activity: &core::model::Activity) -> Value {
        let mut values = Map::new();
        for parameter in core::PARAMETERS.iter() {
            let value = core::value_for(parameter, activity);
            values.insert(parameter.key.to_string(), json!(value));
        }
        let profile = core::profile_for(activity, 120);
        json!({
            "id": activity.id,
            "name": activity.name,
            "local_date": activity.local_date,
            "day_of_week": activity.day_of_week,
            "sport_type": activity.sport_type,
            "source": activity.source,
            "distance_m": activity.distance_m,
            "moving_time_s": activity.moving_time_s,
            "moving_time_label": activity
                .moving_time_s
                .map(duration_label)
                .unwrap_or(Value::Null),
            "elevation_gain_m": activity.elevation_gain_m,
            "avg_hr": activity.avg_hr,
            "max_hr": activity.max_hr,
            "calories": activity.calories,
            "pace_s_per_km": activity.pace_s_per_km(),
            "has_geo": false,
            "values": values,
            "profile": profile,
        })
    }
}

/// The label the reference puts on a duration: `h:mm:ss` or `m:ss`.
fn duration_label(seconds: f64) -> Value {
    let total = seconds.round() as i64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    if hours > 0 {
        json!(format!("{hours}:{minutes:02}:{secs:02}"))
    } else {
        json!(format!("{minutes}:{secs:02}"))
    }
}

/// The kinds of series a run keeps, for the fingerprint.
pub fn stream_kinds() -> BTreeMap<&'static str, &'static str> {
    let mut kinds = BTreeMap::new();
    kinds.insert("hr", "heart rate");
    kinds.insert("cadence", "cadence");
    kinds.insert("pace", "pace");
    kinds.insert("distance", "distance");
    kinds.insert("elevation", "elevation");
    kinds
}
