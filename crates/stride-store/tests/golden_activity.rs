//! The golden activities, through the store.
//!
//! `backend/tests/golden/golden/*.json` holds the activity the **Python**
//! reference produced for each FIT fixture. This test takes those activities
//! exactly as the reference wrote them, puts them through the store, and
//! reads them back: the values have to survive, and the timestamp has to
//! come back as the same text the golden file holds.
//!
//! The file is named `golden_activity.rs` and not `golden.rs` on purpose:
//! `crates/*/tests/golden/**` belongs to the golden workstream.

// An integration test is its own crate, so clippy's allow-in-tests
// configuration does not cover it: the test code opts out here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use stride_store::{datetimes, Activity, Store};

/// The golden files.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../backend/tests/golden/golden")
}

/// The text a `Value` holds, if it is a string.
fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

/// The number a `Value` holds, if it is a number.
fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(|v| v.as_f64())
}

/// The whole number a `Value` holds, if it is one.
fn integer(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(|v| v.as_i64())
}

/// The flag a `Value` holds, if it is one.
fn flag(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(|v| v.as_bool())
}

/// The activity of a golden file, as the store's row.
fn activity_from_golden(golden: &Value) -> Activity {
    let start_time = DateTime::parse_from_rfc3339(golden["start_time"].as_str().unwrap())
        .unwrap()
        .with_timezone(&Utc);
    Activity {
        source: text(golden, "source").unwrap_or_else(|| "garmin".to_string()),
        external_id: text(golden, "external_id").unwrap_or_default(),
        name: text(golden, "name").unwrap_or_else(|| "Run".to_string()),
        sport_type: text(golden, "sport_type").unwrap_or_else(|| "running".to_string()),
        description: text(golden, "description"),
        start_time,
        local_date: text(golden, "local_date").unwrap_or_default(),
        day_of_week: text(golden, "day_of_week"),
        distance_m: number(golden, "distance_m").unwrap_or(0.0),
        moving_time_s: number(golden, "moving_time_s").unwrap_or(0.0),
        elapsed_time_s: number(golden, "elapsed_time_s").unwrap_or(0.0),
        elevation_gain_m: number(golden, "elevation_gain_m").unwrap_or(0.0),
        avg_hr: number(golden, "avg_hr"),
        max_hr: number(golden, "max_hr"),
        calories: number(golden, "calories"),
        avg_cadence: number(golden, "avg_cadence"),
        max_cadence: number(golden, "max_cadence"),
        avg_speed_ms: number(golden, "avg_speed_ms"),
        max_speed_ms: number(golden, "max_speed_ms"),
        running_time_s: number(golden, "running_time_s"),
        walking_time_s: number(golden, "walking_time_s"),
        stopped_time_s: number(golden, "stopped_time_s"),
        running_distance_m: number(golden, "running_distance_m"),
        perceived_exertion: integer(golden, "perceived_exertion"),
        device: text(golden, "device"),
        gear: text(golden, "gear"),
        is_trail: flag(golden, "is_trail").unwrap_or(false),
        is_commute: flag(golden, "is_commute").unwrap_or(false),
        ..Activity::default()
    }
}

