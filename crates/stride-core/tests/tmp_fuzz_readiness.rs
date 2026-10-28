use serde_json::Value;
use std::path::Path;
use stride_core::readiness::{baselines, readiness};

#[test]
fn fuzz() {
    let path = Path::new("/tmp/readiness_fuzz.json");
    let text = std::fs::read_to_string(path).unwrap();
    let cases: Value = serde_json::from_str(&text).unwrap();
    let mut failures = 0;
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let rows: Vec<stride_core::model::DailyPhysiology> = case["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| serde_json::from_value(r.clone()).unwrap())
            .collect();
        let day = case["day"].as_str().unwrap();
        let computed_baselines = baselines(&rows, day);
        let row = rows.iter().find(|r| r.date == day);
        let computed = serde_json::to_value(readiness(day, row, &computed_baselines)).unwrap();
        let expected = &case["expected"];

        let mut diffs = Vec::new();
        // baselines
        for (key, want) in case["expected_baselines"].as_object().unwrap() {
            let got = computed_baselines_field(&computed_baselines, key);
            let ok = match (got, want) {
                (Some(a), Value::Number(b)) => {
                    (a - b.as_f64().unwrap()).abs()
                        <= 1e-9 * f64::max(1.0, b.as_f64().unwrap().abs())
                }
                (None, Value::Null) => true,
                _ => false,
            };
            if !ok {
                diffs.push(format!("baseline.{key}: got {got:?}, want {want}"));
            }
        }
        // readiness
        for (key, want) in expected.as_object().unwrap() {
            let got = computed.get(key).unwrap_or(&Value::Null);
            if key == "drivers" {
                let wants = want.as_array().unwrap();
                let gots = got.as_array().unwrap();
                if wants.len() != gots.len() {
                    diffs.push(format!("drivers: {} vs {}", gots.len(), wants.len()));
                } else {
                    for (g, w) in gots.iter().zip(wants.iter()) {
                        for (dk, dv) in w.as_object().unwrap() {
                            let gv = g.get(dk).unwrap_or(&Value::Null);
                            if !numbers_match(gv, dv) {
                                diffs.push(format!("driver.{dk}: got {gv}, want {dv}"));
                            }
                        }
                    }
                }
            } else if !numbers_match(got, want) {
                diffs.push(format!("{key}: got {got}, want {want}"));
            }
        }
        if !diffs.is_empty() {
            failures += 1;
            if failures < 6 {
                println!("case {index}: {}", diffs.join(" | "));
                println!("  rows: {}", serde_json::to_string(&case["rows"]).unwrap());
            }
        }
    }
    println!("failures: {failures}");
    assert_eq!(failures, 0);
}

fn computed_baselines_field(b: &stride_core::readiness::Baselines, key: &str) -> Option<f64> {
    match key {
        "sleep_s" => b.sleep_s,
        "hrv_ms" => b.hrv_ms,
        "resting_hr" => b.resting_hr,
        _ => None,
    }
}

fn numbers_match(got: &Value, want: &Value) -> bool {
    match (got, want) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            (a - b).abs() <= 1e-9 * f64::max(1.0, b.abs())
        }
        _ => got == want,
    }
}
