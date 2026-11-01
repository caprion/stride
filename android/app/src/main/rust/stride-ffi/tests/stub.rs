//! The stub's own rules, proved.
//!
//! The stub is not what the phone runs any more — the real core is — but
//! it is still here, and it is still honest: **a stub can refuse; it cannot
//! be wrong.** Every route it answers with a number is a route it must
//! never answer.
//!
//! The stub is put in front of the boundary on purpose: the phone's core
//! is the real one, and this is the test that says the stub still refuses.

// The tests are allowed to expect: they are a separate crate, not a
// `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::Value;

/// Put the stub in front of the boundary.
///
/// The core is process-wide, and the phone runs the **real** one — so the
/// stub has to be installed for the test that is about the stub. Every test
/// in this file installs the same stub, so they cannot disagree.
fn the_stub() {
    uniffi_stride::install(Arc::new(uniffi_stride::stub::StubCore::new()));
}

#[test]
fn the_stub_refuses_every_route() {
    the_stub();
    let answer = uniffi_stride::stride_call(
        "GET".to_string(),
        "/activities?per_page=1".to_string(),
        None,
    );
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], serde_json::json!(false));
    assert_eq!(parsed["status"], serde_json::json!(503));
    let detail = parsed["error"]["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("not in this build"),
        "the stub says what it is: {detail}"
    );
}

#[test]
fn the_stub_never_echoes_a_key() {
    the_stub();
    let answer = uniffi_stride::stride_set_garmin_key("the-garmin-key".to_string());
    assert!(
        !answer.contains("the-garmin-key"),
        "the key is never echoed back: {answer}"
    );
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["body"]["in_memory"], serde_json::json!(true));
}

#[test]
fn the_stub_says_it_is_a_stub() {
    the_stub();
    let answer = uniffi_stride::stride_status();
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["body"]["backend"], serde_json::json!("stub"));
    assert_eq!(parsed["body"]["open"], serde_json::json!(false));
}
