//! The Garmin Connect **Personal API** client.
//!
//! Garmin's Personal API needs no developer registration and no OAuth dance:
//! one long-lived Personal API Key is exchanged for a bearer token that is
//! valid for 24 hours. There are no refresh tokens — "refreshing" means
//! exchanging the key again.
//!
//! The client follows that lifecycle exactly:
//!
//! * the token is cached with its expiry and reused until it is close to
//!   expiring;
//! * a `401` triggers **one** re-exchange and **one** retry;
//! * a second `401` is [`GarminError::NeedsReconnect`] — the athlete has to
//!   supply a new key;
//! * a `403` is a scope failure and is never retried;
//! * nothing ever loops.
//!
//! # The key
//!
//! The Personal API key is a credential for the athlete's whole Garmin
//! account. It is passed in by the caller — the Android Keystore on the
//! phone, Key Vault on the desktop — and this crate never reads it from the
//! environment, never writes it to disk, never puts it in a URL and never
//! puts it in an error message or a `Debug` output.
//!
//! The reference implementation is `backend/app/ingest/garmin_personal.py`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, NaiveDate, Utc};
use reqwest::blocking::Client as HttpClient;
use reqwest::header::{AUTHORIZATION, LOCATION, RETRY_AFTER};
use reqwest::{Method, Url};
use serde_json::Value;

use crate::error::GarminError;
use crate::fit::extract_fit;
use crate::types::{ActivityDetail, ActivitySummary, Profile, ZoneModel};

/// Garmin's Personal API host.
pub const DEFAULT_BASE_URL: &str = "https://personal.connectapi.garmin.com";

/// The path of the token exchange.
pub const TOKEN_PATH: &str = "/personalapitoken-service/personal-api/personal-api-key/token";

/// The activity search endpoint.
pub const ACTIVITIES_PATH: &str = "/activitylist-service/activities/search/activities";

/// The activity detail endpoint.
pub const ACTIVITY_PATH: &str = "/activity-service/activity";

/// The activity download endpoint.
pub const DOWNLOAD_PATH: &str = "/download-service/files/activity";

/// The profile endpoint.
pub const PROFILE_PATH: &str = "/userprofile-service/userprofile/settings";

/// The heart-rate zone endpoint.
pub const HEART_RATE_ZONES_PATH: &str = "/biometric-service/heartRateZones/v2";

/// The hosts Garmin's credentials may be sent to.
const GARMIN_HOSTS: [&str; 3] = [
    "personal.connectapi.garmin.com",
    "connectapi.garmin.com",
    "connect.garmin.com",
];

/// The loopback hosts a local stub may be served from.
const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "::1"];

/// The most redirects the client will follow.
const MAX_REDIRECTS: usize = 6;

/// The most bytes a single response may carry.
const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// The most activities one window may yield.
const MAX_ACTIVITIES: usize = 300;

/// The page size of the activity search.
const PAGE_SIZE: usize = 100;

/// The longest window, in days, one request may span.
const MAX_RANGE_DAYS: i64 = 365;

/// How long a token is treated as valid. Garmin issues 24 hours; the client
/// exchanges a minute early, as the reference implementation does.
const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(23 * 3600);

/// How long before the expiry a token is refreshed.
const DEFAULT_REFRESH_MARGIN: Duration = Duration::from_secs(60);

/// The connect timeout.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// The request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

struct TokenCache {
    token: String,
    expires_at: SystemTime,
}

#[derive(Default)]
struct State {
    token: Option<TokenCache>,
    seen: BTreeSet<String>,
}

/// The Garmin Connect Personal API client.
///
/// The client is cheap to clone-free but is meant to be long-lived: it holds
/// the token cache. It is `Send` and `Sync`, so the phone can share one
/// between its sync worker and its UI.
pub struct GarminClient {
    key: String,
    base_url: String,
    http: HttpClient,
    state: Mutex<State>,
    token_ttl: Duration,
    refresh_margin: Duration,
}

