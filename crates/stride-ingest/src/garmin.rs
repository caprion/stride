//! Garmin Connect Personal API client.
//!
//! One long-lived Personal API key is exchanged for a short-lived bearer
//! value. The key is supplied by the caller, stays in memory, and is never
//! written, printed, or placed in a URL. A `401` causes exactly one key
//! re-exchange and one request retry — never a loop.

use std::fmt;
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use reqwest::blocking::Client as HttpClient;
use reqwest::{Method, Url};
use serde_json::Value;
use zip::ZipArchive;

/// Garmin's Personal API host.
pub const DEFAULT_BASE_URL: &str = "https://personal.connectapi.garmin.com";

/// The Personal API key exchange endpoint.
pub const TOKEN_PATH: &str = "/personalapitoken-service/personal-api/personal-api-key/token";

/// The activity search endpoint.
pub const ACTIVITIES_PATH: &str = "/activitylist-service/activities/search/activities";

/// The activity detail endpoint.
pub const ACTIVITY_PATH: &str = "/activity-service/activity";

/// The activity FIT download endpoint.
pub const DOWNLOAD_PATH: &str = "/download-service/files/activity";

/// The sleep daily endpoint.
pub const SLEEP_DAILY_PATH: &str = "/sleep-service/sleep/dailySleep";

/// The sleep summary endpoint.
pub const SLEEP_SUMMARY_PATH: &str = "/sleep-service/sleep/dailySleepSummariesByDate";

/// The daily HRV endpoint.
pub const HRV_DAILY_PATH: &str = "/hrv-service/hrv";

/// The daily HRV range endpoint.
pub const HRV_RANGE_PATH: &str = "/hrv-service/hrv/daily";

/// The daily wellness heart-rate endpoint.
pub const DAILY_HEART_RATE_PATH: &str = "/wellness-service/wellness/dailyHeartRate";

/// The body-battery endpoint.
pub const BODY_BATTERY_PATH: &str = "/wellness-service/wellness/bodyBattery/reports/daily";

/// The user daily-summary endpoint.
pub const USER_SUMMARY_PATH: &str = "/usersummary-service/usersummary/daily";

/// The daily stress-series endpoint.
pub const STRESS_PATH: &str = "/usersummary-service/stats/stress/daily";

/// The daily steps-series endpoint.
pub const STEPS_PATH: &str = "/usersummary-service/stats/steps/daily";

/// The heart-rate zone endpoint.
pub const HEART_RATE_ZONES_PATH: &str = "/biometric-service/heartRateZones/v2";

/// The profile endpoint.
pub const PROFILE_PATH: &str = "/userprofile-service/userprofile/settings";

/// The largest response body this client accepts: 5 MiB.
pub const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// The largest number of members in a downloaded activity archive.
pub const MAX_ZIP_ENTRIES: usize = 8;

/// The largest expanded size of one downloaded archive member.
pub const MAX_FIT_MEMBER_BYTES: u64 = 20 * 1024 * 1024;

/// The largest compressed size of one downloaded archive member.
pub const MAX_FIT_COMPRESSED_BYTES: u64 = 5 * 1024 * 1024;

