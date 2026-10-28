//! The seam: the API the other crates call.
//!
//! Two crates call into this one — the desktop's router
//! (`crates/stride-api/src/state.rs`) and the phone's FFI
//! (`android/app/src/main/rust/stride-ffi/src/garmin.rs`). Neither is in this
//! crate's test run, and the phone's is not even in the workspace, so a change
//! to a signature or a field name here would break them silently.
//!
//! This file makes the same calls they make, so the break lands here first.
//!
//! Every client in a test is pointed at the local stub. `GarminClient::new`
//! talks to Garmin for real, and a test that made that call would be a test
//! that phoned home; the last test in this file makes sure no test does that.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::fs;
use std::path::PathBuf;

use chrono::{Duration, Utc};
use common::{Reply, StubServer};
use serde_json::{json, Value};
use stride_garmin::{ActivityDetail, GarminClient, GarminError};

const PROFILE_PATH: &str = "/userprofile-service/userprofile/settings";
const ACTIVITIES_PATH: &str = "/activitylist-service/activities/search/activities";
const DETAIL_PATH: &str = "/activity-service/activity/19012837465";
const DAILY_SUMMARY_PATH: &str = "/usersummary-service/usersummary/daily";
const SLEEP_PATH: &str = "/sleep-service/sleep/dailySleep";
const HRV_PATH: &str = "/hrv-service/hrv/2026-03-07";

fn stub() -> StubServer {
    let stub = StubServer::start();
    stub.push(
        "POST",
        stride_garmin::TOKEN_PATH,
        Reply::text("eyJhbGciOiJIUzI1NiJ9.compat-jwt.compat"),
    );
    stub.push(
        "GET",
        PROFILE_PATH,
        Reply::json(json!({
            "displayName": "Aarav Sharma",
            "fullName": "Aarav Kumar Sharma",
            "userName": "aarav.sharma",
        })),
    );
    stub.push(
        "GET",
        ACTIVITIES_PATH,
        Reply::json(json!({
            "activityList": [{
                "activityId": 19012837465u64,
                "activityName": "Morning Run",
                "startTimeLocal": "2026-03-07T06:15:00",
                "activityTypeDTO": { "typeKey": "running" },
                "summaryDTO": { "distance": 4993.22, "movingDuration": 2986.12 }
            }]
        })),
    );
    stub.push(
        "GET",
        DETAIL_PATH,
        Reply::json(json!({
            "activityId": 19012837465u64,
            "activityName": "Morning Run",
            "activityTypeDTO": { "typeKey": "running" },
            "summaryDTO": {
                "startTimeLocal": "2026-03-07T06:15:00",
                "distance": 4993.22,
                "movingDuration": 2986.12,
                "elapsedDuration": 3125.44
            }
        })),
    );
    stub.push(
        "GET",
        DAILY_SUMMARY_PATH,
        Reply::json(json!({ "summaryDTO": {} })),
    );
    stub.push("GET", SLEEP_PATH, Reply::json(json!({ "dailySleep": {} })));
    stub.push("GET", HRV_PATH, Reply::json(json!({ "hrvSummary": {} })));
    stub
}

