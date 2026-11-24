//! The shared state: the store, the Garmin key, the configuration, the clock.
//!
//! Four things, and each of them is there for a reason.
//!
//! * **The store.** The one handle on the database. The router never opens
//!   a connection of its own, and it never writes SQL.
//! * **The Garmin key.** It goes in, and it does not come out. There is no
//!   accessor that returns it, no `Display`, and the `Debug` output is a
//!   fixed phrase. The only way it reaches Garmin is
//!   [`GarminKey::client`], which hands it to the client that needs it.
//! * **The configuration.** The window sizes and the limits. They are
//!   configuration rather than constants because the desktop and the phone
//!   want different windows.
//! * **The clock.** **Injected.** No handler ever asks what time it is;
//!   it asks the clock, and a test can hold the clock still. That is what
//!   makes `generated_at` — and every date the router derives — reproducible.

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use chrono::{DateTime, SecondsFormat, Utc};
use stride_ingest::{FitActivity, GarminClient, HttpTransport, ReqwestTransport, TransportError};
use stride_store::{DatabaseKey, Store, StoreError};

use crate::error::{ApiError, ApiResult};

/// The clock the router reads the time from.
///
/// The core is pure — it never asks what time it is — so every time-dependent
/// value is injected. The router injects this.
pub trait Clock: Send + Sync {
    /// The current instant, in UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// The clock of the machine.
///
/// This is the one place the wall clock is read. It is here, and not in a
/// handler, so that "no `Utc::now()` in the request path" is a fact about
/// the code rather than a rule someone has to remember.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock that stands still.
///
/// The tests use it so that `generated_at` — and every date the router
/// derives from it — is the same on every run.
#[derive(Clone, Copy, Debug)]
pub struct FixedClock {
    instant: DateTime<Utc>,
}

impl FixedClock {
    /// A clock that always reads the same instant.
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }
}

/// The Garmin Personal API key.
///
/// It goes in, and it does not come out.
///
/// There is no accessor that returns it, no `Display`, and the `Debug`
/// output is a fixed phrase. The only way it reaches Garmin is
/// [`GarminKey::client`], which hands a copy to the client that needs it
/// and gives nothing back: the caller gets a client, never a key.
///
/// The key is held in memory for the length of a pull and no longer. The
/// phone's copy lives in the Android Keystore and the desktop's in the
/// user's key store; this is a working copy, and it is dropped as soon as
/// the pull is over.
pub struct GarminKey {
    secret: String,
}

impl GarminKey {
    /// A key.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Whether there is a key at all.
    pub fn is_empty(&self) -> bool {
        self.secret.trim().is_empty()
    }

    /// The last four characters of the key — and never the key.
    ///
    /// The Settings screen shows an identifier the athlete can check
    /// against their own key: the last four characters, and nothing else.
    /// The key itself never leaves this type: there is no accessor for it,
    /// no `Display`, and the `Debug` output is a fixed phrase.
    pub fn last_four(&self) -> Option<String> {
        let characters: Vec<char> = self.secret.chars().collect();
        if characters.len() < 4 {
            return None;
        }
        Some(characters[characters.len() - 4..].iter().collect())
    }

    /// The client that talks to Garmin with this key.
    ///
    /// The key is copied into the client and stays there; nothing copies it
    /// back out.
    pub fn client(&self, transport: Arc<dyn HttpTransport>) -> GarminClient {
        GarminClient::with_transport(self.secret.clone(), transport)
    }
}

impl fmt::Debug for GarminKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GarminKey")
            .field("secret", &"[the Garmin key is not printed]")
            .finish()
    }
}

/// What the router knows about the Garmin connection.
#[derive(Debug, Default)]
struct GarminState {
    key: Option<GarminKey>,
    connected: bool,
    last_error: Option<String>,
}

