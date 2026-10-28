//! The golden tests: the fixtures decide.
//!
//! Every number in `backend/tests/golden/golden/` was produced by the Python
//! reference implementation, and the comparison rules are the ones in
//! `docs/core-contract.md` §8: strings, booleans, nulls and integers are
//! exactly equal; floating point numbers differ by at most
//! `1e-9 × max(1, |golden value|)`.
//!
//! The fixtures live outside the crate, so the tests read them from the
//! repository at run time.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use stride_core::comparison::{compare, profile_for, PROFILE_POINTS};
use stride_core::fingerprint::{default_zone_model, zone_model, FingerprintInput, StoredZone};
use stride_core::model::{Activity, DailyPhysiology, Lap, PlanSession, Stream, Zone};
use stride_core::plan::{adherence_for, match_sessions, PlanInputs};
use stride_core::readiness::{baselines, readiness};

/// The golden fixtures.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../backend/tests/golden")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

/// Compare a computed value with a golden value, using the contract's rules.
fn assert_same(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => {
            let left = left.as_f64().unwrap_or(f64::NAN);
            let right = right.as_f64().unwrap_or(f64::NAN);
            let tolerance = 1e-9 * f64::max(1.0, right.abs());
            assert!(
                (left - right).abs() <= tolerance,
                "{path}: computed {left}, golden {right} (tolerance {tolerance})"
            );
        }
        (Value::Array(left), Value::Array(right)) => {
            assert_eq!(
                left.len(),
                right.len(),
                "{path}: computed {} items, golden {} items",
                left.len(),
                right.len()
            );
            for (index, (left, right)) in left.iter().zip(right.iter()).enumerate() {
                assert_same(left, right, &format!("{path}[{index}]"));
            }
        }
        (Value::Object(left), Value::Object(right)) => {
            let left_keys: Vec<&String> = left.keys().collect();
            let right_keys: Vec<&String> = right.keys().collect();
            assert_eq!(
                left_keys, right_keys,
                "{path}: computed and golden have different keys"
            );
            for (key, right) in right {
                let left = left
                    .get(key)
                    .unwrap_or_else(|| panic!("{path}: missing key {key}"));
                assert_same(left, right, &format!("{path}.{key}"));
            }
        }
        _ => assert_eq!(actual, expected, "{path}: computed and golden differ"),
    }
}

/// Build an activity from a golden file's `import` section.
fn activity_from_import(import: &Value, id: i64, name: Option<&str>) -> Activity {
    let scalars = &import["activity"];
    let mut activity = Activity {
        id,
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| scalars["name"].as_str().unwrap_or_default().to_string()),
        local_date: scalars["local_date"].as_str().unwrap_or_default().to_string(),
        start_date: scalars["start_time"]
            .as_str()
            .map(|value| value.get(..10).unwrap_or(value).to_string())
            .unwrap_or_default(),
        day_of_week: scalars["day_of_week"].as_str().map(str::to_string),
        sport_type: scalars["sport_type"].as_str().unwrap_or_default().to_string(),
        source: scalars["source"].as_str().unwrap_or_default().to_string(),
        distance_m: scalars["distance_m"].as_f64().unwrap_or(0.0),
        moving_time_s: scalars["moving_time_s"].as_f64(),
        elapsed_time_s: scalars["elapsed_time_s"].as_f64(),
        elevation_gain_m: scalars["elevation_gain_m"].as_f64(),
        avg_hr: scalars["avg_hr"].as_f64(),
        max_hr: scalars["max_hr"].as_f64(),
        calories: scalars["calories"].as_f64(),
        running_time_s: scalars["running_time_s"].as_f64(),
        walking_time_s: scalars["walking_time_s"].as_f64(),
        stopped_time_s: scalars["stopped_time_s"].as_f64(),
        laps: Vec::new(),
        zones: Vec::new(),
        streams: BTreeMap::new(),
        fingerprint: None,
    };
    if let Some(laps) = import["laps"].as_array() {
        activity.laps = laps
            .iter()
            .map(|lap| serde_json::from_value::<Lap>(lap.clone()).expect("a lap"))
            .collect();
    }
    if let Some(zones) = import["hr_zones"].as_array() {
        activity.zones = zones
            .iter()
            .map(|zone| stride_core::model::ZoneTime {
                zone_no: zone["zone_no"].as_i64().unwrap_or_default(),
                time_s: zone["time_s"].as_f64().unwrap_or_default(),
            })
            .collect();
    }
    if let Some(streams) = import["streams"].as_object() {
        for (kind, stream) in streams {
            activity.streams.insert(
                kind.clone(),
                serde_json::from_value::<Stream>(stream.clone()).expect("a stream"),
            );
        }
    }
    activity
}

