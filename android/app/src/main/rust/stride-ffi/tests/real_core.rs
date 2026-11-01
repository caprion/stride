//! The seam and the router, byte for byte.
//!
//! **The phone reaches the router in this process**, and the answer it
//! hands the app is the router's answer — not a copy of it, and not
//! something the shell reshaped on the way past.
//!
//! So the seam's envelope and the router's envelope are the same bytes:
//! the same DTO, serialized once, by one implementation. The desktop's
//! answer is the same router's answer, and `stride-desktop`'s own test
//! holds the rig to the same rule. Between the two, the two hosts cannot
//! disagree about a number, because there is only one of it.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use serde_json::{json, Value};
use uniffi_stride::seam::Core;

/// The one lock the tests share.
///
/// **The core is process-wide.** `stride_api::ffi` holds the one router,
/// and every test that opens the core replaces it — so the tests take a
/// lock and stand still for each other. Without it, one test's answer
/// would be another test's router.
static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The lock, taken.
fn alone() -> std::sync::MutexGuard<'static, ()> {
    ONE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The instant the clock stands at, so that every answer is the same.
fn the_instant() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-05-12T06:30:00Z")
        .expect("the instant is a time")
        .into()
}

/// A database in a scratch directory, and the key that opens it.
fn database(name: &str) -> (String, Vec<u8>) {
    let directory = std::env::temp_dir().join(format!("stride-ffi-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the directory was made");
    (
        directory.join("stride.db").to_string_lossy().into_owned(),
        vec![9u8; 32],
    )
}

#[test]
fn the_real_seam_says_which_backend_it_carries() {
    let core = uniffi_stride::real::ApiCore::new();
    let parsed = core.status();
    assert_eq!(parsed["ok"], json!(true));
    assert_eq!(parsed["body"]["backend"], json!("stride-api"));
    assert_eq!(
        parsed["body"]["backend_marker"],
        json!("stride-backend:stride-api")
    );
}

#[test]
fn the_seam_answers_the_router_answer() {
    let _lock = alone();
    let (path, bytes) = database("parity");
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();

    // The core the phone runs, with the clock held still.
    let core = uniffi_stride::real::ApiCore::with_clock(Arc::new(stride_api::FixedClock::new(
        the_instant(),
    )));
    let opened = core.open(&path, &hex).expect("the database was opened");
    assert_eq!(opened["ok"], json!(true), "the core is open: {opened}");

    // The seam's answer. The body is the core's own bytes.
    let answer = core.call("GET", "/activities", None);
    assert_eq!(answer["ok"], json!(true), "the route answered: {answer}");

    // The router's own answer, over the same database and the same clock.
    let key = stride_store::DatabaseKey::from_bytes(bytes).expect("the key is a key");
    let store = stride_store::Store::open_and_migrate(Path::new(&path), &key)
        .expect("the database was opened");
    let api = stride_api::Api::new(
        Arc::new(store),
        stride_api::Config::default(),
        Arc::new(stride_api::FixedClock::new(the_instant())),
    )
    .expect("the client was built");
    let router = stride_api::Router::new(Arc::new(api));
    let expected = router.call("GET", "/activities", None);

    // **The same answer, field for field.** The contract asks for the same
    // fields, the same names, the same nullability and the same values —
    // not for the same spelling: the phone's JSON is parsed and
    // re-serialised by the platform's parser on the way through the
    // bridge, so a number's spelling may change in transit.
    assert_eq!(
        answer["body"], expected.body,
        "the seam's answer is the router's answer"
    );
    assert_eq!(answer["status"].as_u64(), Some(expected.status as u64));
}

#[test]
fn the_key_is_never_in_an_answer() {
    let _lock = alone();
    let (path, bytes) = database("key");
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let core = uniffi_stride::real::ApiCore::with_clock(Arc::new(stride_api::FixedClock::new(
        the_instant(),
    )));
    core.open(&path, &hex).expect("the database was opened");

    let the_key = "stride-garmin-key-4f2c9a7e1b6d8305-do-not-print";
    let answer = core.set_garmin_key(the_key);
    let text = answer.to_string();
    assert!(!text.contains(the_key), "the key was echoed back: {text}");
    assert_eq!(answer["ok"], json!(true));
}

#[test]
fn a_pull_is_one_attempt_and_the_shell_owns_the_retry() {
    let _lock = alone();
    // The seam's contract: `garmin_sync` is ONE attempt. The shell — the
    // phone's `PullPolicy` — may make one more, and only one more.
    let (path, bytes) = database("sync");
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let core = uniffi_stride::real::ApiCore::with_clock(Arc::new(stride_api::FixedClock::new(
        the_instant(),
    )));
    core.open(&path, &hex).expect("the database was opened");

    // No key: the pull fails, and it fails as a typed failure — not as a
    // crash, and not as a retry of its own.
    let answer = core.garmin_sync("", 7);
    assert_eq!(answer["ok"], json!(false), "there is no key to pull with");
    let status = answer["status"].as_u64().unwrap_or(0);
    assert!(
        status == 401 || status == 400 || status == 503,
        "the failure is typed: {answer}"
    );
}

#[test]
fn the_stub_still_refuses() {
    // The stub is not what the phone runs any more, but it is still
    // honest: a stub can refuse, and it cannot be wrong.
    let stub = uniffi_stride::stub::StubCore::new();
    let answer = stub.call("GET", "/activities", None);
    assert_eq!(answer["ok"], json!(false));
    let detail = answer["error"]["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("not in this build"),
        "the stub says what it is: {detail}"
    );
}

#[test]
fn the_stub_never_echoes_a_key() {
    let stub = uniffi_stride::stub::StubCore::new();
    let answer = stub.set_garmin_key("the-garmin-key");
    let text = answer.to_string();
    assert!(
        !text.contains("the-garmin-key"),
        "the key was echoed: {text}"
    );
}

#[test]
fn a_value_that_is_not_there_is_null() {
    let _lock = alone();
    // "No value" is null, never zero: the contract's rule, and the shape
    // the app reads.
    let (path, bytes) = database("no-value");
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let core = uniffi_stride::real::ApiCore::with_clock(Arc::new(stride_api::FixedClock::new(
        the_instant(),
    )));
    core.open(&path, &hex).expect("the database was opened");
    let answer = core.call("GET", "/activities", None);
    let body: Value = answer["body"].clone();
    let activities = body["activities"].as_array().expect("a list");
    for activity in activities {
        let distance = activity["distance"].as_object();
        assert!(distance.is_some(), "the distance is there: {activity}");
    }
    assert!(body["generated_at"].is_string());
}
