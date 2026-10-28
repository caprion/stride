//! Shared helpers for the crate's tests.
//!
//! The golden test and the reference-parity test build the same JSON from a
//! parsed activity and compare it the same way, so the two cannot drift.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use stride_fit::{parse_fit, ParsedActivity};

/// The repository root, two levels above this crate.
pub fn repo_root() -> PathBuf {
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

/// The `import` section of a golden file, rendered from the parsed activity.
pub fn import_section(activity: &ParsedActivity) -> Value {
    let activity_scalars = json!({
        "external_id": activity.external_id,
        "source": activity.source,
        "name": activity.name,
        "sport_type": activity.sport_type,
        "description": Value::Null,
        "start_time": activity.start_time.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        "local_date": activity.local_date,
        "day_of_week": activity.day_of_week,
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
        "perceived_exertion": activity.perceived_exertion,
        "device": activity.device,
        "gear": activity.gear,
        "is_trail": activity.is_trail,
        "is_commute": activity.is_commute,
        "running_time_s": activity.running_time_s,
        "walking_time_s": activity.walking_time_s,
        "stopped_time_s": activity.stopped_time_s,
        "running_distance_m": activity.running_distance_m,
    });

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

    let totals = json!({
        "running_time_s": activity.running_time_s,
        "walking_time_s": activity.walking_time_s,
        "stopped_time_s": activity.stopped_time_s,
        "running_distance_m": activity.running_distance_m,
    });

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

    let mut streams = serde_json::Map::new();
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

    let geo = activity.geo_points.clone().unwrap_or_default();
    let first: Vec<Value> = geo
        .iter()
        .take(3)
        .map(|point| json!([point[0], point[1]]))
        .collect();
    let last: Vec<Value> = geo
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|point| json!([point[0], point[1]]))
        .collect();

    json!({
        "activity": activity_scalars,
        "segments": segments,
        "totals": totals,
        "samples": samples,
        "streams": Value::Object(streams),
        "laps": laps,
        "lap_bounds": activity.lap_bounds,
        "hr_zones": [],
        "geo_points": {
            "count": geo.len(),
            "first": first,
            "last": last,
        },
    })
}

/// Compare two JSON trees with the contract's rules.
pub fn compare(path: &str, golden: &Value, actual: &Value) {
    match (golden, actual) {
        (Value::Null, Value::Null) => {}
        (Value::Bool(expected), Value::Bool(found)) => {
            assert_eq!(expected, found, "{path}: golden {expected}, got {found}");
        }
        (Value::String(expected), Value::String(found)) => {
            assert_eq!(
                expected, found,
                "{path}: golden {expected:?}, got {found:?}"
            );
        }
        (Value::Number(expected), Value::Number(found)) => {
            let expected = expected.as_f64().expect("a number");
            let found = found.as_f64().expect("a number");
            let tolerance = 1e-9 * expected.abs().max(1.0);
            assert!(
                (expected - found).abs() <= tolerance,
                "{path}: golden {expected}, got {found} (tolerance {tolerance})"
            );
        }
        (Value::Array(expected), Value::Array(found)) => {
            assert_eq!(expected.len(), found.len(), "{path}: length differs");
            for (index, (expected, found)) in expected.iter().zip(found.iter()).enumerate() {
                compare(&format!("{path}[{index}]"), expected, found);
            }
        }
        (Value::Object(expected), Value::Object(found)) => {
            assert_eq!(expected.len(), found.len(), "{path}: key count differs");
            for (key, expected) in expected {
                let found = found
                    .get(key)
                    .unwrap_or_else(|| panic!("{path}: the golden file has no key {key:?}"));
                compare(&format!("{path}.{key}"), expected, found);
            }
        }
        (expected, found) => {
            panic!("{path}: type differs: golden {expected}, got {found}")
        }
    }
}

/// The fixture of a case.
pub fn fixture(case: &str) -> PathBuf {
    repo_root()
        .join("backend/tests/golden/fixtures")
        .join(format!("{case}.fit"))
}

/// The golden output of a case.
pub fn golden(case: &str) -> Value {
    let path = repo_root()
        .join("backend/tests/golden/golden")
        .join(format!("{case}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

/// Parse one fixture.
pub fn parse(case: &str) -> ParsedActivity {
    let path = fixture(case);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    parse_fit(&bytes, &format!("{case}.fit"))
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}