const GARMIN_HOSTS: [&str; 3] = [
    "personal.connectapi.garmin.com",
    "connectapi.garmin.com",
    "connect.garmin.com",
];
const MAX_REDIRECTS: usize = 6;
const MAX_ACTIVITIES: usize = 300;
const MAX_PAGE_SIZE: usize = 100;
const MAX_RANGE_DAYS: i64 = 365;
const DEFAULT_BEARER_TTL: Duration = Duration::from_secs(23 * 60 * 60);
const DEFAULT_REFRESH_MARGIN: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The failures the Garmin client can report.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GarminError {
    /// Garmin rejected the Personal API key.
    #[error("Garmin rejected the Personal API key (HTTP {status})")]
    InvalidKey {
        /// HTTP status returned by the key exchange.
        status: u16,
    },
    /// The key exchange failed for another reason.
    #[error("the Garmin key exchange failed (HTTP {status})")]
    KeyExchangeFailed {
        /// HTTP status returned by the key exchange.
        status: u16,
    },
    /// The key exchange returned no bearer value.
    #[error("Garmin returned an empty access value")]
    EmptyAccessValue,
    /// A request remained unauthorized after the one permitted retry.
    #[error("the Garmin session expired; a new Personal API key is required")]
    NeedsReconnect,
    /// Garmin denied the request's scope.
    #[error("Garmin refused the request (HTTP 403): the key lacks the required scope")]
    Forbidden,
    /// Garmin is rate limiting the request.
    #[error("Garmin is rate limiting the request (HTTP 429)")]
    RateLimited {
        /// The `Retry-After` value, when Garmin sent one.
        retry_after_secs: Option<u64>,
    },
    /// The request did not reach Garmin.
    #[error("could not reach Garmin: {0}")]
    Transport(String),
    /// Garmin returned an error status.
    #[error("Garmin returned HTTP {status} for {path}: {body}")]
    Http {
        /// HTTP status returned by Garmin.
        status: u16,
        /// Endpoint path, without its query string.
        path: String,
        /// A short, credential-redacted response body.
        body: String,
    },
    /// The response was larger than the 5 MiB ceiling.
    #[error("the Garmin response for {path} exceeds the 5 MiB limit")]
    ResponseTooLarge {
        /// Endpoint path, without its query string.
        path: String,
    },
    /// Garmin sent a redirect without a usable location.
    #[error("Garmin returned a redirect without a location for {path}")]
    Redirect {
        /// Endpoint path, without its query string.
        path: String,
    },
    /// Too many redirects were returned.
    #[error("too many redirects while requesting {path}")]
    TooManyRedirects {
        /// Endpoint path, without its query string.
        path: String,
    },
    /// The request tried to leave Garmin's allowed hosts.
    #[error("refusing to send Garmin credentials to {host}")]
    UntrustedHost {
        /// The host that was refused.
        host: String,
    },
    /// The activity id is not a Garmin activity id.
    #[error("{0:?} is not a valid Garmin activity id")]
    InvalidActivityId(String),
    /// A date is not an ISO `YYYY-MM-DD` date.
    #[error("{0:?} is not a valid date; expected YYYY-MM-DD")]
    InvalidDate(String),
    /// The activity download did not contain a usable FIT file.
    #[error("no usable FIT file in the download for activity {activity_id}")]
    FitUnavailable {
        /// Garmin activity id whose download was unusable.
        activity_id: String,
    },
}

/// A transport-level failure.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct TransportError {
    message: String,
}

impl TransportError {
    /// Build a transport failure.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// The HTTP methods this client uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// A read request.
    Get,
    /// The key-exchange request.
    Post,
}

impl HttpMethod {
    fn as_reqwest(self) -> Method {
        match self {
            Self::Get => Method::GET,
            Self::Post => Method::POST,
        }
    }
}

/// One HTTP request handed to the transport.
pub struct HttpRequest {
    /// Request method.
    pub method: HttpMethod,
    /// Fully built request URL.
    pub url: Url,
    /// Authorization header value, when the request has one.
    pub auth_value: Option<String>,
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("has_auth_value", &self.auth_value.is_some())
            .finish()
    }
}

/// One HTTP response returned by the transport.
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: Vec<(String, String)>,
    /// Response body.
    pub body: Vec<u8>,
}

impl fmt::Debug for HttpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body_bytes", &self.body.len())
            .finish()
    }
}

/// The HTTP layer used by [`GarminClient`].
///
/// The production implementation is [`ReqwestTransport`]. Tests and other
/// embedders can supply their own implementation without opening a socket.
pub trait HttpTransport: Send + Sync {
    /// Execute one request without following redirects.
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError>;
}

/// The production blocking HTTP transport.
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: HttpClient,
}

