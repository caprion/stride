//! The FFI surface, proved.
//!
//! The phone runs this core in its own process, so the boundary is the
//! place where things go wrong: a panic must not kill the app, and the
//! panic's payload — which might be the Garmin key — must not be printed.
//!
//! The tests are allowed to panic and to expect: they are a separate
//! crate, not a `#[cfg(test)]` module, so the lint configuration does not
//! reach them. The panic below is the thing being tested.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use serde_json::{json, Value};

use stride_api::ffi;

#[test]
fn a_call_answers_with_the_envelope() {
    let harness = common::Harness::empty("ffi-envelope");
    ffi::install(stride_api::Router::new(Arc::clone(&harness.api)));

    let answer = ffi::stride_call("/activities", None);
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], json!(true));
    assert_eq!(parsed["status"], json!(200));
    assert!(parsed["body"].is_object());
    assert!(parsed["body"]["activities"].is_array());
}

#[test]
fn a_failure_answers_with_the_error_envelope() {
    let harness = common::Harness::empty("ffi-failure");
    ffi::install(stride_api::Router::new(Arc::clone(&harness.api)));

    let answer = ffi::stride_call("/nothing/here", None);
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], json!(false));
    assert_eq!(parsed["status"], json!(404));
    assert_eq!(parsed["error"]["code"], json!("not_found"));
    assert!(parsed["error"]["detail"].is_string());
}

#[test]
fn a_panic_in_the_core_is_an_error_and_not_a_kill() {
    let answer = ffi::guarded("request", || panic!("the core panicked on call"));
    assert_eq!(answer["ok"], json!(false));
    assert_eq!(answer["status"], json!(500));
    let detail = answer["error"]["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("could not be completed"),
        "the detail is a sentence the app can show: {detail}"
    );
}

#[test]
fn the_panic_payload_is_not_in_the_answer() {
    let answer = ffi::guarded("request", || panic!("the core panicked with the key in it"));
    let text = answer.to_string();
    assert!(
        !text.contains("panicked"),
        "the panic's payload is not carried back to the app: {text}"
    );
}

#[test]
fn an_answer_handed_over_as_a_pointer_is_freed() {
    let harness = common::Harness::empty("ffi-free");
    ffi::install(stride_api::Router::new(Arc::clone(&harness.api)));

    let before = ffi::answers_in_flight();
    let pointer = ffi::stride_call_ptr("GET", "/activities", None);
    assert!(
        !pointer.is_null(),
        "the answer was not handed over as a pointer"
    );
    assert_eq!(ffi::answers_in_flight(), before + 1);

    ffi::stride_free(pointer);
    assert_eq!(ffi::answers_in_flight(), before);
}

#[test]
fn freeing_a_pointer_that_is_not_ours_is_harmless() {
    let before = ffi::answers_in_flight();
    let pointer = std::ptr::null_mut::<std::ffi::c_void>();
    ffi::stride_free(pointer);
    assert_eq!(ffi::answers_in_flight(), before);
}

#[test]
fn the_key_goes_in_and_nothing_comes_out() {
    let harness = common::Harness::empty("ffi-key");
    ffi::install(stride_api::Router::new(Arc::clone(&harness.api)));

    let answer = ffi::stride_set_garmin_key(common::GARMIN_KEY);
    assert!(!answer.contains(common::GARMIN_KEY));

    let answer = ffi::stride_call("/auth/status", None);
    assert!(!answer.contains(common::GARMIN_KEY));

    let answer = ffi::stride_set_garmin_key("");
    assert!(!answer.contains(common::GARMIN_KEY));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["body"]["connected"], json!(false));
}

#[test]
fn the_status_says_what_the_core_is() {
    let harness = common::Harness::empty("ffi-status");
    ffi::install(stride_api::Router::new(Arc::clone(&harness.api)));
    let answer = ffi::stride_status();
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], json!(true));
    assert_eq!(parsed["body"]["core"], json!("stride-api"));
    assert_eq!(parsed["body"]["open"], json!(true));
}
