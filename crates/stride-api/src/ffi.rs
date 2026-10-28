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
//!   error JSON, and the release profile unwinds so the catch works there
//!   too. The code does not rely on the catch: there is no `unwrap`, no
//!   `expect` and no `panic!` in this crate's library code at all.

// The C ABI is unsafe, and this file is the one place in the workspace where
// `unsafe` is allowed: it is the boundary the phone crosses.
#![allow(unsafe_code)]

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

#[cfg(test)]
mod tests {
    //! The FFI, called the way Kotlin calls it.
    //!
    //! These tests live here rather than in `tests/ffi.rs` for one reason:
    //! `unsafe` is confined to this file. The C ABI is unsafe, and so is
    //! every call to it — so the calls live next to the boundary they cross.

    use std::ffi::{CStr, CString};
    use std::sync::{Arc, Mutex, OnceLock};

    use serde_json::Value;

    use super::{install_backend, stride_call, stride_free};
    use crate::backend::{
        ActivityFilter, ActivityPatch, ActivityRow, Backend, Counts, DailyMetricRow, ImportRequest,
        LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields, PlanPhaseRow,
        PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SessionWithResult,
        SettingsPatch, StatusView, StreamRow, Tile, ZoneRow,
    };
    use crate::error::BackendError;

    /// The tests share one process-wide backend, so they take a turn.
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Call the FFI and hand back the JSON it answered with.
    fn call(method: &str, path: &str, body: Option<&str>) -> Value {
        let method = CString::new(method).expect("a method without a NUL");
        let path = CString::new(path).expect("a path without a NUL");
        let body = body.map(|body| CString::new(body).expect("a body without a NUL"));
        let raw = unsafe {
            stride_call(
                method.as_ptr(),
                path.as_ptr(),
                body.as_ref()
                    .map(|body| body.as_ptr())
                    .unwrap_or(std::ptr::null()),
            )
        };
        assert!(!raw.is_null(), "the FFI must always answer with a string");
        let text = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .expect("the answer is UTF-8")
            .to_string();
        unsafe { stride_free(raw) };
        serde_json::from_str(&text).expect("the answer is JSON")
    }

    /// A backend that answers the health route and explodes on everything else.
    struct ExplodingBackend;

    impl Backend for ExplodingBackend {
        fn public_status(&self) -> Result<StatusView, BackendError> {
            Ok(StatusView::default())
        }

        fn counts(&self) -> Result<Counts, BackendError> {
            Ok(Counts::default())
        }

