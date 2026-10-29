//! The sync is incremental: a second pull costs nothing.
//!
//! This is the property the phone cannot afford to be wrong about. The
//! Activity API serves the whole history of the account, and the FIT file of
//! every activity in it, so a pull that re-fetches the detail and re-downloads
//! the FIT of every activity on every sync is 16 MB of mobile data a day, for
//! ever, for nothing.
//!
//! The reference's rule, which this reproduces:
//!
//! * an activity that has already been rebuilt from its original FIT file
//!   cannot have changed, so its detail is not fetched and its FIT is not
//!   downloaded again;
//! * a day of wellness that is already here is not pulled again.
//!
//! The client here is a stub that counts its calls, because that is the only
//! way to measure what a pull costs. The recorded fixtures cannot: they were
//! recorded from a single run.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::{json, Value};
use stride_api::backend::Backend;
use stride_api::sync::{run_with, GarminSource};
use stride_garmin::{ActivityDetail, ActivitySummary, GarminError};

/// A real FIT file: the one the reference's own fixtures use.
const FIT: &[u8] = include_bytes!("../../../backend/tests/golden/fixtures/real_garmin_run.fit");

/// The day the pull is made on.
fn today() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 10, 29).expect("a date")
}

/// A Garmin client that counts what it is asked for.
struct CountingClient {
    /// How many detail calls were made.
    detail_calls: AtomicUsize,
    /// How many FIT files were downloaded.
    fit_downloads: AtomicUsize,
    /// How many wellness calls were made.
    wellness_calls: AtomicUsize,
}

impl CountingClient {
    fn new() -> Self {
        Self {
            detail_calls: AtomicUsize::new(0),
            fit_downloads: AtomicUsize::new(0),
            wellness_calls: AtomicUsize::new(0),
        }
    }

    fn details(&self) -> usize {
        self.detail_calls.load(Ordering::Relaxed)
    }

    fn fits(&self) -> usize {
        self.fit_downloads.load(Ordering::Relaxed)
    }

    fn wellness(&self) -> usize {
        self.wellness_calls.load(Ordering::Relaxed)
    }
}

/// The activity the account holds.
const ACTIVITY_ID: &str = "17000000000";

