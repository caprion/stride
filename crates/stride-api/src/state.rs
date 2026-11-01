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
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, SecondsFormat, Utc};
use stride_ingest::{GarminClient, HttpTransport, ReqwestTransport, TransportError};
use stride_store::Store;

use crate::error::ApiError;

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_prefix: "/api/v1".to_string(),
            feed_page_size: 50,
            max_feed_page: 200,
            sync_days: 30,
            history_months: 12,
            health_history_days: 90,
            stale_after_hours: 36,
            max_fit_import_bytes: 20 * 1024 * 1024,
            max_samples_in_response: 1_000,
            utc_offset_minutes: 0,
        }
    }
}

/// The shared state every handler reads.
pub struct Api {
    store: Arc<Store>,
    config: Config,
    clock: Arc<dyn Clock>,
    garmin: Mutex<GarminState>,
    transport: Arc<dyn HttpTransport>,
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
        Self {
            store,
            config,
            clock,
            garmin: Mutex::new(GarminState::default()),
            transport,
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

    /// The clock.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// The transport the Garmin client uses.
    pub fn transport(&self) -> &Arc<dyn HttpTransport> {
        &self.transport
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

    /// The last Garmin failure, when there was one.
    pub fn garmin_last_error(&self) -> Option<String> {
        self.garmin().last_error.clone()
    }

    /// Note a Garmin failure, so the status route can show it.
    pub fn set_garmin_last_error(&self, error: Option<String>) {
        self.garmin().last_error = error;
    }

    /// A Garmin client for the key that is held, if there is one.
    pub fn garmin_client(&self) -> Option<GarminClient> {
        let state = self.garmin();
        state
            .key
            .as_ref()
            .map(|key| key.client(self.transport.clone()))
    }

    /// The key that is held, for the routes that test it.
    ///
    /// The key is copied into the client and nowhere else: the caller gets
    /// a client, and the key stays inside the state.
    pub fn garmin_client_for(&self, key: &str) -> GarminClient {
        GarminKey::new(key).client(self.transport.clone())
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
                ApiError::garmin_rejected("Garmin is rate limiting the request")
            }
            GarminError::Transport(_) => {
                ApiError::garmin_rejected("Garmin could not be reached")
            }
            _ => ApiError::garmin_rejected("Garmin refused the request"),
        }
    }
}

/// An instant as the responses carry it: `2026-05-12T06:30:00Z`.
pub fn format_instant(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// A date as the responses carry it: `2026-05-12`.
pub fn format_date(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}