        fn activities(&self, _filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError> {
            panic!("the backend exploded");
        }

        fn activity_count(&self, _filter: &ActivityFilter) -> Result<i64, BackendError> {
            panic!("the backend exploded");
        }

        fn activity(&self, _id: i64) -> Result<Option<ActivityRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn activity_laps(&self, _id: i64) -> Result<Vec<LapRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn activity_streams(&self, _id: i64) -> Result<Vec<StreamRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn activity_zones(&self, _id: i64) -> Result<Vec<ZoneRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn activity_geo(&self, _id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn patch_activity(
            &self,
            _id: i64,
            _patch: &ActivityPatch,
        ) -> Result<Option<ActivityRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn physiology(&self, _from: &str, _to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn daily_metrics(
            &self,
            _from: Option<&str>,
            _to: Option<&str>,
        ) -> Result<Vec<DailyMetricRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_block(&self, _id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_phases(&self, _block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_weeks(&self, _phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_sessions(&self, _week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_sessions_in_range(
            &self,
            _from: &str,
            _to: &str,
        ) -> Result<Vec<PlanSessionRow>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn create_plan_block(
            &self,
            _fields: &PlanBlockFields,
        ) -> Result<PlanBlockRow, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn create_plan_phase(
            &self,
            _block_id: i64,
            _fields: &PlanPhaseFields,
        ) -> Result<PlanPhaseRow, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn create_plan_week(
            &self,
            _phase_id: i64,
            _fields: &PlanWeekFields,
        ) -> Result<PlanWeekRow, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn create_plan_session(
            &self,
            _week_id: i64,
            _fields: &PlanSessionFields,
        ) -> Result<PlanSessionRow, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn import_plan(&self, _request: &ImportRequest) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn fingerprint(&self, _activity_id: i64) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn fingerprints(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn readiness(&self, _day: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn readiness_range(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn baselines(&self, _day: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn compare(&self, _ids: &[i64]) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn compare_candidates(&self, _limit: usize) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_results(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_adherence(&self, _from: &str, _to: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn plan_sessions_with_results(
            &self,
            _from: &str,
            _to: &str,
        ) -> Result<Vec<SessionWithResult>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn max_hr_reference(&self, _activity_id: i64) -> Result<f64, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn network_status(&self) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn public_identity(&self) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn set_network_route(&self, _route: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn sync_status(&self) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn sync_runs(&self, _limit: usize) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn run_sync(&self, _days: Option<i64>, _background: bool) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn load_history(
            &self,
            _activities: Option<i64>,
            _health_days: Option<i64>,
            _background: bool,
        ) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn update_settings(&self, _patch: &SettingsPatch) -> Result<StatusView, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn gear(&self) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn pair_code(&self) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn pair(&self, _code: &str) -> Result<Option<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn devices(&self) -> Result<Vec<Value>, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn revoke_device(&self, _device_id: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn connect_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn test_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn tile_policy(&self) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn tile(&self, _z: i64, _x: i64, _y: i64) -> Result<Tile, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn garmin_oauth(&self, _action: &str, _body: &Value) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
        fn import_fit(&self, _filename: &str, _bytes: &[u8]) -> Result<Value, BackendError> {
            Err(BackendError::Unsupported(
                "the FFI test does not call this".to_string(),
            ))
        }
    }

    /// A successful call answers with the body of the route.
    #[test]
    fn a_successful_call_answers_with_the_body() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let answer = call("GET", "/api/v1/health", None);
        assert_eq!(answer["ok"], true);
        assert_eq!(answer["status"], 200);
        assert_eq!(answer["body"]["ok"], true);
        assert_eq!(answer["body"]["service"], "stride");
    }

    /// An error answers with the status and the detail the Python app returns.
    #[test]
    fn an_error_call_answers_with_the_detail() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let answer = call("GET", "/api/v1/activities/999", None);
        assert_eq!(answer["ok"], false);
        assert_eq!(answer["status"], 404);
        assert_eq!(answer["error"]["detail"], "Activity not found.");
    }

    /// A panic in the backend becomes an error, not an abort.
    #[test]
    fn a_panic_in_the_backend_becomes_an_error() {
        let _guard = lock();
        install_backend(Arc::new(ExplodingBackend)).expect("the backend installs");
        let answer = call("GET", "/api/v1/activities", None);
        assert_eq!(answer["ok"], false);
        assert_eq!(answer["status"], 500);
        assert!(answer["error"]["detail"].is_string());
    }

    /// A null pointer is answered, not dereferenced.
    #[test]
    fn a_null_path_is_answered() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let method = CString::new("GET").expect("a method without a NUL");
        let raw = unsafe { stride_call(method.as_ptr(), std::ptr::null(), std::ptr::null()) };
        assert!(!raw.is_null());
        let text = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .expect("UTF-8")
            .to_string();
        unsafe { stride_free(raw) };
        let answer: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(answer["ok"], false);
    }

    /// Freeing a null pointer is a no-op, not a crash.
    #[test]
    fn freeing_nothing_is_safe() {
        unsafe { stride_free(std::ptr::null_mut()) };
    }

    /// The query string travels with the path, as the phone sends it.
    #[test]
    fn the_query_string_travels_with_the_path() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let answer = call("GET", "/api/v1/activities?per_page=5&page=1", None);
        assert_eq!(answer["ok"], true);
        assert_eq!(answer["body"]["per_page"], 5);
        assert_eq!(answer["body"]["page"], 1);
    }

    /// A body travels with the call.
    #[test]
    fn a_body_travels_with_the_call() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let answer = call(
            "POST",
            "/api/v1/auth/garmin/personal-key/test",
            Some(r#"{"api_key": "0123456789012345678901234567890123456789"}"#),
        );
        assert_eq!(answer["ok"], true);
        assert_eq!(answer["body"]["ok"], true);
    }

    /// The string the caller gets back is owned by Rust.
    #[test]
    fn the_returned_string_is_owned_by_rust() {
        let _guard = lock();
        install_backend(Arc::new(crate::mem::MemBackend::new())).expect("the backend installs");
        let method = CString::new("GET").expect("a method without a NUL");
        let path = CString::new("/api/v1/health").expect("a path without a NUL");
        let raw = unsafe { stride_call(method.as_ptr(), path.as_ptr(), std::ptr::null()) };
        assert!(!raw.is_null());
        // Reading it twice is fine: it is a plain C string.
        let first = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .expect("UTF-8")
            .to_string();
        let second = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .expect("UTF-8")
            .to_string();
        assert_eq!(first, second);
        unsafe { stride_free(raw) };
    }
}
