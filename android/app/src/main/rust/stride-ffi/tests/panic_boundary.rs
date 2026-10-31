//! The panic boundary, proved.
//!
//! **A panic in the core must become an error the app can show, never a
//! process kill.** If the boundary did not hold, this test would take the
//! test binary down with it — which is exactly the failure it is here to
//! catch.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach
// them. The panic below is the thing being tested.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::{json, Value};
use uniffi_stride::seam::Core;

/// A core that panics on every call.
struct PanickingCore;

impl Core for PanickingCore {
    fn open(&self, _db_path: &str, _db_key_hex: &str) -> Result<Value, String> {
        panic!("the core panicked on open");
    }

    fn call(&self, _method: &str, _path: &str, _body: Option<&str>) -> Value {
        panic!("the core panicked on call");
    }

    fn garmin_sync(&self, _api_key: &str, _days: i64) -> Value {
        panic!("the core panicked on sync");
    }

    fn set_garmin_key(&self, _api_key: &str) -> Value {
        panic!("the core panicked on set_garmin_key");
    }

    fn set_garmin_connected(&self, _connected: bool) -> Value {
        panic!("the core panicked on set_garmin_connected");
    }

    fn status(&self) -> Value {
        panic!("the core panicked on status");
    }
}

#[test]
fn a_panic_in_the_core_is_an_error_and_not_a_kill() {
    uniffi_stride::install(Arc::new(PanickingCore));

    for answer in [
        uniffi_stride::stride_call("GET".to_string(), "/activities".to_string(), None),
        uniffi_stride::stride_open("/tmp/stride.db".to_string(), "00".repeat(32)),
        uniffi_stride::stride_garmin_sync("the-key".to_string(), 30),
        uniffi_stride::stride_set_garmin_key("the-key".to_string()),
        uniffi_stride::stride_set_garmin_connected(true),
        uniffi_stride::stride_status(),
    ] {
        let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
        assert_eq!(parsed["ok"], json!(false), "the answer says it failed");
        assert_eq!(parsed["status"], json!(500));
        let detail = parsed["error"]["detail"].as_str().unwrap_or("");
        assert!(
            detail.contains("could not be completed"),
            "the detail is a sentence the app can show: {detail}"
        );
    }
}

#[test]
fn the_panic_payload_is_not_in_the_answer() {
    uniffi_stride::install(Arc::new(PanickingCore));
    let answer = uniffi_stride::stride_call("GET".to_string(), "/activities".to_string(), None);
    assert!(
        !answer.contains("the core panicked"),
        "the panic's payload is not carried back to the app: {answer}"
    );
}
