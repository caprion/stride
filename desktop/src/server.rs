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
use crate::security::{redact, same_origin, RateLimiter, Security, BOOTSTRAP_PATH};
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
        let origin = request.headers.get("origin");
        let asked = request
            .headers
            .get("access-control-request-method")
            .is_some();
        if origin.is_none() || !asked {
            // An ordinary `OPTIONS`: the router answers it.
            return json_response(405, &serde_json::json!({ "detail": "Method Not Allowed" }));
        }
        let origin = origin.unwrap_or_default();
        let allowed = self
            .allowed_origins
            .iter()
            .any(|allowed| allowed == origin.trim_end_matches('/'))
            || same_origin(Some(origin), request.host());
        if !allowed {
            return Response::with_body(
                400,
                "text/plain; charset=utf-8",
                b"Disallowed CORS origin".to_vec(),
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
    ///
    /// Only a trusted client may have one: the loopback browser, or a client
    /// on a network the operator has chosen to trust. The reference answers a
    /// request that is not from this machine with the middleware's message,
    /// and a request that carries a foreign `Origin` with the route's.
    fn bootstrap(&self, request: &Request) -> Response {
        if !self.security.enabled() {
            return Response::empty(204);
        }
        if !self.security.is_trusted_client(request) {
            return json_response(
                403,
                &serde_json::json!({
                    "detail": "Bootstrap is available only from the local Stride origin."
                }),
            );
        }
        if let Some(origin) = request.headers.get("origin") {
            let allowed = same_origin(Some(origin), request.host())
                || self
                    .allowed_origins
                    .iter()
                    .any(|allowed| allowed == origin.trim_end_matches('/'));
            if !allowed {
                return json_response(
                    403,
                    &serde_json::json!({ "detail": "Local same-origin bootstrap required." }),
                );
            }
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
        // The router does not know who is asking; the host does. The
        // pairing code is the one route that depends on it, so the caller's
        // address and the name it used travel with the request.
        let caller = stride_api::Caller {
            peer: Some(request.peer),
            host: Some(request.host().to_string()),
        };
        match self
            .api
            .call_response_from(&caller, &request.method, &target, body)
        {
            Ok(ApiResponse::Json(value)) => {
                // The pairing route hands over a device credential. The
                // browser has a cookie jar and the phone does not, so the
                // desktop puts the token in a cookie and answers with
                // nothing — exactly as the reference does — and the phone's
                // shell reads the token from the body.
                if target == "/api/v1/auth/pair" {
                    if let Some(token) = value.get("token").and_then(|t| t.as_str()) {
                        return json_response(204, &serde_json::json!({}))
                            .header("Set-Cookie", &self.security.device_cookie(token));
                    }
                }
                json_response(200, &value)
            }
            Ok(ApiResponse::Binary {
                content_type,
                bytes,
            }) => Response::with_body(200, &content_type, bytes),
            Ok(ApiResponse::NoContent) => Response::empty(204),
            Err(error) => api_error(&error),
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
        // A client from a trusted network may act without an `Origin` header:
        // that is a native client, not a page. When an `Origin` header *is*
        // present it still has to match — the network says who is calling,
        // not which page the call came from.
        if self.security.is_trusted_remote_client(request) && origin.is_none() {
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

        // The reference mounts `frontend/dist/assets` as a static directory,
        // and a path under it that is not a file is a 404 there — not the
        // app. A missing script with a 200 and an HTML body is a debugging
        // nightmare, and the reference does not do it.
        if relative.starts_with("assets/") {
            return match spa::resolve(&self.dist, path) {
                Some(file) => match std::fs::read(&file) {
                    Ok(bytes) => {
                        let content_type = spa::content_type(&file);
                        Response::with_body(200, content_type, bytes)
                    }
                    Err(_) => json_response(404, &serde_json::json!({ "detail": "Not Found" })),
                },
                None => json_response(404, &serde_json::json!({ "detail": "Not Found" })),
            };
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

    /// Hand the browser its capability, when it is a trusted client.
    ///
    /// Loading the app establishes the session: the cookie is HttpOnly, so
    /// the page never sees the capability. It is minted for the loopback peer
    /// whose `Host` header also says loopback, and for a client on a network
    /// the operator has chosen to trust.
    fn maybe_mint_cookie(&self, request: &Request, response: &Response) -> Response {
        if !self.security.enabled() {
            return response.clone();
        }
        let is_html = response
            .get("Content-Type")
            .is_some_and(|value| value.starts_with("text/html"));
        if !is_html || !self.security.is_trusted_client(request) {
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

/// The answer to a route that failed.
///
/// The `detail` is redacted on the way out: an upstream error can quote the
/// credential it was given, and the reference redacts every answer that
/// carries an exception detail.
fn api_error(error: &stride_api::error::ApiError) -> Response {
    let detail = match &error.detail {
        Value::String(text) => Value::String(redact(text)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| match item {
                    Value::Object(map) => {
                        let mut out = serde_json::Map::new();
                        for (name, value) in map {
                            out.insert(
                                name.clone(),
                                match value {
                                    Value::String(text) => Value::String(redact(text)),
                                    other => other.clone(),
                                },
                            );
                        }
                        Value::Object(out)
                    }
                    other => other.clone(),
                })
                .collect(),
        ),
        other => other.clone(),
    };
    json_response(error.status, &serde_json::json!({ "detail": detail }))
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