/// The configuration the router reads.
#[derive(Clone, Debug)]
pub struct Config {
    /// The prefix the desktop puts in front of a path.
    ///
    /// The phone may be asked for either form, so the router cuts the
    /// prefix off whichever it is given.
    pub api_prefix: String,
    /// How many runs the feed answers with when the caller does not say.
    pub feed_page_size: u32,
    /// The most runs the feed will answer with.
    pub max_feed_page: u32,
    /// How many days a pull reaches back by default.
    pub sync_days: i64,
    /// How many runs one pull downloads.
    ///
    /// **A pull is bounded.** Garmin's Personal API answers a few requests
    /// a minute, and a pull that asked for a year of history would spend
    /// the whole rate limit on the first day and come back with nothing.
    /// So a pull downloads this many runs — the newest ones, which are the
    /// ones the athlete is waiting for — and the next pull takes the next
    /// ones. The runs are the same runs; they arrive over more than one
    /// pull.
    pub max_activities_per_sync: usize,
    /// How many days of wellness one pull reads.
    ///
    /// The same bound, for the same reason: the wellness of a day is three
    /// requests, and a pull that reads a year of it is a rate limit spent
    /// before anything has landed.
    pub max_wellness_days_per_sync: i64,
    /// Maximum HTTP attempts, including authentication and redirects, per job.
    pub max_sync_requests: usize,
    /// Soft response-body byte limit; a bounded in-flight response may cross it.
    pub max_sync_response_bytes: usize,
    /// Soft elapsed-time limit before starting another request.
    pub max_sync_seconds: u64,
    /// How many months of history the settings say the app keeps.
    pub history_months: i64,
    /// How many days of health data the settings say the app keeps.
    pub health_history_days: i64,
    /// How many hours a sync may be old before the data is called stale.
    pub stale_after_hours: i64,
    /// The most bytes a FIT import may carry.
    pub max_fit_import_bytes: usize,
    /// The most sample points one run's detail may carry.
    pub max_samples_in_response: usize,
    /// How far the athlete's days are from UTC, in minutes.
    ///
    /// The athlete's day is a local day, and the router only knows the
    /// offset the host gives it. Zero — UTC — is the default.
    pub utc_offset_minutes: i32,
    /// Where the export and the import live: the host's own directory.
    ///
    /// `None` on a device that has no place to write an export into — the
    /// phone, for one — and the export routes say so rather than writing
    /// somewhere the host did not choose.
    pub export_root: Option<PathBuf>,
    /// How many weeks of the athlete's own history the baseline is read
    /// from.
    ///
    /// The baseline is inferred, and the weeks it was read from are said
    /// out loud: "your usual week, from the last 8 weeks".
    pub baseline_weeks: usize,
    /// How many decoded runs the router holds in memory.
    ///
    /// **The decode cache is bounded.** A run's file is decoded on read,
    /// and a twelve-week window is about sixty runs: without a bound the
    /// router would hold every file it has ever decoded, and the phone
    /// would run out of memory. Sixty-four holds a window's worth of runs
    /// and evicts the oldest when it is full.
    pub decode_cache_capacity: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_prefix: "/api/v1".to_string(),
            feed_page_size: 50,
            max_feed_page: 200,
            sync_days: 365,
            max_activities_per_sync: 20,
            max_wellness_days_per_sync: 14,
            max_sync_requests: 80,
            max_sync_response_bytes: 32 * 1024 * 1024,
            max_sync_seconds: 45,
            history_months: 12,
            health_history_days: 90,
            stale_after_hours: 36,
            max_fit_import_bytes: 20 * 1024 * 1024,
            max_samples_in_response: 1_000,
            utc_offset_minutes: 0,
            export_root: None,
            baseline_weeks: 8,
            decode_cache_capacity: 64,
        }
    }
}

