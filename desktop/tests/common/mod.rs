//! The helpers the tests share.
//!
//! Every test gets its own server, and the server is built on top of a
//! **fake router** — one that answers with a known body. That is what makes
//! the "one implementation" test possible: the desktop's answer has to be
//! the router's answer, byte for byte, and a router that says something
//! fixed is the only way to say that.

#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr};

use stride_desktop::api::{Request, Response, Router};
use stride_desktop::http;
use stride_desktop::server::Server;
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;

/// A router that answers with a known body.
///
/// It is not a stub that refuses: it answers every route, so a test can see
/// that the desktop passed the answer on without touching it.
#[derive(Debug, Clone, Copy, Default)]
pub struct FakeRouter;

/// The body the fake router answers with.
pub const FAKE_BODY: &str = "{\"ok\":true,\"status\":200,\"body\":{\"route\":\"the fake router\"}}";

impl Router for FakeRouter {
    fn call(&self, request: &Request<'_>) -> Response {
        Response::json(
            200,
            &format!(
                "{{\"ok\":true,\"status\":200,\"body\":{{\"route\":\"the fake router\",\"path\":\"{}\"}}}}",
                request.path
            ),
        )
    }
}

/// A server with the fake router behind it.
pub fn server(trust: Trust) -> Server {
    Server::new(trust, std::sync::Arc::new(FakeRouter), Spa::new(None))
}

/// A server that answers every route with the fixed body.
pub fn server_with_fixed_router(trust: Trust) -> Server {
    Server::new(trust, std::sync::Arc::new(FixedRouter), Spa::new(None))
}

/// A router that answers with exactly one body.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedRouter;

impl Router for FixedRouter {
    fn call(&self, _request: &Request<'_>) -> Response {
        Response::json(200, FAKE_BODY)
    }
}

/// A request, as the rig would read it.
///
/// The path is decoded, because the rig decodes it: the guard looks at the
/// path the request really names, not at the one it was spelled with.
pub fn request(method: &str, target: &str) -> http::Request {
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (target.to_string(), String::new()),
    };
    http::Request {
        method: method.to_string(),
        target: target.to_string(),
        path: stride_desktop::http::percent_decode(&path),
        query: stride_desktop::http::percent_decode(&query),
        headers: Vec::new(),
        body: Vec::new(),
    }
}

/// A request with a body.
pub fn request_with_body(method: &str, target: &str, body: &str) -> http::Request {
    let mut request = request(method, target);
    request.body = body.as_bytes().to_vec();
    request
}

/// An address on the loopback.
pub fn loopback() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

/// An address on a network, for the tests that need one.
pub fn elsewhere() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5))
}

/// An address inside `10.0.0.0/8`.
pub fn inside_the_trusted_network() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7))
}

/// The body of a response, as text.
pub fn body_of(response: &http::Response) -> String {
    String::from_utf8_lossy(&response.body).into_owned()
}
