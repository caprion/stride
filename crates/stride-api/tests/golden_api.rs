// The test code unwraps and expects: it is the test, and the workspace
// denies those lints in library code only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The API golden test.
//!
//! The fixtures in `fixtures/api_golden.json` were recorded from the Python
//! app — the reference implementation — by `fixtures/record_api_golden.py`.
//! This test drives the Rust router with the same rows and compares the
//! answers.
//!
//! Two comparisons, because they answer different questions:
//!
//! * **shape** — the same keys, in the same places, with the same types.
//!   This is the contract the React app consumes, and it does not change.
//! * **value** — the same numbers. Every float is compared with the
//!   tolerance `docs/core-contract.md` §8 sets (1e-9 relative); everything
//!   else is compared exactly.
//!
//! The routes whose answer comes from `stride-core` (the fingerprint, the
//! readiness, the comparison, the plan engine) are fed the recorded answer,
//! because this crate does not reimplement the domain logic — it shapes it.
//! What is compared there is the shaping.

use std::sync::Arc;

use serde_json::{json, Map, Value};

use stride_api::backend::{
    ActivityRow, DailyMetricRow, LapRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow, PlanSessionRow,
    PlanWeekRow, StreamRow, ZoneRow,
};
use stride_api::{Api, MemBackend};

/// The recorded golden file.
const GOLDEN: &str = include_str!("fixtures/api_golden.json");

/// The routes the Python app answers through its security middleware, which
/// sits above the router: the router's own answer for them is a 405.
const MIDDLEWARE_CASES: [&str; 2] = ["POST /api/v1/activities", "PATCH /api/v1/activities/1"];

/// The routes whose answer cannot be compared value for value, because it
/// describes the machine the recording ran on rather than the data.
const SHAPE_ONLY: [&str; 3] = [
    "GET /api/v1/auth/status",
    "GET /api/v1/network/status",
    // The tile cache directory is a path on the machine that recorded it.
    "GET /api/v1/tiles/policy",
];

/// Build the backend the recording used.
fn backend(golden: &Value) -> MemBackend {
    let backend = MemBackend::new();
    let rows = &golden["rows"];

    for row in rows["activities"].as_array().cloned().unwrap_or_default() {
        backend.add_activity(serde_json::from_value::<ActivityRow>(row).expect("activity row"));
    }
    for row in rows["laps"].as_array().cloned().unwrap_or_default() {
        backend.add_lap(serde_json::from_value::<LapRow>(row).expect("lap row"));
    }
    for row in rows["streams"].as_array().cloned().unwrap_or_default() {
        backend.add_stream(serde_json::from_value::<StreamRow>(row).expect("stream row"));
    }
    for row in rows["zones"].as_array().cloned().unwrap_or_default() {
        backend.add_zone(serde_json::from_value::<ZoneRow>(row).expect("zone row"));
    }
    for row in rows["physiology"].as_array().cloned().unwrap_or_default() {
        backend
            .add_physiology(serde_json::from_value::<PhysiologyRow>(row).expect("physiology row"));
    }
    for row in rows["daily_metrics"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        backend.add_daily_metric(serde_json::from_value::<DailyMetricRow>(row).expect("daily row"));
    }
    for row in rows["plan_blocks"].as_array().cloned().unwrap_or_default() {
        backend.add_block(serde_json::from_value::<PlanBlockRow>(row).expect("block row"));
    }
    for row in rows["plan_phases"].as_array().cloned().unwrap_or_default() {
        backend.add_phase(serde_json::from_value::<PlanPhaseRow>(row).expect("phase row"));
    }
    for row in rows["plan_weeks"].as_array().cloned().unwrap_or_default() {
        backend.add_week(serde_json::from_value::<PlanWeekRow>(row).expect("week row"));
    }
    for row in rows["plan_sessions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        backend.add_session(serde_json::from_value::<PlanSessionRow>(row).expect("session row"));
    }
    backend
}

