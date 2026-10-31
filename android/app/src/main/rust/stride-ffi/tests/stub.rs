//! The seam's own rules, proved.
//!
//! The stub is what the APK carries while the core is not in the tree, and
//! the rule it has to keep is simple: **a stub can refuse; it cannot be
//! wrong.** There is no route it answers with a number.

// The tests are allowed to expect: they are a separate crate, not a
// `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::Value;

#[test]
fn the_stub_refuses_every_route() {
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
    let answer = uniffi_stride::stride_status();
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["body"]["backend"], serde_json::json!("stub"));
    assert_eq!(parsed["body"]["open"], serde_json::json!(false));
}