impl ReqwestTransport {
    /// Build a transport with bounded timeouts and no automatic redirects.
    pub fn new() -> Result<Self, TransportError> {
        let client = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("stride-ingest/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| TransportError::new(error.to_string()))?;
        Ok(Self { client })
    }
}

impl HttpTransport for ReqwestTransport {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let mut builder = self
            .client
            .request(request.method.as_reqwest(), request.url.clone());
        if let Some(value) = request.auth_value.as_deref() {
            builder = builder.header(reqwest::header::AUTHORIZATION, value);
        }
        let response = builder
            .send()
            .map_err(|error| TransportError::new(error.to_string()))?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
            })
            .collect();
        if let Some(length) = response.content_length() {
            if length > MAX_RESPONSE_BYTES as u64 {
                return Err(TransportError::new("response exceeds the 5 MiB limit"));
            }
        }
        let mut body = Vec::new();
        response
            .take(MAX_RESPONSE_BYTES as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|error| TransportError::new(error.to_string()))?;
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(TransportError::new("response exceeds the 5 MiB limit"));
        }
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[derive(Clone)]
struct CachedAccess {
    value: String,
    expires_at: Instant,
}

#[derive(Default)]
struct ClientState {
    access: Option<CachedAccess>,
    exchange_count: usize,
    request_count: usize,
}

/// The Garmin Connect Personal API client.
///
/// The client is meant to live for the duration of a sync. It caches the
/// short-lived access value in memory and refreshes it once, without ever
/// retrying in a loop.
pub struct GarminClient {
    personal_key: String,
    transport: Arc<dyn HttpTransport>,
    state: Mutex<ClientState>,
    access_ttl: Duration,
    refresh_margin: Duration,
}

impl fmt::Debug for GarminClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GarminClient")
            .field("base_url", &DEFAULT_BASE_URL)
            .field("has_personal_key", &!self.personal_key.is_empty())
            .field("access_cached", &self.lock().access.is_some())
            .finish()
    }
}

impl GarminClient {
    /// Build a client for Garmin's Personal API.
    ///
    /// The key is held only in this process. It is not logged, persisted, or
    /// returned by any method.
    pub fn new(personal_key: impl Into<String>) -> Result<Self, TransportError> {
        let transport = Arc::new(ReqwestTransport::new()?);
        Ok(Self::with_transport(personal_key, transport))
    }