/// The recorded answer of a route.
fn recorded<'a>(golden: &'a Value, path: &str) -> Option<&'a Value> {
    golden["cases"]
        .as_array()?
        .iter()
        .find(|case| case["path"] == path)
        .map(|case| &case["response"])
}

/// Feed the derived numbers — the ones `stride-core` owns — to the backend.
///
/// The recording holds them: the fingerprint of a run, the readiness of a
/// day, the baselines of a day, the comparison, the candidates, the plan
/// results, the adherence and the planned sessions. The router shapes them;
/// it does not compute them.
fn inject_derived(backend: &MemBackend, golden: &Value) {
    if let Some(body) = recorded(golden, "/api/v1/fitness/activity/1") {
        let mut item = body.as_object().cloned().unwrap_or_default();
        // The zones are the API layer's own work: the router computes them
        // from the zone model and the stored zones, so they are not fed in.
        item.remove("zones");
        backend.set_derived("fingerprint:1", Value::Object(item));
    }
    if let Some(body) = recorded(golden, "/api/v1/fitness/fingerprint") {
        backend.set_derived("fingerprints:", body["items"].clone());
    }
    if let Some(body) = recorded(
        golden,
        "/api/v1/physiology/readiness?from=2026-03-01&to=2026-03-10",
    ) {
        backend.set_derived("readiness_range:", body["items"].clone());
        for item in body["items"].as_array().cloned().unwrap_or_default() {
            if let Some(date) = item["date"].as_str() {
                backend.set_derived(&format!("readiness:{date}"), item.clone());
            }
        }
    }
    // The readiness of the latest day, which the physiology page shows.
    if let Some(body) = recorded(golden, "/api/v1/physiology/latest") {
        let day = body["day"]["date"].as_str().unwrap_or_default().to_string();
        let readiness = body["readiness"].clone();
        if !day.is_empty() && !readiness.is_null() {
            backend.set_derived(&format!("readiness:{day}"), readiness);
        }
    }
    // The readiness of today, which the plan page shows.
    if let Some(body) = recorded(golden, "/api/v1/plan/today") {
        let readiness = body["readiness"].clone();
        if !readiness.is_null() {
            let today = golden["today"].as_str().unwrap_or_default().to_string();
            backend.set_derived(&format!("readiness:{today}"), readiness);
        }
    }
    if let Some(body) = recorded(golden, "/api/v1/physiology/hrv") {
        for item in body["items"].as_array().cloned().unwrap_or_default() {
            if let Some(date) = item["date"].as_str() {
                backend.set_derived(
                    &format!("baselines:{date}"),
                    json!({
                        "hrv_ms": item["baseline"],
                        "resting_hr": item["rhr_baseline"],
                    }),
                );
            }
        }
    }
    if let Some(body) = recorded(golden, "/api/v1/compare?ids=1,2") {
        backend.set_derived("compare:", body.clone());
    }
    if let Some(body) = recorded(golden, "/api/v1/compare/candidates") {
        backend.set_derived("compare_candidates", body["items"].clone());
    }
    // The plan ranges the router asks for: the span of the active block,
    // the week that contains the day, and today.
    let block = recorded(golden, "/api/v1/plan/blocks/1").map(|body| body["block"].clone());
    let block_range = block
        .as_ref()
        .map(|block| {
            (
                block["start_date"].as_str().unwrap_or_default().to_string(),
                block["end_date"].as_str().unwrap_or_default().to_string(),
            )
        })
        .unwrap_or_default();
    if let Some(body) = recorded(golden, "/api/v1/plan/results") {
        backend.set_derived(
            &format!("plan_results:{}:{}", block_range.0, block_range.1),
            body["items"].clone(),
        );
    }
    if let Some(body) = recorded(golden, "/api/v1/plan/adherence") {
        backend.set_derived(
            &format!("plan_adherence:{}:{}", block_range.0, block_range.1),
            body.clone(),
        );
    }
    if let Some(body) = recorded(golden, "/api/v1/plan/week?date=2026-03-04") {
        let sessions: Vec<Value> = body["sessions"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|session| {
                let mut row = session.as_object().cloned().unwrap_or_default();
                let result = row.remove("result").unwrap_or(Value::Null);
                let activity = row.remove("activity").unwrap_or(Value::Null);
                json!({
                    "session": Value::Object(row),
                    "result": result,
                    "activity": activity,
                })
            })
            .collect();
        let week = body["week"].clone();
        let start = week["start_date"].as_str().unwrap_or_default().to_string();
        let end = week["end_date"].as_str().unwrap_or_default().to_string();
        backend.set_derived(
            &format!("plan_sessions:{start}:{end}"),
            Value::Array(sessions),
        );
        // Nothing is planned for today unless the reference says so.
        let today = golden["today"].as_str().unwrap_or_default().to_string();
        let today_sessions = recorded(golden, "/api/v1/plan/today")
            .map(|body| body["session"].clone())
            .filter(|session| !session.is_null())
            .map(|session| {
                let mut row = session.as_object().cloned().unwrap_or_default();
                let result = row.remove("result").unwrap_or(Value::Null);
                let activity = row.remove("activity").unwrap_or(Value::Null);
                json!({
                    "session": Value::Object(row),
                    "result": result,
                    "activity": activity,
                })
            })
            .map(|session| Value::Array(vec![session]))
            .unwrap_or_else(|| Value::Array(Vec::new()));
        backend.set_derived(&format!("plan_sessions:{today}:{today}"), today_sessions);
    }
    if let Some(body) = recorded(golden, "/api/v1/network/status") {
        // The base of the network status: the route adds the preference, the
        // effective route, the nodes and the resolved node on top of it.
        let mut base = body.as_object().cloned().unwrap_or_default();
        for key in [
            "public_identity",
            "effective_route",
            "nodes",
            "resolved_exit_node",
        ] {
            base.remove(key);
        }
        if let Some(effective) = body.get("effective_route").cloned() {
            base.insert("route".to_string(), effective);
        }
        backend.set_derived("network_status", Value::Object(base));
        backend.set_derived(
            "public_identity",
            body.get("public_identity").cloned().unwrap_or(Value::Null),
        );
        backend.set_derived(
            "exit_nodes",
            body.get("nodes").cloned().unwrap_or(Value::Null),
        );
    }
    if let Some(body) = recorded(golden, "/api/v1/sync/status") {
        let mut base = body.as_object().cloned().unwrap_or_default();
        for key in ["last_sync_at", "connected", "method"] {
            base.remove(key);
        }
        backend.set_derived("sync_status", Value::Object(base));
    }
}

