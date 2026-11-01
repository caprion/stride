// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The shapes the app consumes, checked against the app's own declarations.
//!
//! The response shapes are the contract between this crate and the app, and
//! the app writes its side of it down in `frontend/src/lib/types.ts`. This
//! test reads that file — the declarations themselves, not a copy of them —
//! and holds every answer to them.
//!
//! **It holds the shape, not only the names.** A field the app declares as
//! a rendered value must be an object with a `text` and a `raw`; a field
//! the app declares as nullable must be nullable in the answer; a field the
//! app declares as a plain number must be a number. A test that compared
//! only the names would pass while the wire shape was incompatible — the
//! names are the same whether the value is `8400` or `{text: "8.4 km",
//! raw: 8400}` — and that is the drift this file exists to prevent.
//!
//! So the two are compared field by field, with their types.

mod common;

use std::collections::BTreeMap;

use serde_json::Value;

/// The declarations of the app: an interface's name and its fields.
///
/// The fields carry the **declared type as the app writes it**, so that the
/// comparison is a comparison of shapes and not of names.
type Interface = BTreeMap<String, String>;

/// The app's declarations.
fn declarations() -> BTreeMap<String, Interface> {
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

    let mut declared: BTreeMap<String, (Option<String>, Interface)> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut pending: Option<(String, String)> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("export interface ") {
            let head = rest.split('{').next().unwrap_or_default();
            let (name, extends) = match head.split_once(" extends ") {
                Some((name, parent)) => (name.trim().to_string(), Some(parent.trim().to_string())),
                None => (head.trim().to_string(), None),
            };
            declared.insert(name.clone(), (extends, Interface::new()));
            current = Some(name);
            pending = None;
            continue;
        }
        if trimmed == "}" && pending.is_none() {
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
        if indent != 2 && pending.is_none() {
            continue;
        }
        // A field whose type is an inline object continues on the next
        // lines, until its braces balance.
        if let Some((field, type_text)) = pending.take() {
            let joined = format!("{type_text}\n{trimmed}");
            if balanced(&joined) {
                if let Some(interface) = declared.get_mut(&name) {
                    interface.1.insert(field, joined.trim().to_string());
                }
            } else {
                pending = Some((field, joined.trim().to_string()));
            }
            continue;
        }
        let field = match trimmed.split_once(':') {
            Some((field, type_text)) => {
                let field = field.trim().trim_end_matches('?').to_string();
                if field.is_empty()
                    || !field
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                {
                    continue;
                }
                (field, type_text.trim().to_string())
            }
            None => continue,
        };
        if balanced(&field.1) {
            if let Some(interface) = declared.get_mut(&name) {
                interface.1.insert(field.0, field.1);
            }
        } else {
            pending = Some(field);
        }
    }

    // An interface that extends another carries the other's fields too.
    // The merge is done in two passes, because the order the interfaces are
    // read in is not the order they are written in.
    let mut merged: BTreeMap<String, Interface> = BTreeMap::new();
    for (name, (_, fields)) in declared.iter() {
        merged.insert(name.clone(), fields.clone());
    }
    for (name, (extends, _)) in declared.iter() {
        let mut all: Interface = Interface::new();
        if let Some(parent) = extends {
            if let Some(parent_fields) = merged.get(parent) {
                for (field, type_text) in parent_fields {
                    all.insert(field.clone(), type_text.clone());
                }
            }
        }
        if let Some(fields) = merged.get(name) {
            for (field, type_text) in fields {
                all.insert(field.clone(), type_text.clone());
            }
        }
        merged.insert(name.clone(), all);
    }
    merged
}

