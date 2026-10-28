//! The capability gate.
//!
//! The capability is deliberately *not* an account login. It is a random
//! secret that the local browser holds in an HttpOnly cookie, and it stops
//! another process on this machine — or a page in another tab — from reading
//! the athlete's health data.
//!
//! The rules are ported from `app/security.py`, and the reasons are the
//! same:
//!
//! * A few paths answer without a capability: `health` is a liveness probe,
//!   `pair-code` and `pair` are the operator's own console, and `bootstrap`
//!   mints the cookie.
//! * A request that carries a capability proves itself. A request that does
//!   not is refused with `401`.
//! * A request that *changes* something has to come from this origin. A
//!   bearer header is explicit authority and is allowed from anywhere; a
//!   cookie is ambient authority and is only allowed same-origin, so that a
//!   page on another site cannot drive the API with the athlete's browser.
//! * The cookie is minted for the loopback browser only, and only when the
//!   peer address *and* the `Host` header both say loopback — a rebound name
//!   must not be able to ask for a credential.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use crate::http::Request;

/// The cookie the capability travels in.
pub const AUTH_COOKIE: &str = "stride_capability";

/// The prefix of the API.
pub const API_PREFIX: &str = "/api/v1";

/// The path that mints the cookie.
pub const BOOTSTRAP_PATH: &str = "/api/v1/bootstrap";

/// The paths that answer without a capability.
pub const CAPABILITY_FREE: [&str; 4] = [
    "/api/v1/health",
    BOOTSTRAP_PATH,
    "/api/v1/auth/pair-code",
    "/api/v1/auth/pair",
];

/// The gate.
#[derive(Debug, Clone)]
pub struct Security {
    /// The capability the browser must present.
    token: String,
    /// Whether the gate is closed at all.
    enabled: bool,
    /// Whether the cookie is marked `Secure`.
    cookie_secure: bool,
}

impl Security {
    /// A gate with a capability.
    pub fn new(token: impl Into<String>, enabled: bool, cookie_secure: bool) -> Self {
        Self {
            token: token.into(),
            enabled,
            cookie_secure,
        }
    }

    /// The capability.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Whether the gate is closed.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Whether a path answers without a capability.
    pub fn is_capability_free(path: &str) -> bool {
        CAPABILITY_FREE.contains(&path)
    }

    /// The `Set-Cookie` header that hands the capability to the browser.
    pub fn capability_cookie(&self) -> String {
        format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict{}",
            AUTH_COOKIE,
            self.token,
            if self.cookie_secure { "; Secure" } else { "" }
        )
    }

    /// Does the request carry the capability?
    ///
    /// Three ways to present it, exactly as the reference accepts them: the
    /// `Authorization: Bearer` header, the `X-Stride-Token` header, and the
    /// cookie.
    pub fn provided_token(&self, request: &Request) -> Option<(String, bool)> {
        if let Some(authorization) = request.headers.get("authorization") {
            if let Some(token) = authorization.strip_prefix("Bearer ") {
                let token = token.trim();
                if !token.is_empty() {
                    return Some((token.to_string(), true));
                }
            }
            if let Some(token) = authorization.strip_prefix("bearer ") {
                let token = token.trim();
                if !token.is_empty() {
                    return Some((token.to_string(), true));
                }
            }
        }
        if let Some(token) = request.headers.get("x-stride-token") {
            let token = token.trim();
            if !token.is_empty() {
                return Some((token.to_string(), true));
            }
        }
        request
            .cookie(AUTH_COOKIE)
            .map(|token| (token.to_string(), false))
    }

    /// Is the request authorised?
    ///
    /// `Ok(true)` when the request carries the capability, `Ok(false)` when
    /// the path does not need one, and an error when the request must be
    /// refused.
    pub fn authorize(&self, request: &Request, path: &str) -> Result<bool, GateError> {
        if !self.enabled {
            return Ok(false);
        }
        if Self::is_capability_free(path) {
            return Ok(false);
        }
        let provided = self.provided_token(request);
        let authorised = provided
            .as_ref()
            .is_some_and(|(token, _)| constant_time_eq(token, &self.token));
        if !authorised {
            return Err(GateError::Unauthenticated);
        }
        Ok(true)
    }

    /// May this request change something?
    ///
    /// An explicit bearer header is authority the caller chose to present, so
    /// it is accepted from anywhere. A cookie is ambient authority, so it is
    /// only accepted from this origin.
    pub fn allows_mutation(&self, request: &Request) -> bool {
        let explicit = self
            .provided_token(request)
            .is_some_and(|(_, explicit)| explicit);
        if explicit {
            return true;
        }
        same_origin(request.headers.get("origin"), request.host())
    }
}