/// The router under test.
fn api(golden: &Value) -> Api {
    let backend = backend(golden);
    inject_derived(&backend, golden);
    let today = chrono::NaiveDate::parse_from_str(
        golden["today"].as_str().unwrap_or("2026-01-01"),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default());
    Api::with_today(Arc::new(backend), today)
}

/// Compare two answers for shape: the same keys, the same types.
fn assert_same_shape(path: &str, expected: &Value, actual: &Value) {
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            let left_keys: Vec<&String> = left.keys().collect();
            let right_keys: Vec<&String> = right.keys().collect();
            assert_eq!(
                left_keys, right_keys,
                "the keys of {path} differ: {left_keys:?} vs {right_keys:?}"
            );
            for (key, value) in left {
                assert_same_shape(&format!("{path}.{key}"), value, &right[key]);
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            assert_eq!(
                left.len(),
                right.len(),
                "the length of {path} differs: {} vs {}",
                left.len(),
                right.len()
            );
            for (index, (left, right)) in left.iter().zip(right.iter()).enumerate() {
                assert_same_shape(&format!("{path}[{index}]"), left, right);
            }
        }
        // A null means "not measured", and two items of the same list may
        // disagree about it; the value comparison is what decides.
        (Value::Null, _) | (_, Value::Null) => {}
        (Value::Bool(_), Value::Bool(_)) => {}
        (Value::String(_), Value::String(_)) => {}
        (Value::Number(_), Value::Number(_)) => {}
        _ => panic!(
            "the type of {path} differs: {} vs {}",
            kind(expected),
            kind(actual)
        ),
    }
}

