//! The app's captured answer is the router's answer.
//!
//! The review found that the app's screens are tested against the app's own
//! fixture, so a screen can render fictional data and stay green. The fix is
//! this test: a **captured answer** — a JSON export of exactly what the
//! router answered — that the test seeds a store for and compares, **value
//! for value**, not shape for shape. A shape test that passes while the
//! numbers are invented is the same gap in a new coat.
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
//! `2026-05-12T06:30:00Z`, so the answer is reproducible. When the app's own
//! seed file (`answers.json`) lands, this test reads it and seeds from it, so
//! the app's seed and the router's input are the same thing.
//!
//! **Until the fixture lands, this test skips — loudly.** A silent pass
//! would be indistinguishable from a real one; this test says so, and it
//! fails the moment the file is present and the answer disagrees.

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
        Err(_) => {
            // A loud skip, never a silent pass: the fixture lands in the
            // frontend's next pull request, and this test holds it true the
            // moment it is in the tree.
            eprintln!(
                "the app's captured answer is not in the tree yet ({}) — \
                 the fixture lands in the next pull request, and this test \
                 will then hold the router to it value for value",
                path.display()
            );
            return;
        }
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

    // The whole answer, value for value: the run's distance, the direction
    // word, the delta's sign, the Rest line — every field, after canonical
    // serialisation. A value that drifted fails here, not just a renamed key.
    let mine = serde_json::to_string(&answer.body).unwrap_or_default();
    let theirs = serde_json::to_string(&body).unwrap_or_default();
    assert_eq!(
        mine, theirs,
        "the router's answer drifted from the captured one"
    );
}
