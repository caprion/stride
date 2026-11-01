//! The dispatcher: route → handler → JSON.
//!
//! One router, two hosts. The desktop serves it over loopback HTTP and the
//! phone calls it in-process through the FFI, and neither can disagree
//! about a number: there is one route table, one set of response shapes and
//! one clock.
//!
//! ## The route table is frozen
//!
//! The table below is `docs/contract/api-paths.md`, route for route. A
//! route that is not in that file is not here, and a route that is in that
//! file is here — `tests/route_parity.rs` walks the contract's own list and
//! holds this table to it.
//!
//! ## The path is matched without its query string
//!
//! The router cuts the path at the `?` and matches what is left, so
//! `/auth/garmin/personal-key?x=1` is the `/auth/garmin/personal-key`
//! route. The prefix the desktop puts in front of a path is cut off too:
//! the phone may be asked for either form, and the two forms are the same
//! route.
//!
//! ## The router has no I/O of its own
//!
//! It reads the store through the store's API and Garmin through the
//! ingest crate's client. It opens no connection, writes no SQL and keeps
//! no state of its own beyond what the shared state holds.

use serde::Serialize;
use serde_json::Value;

use crate::error::{ApiError, ApiResult, ErrorEnvelope};
use crate::state::Api;

/// The methods the router answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// A read.
    Get,
    /// A write.
    Post,
}

impl Method {
    /// The method a name means.
    pub fn parse(text: &str) -> Option<Method> {
        match text.trim().to_ascii_uppercase().as_str() {
            "GET" => Some(Method::Get),
            "POST" => Some(Method::Post),
            _ => None,
        }
    }

    /// The method's name.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
        }
    }
}

/// The query string, as the caller wrote it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pairs: Vec<(String, String)>,
}

impl Query {
    /// The query string of a path, without its `?`.
    pub fn parse(text: &str) -> Query {
        let mut pairs = Vec::new();
        for part in text.split('&') {
            if part.is_empty() {
                continue;
            }
            let (name, value) = match part.split_once('=') {
                Some((name, value)) => (name, value),
                None => (part, ""),
            };
            pairs.push((decode(name), decode(value)));
        }
        Query { pairs }
    }

    /// The first value of a name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The first value of a name, as a number.
    pub fn integer(&self, name: &str) -> ApiResult<Option<i64>> {
        match self.get(name) {
            None => Ok(None),
            Some(text) if text.trim().is_empty() => Ok(None),
            Some(text) => match text.trim().parse::<i64>() {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(ApiError::bad_request(format!("the {name} is not a number"))),
            },
        }
    }
}

/// Decode the percent-escapes of a query string.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let high = (bytes[index + 1] as char).to_digit(16);
                let low = (bytes[index + 2] as char).to_digit(16);
                match (high, low) {
                    (Some(high), Some(low)) => {
                        out.push((high * 16 + low) as u8);
                        index += 3;
                    }
                    _ => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// One request, as the router sees it.
#[derive(Clone, Debug)]
pub struct Request {
    /// The method.
    pub method: Method,
    /// The route, without the query string and without the prefix.
    pub path: String,
    /// The query string.
    pub query: Query,
    /// The values of the route's placeholders: `id` for `/activities/{id}`.
    pub params: Vec<(String, String)>,
    /// The body, already parsed.
    pub body: Option<Value>,
}

impl Request {
    /// The value of a placeholder.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The value of a placeholder, as a number.
    pub fn param_id(&self, name: &str) -> ApiResult<i64> {
        match self.param(name) {
            None => Err(ApiError::not_found("there is no such route")),
            Some(text) => match text.trim().parse::<i64>() {
                Ok(value) => Ok(value),
                Err(_) => Err(ApiError::bad_request(format!("the {name} is not a number"))),
            },
        }
    }

    /// A body field, as a string.
    pub fn body_str(&self, name: &str) -> Option<&str> {
        self.body
            .as_ref()
            .and_then(|body| body.get(name))
            .and_then(|value| value.as_str())
    }

    /// A body field, as a number.
    pub fn body_f64(&self, name: &str) -> ApiResult<Option<f64>> {
        match self.body.as_ref().and_then(|body| body.get(name)) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(number)) => match number.as_f64() {
                Some(value) if value.is_finite() => Ok(Some(value)),
                _ => Err(ApiError::bad_request(format!("the {name} is not a number"))),
            },
            Some(_) => Err(ApiError::bad_request(format!("the {name} is not a number"))),
        }
    }
}

