//! The phone's Rust core, behind UniFFI.
//!
//! The Kotlin shell owns the WebView, the Keystore, the WorkManager and the
//! notifications. Everything else — the routes, the database and the Garmin
//! pull — lives here, in the same process, with no server in between.
//!
//! ## What this crate is
//!
//! * **the seam** ([`seam`]) — the narrow trait the core has to meet;
//! * **the boundary** ([`boundary`]) — a panic in the core becomes an error
//!   the app can show, never a process kill;
//! * **the core** ([`real`]) — `stride-api`'s router, in this process;
//! * **the stub** ([`stub`]) — the core that is not there, kept for the
//!   tests that prove a stub can refuse;
//! * **the surface** — the six functions the UDL declares, every one of
//!   them answering with the same JSON envelope.
//!
//! ## What the shell promises, and what this crate keeps
//!
//! * The Garmin key is an **argument**. It is passed in, used and dropped.
//!   No function here returns it, and nothing here logs it.
//! * The database key is an argument too, and it is never written anywhere
//!   but the file the Keystore protects.
//! * The routes are the frozen route table's (`docs/contract/api-paths.md`
//!   in the repository root). This crate does not
//!   decide which route a path names — the router does — and it does not add
//!   one.
//!
//! ## The core that is in the tree
//!
//! The core is `stride-api`, in the workspace above this crate, and this
//! crate calls it **in this process** — through the api's own FFI surface,
//! which is the other side of the seam's `stride_call`. So the phone's
//! answer is the router's answer, and the desktop's answer is the same
//! router's answer: the two hosts cannot disagree about a number.
//!
//! The stub is still here, and it is still honest — it refuses every route
//! with a sentence saying so — but it is no longer what the phone runs. It
//! is what the tests put in front of the boundary.

pub mod boundary;
#[cfg(feature = "real-core")]
pub mod real;
pub mod seam;
pub mod stub;

use std::sync::{Arc, OnceLock, RwLock};

use serde_json::{json, Value};

use crate::seam::{error_envelope, ok_envelope, Core};

// The UniFFI scaffolding, generated from the UDL by the build script.
//
// It is the FFI boundary: the `#[no_mangle]` symbols it declares are the
// ones the Kotlin bindings call. The build script gives the generated items
// the `unsafe_code` allowance they need, so the lint stays armed for the
// rest of the crate.
uniffi::include_scaffolding!("stride");

/// The process-wide core.
static CORE: OnceLock<RwLock<Arc<dyn Core>>> = OnceLock::new();

/// The process-wide slot.
fn slot() -> &'static RwLock<Arc<dyn Core>> {
    CORE.get_or_init(|| RwLock::new(default_core()))
}

/// The core the phone runs.
///
/// **The real core**: `stride-api`'s router, in this process. A build
/// without the `real-core` feature runs the stub instead — a development
/// build, and one that says so on every route.
#[cfg(feature = "real-core")]
fn default_core() -> Arc<dyn Core> {
    Arc::new(real::ApiCore::new())
}

/// The core that is not there.
///
/// It is here for the builds that ask for it on purpose: a core that opens
/// nothing, answers no route, and says so.
#[cfg(not(feature = "real-core"))]
fn default_core() -> Arc<dyn Core> {
    Arc::new(stub::StubCore::new())
}

/// The core that is installed.
///
/// It is the real core — `stride-api`'s router, in this process — until
/// something else is installed over it.
fn installed() -> Arc<dyn Core> {
    slot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Install a core.
///
/// This is how the stub replaces the real core in a test, and how the tests
/// put a core that panics in front of the boundary.
pub fn install(core: Arc<dyn Core>) {
    let mut guard = slot()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = core;
}

/// Open the phone's database and install the backend behind the routes.
///
/// The answer is a JSON envelope: `{"ok": true, ...}` or
/// `{"ok": false, "error": {"detail": "..."}}`. The Kotlin side reads it as
/// it reads every other answer.
pub fn stride_open(db_path: String, db_key_hex: String) -> String {
    boundary::guarded_fallible("database", || installed().open(&db_path, &db_key_hex)).to_string()
}

/// Call a route.
///
/// The answer is the envelope the SPA reads:
///
/// ```text
/// {"ok": true,  "status": 200, "body": { ... }}
/// {"ok": false, "status": 404, "error": {"detail": "..."}}
/// ```
pub fn stride_call(method: String, path: String, body: Option<String>) -> String {
    boundary::guarded("request", || {
        installed().call(&method, &path, body.as_deref())
    })
    .to_string()
}

/// Pull from Garmin into the phone's database.
///
/// The pull itself is the core's: the same one the desktop runs, over the
/// same database. The key is used for the token exchange and then dropped:
/// the bearer token stays in memory, inside the client.
///
/// **One attempt.** The core performs exactly one token exchange and one
/// HTTP attempt, and then answers. When the token has expired the answer
/// is the typed failure `needs_reconnect` (status 401).
///
/// **The retry belongs to the caller.** The contract says so
/// (`docs/contract/core-contract.md` §7 — "The retry rule, and who owns
/// it"): the client does not re-exchange and try again, and the shell may
/// make exactly **one** more attempt, which performs one fresh exchange.
/// A second failure stops, the phone says the key was refused, and the next
/// scheduled sync tries again.
pub fn stride_garmin_sync(key: String, days: i64) -> String {
    boundary::guarded("sync", || installed().garmin_sync(&key, days)).to_string()
}

/// Put the Garmin key in memory, or take it out again.
///
/// The key is in memory for the length of one pull and no longer: the
/// Kotlin shell unwraps it from the Keystore, hands it over, and clears it
/// again as soon as the pull is over.
///
/// **Taking the key out of the memory is not disconnecting.** The key lives
/// in the Keystore; the copy here is there for the length of a pull. The
/// connection is a separate fact, and [`stride_set_garmin_connected`] is
/// what says it.
pub fn stride_set_garmin_key(key: String) -> String {
    boundary::guarded("connection", || installed().set_garmin_key(&key)).to_string()
}

/// Say whether the phone holds a Garmin key.
///
/// The key itself lives in the Keystore, and the core holds it for the
/// length of a pull and no longer. This says the *fact* of the connection,
/// and the page never sees the key.
pub fn stride_set_garmin_connected(connected: bool) -> String {
    boundary::guarded("connection", || installed().set_garmin_connected(connected)).to_string()
}

/// What the core is, and whether it is open.
pub fn stride_status() -> String {
    boundary::guarded("status", || installed().status()).to_string()
}

/// The envelope of a core that is not open.
///
/// It is here so that the shell and the tests can ask for it without a
/// core: the answer is a fact about the envelope, not about the core.
pub fn not_open_envelope() -> Value {
    error_envelope(503, "not_open", "the database is not open")
}

/// The envelope of a status.
pub fn status_envelope(open: bool) -> Value {
    ok_envelope(json!({
        "core": "stride-ffi",
        "version": env!("CARGO_PKG_VERSION"),
        "open": open,
        "backend": seam::COMPILED_BACKEND,
        "backend_marker": seam::COMPILED_BACKEND_MARKER,
    }))
}
