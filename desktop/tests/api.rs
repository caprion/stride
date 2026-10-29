#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The API through the desktop: the capability, the origin and the body.
//!
//! The desktop serves the router over HTTP, and the HTTP layer has three
//! rules of its own that the router knows nothing about:
//!
//! * a request needs the capability, unless it is one of the paths that
//!   answer without one;
//! * a request that **changes** something has to come from this origin —
//!   an explicit bearer header is authority the caller chose to present, a
//!   cookie is ambient authority and is only accepted from this origin;
//! * the body is passed to the router as it arrived.
//!
//! The reference's security middleware does the same three things, and the
//! comparison against the running reference confirmed that it does.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use stride_api::backend::ActivityRow;
use stride_api::{Api, CoreBackend, MemBackend};
use stride_desktop::http::{Headers, Request};
use stride_desktop::security::Security;
use stride_desktop::server::App;

/// The capability the gate expects.
const TOKEN: &str = "a-capability-token-that-is-long-enough";

/// An application over one run.
fn app() -> App {
    let backend = MemBackend::new();
    backend.add_activity(ActivityRow {
        id: 7,
        source: "garmin_fit".to_string(),
        external_id: "3117001".to_string(),
        name: "morning run".to_string(),
        sport_type: "running".to_string(),
        local_date: "2026-03-02".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        ..ActivityRow::default()
    });
    let api = Api::new(Arc::new(CoreBackend::new(Arc::new(backend))));
    let security = Security::new(TOKEN, true, false);
    App::new(api, security, PathBuf::from("/nonexistent"), Vec::new())
}

/// A request from the loopback browser.
fn request(method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Request {
    let mut head = Headers::new();
    head.push("Host", "127.0.0.1:8000");
    for (name, value) in headers {
        head.push(name, value);
    }
    Request {
        method: method.to_string(),
        path: path.to_string(),
        query: String::new(),
        headers: head,
        body: body.as_bytes().to_vec(),
        peer: IpAddr::from([127, 0, 0, 1]),
    }
}

/// The body of an answer.
fn body(response: &stride_desktop::http::Response) -> serde_json::Value {
    serde_json::from_slice(&response.body).unwrap_or(serde_json::Value::Null)
}

#[test]
fn a_mutation_needs_the_capability() {
    let app = app();
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/7",
        &[("Origin", "http://127.0.0.1:8000")],
        r#"{"name":"renamed"}"#,
    ));
    assert_eq!(response.status, 401);
    assert_eq!(body(&response)["detail"], "Capability token required.");
}

#[test]
fn a_mutation_needs_the_origin() {
    let app = app();
    // The cookie is ambient authority: it is only accepted from this
    // origin. A request with no `Origin` is not the page.
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/7",
        &[("Cookie", &format!("stride_capability={TOKEN}"))],
        r#"{"name":"renamed"}"#,
    ));
    assert_eq!(response.status, 403);
    assert_eq!(body(&response)["detail"], "Same-origin request required.");

    // The same request from this origin is allowed.
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/7",
        &[
            ("Cookie", &format!("stride_capability={TOKEN}")),
            ("Origin", "http://127.0.0.1:8000"),
        ],
        r#"{"name":"renamed"}"#,
    ));
    assert_eq!(response.status, 200);
}

#[test]
fn an_explicit_bearer_is_allowed_from_anywhere() {
    let app = app();
    // A bearer header is authority the caller chose to present, so it does
    // not have to come from this origin.
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/7",
        &[("Authorization", &format!("Bearer {TOKEN}"))],
        r#"{"name":"renamed"}"#,
    ));
    assert_eq!(response.status, 200);
}

#[test]
fn the_change_is_the_change_the_body_asks_for() {
    let app = app();
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/7",
        &[
            ("Cookie", &format!("stride_capability={TOKEN}")),
            ("Origin", "http://127.0.0.1:8000"),
        ],
        r#"{"name":"renamed","description":"felt good","is_trail":true}"#,
    ));
    assert_eq!(response.status, 200);
    let answer = body(&response);
    assert_eq!(answer["name"], "renamed");
    assert_eq!(answer["description"], "felt good");
    assert_eq!(answer["is_trail"], true);
    assert_eq!(answer["id"], 7);

    // And it is the change that is kept: the run reads back renamed.
    let response = app.handle(&request(
        "GET",
        "/api/v1/activities/7",
        &[("Cookie", &format!("stride_capability={TOKEN}"))],
        "",
    ));
    assert_eq!(response.status, 200);
    assert_eq!(body(&response)["name"], "renamed");
}

#[test]
fn a_mutation_that_names_no_run_is_a_404() {
    let app = app();
    let response = app.handle(&request(
        "PATCH",
        "/api/v1/activities/999",
        &[
            ("Cookie", &format!("stride_capability={TOKEN}")),
            ("Origin", "http://127.0.0.1:8000"),
        ],
        r#"{"name":"renamed"}"#,
    ));
    assert_eq!(response.status, 404);
}

#[test]
fn the_capability_free_paths_answer_without_one() {
    let app = app();
    for path in [
        "/api/v1/health",
        "/api/v1/auth/pair-code",
        "/api/v1/auth/pair",
    ] {
        let response = app.handle(&request("GET", path, &[], ""));
        assert_ne!(response.status, 401, "{path}");
    }
}

#[test]
fn the_answers_are_the_shapes_the_app_consumes() {
    let app = app();
    let response = app.handle(&request(
        "GET",
        "/api/v1/activities",
        &[("Cookie", &format!("stride_capability={TOKEN}"))],
        "",
    ));
    assert_eq!(response.status, 200);
    let answer = body(&response);
    for key in ["items", "total", "page", "per_page", "pages"] {
        assert!(answer.get(key).is_some(), "the list is missing {key}");
    }
    let item = &answer["items"][0];
    for key in [
        "id",
        "name",
        "distance_m",
        "moving_time_s",
        "local_date",
        "avg_hr",
        "sport_type",
    ] {
        assert!(item.get(key).is_some(), "the run is missing {key}");
    }
}
