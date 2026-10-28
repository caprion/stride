//! The phone's Rust core, behind UniFFI.
//!
//! The Kotlin shell owns the WebView, the Keystore, the WorkManager and the
//! notifications. Everything else — the routes, the database and the Garmin
//! pull — lives here, in the same process, with no server in between.
//!
//! The contract is `docs/rust-core-spec.md`: `stride-api` owns the router and
//! the response shapes, and this crate is the thin shell around it.
//!
//! ## Why there is a store backend here
//!
//! `stride-api` defines the `Backend` trait the router reads through, and
//! documents the production implementation as "the `stride-store` adapter
//! (proposed in the workstream report)". That adapter does not exist yet, and
//! the phone cannot ship without it, so this crate carries one:
//! `backend::PhoneBackend` implements `Backend` over `stride_store::Store`
//! and computes the derived numbers with `stride_core`.
//!
//! It is **meant to move**. When `stride-api` grows its own store adapter —
//! which is where it belongs, because `stride-api` is the crate that depends
//! on `stride-store` — this copy should be deleted and this crate should call
//! the one there. The workstream report says so.

pub mod backend;
pub mod garmin;
pub mod rows;

use std::sync::{Arc, OnceLock, RwLock};

use serde_json::{json, Value};

use stride_api::Api;

uniffi::include_scaffolding!("stride");

/// The backend the routes read through.
///
/// The phone opens its database once, at start-up, and installs the backend
/// here. Every call after that goes through the same router the desktop
/// uses.
static BACKEND: OnceLock<RwLock<Option<Arc<backend::PhoneBackend>>>> = OnceLock::new();

/// The process-wide backend slot.
fn slot() -> &'static RwLock<Option<Arc<backend::PhoneBackend>>> {
    BACKEND.get_or_init(|| RwLock::new(None))
}

/// The installed backend, if there is one.
fn installed() -> Option<Arc<backend::PhoneBackend>> {
    slot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Open the phone's database and install the backend.
///
/// The answer is a JSON envelope: `{"ok": true, ...}` or
/// `{"ok": false, "error": {"detail": "..."}}`. The Kotlin side reads it as
/// it reads every other answer.
pub fn stride_open(db_path: String, db_key_hex: String) -> String {
    let outcome = std::panic::catch_unwind(|| match open(&db_path, &db_key_hex) {
        Ok(summary) => summary,
        Err(detail) => error_envelope(500, &detail),
    });
    outcome
        .unwrap_or_else(|_| error_envelope(500, "the database could not be opened"))
        .to_string()
}

/// The result of opening the database.
fn open(db_path: &str, db_key_hex: &str) -> Result<Value, String> {
    let key = decode_key(db_key_hex)?;
    let path = std::path::Path::new(db_path);
    let store = stride_store::Store::open_and_migrate(path, &key)
        .map_err(|err| format!("the database could not be opened: {err}"))?;

    let backend = Arc::new(backend::PhoneBackend::new(store));
    {
        let mut guard = slot()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(backend);
    }

    Ok(json!({
        "ok": true,
        "status": 200,
        "body": {
            "db": db_path,
            "schema": "migrated",
        }
    }))
}

/// The 32-byte SQLCipher key, from its hex form.
///
/// The key comes from the Android Keystore. It is hex so that it survives the
/// trip through UniFFI as a string without a byte-array marshalling question.
fn decode_key(hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex.trim();
    if trimmed.len() != 64 {
        return Err("the database key is not 32 bytes".to_string());
    }
    let mut key = Vec::with_capacity(32);
    let bytes = trimmed.as_bytes();
    for pair in bytes.chunks(2) {
        let text = std::str::from_utf8(pair).map_err(|_| "the database key is not hex".to_string())?;
        let byte =
            u8::from_str_radix(text, 16).map_err(|_| "the database key is not hex".to_string())?;
        key.push(byte);
    }
    Ok(key)
}

/// Call a route.
///
/// The answer is the same envelope `stride-api`'s C ABI produces, because it
/// *is* that function: this crate calls the ABI and frees the string it
/// returns.
pub fn stride_call(method: String, path: String, body: Option<String>) -> String {
    let outcome = std::panic::catch_unwind(|| call(&method, &path, body.as_deref()));
    outcome
        .unwrap_or_else(|_| error_envelope(500, "the request could not be completed").to_string())
}

/// The call, through the router.
///
/// The envelope is the one `stride-api`'s C ABI produces, so the Kotlin side
/// reads one shape whichever way it asks:
///
/// ```text
/// {"ok": true,  "status": 200, "body": { ... }}
/// {"ok": false, "status": 404, "error": {"detail": "..."}}
/// ```
fn call(method: &str, path: &str, body: Option<&str>) -> String {
    let Some(backend) = installed() else {
        return error_envelope(503, "the database is not open").to_string();
    };
    let api = Api::new(backend);
    let outcome = match api.call_response(method, path, body) {
        Ok(stride_api::Response::Json(value)) => json!({
            "ok": true,
            "status": 200,
            "body": value,
        }),
        Ok(stride_api::Response::NoContent) => json!({
            "ok": true,
            "status": 204,
            "body": Value::Null,
        }),
        Ok(stride_api::Response::Binary {
            content_type,
            bytes,
        }) => json!({
            "ok": true,
            "status": 200,
            "body": {
                "content_type": content_type,
                "base64": stride_api::util::base64_encode(&bytes),
            },
        }),
        Err(err) => json!({
            "ok": false,
            "status": err.status,
            "error": err.to_body(),
        }),
    };
    outcome.to_string()
}

/// Pull from Garmin into the local database.
///
/// The key is used for the token exchange and then dropped. The bearer token
/// stays in memory, inside the client, and is refreshed by exchanging the key
/// again — Garmin has no refresh token.
pub fn stride_garmin_sync(api_key: String, days: i64) -> String {
    let outcome = std::panic::catch_unwind(|| match installed() {
        Some(backend) => garmin::sync(&backend, &api_key, days)
            .unwrap_or_else(|err| error_envelope(500, &err.to_string())),
        None => error_envelope(503, "the database is not open"),
    });
    outcome
        .unwrap_or_else(|_| error_envelope(500, "the sync could not be completed"))
        .to_string()
}

/// Put the Garmin key in memory, or take it out again.
///
/// The key is in memory for the length of one pull. The Kotlin shell
/// unwraps it from the Keystore, hands it over, and clears it again as soon
/// as the pull is over.
pub fn stride_set_garmin_key(api_key: String) -> String {
    match installed() {
        Some(backend) => {
            backend.set_garmin_key(&api_key);
            json!({
                "ok": true,
                "status": 200,
                "body": {"in_memory": !api_key.is_empty()},
            })
            .to_string()
        }
        None => error_envelope(503, "the database is not open").to_string(),
    }
}

/// What the core is, and whether it is open.
pub fn stride_status() -> String {
    let backend = installed();
    json!({
        "ok": true,
        "status": 200,
        "body": {
            "core": "stride-ffi",
            "version": env!("CARGO_PKG_VERSION"),
            "open": backend.is_some(),
            "backend": "phone",
        }
    })
    .to_string()
}

/// The envelope of a failure.
fn error_envelope(status: u16, detail: &str) -> Value {
    json!({
        "ok": false,
        "status": status,
        "error": {"detail": detail},
    })
}
