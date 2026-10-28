//! The phone's own tests.
//!
//! They exercise the path the phone takes: open the database with a key, ask
//! the router for a route, and check the answer is the shape the SPA reads.
//!
//! No fixture is invented here: the numbers come from the routes, and the
//! routes are the contract.

// The tests are allowed to expect: they are a separate crate, not a
// `#[cfg(test)]` module, so the workspace's lint configuration does not
// reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::Value;
use stride_store::model::{Activity, ActivityLap};
use stride_store::Store;

/// The lock the tests share.
///
/// The backend is process-wide — one phone, one database — so the tests take
/// turns rather than fight over it.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let mutex = LOCK.get_or_init(|| Mutex::new(()));
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// A temporary directory that goes away with the test.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("stride-ffi-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the temporary directory could not be made");
        Self(path)
    }

    fn db(&self) -> String {
        self.0.join("stride.db").to_string_lossy().into_owned()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A 32-byte key, hex-encoded, the way the Keystore hands it over.
fn key_hex(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

/// A run, as the store keeps it.
fn a_run() -> Activity {
    Activity {
        source: "garmin_personal".to_string(),
        external_id: "run-1".to_string(),
        name: "Morning run".to_string(),
        sport_type: "running".to_string(),
        start_time: chrono::Utc::now(),
        local_date: "2026-03-02".to_string(),
        day_of_week: Some("Monday".to_string()),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 120.0,
        avg_hr: Some(150.0),
        max_hr: Some(172.0),
        calories: Some(700.0),
        avg_cadence: Some(170.0),
        ..Activity::default()
    }
}

#[test]
fn the_database_opens_and_the_router_answers() {
    let _guard = lock();
    let dir = TempDir::new("open");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(7));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {answer}");
    assert_eq!(parsed["body"]["schema"], "migrated");

    let health = uniffi_stride::stride_call("GET".into(), "/api/v1/health".into(), None);
    let parsed: Value = serde_json::from_str(&health).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {health}");
    assert_eq!(parsed["status"], 200);
}

#[test]
fn a_wrong_key_is_an_error_and_not_a_crash() {
    let _guard = lock();
    let dir = TempDir::new("wrong-key");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(9));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {answer}");

    // A different key cannot open the file that is already there.
    let other = TempDir::new("wrong-key-other");
    std::fs::copy(dir.db(), other.db()).expect("the database could not be copied");
    let answer = uniffi_stride::stride_open(other.db(), key_hex(8));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], false, "the answer was {answer}");
    assert!(parsed["error"]["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("could not be opened"));
}

#[test]
fn the_routes_answer_from_the_phone_database() {
    let _guard = lock();
    let dir = TempDir::new("routes");
    let key = key_hex(3);
    let answer = uniffi_stride::stride_open(dir.db(), key.clone());
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // Two runs, written the way the sync writes them.
    {
        let store = Store::open(std::path::Path::new(&dir.db()), &[3u8; 32])
            .expect("the store could not be opened");
        let run = a_run();
        let id = store.upsert_activity(&run).expect("the run was not written");
        let lap = ActivityLap {
            activity_id: id,
            lap_index: 1,
            distance_m: 1_000.0,
            duration_s: 300.0,
            avg_hr: Some(150.0),
            avg_cadence: Some(170.0),
            ..ActivityLap::default()
        };
        store
            .replace_activity_laps(id, &[lap])
            .expect("the lap was not written");
    }

    let list = uniffi_stride::stride_call(
        "GET".into(),
        "/api/v1/activities?per_page=10".into(),
        None,
    );
    let parsed: Value = serde_json::from_str(&list).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {list}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 1, "the list was {list}");
    assert_eq!(items[0]["name"], "Morning run");
    assert_eq!(items[0]["distance_m"], 10_000.0);

    let detail = uniffi_stride::stride_call(
        "GET".into(),
        "/api/v1/activities/1".into(),
        None,
    );
    let parsed: Value = serde_json::from_str(&detail).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {detail}");
    assert_eq!(parsed["body"]["name"], "Morning run");
    assert_eq!(parsed["body"]["distance_m"], 10_000.0);
    // The splits come with the run.
    let laps = parsed["body"]["laps"].as_array().expect("there are laps");
    assert_eq!(laps.len(), 1, "the detail was {detail}");
    assert_eq!(laps[0]["distance_m"], 1_000.0);

    // The fingerprint is the domain's answer, computed on the phone.
    let fingerprint = uniffi_stride::stride_call(
        "GET".into(),
        "/api/v1/fitness/fingerprint".into(),
        None,
    );
    let parsed: Value = serde_json::from_str(&fingerprint).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {fingerprint}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 1, "the fingerprint list was {fingerprint}");
    assert_eq!(items[0]["activity_id"], 1);
    assert_eq!(items[0]["distance_m"], 10_000.0);
}

#[test]
fn the_routes_the_phone_cannot_answer_say_so() {
    let _guard = lock();
    let dir = TempDir::new("unsupported");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(5));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    let pairing = uniffi_stride::stride_call("GET".into(), "/api/v1/auth/pair-code".into(), None);
    let parsed: Value = serde_json::from_str(&pairing).expect("the answer is JSON");
    assert_eq!(parsed["ok"], false, "the answer was {pairing}");
    assert_eq!(parsed["status"], 501);

    // The tile policy is the honest one: the phone ships no basemap.
    let policy = uniffi_stride::stride_call("GET".into(), "/api/v1/tiles/policy".into(), None);
    let parsed: Value = serde_json::from_str(&policy).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {policy}");
    assert_eq!(parsed["body"]["policy"], "none");
}

#[test]
fn the_status_answers_before_anything_is_open() {
    let status = uniffi_stride::stride_status();
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["body"]["core"], "stride-ffi");
}
