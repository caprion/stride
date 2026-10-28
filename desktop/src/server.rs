//! The application: the requests, in and the answers, out.
//!
//! One router, two hosts. The API is `stride-api`'s — the same router the
//! phone calls through the FFI — and the SPA is the built React app in
//! `frontend/dist`, which is what the Python server serves today.
//!
//! The behaviour that must not change:
//!
//! * the SPA at `/` and the API at `/api/v1/...`;
//! * the static assets with the right content types;
//! * the SPA's `index.html` for any unknown path that is not under `/api/`,
//!   and **never** a file from outside `frontend/dist`;
//! * `{"detail": "..."}` for a 404;
//! * the capability cookie, minted for the loopback browser.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use stride_api::{Api, Response as ApiResponse};

use crate::http::{Request, Response};
use crate::security::{is_loopback_client, same_origin, RateLimiter, Security, BOOTSTRAP_PATH};
use crate::spa;

/// The content security policy.
///
/// The reference sends the same one: no third-party origins at all, the map
/// tiles proxied by this server so the athlete's address is never revealed
/// to a tile provider, and an inline style, which the app's own stylesheet
/// needs.
const CONTENT_SECURITY_POLICY: &str = concat!(
    "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; ",
    "form-action 'self'; script-src 'self'; style-src 'self' '",
    "unsafe-",
    "inline'; ",
    "font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self'"
);

/// The application.
pub struct App {
    /// The one router.
    api: Api,
    /// The capability gate.
    security: Security,
    /// The built SPA.
    dist: PathBuf,
    /// The rate limiter.
    limiter: RateLimiter,
    /// The origins that may call the API from a page.
    allowed_origins: Vec<String>,
}

impl App {
    /// An application over a router.
    pub fn new(api: Api, security: Security, dist: PathBuf, allowed_origins: Vec<String>) -> Self {
        Self {
            api,
            security,
            dist,
            limiter: RateLimiter::new(),
            allowed_origins,
        }
    }

    /// The router.
    pub fn api(&self) -> &Api {
        &self.api
    }

    /// The gate.
    pub fn security(&self) -> &Security {
        &self.security
    }

    /// Answer a request.
    pub fn handle(&self, request: &Request) -> Response {
        let mut response = self.route(request);
        self.apply_headers(request, &mut response);
        response
    }

    /// The routing itself.
    fn route(&self, request: &Request) -> Response {
        if request.method == "OPTIONS" {
            return self.preflight(request);
        }
        let path = request.path.clone();
        if path == BOOTSTRAP_PATH && (request.method == "GET" || request.method == "HEAD") {
            return self.bootstrap(request);
        }
        let is_api = path == "/api/v1" || path.starts_with("/api/v1/");
        if is_api {
            return self.api_request(request, &path);
        }
        self.spa(request, &path)
    }