/// One answer.
///
/// **The answer is the body.** The body is the DTO, and nothing around it:
/// the host puts the status in the status line — the desktop's HTTP layer
/// in the status line, the phone's bridge by rejecting the call. A body
/// that says `{"error": …}` with a 200 is never the shape.
///
/// [`Response::envelope`] is the FFI's own: the phone's seam reads an
/// envelope and hands the app the body inside it, so that the two hosts
/// give the app the same bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    /// The status the answer travels under.
    pub status: u16,
    /// The body of a success, or the error object of a failure.
    pub body: Value,
}

impl Response {
    /// Whether the answer is a success.
    pub fn is_ok(&self) -> bool {
        self.status < 400
    }

    /// The answer as the envelope both hosts read.
    ///
    /// ```text
    /// {"ok": true,  "status": 200, "body": { ... }}
    /// {"ok": false, "status": 404, "error": {"code": "not_found", "detail": "…"}}
    /// ```
    pub fn envelope(&self) -> Value {
        if self.is_ok() {
            serde_json::json!({
                "ok": true,
                "status": self.status,
                "body": self.body,
            })
        } else {
            serde_json::json!({
                "ok": false,
                "status": self.status,
                "error": self.body,
            })
        }
    }
}

/// What a handler does.
pub type Handler = fn(&Api, &Request) -> ApiResult<Value>;

/// One route of the frozen table.
#[derive(Clone, Copy, Debug)]
pub struct Route {
    /// The route's pattern: `/activities/{id}`.
    pub pattern: &'static str,
    /// The method the route answers.
    pub method: Method,
    /// The handler that answers it.
    pub handler: Handler,
}