impl fmt::Debug for GarminClient {
    /// Prints the client without the Personal API key.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GarminClient")
            .field("base_url", &self.base_url)
            .field("has_key", &!self.key.is_empty())
            .field("token_cached", &self.lock().token.is_some())
            .finish()
    }
}

impl GarminClient {
    /// A client for Garmin's Personal API.
    ///
    /// The key is the athlete's Personal API Key. It is kept in memory, it is
    /// never written to disk, and it never appears in an error message.
    pub fn new(key: impl Into<String>) -> Self {
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("stride-garmin/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();
        Self {
            key: key.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            http,
            state: Mutex::new(State::default()),
            token_ttl: DEFAULT_TOKEN_TTL,
            refresh_margin: DEFAULT_REFRESH_MARGIN,
        }
    }

    /// A client pointed at another base URL.
    ///
    /// The host must be one of Garmin's, or a loopback host — the tests use
    /// this to talk to a local stub, and it is the only way the credentials
    /// could ever leave Garmin's own machines.
    pub fn with_base_url(key: impl Into<String>, base_url: &str) -> Result<Self, GarminError> {
        let trimmed = base_url.trim_end_matches('/');
        let parsed = Url::parse(trimmed).map_err(|_| GarminError::UntrustedHost {
            host: trimmed.to_owned(),
        })?;
        let host = parsed.host_str().unwrap_or_default().to_owned();
        if !host_allowed(parsed.scheme(), &host) {
            return Err(GarminError::UntrustedHost { host });
        }
        let mut client = Self::new(key);
        client.base_url = trimmed.to_owned();
        Ok(client)
    }

    /// Sets how long an exchanged token is treated as valid.
    ///
    /// The default is 23 hours, one hour short of Garmin's 24. This exists so
    /// the tests can simulate an expiring token without waiting a day.
    pub fn with_token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = ttl;
        self
    }

    /// Sets how long before its expiry a token is refreshed.
    pub fn with_refresh_margin(mut self, margin: Duration) -> Self {
        self.refresh_margin = margin;
        self
    }

    /// Forgets the cached token, so the next call exchanges the key again.
    pub fn invalidate_token(&self) {
        let mut state = self.lock();
        state.token = None;
    }

    /// Whether a usable token is cached.
    pub fn has_cached_token(&self) -> bool {
        self.lock().token.is_some()
    }

    // ------------------------------------------------------------- lifecycle

    /// Proves the key works: exchanges it, then reads the profile.
    ///
    /// The ordering matters. The caller must not store a key that has not been
    /// proven, and a profile read that fails after a successful token exchange
    /// is a distinct failure from a rejected key.
    pub fn validate(&self) -> Result<Profile, GarminError> {
        self.access_token(true)?;
        self.profile()
    }

    /// The athlete's profile.
    pub fn profile(&self) -> Result<Profile, GarminError> {
        let payload = self.get_json(PROFILE_PATH, &[])?;
        Ok(Profile::from_value(&payload))
    }

    /// The athlete's heart-rate zone model.
    pub fn heart_rate_zones(&self) -> Result<ZoneModel, GarminError> {
        let payload = self.get_json(HEART_RATE_ZONES_PATH, &[])?;
        Ok(ZoneModel::from_value(&payload))
    }

    // ------------------------------------------------------------ activities

