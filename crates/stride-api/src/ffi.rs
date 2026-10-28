//! The C ABI the phone calls.
//!
//! One entry point, so the Kotlin side never changes when a route is added.
//! The contract is:
//!
//! * `stride_call` returns a **JSON string** the caller owns until it calls
//!   `stride_free` on it. The string is always valid JSON, and it always
//!   says what happened:
//!
//!   ```json
//!   {"ok": true,  "status": 200, "body": { ... }}
//!   {"ok": false, "status": 404, "error": {"detail": "Activity not found."}}
//!   ```
//!
//!   `body` is the response the HTTP route would return. `error` carries the
//!   same `detail` the Python app returns, so the frontend reads one shape
//!   whichever side answered.
//!
//! * `stride_free` releases a string that `stride_call` returned. Nothing
//!   else may be passed to it.
//!
//! * **No panic crosses the boundary.** A panic is caught and turned into an
//!   error JSON. (The workspace release profile sets `panic = "abort"`, which
//!   makes an abort the only possible outcome of a panic in a release build;
//!   the workstream report asks the orchestrator to decide whether the
//!   library profile should unwind instead. The code does not panic either
//!   way: there is no `unwrap`, no `expect` and no `panic!` in this crate.)

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, OnceLock, RwLock};

use serde_json::{json, Value};

use crate::backend::Backend;
use crate::error::ApiError;
use crate::Api;

/// The backend the FFI calls into.
///
/// The phone installs it once, at start-up, with the database it opened.
static BACKEND: OnceLock<RwLock<Option<Arc<dyn Backend>>>> = OnceLock::new();

/// The process-wide backend slot.
fn slot() -> &'static RwLock<Option<Arc<dyn Backend>>> {
    BACKEND.get_or_init(|| RwLock::new(None))
}

/// Install the backend the FFI calls into.
///
/// Calling it twice replaces the backend; the phone does it once, when it
/// opens its database.
pub fn install_backend(backend: Arc<dyn Backend>) -> Result<(), ApiError> {
    let mut guard = slot()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(backend);
    Ok(())
}

/// The installed backend, if there is one.
pub fn installed_backend() -> Option<Arc<dyn Backend>> {
    slot()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Call a route.
///
/// # Safety
///
/// `method`, `path` and `body` must be null or point to a NUL-terminated
/// C string that stays valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn stride_call(
    method: *const c_char,
    path: *const c_char,
    body: *const c_char,
) -> *mut c_char {
    let method = read_c_string(method).unwrap_or_default();
    let path = read_c_string(path).unwrap_or_default();
    let body = read_c_string(body);

    let outcome = catch_unwind(AssertUnwindSafe(|| {
        call_json(&method, &path, body.as_deref())
    }));

    let payload = match outcome {
        Ok(Ok(value)) => value,
        Ok(Err(err)) => error_payload(&err),
        Err(_) => error_payload(&ApiError::internal("the request could not be completed")),
    };
    let text = payload.to_string();
    match CString::new(text) {
        Ok(owned) => owned.into_raw(),
        // A JSON string never contains a NUL, so this cannot happen; if it
        // somehow does, the caller gets null and frees nothing.
        Err(_) => std::ptr::null_mut(),
    }
}

/// Release a string `stride_call` returned.
///
/// # Safety
///
/// `ptr` must be a pointer that `stride_call` returned and that has not
/// already been freed.
#[no_mangle]
pub unsafe extern "C" fn stride_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // The pointer came from `CString::into_raw`, so it is ours to free.
    drop(CString::from_raw(ptr));
}

/// Read a C string, if there is one.
unsafe fn read_c_string(raw: *const c_char) -> Option<String> {
    if raw.is_null() {
        return None;
    }
    CStr::from_ptr(raw).to_str().ok().map(str::to_string)
}

/// Run the router and wrap the answer in the FFI envelope.
fn call_json(method: &str, path: &str, body: Option<&str>) -> Result<Value, ApiError> {
    let Some(backend) = installed_backend() else {
        return Err(ApiError::internal("no backend is installed"));
    };
    let api = Api::new(backend);
    match api.call_response(method, path, body)? {
        crate::Response::Json(value) => Ok(json!({"ok": true, "status": 200, "body": value})),
        crate::Response::NoContent => Ok(json!({"ok": true, "status": 204, "body": Value::Null})),
        crate::Response::Binary {
            content_type,
            bytes,
        } => Ok(json!({
            "ok": true,
            "status": 200,
            "body": {
                "content_type": content_type,
                "base64": crate::util::base64_encode(&bytes),
            },
        })),
    }
}

/// The envelope of a failed call.
fn error_payload(err: &ApiError) -> Value {
    json!({
        "ok": false,
        "status": err.status,
        "error": err.to_body(),
    })
}