/// The frozen route table.
///
/// Every route of `docs/contract/api-paths.md`, and nothing else. The
/// routes the contract cut are not here, and a path that names one is
/// answered with `not_found`.
///
/// The cut list is not written out here on purpose: the contract's own
/// grep refuses those names anywhere in the crates, and the test that
/// holds the cut routes out reads the list out of the contract itself.
pub const ROUTES: &[Route] = &[
    // The data.
    Route {
        pattern: "/activities",
        method: Method::Get,
        handler: crate::routes::data::activities,
    },
    Route {
        pattern: "/activities/{id}",
        method: Method::Get,
        handler: crate::routes::data::activity,
    },
    Route {
        pattern: "/physiology/daily",
        method: Method::Get,
        handler: crate::routes::data::physiology_daily,
    },
    Route {
        pattern: "/physiology/sleep",
        method: Method::Get,
        handler: crate::routes::data::physiology_sleep,
    },
    Route {
        pattern: "/physiology/hrv",
        method: Method::Get,
        handler: crate::routes::data::physiology_hrv,
    },
    Route {
        pattern: "/physiology/vitals",
        method: Method::Get,
        handler: crate::routes::data::physiology_vitals,
    },
    Route {
        pattern: "/physiology/latest",
        method: Method::Get,
        handler: crate::routes::data::physiology_latest,
    },
    Route {
        pattern: "/physiology/sources",
        method: Method::Get,
        handler: crate::routes::data::physiology_sources,
    },
    Route {
        pattern: "/fitness/fingerprint",
        method: Method::Get,
        handler: crate::routes::data::fitness_fingerprint,
    },
    Route {
        pattern: "/fitness/trends",
        method: Method::Get,
        handler: crate::routes::data::fitness_trends,
    },
    Route {
        pattern: "/fitness/zones",
        method: Method::Get,
        handler: crate::routes::data::fitness_zones,
    },
    Route {
        pattern: "/stats/summary",
        method: Method::Get,
        handler: crate::routes::data::stats_summary,
    },
    Route {
        pattern: "/stats/weekly",
        method: Method::Get,
        handler: crate::routes::data::stats_weekly,
    },
    Route {
        pattern: "/stats/pace-trends",
        method: Method::Get,
        handler: crate::routes::data::stats_pace_trends,
    },
    Route {
        pattern: "/stats/elevation",
        method: Method::Get,
        handler: crate::routes::data::stats_elevation,
    },
    Route {
        pattern: "/stats/personal-records",
        method: Method::Get,
        handler: crate::routes::data::stats_personal_records,
    },
    // The plan.
    Route {
        pattern: "/plan/current",
        method: Method::Get,
        handler: crate::routes::plan::current,
    },
    Route {
        pattern: "/plan/today",
        method: Method::Get,
        handler: crate::routes::plan::today,
    },
    Route {
        pattern: "/plan/week",
        method: Method::Get,
        handler: crate::routes::plan::week,
    },
    Route {
        pattern: "/plan/blocks",
        method: Method::Get,
        handler: crate::routes::plan::blocks,
    },
    Route {
        pattern: "/plan/sources",
        method: Method::Get,
        handler: crate::routes::plan::sources,
    },
    Route {
        pattern: "/plan/adherence",
        method: Method::Get,
        handler: crate::routes::plan::adherence,
    },
    Route {
        pattern: "/plan/results",
        method: Method::Get,
        handler: crate::routes::plan::results,
    },
    Route {
        pattern: "/plan/import",
        method: Method::Post,
        handler: crate::routes::plan::import,
    },
    // The insights.
    Route {
        pattern: "/insights/today",
        method: Method::Get,
        handler: crate::routes::insight::today,
    },
    Route {
        pattern: "/insights/progress",
        method: Method::Get,
        handler: crate::routes::insight::progress,
    },
    Route {
        pattern: "/insights/chips",
        method: Method::Get,
        handler: crate::routes::insight::chips,
    },
    Route {
        pattern: "/compare",
        method: Method::Get,
        handler: crate::routes::insight::compare,
    },
    Route {
        pattern: "/compare/candidates",
        method: Method::Get,
        handler: crate::routes::insight::compare_candidates,
    },
    // The app.
    Route {
        pattern: "/health",
        method: Method::Get,
        handler: crate::routes::app::health,
    },
    Route {
        pattern: "/health/daily",
        method: Method::Get,
        handler: crate::routes::app::health_daily,
    },
    Route {
        pattern: "/sync/status",
        method: Method::Get,
        handler: crate::routes::app::sync_status,
    },
    Route {
        pattern: "/sync/run",
        method: Method::Post,
        handler: crate::routes::app::sync_run,
    },
    Route {
        pattern: "/sync/runs",
        method: Method::Get,
        handler: crate::routes::app::sync_runs,
    },
    Route {
        pattern: "/sync/history",
        method: Method::Get,
        handler: crate::routes::app::sync_history,
    },
    Route {
        pattern: "/sync/settings",
        method: Method::Get,
        handler: crate::routes::app::sync_settings,
    },
    Route {
        pattern: "/auth/status",
        method: Method::Get,
        handler: crate::routes::app::auth_status,
    },
    Route {
        pattern: "/auth/garmin/personal-key",
        method: Method::Post,
        handler: crate::routes::app::auth_personal_key,
    },
    Route {
        pattern: "/auth/garmin/personal-key/test",
        method: Method::Post,
        handler: crate::routes::app::auth_personal_key_test,
    },
    Route {
        pattern: "/auth/garmin/fit-import",
        method: Method::Post,
        handler: crate::routes::app::auth_fit_import,
    },
    Route {
        pattern: "/auth/garmin/logout",
        method: Method::Post,
        handler: crate::routes::app::auth_logout,
    },
    Route {
        pattern: "/calendar",
        method: Method::Get,
        handler: crate::routes::app::calendar,
    },
];

/// The router.
pub struct Router {
    api: std::sync::Arc<Api>,
}

impl Router {
    /// A router over the shared state.
    pub fn new(api: std::sync::Arc<Api>) -> Self {
        Self { api }
    }

    /// The shared state.
    pub fn api(&self) -> &Api {
        &self.api
    }

