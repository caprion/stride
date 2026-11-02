//! The app's captured answer is the router's answer.
//!
//! The review found that the app's screens are tested against the app's own
//! fixture, so a screen can render fictional data and stay green. The fix is
//! this test: a **captured answer** — a JSON export of exactly what the
//! router answered — that the test seeds a store for and compares, byte for
//! byte.
//!
//! The capture lives at `frontend/src/fixtures/captured.json`, written by
//! the frontend thread. Its shape is:
//!
//! ```json
//! { "route": "/physiology/vitals", "body": { … the router's answer … } }
//! ```
//!
//! The store is seeded the same way every run (the `seeded` harness: one run
//! on 2026-05-10, three nights of wellness), and the clock stands still at
//! `2026-05-12T06:30:00Z`, so the answer is reproducible.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;

use serde_json::Value;

/// The path of the captured answer.
fn captured_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("frontend")
        .join("src")
        .join("fixtures")
        .join("captured.json")
}

#[test]
fn the_apps_captured_answer_is_the_routers_answer() {
    let path = captured_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => panic!(
            "the captured answer is not here yet: {} — the frontend thread \
             exports it, and this test holds it true",
            path.display()
        ),
    };
    let captured: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => panic!("the captured answer is not JSON: {}", path.display()),
    };
    let route = captured
        .get("route")
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| panic!("the captured answer has no route"));
    let body = captured
        .get("body")
        .cloned()
        .unwrap_or_else(|| panic!("the captured answer has no body"));

    // The same seed the capture was taken from.
    let harness = common::Harness::seeded("fixture-parity");
    let answer = harness.router.call("GET", route, None);
    assert!(
        answer.is_ok(),
        "the route {route} did not answer: {}",
        answer.body
    );

    // Byte for byte, after canonical serialisation: a drifted field, a
    // changed nullability or a renamed key fails here.
    let mine = serde_json::to_string(&answer.body).unwrap_or_default();
    let theirs = serde_json::to_string(&body).unwrap_or_default();
    assert_eq!(
        mine, theirs,
        "the router's answer drifted from the captured one"
    );
}
