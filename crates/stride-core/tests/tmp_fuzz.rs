use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use stride_core::fingerprint::{default_zone_model, FingerprintInput};

#[test]
fn fuzz() {
    let path = Path::new("/tmp/fingerprint_fuzz.json");
    let text = std::fs::read_to_string(path).unwrap();
    let cases: Value = serde_json::from_str(&text).unwrap();
    let mut failures = 0;
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let spec = &case["activity"];
        let mut activity = stride_core::model::Activity {
            id: 1,
            name: "fuzz".into(),
            start_date: String::new(),
            day_of_week: None,
            local_date: "2026-03-02".into(),
            sport_type: "running".into(),
            source: spec["source"].as_str().unwrap().to_string(),
            distance_m: spec["distance_m"].as_f64().unwrap(),
            moving_time_s: None,
            elapsed_time_s: None,
            elevation_gain_m: None,
            avg_hr: spec["avg_hr"].as_f64(),
            max_hr: spec["max_hr"].as_f64(),
            calories: None,
            running_time_s: spec["running_time_s"].as_f64(),
            walking_time_s: spec["walking_time_s"].as_f64(),
            stopped_time_s: spec["stopped_time_s"].as_f64(),
            laps: Vec::new(),
            zones: Vec::new(),
            streams: BTreeMap::new(),
            fingerprint: None,
        };
        for lap in spec["laps"].as_array().unwrap() {
            activity.laps.push(stride_core::model::Lap {
                lap_index: lap["lap_index"].as_i64().unwrap(),
                distance_m: lap["distance_m"].as_f64(),
                duration_s: lap["duration_s"].as_f64(),
                avg_speed_ms: lap["avg_speed_ms"].as_f64(),
                avg_hr: lap["avg_hr"].as_f64(),
                max_hr: lap["max_hr"].as_f64(),
                avg_cadence: lap["avg_cadence"].as_f64(),
                stride_length_m: lap["stride_length_m"].as_f64(),
                ..Default::default()
            });
        }
        for zone in spec["zones"].as_array().unwrap() {
            activity.zones.push(stride_core::model::ZoneTime {
                zone_no: zone["zone_no"].as_i64().unwrap(),
                time_s: zone["time_s"].as_f64().unwrap(),
            });
        }
        for stream in spec["streams"].as_array().unwrap() {
            let kind = stream["kind"].as_str().unwrap().to_string();
            let values = stream["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64())
                .collect();
            activity.streams.insert(
                kind,
                stride_core::model::Stream {
                    seconds: Vec::new(),
                    values,
                    unit: None,
                },
            );
        }
        let reference = case["reference"].as_f64().unwrap();
        let zones = default_zone_model(reference);
        let computed = serde_json::to_value(stride_core::fingerprint(&FingerprintInput {
            max_hr_reference: reference,
            zone_model: &zones,
            activity: &activity,
        }))
        .unwrap();
        let expected = &case["expected"];
        let mut diffs = Vec::new();
        for (key, want) in expected.as_object().unwrap() {
            let got = computed.get(key).unwrap_or(&Value::Null);
            let same = match (got, want) {
                (Value::Number(a), Value::Number(b)) => {
                    let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
                    (a - b).abs() <= 1e-9 * f64::max(1.0, b.abs())
                }
                (Value::Object(a), Value::Object(b)) => {
                    let mut ok = a.len() == b.len();
                    for (k, bv) in b {
                        match a.get(k) {
                            Some(Value::Number(av)) => {
                                let (av, bv) = (av.as_f64().unwrap(), bv.as_f64().unwrap());
                                if (av - bv).abs() > 1e-9 * f64::max(1.0, bv.abs()) {
                                    ok = false;
                                }
                            }
                            _ => ok = false,
                        }
                    }
                    ok
                }
                _ => got == want,
            };
            if !same {
                diffs.push(format!("{key}: got {got}, want {want}"));
            }
        }
        if !diffs.is_empty() {
            failures += 1;
            if failures < 6 {
                println!("case {index}: {}", diffs.join(" | "));
                println!(
                    "  input: {}",
                    serde_json::to_string(&case["activity"]).unwrap()
                );
            }
        }
    }
    println!("failures: {failures}");
    assert_eq!(failures, 0);
}
