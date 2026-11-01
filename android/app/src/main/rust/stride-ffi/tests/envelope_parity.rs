//! The two sides of the boundary, and one envelope between them.
//!
//! The router answers with
//!
//! ```text
//! {"ok": false, "status": 404, "error": {"code": "not_found", "detail": "…"}}
//! ```
//!
//! and the seam answers with the same shape. **They have to**: the code is
//! not decoration — the shell reads it, and a refusal is
//! `needs_reconnect` — so an envelope without a code is a shape the two
//! sides disagree about, and the disagreement shows up as a pull that never
//! retries.
//!
//! So the test is not a golden: it calls the router's own function and the
//! seam's own function, for the same failure, and asks whether the two
//! answers are the same document.
//!
//! That is why the seam has a **dev-dependency** on the router's crate: a
//! seam that invents its own error shape is a second implementation of the
//! seam, and the only way to prove there is one is to compare the two.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use stride_api::error::ApiError;

#[test]
fn a_failure_answers_with_the_same_envelope_on_both_sides() {
    let error = ApiError::needs_reconnect("the token has expired");

    let from_the_router = stride_api::ffi::failure_envelope(&error);
    let from_the_seam =
        uniffi_stride::seam::error_envelope(error.status(), error.code(), error.detail());

    assert_eq!(
        from_the_router, from_the_seam,
        "the seam and the router answered the same failure differently"
    );
}

#[test]
fn a_panic_answers_with_the_same_envelope_on_both_sides() {
    let from_the_router = stride_api::ffi::guarded("request", || panic!("boom"));
    let from_the_seam = uniffi_stride::boundary::guarded("request", || panic!("boom"));

    assert_eq!(
        from_the_router, from_the_seam,
        "the seam and the router turned a panic into different envelopes"
    );
}

#[test]
fn the_envelope_carries_a_code_the_shell_reads() {
    // The shell's `PullPolicy` reads `code == "needs_reconnect"`, so the
    // code has to be there and has to be the router's.
    let error = ApiError::needs_reconnect("the token has expired");
    let envelope =
        uniffi_stride::seam::error_envelope(error.status(), error.code(), error.detail());

    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["status"], json!(401));
    assert_eq!(envelope["error"]["code"], json!("needs_reconnect"));
    assert_eq!(envelope["error"]["detail"], json!("the token has expired"));
}