#[test]
fn the_golden_activities_survive_the_round_trip_through_the_store() {
    let dir = common::temp_dir("golden-activities");
    let store = Store::open_and_migrate(&dir.join("stride.db"), &common::key()).unwrap();

    let mut checked = 0usize;
    let mut names: Vec<String> = Vec::new();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(golden_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for path in entries {
        let golden: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let Some(activity) = golden
            .get("import")
            .and_then(|import| import.get("activity"))
        else {
            continue;
        };
        let case = golden
            .get("case_id")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string();
        names.push(case.clone());

        let row = activity_from_golden(activity);
        let id = store.upsert_activity(&row).unwrap();
        let read = store.activity(id).unwrap().unwrap();

        // The values are the golden's values.
        assert_eq!(read.source, text(activity, "source").unwrap(), "{case}");
        assert_eq!(
            read.external_id,
            text(activity, "external_id").unwrap(),
            "{case}"
        );
        assert_eq!(read.name, text(activity, "name").unwrap(), "{case}");
        assert_eq!(
            read.sport_type,
            text(activity, "sport_type").unwrap(),
            "{case}"
        );
        assert_eq!(read.description, text(activity, "description"), "{case}");
        assert_eq!(
            read.local_date,
            text(activity, "local_date").unwrap(),
            "{case}"
        );
        assert_eq!(read.day_of_week, text(activity, "day_of_week"), "{case}");
        assert_eq!(read.device, text(activity, "device"), "{case}");
        assert_eq!(read.gear, text(activity, "gear"), "{case}");
        assert_eq!(
            read.distance_m,
            number(activity, "distance_m").unwrap(),
            "{case}"
        );
        assert_eq!(
            read.moving_time_s,
            number(activity, "moving_time_s").unwrap(),
            "{case}"
        );
        assert_eq!(
            read.elapsed_time_s,
            number(activity, "elapsed_time_s").unwrap(),
            "{case}"
        );
        assert_eq!(
            read.elevation_gain_m,
            number(activity, "elevation_gain_m").unwrap(),
            "{case}"
        );
        assert_eq!(read.avg_hr, number(activity, "avg_hr"), "{case}");
        assert_eq!(read.max_hr, number(activity, "max_hr"), "{case}");
        assert_eq!(read.calories, number(activity, "calories"), "{case}");
        assert_eq!(read.avg_cadence, number(activity, "avg_cadence"), "{case}");
        assert_eq!(read.max_cadence, number(activity, "max_cadence"), "{case}");
        assert_eq!(
            read.avg_speed_ms,
            number(activity, "avg_speed_ms"),
            "{case}"
        );
        assert_eq!(
            read.max_speed_ms,
            number(activity, "max_speed_ms"),
            "{case}"
        );
        assert_eq!(
            read.running_time_s,
            number(activity, "running_time_s"),
            "{case}"
        );
        assert_eq!(
            read.walking_time_s,
            number(activity, "walking_time_s"),
            "{case}"
        );
        assert_eq!(
            read.stopped_time_s,
            number(activity, "stopped_time_s"),
            "{case}"
        );
        assert_eq!(
            read.running_distance_m,
            number(activity, "running_distance_m"),
            "{case}"
        );
        assert_eq!(
            read.perceived_exertion,
            integer(activity, "perceived_exertion"),
            "{case}"
        );
        assert_eq!(
            read.is_trail,
            flag(activity, "is_trail").unwrap_or(false),
            "{case}"
        );
        assert_eq!(
            read.is_commute,
            flag(activity, "is_commute").unwrap_or(false),
            "{case}"
        );

        // The timestamp comes back as the same text the golden file holds:
        // `2026-03-02T05:30:00+00:00`, with the microseconds only when there
        // are any.
        assert_eq!(
            datetimes::to_iso8601(read.start_time),
            activity["start_time"].as_str().unwrap(),
            "{case}"
        );

        // And the same instant.
        assert_eq!(read.start_time, row.start_time, "{case}");

        // The bytes on disk are the SQLAlchemy storage format:
        // `2026-03-02 05:30:00.000000`.
        let naive = read.start_time.naive_utc();
        assert_eq!(
            datetimes::format_datetime(read.start_time),
            naive.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
            "{case}"
        );

        checked += 1;
    }

    assert!(
        checked >= 10,
        "only {checked} golden activities were checked: {names:?}"
    );
    println!("{checked} golden activities went through the store: {names:?}");
}

#[test]
fn a_golden_timestamp_keeps_its_microseconds() {
    // The contract's text, for a time that has microseconds.
    let moment = Utc.with_ymd_and_hms(2026, 3, 2, 5, 30, 0).unwrap()
        + chrono::Duration::microseconds(123_456);
    assert_eq!(
        datetimes::to_iso8601(moment),
        "2026-03-02T05:30:00.123456+00:00"
    );
    assert_eq!(
        datetimes::format_datetime(moment),
        "2026-03-02 05:30:00.123456"
    );
}
