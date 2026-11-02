//! The real core: `stride-api`'s router, in this process.
//!
//! The phone does not talk to a server. It loads `libuniffi_stride.so` and
//! calls the router **in the same process**: the seam's contract is
//! `stride_call(path, body)`, and the router's own FFI surface
//! (`stride_api::ffi`) is the other side of it.
//!
//! ## What the seam adds, and what it does not
//!
//! The seam adds exactly two things to the router:
//!
//! * **the database.** The phone's own SQLCipher file, in the storage no
//!   backup carries, opened with the key the Keystore protects. The key is
//!   an argument: it is passed in, used and dropped.
//! * **the panic boundary.** A panic in the core is an error the app can
//!   show, never a process kill. The router's FFI surface already guards
//!   its own calls; the seam guards the ones around it.
//!
//! Everything else — the route table, the sentences, the rounding, the
//! Garmin client — is the router's, and is not restated here.
//!
//! ## The Garmin key
//!
//! The key is an argument. It is put in memory for the length of one pull
//! and dropped again, and nothing here returns it, logs it, or writes it
//! down.

use std::path::Path;
use std::sync::Arc;

use serde_json::{json, Value};
use stride_api::{Api, Clock, Config, Router, SystemClock};
use stride_store::{DatabaseKey, Store};

use crate::seam::{ok_envelope, Core};

/// The core the phone runs.
///
/// It holds no state of its own: the router is installed in the api's own
/// FFI surface, and every call goes through it. That is what makes the
/// phone's answer and the desktop's answer the same bytes — they are the
/// same router, and the same DTOs.
pub struct ApiCore {
    clock: Arc<dyn Clock>,
}

impl ApiCore {
    /// A core, with the clock of the machine.
    pub fn new() -> ApiCore {
        ApiCore {
            clock: Arc::new(SystemClock),
        }
    }

    /// A core, with a clock of the caller's choosing.
    ///
    /// The clock is injected, so every date the router derives is
    /// reproducible: a test can hold it still and compare the phone's
    /// answer with the desktop's, byte for byte.
    pub fn with_clock(clock: Arc<dyn Clock>) -> ApiCore {
        ApiCore { clock }
    }
}

impl Default for ApiCore {
    fn default() -> Self {
        Self::new()
    }
}

impl Core for ApiCore {
    /// Open the phone's database and install the router behind the routes.
    ///
    /// The key is hex-encoded, and it is the argument: it is decoded, it is
    /// handed to the store, and the store keeps it — there is no accessor
    /// that reads it back out.
    fn open(&self, db_path: &str, db_key_hex: &str) -> Result<Value, String> {
        let bytes = decode_hex(db_key_hex)
            .ok_or_else(|| "the database key is not hexadecimal.".to_string())?;
        let key = DatabaseKey::from_bytes(bytes)
            .map_err(|_| "the database key was refused.".to_string())?;
        let store = Store::open_and_migrate(Path::new(db_path), &key)
            .map_err(|_| "the database could not be opened.".to_string())?;
        let api = Api::new(Arc::new(store), Config::default(), Arc::clone(&self.clock))
            .map_err(|_| "the Garmin client could not be built.".to_string())?;
        stride_api::ffi::install(Router::new(Arc::new(api)));
        Ok(ok_envelope(json!({"open": true, "db": db_path})))
    }

    /// Call a route.
    ///
    /// The answer is the router's envelope, and it is the **same envelope**
    /// the router answers with — one envelope on both sides of the
    /// boundary. The body inside it is the router's body, and the shell
    /// hands the app that body: the same fields, the same names, the same
    /// nullability and the same values as the desktop's HTTP layer puts on
    /// the wire.
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Value {
        stride_api::ffi::call_value(method, path, body)
    }

    /// Pull from Garmin into the phone's database.
    ///
    /// **One attempt.** The Garmin client performs exactly one token
    /// exchange and one HTTP attempt per call, and the router adds no
    /// retry of its own: a session that has ended comes back as the typed
    /// failure `needs_reconnect`, and the shell — the phone's
    /// `PullPolicy` — decides whether there is one more attempt.
    ///
    /// The key is in memory for the length of the pull and no longer.
    fn garmin_sync(&self, key: &str, days: i64) -> Value {
        stride_api::ffi::set_garmin_key_value(key);
        let body = json!({"days": days}).to_string();
        let answer = stride_api::ffi::call_value("POST", "/sync/run", Some(&body));
        // The key is dropped again: the copy in the Keystore is the one
        // that lasts.
        stride_api::ffi::set_garmin_connected_value(false);
        answer
    }

    /// Put the Garmin key in memory, or take it out again.
    ///
    /// The key is never echoed back: the answer says whether there is a
    /// key in memory, and nothing else.
    fn set_garmin_key(&self, key: &str) -> Value {
        stride_api::ffi::set_garmin_key_value(key)
    }

    /// Say whether the phone holds a Garmin key.
    fn set_garmin_connected(&self, connected: bool) -> Value {
        stride_api::ffi::set_garmin_connected_value(connected)
    }

    /// What the core is, and whether it is open.
    fn status(&self) -> Value {
        let mut answer = stride_api::ffi::status_value();
        if answer.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            if let Some(body) = answer.get_mut("body").and_then(Value::as_object_mut) {
                body.insert(
                    "backend".to_string(),
                    Value::String(crate::seam::BACKEND_STRIDE_API.to_string()),
                );
                body.insert(
                    "backend_marker".to_string(),
                    Value::String(crate::seam::COMPILED_BACKEND_MARKER.to_string()),
                );
            }
            answer
        } else {
            // The status answers what is compiled in even before the local
            // database is open. A release build can therefore ask what it
            // is carrying without first opening a database on a device.
            crate::status_envelope(false)
        }
    }
}

/// Decode a hexadecimal key.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = (bytes[index] as char).to_digit(16)?;
        let low = (bytes[index + 1] as char).to_digit(16)?;
        out.push((high * 16 + low) as u8);
        index += 2;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hexadecimal_key_is_decoded() {
        assert_eq!(decode_hex("00ff10"), Some(vec![0x00, 0xff, 0x10]));
        assert_eq!(decode_hex(""), None);
        assert_eq!(decode_hex("0"), None);
        assert_eq!(decode_hex("zz"), None);
    }
}