/// The zone model the golden file records.
fn zone_model_from_golden(inputs: &Value) -> Vec<Zone> {
    inputs["zone_model"]
        .as_array()
        .expect("a zone model")
        .iter()
        .map(|zone| Zone {
            zone_no: zone["zone_no"].as_i64().unwrap_or_default(),
            name: zone["name"].as_str().unwrap_or_default().to_string(),
            low_bpm: zone["low_bpm"].as_f64().unwrap_or_default(),
            high_bpm: zone["high_bpm"].as_f64(),
        })
        .collect()
}

/// Every FIT case, and its golden file.
fn cases() -> Vec<(String, Value)> {
    let manifest = read_json(&golden_dir().join("cases.json"));
    let mut names: Vec<String> = manifest["cases"]
        .as_object()
        .expect("the manifest")
        .keys()
        .cloned()
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let golden = read_json(&golden_dir().join("golden").join(format!("{name}.json")));
            (name, golden)
        })
        .collect()
}

// ---------------------------------------------------------------- fingerprint

#[test]
fn fingerprints_match_the_golden_files() {
    for (name, golden) in cases() {
        let import = &golden["import"];
        let inputs = &golden["fingerprint"]["inputs"];
        let activity = activity_from_import(import, 1, None);
        let zone_model = zone_model_from_golden(inputs);
        let reference = inputs["max_hr_reference"].as_f64().unwrap_or_default();

        let computed = stride_core::fingerprint(&FingerprintInput {
            max_hr_reference: reference,
            zone_model: &zone_model,
            activity: &activity,
        });

        // The golden file's key names must be the ones the crate serialises.
        let expected: stride_core::model::Fingerprint =
            serde_json::from_value(golden["fingerprint"]["fingerprint"].clone())
                .expect("a fingerprint");
        let actual = serde_json::to_value(&computed).expect("a fingerprint");
        assert_same(
            &actual,
            &serde_json::to_value(&expected).expect("a fingerprint"),
            &format!("{name}.fingerprint"),
        );

        // The recorded inputs are the ones the function was given.
        assert_eq!(
            inputs["lap_count"].as_i64().unwrap_or_default(),
            activity.laps.len() as i64,
            "{name}: lap count"
        );
        assert_eq!(
            inputs["activity_source"].as_str().unwrap_or_default(),
            activity.source,
            "{name}: activity source"
        );
    }
}

#[test]
fn the_zone_model_is_derived_from_the_maximum_heart_rate() {
    // The fallback model: reference × {0, .6, .7, .8, .9}.
    let zones = default_zone_model(155.0);
    let bounds: Vec<f64> = zones.iter().map(|zone| zone.low_bpm).collect();
    assert_eq!(bounds, vec![0.0, 93.0, 108.5, 124.0, 139.5]);
    assert_eq!(zones[4].high_bpm, None);
    assert_eq!(zones[0].name, "Recovery");
}

#[test]
fn five_stored_zones_replace_the_derived_model() {
    let stored: Vec<StoredZone> = (0..5)
        .map(|index| StoredZone {
            zone_no: Some(index as i64 + 1),
            name: Some(format!("Zone {}", index + 1)),
            low_bpm: Some(50.0 * index as f64),
            high_bpm: Some(50.0 * index as f64 + 50.0),
        })
        .collect();
    let zones = zone_model(190.0, Some(&stored));
    assert_eq!(zones.len(), 5);
    assert_eq!(zones[0].low_bpm, 0.0);
    assert_eq!(zones[4].high_bpm, Some(250.0));

    // Fewer than five usable zones fall back to the derived model.
    let zones = zone_model(190.0, Some(&stored[..4]));
    assert_eq!(zones[0].low_bpm, 0.0);
    assert_eq!(zones[1].low_bpm, 114.0);
}

// ----------------------------------------------------------------- readiness

#[test]
fn readiness_matches_the_golden_file() {
    let scenarios = read_json(&golden_dir().join("scenarios/readiness.json"));
    let golden = read_json(&golden_dir().join("golden/readiness.json"));

    let scenarios = scenarios["scenarios"].as_array().expect("the scenarios");
    let expected = golden["scenarios"].as_array().expect("the scenarios");
    assert_eq!(scenarios.len(), expected.len());

    for (scenario, expected) in scenarios.iter().zip(expected.iter()) {
        let day = scenario["day"].as_str().expect("a day");
        let rows: Vec<DailyPhysiology> = scenario["physiology"]
            .as_array()
            .expect("the physiology")
            .iter()
            .map(|row| serde_json::from_value::<DailyPhysiology>(row.clone()).expect("a row"))
            .collect();

        let computed_baselines = baselines(&rows, day);
        let actual = serde_json::to_value(&computed_baselines).expect("the baselines");
        assert_same(
            &actual,
            &expected["baselines"],
            &format!("{}.baselines", scenario["id"].as_str().unwrap_or("?")),
        );

        let row = rows.iter().find(|row| row.date == day);
        let computed = readiness(day, row, &computed_baselines);
        let actual = serde_json::to_value(&computed).expect("the readiness");
        assert_same(
            &actual,
            &expected["readiness"],
            &format!("{}.readiness", scenario["id"].as_str().unwrap_or("?")),
        );
    }
}

