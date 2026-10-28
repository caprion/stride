//! The FFI tests.
//!
//! The phone calls one function and frees what it gets back. These tests call
//! it exactly as Kotlin does: through `extern "C"`, with a C string in and a
//! C string out.
//!
//! What they prove:
//!
//! * a successful call answers with the JSON envelope the phone expects;
//! * an error call answers with the status and the `detail` the Python app
//!   returns;
//! * **a panic in the backend does not cross the boundary** — it becomes an
//!   error, not an abort;
//! * a null pointer is answered, not dereferenced;
//! * the string the caller gets is owned by Rust and released by
//!   `stride_free`.

use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::Value;

use stride_api::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, Counts, DailyMetricRow, ImportRequest,
    LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields, PlanPhaseRow,
    PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SessionWithResult,
    SettingsPatch, StatusView, StreamRow, Tile, ZoneRow,
};
use stride_api::error::BackendError;
use stride_api::ffi;

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
        ffi::stride_call(
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
    unsafe { ffi::stride_free(raw) };
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
        unimplemented!()
    }
    fn activity_laps(&self, _id: i64) -> Result<Vec<LapRow>, BackendError> {
        unimplemented!()
    }
    fn activity_streams(&self, _id: i64) -> Result<Vec<StreamRow>, BackendError> {
        unimplemented!()
    }
    fn activity_zones(&self, _id: i64) -> Result<Vec<ZoneRow>, BackendError> {
        unimplemented!()
    }
    fn activity_geo(&self, _id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
        unimplemented!()
    }
    fn patch_activity(
        &self,
        _id: i64,
        _patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        unimplemented!()
    }
    fn physiology(&self, _from: &str, _to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
        unimplemented!()
    }
    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
        unimplemented!()
    }
    fn daily_metrics(
        &self,
        _from: Option<&str>,
        _to: Option<&str>,
    ) -> Result<Vec<DailyMetricRow>, BackendError> {
        unimplemented!()
    }
    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
        unimplemented!()
    }
    fn plan_block(&self, _id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
        unimplemented!()
    }
    fn plan_phases(&self, _block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
        unimplemented!()
    }
    fn plan_weeks(&self, _phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
        unimplemented!()
    }
    fn plan_sessions(&self, _week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
        unimplemented!()
    }
    fn plan_sessions_in_range(
        &self,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError> {
        unimplemented!()
    }
    fn create_plan_block(&self, _fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError> {
        unimplemented!()
    }
    fn create_plan_phase(
        &self,
        _block_id: i64,
        _fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError> {
        unimplemented!()
    }
    fn create_plan_week(
        &self,
        _phase_id: i64,
        _fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError> {
        unimplemented!()
    }
    fn create_plan_session(
        &self,
        _week_id: i64,
        _fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        unimplemented!()
    }
    fn import_plan(&self, _request: &ImportRequest) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn fingerprint(&self, _activity_id: i64) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn fingerprints(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn readiness(&self, _day: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn readiness_range(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn baselines(&self, _day: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn compare(&self, _ids: &[i64]) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn compare_candidates(&self, _limit: usize) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn plan_results(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn plan_adherence(&self, _from: &str, _to: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn plan_sessions_with_results(
        &self,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<SessionWithResult>, BackendError> {
        unimplemented!()
    }
    fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn max_hr_reference(&self, _activity_id: i64) -> Result<f64, BackendError> {
        unimplemented!()
    }
    fn network_status(&self) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn public_identity(&self) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn set_network_route(&self, _route: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn sync_status(&self) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn sync_runs(&self, _limit: usize) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn run_sync(&self, _days: Option<i64>, _background: bool) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn load_history(
        &self,
        _activities: Option<i64>,
        _health_days: Option<i64>,
        _background: bool,
    ) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn update_settings(&self, _patch: &SettingsPatch) -> Result<StatusView, BackendError> {
        unimplemented!()
    }
    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn pair_code(&self) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn pair(&self, _code: &str) -> Result<Option<Value>, BackendError> {
        unimplemented!()
    }
    fn devices(&self) -> Result<Vec<Value>, BackendError> {
        unimplemented!()
    }
    fn revoke_device(&self, _device_id: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn connect_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn test_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn tile_policy(&self) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn tile(&self, _z: i64, _x: i64, _y: i64) -> Result<Tile, BackendError> {
        unimplemented!()
    }
    fn garmin_oauth(&self, _action: &str, _body: &Value) -> Result<Value, BackendError> {
        unimplemented!()
    }
    fn import_fit(&self, _filename: &str, _bytes: &[u8]) -> Result<Value, BackendError> {
        unimplemented!()
    }
}

/// A successful call answers with the body of the route.
#[test]
fn a_successful_call_answers_with_the_body() {
    let _guard = lock();
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
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
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
    let answer = call("GET", "/api/v1/activities/999", None);
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["status"], 404);
    assert_eq!(answer["error"]["detail"], "Activity not found.");
}

/// A panic in the backend becomes an error, not an abort.
#[test]
fn a_panic_in_the_backend_becomes_an_error() {
    let _guard = lock();
    ffi::install_backend(Arc::new(ExplodingBackend)).expect("the backend installs");
    let answer = call("GET", "/api/v1/activities", None);
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["status"], 500);
    assert!(answer["error"]["detail"].is_string());
}

/// A null pointer is answered, not dereferenced.
#[test]
fn a_null_path_is_answered() {
    let _guard = lock();
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
    let method = CString::new("GET").expect("a method without a NUL");
    let raw = unsafe { ffi::stride_call(method.as_ptr(), std::ptr::null(), std::ptr::null()) };
    assert!(!raw.is_null());
    let text = unsafe { CStr::from_ptr(raw) }
        .to_str()
        .expect("UTF-8")
        .to_string();
    unsafe { ffi::stride_free(raw) };
    let answer: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(answer["ok"], false);
}

/// Freeing a null pointer is a no-op, not a crash.
#[test]
fn freeing_nothing_is_safe() {
    unsafe { ffi::stride_free(std::ptr::null_mut()) };
}

/// The query string travels with the path, as the phone sends it.
#[test]
fn the_query_string_travels_with_the_path() {
    let _guard = lock();
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
    let answer = call("GET", "/api/v1/activities?per_page=5&page=1", None);
    assert_eq!(answer["ok"], true);
    assert_eq!(answer["body"]["per_page"], 5);
    assert_eq!(answer["body"]["page"], 1);
}

/// A body travels with the call.
#[test]
fn a_body_travels_with_the_call() {
    let _guard = lock();
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
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
    ffi::install_backend(Arc::new(stride_api::MemBackend::new())).expect("the backend installs");
    let method = CString::new("GET").expect("a method without a NUL");
    let path = CString::new("/api/v1/health").expect("a path without a NUL");
    let raw = unsafe { ffi::stride_call(method.as_ptr(), path.as_ptr(), std::ptr::null()) };
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
    unsafe { ffi::stride_free(raw) };
}