    /// Build a client with a caller-supplied transport.
    pub fn with_transport(
        personal_key: impl Into<String>,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            personal_key: personal_key.into(),
            transport,
            state: Mutex::new(ClientState::default()),
            access_ttl: DEFAULT_BEARER_TTL,
            refresh_margin: DEFAULT_REFRESH_MARGIN,
        }
    }

    /// Set how long an exchanged access value is treated as valid.
    pub fn with_access_ttl(mut self, ttl: Duration) -> Self {
        self.access_ttl = ttl;
        self
    }

    /// Set how long before expiry the access value is refreshed.
    pub fn with_refresh_margin(mut self, margin: Duration) -> Self {
        self.refresh_margin = margin;
        self
    }

    /// Forget the cached access value.
    pub fn invalidate_access(&self) {
        self.lock().access = None;
    }

    /// Whether an access value is cached.
    pub fn has_cached_access(&self) -> bool {
        self.lock().access.is_some()
    }

    /// The number of Personal API key exchanges this client has made.
    pub fn exchange_count(&self) -> usize {
        self.lock().exchange_count
    }

    /// The number of HTTP requests this client has made, key exchanges included.
    pub fn request_count(&self) -> usize {
        self.lock().request_count
    }

    /// Search for activities.
    ///
    /// `start` is a zero-based page offset and `limit` is the page size.
    /// Both are bounded to Garmin's 300-activity search window.
    pub fn activity_search(
        &self,
        start: u32,
        limit: u32,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Value, GarminError> {
        let start = start.min((MAX_ACTIVITIES - 1) as u32);
        let limit = (limit as usize)
            .clamp(1, MAX_PAGE_SIZE)
            .min(MAX_ACTIVITIES - start as usize);
        let mut params = vec![
            ("start".to_owned(), start.to_string()),
            ("limit".to_owned(), limit.to_string()),
        ];
        if start_date.is_some() || end_date.is_some() {
            let (from, to) = bounded_range(optional_date(start_date)?, optional_date(end_date)?);
            params.push(("startDate".to_owned(), from.to_string()));
            params.push(("endDate".to_owned(), to.to_string()));
        }
        self.get_value(ACTIVITIES_PATH, &params)
    }

    /// Alias for [`GarminClient::activity_search`].
    pub fn activities(
        &self,
        start: u32,
        limit: u32,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Value, GarminError> {
        self.activity_search(start, limit, start_date, end_date)
    }

    /// One activity's detail payload.
    pub fn activity_detail(&self, activity_id: &str) -> Result<Value, GarminError> {
        let id = check_activity_id(activity_id)?;
        self.get_value(&format!("{ACTIVITY_PATH}/{id}"), &[])
    }

    /// The original FIT file for one activity.
    ///
    /// Garmin normally serves the file in a zip archive. The archive is
    /// checked before any member is read: path-traversal names, too many entries and
    /// implausible sizes are all rejected.
    pub fn activity_fit(&self, activity_id: &str) -> Result<Vec<u8>, GarminError> {
        let id = check_activity_id(activity_id)?;
        let blob = self.get_bytes(&format!("{DOWNLOAD_PATH}/{id}"))?;
        extract_fit(&blob, &id)
    }

    /// One night's sleep payload.
    pub fn sleep_daily(&self, date: &str) -> Result<Value, GarminError> {
        let date = check_date(date)?;
        self.get_value(SLEEP_DAILY_PATH, &[("date".to_owned(), date)])
    }

    /// Alias for [`GarminClient::sleep_daily`].
    pub fn sleep(&self, date: &str) -> Result<Value, GarminError> {
        self.sleep_daily(date)
    }

    /// The sleep-summary payload beginning on `date`.
    pub fn sleep_summary(&self, date: &str) -> Result<Value, GarminError> {
        let date = check_date(date)?;
        self.get_value(SLEEP_SUMMARY_PATH, &[("date".to_owned(), date)])
    }

    /// Alias for [`GarminClient::sleep_summary`].
    pub fn sleep_range(&self, date: &str) -> Result<Value, GarminError> {
        self.sleep_summary(date)
    }

    /// One day's HRV payload.
    pub fn hrv_daily(&self, date: &str) -> Result<Value, GarminError> {
        let date = check_date(date)?;
        self.get_value(&format!("{HRV_DAILY_PATH}/{date}"), &[])
    }

    /// Alias for [`GarminClient::hrv_daily`].
    pub fn hrv(&self, date: &str) -> Result<Value, GarminError> {
        self.hrv_daily(date)
    }

    /// A range of daily HRV summaries.
    pub fn hrv_range(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (from, to) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_value(&format!("{HRV_RANGE_PATH}/{from}/{to}"), &[])
    }

    /// The daily wellness heart-rate payload.
    pub fn wellness_daily_heart_rate(&self, date: &str) -> Result<Value, GarminError> {
        let date = check_date(date)?;
        self.get_value(DAILY_HEART_RATE_PATH, &[("date".to_owned(), date)])
    }

    /// Alias for [`GarminClient::wellness_daily_heart_rate`].
    pub fn daily_heart_rate(&self, date: &str) -> Result<Value, GarminError> {
        self.wellness_daily_heart_rate(date)
    }

    /// The body-battery reports for a date range.
    pub fn body_battery(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (from, to) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_value(
            BODY_BATTERY_PATH,
            &[
                ("startDate".to_owned(), from.to_string()),
                ("endDate".to_owned(), to.to_string()),
            ],
        )
    }

    /// The user summary for one day.
    pub fn user_summary_daily(&self, date: &str) -> Result<Value, GarminError> {
        let date = check_date(date)?;
        self.get_value(USER_SUMMARY_PATH, &[("calendarDate".to_owned(), date)])
    }

    /// Alias for [`GarminClient::user_summary_daily`].
    pub fn daily_summary(&self, date: &str) -> Result<Value, GarminError> {
        self.user_summary_daily(date)
    }

    /// The daily stress series for a date range.
    pub fn stress(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (from, to) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_value(&format!("{STRESS_PATH}/{from}/{to}"), &[])
    }

    /// The daily steps series for a date range.
    pub fn steps(&self, from: &str, to: &str) -> Result<Value, GarminError> {
        let (from, to) = bounded_range(Some(parse_date(from)?), Some(parse_date(to)?));
        self.get_value(&format!("{STEPS_PATH}/{from}/{to}"), &[])
    }

    /// The athlete's heart-rate zone model.
    pub fn heart_rate_zones(&self) -> Result<Value, GarminError> {
        self.get_value(HEART_RATE_ZONES_PATH, &[])
    }

    /// The athlete's Garmin profile.
    pub fn profile(&self) -> Result<Value, GarminError> {
        self.get_value(PROFILE_PATH, &[])
    }

    fn get_value(&self, path: &str, params: &[(String, String)]) -> Result<Value, GarminError> {
        let access = self.access_value(false)?;
        match self.get_value_with(&access, path, params) {
            Ok(value) => Ok(value),
            Err(RequestFailure::Unauthorized) => {
                let fresh = self.access_value(true)?;
                match self.get_value_with(&fresh, path, params) {
                    Ok(value) => Ok(value),
                    Err(RequestFailure::Unauthorized) => Err(GarminError::NeedsReconnect),
                    Err(failure) => Err(failure.into_error(path)),
                }
            }
            Err(failure) => Err(failure.into_error(path)),
        }
    }

    fn get_value_with(
        &self,
        access: &str,
        path: &str,
        params: &[(String, String)],
    ) -> Result<Value, RequestFailure> {
        let fetched = self.fetch(
            HttpMethod::Get,
            path,
            params,
            Some(&format!("Bearer {access}")),
        )?;
        match fetched.status {
            401 => Err(RequestFailure::Unauthorized),
            403 => Err(RequestFailure::Forbidden),
            429 => Err(RequestFailure::RateLimited(fetched.retry_after_secs)),
            status if status >= 400 => Err(RequestFailure::Status(
                status,
                self.redacted_body(&fetched.body),
            )),
            204 => Ok(Value::Null),
            _ if fetched.body.is_empty() => Ok(Value::Null),
            _ => Ok(serde_json::from_slice(&fetched.body).unwrap_or_else(|_| {
                Value::String(String::from_utf8_lossy(&fetched.body).into_owned())
            })),
        }
    }

    fn get_bytes(&self, path: &str) -> Result<Vec<u8>, GarminError> {
        let access = self.access_value(false)?;
        match self.get_bytes_with(&access, path) {
            Ok(bytes) => Ok(bytes),
            Err(RequestFailure::Unauthorized) => {
                let fresh = self.access_value(true)?;
                match self.get_bytes_with(&fresh, path) {
                    Ok(bytes) => Ok(bytes),
                    Err(RequestFailure::Unauthorized) => Err(GarminError::NeedsReconnect),
                    Err(failure) => Err(failure.into_error(path)),
                }
            }
            Err(failure) => Err(failure.into_error(path)),
        }
    }

    fn get_bytes_with(&self, access: &str, path: &str) -> Result<Vec<u8>, RequestFailure> {
        let fetched = self.fetch(
            HttpMethod::Get,
            path,
            &[],
            Some(&format!("Bearer {access}")),
        )?;
        match fetched.status {
            401 => Err(RequestFailure::Unauthorized),
            403 => Err(RequestFailure::Forbidden),
            429 => Err(RequestFailure::RateLimited(fetched.retry_after_secs)),
            status if status >= 400 => Err(RequestFailure::Status(
                status,
                self.redacted_body(&fetched.body),
            )),
            _ => Ok(fetched.body),
        }
    }

    fn access_value(&self, force: bool) -> Result<String, GarminError> {
        if !force {
            let state = self.lock();
            if let Some(cached) = state.access.as_ref() {
                if cached
                    .expires_at
                    .checked_duration_since(Instant::now())
                    .unwrap_or_default()
                    > self.refresh_margin
                {
                    return Ok(cached.value.clone());
                }
            }
        }
        self.exchange_key()
    }

    fn exchange_key(&self) -> Result<String, GarminError> {
        let auth_value = format!("ApiKey {}", self.personal_key);
        let fetched = self.fetch(HttpMethod::Post, TOKEN_PATH, &[], Some(&auth_value))?;
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
            status => {
                return Err(GarminError::KeyExchangeFailed { status });
            }
        }
        let value = parse_access_value(&fetched.body)?;
        let mut state = self.lock();
        state.access = Some(CachedAccess {
            value: value.clone(),
            expires_at: Instant::now() + self.access_ttl,
        });
        state.exchange_count += 1;
        Ok(value)
    }

    fn fetch(
        &self,
        method: HttpMethod,
        path: &str,
        params: &[(String, String)],
        auth_value: Option<&str>,
    ) -> Result<Fetched, GarminError> {
        let mut url = build_url(path, params)?;
        let mut header = auth_value.map(str::to_owned);
        for _ in 0..MAX_REDIRECTS {
            self.lock().request_count += 1;
            let request = HttpRequest {
                method,
                url: url.clone(),
                auth_value: header.clone(),
            };
            let response = self
                .transport
                .execute(&request)
                .map_err(|error| GarminError::Transport(self.redact(&error.to_string())))?;
            if response.body.len() > MAX_RESPONSE_BYTES {
                return Err(GarminError::ResponseTooLarge {
                    path: path.to_owned(),
                });
            }
            let status = response.status;
            if (300..400).contains(&status) {
                let location = response
                    .headers
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("location"))
                    .map(|(_, value)| value.as_str());
                let Some(location) = location else {
                    return Err(GarminError::Redirect {
                        path: path.to_owned(),
                    });
                };
                let destination = url.join(location).map_err(|_| GarminError::Redirect {
                    path: path.to_owned(),
                })?;
                ensure_allowed_url(&destination)?;
                if destination.host_str() != url.host_str() {
                    header = None;
                }
                url = destination;
                continue;
            }
            let retry_after_secs = response
                .headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("retry-after"))
                .and_then(|(_, value)| value.trim().parse::<u64>().ok());
            return Ok(Fetched {
                status,
                body: response.body,
                retry_after_secs,
            });
        }
        Err(GarminError::TooManyRedirects {
            path: path.to_owned(),
        })
    }

    fn redacted_body(&self, body: &[u8]) -> String {
        let text = String::from_utf8_lossy(body);
        self.redact(&text).chars().take(200).collect()
    }

    fn redact(&self, text: &str) -> String {
        let mut output = text.to_owned();
        if !self.personal_key.is_empty() {
            output = output.replace(&self.personal_key, "[redacted]");
        }
        if let Some(cached) = self.lock().access.as_ref() {
            if !cached.value.is_empty() {
                output = output.replace(&cached.value, "[redacted]");
            }
        }
        output
    }

    fn lock(&self) -> MutexGuard<'_, ClientState> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

