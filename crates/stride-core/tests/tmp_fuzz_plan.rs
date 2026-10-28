use std::path::Path;
use serde_json::{json, Map, Value};
use stride_core::fingerprint::{default_zone_model, FingerprintInput};
use stride_core::plan::{adherence_for, match_sessions, PlanInputs};

#[test]
fn fuzz() {
    let path = Path::new("/tmp/plan_fuzz.json");
    let text = std::fs::read_to_string(path).unwrap();
    let cases: Value = serde_json::from_str(&text).unwrap();
    let mut failures = 0;
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let spec = &case["case"];
        let sessions: Vec<stride_core::model::PlanSession> = spec["sessions"].as_array().unwrap().iter()
            .map(|s| serde_json::from_value(s.clone()).unwrap()).collect();
        let mut activities: Vec<stride_core::model::Activity> = Vec::new();
        for a in spec["activities"].as_array().unwrap() {
            let mut activity = stride_core::model::Activity {
                id: a["id"].as_i64().unwrap(),
                name: a["name"].as_str().unwrap().to_string(),
                local_date: a["local_date"].as_str().unwrap().to_string(),
                start_date: a["local_date"].as_str().unwrap().to_string(),
                sport_type: a["sport_type"].as_str().unwrap().to_string(),
                source: "garmin".to_string(),
                distance_m: a["distance_m"].as_f64().unwrap(),
                moving_time_s: a["moving_time_s"].as_f64(),
                elapsed_time_s: a["moving_time_s"].as_f64(),
                elevation_gain_m: Some(0.0),
                ..Default::default()
            };
            for lap in a["laps"].as_array().unwrap() {
                activity.laps.push(stride_core::model::Lap {
                    lap_index: lap["lap_index"].as_i64().unwrap(),
                    distance_m: lap["distance_m"].as_f64(),
                    duration_s: lap["duration_s"].as_f64(),
                    ..Default::default()
                });
            }
            activities.push(activity);
        }
        let physiology: Vec<stride_core::model::DailyPhysiology> = spec["physiology"].as_array().unwrap().iter()
            .map(|r| serde_json::from_value(r.clone()).unwrap()).collect();

        // Fingerprints for the run activities, exactly as the reference does.
        let reference = 190.0f64;
        let zones = default_zone_model(reference);
        for activity in activities.iter_mut() {
            if stride_core::plan::is_run_sport(&activity.sport_type) {
                activity.fingerprint = Some(stride_core::fingerprint(&FingerprintInput {
                    max_hr_reference: reference,
                    zone_model: &zones,
                    activity,
                }));
            }
        }

        let from = spec["range"][0].as_str().unwrap();
        let to = spec["range"][1].as_str().unwrap();
        let inputs = PlanInputs { from, to, sessions: &sessions, activities: &activities, physiology: &physiology };
        let matches = match_sessions(&inputs);
        let expected_rows = case["expected"]["rows"].as_array().unwrap();
        let mut diffs = Vec::new();
        if matches.len() != expected_rows.len() {
            diffs.push(format!("rows: {} vs {}", matches.len(), expected_rows.len()));
        }
        for (entry, expected) in matches.iter().zip(expected_rows.iter()) {
            let mut row = Map::new();
            row.insert("session".to_string(), serde_json::to_value(&entry.session).unwrap());
            row.insert("activity".to_string(), match &entry.activity {
                Some(a) => serde_json::to_value(a).unwrap(),
                None => Value::Null,
            });
            row.insert("verdict".to_string(), Value::String(entry.verdict.verdict.clone()));
            row.insert("distance_hit".to_string(), Value::Bool(entry.verdict.distance_hit));
            row.insert("pace_band_hit".to_string(), match entry.verdict.pace_band_hit {
                Some(v) => Value::Bool(v), None => Value::Null });
            row.insert("distance_delta_m".to_string(), opt(entry.verdict.distance_delta_m));
            row.insert("pace_delta_s_km".to_string(), opt(entry.verdict.pace_delta_s_km));
            row.insert("adherence_score".to_string(), opt(entry.verdict.adherence_score));
            row.insert("cause".to_string(), match &entry.verdict.cause {
                Some(c) => Value::String(c.clone()), None => Value::Null });
            row.insert("cause_confidence".to_string(), opt(entry.verdict.cause_confidence));
            let actual = Value::Object(row);
            if !same(&actual, expected) {
                diffs.push(format!("session {}: got {} want {}", entry.session.id,
                    serde_json::to_string(&actual).unwrap(),
                    serde_json::to_string(expected).unwrap()));
            }
        }
        let summary = adherence_for(&inputs);
        let actual = serde_json::to_value(&summary).unwrap();
        if !same(&actual, &case["expected"]["adherence"]) {
            diffs.push(format!("adherence: got {} want {}",
                serde_json::to_string(&actual).unwrap(),
                serde_json::to_string(&case["expected"]["adherence"]).unwrap()));
        }
        if !diffs.is_empty() {
            failures += 1;
            if failures < 4 {
                for d in &diffs { println!("case {index}: {d}"); }
                println!("  input: {}", serde_json::to_string(spec).unwrap());
            }
        }
    }
    println!("failures: {failures}");
    assert_eq!(failures, 0);
}

fn opt(value: Option<f64>) -> Value {
    match value { Some(v) => json!(v), None => Value::Null }
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            (x - y).abs() <= 1e-9 * f64::max(1.0, y.abs())
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(x, y)| same(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).map_or(false, |w| same(v, w)))
        }
        _ => a == b,
    }
}
