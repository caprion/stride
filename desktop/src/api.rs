//! The seam to the router.
//!
//! **The router is `stride-api`'s.** One router, two hosts: the phone
//! reaches it in-process through the FFI, and the desktop reaches it over
//! loopback HTTP. Neither can disagree about a number, because there is only
//! one of it.
//!
//! ## What the rig hosts
//!
//! The rig builds `stride_api::Router` over the rig's own database and hands
//! it to the HTTP surface. The router's answer goes back on the wire **byte
//! for byte**: the rig does not re-shape a response, add a field to one, or
//! turn a number into a string.
//!
//! ## The key
//!
//! The Garmin key is an argument and a session copy, and never more.
//!
//! * the rig's **key store** is the holder: it holds the key the athlete
//!   typed into the app's own field, the key in the environment, or the
//!   key in a file with `0600` — and it forgets it on a logout;
//! * the **core** gets a working copy for the length of the call, and the
//!   copy is dropped when the call is over;
//! * and neither a response, nor a log line, nor an error, nor a file
//!   carries it.
//!
//! The rig reads the key out of the request body exactly once — when the
//! athlete's own field posts it — and forgets it the moment it is not
//! wanted.

use std::sync::Arc;

use serde_json::Value;
use stride_api::{Api, Clock, Config, Router as CoreRouter, SystemClock};
use stride_store::Store;

use crate::error::Error;
use crate::key::KeyStore;

/// The prefix the app's requests carry.
///
/// The frozen route table (`docs/contract/api-paths.md`) names the routes
/// without it, so the rig cuts it off before the router sees a path. The
/// phone's shell cuts the same prefix off, for the same reason: both hosts
/// match on the same route.
pub const API_PREFIX: &str = "/api/v1";

/// A request, as the router sees it.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    /// The method.
    pub method: &'a str,
    /// The path, without the API prefix and without the query string.
    pub path: &'a str,
    /// The query string.
    pub query: &'a str,
    /// The body.
    pub body: &'a [u8],
}

/// A response, as the router gives it.
#[derive(Debug, Clone)]
pub struct Response {
    /// The status code.
    pub status: u16,
    /// The content type.
    pub content_type: String,
    /// The body.
    pub body: Vec<u8>,
}

impl Response {
    /// A response with a status and a body.
    pub fn new(status: u16, content_type: &str, body: Vec<u8>) -> Response {
        Response {
            status,
            content_type: content_type.to_string(),
            body,
        }
    }

    /// A response that is JSON.
    pub fn json(status: u16, body: &str) -> Response {
        Response::new(status, "application/json", body.as_bytes().to_vec())
    }
}

/// The router: the one that answers every route.
pub trait Router: Send + Sync {
    /// Answer one route.
    fn call(&self, request: &Request<'_>) -> Response;
}

/// What the router may ask of the machine it runs on.
///
/// The rig owns the database and the Garmin key, and the router reads them.
/// This is the whole of the surface: two services, and nothing else.
pub trait Host: Send + Sync {
    /// The database, for the routes that read.
    fn store(&self) -> Arc<Store>;

    /// The Garmin key, for the routes that connect and pull.
    fn keys(&self) -> Arc<KeyStore>;
}

/// The real router: `stride-api`'s, hosted by the rig.
///
/// The adapter is a dozen lines, and that is the whole of it: the rig
/// builds the router, hands it the rig's database, and passes on what it
/// says. Nothing else in the rig knows how a route is answered.
pub struct ApiRouter {
    /// The router.
    router: Arc<CoreRouter>,
    /// The rig's key store: the holder of the session key.
    keys: Arc<KeyStore>,
}

impl ApiRouter {
    /// The router over a host.
    ///
    /// The host's database is the router's database, and the host's clock
    /// is the router's clock. The key is **not** installed here: it is
    /// installed for the length of each call, from the rig's key store,
    /// so that a key the athlete typed and a key the environment holds are
    /// the same key by the time the router sees them.
    pub fn over(host: &dyn Host) -> Result<ApiRouter, Error> {
        Self::with_clock(host, Arc::new(SystemClock))
    }

    /// The router over a host, with a clock of the caller's choosing.
    ///
    /// The clock is injected, so every date the router derives is
    /// reproducible: a test can hold it still and compare two hosts'
    /// answers byte for byte.
    pub fn with_clock(host: &dyn Host, clock: Arc<dyn Clock>) -> Result<ApiRouter, Error> {
        let api = Api::new(host.store(), Config::default(), clock)
            .map_err(|_| Error::config("the Garmin client could not be built"))?;
        Ok(ApiRouter {
            router: Arc::new(CoreRouter::new(Arc::new(api))),
            keys: host.keys(),
        })
    }

    /// The router underneath.
    pub fn router(&self) -> &CoreRouter {
        &self.router
    }

    /// The shared state.
    pub fn api(&self) -> &stride_api::Api {
        self.router.api()
    }

    /// Give the core the rig's key, for the length of the call.
    ///
    /// The core's copy is a working copy: the rig's key store is the
    /// holder, and this is what the core borrows for one call. A key store
    /// with nothing in it leaves the core with nothing in it, so a logout
    /// on either side is a logout on both.
    fn lend_key(&self) {
        let lent = self
            .keys
            .with_secret(|secret| self.router.api().set_garmin_key(secret));
        if lent.is_none() {
            self.router.api().clear_garmin_key();
        }
    }