struct Fetched {
    status: u16,
    body: Vec<u8>,
    retry_after_secs: Option<u64>,
}

enum RequestFailure {
    Unauthorized,
    Forbidden,
    RateLimited(Option<u64>),
    Status(u16, String),
    Other(GarminError),
}

impl From<GarminError> for RequestFailure {
    fn from(error: GarminError) -> Self {
        Self::Other(error)
    }
}

impl RequestFailure {
    fn into_error(self, path: &str) -> GarminError {
        match self {
            Self::Unauthorized => GarminError::NeedsReconnect,
            Self::Forbidden => GarminError::Forbidden,
            Self::RateLimited(retry_after_secs) => GarminError::RateLimited { retry_after_secs },
            Self::Status(status, body) => GarminError::Http {
                status,
                path: path.to_owned(),
                body,
            },
            Self::Other(error) => error,
        }
    }
}

/// Extract the original FIT file from Garmin's activity download.
///
/// The archive is accepted only when it has a bounded number of entries,
/// every member name is safe, and the selected member expands within the
/// size limit. A bare FIT file is accepted as-is.
pub fn extract_fit(blob: &[u8], activity_id: &str) -> Result<Vec<u8>, GarminError> {
    let unavailable = || GarminError::FitUnavailable {
        activity_id: activity_id.to_owned(),
    };
    if blob.starts_with(b"PK") {
        let mut archive = ZipArchive::new(Cursor::new(blob)).map_err(|_| unavailable())?;
        if archive.len() > MAX_ZIP_ENTRIES {
            return Err(unavailable());
        }
        let mut fit_index = None;
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|_| unavailable())?;
            let name = entry.name();
            if unsafe_member_name(name) {
                return Err(unavailable());
            }
            if entry.size() > MAX_FIT_MEMBER_BYTES
                || entry.compressed_size() > MAX_FIT_COMPRESSED_BYTES
            {
                return Err(unavailable());
            }
            if fit_index.is_none() && name.to_ascii_lowercase().ends_with(".fit") {
                fit_index = Some(index);
            }
        }
        let Some(index) = fit_index else {
            return Err(unavailable());
        };
        let mut entry = archive.by_index(index).map_err(|_| unavailable())?;
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(MAX_FIT_MEMBER_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| unavailable())?;
        if bytes.len() as u64 > MAX_FIT_MEMBER_BYTES {
            return Err(unavailable());
        }
        return Ok(bytes);
    }
    if blob.len() as u64 <= MAX_FIT_MEMBER_BYTES && looks_like_fit(blob) {
        return Ok(blob.to_vec());
    }
    Err(unavailable())
}

