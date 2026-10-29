// The test code unwraps and expects: it is the test, and the workspace
// denies those lints in library code only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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

/// A FIT file goes in, and the router serves it.
#[test]
fn a_fit_file_goes_in_and_comes_out() {
    let (store, _directory) = store();
    let store = Arc::new(store);
    let backend = stride_api::store::backend(store.clone());
    let api = Api::with_today(
        Arc::new(CoreBackend::new(backend)),
        chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
    );

    // One of the golden fixtures: a run with a walk and a stand in it.
    let path = std::path::Path::new("../../backend/tests/golden/fixtures/mixed_run.fit");
    let bytes = std::fs::read(path).expect("the fixture is there");

    let imported = api
        .call(
            "POST",
            "/api/v1/auth/garmin/fit-import",
            Some(&format!(
                r#"{{"files": [{{"filename": "mixed_run.fit", "content_base64": "{}"}}]}}"#,
                stride_api::util::base64_encode(&bytes)
            )),
        )
        .expect("the import answers");
    assert_eq!(imported["imported"], 1);
    assert_eq!(imported["errors"].as_array().map(Vec::len), Some(0));
    let activity = &imported["activities"][0];
    assert_eq!(activity["name"], "mixed run");
    assert_eq!(activity["date"], "2026-03-02");
    assert_eq!(activity["distance_m"], 443.0);
    assert_eq!(activity["created"], true);

    // The router serves it, with the numbers the reference recorded.
    let id = activity["id"].as_i64().expect("an id");
    let detail = api
        .call("GET", &format!("/api/v1/activities/{id}"), None)
        .expect("the detail answers");
    assert_eq!(detail["distance_m"], 443.0);
    assert_eq!(detail["pace_s_per_km"], 338.6);
    assert_eq!(detail["pace_label"], "5:39");
    assert_eq!(detail["running_time_s"], 139.0);
    assert_eq!(detail["walking_time_s"], 10.0);
    assert_eq!(detail["stopped_time_s"], 20.0);
    assert_eq!(detail["lap_count"], 2);

    // Importing the same file again is the same run, not a second one.
    let again = api
        .call(
            "POST",
            "/api/v1/auth/garmin/fit-import",
            Some(&format!(
                r#"{{"files": [{{"filename": "mixed_run.fit", "content_base64": "{}"}}]}}"#,
                stride_api::util::base64_encode(&bytes)
            )),
        )
        .expect("the import answers");
    assert_eq!(again["activities"][0]["id"], id);
    assert_eq!(again["activities"][0]["created"], false);
}

/// A plan document goes in, and the plan tables hold it.
#[test]
fn a_plan_document_goes_in_and_comes_out() {
    let (store, _directory) = store();
    // A run in March, so the plan has a history to be anchored to.
    seed(&store);
    let store = Arc::new(store);
    let backend = stride_api::store::backend(store.clone());
    let api = Api::with_today(
        Arc::new(CoreBackend::new(backend)),
        chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
    );

    let document = "# 2026 Spring Block\n\n## Weekly running volume\n\n| Week | Phase | Running | Tuesday | Notes |\n|---|---|---|---|---|\n| 1 | Base | 25 km | Quality | easy |\n| 2 | Base | 30 km | Tempo 12km @ 4:30/km | |\n\n## Week 1 - Base (March 2 - March 8) 25 km\n\n| Day | Session | Details |\n|---|---|---|\n| Mon | Rest | |\n| Tue | Easy 8km | 4:45-5:00/km |\n| Sat | Long 14km | 5:15/km |\n";
    let body = serde_json::json!({
        "format": "markdown",
        "content": document,
        "name": "Spring block",
        "anchor": "activities",
    })
    .to_string();

    let imported = api
        .call("POST", "/api/v1/plan/import", Some(&body))
        .expect("the import answers");
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["name"], "Spring block");
    assert_eq!(imported["weeks"], 2);
    // Three dated sessions in the first week, and four from the template in
    // the second, which has no dated rows.
    assert_eq!(imported["sessions"], 7);
    assert_eq!(imported["error"], serde_json::Value::Null);
    let block_id = imported["block_id"].as_i64().expect("a block");

    // The block is there, with its phases, weeks and sessions.
    let blocks = api
        .call("GET", "/api/v1/plan/blocks", None)
        .expect("the blocks answer");
    assert_eq!(blocks["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(blocks["items"][0]["id"], block_id);
    assert_eq!(blocks["items"][0]["source"], "imported");
    assert_eq!(blocks["items"][0]["week_count"], 2);
    assert_eq!(blocks["items"][0]["session_count"], 7);

    let detail = api
        .call("GET", &format!("/api/v1/plan/blocks/{block_id}"), None)
        .expect("the block answers");
    let weeks = detail["phases"][0]["weeks"].as_array().expect("the weeks");
    assert_eq!(weeks.len(), 2);
    // The plan was anchored to the run: it now ends in the week of the run.
    let first_start = weeks[0]["week"]["start_date"].as_str().expect("a date");
    assert_eq!(first_start, "2026-02-23");
    assert_eq!(weeks[0]["week"]["end_date"], "2026-03-01");
    let sessions = weeks[0]["sessions"].as_array().expect("the sessions");
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[1]["session_type"], "easy");
    assert_eq!(sessions[1]["target_distance_m"], 8000.0);
    // Read back out of the database the pace is a float, as the reference's
    // is: the importer's 285 is an integer, the column is a float.
    assert_eq!(sessions[1]["pace_low_s_km"], 285.0);
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