/// The shared state every handler reads.
pub struct Api {
    store: Arc<Store>,
    config: Config,
    utc_offset_minutes: AtomicI32,
    clock: Arc<dyn Clock>,
    garmin: Mutex<GarminState>,
    transport: Arc<dyn HttpTransport>,
    export: Mutex<Option<DatabaseKey>>,
    sync: Mutex<()>,
    decoded: DecodeCache,
}

impl fmt::Debug for Api {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Api")
            .field("config", &self.config)
            .field("garmin_connected", &self.garmin_connected())
            .finish_non_exhaustive()
    }
}

impl Api {
    /// The state, with the clock of the machine and the real transport.
    pub fn new(
        store: Arc<Store>,
        config: Config,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, TransportError> {
        let transport = Arc::new(ReqwestTransport::new()?);
        Ok(Self::with_transport(store, config, clock, transport))
    }

    /// The state, with a transport of the caller's choosing.
    ///
    /// The tests use this to answer Garmin without a network.
    pub fn with_transport(
        store: Arc<Store>,
        config: Config,
        clock: Arc<dyn Clock>,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        let capacity = config.decode_cache_capacity;
        Self {
            store,
            utc_offset_minutes: AtomicI32::new(config.utc_offset_minutes),
            config,
            clock,
            garmin: Mutex::new(GarminState::default()),
            transport,
            export: Mutex::new(None),
            sync: Mutex::new(()),
            decoded: DecodeCache::new(capacity),
        }
    }

    /// The store.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The decoded runs the router holds in memory.
    ///
    /// **The cache is the one place a file is decoded**, and it is
    /// bounded: the oldest decoded run is thrown away when the cache is
    /// full.
    pub fn decoded_cache(&self) -> &DecodeCache {
        &self.decoded
    }

    /// The host's current calendar offset, including daylight-saving changes.
    pub fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes.load(Ordering::Relaxed)
    }

    /// Update the host calendar without reopening the store or interrupting sync.
    pub fn set_utc_offset_minutes(&self, minutes: i32) -> ApiResult<()> {
        if !(-840..=840).contains(&minutes) {
            return Err(ApiError::bad_request(
                "the calendar offset is outside -840 to 840 minutes",
            ));
        }
        self.utc_offset_minutes.store(minutes, Ordering::Relaxed);
        Ok(())
    }

    /// The clock.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// The transport the Garmin client uses.
    pub fn transport(&self) -> &Arc<dyn HttpTransport> {
        &self.transport
    }

