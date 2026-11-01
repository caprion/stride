//! The Garmin key never leaves.
//!
//! The key is an **argument**: it arrives, it is used for the calls that
//! need it, and it is not returned, not logged and not embedded in an
//! error. The phone's key is unwrapped in the native layer; the desktop's
//! key stays in the user's key store. Neither is ever in a response.
//!
//! This test puts a distinctive key through every route — every route,
//! not the ones that are supposed to take it — and asserts that it appears
//! nowhere: not in a body, not in an error, not in the log.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::{Arc, Mutex};

use common::GARMIN_KEY;

/// Every route, with the method and the body it takes.
fn calls() -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    vec![
        ("GET", "/activities", None),
        ("GET", "/activities/1", None),
        ("GET", "/physiology/daily", None),
        ("GET", "/physiology/sleep", None),
        ("GET", "/physiology/hrv", None),
        ("GET", "/physiology/vitals", None),
        ("GET", "/physiology/latest", None),
        ("GET", "/physiology/sources", None),
        ("GET", "/fitness/fingerprint", None),
        ("GET", "/fitness/trends", None),
        ("GET", "/fitness/zones", None),
        ("GET", "/stats/summary", None),
        ("GET", "/stats/weekly", None),
        ("GET", "/stats/pace-trends", None),
        ("GET", "/stats/elevation", None),
        ("GET", "/stats/personal-records", None),
        ("GET", "/plan/current", None),
        ("GET", "/plan/today", None),
        ("GET", "/plan/week", None),
        ("GET", "/plan/blocks", None),
        ("GET", "/plan/sources", None),
        ("GET", "/plan/adherence", None),
        ("GET", "/plan/results", None),
        ("POST", "/plan/import", Some("{}")),
        ("GET", "/insights/today", None),
        ("GET", "/insights/progress", None),
        ("GET", "/insights/chips?id=1", None),
        ("GET", "/compare?a=1&b=2", None),
        ("GET", "/compare/candidates", None),
        ("GET", "/health", None),
        ("GET", "/health/daily", None),
        ("GET", "/sync/status", None),
        ("POST", "/sync/run", Some("{}")),
        ("GET", "/sync/runs", None),
        ("GET", "/sync/history", None),
        ("GET", "/sync/settings", None),
        ("GET", "/auth/status", None),
        (
            "POST",
            "/auth/garmin/personal-key",
            Some("{\"key\":\"the key in the body\"}"),
        ),
        (
            "POST",
            "/auth/garmin/personal-key/test",
            Some("{\"key\":\"the key in the body\"}"),
        ),
        ("POST", "/auth/garmin/fit-import", Some("{}")),
        ("POST", "/auth/garmin/logout", Some("{}")),
        ("GET", "/calendar", None),
        ("GET", "/settings", None),
        ("POST", "/settings", Some("{}")),
        ("GET", "/notes", None),
        ("POST", "/notes", Some("{}")),
        ("POST", "/data/export", Some("{}")),
        ("POST", "/data/import", Some("{}")),
    ]
}

#[test]
fn no_answer_carries_the_key() {
    let harness = common::Harness::seeded("key-never-leaves");
    harness.router.api().set_garmin_key(GARMIN_KEY);

    for (method, path, body) in calls() {
        let answer = harness.router.call(method, path, body);
        let text = answer.body.to_string();
        assert!(
            !text.contains(GARMIN_KEY),
            "the key is in the answer of {path}: {text}"
        );
        let envelope = answer.envelope().to_string();
        assert!(
            !envelope.contains(GARMIN_KEY),
            "the key is in the envelope of {path}: {envelope}"
        );
    }
}

#[test]
fn no_answer_carries_the_key_that_came_in_the_body() {
    let harness = common::Harness::seeded("key-in-the-body");
    let key = "the-key-that-came-in-the-body-0000";
    for (method, path, body) in calls() {
        let body = body.map(|text| text.replace("the key in the body", key));
        let answer = harness.router.call(method, path, body.as_deref());
        let text = answer.body.to_string();
        assert!(
            !text.contains(key),
            "the key is in the answer of {path}: {text}"
        );
    }
}

#[test]
fn no_log_line_carries_the_key() {
    let harness = common::Harness::seeded("key-never-logged");
    harness.router.api().set_garmin_key(GARMIN_KEY);

    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    stride_store::logging::set_sink(Arc::new(move |line: &str| {
        if let Ok(mut guard) = sink.lock() {
            guard.push(line.to_string());
        }
    }));

    for (method, path, body) in calls() {
        harness.router.call(method, path, body);
    }
    stride_store::logging::clear_sink();

    let guard = lines
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for line in guard.iter() {
        assert!(
            !line.contains(GARMIN_KEY),
            "the key is in a log line: {line}"
        );
    }
}

#[test]
fn the_key_is_not_readable_back_out_of_the_state() {
    let harness = common::Harness::empty("key-not-readable");
    harness.router.api().set_garmin_key(GARMIN_KEY);
    // The state says the key is there, and says nothing else about it.
    assert!(harness.router.api().garmin_connected());
    let printed = format!("{:?}", harness.router.api());
    assert!(
        !printed.contains(GARMIN_KEY),
        "the key is in the state's own output: {printed}"
    );
    let key = stride_api::GarminKey::new(GARMIN_KEY);
    let printed = format!("{key:?}");
    assert!(
        !printed.contains(GARMIN_KEY),
        "the key is in the key's own output: {printed}"
    );
}

#[test]
fn the_key_is_forgotten_when_it_is_taken_out() {
    let harness = common::Harness::empty("key-forgotten");
    harness.router.api().set_garmin_key(GARMIN_KEY);
    assert!(harness.router.api().garmin_connected());
    let answer = harness
        .router
        .call("POST", "/auth/garmin/logout", Some("{}"));
    assert!(answer.is_ok());
    assert!(!harness.router.api().garmin_connected());
    assert_eq!(answer.body["connected"], serde_json::json!(false));
}
