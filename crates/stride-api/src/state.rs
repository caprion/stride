//! The application state.
//!
//! The Garmin credentials, the pairing code, the paired devices and the sync
//! settings are **not** in the database: the Python keeps them in
//! `data/state.json`, and the phone keeps them in the Android preferences.
//! They are state, not data, and they are the state of the machine the app
//! runs on.
//!
//! This decorator owns that state and answers the routes that read it; the
//! rows still come from the store underneath it. The desktop builds it over
//! its state file, the phone over its preferences, and the tests over a plain
//! in-memory state.
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use stride_api::{Api, Backend, CoreBackend, StateBackend};
//! # fn run(store: Arc<dyn Backend>) {
//! let state = StateBackend::new(store);
//! let api = Api::new(Arc::new(CoreBackend::new(Arc::new(state))));
//! # }
//! ```
//!
//! The example takes a [`Backend`] and not a store, so that it does not
//! depend on which copy of the store crate the graph happens to hold.

use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::backend::{Backend, SettingsPatch, StatusView, SyncRunRow};
use crate::error::BackendError as Error;
use crate::network::TrustConfig;

/// The characters a pairing code is made of.
///
/// No look-alikes: no `I`, no `O`, no `0`, no `1`. The code is read aloud
/// and typed on a phone keyboard.
const PAIR_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// How long a pairing code is good for, in seconds.
pub const PAIR_CODE_TTL_SECONDS: i64 = 600;

/// The state of the machine.
#[derive(Debug, Clone)]
pub struct State {
    /// What the browser is allowed to see.
    pub status: StatusView,
    /// The Garmin Personal API key. A secret: never rendered, never logged.
    pub personal_api_key: Option<String>,
    /// The pairing code waiting to be exchanged.
    pub pairing_code: Option<String>,
    /// When that code stops working, as a Unix timestamp.
    pub pairing_expires_at: Option<f64>,
    /// The paired devices. Their tokens are never kept.
    pub devices: Vec<Value>,
    /// Whether a sync is running right now.
    pub sync_running: bool,
    /// When the last sync started.
    pub sync_started_at: Option<String>,
    /// What the last sync said.
    pub sync_message: Option<String>,
    /// Who may use the API without a pairing code.
    ///
    /// This is *deployment* configuration, not application state: it is read
    /// from the environment (`STRIDE_TRUST_REMOTE` and
    /// `STRIDE_TRUSTED_NETWORKS`) and it is never written to the state file,
    /// so a stale file cannot keep a network trusted after the operator
    /// turned it off.
    pub trust: TrustConfig,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: StatusView {
                athlete: json!({}),
                network_route: "auto".to_string(),
                sync_days: 90,
                history_months: Some(12),
                health_history_days: Some(90),
                auto_sync: true,
                sync_interval_minutes: 180,
                demo_visible: true,
                ..StatusView::default()
            },
            personal_api_key: None,
            pairing_code: None,
            pairing_expires_at: None,
            devices: Vec::new(),
            sync_running: false,
            sync_started_at: None,
            sync_message: None,
            trust: TrustConfig::default(),
        }
    }
}

/// A backend that owns the application state.
pub struct StateBackend {
    inner: Arc<dyn Backend>,
    state: Arc<Mutex<State>>,
}

impl StateBackend {
    /// Wrap a backend with a state.
    pub fn new(inner: Arc<dyn Backend>) -> Self {
        Self::with_state(inner, State::default())
    }