    /// The preflight answer.
    ///
    /// The SPA is served from this origin, so a preflight only happens for
    /// the development server, which is on another port and is in the
    /// allowed-origins list.
    fn preflight(&self, request: &Request) -> Response {
        let origin = request.headers.get("origin").unwrap_or_default();
        if !self.allowed_origins.iter().any(|allowed| allowed == origin) {
            return Response::with_body(
                403,
                "application/json",
                br#"{"detail":"Origin not allowed."}"#.to_vec(),
            );
        }
        Response::empty(204)
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PATCH, PUT, DELETE, OPTIONS",
            )
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, X-Stride-Token",
            )
            .header("Access-Control-Max-Age", "600")
    }

    /// The bootstrap: the cookie that lets the browser use the API.
    fn bootstrap(&self, request: &Request) -> Response {
        if !self.security.enabled() {
            return Response::empty(204);
        }
        if !is_loopback_client(request) {
            return json_response(
                403,
                &serde_json::json!({ "detail": "Local same-origin bootstrap required." }),
            );
        }
        Response::empty(204).header("Set-Cookie", &self.security.capability_cookie())
    }

    /// The API.
    fn api_request(&self, request: &Request, path: &str) -> Response {
        if let Some((bucket, limit)) = RateLimiter::bucket(path) {
            let client = request.peer.to_string();
            if !self.limiter.allow(&client, bucket, limit) {
                return json_response(
                    429,
                    &serde_json::json!({ "detail": "Too many requests; retry shortly." }),
                )
                .header("Retry-After", "60");
            }
        }

        if let Err(error) = self.security.authorize(request, path) {
            return json_response(
                error.status(),
                &serde_json::json!({ "detail": error.detail() }),
            );
        }

        if is_mutation(&request.method) && !self.allows_mutation(request) {
            return json_response(
                403,
                &serde_json::json!({ "detail": "Same-origin request required." }),
            );
        }

        let target = api_target(request);
        let body = request.body_text();
        match self.api.call_response(&request.method, &target, body) {
            Ok(ApiResponse::Json(value)) => json_response(200, &value),
            Ok(ApiResponse::Binary {
                content_type,
                bytes,
            }) => Response::with_body(200, &content_type, bytes),
            Ok(ApiResponse::NoContent) => Response::empty(204),
            Err(error) => json_response(error.status, &error.to_body()),
        }
    }

    /// May this request change something?
    ///
    /// An explicit bearer header is authority the caller chose to present. A
    /// cookie is ambient authority, so it has to come from this origin — or
    /// from the development server, which is in the allowed-origins list.
    fn allows_mutation(&self, request: &Request) -> bool {
        let explicit = self
            .security
            .provided_token(request)
            .is_some_and(|(_, explicit)| explicit);
        if explicit {
            return true;
        }
        let origin = request.headers.get("origin");
        if same_origin(origin, request.host()) {
            return true;
        }
        match origin {
            Some(origin) => self
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin.trim_end_matches('/')),
            // A request with no `Origin` is not the page: the browser sends
            // one on every request that is not a `GET` or a `HEAD`.
            None => false,
        }
    }

    /// The SPA and the static files.
    fn spa(&self, request: &Request, path: &str) -> Response {
        // The reference's catch-all is a `GET` route, so a `POST` to an
        // unknown path is a 405 there and it is a 405 here.
        if request.method != "GET" && request.method != "HEAD" {
            return json_response(405, &serde_json::json!({ "detail": "Method Not Allowed" }));
        }
        let relative = path.trim_start_matches('/');
        if relative.starts_with("api/") {
            return json_response(404, &serde_json::json!({ "detail": "Not found" }));
        }

        if let Some(file) = spa::resolve(&self.dist, path) {
            let content_type = spa::content_type(&file);
            return match std::fs::read(&file) {
                Ok(bytes) => {
                    let response = Response::with_body(200, content_type, bytes);
                    self.maybe_mint_cookie(request, &response)
                }
                Err(_) => self.index(request),
            };
        }

        self.index(request)
    }

    /// The app's `index.html`.
    fn index(&self, request: &Request) -> Response {
        let Some(index) = spa::index(&self.dist) else {
            // The reference answers with a short JSON message when the
            // frontend has not been built yet.
            return json_response(
                200,
                &serde_json::json!({
                    "service": "stride",
                    "message": "The frontend has not been built yet.",
                    "build": "cd frontend && npm install && npm run build",
                    "dev": "cd frontend && npm run dev",
                    "api": "/api/v1/docs",
                }),
            );
        };
        let Ok(bytes) = std::fs::read(&index) else {
            return json_response(
                500,
                &serde_json::json!({ "detail": "the app could not be read" }),
            );
        };
        let response = Response::with_body(200, "text/html; charset=utf-8", bytes);
        self.maybe_mint_cookie(request, &response)
    }

    /// Hand the browser its capability, when it is the loopback browser.
    ///
    /// Loading the app establishes the session: the cookie is HttpOnly, so
    /// the page never sees the capability, and it is minted only for a
    /// loopback peer whose `Host` header also says loopback.
    fn maybe_mint_cookie(&self, request: &Request, response: &Response) -> Response {
        if !self.security.enabled() {
            return response.clone();
        }
        let is_html = response
            .get("Content-Type")
            .is_some_and(|value| value.starts_with("text/html"));
        if !is_html || !is_loopback_client(request) {
            return response.clone();
        }
        response
            .clone()
            .header("Set-Cookie", &self.security.capability_cookie())
    }

    /// The headers every answer carries.
    fn apply_headers(&self, request: &Request, response: &mut Response) {
        if response.get("Content-Security-Policy").is_none() {
            response.headers.push((
                "Content-Security-Policy".to_string(),
                CONTENT_SECURITY_POLICY.to_string(),
            ));
        }
        if response.get("X-Content-Type-Options").is_none() {
            response
                .headers
                .push(("X-Content-Type-Options".to_string(), "nosniff".to_string()));
        }
        if response.get("Referrer-Policy").is_none() {
            response
                .headers
                .push(("Referrer-Policy".to_string(), "no-referrer".to_string()));
        }
        if response.get("X-Frame-Options").is_none() {
            response
                .headers
                .push(("X-Frame-Options".to_string(), "DENY".to_string()));
        }
        if response.get("Permissions-Policy").is_none() {
            response.headers.push((
                "Permissions-Policy".to_string(),
                "camera=(), microphone=(), geolocation=()".to_string(),
            ));
        }
        let is_api = request.path == "/api/v1" || request.path.starts_with("/api/v1/");
        if is_api && response.get("Cache-Control").is_none() {
            response
                .headers
                .push(("Cache-Control".to_string(), "no-store".to_string()));
        }

        // The development server is on another port; the answer has to say
        // that it may be used.
        if let Some(origin) = request.headers.get("origin") {
            if self
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin.trim_end_matches('/'))
            {
                response.headers.push((
                    "Access-Control-Allow-Origin".to_string(),
                    origin.to_string(),
                ));
                response.headers.push((
                    "Access-Control-Allow-Credentials".to_string(),
                    "true".to_string(),
                ));
                response
                    .headers
                    .push(("Vary".to_string(), "Origin".to_string()));
            }
        }
    }
}