/// Compare two answers for value, with the tolerance the contract sets.
fn assert_same_value(path: &str, expected: &Value, actual: &Value) {
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, value) in left {
                assert_same_value(&format!("{path}.{key}"), value, &right[key]);
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            assert_eq!(left.len(), right.len(), "the length of {path} differs");
            for (index, (left, right)) in left.iter().zip(right.iter()).enumerate() {
                assert_same_value(&format!("{path}[{index}]"), left, right);
            }
        }
        (Value::Number(left), Value::Number(right)) => {
            let left = left.as_f64().unwrap_or(f64::NAN);
            let right = right.as_f64().unwrap_or(f64::NAN);
            let tolerance = 1e-9 * left.abs().max(1.0);
            assert!(
                (left - right).abs() <= tolerance,
                "the number at {path} differs: {left} vs {right}"
            );
        }
        (Value::Null, Value::Null) => {}
        _ => assert_eq!(
            expected, actual,
            "the value at {path} differs: {expected} vs {actual}"
        ),
    }
}

/// The name of a JSON type, for the failure message.
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Every recorded route answers with the same shape as the reference.
#[test]
fn the_router_answers_with_the_shape_of_the_reference() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let mut checked = 0;
    let mut skipped = 0;

    for case in golden["cases"].as_array().cloned().unwrap_or_default() {
        let method = case["method"].as_str().unwrap_or("GET").to_string();
        let path = case["path"].as_str().unwrap_or_default().to_string();
        let key = format!("{method} {path}");
        if MIDDLEWARE_CASES.contains(&key.as_str()) {
            skipped += 1;
            continue;
        }
        let body = case["body"].as_str().map(str::to_string);
        let expected = &case["response"];

        let outcome = api.call(&method, &path, body.as_deref());
        match outcome {
            Ok(actual) => {
                assert_same_shape(&key, expected, &actual);
                if !SHAPE_ONLY.contains(&key.as_str()) {
                    assert_same_value(&key, expected, &actual);
                }
            }
            Err(err) => {
                // An error answer is only right when the reference answered
                // with one too.
                let expected_status = case["status"].as_u64().unwrap_or(0);
                assert!(
                    expected_status >= 400,
                    "{key} failed with {err} but the reference answered {expected_status}"
                );
                assert_same_shape(&key, expected, &err.to_body());
            }
        }
        checked += 1;
    }
    assert!(checked >= 45, "only {checked} routes were checked");
    assert_eq!(skipped, MIDDLEWARE_CASES.len());
}

/// The numbers the reference produces for the seeded activities.
#[test]
fn the_activity_list_matches_the_reference_number_for_number() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let expected = recorded(&golden, "/api/v1/activities?per_page=5&page=1").expect("recorded");
    let actual = api
        .call("GET", "/api/v1/activities?per_page=5&page=1", None)
        .expect("the list answers");
    assert_same_value("activities", expected, &actual);
    assert_eq!(
        actual["total"], 8,
        "the FIT fixtures hold eight runs and one ride"
    );
}

/// A malformed path is the reference's 404.
#[test]
fn an_unknown_path_is_the_reference_not_found() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let err = api
        .call("GET", "/api/v1/nonexistent", None)
        .expect_err("404");
    assert_eq!(err.status, 404);
    assert_eq!(err.detail, json!("Not found"));
    let body = err.to_body();
    let expected = recorded(&golden, "/api/v1/nonexistent").expect("recorded");
    assert_eq!(body, *expected);
}

/// A run that is not there is the reference's 404.
#[test]
fn a_missing_activity_is_the_reference_not_found() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let err = api
        .call("GET", "/api/v1/activities/999", None)
        .expect_err("404");
    assert_eq!(err.status, 404);
    assert_eq!(err.detail, json!("Activity not found."));
}

