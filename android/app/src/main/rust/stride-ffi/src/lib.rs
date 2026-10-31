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
//! ## The core that is not there yet
//!
//! The core is `stride-api`, in the workspace above. While it is not in the
//! tree, the seam is built against [`stub`]: every route answers with a
//! sentence saying so. See `README.md` in this directory for what the core
//! has to provide.

pub mod boundary;
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
    CORE.get_or_init(|| RwLock::new(Arc::new(stub::StubCore::new())))
}

/// The core that is installed.
///
/// It is the stub until the real core is installed over it.
fn installed() -> Arc<dyn Core> {
    slot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Install a core.
///
/// This is how the real core replaces the stub, and how the tests put a
/// core that panics in front of the boundary.
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
/// **Garmin has no refresh token.** A "refresh" is a second exchange, and
/// the shell decides how many there are — exactly one, and one retry. This
/// function is one attempt.
pub fn stride_garmin_sync(api_key: String, days: i64) -> String {
    boundary::guarded("sync", || installed().garmin_sync(&api_key, days)).to_string()
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
pub fn stride_set_garmin_key(api_key: String) -> String {
    boundary::guarded("connection", || installed().set_garmin_key(&api_key)).to_string()
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
    error_envelope(503, "the database is not open")
}

/// The envelope of a status.
pub fn status_envelope(open: bool) -> Value {
    ok_envelope(json!({
        "core": "stride-ffi",
        "version": env!("CARGO_PKG_VERSION"),
        "open": open,
    }))
}