    pub(crate) fn begin_sync(&self) -> ApiResult<MutexGuard<'_, ()>> {
        self.sync.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => ApiError::new(
                crate::error::ErrorKind::SyncBusy,
                "a Garmin sync is already running",
            ),
            TryLockError::Poisoned(_) => {
                ApiError::internal("the sync coordinator could not be opened")
            }
        })
    }

    pub(crate) fn sync_client(
        &self,
        key: Option<&str>,
        transport: Arc<dyn HttpTransport>,
    ) -> Option<GarminClient> {
        match key {
            Some(key) => Some(GarminKey::new(key).client(transport)),
            None => self.garmin().key.as_ref().map(|key| key.client(transport)),
        }
    }

    /// The current instant, as the injected clock reads it.
    pub fn now(&self) -> DateTime<Utc> {
        self.clock.now()
    }

    /// The current instant, as the responses carry it.
    ///
    /// Every response carries one: the UI shows staleness rather than
    /// silently showing old numbers.
    pub fn generated_at(&self) -> String {
        format_instant(self.now())
    }

    /// The lock on the Garmin state.
    fn garmin(&self) -> MutexGuard<'_, GarminState> {
        self.garmin
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Hold the Garmin key, for the length of a pull.
    ///
    /// The key is never readable back out of the state: this is the only
    /// way in, and the client it builds is the only way it is used.
    pub fn set_garmin_key(&self, key: &str) -> bool {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            self.clear_garmin_key();
            return false;
        }
        let mut state = self.garmin();
        state.key = Some(GarminKey::new(trimmed));
        state.connected = true;
        state.last_error = None;
        true
    }

    /// Forget the Garmin key.
    pub fn clear_garmin_key(&self) {
        let mut state = self.garmin();
        state.key = None;
        state.connected = false;
    }

    /// Whether the phone or the desktop holds a Garmin key.
    pub fn garmin_connected(&self) -> bool {
        self.garmin().connected
    }

    /// The last four characters of the key that is held, when there is one.
    ///
    /// An identifier the athlete can check against their own key — and
    /// never the key.
    pub fn garmin_key_last4(&self) -> Option<String> {
        let state = self.garmin();
        state.key.as_ref().and_then(|key| key.last_four())
    }

    /// The last Garmin failure, when there was one.
    pub fn garmin_last_error(&self) -> Option<String> {
        self.garmin().last_error.clone()
    }

    /// Note a Garmin failure, so the status route can show it.
    pub fn set_garmin_last_error(&self, error: Option<String>) {
        self.garmin().last_error = error;
    }

    /// The key that is held, for the routes that test it.
    ///
    /// The key is copied into the client and nowhere else: the caller gets
    /// a client, and the key stays inside the state.
    pub fn garmin_client_for(&self, key: &str) -> GarminClient {
        GarminKey::new(key).client(self.transport.clone())
    }

    /// A failure of the store, as the router reports it.
    ///
    /// **A rejected database key is the lock.** The phone's key comes from
    /// the Keystore, and it is not there when the phone is locked: the
    /// failure is the typed `locked`, which the app shows as "unlock the
    /// phone" rather than as a broken app.
    ///
    /// Everything else is a failure of the router itself, and it says so.
    pub fn store_error(&self, context: &str, error: &StoreError) -> ApiError {
        match error {
            StoreError::InvalidKey { .. } => ApiError::locked("the database key is not available"),
            _ => ApiError::internal(context),
        }
    }

    /// The Garmin failure as the router reports it.
    ///
    /// The client already redacts what Garmin sent; the router adds the
    /// kind and keeps the key out of it.
    pub fn garmin_error(&self, error: &stride_ingest::GarminError) -> ApiError {
        use stride_ingest::GarminError;
        match error {
            GarminError::InvalidKey { .. } => {
                ApiError::garmin_rejected("Garmin refused the Personal API key")
            }
            GarminError::Forbidden => ApiError::garmin_rejected(
                "Garmin refused the request: the Personal API key lacks the scope",
            ),
            GarminError::NeedsReconnect => ApiError::needs_reconnect(
                "the Garmin session ended; a new Personal API key is needed",
            ),
            GarminError::RateLimited { .. } => {
                ApiError::rate_limited("Garmin is rate limiting the request")
            }
            GarminError::Transport(_) => ApiError::garmin_rejected("Garmin could not be reached"),
            GarminError::Http { status, .. } if *status >= 500 => {
                ApiError::garmin_rejected("Garmin could not be reached")
            }
            _ => ApiError::garmin_rejected("Garmin refused the request"),
        }
    }

    /// Hold the key the export and the import use.
    ///
    /// The key is an argument: the host hands it over, the routes use it
    /// for the length of the call, and it is never returned, never logged
    /// and never in an error. It is the key that opens the database, and it
    /// is the key the export is written with — so the export opens with the
    /// same key the database does.
    pub fn set_export_key(&self, key: DatabaseKey) {
        let mut state = self
            .export
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state = Some(key);
    }

    /// Whether the host gave a key for the export.
    pub fn has_export_key(&self) -> bool {
        let state = self
            .export
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.is_some()
    }

    /// Do something with the export key, if the host gave one.
    ///
    /// The key is borrowed for the length of the call and given back: it
    /// never comes out of here.
    pub fn with_export_key<T>(
        &self,
        action: impl FnOnce(&DatabaseKey) -> ApiResult<T>,
    ) -> ApiResult<T> {
        let state = self
            .export
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match state.as_ref() {
            Some(key) => action(key),
            None => Err(ApiError::bad_request(
                "this device has no key for the export",
            )),
        }
    }
}