fn looks_like_fit(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && (bytes[0] == 12 || bytes[0] == 14) && &bytes[8..12] == b".FIT"
}

fn unsafe_member_name(name: &str) -> bool {
    if name.starts_with('/') || name.starts_with('\\') {
        return true;
    }
    let normalized = name.replace('\\', "/");
    if normalized.split('/').any(|part| part.contains("..")) {
        return true;
    }
    let bytes = name.as_bytes();
    bytes.len() > 1 && bytes[1] == b':'
}

fn parse_access_value(body: &[u8]) -> Result<String, GarminError> {
    let text = std::str::from_utf8(body)
        .map_err(|_| GarminError::EmptyAccessValue)?
        .trim();
    let value = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text)
        .trim();
    if value.is_empty()
        || value.len() > 16_384
        || value.chars().any(char::is_whitespace)
        || value.contains('{')
        || value.contains('}')
    {
        return Err(GarminError::EmptyAccessValue);
    }
    Ok(value.to_owned())
}

fn build_url(path: &str, params: &[(String, String)]) -> Result<Url, GarminError> {
    let mut url = Url::parse(&format!("{DEFAULT_BASE_URL}{path}")).map_err(|_| {
        GarminError::UntrustedHost {
            host: DEFAULT_BASE_URL.to_owned(),
        }
    })?;
    ensure_allowed_url(&url)?;
    if !params.is_empty() {
        let mut query = url.query_pairs_mut();
        for (name, value) in params {
            query.append_pair(name, value);
        }
    }
    Ok(url)
}

