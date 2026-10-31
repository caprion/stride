//! An in-memory backend.
//!
//! It exists so the router can be exercised end to end — every route, every
//! shape — without a database, and so the golden test can drive the router
//! with a known dataset.
//!
//! It is **not** the production backend. The production backend is the
//! `stride-store` adapter (proposed in the workstream report); this one holds
//! rows in memory and implements the derived numbers by looking them up in a
//! table of canned values, because the derived numbers belong to
//! `stride-core` and this crate must not reimplement them.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Value};

use crate::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, Counts, DailyMetricRow, ImportRequest,
    LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields, PlanPhaseRow,
    PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SessionWithResult,
    SettingsPatch, StatusView, StreamRow, SyncRunRow, Tile, ZoneRow,
};
use crate::error::BackendError;

/// The in-memory backend.
#[derive(Debug, Default)]
pub struct MemBackend {
    activities: Mutex<Vec<ActivityRow>>,
    laps: Mutex<Vec<LapRow>>,
    streams: Mutex<Vec<StreamRow>>,
    zones: Mutex<Vec<ZoneRow>>,
    physiology: Mutex<Vec<PhysiologyRow>>,
    daily_metrics: Mutex<Vec<DailyMetricRow>>,
    blocks: Mutex<Vec<PlanBlockRow>>,
    phases: Mutex<Vec<PlanPhaseRow>>,
    weeks: Mutex<Vec<PlanWeekRow>>,
    sessions: Mutex<Vec<PlanSessionRow>>,
    gear: Mutex<Vec<Value>>,
    sync_runs: Mutex<Vec<Value>>,
    status: Mutex<StatusView>,
    /// The derived numbers, keyed by the lookup the router would make.
    derived: Mutex<HashMap<String, Value>>,
    next_id: Mutex<i64>,
}

impl MemBackend {
    /// An empty backend.
    pub fn new() -> Self {
        Self {
            next_id: Mutex::new(1),
            status: Mutex::new(StatusView {
                athlete: json!({}),
                demo_visible: true,
                sync_days: 90,
                auto_sync: true,
                sync_interval_minutes: 180,
                history_months: Some(12),
                health_history_days: Some(90),
                network_route: "auto".to_string(),
                ..StatusView::default()
            }),
            ..Self::default()
        }
    }

    /// Add a run.
    pub fn add_activity(&self, activity: ActivityRow) {
        self.activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(activity);
    }

    /// Add a split.
    pub fn add_lap(&self, lap: LapRow) {
        self.laps
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(lap);
    }

    /// Add a series.
    pub fn add_stream(&self, stream: StreamRow) {
        self.streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(stream);
    }

    /// Add a heart-rate zone.
    pub fn add_zone(&self, zone: ZoneRow) {
        self.zones
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(zone);
    }

    /// Add a physiology row.
    pub fn add_physiology(&self, row: PhysiologyRow) {
        self.physiology
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(row);
    }

    /// Add a daily metric.
    pub fn add_daily_metric(&self, row: DailyMetricRow) {
        self.daily_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(row);
    }

    /// Add a plan block.
    pub fn add_block(&self, block: PlanBlockRow) {
        self.blocks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(block);
    }

    /// Add a plan phase.
    pub fn add_phase(&self, phase: PlanPhaseRow) {
        self.phases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(phase);
    }

    /// Add a plan week.
    pub fn add_week(&self, week: PlanWeekRow) {
        self.weeks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(week);
    }

    /// Add a planned session.
    pub fn add_session(&self, session: PlanSessionRow) {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(session);
    }

    /// Add a shoe.
    pub fn add_gear(&self, gear: Value) {
        self.gear
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(gear);
    }