/// One decoded run, in the cache.
struct DecodedEntry {
    /// The run's id.
    activity_id: i64,
    /// The SHA-256 of the file it was decoded from.
    sha256: String,
    /// The decoded file.
    activity: Arc<FitActivity>,
}

/// The decoded runs the router holds in memory.
///
/// **What is cached is the decode, and nothing else.** Every derived
/// series — the samples, the splits, the zones, the pace bands — is
/// recomputed on read, from the decoded file, and the cache is not a
/// place where a derived value is stored.
///
/// **The key is the run's identity and the file's own digest.** A file
/// that is re-imported changes its digest, so the entry it had is not
/// the entry it gets: a new file is never served from the old decode.
///
/// **The bound is a number of runs.** The cache holds
/// `decode_cache_capacity` decoded files — sixty-four by default, which
/// is a window's worth of runs — and throws the oldest away when it is
/// full. The lock is held for the length of one lookup and never across
/// a decode.
pub struct DecodeCache {
    /// The decoded runs, newest first.
    entries: Mutex<Vec<DecodedEntry>>,
    /// How many decoded runs the cache holds at most.
    capacity: usize,
    /// How many times a run was served from the cache.
    hits: AtomicU64,
    /// How many times a run had to be decoded.
    misses: AtomicU64,
}

/// What the cache has done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeCacheStats {
    /// How many times a run was served from the cache.
    pub hits: u64,
    /// How many times a run had to be decoded.
    pub misses: u64,
    /// How many decoded runs the cache holds now.
    pub entries: usize,
    /// How many decoded runs the cache holds at most.
    pub capacity: usize,
}

impl DecodeCache {
    /// A cache that holds `capacity` decoded runs.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            capacity,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// The decoded run, when the cache holds that file.
    ///
    /// The run's id **and** the file's digest must both match: a file
    /// that was re-imported has a new digest, and the old decode is not
    /// the new file.
    pub fn get(&self, activity_id: i64, sha256: &str) -> Option<Arc<FitActivity>> {
        let mut entries = self.lock();
        let position = entries
            .iter()
            .position(|entry| entry.activity_id == activity_id);
        let position = match position {
            Some(position) => position,
            None => {
                // The cache holds no decode of this run: the file has to
                // be decoded.
                self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        if entries[position].sha256 != sha256 {
            // The file behind this id changed: the decode that is held is
            // of a file that is not there any more.
            self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let entry = entries.remove(position);
        let activity = Arc::clone(&entry.activity);
        entries.insert(0, entry);
        self.hits.fetch_add(1, Ordering::Relaxed);
        Some(activity)
    }

    /// Keep the decoded run, and throw the oldest away when the cache is
    /// full.
    pub fn put(&self, activity_id: i64, sha256: &str, activity: Arc<FitActivity>) {
        let mut entries = self.lock();
        entries.retain(|entry| entry.activity_id != activity_id);
        entries.insert(
            0,
            DecodedEntry {
                activity_id,
                sha256: sha256.to_string(),
                activity,
            },
        );
        while entries.len() > self.capacity {
            entries.pop();
        }
    }

    /// What the cache has done.
    pub fn stats(&self) -> DecodeCacheStats {
        DecodeCacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: self.lock().len(),
            capacity: self.capacity,
        }
    }

    /// The lock on the cache.
    ///
    /// A poisoned lock is taken back rather than propagated: the cache
    /// holds decoded files and counters, and neither is worth refusing
    /// every read over.
    fn lock(&self) -> MutexGuard<'_, Vec<DecodedEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// An instant as the responses carry it: `2026-05-12T06:30:00Z`.
pub fn format_instant(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Secs, true)
}