    /// Keep the rig's key store in step with the key the athlete typed.
    ///
    /// Two routes change the key, and the rig is the one that holds it:
    ///
    /// * `POST /auth/garmin/personal-key` — the athlete's own field posts
    ///   the key, and the rig keeps it for the session, in memory;
    /// * `POST /auth/garmin/logout` — and the rig forgets it.
    ///
    /// The key is read out of the body once, and it never reaches a log
    /// line: the rig logs the method and the path, and nothing else.
    fn keep_key_store_in_step(&self, request: &Request<'_>, status: u16) -> Option<Response> {
        if !request.method.eq_ignore_ascii_case("POST") {
            return None;
        }
        match request.path {
            "/api/v1/auth/garmin/personal-key" if status == 200 => {
                let key = key_in_body(request.body)?;
                if let Err(error) = self.keys.set(&key) {
                    return Some(key_store_failure(&error));
                }
                if let Err(error) = self.keys.persist() {
                    self.keys.forget();
                    crate::log::log(&format!(
                        "the Garmin key could not be kept on disk: {error}"
                    ));
                    return Some(key_store_failure(&error));
                }
                None
            }
            "/api/v1/auth/garmin/logout" => {
                self.keys.forget();
                if let Err(error) = self.keys.remove_persisted() {
                    crate::log::log(&format!(
                        "the private Garmin key file could not be removed: {error}"
                    ));
                    return Some(key_store_failure(&error));
                }
                None
            }
            _ => None,
        }
    }
}

impl Router for ApiRouter {
    fn call(&self, request: &Request<'_>) -> Response {
        // The core borrows the rig's key for the length of this call.
        self.lend_key();

        let path = with_query(request.path, request.query);
        let body = match std::str::from_utf8(request.body) {
            Ok(text) if text.trim().is_empty() => None,
            Ok(text) => Some(text),
            Err(_) => {
                return Response::json(
                    400,
                    "{\"code\":\"bad_request\",\"detail\":\"the body is not text\"}",
                )
            }
        };

        let answer = self.router.call(request.method, &path, body);

        // The rig's key store is the holder, so it is told what the athlete
        // typed — and told to forget it on a logout.
        if let Some(failure) = self.keep_key_store_in_step(request, answer.status) {
            return failure;
        }

        // **The answer is the body, byte for byte.** The router's status
        // goes in the status line, and its body goes on the wire as it is:
        // the rig does not wrap it, and it does not add a field to it.
        let body = serde_json::to_vec(&answer.body).unwrap_or_else(|_| b"{}".to_vec());
        Response::new(answer.status, "application/json", body)
    }
}

/// The answer when the rig could not keep the key safely.
///
/// The detail says what to do and carries neither the key nor a file path.
/// The path is in the rig's own log, where it belongs.
fn key_store_failure(error: &Error) -> Response {
    let _ = error;
    Response::json(
        500,
        "{\"code\":\"key_store_failed\",\"detail\":\"the key was accepted, but this rig could not keep it safely; check the data directory's permissions and try again\"}",
    )
}

/// The path, with its query string.
fn with_query(path: &str, query: &str) -> String {
    if query.trim().is_empty() {
        return path.to_string();
    }
    format!("{path}?{query}")
}

/// The key in a request body, if the body carries one.
fn key_in_body(body: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let parsed: Value = serde_json::from_str(text).ok()?;
    let key = parsed.get("key")?.as_str()?.trim();
    if key.is_empty() {
        return None;
    }
    Some(key.to_string())
}

/// Is this path a request for the router?
///
/// The app's requests all carry the [`API_PREFIX`]; everything else is the
/// app's own routing, and the app answers it.
///
/// The prefix is **not** taken off: the router sees the path the app asked
/// for, and it is the router — and only the router — that decides which
/// route a path names.
pub fn route_path(path: &str) -> Option<&str> {
    if path == API_PREFIX {
        return Some(path);
    }
    path.strip_prefix(API_PREFIX)
        .filter(|rest| rest.starts_with('/'))
        .map(|_| path)
}

/// Is this request a pull from Garmin?
///
/// The pull is the one request the rig owns a rule for: the client makes
/// one exchange and one attempt, and the rig may make one more attempt.
/// See [`crate::sync`].
pub fn is_pull(method: &str, path: &str) -> bool {
    method.eq_ignore_ascii_case("POST") && path == "/api/v1/sync/run"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prefix_decides_and_is_not_taken_off() {
        assert_eq!(route_path("/api/v1/activities"), Some("/api/v1/activities"));
        assert_eq!(route_path("/api/v1"), Some("/api/v1"));
        assert_eq!(route_path("/runs"), None);
        assert_eq!(route_path("/api/v2/activities"), None);
    }

    #[test]
    fn the_query_string_is_put_back_on_the_path() {
        assert_eq!(with_query("/api/v1/activities", ""), "/api/v1/activities");
        assert_eq!(
            with_query("/api/v1/activities", "limit=3"),
            "/api/v1/activities?limit=3"
        );
    }

    #[test]
    fn only_the_pull_is_a_pull() {
        assert!(is_pull("POST", "/api/v1/sync/run"));
        assert!(!is_pull("GET", "/api/v1/sync/run"));
        assert!(!is_pull("POST", "/api/v1/sync/status"));
    }

    #[test]
    fn the_key_is_read_out_of_the_body_once() {
        let body = br#"{"key":"a-key","days":30}"#;
        assert_eq!(key_in_body(body).as_deref(), Some("a-key"));
        assert_eq!(key_in_body(br#"{"days":30}"#), None);
        assert_eq!(key_in_body(br#"{"key":"  "}"#), None);
        assert_eq!(key_in_body(b"not json"), None);
    }
}
