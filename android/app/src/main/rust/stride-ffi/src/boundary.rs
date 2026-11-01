//! The panic boundary.
//!
//! **A panic in the core is an error the app can show, never a process
//! kill.**
//!
//! The Rust core is a library inside the phone's process. An unwinding panic
//! that crossed the FFI boundary would abort the app — and the app is the
//! athlete's history, open on the screen, in the middle of a pull. So every
//! call across the boundary is wrapped: the panic is caught here, turned
//! into an error envelope, and handed back as an ordinary answer.
//!
//! ## What is logged, and what is not
//!
//! The panic hook is replaced, and it logs **where** the panic happened and
//! nothing else. The default hook prints the panic's payload, and a payload
//! is whatever the failing code was holding — which, in this app, might be
//! the Garmin key. So the payload is not printed. The location is not a
//! secret and it is the only thing a developer needs.

use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

use serde_json::Value;

use crate::seam::error_envelope;

static HOOK: Once = Once::new();

/// Install the hook that keeps a panic's payload out of the log.
///
/// It is installed once, and it is installed by the first call that crosses
/// the boundary — so it is in place before anything can panic.
pub fn install_hook() {
    HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| match info.location() {
            Some(place) => {
                eprintln!(
                    "the Stride core panicked at {}:{} — the details are not logged",
                    place.file(),
                    place.line()
                );
            }
            None => eprintln!("the Stride core panicked — the details are not logged"),
        }));
    });
}

/// Run a call across the boundary, and turn a panic into an error.
///
/// The answer is always a JSON envelope. A panic becomes
/// `{"ok": false, "status": 500, "error": {"detail": "..."}}` — the same
/// shape as every other failure, so the shell has one thing to read.
pub fn guarded(what: &str, call: impl FnOnce() -> Value) -> Value {
    install_hook();
    let outcome = panic::catch_unwind(AssertUnwindSafe(call));
    match outcome {
        Ok(value) => value,
        Err(_) => {
            let detail = format!("the {what} could not be completed");
            error_envelope(500, &detail)
        }
    }
}

/// Run a call across the boundary that may fail, and turn a panic into the
/// failure.
pub fn guarded_fallible(what: &str, call: impl FnOnce() -> Result<Value, String>) -> Value {
    install_hook();
    let outcome = panic::catch_unwind(AssertUnwindSafe(call));
    match outcome {
        Ok(Ok(value)) => value,
        Ok(Err(detail)) => error_envelope(500, &detail),
        Err(_) => {
            let detail = format!("the {what} could not be completed");
            error_envelope(500, &detail)
        }
    }
}