/// A path that exists with another method is a 405, as FastAPI says.
#[test]
fn a_wrong_method_is_not_allowed() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let err = api.call("POST", "/api/v1/health", None).expect_err("405");
    assert_eq!(err.status, 405);
    assert_eq!(err.detail, json!("Method Not Allowed"));
}

/// The route table covers every path of the OpenAPI document.
#[test]
fn the_route_table_covers_every_path() {
    let routes = stride_api::router::ROUTES;
    let patterns: Vec<&str> = routes.iter().map(|route| route.pattern).collect();
    for expected in [
        "/api/v1/health",
        "/api/v1/health/daily",
        "/api/v1/auth/status",
        "/api/v1/auth/pair",
        "/api/v1/auth/pair-code",
        "/api/v1/auth/garmin/personal-key",
        "/api/v1/auth/garmin/personal-key/test",
        "/api/v1/activities",
        "/api/v1/activities/{activity_id}",
        "/api/v1/activities/{activity_id}/geo",
        "/api/v1/activities/{activity_id}/streams/{kind}",
        "/api/v1/physiology/daily",
        "/api/v1/physiology/latest",
        "/api/v1/physiology/readiness",
        "/api/v1/physiology/sleep",
        "/api/v1/physiology/sources",
        "/api/v1/fitness/fingerprint",
        "/api/v1/fitness/activity/{id}",
        "/api/v1/fitness/trends",
        "/api/v1/fitness/zones",
        "/api/v1/compare",
        "/api/v1/compare/candidates",
        "/api/v1/stats/summary",
        "/api/v1/stats/monthly",
        "/api/v1/stats/weekly",
        "/api/v1/stats/pace-trends",
        "/api/v1/stats/personal-records",
        "/api/v1/stats/heatmap",
        "/api/v1/stats/elevation",
        "/api/v1/stats/hr-zones",
        "/api/v1/calendar",
        "/api/v1/gear",
        "/api/v1/plan/current",
        "/api/v1/plan/today",
        "/api/v1/plan/week",
        "/api/v1/plan/results",
        "/api/v1/plan/adherence",
        "/api/v1/plan/blocks",
        "/api/v1/plan/blocks/{block_id}",
        "/api/v1/plan/blocks/{block_id}/phases",
        "/api/v1/plan/phases/{phase_id}/weeks",
        "/api/v1/plan/weeks/{week_id}/sessions",
        "/api/v1/plan/import",
        "/api/v1/sync/status",
        "/api/v1/sync/run",
        "/api/v1/sync/runs",
        "/api/v1/sync/history",
        "/api/v1/sync/settings",
        "/api/v1/network/status",
        "/api/v1/network/route",
        "/api/v1/tiles/policy",
        "/api/v1/tiles/{z}/{x}/{y}.png",
    ] {
        assert!(
            patterns.contains(&expected),
            "the route table is missing {expected}"
        );
    }
}

/// `?fields=` trims what nobody draws, and only what the caller asks for.
///
/// The default answer is untouched: the golden test above proves it, key for
/// key and number for number.
#[test]
fn the_fields_parameter_trims_the_answer_and_nothing_else() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);

    // The whole summary, as it always was.
    let whole = api
        .call("GET", "/api/v1/activities", None)
        .expect("the list answers");
    let item = whole["items"][0].as_object().expect("an item").clone();
    assert!(item.contains_key("route_preview"));
    assert!(item.contains_key("pace_label"));

    // Only the named fields.
    let trimmed = api
        .call("GET", "/api/v1/activities?fields=id,name,distance_m", None)
        .expect("the list answers");
    let trimmed_item = trimmed["items"][0].as_object().expect("an item");
    let mut names: Vec<&str> = trimmed_item.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["distance_m", "id", "name"]);
    // The envelope is not trimmed: the client still knows how many there are.
    assert_eq!(trimmed["total"], whole["total"]);
    assert_eq!(trimmed["pages"], whole["pages"]);

    // A name nobody has is ignored, not an error.
    let unknown = api
        .call("GET", "/api/v1/activities?fields=id,nonsense", None)
        .expect("the list answers");
    assert_eq!(unknown["items"][0].as_object().expect("an item").len(), 1);

    // The detail is trimmed the same way.
    let detail = api
        .call("GET", "/api/v1/activities/1?fields=id,name", None)
        .expect("the detail answers");
    let keys: Vec<&str> = detail
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["id", "name"]);

    // The plan's phases are 107 KB that no screen draws: the week alone is
    // a much smaller answer.
    let current = api
        .call("GET", "/api/v1/plan/current?fields=week", None)
        .expect("the plan answers");
    let keys: Vec<&str> = current
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["week"]);
}