impl GarminSource for CountingClient {
    fn activities(
        &self,
        _from: chrono::DateTime<chrono::Utc>,
        _to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ActivitySummary>, GarminError> {
        let mut raw = serde_json::Map::new();
        raw.insert("activityId".to_string(), json!(ACTIVITY_ID));
        raw.insert("activityName".to_string(), json!("Morning Run"));
        raw.insert("startTimeLocal".to_string(), json!("2026-10-20 06:30:00"));
        Ok(vec![ActivitySummary {
            external_id: ACTIVITY_ID.to_string(),
            name: "Morning Run".to_string(),
            sport_type: Some("running".to_string()),
            start_time_local: Some("2026-10-20 06:30:00".to_string()),
            raw,
        }])
    }

    fn activity_detail_canonical(&self, id: &str) -> Result<ActivityDetail, GarminError> {
        self.detail_calls.fetch_add(1, Ordering::Relaxed);
        Ok(ActivityDetail {
            external_id: id.to_string(),
            name: "Morning Run".to_string(),
            sport_type: "running".to_string(),
            start_time: Some("2026-10-20T01:00:00+00:00".to_string()),
            local_date: Some("2026-10-20".to_string()),
            day_of_week: Some("Tuesday".to_string()),
            distance_m: 10_000.0,
            moving_time_s: 3_000.0,
            elapsed_time_s: 3_200.0,
            elevation_gain_m: 50.0,
            avg_hr: Some(150.0),
            max_hr: Some(180.0),
            min_hr: Some(120.0),
            calories: Some(700.0),
            avg_cadence: Some(170.0),
            max_cadence: Some(180.0),
            avg_speed_ms: Some(3.3),
            max_speed_ms: Some(4.5),
            device: Some("Watch".to_string()),
            raw: serde_json::Map::new(),
        })
    }

    fn activity_fit(&self, _id: &str) -> Result<Vec<u8>, GarminError> {
        self.fit_downloads.fetch_add(1, Ordering::Relaxed);
        Ok(FIT.to_vec())
    }

    fn daily_summary(&self, _day: &str) -> Result<Value, GarminError> {
        self.wellness_calls.fetch_add(1, Ordering::Relaxed);
        Ok(json!({
            "summaryDTO": {
                "totalSteps": 8_000,
                "restingHeartRate": 48,
                "maxHeartRate": 170,
                "averageStressLevel": 30,
                "maxStressLevel": 90,
                "bodyBatteryLowestValue": 20,
                "bodyBatteryHighestValue": 90,
                "bodyBatteryMostRecentValue": 55,
            }
        }))
    }

    fn sleep(&self, _day: &str) -> Result<Value, GarminError> {
        self.wellness_calls.fetch_add(1, Ordering::Relaxed);
        Ok(json!({
            "sleepTimeSeconds": 28_800,
            "deepSleepSeconds": 7_200,
            "remSleepSeconds": 5_400,
            "lightSleepSeconds": 14_400,
            "awakeSleepSeconds": 1_800,
        }))
    }

    fn hrv(&self, _day: &str) -> Result<Value, GarminError> {
        self.wellness_calls.fetch_add(1, Ordering::Relaxed);
        Ok(json!({ "hrvSummary": { "lastNightAvg": 62, "status": "BALANCED" } }))
    }

    fn daily_heart_rate(&self, _day: &str) -> Result<Value, GarminError> {
        self.wellness_calls.fetch_add(1, Ordering::Relaxed);
        Ok(json!({
            "heartRateSummary": { "restingHeartRate": 48, "maxHeartRate": 170 }
        }))
    }
}

/// The store the pull writes into.
fn store(name: &str) -> Arc<dyn Backend> {
    let path = std::env::temp_dir().join(format!("stride-api-incremental-{name}.db"));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    let store = stride_store::Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    stride_api::store::backend(Arc::new(store))
}

/// The second pull fetches nothing: no detail, no FIT, no wellness.
///
/// This is the property that matters on a phone, and nothing else catches
/// it: the recorded fixtures were recorded from a single run.
#[test]
fn the_second_pull_costs_nothing() {
    let backend = store("second");
    let client = CountingClient::new();

    // The first pull: everything is fetched, because nothing is here yet.
    let first = run_with(backend.as_ref(), &client, Some(90), Some(3), today())
        .expect("the first pull works");
    assert_eq!(first.activities_found, 1);
    assert_eq!(first.activities_new, 1);
    assert_eq!(first.detail_calls, 1, "the first pull fetches the detail");
    assert_eq!(first.fit_downloads, 1, "the first pull downloads the FIT");
    assert_eq!(
        first.health_days_fetched, 3,
        "the first pull fetches the days"
    );
    assert_eq!(first.health_days_skipped, 0);

    // The second pull: the activity is already here, rebuilt from its FIT,
    // and the days of wellness are already here too.
    let second = run_with(backend.as_ref(), &client, Some(90), Some(3), today())
        .expect("the second pull works");
    assert_eq!(second.activities_found, 1);
    assert_eq!(second.activities_new, 0);
    assert_eq!(second.activities_updated, 1);
    assert_eq!(
        second.detail_calls, 0,
        "the second pull must not fetch the detail again"
    );
    assert_eq!(
        second.fit_downloads, 0,
        "the second pull must not download the FIT again"
    );
    assert_eq!(
        second.health_days_fetched, 0,
        "the second pull must not fetch the wellness again"
    );
    assert_eq!(second.health_days_skipped, 3);
    assert_eq!(client.details(), 1, "one detail call in total");
    assert_eq!(client.fits(), 1, "one FIT download in total");
    assert_eq!(
        client.wellness(),
        12,
        "four calls for each of the three days"
    );
}

/// The activity keeps its track, its splits and its per-second series.
///
/// The reference reuses the stored track when the activity has already been
/// rebuilt: without this the canonical `None` would overwrite the stored
/// track and the streams.
#[test]
fn the_second_pull_keeps_the_streams() {
    let backend = store("streams");
    let client = CountingClient::new();
    run_with(backend.as_ref(), &client, Some(90), Some(3), today()).expect("the first pull works");

    let activity = backend
        .activity_by_external("garmin_personal", ACTIVITY_ID)
        .expect("the activity is readable")
        .expect("the activity is there");
    let streams = backend
        .activity_streams(activity.id)
        .expect("the streams are readable");
    assert!(
        !streams.is_empty(),
        "the FIT's per-second series was written"
    );
    let laps = backend
        .activity_laps(activity.id)
        .expect("the splits are readable");
    assert!(!laps.is_empty(), "the FIT's splits were written");

    // And they are still there after the second pull.
    run_with(backend.as_ref(), &client, Some(90), Some(3), today()).expect("the second pull works");
    let again = backend
        .activity_streams(activity.id)
        .expect("the streams are readable");
    assert_eq!(
        again.len(),
        streams.len(),
        "the second pull kept the per-second series"
    );
    let splits = backend
        .activity_laps(activity.id)
        .expect("the splits are readable");
    assert_eq!(splits.len(), laps.len(), "the second pull kept the splits");
}