    /// Set the state the browser sees.
    pub fn set_status(&self, status: StatusView) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }

    /// Set one of the derived numbers.
    ///
    /// The keys are the lookups the router makes: `fingerprint:12`,
    /// `fingerprints:2026-01-01:2026-12-31`, `readiness:2026-03-02`,
    /// `readiness_range:from:to`, `baselines:2026-03-02`, `compare:1,2`,
    /// `compare_candidates`, `plan_results:from:to`,
    /// `plan_adherence:from:to` and `plan_sessions:from:to`.
    pub fn set_derived(&self, key: &str, value: Value) {
        self.derived
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(key.to_string(), value);
    }

    /// The derived number for a key.
    ///
    /// An exact match wins; failing that, the first key with the same prefix
    /// is used, so a caller that does not know which date range the router
    /// asked for can still answer it.
    fn derived(&self, key: &str) -> Option<Value> {
        let guard = self
            .derived
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(value) = guard.get(key) {
            return Some(value.clone());
        }
        let prefix = match key.split_once(':') {
            Some((head, _)) => format!("{head}:"),
            None => key.to_string(),
        };
        guard
            .iter()
            .find(|(name, _)| name.starts_with(&prefix))
            .map(|(_, value)| value.clone())
    }

    /// The next free id.
    fn next_id(&self) -> i64 {
        let mut guard = self
            .next_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let id = *guard;
        *guard += 1;
        id
    }
}