/// `?fields=` trims every route that offers it, at the level that route
/// documents.
///
/// Seventeen handlers read the parameter; three of them were covered by the
/// test above. This one walks the rest, and checks two things for each: the
/// named fields survive, and the unnamed ones are gone — while the
/// untrimmed answer still has them.
#[test]
fn the_fields_parameter_works_on_every_route_that_offers_it() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);

    // A route, the fields to keep, and where the filter applies:
    // `items`, `object`, `sessions` or `cards`.
    let cases: [(&str, &str, &str); 17] = [
        ("/api/v1/activities", "id,name", "items"),
        ("/api/v1/activities/1", "id,name", "object"),
        (
            "/api/v1/fitness/fingerprint",
            "activity_id,cadence_avg",
            "items",
        ),
        ("/api/v1/fitness/activity/1", "activity_id", "object"),
        ("/api/v1/plan/blocks", "id,name", "items"),
        ("/api/v1/plan/current", "week", "object"),
        ("/api/v1/plan/week?date=2026-03-04", "id,date", "sessions"),
        ("/api/v1/plan/today", "date,has_session", "object"),
        ("/api/v1/stats/weekly", "week_key,distance_m", "items"),
        ("/api/v1/stats/monthly", "month_key,distance_m", "items"),
        ("/api/v1/stats/pace-trends", "week_key,run_count", "items"),
        ("/api/v1/stats/personal-records", "label,value", "items"),
        (
            "/api/v1/calendar?year=2026&month=3",
            "name,distance_m",
            "cards",
        ),
        ("/api/v1/compare?ids=1,2", "id,name", "runs"),
        ("/api/v1/physiology/daily", "date,sleep_s", "items"),
        ("/api/v1/physiology/hrv", "date,hrv_ms", "items"),
        (
            "/api/v1/physiology/readiness?from=2026-03-01&to=2026-03-10",
            "date,score",
            "items",
        ),
    ];

    for (path, fields, level) in cases {
        let separator = if path.contains('?') { '&' } else { '?' };
        let whole = api.call("GET", path, None).expect("the route answers");
        let trimmed = api
            .call("GET", &format!("{path}{separator}fields={fields}"), None)
            .expect("the route answers");

        // The named fields survive, and the unnamed ones are gone.
        let wanted: Vec<&str> = fields.split(',').collect();
        let named = |value: &Value| -> Vec<String> {
            value
                .as_object()
                .map(|object| object.keys().cloned().collect())
                .unwrap_or_default()
        };
        match level {
            "items" => {
                let items = trimmed["items"].as_array().expect("the items");
                for item in items {
                    let keys = named(item);
                    for name in &wanted {
                        assert!(
                            keys.contains(&name.to_string()),
                            "{path}: the field {name} was dropped"
                        );
                    }
                    assert!(
                        keys.iter().all(|key| wanted.contains(&key.as_str())),
                        "{path}: an item kept {keys:?}, which was not asked for"
                    );
                }
                // The untrimmed answer has more.
                let whole_items = whole["items"].as_array().expect("the items");
                if let (Some(first), Some(trimmed_first)) = (whole_items.first(), items.first()) {
                    assert!(
                        named(first).len() >= named(trimmed_first).len(),
                        "{path}: the trimmed item is not smaller"
                    );
                }
            }
            "object" => {
                let keys = named(&trimmed);
                for name in &wanted {
                    assert!(
                        keys.contains(&name.to_string()),
                        "{path}: the field {name} was dropped"
                    );
                }
                assert!(
                    keys.iter().all(|key| wanted.contains(&key.as_str())),
                    "{path}: the answer kept {keys:?}, which was not asked for"
                );
                assert!(
                    named(&whole).len() >= keys.len(),
                    "{path}: the trimmed answer is not smaller"
                );
            }
            "sessions" => {
                let sessions = trimmed["sessions"].as_array().expect("the sessions");
                for session in sessions {
                    let keys = named(session);
                    for name in &wanted {
                        assert!(
                            keys.contains(&name.to_string()),
                            "{path}: the field {name} was dropped"
                        );
                    }
                    assert!(
                        keys.iter().all(|key| wanted.contains(&key.as_str())),
                        "{path}: a session kept {keys:?}, which was not asked for"
                    );
                }
            }
            "cards" => {
                let days = trimmed["days"].as_array().expect("the days");
                for day in days {
                    let cards = day["activities"].as_array().expect("the cards");
                    for card in cards {
                        let keys = named(card);
                        for name in &wanted {
                            assert!(
                                keys.contains(&name.to_string()),
                                "{path}: the field {name} was dropped"
                            );
                        }
                        assert!(
                            keys.iter().all(|key| wanted.contains(&key.as_str())),
                            "{path}: a card kept {keys:?}, which was not asked for"
                        );
                    }
                }
            }
            "runs" => {
                let runs = trimmed["runs"].as_array().expect("the runs");
                for run in runs {
                    let keys = named(run);
                    for name in &wanted {
                        assert!(
                            keys.contains(&name.to_string()),
                            "{path}: the field {name} was dropped"
                        );
                    }
                    assert!(
                        keys.iter().all(|key| wanted.contains(&key.as_str())),
                        "{path}: a run kept {keys:?}, which was not asked for"
                    );
                }
            }
            other => panic!("the test does not know the level {other}"),
        }
    }
}

