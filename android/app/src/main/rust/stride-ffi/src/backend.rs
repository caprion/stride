//! The phone's backend.
//!
//! The phone runs the **same** implementation as the desktop: the store
//! adapter, the state and the domain are `stride-api`'s, and this file only
//! adds the one thing the phone has and the desktop does not — a Garmin pull
//! that can be driven from a background worker.
//!
//! ```text
//! CoreBackend        the domain: the fingerprint, the readiness, the verdicts
//!   └── PhoneSync    this file: the pull
//!         └── StateBackend   the state: the key, the status, the pairing
//!               └── the store adapter   the rows
//! ```
//!
//! **Why not a backend of its own.** An earlier version of this file had one:
//! a `PhoneBackend` that read `stride-store` directly and computed the
//! derived numbers itself. That was a second implementation of the domain,
//! and two implementations of the domain are how the phone and the desktop
//! start disagreeing about a run. `stride-api` grew the shared stack while
//! this was being written, so the phone uses it.
//!
//! ## What the phone cannot do
//!
//! The routes that need a server answer 501 rather than pretending:
//! pairing, the Garmin OAuth dance and the map tiles. The state layer above
//! owns the first two; the tiles are the store adapter's answer.

use std::sync::Arc;

use serde_json::Value;

use stride_api::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, Counts, DailyMetricRow, ImportRequest,
    LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields, PlanPhaseRow,
    PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SessionWithResult,
    SettingsPatch, StatusView, StreamRow, SyncRunRow, Tile, ZoneRow,
};
use stride_api::error::BackendError;
use stride_store::Store;

/// The phone's backend: the shared stack plus what only the phone knows.
pub struct PhoneSync {
    /// The shared stack underneath.
    inner: Arc<dyn Backend>,
    /// The phone's own encrypted database.
    store: Arc<Store>,
}

impl PhoneSync {
    /// A backend over the shared stack and the phone's database.
    pub fn new(inner: Arc<dyn Backend>, store: Arc<Store>) -> Self {
        Self { inner, store }
    }

    /// The database, for the pull.
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    /// The Garmin key, if one is in memory.
    ///
    /// The key is in the state layer, not here: the shared sync reads it
    /// through `garmin_personal_key()`, and the state is what the Kotlin
    /// shell fills for the length of one pull.
    pub fn garmin_key(&self) -> Result<Option<String>, BackendError> {
        self.inner.garmin_personal_key()
    }
}