    /// The frozen route table.
    pub fn routes(&self) -> &'static [Route] {
        ROUTES
    }

    /// The answer, as the envelope the hosts read.
    ///
    /// **One envelope on both sides of the boundary.** The desktop's HTTP
    /// layer puts the body on the wire and the status in the status line;
    /// the phone's bridge hands the body across and rejects the call. Both
    /// read the same envelope, so a number cannot differ between the two
    /// hosts without one of them being wrong.
    ///
    /// ```text
    /// {"ok": true,  "status": 200, "body": { ... }}
    /// {"ok": false, "status": 401, "error": {"code": "needs_reconnect", "detail": "…"}}
    /// ```
    ///
    /// This is what the phone's seam calls, and what the rig answers with
    /// when it hands the body over: one function, one envelope.
    pub fn envelope(&self, method: &str, path: &str, body: Option<&str>) -> Value {
        self.call(method, path, body).envelope()
    }

    /// Answer a request.
    ///
    /// The path may carry a query string and may carry the desktop's
    /// prefix: both are cut before the route is matched.
    pub fn call(&self, method: &str, path: &str, body: Option<&str>) -> Response {
        let method = Method::parse(method).unwrap_or(Method::Get);
        let (path, query) = split_query(path);
        let path = self.strip_prefix(&path);

        let body = match body {
            None | Some("") => None,
            Some(text) => match serde_json::from_str::<Value>(text) {
                Ok(value) => Some(value),
                Err(_) => {
                    return failure(&ApiError::bad_request("the body is not a JSON document"))
                }
            },
        };

        let found = match_route(method, &path);
        let (route, params) = match found {
            Some(found) => found,
            None => {
                // The route may exist with the other method: that is a
                // different answer from a route that is not there at all.
                if path_route_exists(&path) {
                    return failure(&ApiError::bad_request(format!(
                        "this route is not read with {}",
                        method.as_str()
                    )));
                }
                return failure(&ApiError::not_found(format!("there is no route at {path}")));
            }
        };

        let request = Request {
            method,
            path: path.clone(),
            query,
            params,
            body,
        };

        match (route.handler)(&self.api, &request) {
            Ok(value) => Response {
                status: 200,
                body: value,
            },
            Err(error) => failure(&error),
        }
    }

    /// The route a path names, whatever the method.
    ///
    /// The path may carry a query string and the desktop's prefix: both
    /// are cut before the route is matched.
    pub fn route_at(&self, path: &str) -> Option<&'static Route> {
        let (path, _) = split_query(path);
        let path = self.strip_prefix(&path);
        ROUTES
            .iter()
            .find(|route| capture(route.pattern, &path).is_some())
    }

    /// Cut the desktop's prefix off a path.
    fn strip_prefix(&self, path: &str) -> String {
        let prefix = self.api.config().api_prefix.trim_end_matches('/');
        if prefix.is_empty() {
            return path.to_string();
        }
        if path == prefix {
            return "/".to_string();
        }
        match path.strip_prefix(prefix) {
            Some(rest) if rest.starts_with('/') => rest.to_string(),
            _ => path.to_string(),
        }
    }
}

/// The failure, as a response.
pub fn failure(error: &ApiError) -> Response {
    let envelope: ErrorEnvelope = error.envelope();
    Response {
        status: error.status(),
        body: serde_json::to_value(envelope).unwrap_or_else(|_| {
            serde_json::json!({"code": "internal", "detail": "the request could not be completed"})
        }),
    }
}

/// The success, as a response.
pub fn success<T: Serialize>(value: &T) -> ApiResult<Value> {
    serde_json::to_value(value).map_err(|_| ApiError::internal("the answer could not be written"))
}

/// The path and the query string of a path.
fn split_query(path: &str) -> (String, Query) {
    match path.split_once('?') {
        Some((path, query)) => (path.to_string(), Query::parse(query)),
        None => (path.to_string(), Query::default()),
    }
}

/// The route a path names, and the value of its placeholder.
fn match_route(method: Method, path: &str) -> Option<(&'static Route, Vec<(String, String)>)> {
    ROUTES.iter().find_map(|route| {
        if route.method != method {
            return None;
        }
        capture(route.pattern, path).map(|captured| {
            let params = match captured {
                Some(value) => vec![(placeholder(route.pattern), value)],
                None => Vec::new(),
            };
            (route, params)
        })
    })
}

/// Whether a path names a route at all, whatever the method.
fn path_route_exists(path: &str) -> bool {
    ROUTES
        .iter()
        .any(|route| capture(route.pattern, path).is_some())
}

/// The name of a route's placeholder.
fn placeholder(pattern: &str) -> String {
    pattern
        .trim_matches('/')
        .split('/')
        .find(|part| part.starts_with('{') && part.ends_with('}'))
        .map(|part| part.trim_matches(|character| character == '{' || character == '}'))
        .unwrap_or_default()
        .to_string()
}

/// The value of a route's placeholder, when the path names the route.
///
/// `None` when the path does not name the route, and `Some(None)` when it
/// does and the route has no placeholder.
fn capture(pattern: &str, path: &str) -> Option<Option<String>> {
    let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    if pattern_parts.len() != path_parts.len() {
        return None;
    }
    let mut captured: Option<String> = None;
    for (wanted, given) in pattern_parts.iter().zip(path_parts.iter()) {
        if wanted.starts_with('{') && wanted.ends_with('}') {
            if captured.is_some() {
                return None;
            }
            captured = Some((*given).to_string());
        } else if wanted != given {
            return None;
        }
    }
    Some(captured)
}