/// The phone's FFI, `garmin.rs`, from `validate()` to the wellness pull.
#[test]
fn the_phone_ffi_calls_still_work() {
    let stub = stub();
    // The stub, never Garmin: a test must not make a network call.
    let client = GarminClient::with_base_url("the-key".to_string(), &stub.base_url())
        .expect("the stub is a loopback host");

    // `validate()` — the profile, read as the FFI reads it.
    let profile = client.validate().expect("the key is proven");
    let view = json!({
        "name": profile.athlete_name(),
        "display_name": profile.display_name,
        "full_name": profile.full_name,
        "user_name": profile.user_name,
    });
    assert_eq!(view["name"], "Aarav Sharma");
    assert_eq!(view["display_name"], "Aarav Sharma");

    // `sources()` — the map the FFI turns into JSON.
    let sources = client.sources();
    let mut map = serde_json::Map::new();
    for (name, available) in sources {
        map.insert(name.to_string(), json!(available));
    }
    assert!(
        map.contains_key("profile"),
        "the source list has not changed shape"
    );
    assert_eq!(map.len(), 12, "the source list has not changed shape");

    // The activity pull: list, detail, canonicalise.
    let to = Utc::now();
    let from = to - Duration::days(7);
    let summaries = client
        .activities(from, to)
        .expect("the activity list succeeds");
    assert_eq!(summaries.len(), 1);

    for summary in &summaries {
        // The raw map is what the phone hands back to the canonicaliser as the
        // list item, so it has to carry the id, the name and the start time.
        assert!(
            summary.raw.contains_key("activityId"),
            "the raw map lost the activity id"
        );
        assert!(
            summary.raw.contains_key("activityName"),
            "the raw map lost the activity name"
        );
        assert!(
            summary.raw.contains_key("startTimeLocal"),
            "the raw map lost the start time"
        );

        let detail = client
            .activity_detail(&summary.external_id)
            .expect("the detail succeeds");
        let list_item = Value::Object(summary.raw.clone());
        let canonical = ActivityDetail::from_value(&detail, Some(&list_item))
            .expect("the detail canonicalises");

        let row = json!({
            "external_id": canonical.external_id,
            "name": canonical.name,
            "sport_type": canonical.sport_type,
            "start_time": canonical.start_time,
            "local_date": canonical.local_date,
            "day_of_week": canonical.day_of_week,
            "distance_m": canonical.distance_m,
            "moving_time_s": canonical.moving_time_s,
            "elapsed_time_s": canonical.elapsed_time_s,
            "elevation_gain_m": canonical.elevation_gain_m,
            "avg_hr": canonical.avg_hr,
            "max_hr": canonical.max_hr,
            "calories": canonical.calories,
            "avg_cadence": canonical.avg_cadence,
            "max_cadence": canonical.max_cadence,
            "avg_speed_ms": canonical.avg_speed_ms,
            "max_speed_ms": canonical.max_speed_ms,
            "device": canonical.device,
            "is_trail": canonical.sport_type == "trail_running",
        });
        assert_eq!(row["external_id"], "19012837465");
        assert_eq!(row["name"], "Morning Run");
        assert_eq!(row["distance_m"], 4993.22);
        assert_eq!(row["moving_time_s"], 2986.12);
        assert_eq!(row["elapsed_time_s"], 3125.44);
        assert_eq!(row["local_date"], "2026-03-07");
        assert_eq!(row["day_of_week"], "Saturday");
    }

    // The wellness pull.
    let _ = client.daily_summary("2026-03-07");
    let _ = client.sleep("2026-03-07");
    let _ = client.hrv("2026-03-07");
}

/// The desktop's router, `state.rs`, reading the athlete's name.
#[test]
fn the_desktop_router_calls_still_work() {
    let stub = stub();
    // The stub, never Garmin: a test must not make a network call.
    let client = GarminClient::with_base_url("the-key".to_string(), &stub.base_url())
        .expect("the stub is a loopback host");

    let profile = client.validate().expect("the key is proven");
    let name = profile
        .display_name
        .as_ref()
        .or(profile.full_name.as_ref())
        .or(profile.user_name.as_ref())
        .or(profile.name.as_ref());
    assert_eq!(name.map(String::as_str), Some("Aarav Sharma"));
}

/// The error mapping the phone's FFI does, which must stay exhaustive over
/// the variants it names.
#[test]
fn the_error_mapping_the_consumers_use_still_matches() {
    fn map(err: GarminError) -> String {
        match err {
            GarminError::InvalidKey { .. } => "invalid".to_owned(),
            GarminError::RateLimited { .. } => "limited".to_owned(),
            GarminError::Forbidden => "forbidden".to_owned(),
            other => other.to_string(),
        }
    }

    assert_eq!(map(GarminError::InvalidKey { status: 401 }), "invalid");
    assert_eq!(
        map(GarminError::RateLimited {
            retry_after_secs: Some(30)
        }),
        "limited"
    );
    assert_eq!(map(GarminError::Forbidden), "forbidden");
    assert!(
        map(GarminError::NeedsReconnect).contains("reconnect"),
        "the reconnect error has to say so"
    );
}

/// No test may point a client at Garmin.
///
/// `GarminClient::new` uses Garmin's real base URL. A test that built one
/// would make a network call — to Garmin's token endpoint, with a key that
/// is not the athlete's. Every client in a test is built with
/// `with_base_url`, against the local stub.
#[test]
fn no_test_points_a_client_at_garmin() {
    // The needle is assembled at run time so that this file does not contain
    // it either.
    let needle = format!("GarminClient::new{}", "(");
    let tests: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut checked = 0;
    for entry in fs::read_dir(&tests).expect("the tests directory is there") {
        let path = entry.expect("the entry is there").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("the test file is there");
        assert!(
            !text.contains(&needle),
            "{} builds a client against Garmin; a test must use the stub",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 3, "every test file has to be checked");
}
