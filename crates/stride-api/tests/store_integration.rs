//! The store and the core, end to end.
//!
//! The router is only useful if it can read a real database and answer with
//! the domain logic on top. This test opens a real store in a temporary
//! directory, writes a run into it, and drives the router with the result —
//! the same path the desktop and the phone take.

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use serde_json::json;
use stride_api::{Api, CoreBackend};
use stride_store::{Activity, ActivityLap, Store};

/// A store in a temporary directory, with the schema in it.
fn store() -> (Store, tempdir::TempDir) {
    let directory = tempdir::TempDir::new("stride-api-store").expect("a temporary directory");
    let path = directory.path().join("stride.db");
    let store = Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    (store, directory)
}

/// One run, with its splits and its series.
fn seed(store: &Store) -> i64 {
    let activity = Activity {
        source: "garmin_fit".to_string(),
        external_id: "3117001".to_string(),
        name: "morning run".to_string(),
        sport_type: "running".to_string(),
        start_time: Utc.with_ymd_and_hms(2026, 3, 2, 6, 15, 0).unwrap(),
        local_date: "2026-03-02".to_string(),
        day_of_week: Some("Monday".to_string()),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_100.0,
        elevation_gain_m: 120.0,
        avg_hr: Some(150.0),
        max_hr: Some(175.0),
        calories: Some(700.0),
        avg_cadence: Some(170.0),
        max_cadence: Some(180.0),
        avg_speed_ms: Some(3.3333333333333335),
        max_speed_ms: Some(4.2),
        running_time_s: Some(3_000.0),
        walking_time_s: Some(0.0),
        stopped_time_s: Some(0.0),
        running_distance_m: Some(10_000.0),
        device: Some("Forerunner 265".to_string()),
        ..Activity::default()
    };
    let id = store
        .upsert_activity(&activity)
        .expect("the run is written");
    store
        .replace_activity_laps(
            id,
            &[
                ActivityLap {
                    activity_id: id,
                    lap_index: 1,
                    distance_m: 5_000.0,
                    duration_s: 1_500.0,
                    avg_speed_ms: Some(3.3333333333333335),
                    avg_hr: Some(150.0),
                    max_hr: Some(170.0),
                    avg_cadence: Some(170.0),
                    max_cadence: Some(175.0),
                    ..ActivityLap::default()
                },
                ActivityLap {
                    activity_id: id,
                    lap_index: 2,
                    distance_m: 5_000.0,
                    duration_s: 1_500.0,
                    avg_speed_ms: Some(3.3333333333333335),
                    avg_hr: Some(152.0),
                    max_hr: Some(175.0),
                    avg_cadence: Some(172.0),
                    max_cadence: Some(180.0),
                    ..ActivityLap::default()
                },
            ],
        )
        .expect("the splits are written");
    store
        .replace_activity_streams(
            id,
            &[stride_store::ActivityStream {
                activity_id: id,
                kind: "hr".to_string(),
                seconds: json!([0.0, 1.0, 2.0]),
                values: json!([140.0, 150.0, 160.0]),
                label: Some("Heart rate".to_string()),
                unit: Some("bpm".to_string()),
                ..stride_store::ActivityStream::default()
            }],
        )
        .expect("the series is written");
    id
}

#[test]
fn the_router_reads_a_real_database() {
    let (store, _directory) = store();
    let id = seed(&store);

    // The store cannot be cloned, so the router borrows it through the same
    // `Arc` the caller owns.
    let store = Arc::new(store);
    let backend = stride_api::store::backend(store.clone());
    let api = Api::with_today(
        Arc::new(CoreBackend::new(backend)),
        chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
    );

    let list = api
        .call("GET", "/api/v1/activities", None)
        .expect("the list answers");
    assert_eq!(list["total"], 1);
    let item = &list["items"][0];
    assert_eq!(item["id"], id);
    assert_eq!(item["name"], "morning run");
    assert_eq!(item["distance_m"], 10_000.0);
    // 3000 s over 10 km is 5:00 per kilometre.
    assert_eq!(item["pace_s_per_km"], 300.0);
    assert_eq!(item["pace_label"], "5:00");
    assert_eq!(item["moving_time_label"], "50:00");
    assert_eq!(item["lap_count"], 2);
    assert_eq!(
        item["garmin_url"],
        "https://connect.garmin.com/modern/activity/3117001"
    );

    let detail = api
        .call("GET", &format!("/api/v1/activities/{id}"), None)
        .expect("the detail answers");
    assert_eq!(detail["laps"].as_array().map(Vec::len), Some(2));
    assert_eq!(detail["laps"][0]["pace_s_per_km"], 300.0);
    assert_eq!(detail["laps"][0]["distance_label"], "5.00 km");
    assert!(detail["streams"].get("hr").is_some());

    let health = api
        .call("GET", "/api/v1/health", None)
        .expect("the health answers");
    assert_eq!(health["data"]["activities"], 1);
    assert_eq!(health["data"]["from_garmin"], 1);

    // The fingerprint is `stride-core`'s work, on top of the same rows.
    let fingerprint = api
        .call("GET", &format!("/api/v1/fitness/activity/{id}"), None)
        .expect("the fingerprint answers");
    assert_eq!(fingerprint["activity_id"], id);
    assert_eq!(fingerprint["cadence_avg"], 171.0);
    assert_eq!(fingerprint["hr_avg"], 150.0);
    assert_eq!(fingerprint["local_date"], "2026-03-02");
    assert_eq!(fingerprint["pace_s_per_km"], 300.0);

    let stats = api
        .call("GET", "/api/v1/stats/summary", None)
        .expect("the summary answers");
    assert_eq!(stats["totals"]["activity_count"], 1);
    assert_eq!(stats["totals"]["distance_m"], 10_000.0);
    assert_eq!(stats["totals"]["avg_pace_s_per_km"], 300.0);
}

/// A tiny temporary-directory helper, so the test does not need a crate.
mod tempdir {
    use std::path::{Path, PathBuf};

    /// A directory that removes itself when it is dropped.
    pub struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        /// Make a directory with a name nobody else is using.
        pub fn new(prefix: &str) -> std::io::Result<Self> {
            let mut path = std::env::temp_dir();
            let unique = format!(
                "{prefix}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|since| since.as_nanos())
                    .unwrap_or_default()
            );
            path.push(unique);
            std::fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        /// The directory itself.
        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