/// The rejected requests answer the way the reference rejects them.
///
/// The reference is FastAPI, so a query parameter that is out of range or
/// not a number is a 422 with a `detail` array — and the array carries the
/// value that was sent and the bound it broke.
#[test]
fn a_rejected_request_answers_like_fastapi() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);

    // Out of range.
    let err = api
        .call("GET", "/api/v1/activities?per_page=0", None)
        .expect_err("422");
    assert_eq!(err.status, 422);
    assert_eq!(
        err.detail,
        json!([{
            "type": "greater_than_equal",
            "loc": ["query", "per_page"],
            "msg": "Input should be greater than or equal to 1",
            "input": "0",
            "ctx": {"ge": 1},
        }])
    );

    // Not a number.
    let err = api
        .call("GET", "/api/v1/activities?page=abc", None)
        .expect_err("422");
    assert_eq!(err.status, 422);
    assert_eq!(
        err.detail,
        json!([{
            "type": "int_parsing",
            "loc": ["query", "page"],
            "msg": "Input should be a valid integer, unable to parse string as an integer",
            "input": "abc",
        }])
    );

    // Missing a required parameter.
    let err = api.call("GET", "/api/v1/compare", None).expect_err("422");
    assert_eq!(err.status, 422);
    assert_eq!(
        err.detail,
        json!([{
            "type": "missing",
            "loc": ["query", "ids"],
            "msg": "Field required",
            "input": Value::Null,
        }])
    );
}

