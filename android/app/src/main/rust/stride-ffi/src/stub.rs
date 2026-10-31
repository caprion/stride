//! The core that is not there yet.
//!
//! The Rust core is `stride-api` in the workspace above this crate. It is
//! not in the tree yet, so the seam is built against this: a core that opens
//! nothing, answers no route, and says so.
//!
//! **A stub can refuse; it cannot be wrong.** Every method here answers
//! with `503` and one sentence. There is no route it answers with a
//! number, so an APK built against the stub shows nothing at all — which is
//! the only honest thing an APK without a core can do.

use serde_json::Value;

use crate::seam::{error_envelope, not_built_envelope, ok_envelope, Core};

/// The core that is not in this build.
pub struct StubCore;

impl StubCore {
    /// A new stub.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubCore {
    fn default() -> Self {
        Self::new()
    }
}

impl Core for StubCore {
    fn open(&self, _db_path: &str, _db_key_hex: &str) -> Result<Value, String> {
        Err("the Rust core is not in this build. Nothing is stored until it is.".to_string())
    }

    fn call(&self, _method: &str, _path: &str, _body: Option<&str>) -> Value {
        not_built_envelope()
    }

    fn garmin_sync(&self, _api_key: &str, _days: i64) -> Value {
        not_built_envelope()
    }

    fn set_garmin_key(&self, api_key: &str) -> Value {
        // The key is never echoed back, not even by the stub: the answer
        // says whether there is a key in memory, and nothing else.
        ok_envelope(serde_json::json!({"in_memory": !api_key.is_empty()}))
    }

    fn set_garmin_connected(&self, connected: bool) -> Value {
        ok_envelope(serde_json::json!({"connected": connected}))
    }

    fn status(&self) -> Value {
        ok_envelope(serde_json::json!({
            "core": "stride-ffi",
            "version": env!("CARGO_PKG_VERSION"),
            "open": false,
            "backend": "stub",
        }))
    }
}

/// The answer for a call to a core that is not there.
///
/// It is a function of its own so that the shell's tests can ask for the
/// answer without a core.
pub fn stub_status() -> Value {
    error_envelope(503, "the Rust core is not in this build")
}