    /// Wrap a backend with a state the caller has already built.
    pub fn with_state(inner: Arc<dyn Backend>, state: State) -> Self {
        Self {
            inner,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// The state, for a caller that wants to look at it.
    pub fn state(&self) -> State {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Replace the state.
    pub fn set_state(&self, state: State) {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = state;
    }

    /// The state, locked.
    fn locked(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Is this address in one of the trusted networks?
    ///
    /// The trust configuration is the deployment's, and it is read from the
    /// environment. Empty means nothing extra is trusted.
    pub fn trusts(&self, address: &IpAddr) -> bool {
        self.locked().trust.trusts(address)
    }
}

/// Random bytes, from the operating system.
///
/// The pairing code and the device token are credentials, so they come from
/// `/dev/urandom` — not from a clock, not from a counter.
fn random_bytes(count: usize) -> Result<Vec<u8>, Error> {
    use std::io::Read;
    let mut file = std::fs::File::open("/dev/urandom")
        .map_err(|err| Error::Domain(format!("the random source is not readable: {err}")))?;
    let mut bytes = vec![0u8; count];
    file.read_exact(&mut bytes)
        .map_err(|err| Error::Domain(format!("the random source failed: {err}")))?;
    Ok(bytes)
}

/// A new pairing code: `XXXX-XXXX`.
fn new_pair_code() -> Result<String, Error> {
    let bytes = random_bytes(8)?;
    let mut code = String::with_capacity(9);
    for (index, byte) in bytes.iter().enumerate() {
        if index == 4 {
            code.push('-');
        }
        let letter = PAIR_ALPHABET[(*byte as usize) % PAIR_ALPHABET.len()];
        code.push(letter as char);
    }
    Ok(code)
}

/// A device token: 32 random bytes, in the URL-safe alphabet.
fn new_device_token() -> Result<String, Error> {
    let bytes = random_bytes(32)?;
    let encoded = crate::util::base64_encode(&bytes);
    Ok(encoded
        .trim_end_matches('=')
        .replace('+', "-")
        .replace('/', "_"))
}

/// The current time, as a Unix timestamp.
fn now_seconds() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs_f64())
        .unwrap_or_default()
}

/// Compare two secrets without leaking where they differ.
fn same_secret(left: &str, right: &str) -> bool {
    let left = left.trim().to_uppercase();
    let right = right.trim().to_uppercase();
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (left, right) in left.bytes().zip(right.bytes()) {
        difference |= left ^ right;
    }
    difference == 0
}

/// The athlete's name, whatever Garmin calls it.
fn athlete_name(profile: &stride_garmin::Profile) -> Value {
    let name = profile
        .display_name
        .as_ref()
        .or(profile.full_name.as_ref())
        .or(profile.user_name.as_ref())
        .or(profile.name.as_ref());
    match name {
        Some(name) => json!({ "name": name }),
        None => json!({}),
    }
}

impl StateBackend {
    /// Pull the athlete's runs and wellness out of Garmin.
    ///
    /// `days` is how far back to look for the runs and `health_days` for the
    /// wellness; `None` for the runs means the account's whole history.
    fn sync(
        &self,
        days: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, Error> {
        let key = self
            .locked()
            .personal_api_key
            .clone()
            .ok_or_else(|| Error::Domain("There is no Garmin key to sync with.".to_string()))?;
        let (source, window) = {
            let state = self.locked();
            (
                state
                    .status
                    .garmin_method
                    .clone()
                    .unwrap_or_else(|| "personal".to_string()),
                days.unwrap_or(state.status.sync_days).clamp(1, 365),
            )
        };
        let today = chrono::Utc::now().date_naive();
        let started = chrono::Utc::now();
        let started_at = started.to_rfc3339();
        {
            let mut state = self.locked();
            state.sync_running = true;
            state.sync_started_at = Some(started_at.clone());
            state.sync_message = None;
        }
        // The sync log: the row that says a sync is running, and the row the
        // history reads afterwards.
        let run_id = self
            .inner
            .upsert_sync_run(&SyncRunRow {
                id: None,
                source: source.clone(),
                started_at: Some(started_at.clone()),
                finished_at: None,
                status: "running".to_string(),
                days: window,
                ..SyncRunRow::default()
            })
            .ok();

        if background {
            // The sync runs on its own thread: the caller is told it started,
            // the way the reference answers.
            let backend = self.inner.clone();
            let state = Arc::clone(&self.state);
            let key = key.clone();
            std::thread::spawn(move || {
                let outcome = crate::sync::run(backend.as_ref(), &key, days, health_days, today);
                Self::finish_sync(
                    &backend,
                    &state,
                    run_id,
                    &source,
                    window,
                    &started_at,
                    &outcome,
                );
            });
            return Ok(json!({ "started": true }));
        }
        let outcome = crate::sync::run(self.inner.as_ref(), &key, days, health_days, today);
        Self::finish_sync(
            &self.inner,
            &self.state,
            run_id,
            &source,
            window,
            &started_at,
            &outcome,
        );
        match outcome {
            Ok(outcome) => Ok(outcome.to_value()),
            Err(err) => Err(err),
        }
    }

    /// Finish a sync: the log, the state and the message.
    ///
    /// The message is the reference's wording: the dashboard shows it, and
    /// the desktop's answer has to be the same sentence.
    fn finish_sync(
        backend: &Arc<dyn Backend>,
        state: &Arc<Mutex<State>>,
        run_id: Option<i64>,
        source: &str,
        days: i64,
        started_at: &str,
        outcome: &Result<crate::sync::SyncOutcome, Error>,
    ) -> String {
        let finished = chrono::Utc::now();
        let finished_at = finished.to_rfc3339();
        let (status, message, error, found, new, updated) = match outcome {
            Ok(outcome) => (
                "ok",
                format!(
                    "Synced {} new and {} updated activities, plus {} days of health data.",
                    outcome.activities_new, outcome.activities_updated, outcome.days
                ),
                None,
                outcome.activities_found as i64,
                outcome.activities_new as i64,
                outcome.activities_updated as i64,
            ),
            Err(err) => ("error", err.to_string(), Some(err.to_string()), 0, 0, 0),
        };
        // The log: the same row, finished.
        let _ = backend.upsert_sync_run(&SyncRunRow {
            id: run_id,
            source: source.to_string(),
            started_at: Some(started_at.to_string()),
            finished_at: Some(finished_at.clone()),
            status: status.to_string(),
            days,
            activities_found: found,
            activities_new: new,
            activities_updated: updated,
            message: Some(message.clone()),
            error,
        });
        {
            let mut guard = state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.sync_running = false;
            guard.sync_message = Some(message.clone());
            if outcome.is_ok() {
                // The dashboard's "Synced 2 days ago" reads this.
                guard.status.last_sync_at = Some(finished_at);
            }
        }
        message
    }
}

impl Backend for StateBackend {
    fn public_status(&self) -> Result<StatusView, Error> {
        let mut status = self.locked().status.clone();
        // The key is a secret: the browser learns that there is one, never
        // what it is.
        status.has_personal_api_key = self.locked().personal_api_key.is_some();
        if status.last_sync_at.is_none() {
            // The log knows when the last sync finished, even after a
            // restart: the dashboard's "Synced 2 days ago" reads it.
            if let Ok(inner) = self.inner.public_status() {
                status.last_sync_at = inner.last_sync_at;
            }
        }
        Ok(status)
    }

    fn update_settings(&self, patch: &SettingsPatch) -> Result<StatusView, Error> {
        let mut state = self.locked();
        if let Some(days) = patch.sync_days {
            state.status.sync_days = days;
        }
        if let Some(auto) = patch.auto_sync {
            state.status.auto_sync = auto;
        }
        if let Some(minutes) = patch.sync_interval_minutes {
            state.status.sync_interval_minutes = minutes;
        }
        if let Some(route) = &patch.network_route {
            state.status.network_route = route.clone();
        }
        if let Some(node) = &patch.exit_node {
            state.status.exit_node = Some(node.clone());
        }
        if let Some(demo) = patch.demo_visible {
            state.status.demo_visible = demo;
        }
        Ok(state.status.clone())
    }

    fn pair_code(&self) -> Result<Value, Error> {
        let code = new_pair_code()?;
        let expires = now_seconds() + PAIR_CODE_TTL_SECONDS as f64;
        let mut state = self.locked();
        state.pairing_code = Some(code.clone());
        state.pairing_expires_at = Some(expires);
        Ok(json!({
            "code": code,
            "expires_in": PAIR_CODE_TTL_SECONDS,
            "instructions": "Open Stride on the other device and enter this code. It works once, and expires in 10 minutes.",
        }))
    }

    /// Whether this host may mint a pairing code.
    ///
    /// The state owns the pairing, and the host owns the answer to "is the
    /// caller on this machine?". The answer is the host's: it has the peer
    /// address and the `Host` header, and the state has neither.
    fn may_mint_pairing_code(&self) -> bool {
        self.inner.may_mint_pairing_code()
    }

    fn pair(&self, code: &str) -> Result<Option<Value>, Error> {
        let mut state = self.locked();
        let expected = state.pairing_code.clone().unwrap_or_default();
        let expires = state.pairing_expires_at.unwrap_or_default();
        if expected.is_empty() || now_seconds() > expires {
            return Err(Error::Unauthorized(
                "That code is wrong, expired, or already used.".to_string(),
            ));
        }
        if !same_secret(&expected, code) {
            return Err(Error::Unauthorized(
                "That code is wrong, expired, or already used.".to_string(),
            ));
        }
        // The code is single use: the test and the clear happen together.
        state.pairing_code = None;
        state.pairing_expires_at = None;

        let token = new_device_token()?;
        let device_id = new_device_token()?.chars().take(12).collect::<String>();
        let now = now_seconds();
        let device = json!({
            "id": device_id,
            "name": "Device",
            "created_at": now,
            "last_seen_at": now,
        });
        state
            .devices
            .retain(|device| device.get("id") != Some(&json!(device_id)));
        state.devices.push(device.clone());
        // The token is handed over once and never kept: the server cannot
        // leak what it does not have.
        Ok(Some(json!({
            "ok": true,
            "device_id": device_id,
            "token": token,
        })))
    }

    fn devices(&self) -> Result<Vec<Value>, Error> {
        let state = self.locked();
        Ok(state
            .devices
            .iter()
            .map(|device| {
                json!({
                    "id": device.get("id").cloned().unwrap_or(Value::Null),
                    "name": device.get("name").cloned().unwrap_or(Value::Null),
                    "created_at": device.get("created_at").cloned().unwrap_or(Value::Null),
                    "last_seen_at": device.get("last_seen_at").cloned().unwrap_or(Value::Null),
                })
            })
            .collect())
    }

    fn revoke_device(&self, device_id: &str) -> Result<Value, Error> {
        let mut state = self.locked();
        let before = state.devices.len();
        state
            .devices
            .retain(|device| device.get("id").and_then(Value::as_str) != Some(device_id));
        if state.devices.len() == before {
            return Err(Error::Domain("No such device.".to_string()));
        }
        Ok(json!({ "ok": true, "remaining": state.devices.len() }))
    }

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        let client = stride_garmin::GarminClient::new(api_key.to_string());
        let profile = client
            .validate()
            .map_err(|err| Error::Domain(format!("the Garmin API key was refused: {err}")))?;
        let athlete = athlete_name(&profile);
        {
            let mut state = self.locked();
            state.personal_api_key = Some(api_key.to_string());
            state.status.garmin_connected = true;
            state.status.garmin_method = Some("personal".to_string());
            state.status.athlete = athlete.clone();
        }
        Ok(json!({
            "ok": true,
            "garmin_method": "personal",
            "athlete": athlete,
            "sources": {},
        }))
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        let client = stride_garmin::GarminClient::new(api_key.to_string());
        let profile = client
            .validate()
            .map_err(|err| Error::Domain(format!("the Garmin API key was refused: {err}")))?;
        Ok(json!({ "ok": true, "athlete": athlete_name(&profile) }))
    }

    fn garmin_personal_key(&self) -> Result<Option<String>, Error> {
        Ok(self.locked().personal_api_key.clone())
    }

    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, Error> {
        self.sync(days, days.map(|_| 30), background)
    }

    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, Error> {
        // The Activity API serves the account's whole history, so "everything"
        // is a real option: `None` means every day the account holds.
        let days = activities.map(|months| months * 30);
        self.sync(days, health_days, background)
    }

    fn sync_status(&self) -> Result<Value, Error> {
        let state = self.locked();
        if state.sync_running || state.sync_started_at.is_some() || state.sync_message.is_some() {
            return Ok(json!({
                "running": state.sync_running,
                "started_at": state.sync_started_at,
                "message": state.sync_message,
            }));
        }
        // This process has not synced, so the log knows: the last run's start
        // time, its message, and whether it is still running.
        self.inner.sync_status()
    }

    fn garmin_oauth(&self, action: &str, _body: &Value) -> Result<Value, Error> {
        if action == "logout" {
            let mut state = self.locked();
            state.personal_api_key = None;
            state.status.garmin_connected = false;
            state.status.garmin_method = None;
            state.status.has_personal_api_key = false;
            state.status.athlete = json!({});
            return Ok(json!({ "ok": true }));
        }
        Err(Error::Unsupported(format!(
            "the Garmin {action} flow needs the OAuth client, which this build does not have"
        )))
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, Error> {
        self.inner.import_fit(filename, bytes)
    }

    fn activities(
        &self,
        filter: &crate::backend::ActivityFilter,
    ) -> Result<Vec<crate::backend::ActivityRow>, Error> {
        self.inner.activities(filter)
    }

    fn activity_count(&self, filter: &crate::backend::ActivityFilter) -> Result<i64, Error> {
        self.inner.activity_count(filter)
    }

    fn activity(&self, id: i64) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.activity(id)
    }

    fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.activity_by_external(source, external_id)
    }

    fn upsert_synced_activity(
        &self,
        activity: &crate::backend::ActivityRow,
        laps: &[crate::backend::LapRow],
        streams: &[crate::backend::StreamRow],
    ) -> Result<i64, Error> {
        self.inner.upsert_synced_activity(activity, laps, streams)
    }

    fn upsert_physiology(&self, row: &crate::backend::PhysiologyRow) -> Result<(), Error> {
        self.inner.upsert_physiology(row)
    }

    fn upsert_daily_metric(&self, row: &crate::backend::DailyMetricRow) -> Result<(), Error> {
        self.inner.upsert_daily_metric(row)
    }

    fn upsert_sync_run(&self, run: &crate::backend::SyncRunRow) -> Result<i64, Error> {
        self.inner.upsert_sync_run(run)
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<crate::backend::LapRow>, Error> {
        self.inner.activity_laps(id)
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<crate::backend::StreamRow>, Error> {
        self.inner.activity_streams(id)
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<crate::backend::ZoneRow>, Error> {
        self.inner.activity_zones(id)
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, Error> {
        self.inner.activity_geo(id)
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &crate::backend::ActivityPatch,
    ) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.patch_activity(id, patch)
    }

    fn counts(&self) -> Result<crate::backend::Counts, Error> {
        self.inner.counts()
    }

    fn physiology(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::PhysiologyRow>, Error> {
        self.inner.physiology(from, to)
    }

    fn physiology_latest(&self) -> Result<Option<crate::backend::PhysiologyRow>, Error> {
        self.inner.physiology_latest()
    }

    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<crate::backend::DailyMetricRow>, Error> {
        self.inner.daily_metrics(from, to)
    }

    fn plan_blocks(&self) -> Result<Vec<crate::backend::PlanBlockRow>, Error> {
        self.inner.plan_blocks()
    }

    fn plan_block(&self, id: i64) -> Result<Option<crate::backend::PlanBlockRow>, Error> {
        self.inner.plan_block(id)
    }

    fn plan_phases(&self, block_id: i64) -> Result<Vec<crate::backend::PlanPhaseRow>, Error> {
        self.inner.plan_phases(block_id)
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<crate::backend::PlanWeekRow>, Error> {
        self.inner.plan_weeks(phase_id)
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<crate::backend::PlanSessionRow>, Error> {
        self.inner.plan_sessions(week_id)
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::PlanSessionRow>, Error> {
        self.inner.plan_sessions_in_range(from, to)
    }

    fn create_plan_block(
        &self,
        fields: &crate::backend::PlanBlockFields,
    ) -> Result<crate::backend::PlanBlockRow, Error> {
        self.inner.create_plan_block(fields)
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &crate::backend::PlanPhaseFields,
    ) -> Result<crate::backend::PlanPhaseRow, Error> {
        self.inner.create_plan_phase(block_id, fields)
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &crate::backend::PlanWeekFields,
    ) -> Result<crate::backend::PlanWeekRow, Error> {
        self.inner.create_plan_week(phase_id, fields)
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &crate::backend::PlanSessionFields,
    ) -> Result<crate::backend::PlanSessionRow, Error> {
        self.inner.create_plan_session(week_id, fields)
    }

    fn import_plan(&self, request: &crate::backend::ImportRequest) -> Result<Value, Error> {
        self.inner.import_plan(request)
    }

    fn fingerprint(&self, activity_id: i64) -> Result<Value, Error> {
        self.inner.fingerprint(activity_id)
    }

    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.fingerprints(from, to)
    }

    fn readiness(&self, day: &str) -> Result<Value, Error> {
        self.inner.readiness(day)
    }

    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.readiness_range(from, to)
    }

    fn baselines(&self, day: &str) -> Result<Value, Error> {
        self.inner.baselines(day)
    }

    fn compare(&self, ids: &[i64]) -> Result<Value, Error> {
        self.inner.compare(ids)
    }

    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, Error> {
        self.inner.compare_candidates(limit)
    }

    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.plan_results(from, to)
    }

    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, Error> {
        self.inner.plan_adherence(from, to)
    }

    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::SessionWithResult>, Error> {
        self.inner.plan_sessions_with_results(from, to)
    }

    fn athlete_zone_model(&self, activity_id: i64) -> Result<Vec<Value>, Error> {
        self.inner.athlete_zone_model(activity_id)
    }

    fn max_hr_reference(&self, activity_id: i64) -> Result<f64, Error> {
        self.inner.max_hr_reference(activity_id)
    }

    fn network_status(&self) -> Result<Value, Error> {
        self.inner.network_status()
    }

    fn public_identity(&self) -> Result<Value, Error> {
        self.inner.public_identity()
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, Error> {
        self.inner.exit_nodes()
    }

    fn set_network_route(&self, route: &str) -> Result<Value, Error> {
        self.inner.set_network_route(route)
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, Error> {
        self.inner.sync_runs(limit)
    }

    fn gear(&self) -> Result<Vec<Value>, Error> {
        self.inner.gear()
    }

    fn tile_policy(&self) -> Result<Value, Error> {
        self.inner.tile_policy()
    }

    fn tile(&self, z: i64, x: i64, y: i64) -> Result<crate::backend::Tile, Error> {
        self.inner.tile(z, x, y)
    }
}