fn ensure_allowed_url(url: &Url) -> Result<(), GarminError> {
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https" || !GARMIN_HOSTS.contains(&host) {
        return Err(GarminError::UntrustedHost {
            host: host.to_owned(),
        });
    }
    Ok(())
}

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

fn check_date(value: &str) -> Result<String, GarminError> {
    parse_date(value).map(|date| date.to_string())
}

fn optional_date(value: Option<&str>) -> Result<Option<NaiveDate>, GarminError> {
    value.map(parse_date).transpose()
}

fn parse_date(value: &str) -> Result<NaiveDate, GarminError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| GarminError::InvalidDate(value.to_owned()))
}

fn bounded_range(start: Option<NaiveDate>, end: Option<NaiveDate>) -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    let end_day = end.unwrap_or(today);
    let mut start_day = start.unwrap_or_else(|| end_day - ChronoDuration::days(MAX_RANGE_DAYS));
    if start_day > end_day {
        start_day = end_day;
    }
    if (end_day - start_day).num_days() > MAX_RANGE_DAYS {
        start_day = end_day - ChronoDuration::days(MAX_RANGE_DAYS);
    }
    (start_day, end_day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_names_cannot_escape() {
        assert!(unsafe_member_name("../escape.fit"));
        assert!(unsafe_member_name("/absolute.fit"));
        assert!(unsafe_member_name("C:\\escape.fit"));
        assert!(unsafe_member_name("folder\\..\\escape.fit"));
        assert!(!unsafe_member_name("42_ACTIVITY.fit"));
    }

    #[test]
    fn access_values_are_bare_text() {
        assert_eq!(
            parse_access_value(b"abc.def.ghi").unwrap_or_default(),
            "abc.def.ghi"
        );
        assert_eq!(
            parse_access_value(b"\"abc.def.ghi\"").unwrap_or_default(),
            "abc.def.ghi"
        );
        assert!(parse_access_value(b"  ").is_err());
    }
}