// --------------------------------------------------------------------- plan

/// The plan scenario's activities, in insertion order.
fn plan_activities(scenario: &Value, imports: &BTreeMap<String, Value>) -> Vec<Activity> {
    let mut activities = Vec::new();
    for (index, spec) in scenario["activities"].as_array().expect("the activities").iter().enumerate()
    {
        let id = index as i64 + 1;
        let mut activity = match spec.get("case_id").and_then(|value| value.as_str()) {
            Some(case_id) => {
                let import = imports
                    .get(case_id)
                    .unwrap_or_else(|| panic!("the {case_id} import"));
                activity_from_import(import, id, spec["name"].as_str())
            }
            None => {
                let scalars = &spec["activity"];
                Activity {
                    id,
                    name: scalars["name"].as_str().unwrap_or_default().to_string(),
                    local_date: scalars["local_date"].as_str().unwrap_or_default().to_string(),
                    start_date: scalars["start_time"]
                        .as_str()
                        .map(|value| value.get(..10).unwrap_or(value).to_string())
                        .unwrap_or_default(),
                    day_of_week: scalars["day_of_week"].as_str().map(str::to_string),
                    sport_type: scalars["sport_type"].as_str().unwrap_or_default().to_string(),
                    source: scalars["source"].as_str().unwrap_or_default().to_string(),
                    distance_m: scalars["distance_m"].as_f64().unwrap_or_default(),
                    moving_time_s: scalars["moving_time_s"].as_f64(),
                    elapsed_time_s: scalars["elapsed_time_s"].as_f64(),
                    elevation_gain_m: scalars["elevation_gain_m"].as_f64(),
                    avg_hr: scalars["avg_hr"].as_f64(),
                    max_hr: scalars["max_hr"].as_f64(),
                    ..Activity::default()
                }
            }
        };
        if let Some(sport) = spec.get("sport_type").and_then(|value| value.as_str()) {
            activity.sport_type = sport.to_string();
        }
        if let Some(date) = spec.get("date").and_then(|value| value.as_str()) {
            activity.local_date = date.to_string();
        }
        if let Some(start) = spec.get("start_time").and_then(|value| value.as_str()) {
            activity.start_date = start.get(..10).unwrap_or(start).to_string();
        }
        activities.push(activity);
    }
    activities
}