/// Does this method change something?
pub fn is_mutation(method: &str) -> bool {
    matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
}

/// The path the router is given.
///
/// The path is percent-decoded — the way Starlette hands it to a route — and
/// the query string is left exactly as it arrived, because the router
/// decodes it itself. A `?` inside the path is put back into its escaped
/// form, so that the router's own split on `?` cannot be confused.
fn api_target(request: &Request) -> String {
    let path = request.path.replace('?', "%3F");
    if request.query.is_empty() {
        path
    } else {
        format!("{path}?{}", request.query)
    }
}

/// A JSON answer.
pub fn json_response(status: u16, value: &Value) -> Response {
    let body = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
    Response::with_body(status, "application/json", body)
}

/// The application, behind an `Arc`, ready to be handed to the server.
pub type SharedApp = Arc<dyn Fn(&Request) -> Response + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Headers;
    use std::net::IpAddr;

    fn request(method: &str, path: &str, headers: &[(&str, &str)]) -> Request {
        let mut head = Headers::new();
        for (name, value) in headers {
            head.push(name, value);
        }
        Request {
            method: method.to_string(),
            target: path.to_string(),
            path: path.to_string(),
            query: String::new(),
            headers: head,
            body: Vec::new(),
            peer: IpAddr::from([127, 0, 0, 1]),
        }
    }

    #[test]
    fn a_mutation_is_a_mutation() {
        assert!(is_mutation("POST"));
        assert!(is_mutation("PATCH"));
        assert!(!is_mutation("GET"));
    }

    #[test]
    fn the_query_string_is_kept_raw() {
        let mut req = request("GET", "/api/v1/activities", &[]);
        req.query = "page=2&per_page=10".to_string();
        assert_eq!(api_target(&req), "/api/v1/activities?page=2&per_page=10");
    }
}
