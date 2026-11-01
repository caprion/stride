//! The FFI surface: JSON in, JSON out.
//!
//! The phone runs this core **in the same process** as the app, so this
//! module is the boundary the Kotlin shell calls across. It answers with
//! the same envelope the desktop's router answers with, so the two hosts
//! cannot disagree about a number:
//!
//! ```text
//! {"ok": true,  "status": 200, "body": { ... }}
//! {"ok": false, "status": 404, "error": {"code": "not_found", "detail": "…"}}
//! ```
//!
//! ## A panic never crosses this boundary
//!
//! A panic in the core is an error the app can show, never a process kill.
//! Every call here is wrapped: the panic is caught, turned into an error
//! envelope and handed back as an ordinary answer.
//!
//! ## The panic's payload is not logged
//!
//! The panic hook is replaced, and it logs **where** the panic happened and
//! nothing else. The default hook prints the payload, and a payload is
//! whatever the failing code was holding — which, in this app, might be the
//! Garmin key. The location is not a secret and it is the only thing a
//! developer needs.
//!
//! ## The key is an argument
//!
//! [`stride_set_garmin_key`] puts the key in memory for the length of a
//! pull and [`stride_set_garmin_connected`] takes it out again. Nothing
//! here returns the key, and nothing here logs it.
//!
//! ## One seam, not two
//!
//! The phone calls this core through **UniFFI**
//! (`android/app/src/main/rust/stride-ffi`), and that seam is the phone's
//! own boundary: it catches the panic and hands the body across. This
//! module is the **JSON in, JSON out** surface underneath it — the same
//! shape the desktop's HTTP layer answers with — and there is no second
//! implementation of it here: a C-ABI surface with its own allocator and
//! its own free would be a second seam, and two seams are two answers to
//! the same question.
//!
//! The envelope is the contract's:
//!
//! ```text
//! {"ok": true,  "status": 200, "body": { ... }}
//! {"ok": false, "status": 401, "error": {"code": "needs_reconnect", "detail": "…"}}
//! ```
//!
//! The failure carries the **code** as well as the sentence, because the
//! phone's reader reads the code: `needs_reconnect` is what tells the
//! caller that one more attempt is allowed.

use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, Once, OnceLock};

use serde_json::{json, Value};

use crate::error::ApiError;
use crate::router::Router;

/// The core the calls go to.
static CORE: OnceLock<Mutex<Option<Arc<Router>>>> = OnceLock::new();

/// The panic hook, installed once.
static HOOK: Once = Once::new();

/// Install the core the calls go to.
///
/// The phone opens its database, builds the core and installs it here; the
/// desktop does the same with its own store. A call made before the core
/// is installed is answered with `not_open` rather than with a crash.
pub fn install(router: Router) {
    let slot = CORE.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(Arc::new(router));
}

/// The core, when one is installed.
fn installed() -> Option<Arc<Router>> {
    let slot = CORE.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.clone()
}

/// Call a route.
///
/// The answer is the envelope, as a JSON string. The method defaults to a
/// read; [`stride_call_with`] names it.
pub fn stride_call(path: &str, body: Option<&str>) -> String {
    stride_call_with("GET", path, body)
}

/// Call a route with a method.
pub fn stride_call_with(method: &str, path: &str, body: Option<&str>) -> String {
    let answer = guarded("request", || match installed() {
        Some(router) => router.call(method, path, body).envelope(),
        None => not_open_envelope(),
    });
    answer.to_string()
}

/// Put the Garmin key in memory, or take it out again.
///
/// The key is in memory for the length of a pull and no longer. An empty
/// key takes it out.
pub fn stride_set_garmin_key(key: &str) -> String {
    let answer = guarded("connection", || match installed() {
        Some(router) => {
            router.api().set_garmin_key(key);
            ok_envelope(json!({
                "connected": router.api().garmin_connected(),
                "method": "personal_key",
            }))
        }
        None => not_open_envelope(),
    });
    answer.to_string()
}

/// Say whether the phone holds a Garmin key.
pub fn stride_set_garmin_connected(connected: bool) -> String {
    let answer = guarded("connection", || match installed() {
        Some(router) => {
            if !connected {
                router.api().clear_garmin_key();
            }
            ok_envelope(json!({
                "connected": router.api().garmin_connected(),
                "method": "personal_key",
            }))
        }
        None => not_open_envelope(),
    });
    answer.to_string()
}

/// What the core is, and whether it is open.
pub fn stride_status() -> String {
    let answer = guarded("status", || match installed() {
        Some(router) => ok_envelope(json!({
            "core": "stride-api",
            "version": env!("CARGO_PKG_VERSION"),
            "open": true,
            // **The marker.** The phone's stub answers "backend": "stub",
            // so a shell that asks the core what it is can tell a stub
            // from a router — and a release build may not ship the stub.
            "backend": "stride-api",
            "generated_at": router.api().generated_at(),
        })),
        None => not_open_envelope(),
    });
    answer.to_string()
}

/// The envelope of a success.
pub fn ok_envelope(body: Value) -> Value {
    json!({
        "ok": true,
        "status": 200,
        "body": body,
    })
}

/// The envelope of a core that is not open.
pub fn not_open_envelope() -> Value {
    json!({
        "ok": false,
        "status": 503,
        "error": {
            "code": "not_open",
            "detail": "the database is not open",
        },
    })
}

/// Run a call across the boundary, and turn a panic into an error.
///
/// This is the boundary's own guard: it catches a panic, turns it into an
/// error envelope and hands it back as an ordinary answer. It is public so
/// that the boundary can be proved — the proof is a test, and the test has
/// to reach the guard.
pub fn guarded(what: &str, call: impl FnOnce() -> Value) -> Value {
    install_hook();
    match panic::catch_unwind(AssertUnwindSafe(call)) {
        Ok(value) => value,
        Err(_) => {
            let detail = format!("the {what} could not be completed");
            json!({
                "ok": false,
                "status": 500,
                "error": {
                    "code": "internal",
                    "detail": detail,
                },
            })
        }
    }
}

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

/// The failure a router reports, as the envelope.
pub fn failure_envelope(error: &ApiError) -> Value {
    json!({
        "ok": false,
        "status": error.status(),
        "error": {
            "code": error.code(),
            "detail": error.detail(),
        },
    })
}
