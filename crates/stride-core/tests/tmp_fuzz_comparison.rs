use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use stride_core::comparison::profile_for;

#[test]
fn fuzz() {
    let path = Path::new("/tmp/comparison_fuzz.json");
    let text = std::fs::read_to_string(path).unwrap();
    let cases: Value = serde_json::from_str(&text).unwrap();
    let mut failures = 0;
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let mut streams = BTreeMap::new();
        for stream in case["streams"].as_array().unwrap() {
            let kind = stream["kind"].as_str().unwrap().to_string();
            let seconds = stream["seconds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let values = stream["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64())
                .collect();
            streams.insert(
                kind,
                stride_core::model::Stream {
                    seconds,
                    values,
                    unit: None,
                },
            );
        }
        let fingerprint =
            serde_json::from_value::<stride_core::model::Fingerprint>(case["fingerprint"].clone())
                .unwrap();
        let activity = stride_core::model::Activity {
            streams,
            distance_m: case["distance_m"].as_f64().unwrap(),
            moving_time_s: case["moving_time_s"].as_f64(),
            elevation_gain_m: case["elevation_gain_m"].as_f64(),
            avg_hr: case["avg_hr"].as_f64(),
            fingerprint: Some(fingerprint),
            ..Default::default()
        };
        for parameter in stride_core::comparison::PARAMETERS.iter() {
            let got =
                serde_json::to_value(stride_core::comparison::value_for(parameter, &activity))
                    .unwrap();
            let want = &case["expected_values"][parameter.key];
            if !same(&got, want) {
                failures += 1;
                if failures < 4 {
                    println!(
                        "case {index} parameter {}: got {got} want {want}",
                        parameter.key
                    );
                }
            }
        }
        let computed = serde_json::to_value(profile_for(&activity, 120)).unwrap();
        let expected = &case["expected"];
        if !same(&computed, expected) {
            failures += 1;
            if failures < 4 {
                println!("case {index}");
                for (i, (got, want)) in computed
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(expected.as_array().unwrap().iter())
                    .enumerate()
                {
                    if !same(got, want) {
                        println!("  point {i}: got {got} want {want}");
                        break;
                    }
                }
            }
        }
    }
    println!("failures: {failures}");
    assert_eq!(failures, 0);
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
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).map_or(false, |w| same(v, w)))
        }
        _ => a == b,
    }
}