impl Backend for MemBackend {
    fn activities(&self, filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError> {
        let mut rows: Vec<ActivityRow> = self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        rows.retain(|row| {
            if !filter.demo_visible && row.source == "demo" {
                return false;
            }
            match &filter.sport {
                Some(sport) => &row.sport_type == sport,
                None => filter.all_sports || crate::backend::is_run_sport(&row.sport_type),
            }
        });
        if let Some(from) = &filter.date_from {
            rows.retain(|row| row.local_date.as_str() >= from.as_str());
        }
        if let Some(to) = &filter.date_to {
            rows.retain(|row| row.local_date.as_str() <= to.as_str());
        }
        if let Some(search) = &filter.query {
            let needle = search.trim().to_lowercase();
            rows.retain(|row| {
                row.name.to_lowercase().contains(&needle)
                    || row
                        .description
                        .as_deref()
                        .is_some_and(|text| text.to_lowercase().contains(&needle))
                    || row.local_date.contains(&needle)
            });
        }
        match filter.sort.as_str() {
            "distance" => rows.sort_by(|a, b| {
                b.distance_m
                    .partial_cmp(&a.distance_m)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            "pace" => rows.sort_by(|a, b| {
                a.moving_time_s
                    .partial_cmp(&b.moving_time_s)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            "elevation" => rows.sort_by(|a, b| {
                b.elevation_gain_m
                    .partial_cmp(&a.elevation_gain_m)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => rows.sort_by(|a, b| b.start_time.cmp(&a.start_time)),
        }
        if filter.per_page > 0 {
            let offset = ((filter.page - 1).max(0) * filter.per_page) as usize;
            rows = rows
                .into_iter()
                .skip(offset)
                .take(filter.per_page as usize)
                .collect();
        }
        Ok(rows)
    }

    fn activity_count(&self, filter: &ActivityFilter) -> Result<i64, BackendError> {
        let unpaged = ActivityFilter {
            page: 1,
            per_page: 0,
            ..filter.clone()
        };
        Ok(self.activities(&unpaged)?.len() as i64)
    }

    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError> {
        Ok(self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .find(|row| row.id == id)
            .cloned())
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<LapRow>, BackendError> {
        let mut laps: Vec<LapRow> = self
            .laps
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|lap| lap.activity_id == id)
            .cloned()
            .collect();
        laps.sort_by_key(|lap| lap.lap_index);
        Ok(laps)
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<StreamRow>, BackendError> {
        Ok(self
            .streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|stream| stream.activity_id == id)
            .cloned()
            .collect())
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<ZoneRow>, BackendError> {
        let mut zones: Vec<ZoneRow> = self
            .zones
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|zone| zone.activity_id == id)
            .cloned()
            .collect();
        zones.sort_by_key(|zone| zone.zone_no);
        Ok(zones)
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
        Ok(self.activity(id)?.and_then(|row| row.geo_points))
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        let mut rows = self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(row) = rows.iter_mut().find(|row| row.id == id) else {
            return Ok(None);
        };
        if let Some(name) = &patch.name {
            row.name = name.clone();
        }
        if let Some(description) = &patch.description {
            row.description = Some(description.clone());
        }
        if let Some(exertion) = patch.perceived_exertion {
            row.perceived_exertion = Some(exertion);
        }
        if let Some(is_trail) = patch.is_trail {
            row.is_trail = is_trail;
        }
        if let Some(is_commute) = patch.is_commute {
            row.is_commute = is_commute;
        }
        Ok(Some(row.clone()))
    }

    fn counts(&self) -> Result<Counts, BackendError> {
        let activities = self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let physiology = self
            .physiology
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(Counts {
            activities: activities.len() as i64,
            from_garmin: activities.iter().filter(|row| row.source != "demo").count() as i64,
            physiology_days: physiology.len() as i64,
            plan_sessions: sessions.len() as i64,
        })
    }

    fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<ActivityRow>, BackendError> {
        Ok(self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .find(|row| row.source == source && row.external_id == external_id)
            .cloned())
    }

    fn upsert_synced_activity(
        &self,
        activity: &ActivityRow,
        laps: &[LapRow],
        _streams: &[StreamRow],
    ) -> Result<i64, BackendError> {
        let mut row = activity.clone();
        if row.id == 0 {
            let mut next = self
                .next_id
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            row.id = *next;
            *next += 1;
        }
        row.lap_count = laps.len() as i64;
        let mut guard = self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match guard.iter_mut().find(|existing| existing.id == row.id) {
            Some(existing) => *existing = row,
            None => guard.push(row),
        }
        Ok(activity.id)
    }

    fn upsert_physiology(&self, row: &PhysiologyRow) -> Result<(), BackendError> {
        let mut guard = self
            .physiology
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match guard.iter_mut().find(|existing| existing.date == row.date) {
            Some(existing) => *existing = row.clone(),
            None => guard.push(row.clone()),
        }
        Ok(())
    }

    fn upsert_daily_metric(&self, row: &DailyMetricRow) -> Result<(), BackendError> {
        let mut guard = self
            .daily_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match guard.iter_mut().find(|existing| existing.date == row.date) {
            Some(existing) => *existing = row.clone(),
            None => guard.push(row.clone()),
        }
        Ok(())
    }

    fn upsert_sync_run(&self, run: &SyncRunRow) -> Result<i64, BackendError> {
        let mut guard = self
            .sync_runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.push(json!({
            "id": run.id,
            "source": run.source,
            "started_at": run.started_at,
            "finished_at": run.finished_at,
            "status": run.status,
            "days": run.days,
            "activities_found": run.activities_found,
            "activities_new": run.activities_new,
            "activities_updated": run.activities_updated,
            "message": run.message,
            "error": run.error,
        }));
        Ok(run.id.unwrap_or_default())
    }

    fn garmin_personal_key(&self) -> Result<Option<String>, BackendError> {
        Ok(None)
    }

    fn physiology(&self, from: &str, to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
        let mut rows: Vec<PhysiologyRow> = self
            .physiology
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|row| row.date.as_str() >= from && row.date.as_str() <= to)
            .cloned()
            .collect();
        rows.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(rows)
    }

    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
        Ok(self
            .physiology
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .max_by(|a, b| a.date.cmp(&b.date))
            .cloned())
    }

    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyMetricRow>, BackendError> {
        let mut rows: Vec<DailyMetricRow> = self
            .daily_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|row| {
                from.map_or(true, |from| row.date.as_str() >= from)
                    && to.map_or(true, |to| row.date.as_str() <= to)
            })
            .cloned()
            .collect();
        rows.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(rows)
    }

    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
        let mut blocks = self
            .blocks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        blocks.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        Ok(blocks)
    }

    fn plan_block(&self, id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
        Ok(self
            .blocks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .find(|block| block.id == id)
            .cloned())
    }

    fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
        let mut phases: Vec<PlanPhaseRow> = self
            .phases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|phase| phase.block_id == block_id)
            .cloned()
            .collect();
        phases.sort_by(|a, b| a.start_date.cmp(&b.start_date));
        Ok(phases)
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
        let mut weeks: Vec<PlanWeekRow> = self
            .weeks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|week| week.phase_id == phase_id)
            .cloned()
            .collect();
        weeks.sort_by_key(|week| week.week_number);
        Ok(weeks)
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
        let mut sessions: Vec<PlanSessionRow> = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|session| session.week_id == Some(week_id))
            .cloned()
            .collect();
        sessions.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(sessions)
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError> {
        let mut sessions: Vec<PlanSessionRow> = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|session| session.date.as_str() >= from && session.date.as_str() <= to)
            .cloned()
            .collect();
        sessions.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(sessions)
    }

    fn create_plan_block(&self, fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError> {
        let block = PlanBlockRow {
            id: self.next_id(),
            name: fields.name.clone(),
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            goal: fields.goal.clone(),
            status: fields
                .status
                .clone()
                .unwrap_or_else(|| "active".to_string()),
            source: fields
                .source
                .clone()
                .unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
        };
        self.blocks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(block.clone());
        Ok(block)
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError> {
        let phase = PlanPhaseRow {
            id: self.next_id(),
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
        self.phases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(phase.clone());
        Ok(phase)
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError> {
        let week = PlanWeekRow {
            id: self.next_id(),
            phase_id,
            week_number: fields.week_number,
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            target_km: fields.target_km,
            key_workout: fields.key_workout.clone(),
            notes: fields.notes.clone(),
        };
        self.weeks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(week.clone());
        Ok(week)
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        let block_id = self
            .weeks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .find(|week| week.id == week_id)
            .and_then(|week| {
                self.phases
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .iter()
                    .find(|phase| phase.id == week.phase_id)
                    .map(|phase| phase.block_id)
            });
        let session = PlanSessionRow {
            id: self.next_id(),
            week_id: Some(week_id),
            block_id,
            date: fields.date.clone(),
            session_type: fields.session_type.clone(),
            title: fields.title.clone(),
            details: fields.details.clone(),
            target_distance_m: fields.target_distance_m,
            pace_low_s_km: fields.pace_low_s_km,
            pace_high_s_km: fields.pace_high_s_km,
            hr_zone_target: fields.hr_zone_target,
            status: fields
                .status
                .clone()
                .unwrap_or_else(|| "planned".to_string()),
            source: fields
                .source
                .clone()
                .unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
        };
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(session.clone());
        Ok(session)
    }

    fn import_plan(&self, request: &ImportRequest) -> Result<Value, BackendError> {
        let key = format!("plan_import:{}", request.format);
        if let Some(value) = self.derived(&key) {
            return Ok(value);
        }
        // The document is parsed by the importer and written by the backend.
        let year = chrono::Datelike::year(&chrono::Utc::now());
        let plan = match crate::plan_import::parse_plan(
            &request.content,
            &request.format,
            request.name.as_deref(),
            year,
        ) {
            Ok(plan) => plan,
            Err(message) => {
                return Ok(json!({
                    "ok": false,
                    "block_id": Value::Null,
                    "name": request.name,
                    "phases": 0,
                    "weeks": 0,
                    "sessions": 0,
                    "warnings": [],
                    "error": message,
                }));
            }
        };
        crate::plan_import::write_plan(self, &plan)
    }

    fn fingerprint(&self, activity_id: i64) -> Result<Value, BackendError> {
        self.derived(&format!("fingerprint:{activity_id}"))
            .ok_or_else(|| BackendError::Unsupported("no fingerprint".to_string()))
    }

    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let value = self
            .derived(&format!("fingerprints:{from}:{to}"))
            .ok_or_else(|| BackendError::Unsupported("no fingerprints".to_string()))?;
        Ok(value.as_array().cloned().unwrap_or_default())
    }

    fn readiness(&self, day: &str) -> Result<Value, BackendError> {
        self.derived(&format!("readiness:{day}"))
            .ok_or_else(|| BackendError::Unsupported("no readiness".to_string()))
    }

    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let value = self
            .derived(&format!("readiness_range:{from}:{to}"))
            .ok_or_else(|| BackendError::Unsupported("no readiness".to_string()))?;
        Ok(value.as_array().cloned().unwrap_or_default())
    }

    fn baselines(&self, day: &str) -> Result<Value, BackendError> {
        self.derived(&format!("baselines:{day}"))
            .ok_or_else(|| BackendError::Unsupported("no baselines".to_string()))
    }

    fn compare(&self, ids: &[i64]) -> Result<Value, BackendError> {
        let key = ids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<String>>()
            .join(",");
        self.derived(&format!("compare:{key}"))
            .ok_or_else(|| BackendError::Unsupported("no comparison".to_string()))
    }

    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        let value = self
            .derived("compare_candidates")
            .ok_or_else(|| BackendError::Unsupported("no candidates".to_string()))?;
        Ok(value
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .take(limit)
            .collect())
    }

    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        let value = self
            .derived(&format!("plan_results:{from}:{to}"))
            .ok_or_else(|| BackendError::Unsupported("no plan results".to_string()))?;
        Ok(value.as_array().cloned().unwrap_or_default())
    }

    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, BackendError> {
        self.derived(&format!("plan_adherence:{from}:{to}"))
            .ok_or_else(|| BackendError::Unsupported("no adherence".to_string()))
    }

    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError> {
        let value = self
            .derived(&format!("plan_sessions:{from}:{to}"))
            .ok_or_else(|| BackendError::Unsupported("no plan sessions".to_string()))?;
        let mut out = Vec::new();
        for item in value.as_array().cloned().unwrap_or_default() {
            let session = item
                .get("session")
                .cloned()
                .and_then(|session| serde_json::from_value::<PlanSessionRow>(session).ok())
                .unwrap_or_default();
            let activity = item
                .get("activity")
                .cloned()
                .and_then(|activity| serde_json::from_value::<ActivityRow>(activity).ok());
            let result = item.get("result").cloned().unwrap_or(Value::Null);
            out.push(SessionWithResult {
                session,
                activity,
                result,
            });
        }
        Ok(out)
    }

    fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
        if let Some(value) = self.derived("athlete_zones") {
            return Ok(value.as_array().cloned().unwrap_or_default());
        }
        // Garmin's default five-zone model, anchored on the athlete's
        // observed maximum heart rate — the reference: `fingerprint.zone_model`.
        let reference = self.max_hr_reference(0)?;
        let bounds = [0.0, 0.6, 0.7, 0.8, 0.9];
        let names = ["Recovery", "Aerobic", "Tempo", "Threshold", "Anaerobic"];
        Ok((0..5)
            .map(|index| {
                json!({
                    "zone_no": index as i64 + 1,
                    "name": names[index],
                    "low_bpm": reference * bounds[index],
                    "high_bpm": if index < 4 {
                        json!(reference * bounds[index + 1])
                    } else {
                        Value::Null
                    },
                })
            })
            .collect())
    }

    fn max_hr_reference(&self, _activity_id: i64) -> Result<f64, BackendError> {
        if let Some(value) = self.derived("max_hr_reference") {
            if let Some(number) = value.as_f64() {
                return Ok(number);
            }
        }
        let peak = self
            .activities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter_map(|row| row.max_hr)
            .filter(|hr| *hr > 0.0)
            .fold(None::<f64>, |acc, hr| {
                Some(match acc {
                    None => hr,
                    Some(current) => current.max(hr),
                })
            });
        Ok(peak.unwrap_or(190.0))
    }

    fn public_status(&self) -> Result<StatusView, BackendError> {
        Ok(self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone())
    }

    fn network_status(&self) -> Result<Value, BackendError> {
        Ok(self.derived("network_status").unwrap_or_else(|| {
            json!({
                "tailscale_available": false,
                "exit_node": Value::Null,
                "exit_node_active": false,
                "public_ip": Value::Null,
                "public_org": Value::Null,
                "route": "direct",
            })
        }))
    }

    fn public_identity(&self) -> Result<Value, BackendError> {
        Ok(self
            .derived("public_identity")
            .unwrap_or_else(|| json!({"ip": Value::Null, "org": Value::Null})))
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        Ok(self
            .derived("exit_nodes")
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default())
    }

    fn set_network_route(&self, route: &str) -> Result<Value, BackendError> {
        {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            status.network_route = route.to_string();
        }
        Ok(
            json!({"ok": true, "message": format!("route set to {route}"), "status": self.network_status()?}),
        )
    }

    fn sync_status(&self) -> Result<Value, BackendError> {
        Ok(self
            .derived("sync_status")
            .unwrap_or_else(|| json!({"running": false, "last_error": null})))
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        Ok(self
            .sync_runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .take(limit)
            .cloned()
            .collect())
    }

    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, BackendError> {
        Ok(json!({"started": background, "days": days}))
    }

    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, BackendError> {
        Ok(json!({
            "started": background,
            "activities": activities,
            "health_days": health_days,
        }))
    }

    fn update_settings(&self, patch: &SettingsPatch) -> Result<StatusView, BackendError> {
        let mut status = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(days) = patch.sync_days {
            status.sync_days = days;
        }
        if let Some(auto) = patch.auto_sync {
            status.auto_sync = auto;
        }
        if let Some(minutes) = patch.sync_interval_minutes {
            status.sync_interval_minutes = minutes;
        }
        if let Some(route) = &patch.network_route {
            status.network_route = route.clone();
        }
        if let Some(node) = &patch.exit_node {
            status.exit_node = Some(node.clone());
        }
        if let Some(demo) = patch.demo_visible {
            status.demo_visible = demo;
        }
        Ok(status.clone())
    }

    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        Ok(self
            .gear
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone())
    }

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        if api_key.len() < 20 {
            return Err(BackendError::Domain(
                "The API key is not valid.".to_string(),
            ));
        }
        {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            status.garmin_connected = true;
            status.garmin_method = Some("personal".to_string());
            status.has_personal_api_key = true;
        }
        Ok(json!({
            "ok": true,
            "garmin_method": "personal",
            "athlete": {"name": "Athlete"},
            "sources": {},
        }))
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        if api_key.len() < 20 {
            return Err(BackendError::Domain(
                "The API key is not valid.".to_string(),
            ));
        }
        Ok(json!({"ok": true, "athlete": {"name": "Athlete"}}))
    }

    fn tile_policy(&self) -> Result<Value, BackendError> {
        Ok(self.derived("tile_policy").unwrap_or_else(|| {
            json!({
                "upstream": "openstreetmap.org",
                "proxied": true,
                "reason": "Tiles are fetched by this server and cached, so the browser never reveals the athlete's IP address to a third party.",
                "user_agent": "Stride/1.0 (private running log; https://github.com/local/stride)",
                "cache_dir": "/tmp/tile-cache",
            })
        }))
    }

    fn tile(&self, z: i64, x: i64, y: i64) -> Result<Tile, BackendError> {
        let key = format!("tile:{z}:{x}:{y}");
        match self.derived(&key) {
            Some(Value::String(encoded)) => {
                let bytes = crate::util::base64_decode(&encoded)
                    .ok_or_else(|| BackendError::Domain("the tile is not base64".to_string()))?;
                Ok(Tile {
                    bytes,
                    source: "cache",
                })
            }
            _ => Err(BackendError::Domain("Map tile unavailable.".to_string())),
        }
    }

    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, BackendError> {
        let key = format!("garmin_oauth:{action}");
        if let Some(value) = self.derived(&key) {
            return Ok(value);
        }
        if action == "logout" {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            status.garmin_connected = false;
            status.garmin_method = None;
            status.has_personal_api_key = false;
            return Ok(json!({"ok": true}));
        }
        let _ = body;
        Err(BackendError::Unsupported(format!(
            "the Garmin {action} flow is not implemented by this backend"
        )))
    }

    fn import_fit(&self, filename: &str, _bytes: &[u8]) -> Result<Value, BackendError> {
        Err(BackendError::Domain(format!(
            "the FIT file {filename} could not be parsed"
        )))
    }
}
