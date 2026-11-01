//! One implementation.
//!
//! **The desktop's responses are the router's responses — the same DTOs,
//! byte for byte.**
//!
//! The rig is a host: it opens the door, it decides who is let in, and it
//! serves the app. It does not have an opinion about a number, and it must
//! not reshape one. So the router's answer — the status, the content type
//! and the body — is what comes back, untouched.
//!
//! The router here is a fake one that answers with a known body, because a
//! router that says something fixed is the only way to say that the answer
//! came from the router and not from the rig.

mod common;

use stride_desktop::api::{Request, Response, Router};
use stride_desktop::error::Result;
use stride_desktop::trust::Trust;

use common::{body_of, loopback, request, server};

/// A router that answers with a status, a content type and a body.
#[derive(Debug, Clone, Copy)]
struct QuietRouter;

impl Router for QuietRouter {
    fn call(&self, _request: &Request<'_>) -> Response {
        Response::new(
            418,
            "application/vnd.stride.test",
            b"the router said this, and nothing else".to_vec(),
        )
    }
}

#[test]
fn the_answer_is_the_router_answer_byte_for_byte() -> Result<()> {
    let server = stride_desktop::server::Server::new(
        Trust::loopback_only(),
        std::sync::Arc::new(QuietRouter),
        stride_desktop::spa::Spa::new(None),
    );
    let response = server.handle(&request("GET", "/api/v1/activities"), loopback());

    assert_eq!(response.status, 418, "the status is the router's");
    assert_eq!(
        body_of(&response),
        "the router said this, and nothing else",
        "the body is the router's, byte for byte"
    );
    let content_type = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    assert_eq!(content_type, "application/vnd.stride.test");
    Ok(())
}

#[test]
fn the_router_is_asked_for_the_path_the_app_asked_for() -> Result<()> {
    // The contract's rule: the app asks for /api/v1/..., and the router
    // matches on the path without the query string. The rig hands the path
    // on as it is — the prefix included — and takes the query string off,
    // because the query string is not part of the route.
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/api/v1/activities?limit=3"), loopback());
    let body = body_of(&response);
    assert!(
        body.contains("\"path\":\"/api/v1/activities\""),
        "the router was asked for the wrong path: {body}"
    );
    Ok(())
}

#[test]
fn a_path_that_is_not_a_route_is_the_apps() -> Result<()> {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/runs"), loopback());
    let body = body_of(&response);
    assert!(
        !body.contains("the fake router"),
        "the app's own route went to the router: {body}"
    );
    Ok(())
}

#[test]
fn the_rig_adds_nothing_to_the_answer() -> Result<()> {
    // The fixed router answers with one body. Whatever the rig does around
    // it — the trust gate, the log, the keep-alive — the body that comes
    // back is that body and nothing else.
    let server = common::server_with_fixed_router(Trust::loopback_only());
    let response = server.handle(&request("GET", "/api/v1/insights/today"), loopback());
    assert_eq!(response.status, 200);
    assert_eq!(body_of(&response), common::FAKE_BODY);
    Ok(())
}
