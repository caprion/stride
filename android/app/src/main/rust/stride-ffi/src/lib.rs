//! The phone's Rust core, behind UniFFI.
//!
//! The Kotlin shell owns the WebView, the Keystore, the WorkManager and the
//! notifications. Everything else — the routes, the database and the Garmin
//! pull — lives here, in the same process, with no server in between.
//!
//! The contract is `docs/rust-core-spec.md`: `stride-api` owns the router and
//! the response shapes, and this crate is the thin shell around it.
//!
//! ## What the shell does
//!
//! * it opens the phone's own SQLCipher database with the key the Keystore
//!   protected, and migrates it;
//! * it stacks the shared backends — the store adapter, the state and the
//!   domain — so the phone answers the routes exactly as the desktop does;
//! * it adds the one thing the phone has and the desktop does not: a Garmin
//!   pull a background worker can drive;
//! * it calls the router, and it never lets a panic out.
//!
//! The backend stack:
//!
//! ```text
//! CoreBackend        the domain: the fingerprint, the readiness, the verdicts
//!   └── PhoneSync    the pull
//!         └── StateBackend   the state: the key, the status, the pairing
//!               └── the store adapter   the rows
//! ```

pub mod backend;

use std::sync::{Arc, OnceLock, RwLock};

use serde_json::{json, Value};

use stride_api::backend::Backend;
use stride_api::backend::SyncRunRow;
use stride_api::{Api, Response};
use stride_api::{CoreBackend, StateBackend};

// The UniFFI scaffolding, generated from the UDL by the build script.
//
// It is the FFI boundary: the `#[no_mangle]` symbols it declares are the ones
// the Kotlin bindings call. The build script gives the generated items the
// `unsafe_code` allowance they need, so the lint stays armed for the rest of
// the crate.
uniffi::include_scaffolding!("stride");

/// The method the phone connects with.
///
/// The desktop has two: a developer key and an account. The phone has one —
/// the athlete's Personal API key — and the app says so by name.
const METHOD: &str = "personal";

/// The backends the phone runs, and the state they share.
struct PhoneCore {
    /// The state: the Garmin key, the status the browser reads.
    state: Arc<StateBackend>,
    /// The pull.
    sync: Arc<backend::PhoneSync>,
    /// The domain.
    core: Arc<CoreBackend>,
}

/// The process-wide stack.
static CORE: OnceLock<RwLock<Option<Arc<PhoneCore>>>> = OnceLock::new();

/// The process-wide slot.
fn slot() -> &'static RwLock<Option<Arc<PhoneCore>>> {
    CORE.get_or_init(|| RwLock::new(None))
}

