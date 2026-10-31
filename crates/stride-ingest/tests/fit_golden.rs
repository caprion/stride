//! The frozen FIT corpus is the decoder's oracle.
//!
//! Every file in `fixtures/fit` is decoded and compared with the matching
//! `fixtures/golden/golden` output. The comparison follows the frozen
//! contract: strings, booleans, nulls and integers are exact; floats use the
//! contract's relative tolerance.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};
use stride_ingest::{decode_fit, FitActivity, SegmentKind};

const CASES: [&str; 15] = [
    "cycling",
    "gaps",
    "hilly",
    "hr_gps_run",
    "low_resolution",
    "mixed_run",
    "no_cadence",
    "real_garmin_gap",
    "real_garmin_nulls",
    "real_garmin_run",
    "short_effort",
    "treadmill",
    "vendor_coros_export",
    "vendor_strava_export",
    "vendor_wahoo_export",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
        })
}

fn parse_case(case: &str) -> FitActivity {
    let path = repo_root()
        .join("fixtures")
        .join("fit")
        .join(format!("{case}.fit"));
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    decode_fit(&bytes, &format!("{case}.fit"))
        .unwrap_or_else(|error| panic!("cannot decode {}: {error}", path.display()))
}

fn golden_case(case: &str) -> Value {
    let path = repo_root()
        .join("fixtures")
        .join("golden")
        .join("golden")
        .join(format!("{case}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

fn import_json(activity: &FitActivity) -> Value {
    let start_time = activity
        .start_time
        .map(|value| value.format("%Y-%m-%dT%H:%M:%S%:z").to_string());
    let local_date = activity.local_date.clone();
    let day_of_week = activity.day_of_week.clone();
    let device = activity.device.clone();

    let segments: Vec<Value> = activity
        .segments
        .iter()
        .map(|segment| {
            json!({
                "kind": segment.kind.as_str(),
                "start_s": segment.start_s,
                "end_s": segment.end_s,
                "samples": segment.samples,
                "duration_s": segment.duration_s,
            })
        })
        .collect();
    let samples: Vec<Value> = activity
        .samples
        .iter()
        .map(|sample| {
            json!({
                "index": sample.index,
                "elapsed_s": sample.elapsed_s,
                "kind": sample.kind.as_str(),
                "distance_m": sample.distance_m,
                "speed_ms": sample.speed_ms,
                "stored_cadence": sample.stored_cadence,
                "cadence_spm": sample.cadence_spm,
                "heart_rate_bpm": sample.heart_rate_bpm,
                "altitude_m": sample.altitude_m,
            })
        })
        .collect();
    let mut streams = Map::new();
    for (name, stream) in &activity.streams {
        streams.insert(
            name.clone(),
            json!({
                "seconds": stream.seconds,
                "values": stream.values,
                "unit": stream.unit,
            }),
        );
    }
    let laps: Vec<Value> = activity
        .laps
        .iter()
        .map(|lap| {
            json!({
                "lap_index": lap.lap_index,
                "distance_m": lap.distance_m,
                "duration_s": lap.duration_s,
                "avg_speed_ms": lap.avg_speed_ms,
                "max_speed_ms": lap.max_speed_ms,
                "avg_hr": lap.avg_hr,
                "max_hr": lap.max_hr,
                "avg_cadence": lap.avg_cadence,
                "max_cadence": lap.max_cadence,
                "stride_length_m": lap.stride_length_m,
                "elevation_gain_m": lap.elevation_gain_m,
                "elevation_loss_m": lap.elevation_loss_m,
                "start_elevation_m": lap.start_elevation_m,
                "end_elevation_m": lap.end_elevation_m,
                "elapsed_s": lap.elapsed_s,
                "running_distance_m": lap.running_distance_m,
                "pace_s_per_km": lap.pace_s_per_km,
            })
        })
        .collect();

    json!({
        "activity": {
            "external_id": activity.external_id,
            "source": "garmin_fit",
            "name": activity.name,
            "sport_type": activity.sport_type,
            "description": Value::Null,
            "start_time": start_time,
            "local_date": local_date,
            "day_of_week": day_of_week,
            "distance_m": activity.distance_m,
            "moving_time_s": activity.moving_time_s,
            "elapsed_time_s": activity.elapsed_time_s,
            "elevation_gain_m": activity.elevation_gain_m,
            "avg_hr": activity.avg_hr,
            "max_hr": activity.max_hr,
            "calories": activity.calories,
            "avg_cadence": activity.avg_cadence,
            "max_cadence": activity.max_cadence,
            "avg_speed_ms": activity.avg_speed_ms,
            "max_speed_ms": activity.max_speed_ms,
            "perceived_exertion": Value::Null,
            "device": device,
            "gear": Value::Null,
            "is_trail": false,
            "is_commute": false,
            "running_time_s": activity.running_time_s,
            "walking_time_s": activity.walking_time_s,
            "stopped_time_s": activity.stopped_time_s,
            "running_distance_m": activity.running_distance_m,
        },
        "segments": segments,
        "totals": {
            "running_time_s": activity.running_time_s,
            "walking_time_s": activity.walking_time_s,
            "stopped_time_s": activity.stopped_time_s,
            "running_distance_m": activity.running_distance_m,
        },
        "samples": samples,
        "streams": Value::Object(streams),
        "laps": laps,
        "lap_bounds": activity.lap_bounds,
        "hr_zones": [],
        "geo_points": {
            "count": activity.geo_points.count,
            "first": activity.geo_points.first,
            "last": activity.geo_points.last,
        },
    })
}

fn compare(path: &str, expected: &Value, actual: &Value) {
    match (expected, actual) {
        (Value::Null, Value::Null) => {}
        (Value::Bool(expected), Value::Bool(actual)) => {
            assert_eq!(expected, actual, "{path}: boolean differs");
        }
        (Value::String(expected), Value::String(actual)) => {
            assert_eq!(expected, actual, "{path}: string differs");
        }
        (Value::Number(expected), Value::Number(actual)) => {
            let expected = expected.as_f64().expect("golden number");
            let actual = actual.as_f64().expect("actual number");
            let tolerance = 1e-9 * expected.abs().max(1.0);
            assert!(
                (expected - actual).abs() <= tolerance,
                "{path}: golden {expected}, got {actual} (tolerance {tolerance})"
            );
        }
        (Value::Array(expected), Value::Array(actual)) => {
            assert_eq!(expected.len(), actual.len(), "{path}: length differs");
            for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                compare(&format!("{path}[{index}]"), expected, actual);
            }
        }
        (Value::Object(expected), Value::Object(actual)) => {
            assert_eq!(expected.len(), actual.len(), "{path}: key count differs");
            for (key, expected) in expected {
                let actual = actual
                    .get(key)
                    .unwrap_or_else(|| panic!("{path}: actual has no key {key:?}"));
                compare(&format!("{path}.{key}"), expected, actual);
            }
        }
        (expected, actual) => {
            panic!("{path}: type differs: golden {expected}, got {actual}");
        }
    }
}

#[test]
fn every_frozen_fit_matches_the_golden_output() {
    for case in CASES {
        let activity = parse_case(case);
        let golden = golden_case(case);
        let mut expected = golden
            .get("import")
            .unwrap_or_else(|| panic!("{case}: golden has no import section"))
            .clone();
        if case == "real_garmin_nulls" {
            // The frozen reference spells a null uint32z serial as
            // "serial_number: None". That string is a reference bug, not a
            // value: the current contract says a sentinel is absent. The
            // dedicated test below asserts the corrected behaviour; every
            // other reading in this file is compared exactly.
            expected
                .get_mut("activity")
                .and_then(Value::as_object_mut)
                .expect("golden activity")
                .insert("external_id".to_owned(), json!(activity.external_id));
        }
        compare(
            &format!("{case}.import"),
            &expected,
            &import_json(&activity),
        );
    }
}

#[test]
fn the_headline_segmentation_numbers_are_explicit() {
    let mixed = parse_case("mixed_run");
    assert_eq!(mixed.running_time_s, Some(139.0));
    assert_eq!(mixed.walking_time_s, Some(10.0));
    assert_eq!(mixed.stopped_time_s, Some(20.0));
    assert_eq!(mixed.running_distance_m, Some(426.8));
    assert_eq!(mixed.distance_m, 443.0);
    assert_eq!(mixed.segments.len(), 4);
    assert_eq!(mixed.segments[0].kind, SegmentKind::Walk);
    assert_eq!(mixed.segments[1].kind, SegmentKind::Run);
    assert_eq!(mixed.segments[2].kind, SegmentKind::Stand);
    assert_eq!(mixed.segments[3].kind, SegmentKind::Run);

    let cycling = parse_case("cycling");
    assert_eq!(cycling.avg_cadence, Some(90.0));
    assert_eq!(cycling.laps[0].avg_cadence, Some(90.0));

    let no_cadence = parse_case("no_cadence");
    assert_eq!(no_cadence.avg_cadence, None);
    assert_eq!(no_cadence.max_cadence, None);
    assert!(no_cadence.streams["cadence"].values.is_empty());

    let low = parse_case("low_resolution");
    assert_eq!(low.segments[0].samples, 41);
    assert_eq!(low.segments[0].duration_s, 200.0);
}

#[test]
fn vendor_files_name_only_the_watch_they_contain() {
    let coros = parse_case("vendor_coros_export");
    let coros_expected = golden_case("vendor_coros_export");
    assert_eq!(
        coros.device.as_deref(),
        coros_expected["import"]["activity"]["device"].as_str()
    );

    let strava = parse_case("vendor_strava_export");
    assert_eq!(strava.device, None);

    let wahoo = parse_case("vendor_wahoo_export");
    let wahoo_expected = golden_case("vendor_wahoo_export");
    assert_eq!(
        wahoo.device.as_deref(),
        wahoo_expected["import"]["activity"]["device"].as_str()
    );
}

#[test]
fn a_null_serial_number_is_absent_not_zero() {
    let activity = parse_case("real_garmin_nulls");
    assert_eq!(activity.serial_number, None);
    assert_eq!(activity.external_id, "real_garmin_nulls.fit");
}