/// The ways the gate refuses a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// The request carries no capability.
    Unauthenticated,
    /// The request changes something and does not come from this origin.
    CrossOrigin,
}

impl GateError {
    /// The status the reference answers with.
    pub fn status(&self) -> u16 {
        match self {
            GateError::Unauthenticated => 401,
            GateError::CrossOrigin => 403,
        }
    }

    /// The `detail` the reference answers with.
    pub fn detail(&self) -> &'static str {
        match self {
            GateError::Unauthenticated => "Capability token required.",
            GateError::CrossOrigin => "Same-origin request required.",
        }
    }
}

/// Compare two secrets without leaking their length through the timing.
pub fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    difference == 0
}

/// Is this address on the loopback interface?
pub fn is_loopback_address(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

/// Does the `Host` header name a loopback host?
///
/// The peer address is not enough on its own: a page the athlete happens to
/// have open can point a name it controls at 127.0.0.1 and then talk to
/// Stride as the origin it came from. Both have to say loopback.
pub fn host_is_loopback(host: &str) -> bool {
    let host = host.trim();
    if host.is_empty() {
        return false;
    }
    let name = if let Some(rest) = host.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else if host.matches(':').count() > 1 {
        host
    } else {
        host.split(':').next().unwrap_or("")
    };
    if name.is_empty() {
        return false;
    }
    if name.eq_ignore_ascii_case("localhost") {
        return true;
    }
    name.parse::<IpAddr>()
        .is_ok_and(|address| is_loopback_address(&address))
}

/// Is the request genuinely from this machine?
pub fn is_loopback_client(request: &Request) -> bool {
    is_loopback_address(&request.peer) && host_is_loopback(request.host())
}

/// Does the `Origin` header name this host?
///
/// A request with **no** `Origin` header is not same-origin: the browser
/// sends `Origin` on every request that is not a `GET` or a `HEAD`, so a
/// mutation without one is a client that is not the page — `curl`, or a
/// form post from somewhere else. The reference refuses those, and so does
/// this.
pub fn same_origin(origin: Option<&str>, host: &str) -> bool {
    let Some(origin) = origin else {
        return false;
    };
    let origin = origin.trim().trim_end_matches('/');
    if origin.is_empty() {
        return true;
    }
    let Some((scheme, rest)) = origin.split_once("://") else {
        return false;
    };
    if scheme.is_empty() {
        return false;
    }
    let origin_host = rest.split('/').next().unwrap_or("");
    !origin_host.is_empty() && origin_host.eq_ignore_ascii_case(host.trim())
}

/// Remove a credential from a string that is about to be shown.
///
/// The reference redacts every answer that can carry an exception detail
/// (`redacted_http_exception_handler`), because an upstream error can quote
/// the key it was given. The one credential the desktop can see is the
/// Garmin Personal API key, and its shape is known.
pub fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("GCDC_") {
        out.push_str(&rest[..start]);
        out.push_str("<redacted>");
        let after = &rest[start + 5..];
        let end = after
            .find(|character: char| {
                !(character.is_ascii_alphanumeric()
                    || character == '.'
                    || character == '_'
                    || character == '-')
            })
            .unwrap_or(after.len());
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// A fixed-window rate limiter, per client and per bucket.
///
/// The reference limits the paths that cost money — a Garmin sync, a pairing
/// code — and a pairing code is a secret being guessed.
/// The buckets of the limiter: the client, the bucket, the count and the
/// moment the window opened.
type Buckets = HashMap<(String, &'static str), (usize, Instant)>;

#[derive(Debug, Default)]
pub struct RateLimiter {
    buckets: std::sync::Mutex<Buckets>,
}

impl RateLimiter {
    /// A limiter with no buckets.
    pub fn new() -> Self {
        Self::default()
    }

    /// The bucket a path belongs to, if it is limited at all.
    pub fn bucket(path: &str) -> Option<(&'static str, usize)> {
        if path == "/api/v1/sync/run" || path == "/api/v1/sync/history" {
            return Some(("sync", 4));
        }
        if path.starts_with("/api/v1/auth/garmin/") {
            return Some(("garmin-auth", 10));
        }
        if path == "/api/v1/auth/pair" {
            return Some(("pair", 10));
        }
        None
    }

    /// Is the request allowed?
    pub fn allow(&self, client: &str, bucket: &'static str, limit: usize) -> bool {
        let now = Instant::now();
        let mut buckets = match self.buckets.lock() {
            Ok(buckets) => buckets,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = buckets
            .entry((client.to_string(), bucket))
            .or_insert((0, now));
        // The window is a minute, counted from the first request in it.
        if now.duration_since(entry.1) > Duration::from_secs(60) {
            *entry = (0, now);
        }
        if entry.0 >= limit {
            return false;
        }
        entry.0 += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Headers, Request};

    fn request(peer: &str, host: &str, headers: &[(&str, &str)]) -> Request {
        let mut head = Headers::new();
        head.push("Host", host);
        for (name, value) in headers {
            head.push(name, value);
        }
        Request {
            method: "GET".to_string(),
            target: "/".to_string(),
            path: "/".to_string(),
            query: String::new(),
            headers: head,
            body: Vec::new(),
            peer: peer.parse().unwrap_or(IpAddr::from([127, 0, 0, 1])),
        }
    }

    #[test]
    fn a_loopback_client_is_recognised() {
        assert!(is_loopback_client(&request(
            "127.0.0.1",
            "127.0.0.1:8000",
            &[]
        )));
        assert!(is_loopback_client(&request(
            "127.0.0.1",
            "localhost:8000",
            &[]
        )));
        assert!(!is_loopback_client(&request(
            "127.0.0.1",
            "attacker.example",
            &[]
        )));
        assert!(!is_loopback_client(&request(
            "100.89.222.50",
            "127.0.0.1:8000",
            &[]
        )));
    }

    #[test]
    fn the_gate_refuses_a_request_with_no_capability() {
        let security = Security::new("a-capability-token-that-is-long-enough", true, false);
        let request = request("127.0.0.1", "127.0.0.1:8000", &[]);
        assert_eq!(
            security.authorize(&request, "/api/v1/activities"),
            Err(GateError::Unauthenticated)
        );
        assert!(security.authorize(&request, "/api/v1/health").is_ok());
    }

    #[test]
    fn a_mutation_with_no_origin_is_refused() {
        let security = Security::new("a-capability-token-that-is-long-enough", true, false);
        let cookie = request(
            "127.0.0.1",
            "127.0.0.1:8000",
            &[(
                "Cookie",
                "stride_capability=a-capability-token-that-is-long-enough",
            )],
        );
        assert!(
            !security.allows_mutation(&cookie),
            "a cookie with no origin"
        );
    }

    #[test]
    fn a_mutation_needs_the_origin_or_a_bearer() {
        let security = Security::new("a-capability-token-that-is-long-enough", true, false);
        let cookie = request(
            "127.0.0.1",
            "127.0.0.1:8000",
            &[(
                "Cookie",
                "stride_capability=a-capability-token-that-is-long-enough",
            )],
        );
        assert!(
            !security.allows_mutation(&cookie),
            "a cookie from elsewhere"
        );
        let same = request(
            "127.0.0.1",
            "127.0.0.1:8000",
            &[
                (
                    "Cookie",
                    "stride_capability=a-capability-token-that-is-long-enough",
                ),
                ("Origin", "http://127.0.0.1:8000"),
            ],
        );
        assert!(security.allows_mutation(&same), "the same origin");
        let bearer = request(
            "127.0.0.1",
            "127.0.0.1:8000",
            &[(
                "Authorization",
                "Bearer a-capability-token-that-is-long-enough",
            )],
        );
        assert!(security.allows_mutation(&bearer), "an explicit bearer");
    }

    #[test]
    fn the_rate_limiter_counts_a_window() {
        let limiter = RateLimiter::new();
        assert!(limiter.allow("127.0.0.1", "pair", 2));
        assert!(limiter.allow("127.0.0.1", "pair", 2));
        assert!(!limiter.allow("127.0.0.1", "pair", 2));
        assert!(limiter.allow("127.0.0.2", "pair", 2));
    }
}