/// The stack, if it is open.
fn installed() -> Option<Arc<PhoneCore>> {
    slot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Open the phone's database and install the backends.
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

/// Open the database and stack the backends.
fn open(db_path: &str, db_key_hex: &str) -> Result<Value, String> {
    let key = decode_key(db_key_hex)?;
    let path = std::path::Path::new(db_path);
    let store = Arc::new(
        stride_store::Store::open_and_migrate(path, &key)
            .map_err(|err| format!("the database could not be opened: {err}"))?,
    );

    // The shared stack: the rows, the state, the domain.
    let rows = stride_api::store::backend(store.clone());
    let state = Arc::new(StateBackend::new(rows));
    let sync = Arc::new(backend::PhoneSync::new(
        state.clone() as Arc<dyn Backend>,
        store,
    ));
    let core = Arc::new(CoreBackend::new(sync.clone() as Arc<dyn Backend>));

    {
        let mut guard = slot()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(Arc::new(PhoneCore { state, sync, core }));
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
        let text =
            std::str::from_utf8(pair).map_err(|_| "the database key is not hex".to_string())?;
        let byte =
            u8::from_str_radix(text, 16).map_err(|_| "the database key is not hex".to_string())?;
        key.push(byte);
    }
    Ok(key)
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
    let outcome = std::panic::catch_unwind(|| call(&method, &path, body.as_deref()));
    outcome
        .unwrap_or_else(|_| error_envelope(500, "the request could not be completed").to_string())
        .to_string()
}

/// The call, through the router.
fn call(method: &str, path: &str, body: Option<&str>) -> String {
    let Some(core) = installed() else {
        return error_envelope(503, "the database is not open").to_string();
    };
    let api = Api::new(core.core.clone() as Arc<dyn Backend>);
    let outcome = match api.call_response(method, path, body) {
        Ok(Response::Json(value)) => json!({
            "ok": true,
            "status": 200,
            "body": value,
        }),
        Ok(Response::NoContent) => json!({
            "ok": true,
            "status": 204,
            "body": Value::Null,
        }),
        Ok(Response::Binary {
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

/// Pull from Garmin into the phone's database.
///
/// The pull itself is `stride-api`'s: the same one the desktop runs, over
/// the same database. The key is used for the token exchange and then
/// dropped: the bearer token stays in memory, inside the client, and is
/// refreshed by exchanging the key again — Garmin has no refresh token.
pub fn stride_garmin_sync(api_key: String, days: i64) -> String {
    let outcome = std::panic::catch_unwind(|| match installed() {
        Some(core) => {
            // The key is in memory for the length of the pull, and no
            // longer: the shell unwraps it from the Keystore, hands it over
            // and takes it back.
            set_key(&core, &api_key);
            let started = chrono::Utc::now();
            let pulled = core.sync.run_sync(Some(days), false);
            let finished = chrono::Utc::now();
            match pulled {
                Ok(value) => {
                    // The log is in the phone's own database: it is the
                    // evidence that survives the key going back into the
                    // Keystore.
                    write_sync_run(&core, &value, days, &started, &finished, None);
                    remember_connection(&core, true);
                    json!({"ok": true, "status": 200, "body": value})
                }
                Err(err) => {
                    let detail = err.to_string();
                    write_sync_run(
                        &core,
                        &Value::Null,
                        days,
                        &started,
                        &finished,
                        Some(detail.clone()),
                    );
                    remember_connection(&core, false);
                    error_envelope(500, &detail)
                }
            }
        }
        None => error_envelope(503, "the database is not open"),
    });
    outcome
        .unwrap_or_else(|_| error_envelope(500, "the sync could not be completed"))
        .to_string()
}

/// Write one line of the sync log.
///
/// The log is in the phone's own database, in the same table the desktop
/// uses. It answers two questions the app asks: when did the last pull
/// finish, and what did it bring back.
fn write_sync_run(
    core: &PhoneCore,
    value: &Value,
    days: i64,
    started: &chrono::DateTime<chrono::Utc>,
    finished: &chrono::DateTime<chrono::Utc>,
    error: Option<String>,
) {
    let found = value
        .get("activities_found")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let new = value
        .get("activities_new")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let updated = value
        .get("activities_updated")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let message = error
        .clone()
        .or_else(|| Some(format!("{found} runs, {new} new, {updated} updated")));
    let row = SyncRunRow {
        id: None,
        source: "garmin_personal".to_string(),
        started_at: Some(started.to_rfc3339()),
        finished_at: Some(finished.to_rfc3339()),
        status: if error.is_some() { "error" } else { "ok" }.to_string(),
        days,
        activities_found: found,
        activities_new: new,
        activities_updated: updated,
        message,
        error,
    };
    if let Err(failure) = core.sync.upsert_sync_run(&row) {
        eprintln!("the sync log could not be written: {failure}");
    }
}

/// Remember that Garmin is connected, once the pull is over.
///
/// The key goes back into the Keystore: the core holds it for the length of
/// a pull and no longer. The *fact* of the connection stays, because the
/// phone is connected — the next pull will use the same key, and the app
/// has to be able to say so. Without this the dashboard would answer "not
/// connected" a second after the screen said "Connected · 160 runs".
fn remember_connection(core: &PhoneCore, synced: bool) {
    let mut state = core.state.state();
    state.personal_api_key = None;
    state.status.garmin_connected = true;
    state.status.garmin_method = Some(METHOD.to_string());
    if synced {
        state.status.last_sync_at = Some(stride_api::store::python_iso(chrono::Utc::now()));
    }
    core.state.set_state(state);
}

/// Put the Garmin key in the state's memory, or take it out again.
///
/// The state is what the sync reads the key through, and what tells the
/// browser that Garmin is connected.
///
/// **Taking the key out of the memory is not disconnecting.** The key lives
/// in the Keystore; the copy here is there for the length of one pull.
/// So clearing it leaves the connection alone, and the caller that really
/// is disconnecting says so with [`set_connected`].
fn set_key(core: &PhoneCore, api_key: &str) {
    let mut state = core.state.state();
    state.personal_api_key = if api_key.is_empty() {
        None
    } else {
        Some(api_key.to_string())
    };
    state.status.has_personal_api_key = state.personal_api_key.is_some();
    if state.personal_api_key.is_some() {
        state.status.garmin_connected = true;
        state.status.garmin_method = Some(METHOD.to_string());
    }
    core.state.set_state(state);
}

/// Say whether the phone is connected to Garmin.
///
/// This is the *fact* of the connection, and it is separate from the key:
/// the key is in the Keystore, and this says the app may use it.
fn set_connected(core: &PhoneCore, connected: bool) {
    let mut state = core.state.state();
    state.status.garmin_connected = connected;
    state.status.garmin_method = if connected {
        Some(METHOD.to_string())
    } else {
        None
    };
    core.state.set_state(state);
}

/// Put the Garmin key in memory, or take it out again.
///
/// The key is in memory for the length of one pull. The Kotlin shell
/// unwraps it from the Keystore, hands it over, and clears it again as soon
/// as the pull is over.
pub fn stride_set_garmin_key(api_key: String) -> String {
    match installed() {
        Some(core) => {
            set_key(&core, &api_key);
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

/// Say whether the phone holds a Garmin key.
///
/// The key itself lives in the Keystore, and the core holds it for the
/// length of a pull and no longer. This says the *fact* of it, so the app
/// can say "Garmin is connected" — in the app, and to the page, which never
/// sees the key.
pub fn stride_set_garmin_connected(connected: bool) -> String {
    match installed() {
        Some(core) => {
            set_connected(&core, connected);
            json!({
                "ok": true,
                "status": 200,
                "body": {"connected": connected},
            })
            .to_string()
        }
        None => error_envelope(503, "the database is not open").to_string(),
    }
}

/// What the core is, and whether it is open.
pub fn stride_status() -> String {
    let open = installed().is_some();
    json!({
        "ok": true,
        "status": 200,
        "body": {
            "core": "stride-ffi",
            "version": env!("CARGO_PKG_VERSION"),
            "open": open,
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
