//! The shapes the app consumes, checked against the app's own declarations.
//!
//! The response shapes are the contract between this crate and the app, and
//! the app writes its side of it down in `frontend/src/lib/types.ts`. This
//! test reads that file — the declarations themselves, not a copy of them —
//! and holds every answer to them.
//!
//! So the two cannot drift: a field renamed in the app fails this test
//! here, and a field added to an answer without the app knowing fails it
//! too.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::HashMap;

use serde_json::Value;

/// The app's declarations.
fn declarations() -> HashMap<String, Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("frontend")
        .join("src")
        .join("lib")
        .join("types.ts");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => panic!("the app's types could not be read: {error}"),
    };

    let mut declared: HashMap<String, (Option<String>, Vec<String>)> = HashMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("export interface ") {
            let (name, extends) = match rest.split_once('{') {
                Some((head, _)) => match head.split_once(" extends ") {
                    Some((name, parent)) => {
                        (name.trim().to_string(), Some(parent.trim().to_string()))
                    }
                    None => (head.trim().to_string(), None),
                },
                None => continue,
            };
            declared.insert(name.clone(), (extends, Vec::new()));
            current = Some(name);
            continue;
        }
        if trimmed.starts_with('}') {
            current = None;
            continue;
        }
        let name = match &current {
            Some(name) => name.clone(),
            None => continue,
        };
        // The fields of an interface are written at two spaces; the inline
        // object types below them are at four.
        let indent = line.len() - line.trim_start().len();
        if indent != 2 {
            continue;
        }
        let field = match trimmed.split(':').next() {
            Some(field) => field.trim().trim_end_matches('?').to_string(),
            None => continue,
        };
        if field.is_empty() || !field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        if let Some((_, fields)) = declared.get_mut(&name) {
            fields.push(field);
        }
    }

    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for (name, (extends, fields)) in declared.iter() {
        let mut all = fields.clone();
        if let Some(parent) = extends {
            if let Some((_, parent_fields)) = declared.get(parent) {
                all.extend(parent_fields.iter().cloned());
            }
        }
        all.sort();
        all.dedup();
        out.insert(name.clone(), all);
    }
    out
}

/// The fields the app declares and the API deliberately does not carry.
///
/// **`format`** is the one: a hint that tells the app *how* to render a
/// value is the hole the contract closes (core-contract §8). The app's
/// declaration of it goes away when the app takes the rendered value.
const DROPPED_BY_THE_CONTRACT: &[&str] = &["format"];

/// The unit suffixes the app's raw numbers carry.
///
/// The app names a raw number for its unit — `distance_m`, `moving_time_s`,
/// `pace_s_per_km` — and a rendered value has no unit in its name: the
/// unit is in the text the core wrote. The suffix is therefore dropped on
/// both sides before the two are compared, so the test compares the shape
/// and not the convention.
const UNIT_SUFFIXES: &[&str] = &["_s_per_km", "_ms", "_km", "_m", "_s"];

/// A field's name, without its unit suffix.
fn bare(name: &str) -> String {
    for suffix in UNIT_SUFFIXES {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    name.to_string()
}

/// The fields of an answer, against the fields the app declares.
fn assert_declared(declarations: &HashMap<String, Vec<String>>, name: &str, value: &Value) {
    let declared = match declarations.get(name) {
        Some(declared) => declared.clone(),
        None => panic!("the app declares no interface called {name}"),
    };
    let mut expected: Vec<String> = declared
        .iter()
        .filter(|field| !DROPPED_BY_THE_CONTRACT.contains(&field.as_str()))
        .map(|field| bare(field))
        .collect();
    expected.sort();
    expected.dedup();
    let mut actual: Vec<String> = common::keys(value)
        .iter()
        .map(|field| bare(field))
        .collect();
    actual.sort();
    actual.dedup();
    assert_eq!(
        actual, expected,
        "the answer does not carry the shape the app declares for {name}"
    );
}

/// The path into an answer.
fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for step in path {
        current = current.get(step)?;
    }
    Some(current)
}

/// The first item of an array in an answer.
fn first_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let array = at(value, path)?.as_array()?;
    array.first()
}

#[test]
fn the_feed_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-feed");
    let body = harness.ok_get("/activities");
    assert_declared(&declarations, "ActivityListResponse", &body);
    if let Some(run) = first_at(&body, &["activities"]) {
        assert_declared(&declarations, "ActivitySummary", run);
    }
}

#[test]
fn one_run_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-detail");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get(&format!("/activities/{id}"));
    assert_declared(&declarations, "ActivityDetail", &body);
}

#[test]
fn the_vitals_carry_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-vitals");
    let body = harness.ok_get("/physiology/vitals");
    assert_declared(&declarations, "VitalsSnapshot", &body);
    if let Some(rest) = at(&body, &["rest"]) {
        assert_declared(&declarations, "RestReading", rest);
    }
    if let Some(vital) = first_at(&body, &["vitals"]) {
        assert_declared(&declarations, "VitalReading", vital);
        if let Some(band) = at(vital, &["band"]) {
            if band.is_object() {
                assert_declared(&declarations, "VitalBand", band);
            }
        }
    }
    if let Some(point) = first_at(&body, &["history", "sleep"]) {
        assert_declared(&declarations, "VitalHistoryPoint", point);
    }
    if let Some(pattern) = first_at(&body, &["patterns"]) {
        assert_declared(&declarations, "PatternReading", pattern);
    }
}

#[test]
fn the_plan_week_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-week");
    let body = harness.ok_get("/plan/week");
    assert_declared(&declarations, "PlanWeek", &body);
}

#[test]
fn today_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-today");
    let body = harness.ok_get("/insights/today");
    assert_declared(&declarations, "TodayInsight", &body);
}

#[test]
fn progress_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-progress");
    let body = harness.ok_get("/insights/progress");
    assert_declared(&declarations, "ProgressInsight", &body);
}

#[test]
fn compare_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-compare");
    let first_id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let second_id = harness.write_run(common::run("b-run", "2026-05-03", 10_200.0, 3_320.0));
    let body = harness.ok_get(&format!("/compare?a={first_id}&b={second_id}"));
    assert_declared(&declarations, "CompareResponse", &body);
    if let Some(metric) = first_at(&body, &["metrics"]) {
        assert_declared(&declarations, "CompareMetric", metric);
    }
    if let Some(run) = at(&body, &["a"]) {
        assert_declared(&declarations, "CompareRun", run);
    }
}

#[test]
fn the_candidates_carry_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-candidates");
    harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));
    let body = harness.ok_get("/compare/candidates");
    assert!(
        body.is_array(),
        "the app reads the candidates as a list, not as an object"
    );
    if let Some(candidate) = body.as_array().and_then(|items| items.first()) {
        assert_declared(&declarations, "CompareCandidate", candidate);
    }
}

#[test]
fn the_sync_status_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-sync");
    let body = harness.ok_get("/sync/status");
    assert_declared(&declarations, "SyncStatus", &body);
}

#[test]
fn the_health_carries_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-health");
    let body = harness.ok_get("/health/daily");
    assert_declared(&declarations, "HealthDaily", &body);
}