    /// The activities in a window, newest first.
    ///
    /// The window is at most 365 days and at most 300 activities are
    /// returned, which is what the reference implementation's sync loop
    /// fetches.
    pub fn activities(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<ActivitySummary>, GarminError> {
        let (start, end) = bounded_range(Some(from.date_naive()), Some(to.date_naive()));
        let start_date = start.to_string();
        let end_date = end.to_string();
        let mut out: Vec<ActivitySummary> = Vec::new();
        let mut offset = 0usize;
        while offset < MAX_ACTIVITIES {
            let limit = PAGE_SIZE.min(MAX_ACTIVITIES - offset);
            let page = self.activity_page(offset as u32, limit as u32, &start_date, &end_date)?;
            if page.is_empty() {
                break;
            }
            let count = page.len();
            out.extend(page);
            offset += count;
            if count < limit {
                break;
            }
        }
        Ok(out)
    }

    /// One page of the activity search.
    ///
    /// `start` is the offset and `limit` the page size; both are clamped the
    /// way the reference implementation clamps them.
    pub fn activity_page(
        &self,
        start: u32,
        limit: u32,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ActivitySummary>, GarminError> {
        let start = start.min(299);
        let limit = limit.clamp(1, 100).min(300 - start);
        let mut params = vec![
            ("start".to_owned(), start.to_string()),
            ("limit".to_owned(), limit.to_string()),
        ];
        if !start_date.is_empty() {
            params.push(("startDate".to_owned(), start_date.to_owned()));
        }
        if !end_date.is_empty() {
            params.push(("endDate".to_owned(), end_date.to_owned()));
        }
        let payload = self.get_json(ACTIVITIES_PATH, &params)?;
        Ok(as_list(
            &payload,
            &["activityList", "activities", "activitySummaries"],
        )
        .into_iter()
        .filter_map(|item| ActivitySummary::from_value(&item))
        .take(limit as usize)
        .collect())
    }

    /// One activity's detail, with its summary and its splits.
    pub fn activity_detail(&self, activity_id: &str) -> Result<Value, GarminError> {
        let id = check_activity_id(activity_id)?;
        self.get_json(&format!("{ACTIVITY_PATH}/{id}"), &[])
    }

    /// One activity's detail, canonicalised the way the reference
    /// implementation canonicalises it.
    ///
    /// This is the same payload [`GarminClient::activity_detail`] returns, put
    /// through [`ActivityDetail::from_value`], so the numbers are the numbers
    /// the Python reference produces.
    pub fn activity_detail_canonical(
        &self,
        activity_id: &str,
    ) -> Result<ActivityDetail, GarminError> {
        let payload = self.activity_detail(activity_id)?;
        ActivityDetail::from_value(&payload, None)
            .ok_or(GarminError::InvalidActivityId(activity_id.to_owned()))
    }

    /// The workouts associated with an activity.
    pub fn activity_workouts(&self, activity_id: &str) -> Result<Vec<Value>, GarminError> {
        let id = check_activity_id(activity_id)?;
        let payload = self.get_json(&format!("{ACTIVITY_PATH}/{id}/workouts"), &[])?;
        Ok(as_list(
            &payload,
            &["workouts", "activityWorkouts", "workoutSummaries"],
        ))
    }

    /// The activity's original FIT file.
    ///
    /// This is the full-fidelity route: per-second heart rate, cadence and
    /// GPS, which the summary endpoints do not carry.
    pub fn activity_fit(&self, activity_id: &str) -> Result<Vec<u8>, GarminError> {
        let id = check_activity_id(activity_id)?;
        let bytes = self.get_bytes(&format!("{DOWNLOAD_PATH}/{id}"))?;
        extract_fit(&bytes, &id)
    }

    // -------------------------------------------------------------- wellness

    /// One night's sleep.
    pub fn sleep(&self, day: &str) -> Result<Value, GarminError> {
        let date = check_date(day)?;
        self.get_json(
            "/sleep-service/sleep/dailySleep",
            &[("date".to_owned(), date)],
        )
    }

    /// A range of sleep summaries.
    pub fn sleep_range(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let (start, _end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        let payload = self.get_json(
            "/sleep-service/sleep/dailySleepSummariesByDate",
            &[("date".to_owned(), start.to_string())],
        )?;
        Ok(as_list(
            &payload,
            &["sleepSummaries", "dailySleepSummaries"],
        ))
    }

    /// One day's HRV.
    pub fn hrv(&self, day: &str) -> Result<Value, GarminError> {
        let date = check_date(day)?;
        self.get_json(&format!("/hrv-service/hrv/{date}"), &[])
    }

    /// A range of daily HRV summaries.
    pub fn hrv_range(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let (start, end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        let payload = self.get_json(&format!("/hrv-service/hrv/daily/{start}/{end}"), &[])?;
        Ok(as_list(
            &payload,
            &["hrvDailySummaries", "hrvSummaries", "hrvList"],
        ))
    }

    /// The day's resting heart rate and its curve.
    pub fn daily_heart_rate(&self, day: &str) -> Result<Value, GarminError> {
        let date = check_date(day)?;
        self.get_json(
            "/wellness-service/wellness/dailyHeartRate",
            &[("date".to_owned(), date)],
        )
    }

    /// The body's battery, day by day.
    pub fn body_battery(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (start, end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_json(
            "/wellness-service/wellness/bodyBattery/reports/daily",
            &[
                ("startDate".to_owned(), start.to_string()),
                ("endDate".to_owned(), end.to_string()),
            ],
        )
    }

    /// The body's battery, day by day, without the envelope.
    ///
    /// Garmin wraps this list in `bodyBatteryReports`, `bodyBatteryList` or
    /// `reports`; the reference implementation unwraps it with `_as_list` and
    /// this does the same.
    pub fn body_battery_list(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let payload = self.body_battery(from, to)?;
        Ok(as_list(
            &payload,
            &["bodyBatteryReports", "bodyBatteryList", "reports"],
        ))
    }

    /// The day's summary: resting heart rate, stress, body battery, steps.
    pub fn daily_summary(&self, day: &str) -> Result<Value, GarminError> {
        let date = check_date(day)?;
        self.get_json(
            "/usersummary-service/usersummary/daily",
            &[("calendarDate".to_owned(), date)],
        )
    }

    /// The daily stress series.
    pub fn stress(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (start, end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_json(
            &format!("/usersummary-service/stats/stress/daily/{start}/{end}"),
            &[],
        )
    }

    /// The daily stress series, without the envelope.
    pub fn stress_list(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let payload = self.stress(from, to)?;
        Ok(as_list(
            &payload,
            &["stressValuesArray", "stressDailyList", "stressList"],
        ))
    }

    /// The daily steps series.
    pub fn steps(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (start, end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_json(
            &format!("/usersummary-service/stats/steps/daily/{start}/{end}"),
            &[],
        )
    }

    /// The daily steps series, without the envelope.
    pub fn steps_list(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let payload = self.steps(from, to)?;
        Ok(as_list(
            &payload,
            &["stepsDailyList", "stepsList", "stepsValuesArray"],
        ))
    }

    /// The weight series.
    pub fn weight(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (start, end) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_json(&format!("/weight-service/weight/all/{start}/{end}"), &[])
    }

    /// The weight series, without the envelope.
    pub fn weight_list(&self, from: &str, to: &str) -> Result<Vec<Value>, GarminError> {
        let payload = self.weight(from, to)?;
        Ok(as_list(
            &payload,
            &["dateWeightList", "weightList", "weights"],
        ))
    }

    /// The fitness age for a day.
    pub fn fitness_age(&self, day: &str) -> Result<Value, GarminError> {
        let date = check_date(day)?;
        self.get_json(&format!("/fitnessage-service/fitnessage/v2/{date}"), &[])
    }

    // ---------------------------------------------------------------- status

    /// Which endpoint families have answered at least once.
    ///
    /// Reports from what has been fetched: it never calls Garmin, because a
    /// status check must not be able to rate-limit the sync. The keys are the
    /// ones the API's connect response has always carried.
    pub fn sources(&self) -> BTreeMap<&'static str, bool> {
        let state = self.lock();
        let seen = &state.seen;
        let mut out = BTreeMap::new();
        for (name, service) in [
            ("activities", "activitylist-service"),
            ("activity_detail", "activity-service"),
            ("sleep", "sleep-service"),
            ("hrv", "hrv-service"),
            ("daily_heart_rate", "wellness-service"),
            ("body_battery", "wellness-service"),
            ("daily_summary", "usersummary-service"),
            ("stress", "usersummary-service"),
            ("steps", "usersummary-service"),
            ("heart_rate_zones", "biometric-service"),
            ("weight", "weight-service"),
            ("profile", "userprofile-service"),
        ] {
            out.insert(name, seen.contains(service));
        }
        out
    }

    // ------------------------------------------------------------- plumbing

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Exchanges the key for a token, or returns the cached one.
    fn access_token(&self, force: bool) -> Result<String, GarminError> {
        if !force {
            let state = self.lock();
            if let Some(cache) = state.token.as_ref() {
                if let Ok(remaining) = cache.expires_at.duration_since(SystemTime::now()) {
                    if remaining > self.refresh_margin {
                        return Ok(cache.token.clone());
                    }
                }
            }
        }
        self.exchange_token()
    }

    fn exchange_token(&self) -> Result<String, GarminError> {
        let authorization = format!("ApiKey {}", self.key);
        let fetched = self.fetch(&Method::POST, TOKEN_PATH, &[], Some(authorization.as_str()))?;
        match fetched.status {
            200..=299 => {}
            401 | 403 => {
                return Err(GarminError::InvalidKey {
                    status: fetched.status,
                })
            }
            429 => {
                return Err(GarminError::RateLimited {
                    retry_after_secs: fetched.retry_after_secs,
                })
            }
            status => return Err(GarminError::TokenRequestFailed { status }),
        }
        let token = token_from_body(&fetched.bytes);
        if token.is_empty() {
            return Err(GarminError::EmptyToken);
        }
        let expires_at = SystemTime::now() + self.token_ttl;
        let mut state = self.lock();
        state.token = Some(TokenCache {
            token: token.clone(),
            expires_at,
        });
        state.seen.insert("access_token".to_owned());
        Ok(token)
    }

    /// A bearer-authenticated GET, with one re-exchange and one retry on a
    /// `401` — and never more than that.
    fn get_json(&self, path: &str, params: &[(String, String)]) -> Result<Value, GarminError> {
        let token = self.access_token(false)?;
        match self.get_json_with(&token, path, params) {
            Ok(value) => {
                self.mark_seen(path);
                Ok(value)
            }
            Err(FetchFailure::Unauthorized) => {
                let fresh = self.access_token(true)?;
                match self.get_json_with(&fresh, path, params) {
                    Ok(value) => {
                        self.mark_seen(path);
                        Ok(value)
                    }
                    Err(FetchFailure::Unauthorized) => Err(GarminError::NeedsReconnect),
                    Err(failure) => Err(failure.into_error(path)),
                }
            }
            Err(failure) => Err(failure.into_error(path)),
        }
    }

    fn get_json_with(
        &self,
        token: &str,
        path: &str,
        params: &[(String, String)],
    ) -> Result<Value, FetchFailure> {
        let authorization = format!("Bearer {token}");
        let fetched = self.fetch(&Method::GET, path, params, Some(authorization.as_str()))?;
        match fetched.status {
            401 => Err(FetchFailure::Unauthorized),
            403 => Err(FetchFailure::Forbidden),
            429 => Err(FetchFailure::RateLimited(fetched.retry_after_secs)),
            status if status >= 400 => Err(FetchFailure::Status(
                status,
                fetched.body_text(&[self.key.as_str()]),
            )),
            204 => Ok(Value::Null),
            _ => {
                if fetched.bytes.is_empty() {
                    Ok(Value::Null)
                } else {
                    match serde_json::from_slice::<Value>(&fetched.bytes) {
                        Ok(value) => Ok(value),
                        Err(_) => Ok(Value::String(
                            String::from_utf8_lossy(&fetched.bytes).into_owned(),
                        )),
                    }
                }
            }
        }
    }

    /// A bearer-authenticated GET for bytes, with the same 401 handling.
    fn get_bytes(&self, path: &str) -> Result<Vec<u8>, GarminError> {
        let token = self.access_token(false)?;
        match self.get_bytes_with(&token, path) {
            Ok(bytes) => {
                self.mark_seen(path);
                Ok(bytes)
            }
            Err(FetchFailure::Unauthorized) => {
                let fresh = self.access_token(true)?;
                match self.get_bytes_with(&fresh, path) {
                    Ok(bytes) => {
                        self.mark_seen(path);
                        Ok(bytes)
                    }
                    Err(FetchFailure::Unauthorized) => Err(GarminError::NeedsReconnect),
                    Err(failure) => Err(failure.into_error(path)),
                }
            }
            Err(failure) => Err(failure.into_error(path)),
        }
    }

    fn get_bytes_with(&self, token: &str, path: &str) -> Result<Vec<u8>, FetchFailure> {
        let authorization = format!("Bearer {token}");
        let fetched = self.fetch(&Method::GET, path, &[], Some(authorization.as_str()))?;
        match fetched.status {
            401 => Err(FetchFailure::Unauthorized),
            403 => Err(FetchFailure::Forbidden),
            429 => Err(FetchFailure::RateLimited(fetched.retry_after_secs)),
            status if status >= 400 => Err(FetchFailure::Status(
                status,
                fetched.body_text(&[self.key.as_str()]),
            )),
            _ => Ok(fetched.bytes),
        }
    }

    /// One HTTP request, with the redirects followed by hand.
    ///
    /// The redirects are followed by hand because the reference implementation
    /// does: an authorization header is dropped when the redirect leaves the
    /// host it was meant for, and a redirect to a host that is not Garmin is
    /// refused outright.
    fn fetch(
        &self,
        method: &Method,
        path: &str,
        params: &[(String, String)],
        authorization: Option<&str>,
    ) -> Result<Fetched, GarminError> {
        let mut url = self.build_url(path, params)?;
        let mut header = authorization.map(str::to_owned);
        for _ in 0..MAX_REDIRECTS {
            let mut request = self.http.request(method.clone(), url.clone());
            if let Some(value) = header.as_ref() {
                request = request.header(AUTHORIZATION, value);
            }
            let response = request
                .send()
                .map_err(|error| GarminError::Transport(self.scrub(&error.to_string())))?;
            let status = response.status().as_u16();
            if (300..400).contains(&status) {
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                let location = match location {
                    Some(location) => location,
                    None => {
                        return Err(GarminError::Redirect {
                            path: path.to_owned(),
                        })
                    }
                };
                let destination = url.join(&location).map_err(|_| GarminError::Redirect {
                    path: path.to_owned(),
                })?;
                let host = destination.host_str().unwrap_or_default();
                if !host_allowed(destination.scheme(), host) {
                    return Err(GarminError::UntrustedHost {
                        host: host.to_owned(),
                    });
                }
                if destination.host_str() != url.host_str() {
                    header = None;
                }
                url = destination;
                continue;
            }
            let retry_after_secs = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<u64>().ok());
            let mut bytes = Vec::new();
            response
                .take(MAX_RESPONSE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| GarminError::Transport(self.scrub(&error.to_string())))?;
            if bytes.len() > MAX_RESPONSE_BYTES {
                return Err(GarminError::ResponseTooLarge {
                    path: path.to_owned(),
                });
            }
            return Ok(Fetched {
                status,
                bytes,
                retry_after_secs,
            });
        }
        Err(GarminError::TooManyRedirects {
            path: path.to_owned(),
        })
    }

    fn build_url(&self, path: &str, params: &[(String, String)]) -> Result<Url, GarminError> {
        let mut url = Url::parse(&format!("{}{}", self.base_url, path)).map_err(|_| {
            GarminError::UntrustedHost {
                host: self.base_url.clone(),
            }
        })?;
        if !params.is_empty() {
            let mut query = url.query_pairs_mut();
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }
        Ok(url)
    }

    fn mark_seen(&self, path: &str) {
        let service = path
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or(path)
            .to_owned();
        let mut state = self.lock();
        state.seen.insert(service);
    }

    /// Removes the key from a string before it reaches an error message.
    fn scrub(&self, text: &str) -> String {
        scrub(text, &[self.key.as_str()])
    }
}

/// A response, read in full.
struct Fetched {
    status: u16,
    bytes: Vec<u8>,
    retry_after_secs: Option<u64>,
}

impl Fetched {
    /// The body as text, with the credentials removed and truncated.
    fn body_text(&self, secrets: &[&str]) -> String {
        let text = String::from_utf8_lossy(&self.bytes);
        scrub(&text, secrets).chars().take(200).collect()
    }
}

/// The ways a single attempt can fail, before the retry policy is applied.
enum FetchFailure {
    Unauthorized,
    Forbidden,
    RateLimited(Option<u64>),
    Status(u16, String),
    Other(GarminError),
}

impl From<GarminError> for FetchFailure {
    fn from(error: GarminError) -> Self {
        FetchFailure::Other(error)
    }
}

impl FetchFailure {
    fn into_error(self, path: &str) -> GarminError {
        match self {
            FetchFailure::Unauthorized => GarminError::NeedsReconnect,
            FetchFailure::Forbidden => GarminError::Forbidden,
            FetchFailure::RateLimited(retry_after_secs) => {
                GarminError::RateLimited { retry_after_secs }
            }
            FetchFailure::Status(status, body) => GarminError::Http {
                status,
                path: path.to_owned(),
                body,
            },
            FetchFailure::Other(error) => error,
        }
    }
}

/// True when credentials may be sent to this host.
///
/// Garmin's own hosts over HTTPS, or a loopback host over HTTP so that the
/// tests can point the client at a local stub. Nothing else.
fn host_allowed(scheme: &str, host: &str) -> bool {
    if scheme == "https" && GARMIN_HOSTS.contains(&host) {
        return true;
    }
    if scheme == "http" && LOOPBACK_HOSTS.contains(&host) {
        return true;
    }
    false
}

/// Removes every secret from a string.
fn scrub(text: &str, secrets: &[&str]) -> String {
    let mut out = text.to_owned();
    for secret in secrets {
        if !secret.is_empty() {
            out = out.replace(secret, "[redacted]");
        }
    }
    out
}

/// Reads the token out of the token endpoint's body.
///
/// Garmin answers with the JWT as plain text, sometimes quoted. A JSON body
/// with a `token` field is accepted too.
fn token_from_body(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(token) = value.get("token").and_then(Value::as_str) {
            return token.trim().trim_matches('"').to_owned();
        }
        if let Some(token) = value.get("access_token").and_then(Value::as_str) {
            return token.trim().trim_matches('"').to_owned();
        }
    }
    trimmed.trim_matches('"').trim().to_owned()
}

/// Pulls an array of objects out of whichever envelope Garmin used.
fn as_list(payload: &Value, keys: &[&str]) -> Vec<Value> {
    if let Some(items) = payload.as_array() {
        return items
            .iter()
            .filter(|item| item.is_object())
            .cloned()
            .collect();
    }
    let Some(object) = payload.as_object() else {
        return Vec::new();
    };
    for key in keys {
        if let Some(items) = object.get(*key).and_then(Value::as_array) {
            return items
                .iter()
                .filter(|item| item.is_object())
                .cloned()
                .collect();
        }
    }
    Vec::new()
}

/// A Garmin activity id: digits only, at most twenty of them.
fn check_activity_id(activity_id: &str) -> Result<String, GarminError> {
    let valid = !activity_id.is_empty()
        && activity_id.len() <= 20
        && activity_id.bytes().all(|byte| byte.is_ascii_digit());
    if valid {
        Ok(activity_id.to_owned())
    } else {
        Err(GarminError::InvalidActivityId(activity_id.to_owned()))
    }
}

/// Parses an ISO `YYYY-MM-DD` date.
fn parse_date(value: &str) -> Result<NaiveDate, GarminError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| GarminError::InvalidDate(value.to_owned()))
}

/// Parses and echoes an ISO `YYYY-MM-DD` date.
fn check_date(value: &str) -> Result<String, GarminError> {
    parse_date(value).map(|date| date.to_string())
}

/// Normalises a date range: at most 365 days, never inverted.
///
/// This is the reference implementation's `_bounded_date_range`.
fn bounded_range(start: Option<NaiveDate>, end: Option<NaiveDate>) -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    let end_day = end.unwrap_or(today);
    let mut start_day = start.unwrap_or_else(|| end_day - chrono::Duration::days(MAX_RANGE_DAYS));
    if start_day > end_day {
        start_day = end_day;
    }
    if (end_day - start_day).num_days() > MAX_RANGE_DAYS {
        start_day = end_day - chrono::Duration::days(MAX_RANGE_DAYS);
    }
    (start_day, end_day)
}