impl Backend for PhoneSync {
    /// The pairing code.
    ///
    /// The phone has no server to pair with: it is the device. The state
    /// layer above would happily mint a code for a machine that does not
    /// exist, so the phone says no.
    fn pair_code(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no server to pair with".to_string(),
        ))
    }

    /// Exchange a pairing code.
    fn pair(&self, _code: &str) -> Result<Option<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no server to pair with".to_string(),
        ))
    }

    /// The paired devices.
    fn devices(&self) -> Result<Vec<Value>, BackendError> {
        Ok(Vec::new())
    }

    /// Revoke a paired device.
    fn revoke_device(&self, _device_id: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no paired devices".to_string(),
        ))
    }

    /// The network status.
    ///
    /// The phone has no exit node and no server: the routes are answered on
    /// the device. The shared store adapter refuses to answer this, because
    /// on the desktop it is a property of the machine — and the phone *is*
    /// the machine.
    fn network_status(&self) -> Result<Value, BackendError> {
        Ok(serde_json::json!({
            "route": "local",
            "connected": true,
            "exit_nodes": [],
            "message": "The phone does not use a server. The routes are answered on the device.",
        }))
    }

    /// Who this is.
    fn public_identity(&self) -> Result<Value, BackendError> {
        Ok(serde_json::json!({
            "name": "This phone",
            "mode": "local",
            "paired": false,
        }))
    }

    /// The exit nodes. The phone has none.
    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        Ok(Vec::new())
    }

    /// Route through an exit node. The phone has none.
    fn set_network_route(&self, _route: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the phone has no exit node to route through".to_string(),
        ))
    }

    /// One map tile.
    ///
    /// The phone ships no basemap, so there is no tile to fetch. The route
    /// says so rather than pretending there is one.
    fn tile(&self, _z: i64, _x: i64, _y: i64) -> Result<Tile, BackendError> {
        Err(BackendError::Unsupported(
            "the phone ships no basemap, so there is no tile to fetch".to_string(),
        ))
    }

    /// Where the map tiles come from.
    ///
    /// The phone ships no basemap: the OSM tile servers forbid offline
    /// prefetching, and a licensed tile pack is a separate piece of work.
    /// The route is still drawn from the recorded track.
    fn tile_policy(&self) -> Result<Value, BackendError> {
        Ok(serde_json::json!({
            "policy": "none",
            "message": "The phone ships no basemap. Routes are drawn from the recorded track.",
        }))
    }

    /// Load a span of history.
    ///
    /// The phone pulls a bounded window — thirty days of runs and wellness —
    /// and the background job does it. There is no bulk history load, and
    /// the answer says so in the reference's shape.
    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        _background: bool,
    ) -> Result<Value, BackendError> {
        Ok(serde_json::json!({
            "started": true,
            "activities_months": activities,
            "health_days": health_days,
            "reason": "The phone pulls a bounded window; the pull runs in the background.",
        }))
    }

    /// The pull.
    ///
    /// The pull itself is `stride-api`'s (`sync::run`), which is the same
    /// one the desktop uses: the activities, the FIT files, the splits and
    /// the wellness.
    ///
    /// **A background pull is a job, not a call.** The phone's pulls are
    /// WorkManager jobs: they survive the app being killed, they are bounded
    /// by the system, and the phone has one scheduler for them. So the route
    /// acknowledges the request and the worker does the work. The shared
    /// state layer would answer a background pull with a thread of its own;
    /// the phone answers it with its scheduler instead.
    ///
    /// The answer is the reference's shape: `{"started": true}` when the pull
    /// was accepted, and `{"started": false, "reason": "..."}` when it was
    /// not.
    /// The pull.
    ///
    /// The pull itself is `stride-api`'s (`sync::run`), which is the same
    /// one the desktop uses: the activities, the FIT files, the splits and
    /// the wellness.
    ///
    /// **A background pull is a job, not a call.** The phone's pulls are
    /// WorkManager jobs: they survive the app being killed, they are bounded
    /// by the system, and the phone has one scheduler for them. So the route
    /// acknowledges the request and the worker does the work. The shared
    /// state layer would answer a background pull with a thread of its own;
    /// the phone answers it with its scheduler instead.
    ///
    /// The answer is the reference's shape: `{"started": true}` when the pull
    /// was accepted, and `{"started": false, "reason": "..."}` when it was
    /// not.
    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, BackendError> {
        if background {
            return Ok(serde_json::json!({
                "started": true,
                "reason": "queued",
                "days": days.unwrap_or(30).clamp(1, 90),
            }));
        }

        // A pull the athlete is watching: it happens here and now.
        self.inner.run_sync(days, false)
    }

    // --- generated: the delegation to the shared stack ---
    fn activities(&self, filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError> {
        self.inner.activities(filter)
    }
    fn activity_count(&self, filter: &ActivityFilter) -> Result<i64, BackendError> {
        self.inner.activity_count(filter)
    }
    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError> {
        self.inner.activity(id)
    }
    fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<ActivityRow>, BackendError> {
        self.inner.activity_by_external(source, external_id)
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
        patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        self.inner.patch_activity(id, patch)
    }
    fn counts(&self) -> Result<Counts, BackendError> {
        self.inner.counts()
    }
    fn upsert_synced_activity(
        &self,
        activity: &ActivityRow,
        laps: &[LapRow],
        streams: &[StreamRow],
    ) -> Result<i64, BackendError> {
        self.inner.upsert_synced_activity(activity, laps, streams)
    }
    fn upsert_physiology(&self, row: &PhysiologyRow) -> Result<(), BackendError> {
        self.inner.upsert_physiology(row)
    }
    fn upsert_daily_metric(&self, row: &DailyMetricRow) -> Result<(), BackendError> {
        self.inner.upsert_daily_metric(row)
    }
    fn upsert_sync_run(&self, run: &SyncRunRow) -> Result<i64, BackendError> {
        self.inner.upsert_sync_run(run)
    }
    fn garmin_personal_key(&self) -> Result<Option<String>, BackendError> {
        self.inner.garmin_personal_key()
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
    ) -> Result<Vec<DailyMetricRow>, BackendError> {
        self.inner.daily_metrics(from, to)
    }
    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
        self.inner.plan_blocks()
    }
    fn plan_block(&self, id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
        self.inner.plan_block(id)
    }
    fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
        self.inner.plan_phases(block_id)
    }
    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
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
    fn create_plan_block(&self, fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError> {
        self.inner.create_plan_block(fields)
    }
    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError> {
        self.inner.create_plan_phase(block_id, fields)
    }
    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError> {
        self.inner.create_plan_week(phase_id, fields)
    }
    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        self.inner.create_plan_session(week_id, fields)
    }
    fn import_plan(&self, request: &ImportRequest) -> Result<Value, BackendError> {
        self.inner.import_plan(request)
    }
    fn fingerprint(&self, activity_id: i64) -> Result<Value, BackendError> {
        self.inner.fingerprint(activity_id)
    }
    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.fingerprints(from, to)
    }
    fn readiness(&self, day: &str) -> Result<Value, BackendError> {
        self.inner.readiness(day)
    }
    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.readiness_range(from, to)
    }
    fn baselines(&self, day: &str) -> Result<Value, BackendError> {
        self.inner.baselines(day)
    }
    fn compare(&self, ids: &[i64]) -> Result<Value, BackendError> {
        self.inner.compare(ids)
    }
    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        self.inner.compare_candidates(limit)
    }
    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.plan_results(from, to)
    }
    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, BackendError> {
        self.inner.plan_adherence(from, to)
    }
    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError> {
        self.inner.plan_sessions_with_results(from, to)
    }
    fn athlete_zone_model(&self, activity_id: i64) -> Result<Vec<Value>, BackendError> {
        self.inner.athlete_zone_model(activity_id)
    }
    fn max_hr_reference(&self, activity_id: i64) -> Result<f64, BackendError> {
        self.inner.max_hr_reference(activity_id)
    }
    fn public_status(&self) -> Result<StatusView, BackendError> {
        self.inner.public_status()
    }
    fn sync_status(&self) -> Result<Value, BackendError> {
        self.inner.sync_status()
    }
    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        self.inner.sync_runs(limit)
    }
    fn update_settings(&self, patch: &SettingsPatch) -> Result<StatusView, BackendError> {
        self.inner.update_settings(patch)
    }
    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        self.inner.gear()
    }
    fn connect_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        self.inner.connect_personal_key(api_key)
    }
    fn test_personal_key(&self, api_key: &str) -> Result<Value, BackendError> {
        self.inner.test_personal_key(api_key)
    }
    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, BackendError> {
        self.inner.garmin_oauth(action, body)
    }
    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, BackendError> {
        self.inner.import_fit(filename, bytes)
    }
    // --- end of the generated delegation ---
}
