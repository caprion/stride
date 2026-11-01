//! The seam to the router.
//!
//! **The router is `stride-api`'s.** One router, two hosts: the phone
//! reaches it in-process through the FFI, and the desktop reaches it over
//! loopback HTTP. Neither can disagree about a number, because there is only
//! one of it.
//!
//! ## The state of things
//!
//! `crates/stride-api` is an empty skeleton today — it holds its charter and
//! nothing else — so the rig has no router to host yet. Until that changes,
//! the rig answers through a **stub** that refuses rather than guesses.
//!
//! That is the same seam the phone's FFI crate uses (`seam::Core` in
//! `android/app/src/main/rust/stride-ffi`), and it is the same rule: **a
//! stub can refuse, but it cannot be wrong.** An app that shows nothing is a
//! disappointing app; an app that shows a number nobody computed is a lie
//! about somebody's training.
//!
//! So the stub answers every route with one sentence — the same sentence
//! the phone's stub answers with — and the rig is honest about it.
//!
//! ## What changes when the router lands
//!
//! One file: this one. The [`Router`] trait is the port, the [`Host`]
//! trait is what the router may ask of the machine it runs on, and the
//! adapter that joins them to `stride_api` is a dozen lines. Nothing else in
//! the rig knows how a route is answered.

use stride_store::Store;

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
    fn store(&self) -> &Store;

    /// The Garmin key, for the routes that connect and pull.
    fn keys(&self) -> &KeyStore;
}

/// The router that is not in the build yet.
///
/// It refuses every route with the sentence the phone's stub uses, and it
/// answers in the envelope the phone's seam uses — the same envelope, so
/// that the two hosts cannot drift apart in their shape either.
#[derive(Debug, Clone, Copy, Default)]
pub struct StubRouter;

/// The sentence the stub answers with.
pub const NOT_BUILT: &str = "the Rust core is not in this build. Nothing is shown until it is.";

impl Router for StubRouter {
    fn call(&self, _request: &Request<'_>) -> Response {
        Response::json(
            503,
            &format!("{{\"ok\":false,\"status\":503,\"error\":{{\"detail\":\"{NOT_BUILT}\"}}}}"),
        )
    }
}

/// Is this path a request for the router?
///
/// The app's requests all carry the [`API_PREFIX`]; everything else is the
/// app's own routing, and the app answers it.
pub fn route_path(path: &str) -> Option<String> {
    if path == API_PREFIX {
        return Some("/".to_string());
    }
    path.strip_prefix(API_PREFIX)
        .filter(|rest| rest.starts_with('/'))
        .map(|rest| rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prefix_is_cut_off() {
        assert_eq!(
            route_path("/api/v1/activities"),
            Some("/activities".to_string())
        );
        assert_eq!(route_path("/api/v1"), Some("/".to_string()));
        assert_eq!(route_path("/runs"), None);
        assert_eq!(route_path("/api/v2/activities"), None);
    }

    #[test]
    fn the_stub_refuses_rather_than_guesses() {
        let answer = StubRouter.call(&Request {
            method: "GET",
            path: "/activities",
            query: "",
            body: &[],
        });
        assert_eq!(answer.status, 503);
        let body = String::from_utf8_lossy(&answer.body).into_owned();
        assert!(
            body.contains("the Rust core is not in this build"),
            "{body}"
        );
    }
}