#[test]
fn the_plan_matches_the_golden_file() {
    let scenario = read_json(&golden_dir().join("scenarios/plan.json"));
    let golden = read_json(&golden_dir().join("golden/plan.json"));

    let mut imports = BTreeMap::new();
    for (name, case) in cases() {
        imports.insert(name, case["import"].clone());
    }

    let sessions: Vec<PlanSession> = scenario["sessions"]
        .as_array()
        .expect("the sessions")
        .iter()
        .enumerate()
        .map(|(index, session)| PlanSession {
            id: index as i64 + 1,
            date: session["date"].as_str().unwrap_or_default().to_string(),
            session_type: session["session_type"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            title: session["title"].as_str().map(str::to_string),
            details: session["details"].as_str().map(str::to_string),
            target_distance_m: session["target_distance_m"].as_f64(),
            pace_low_s_km: session["pace_low_s_km"].as_f64(),
            pace_high_s_km: session["pace_high_s_km"].as_f64(),
        })
        .collect();

    let physiology: Vec<DailyPhysiology> = scenario["physiology"]
        .as_array()
        .expect("the physiology")
        .iter()
        .map(|row| serde_json::from_value::<DailyPhysiology>(row.clone()).expect("a row"))
        .collect();

    let mut activities = plan_activities(&scenario, &imports);

    // The reference fingerprints every run activity before matching, so the
    // plan engine can compare this run's cadence variability with the
    // athlete's baseline.
    let reference = activities
        .iter()
        .filter_map(|activity| activity.max_hr)
        .filter(|value| *value > 0.0)
        .fold(None, |current: Option<f64>, value| {
            Some(match current {
                Some(existing) if existing >= value => existing,
                _ => value,
            })
        })
        .unwrap_or(190.0);
    let zone_model = default_zone_model(reference);
    for activity in activities.iter_mut() {
        if stride_core::plan::is_run_sport(&activity.sport_type) {
            activity.fingerprint = Some(stride_core::fingerprint(&FingerprintInput {
                max_hr_reference: reference,
                zone_model: &zone_model,
                activity,
            }));
        }
    }

    let from = scenario["range"][0].as_str().unwrap_or_default();
    let to = scenario["range"][1].as_str().unwrap_or_default();
    let inputs = PlanInputs {
        from,
        to,
        sessions: &sessions,
        activities: &activities,
        physiology: &physiology,
    };

    let matches = match_sessions(&inputs);
    let expected_rows = golden["sessions"].as_array().expect("the sessions");
    assert_eq!(matches.len(), expected_rows.len());

    for (entry, expected) in matches.iter().zip(expected_rows.iter()) {
        let mut row = Map::new();
        row.insert(
            "session".to_string(),
            serde_json::to_value(&entry.session).expect("the session"),
        );
        row.insert(
            "activity".to_string(),
            match &entry.activity {
                Some(activity) => serde_json::to_value(activity).expect("the activity"),
                None => Value::Null,
            },
        );
        row.insert(
            "verdict".to_string(),
            Value::String(entry.verdict.verdict.clone()),
        );
        row.insert(
            "distance_hit".to_string(),
            Value::Bool(entry.verdict.distance_hit),
        );
        row.insert(
            "pace_band_hit".to_string(),
            match entry.verdict.pace_band_hit {
                Some(value) => Value::Bool(value),
                None => Value::Null,
            },
        );
        row.insert(
            "distance_delta_m".to_string(),
            match entry.verdict.distance_delta_m {
                Some(value) => json!(value),
                None => Value::Null,
            },
        );
        row.insert(
            "pace_delta_s_km".to_string(),
            match entry.verdict.pace_delta_s_km {
                Some(value) => json!(value),
                None => Value::Null,
            },
        );
        row.insert(
            "adherence_score".to_string(),
            match entry.verdict.adherence_score {
                Some(value) => json!(value),
                None => Value::Null,
            },
        );
        row.insert(
            "cause".to_string(),
            match &entry.verdict.cause {
                Some(cause) => Value::String(cause.clone()),
                None => Value::Null,
            },
        );
        row.insert(
            "cause_confidence".to_string(),
            match entry.verdict.cause_confidence {
                Some(value) => json!(value),
                None => Value::Null,
            },
        );
        assert_same(
            &Value::Object(row),
            expected,
            &format!("plan.sessions[{}]", entry.session.id),
        );
    }

    let summary = adherence_for(&inputs);
    let actual = serde_json::to_value(&summary).expect("the adherence");
    assert_same(&actual, &golden["adherence"], "plan.adherence");
}

// --------------------------------------------------------------- comparison

#[test]
fn the_comparison_has_fifteen_parameters_and_a_shared_axis() {
    let mut imports = BTreeMap::new();
    for (name, case) in cases() {
        imports.insert(name, case["import"].clone());
    }
    let a = activity_from_import(imports.get("mixed_run").expect("mixed_run"), 1, None);
    let b = activity_from_import(imports.get("hr_gps_run").expect("hr_gps_run"), 2, None);

    let comparison = compare(&a, &b);
    assert_eq!(comparison.parameters.len(), 15);
    assert_eq!(comparison.profile_a.len(), PROFILE_POINTS);
    assert_eq!(comparison.profile_b.len(), PROFILE_POINTS);

    // The profile is a shared distance axis: the first point is the run's
    // first distance and the last is its last.
    let first = comparison.profile_a.first().expect("a profile");
    let last = comparison.profile_a.last().expect("a profile");
    assert_eq!(first.pct, 0.0);
    assert_eq!(last.pct, 100.0);
    assert!(first.distance_m < last.distance_m);

    // The pace parameter is the better one per parameter: the slower run
    // loses it.
    let pace = comparison
        .parameters
        .iter()
        .find(|entry| entry.parameter.key == "pace_s_per_km")
        .expect("the pace parameter");
    assert_same(
        &json!(pace.a),
        &json!(338.6004514673),
        "compare.pace.a",
    );
    assert_same(
        &json!(pace.b),
        &json!(303.0303030303),
        "compare.pace.b",
    );
    assert_eq!(
        pace.winner,
        Some(stride_core::comparison::Winner::B),
        "the faster run wins the pace"
    );

    // The distance parameter is neutral: there is no winner.
    let distance = comparison
        .parameters
        .iter()
        .find(|entry| entry.parameter.key == "distance_m")
        .expect("the distance parameter");
    assert_eq!(distance.winner, None);
}

#[test]
fn a_profile_without_a_distance_stream_is_empty() {
    let activity = Activity::default();
    assert!(profile_for(&activity, PROFILE_POINTS).is_empty());
}