/// The comments of a type text, taken out.
///
/// A comment inside an inline type is not a field, and a comment may hold
/// a semicolon — which would cut a field in half.
fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("/*") {
            inside = !trimmed.contains("*/");
            continue;
        }
        if inside {
            if trimmed.contains("*/") {
                inside = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Whether the braces of a type text balance.
fn balanced(text: &str) -> bool {
    let mut depth = 0i32;
    for character in text.chars() {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }
    depth <= 0
}

/// Whether a value is the declared type.
fn matches(value: &Value, type_text: &str, declarations: &BTreeMap<String, Interface>) -> bool {
    let text = type_text.trim();
    if let Some(rest) = text.strip_suffix("| null") {
        return value.is_null() || matches(value, rest, declarations);
    }
    if let Some(rest) = text.strip_suffix("[]") {
        return match value.as_array() {
            Some(items) => items.iter().all(|item| matches(item, rest, declarations)),
            None => false,
        };
    }
    if let Some(rest) = text.strip_prefix("Record<") {
        let inner = rest.trim_end_matches('>');
        let value_type = match inner.rsplit_once(',') {
            Some((_, value_type)) => value_type.trim(),
            None => return false,
        };
        return match value.as_object() {
            Some(object) => object
                .values()
                .all(|item| matches(item, value_type, declarations)),
            None => false,
        };
    }
    if text.starts_with('{') && text.ends_with('}') {
        let inner = strip_comments(&text[1..text.len() - 1]);
        let mut fields: Vec<(String, String)> = Vec::new();
        for part in inner.split([';', '\n']) {
            let part = part.trim();
            // A comment inside an inline type is not a field.
            if part.is_empty() || part.starts_with('/') || part.starts_with('*') {
                continue;
            }
            match part.split_once(':') {
                Some((field, type_text)) => {
                    fields.push((field.trim().to_string(), type_text.trim().to_string()))
                }
                None => return false,
            }
        }
        return match value.as_object() {
            Some(object) => fields.iter().all(|(field, type_text)| {
                object
                    .get(field)
                    .map(|value| matches(value, type_text, declarations))
                    .unwrap_or(false)
            }),
            None => false,
        };
    }
    match text {
        "RenderedValue" => match value.as_object() {
            Some(object) => {
                let mut names: Vec<&String> = object.keys().collect();
                names.sort();
                if names != vec!["raw", "text"] {
                    return false;
                }
                let text_ok = object
                    .get("text")
                    .map(|text| text.is_string())
                    .unwrap_or(false);
                let raw_ok = object
                    .get("raw")
                    .map(|raw| raw.is_number() || raw.is_null())
                    .unwrap_or(false);
                text_ok && raw_ok
            }
            None => false,
        },
        "number" => value.is_number(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "[number, number]" => match value.as_array() {
            Some(items) => items.len() == 2 && items.iter().all(|item| item.is_number()),
            None => false,
        },
        // Every other named type is either one of the app's own interfaces or
        // a string the app uses as a key.
        other => match declarations.get(other) {
            Some(fields) => match value.as_object() {
                Some(object) => fields.iter().all(|(field, type_text)| {
                    object
                        .get(field)
                        .map(|value| matches(value, type_text, declarations))
                        .unwrap_or(false)
                }),
                None => false,
            },
            None => value.is_string(),
        },
    }
}

/// The fields of an answer, against the fields the app declares.
fn assert_declared(declarations: &BTreeMap<String, Interface>, name: &str, value: &Value) {
    let declared = match declarations.get(name) {
        Some(declared) => declared.clone(),
        None => panic!("the app declares no interface called {name}"),
    };
    let actual = match value.as_object() {
        Some(object) => object.clone(),
        None => panic!("the answer of {name} is not an object: {value}"),
    };
    let mut actual_names: Vec<&String> = actual.keys().collect();
    actual_names.sort();
    let mut declared_names: Vec<&String> = declared.keys().collect();
    declared_names.sort();
    assert_eq!(
        actual_names, declared_names,
        "the answer of {name} does not carry the fields the app declares"
    );
    for (field, declared) in declared.iter() {
        let value = actual.get(field).unwrap_or(&Value::Null);
        assert!(
            matches(value, declared, declarations),
            "the field {name}.{field} is not the shape the app declares: {value}"
        );
    }
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

/// The app must not declare a format hint: the core renders the value.
#[test]
fn the_app_declares_no_format_hint() {
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
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("format") && trimmed.contains(':') {
            panic!(
                "the app declares a format hint at line {}: the contract says the core renders the value",
                index + 1
            );
        }
    }
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
    if let Some(split) = first_at(&body, &["splits"]) {
        // The splits are an inline type in the app's declarations; the
        // fields are checked one by one.
        assert!(
            split["index"].is_number(),
            "the split's index is not a number"
        );
        assert!(
            split["distance"].is_object(),
            "the split's distance is not rendered"
        );
        assert!(
            split["duration"].is_object(),
            "the split's duration is not rendered"
        );
        assert!(
            split["heart_rate"].is_null() || split["heart_rate"].is_object(),
            "the split's heart rate is not a rendered value"
        );
    }
    if let Some(point) = first_at(&body, &["samples"]) {
        assert!(
            point["offset_s"].is_number(),
            "the sample's offset is not a number"
        );
        assert!(
            point["pace"].is_null() || point["pace"].is_object(),
            "the sample's pace is not a rendered value"
        );
        assert!(
            point["heart_rate"].is_null() || point["heart_rate"].is_object(),
            "the sample's heart rate is not a rendered value"
        );
    }
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
    if let Some(session) = at(&body, &["planned"]) {
        if session.is_object() {
            assert_declared(&declarations, "PlanSession", session);
        }
    }
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

#[test]
fn the_settings_carry_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-settings");
    let body = harness.ok_get("/settings");
    assert_declared(&declarations, "Settings", &body);
    if let Some(pause) = at(&body, &["pause"]) {
        if pause.is_object() {
            assert_declared(&declarations, "Pause", pause);
        }
    }
}

#[test]
fn the_notes_carry_the_shape_the_app_declares() {
    let declarations = declarations();
    let harness = common::Harness::seeded("frontend-notes");
    let body = harness.ok_get("/notes");
    assert_declared(&declarations, "Notes", &body);
}