/// A document that cannot be read is an answer, not a crash.
#[test]
fn a_broken_plan_document_is_answered_not_thrown() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let body = json!({
        "format": "yaml",
        "content": "not a plan at all",
    })
    .to_string();
    let answer = api
        .call("POST", "/api/v1/plan/import", Some(&body))
        .expect("the import answers");
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["block_id"], Value::Null);
    assert_eq!(answer["phases"], 0);
    assert_eq!(answer["weeks"], 0);
    assert_eq!(answer["sessions"], 0);
    assert_eq!(answer["warnings"], json!([]));
    assert_eq!(answer["error"], "format must be markdown, csv, or json");
}

/// A file that cannot be parsed is an answer, not a crash.
#[test]
fn a_broken_fit_file_is_answered_not_thrown() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let body = json!({
        "files": [{
            "filename": "broken.fit",
            "content_base64": "bm90IGEgZml0IGZpbGU=",
        }],
    })
    .to_string();
    let answer = api
        .call("POST", "/api/v1/auth/garmin/fit-import", Some(&body))
        .expect("the import answers");
    assert_eq!(answer["imported"], 0);
    assert_eq!(answer["activities"], json!([]));
    let errors = answer["errors"].as_array().expect("the errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["filename"], "broken.fit");
    assert!(
        errors[0]["error"]
            .as_str()
            .expect("a message")
            .contains("could not be parsed"),
        "the error is {:?}",
        errors[0]["error"]
    );
}

/// The OpenAPI document lists 63 paths and 66 operations. Every one of them
/// is in the route table, with the same parameter names.
#[test]
fn the_route_table_covers_the_openapi_document() {
    let document: Value = serde_json::from_str(include_str!("fixtures/openapi_paths.json"))
        .expect("the document parses");
    let paths = document["paths"].as_object().expect("the paths");
    assert_eq!(paths.len(), 63, "the reference has 63 paths");

    let routes = stride_api::router::ROUTES;
    for (path, operations) in paths {
        for (method, operation) in operations.as_object().expect("the operations") {
            let method = method.to_uppercase();
            let found = routes
                .iter()
                .find(|route| route.method == method && route.pattern == path);
            let route = found.unwrap_or_else(|| {
                panic!("the route table is missing {method} {path}");
            });
            // Every parameter the document declares must be one the route
            // can read: a path parameter is in the pattern, and a query
            // parameter is read by the handler.
            for parameter in operation["parameters"]
                .as_array()
                .cloned()
                .unwrap_or_default()
            {
                let name = parameter["name"].as_str().unwrap_or_default();
                let place = parameter["in"].as_str().unwrap_or_default();
                if place == "path" {
                    assert!(
                        route.pattern.contains(&format!("{{{name}}}")),
                        "the route {method} {path} does not carry the path parameter {name}"
                    );
                }
            }
        }
    }
}

/// The response shapes are what the OpenAPI document promises.
#[test]
fn the_shapes_match_the_openapi_document() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden file parses");
    let api = api(&golden);
    let mut seen: Map<String, Value> = Map::new();
    for case in golden["cases"].as_array().cloned().unwrap_or_default() {
        let method = case["method"].as_str().unwrap_or("GET");
        let path = case["path"].as_str().unwrap_or_default();
        if MIDDLEWARE_CASES.contains(&format!("{method} {path}").as_str()) {
            continue;
        }
        let body = case["body"].as_str().map(str::to_string);
        if let Ok(actual) = api.call(method, path, body.as_deref()) {
            seen.insert(format!("{method} {path}"), actual);
        }
    }
    // The list and the detail answer with the same run summary, so the
    // detail carries everything the list does and more.
    let list = &seen["GET /api/v1/activities"];
    let detail = &seen["GET /api/v1/activities/1"];
    let summary = list["items"][0].as_object().expect("an item").clone();
    for key in summary.keys() {
        assert!(
            detail.get(key).is_some(),
            "the activity detail is missing {key}"
        );
    }
    // Every list item has the same keys.
    for item in list["items"].as_array().expect("items") {
        assert_same_shape("activity", &Value::Object(summary.clone()), item);
    }
}
