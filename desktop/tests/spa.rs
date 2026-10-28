#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The SPA, the static files, and the traversal hole that is not there.
//!
//! The desktop serves the built React app the way the Python server does:
//! the files in `frontend/dist` are served as they are, and every other path
//! that is not under `/api/` is served the app's `index.html`.
//!
//! The security audit found a live traversal hole in exactly that
//! behaviour, so the test that matters is the one that walks out of the
//! directory and finds nothing.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use stride_api::{Api, CoreBackend, MemBackend};
use stride_desktop::http::{Headers, Request};
use stride_desktop::security::Security;
use stride_desktop::server::App;

/// A directory that looks like a built app.
///
/// Every test gets its own directory: the tests run in parallel, and a
/// shared one would have them removing each other's app.
fn dist(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stride-desktop-spa-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).expect("the directory");
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>Stride</title>",
    )
    .expect("the index");
    std::fs::write(dir.join("assets/app.js"), "console.log(1)").expect("the asset");
    std::fs::write(dir.join("assets/app.css"), "body{}").expect("the asset");
    dir
}

/// The application over the directory.
fn app(dist: &Path) -> App {
    let backend = Arc::new(CoreBackend::new(Arc::new(MemBackend::new())));
    let api = Api::new(backend);
    let security = Security::new("a-capability-token-that-is-long-enough", true, false);
    App::new(api, security, dist.to_path_buf(), Vec::new())
}

/// A request from the loopback browser.
fn request(method: &str, path: &str) -> Request {
    let mut headers = Headers::new();
    headers.push("Host", "127.0.0.1:8000");
    Request {
        method: method.to_string(),
        target: path.to_string(),
        path: path.to_string(),
        query: String::new(),
        headers,
        body: Vec::new(),
        peer: IpAddr::from([127, 0, 0, 1]),
    }
}

/// The body of an answer.
fn body(response: &stride_desktop::http::Response) -> String {
    String::from_utf8_lossy(&response.body).into_owned()
}

#[test]
fn the_root_is_the_app() {
    let dist = dist("the_root_is_the_app");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/"));
    assert_eq!(response.status, 200);
    assert_eq!(
        response.get("Content-Type"),
        Some("text/html; charset=utf-8")
    );
    assert!(body(&response).contains("<title>Stride</title>"));
}

#[test]
fn an_unknown_path_is_the_app() {
    let dist = dist("an_unknown_path_is_the_app");
    let app = app(&dist);
    for path in ["/activities/42", "/fitness", "/plan/today", "/settings"] {
        let response = app.handle(&request("GET", path));
        assert_eq!(response.status, 200, "{path}");
        assert!(
            body(&response).contains("<title>Stride</title>"),
            "{path} did not serve the app"
        );
    }
}

#[test]
fn a_static_file_is_served_with_its_content_type() {
    let dist = dist("a_static_file_is_served_with_its_content_type");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/assets/app.js"));
    assert_eq!(response.status, 200);
    assert_eq!(
        response.get("Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(body(&response), "console.log(1)");

    let response = app.handle(&request("GET", "/assets/app.css"));
    assert_eq!(
        response.get("Content-Type"),
        Some("text/css; charset=utf-8")
    );
}

#[test]
fn a_traversal_returns_nothing_outside_the_directory() {
    let dist = dist("a_traversal_returns_nothing_outside_the_directory");
    // The secret sits next to the app, not inside it.
    let outside = dist
        .parent()
        .expect("a parent")
        .join("stride-desktop-secret.txt");
    std::fs::write(&outside, "the Garmin key").expect("the secret");

    let app = app(&dist);
    for path in [
        "/../stride-desktop-secret.txt",
        "/assets/../../stride-desktop-secret.txt",
        "/../../etc/passwd",
        "/%2e%2e/stride-desktop-secret.txt",
    ] {
        let response = app.handle(&request("GET", path));
        let text = body(&response);
        assert!(
            !text.contains("the Garmin key"),
            "{path} served the file outside the directory"
        );
        assert!(!text.contains("root:"), "{path} served /etc/passwd");
        assert_eq!(response.status, 200, "{path}");
    }
    std::fs::remove_file(&outside).ok();
}

#[test]
fn an_unknown_api_path_is_a_json_404() {
    let dist = dist("an_unknown_api_path_is_a_json_404");
    let app = app(&dist);
    // Without the capability the gate answers first, exactly as the
    // reference's middleware does.
    let response = app.handle(&request("GET", "/api/v1/nothing-here"));
    assert_eq!(response.status, 401);

    let mut authorised = request("GET", "/api/v1/nothing-here");
    authorised.headers.push(
        "Cookie",
        "stride_capability=a-capability-token-that-is-long-enough",
    );
    let response = app.handle(&authorised);
    assert_eq!(response.status, 404);
    assert_eq!(response.get("Content-Type"), Some("application/json"));
    assert_eq!(body(&response), "{\"detail\":\"Not found\"}");
}

#[test]
fn a_post_to_an_unknown_path_is_a_405() {
    let dist = dist("a_post_to_an_unknown_path_is_a_405");
    let app = app(&dist);
    // The reference's catch-all is a `GET` route.
    let response = app.handle(&request("POST", "/somewhere"));
    assert_eq!(response.status, 405);
    assert_eq!(body(&response), "{\"detail\":\"Method Not Allowed\"}");
}

#[test]
fn the_api_needs_the_capability() {
    let dist = dist("the_api_needs_the_capability");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/api/v1/activities"));
    assert_eq!(response.status, 401);
    assert_eq!(
        body(&response),
        "{\"detail\":\"Capability token required.\"}"
    );
}

#[test]
fn the_health_route_answers_without_the_capability() {
    let dist = dist("the_health_route_answers_without_the_capability");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/api/v1/health"));
    assert_eq!(response.status, 200);
    assert!(
        body(&response).contains("\"ok\":true"),
        "{}",
        body(&response)
    );
}

#[test]
fn the_bootstrap_hands_the_browser_its_capability() {
    let dist = dist("the_bootstrap_hands_the_browser_its_capability");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/api/v1/bootstrap"));
    assert_eq!(response.status, 204);
    let cookie = response.get("Set-Cookie").expect("the cookie");
    assert!(cookie.starts_with("stride_capability="), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
}

#[test]
fn the_app_mints_the_cookie_for_the_loopback_browser() {
    let dist = dist("the_app_mints_the_cookie_for_the_loopback_browser");
    let app = app(&dist);
    let response = app.handle(&request("GET", "/"));
    assert!(
        response.get("Set-Cookie").is_some(),
        "the browser was not given a capability"
    );

    // A request that is not from this machine is not given one.
    let mut remote = request("GET", "/");
    remote.peer = IpAddr::from([100, 89, 222, 50]);
    let response = app.handle(&remote);
    assert!(response.get("Set-Cookie").is_none());
}
