//! The seam.
//!
//! **What the phone needs from the core, and nothing more.** The core is
//! `stride-api` in the workspace above this crate; it is not in the tree
//! yet, so what is here is the *shape* of the call, and a stub that answers
//! every route with "the Rust core is not in this build" rather than with a
//! number.
//!
//! A stub can refuse. It cannot be wrong. That is the whole reason it is
//! allowed to exist: an APK that carries a stub says nothing at all, and an
//! APK that carries a wrong core says something false.
//!
//! The trait is the contract the core has to meet. It is deliberately
//! narrow: JSON in, JSON out, and the same envelope the desktop's router
//! answers with, so the phone and the desktop cannot disagree about a
//! number.

use serde_json::{json, Value};

/// What the core has to be able to do for the phone.
///
/// Every method answers with the envelope:
///
/// ```text
/// {"ok": true,  "status": 200, "body": { ... }}
/// {"ok": false, "status": 404, "error": {"detail": "..."}}
/// ```
pub trait Core: Send + Sync {
    /// Open the phone's own database, with the key the Keystore protected.
    fn open(&self, db_path: &str, db_key_hex: &str) -> Result<Value, String>;

    /// Call a route.
    ///
    /// The router matches on the path **without** the query string, so the
    /// query string is not part of the route and never changes which route
    /// is called.
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Value;

    /// Pull from Garmin into the phone's database.
    ///
    /// The key is used for the token exchange and then forgotten. Garmin has
    /// no refresh token, so a second exchange is a second call with the key.
    fn garmin_sync(&self, api_key: &str, days: i64) -> Value;

    /// Put the Garmin key in memory, or take it out again.
    fn set_garmin_key(&self, api_key: &str) -> Value;

    /// Say whether the phone holds a Garmin key.
    fn set_garmin_connected(&self, connected: bool) -> Value;

    /// What the core is, and whether it is open.
    fn status(&self) -> Value;
}

/// The envelope of a success.
pub fn ok_envelope(body: Value) -> Value {
    json!({
        "ok": true,
        "status": 200,
        "body": body,
    })
}

/// The envelope of a failure.
pub fn error_envelope(status: u16, detail: &str) -> Value {
    json!({
        "ok": false,
        "status": status,
        "error": {"detail": detail},
    })
}

/// The envelope of a core that is not in this build.
///
/// The detail is a sentence the app can show, not a stack trace: the app
/// shows this to an athlete, and an athlete is not a compiler.
pub fn not_built_envelope() -> Value {
    error_envelope(
        503,
        "the Rust core is not in this build. Nothing is shown until it is.",
    )
}
