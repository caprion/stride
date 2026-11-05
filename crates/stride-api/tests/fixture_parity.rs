//! The app's captured answers are the router's answers.
//!
//! The review found that the app's screens are tested against the app's own
//! fixture, so a screen can render fictional data and stay green. The fix is
//! this test: the app's **whole fixture set** — `answers.json` (the seed and
//! the answers) and `compare-answers.json` (every pair the Compare screen can
//! show) — is held to the router, **value for value**. A shape test that
//! passes while the numbers are invented is the same gap in a new coat.
//!
//! The test seeds the store from the fixture's own seed (`seed.activities`,
//! `seed.nights`, `seed.plan_sessions`), calls the real router, and compares
//! the whole answer — the run's distance, the direction word, the delta's
//! sign, the Rest line. If the router's answer and the app's fixture
//! disagree, this test names the route; it is never tuned to pass.
//!
//! ## The dump
//!
//! `STRIDE_FIXTURES_OUT=<path>` makes the test write the router's answers for
//! the same seed instead of comparing. The frontend's `fixtures:export` can
//! then take that file as the source, so the fixture is the router's answer
//! rather than a hand-written one.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use stride_api::{Api, Config, FixedClock, Router};
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, NewPlanBlock, NewPlanSession, Store, WellnessDaily};

/// The day the fixture renders: the app's "today".
const FIXTURE_TODAY: &str = "2026-05-13";

fn answers_path() -> std::path::PathBuf {
    fixture_path("answers.json")
}

fn compare_path() -> std::path::PathBuf {
    fixture_path("compare-answers.json")
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("frontend")
        .join("src")
        .join("fixtures")
        .join(name)
}

/// The router over a store seeded from the fixture's own seed.
fn fixture_router() -> (ScratchDir, Router, Arc<Store>) {
    let dir = ScratchDir::new("fixture-set").unwrap_or_else(|_| panic!("scratch"));
    let key = DatabaseKey::from_text("the database key of a test").unwrap();
    let store = Arc::new(Store::open_and_migrate(&dir.file("stride.db"), &key).unwrap());
    let garmin = Arc::new(common::FakeGarmin::new());
    // The fixture's day is 2026-05-13, not the test harness's 2026-05-12.
    let clock = FixedClock::new(
        DateTime::parse_from_rfc3339("2026-05-13T06:30:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    let api = Arc::new(Api::with_transport(
        store.clone(),
        Config::default(),
        Arc::new(clock),
        garmin,
    ));
    (dir, Router::new(api), store)
}

/// Seed the store from the fixture's seed.
///
/// `with_plan` controls whether the prescribed plan block and its sessions
/// are written. The baseline variant — a store with no plan block, so the
/// router infers the baseline from the athlete's own history — is the same
/// seed with `with_plan` off.
fn seed(store: &Store, seed: &Value, with_plan: bool) {
    let now = DateTime::parse_from_rfc3339(common::NOW)
        .unwrap()
        .with_timezone(&Utc);

    if let Some(activities) = seed.get("activities").and_then(|value| value.as_array()) {
        store
            .with_connection(|connection| {
                for activity in activities {
                    let id = activity
                        .get("id")
                        .and_then(|value| value.as_i64())
                        .unwrap_or(0);
                    let external_id = id.to_string();
                    let name = text(activity, "name");
                    let sport_type = text(activity, "sport_type");
                    let start_time = text(activity, "start_time");
                    let local_date = text(activity, "local_date");
                    let distance_m = raw(activity, "distance");
                    let moving_time_s = raw(activity, "duration");
                    let elevation_gain_m = raw(activity, "elevation_gain");
                    let avg_hr = activity
                        .get("heart_rate")
                        .and_then(|value| value.get("raw"))
                        .and_then(|value| value.as_f64());
                    let fit_sha256 = stride_store::hash::sha256_hex(b"");
                    connection
                        .execute(
                            "INSERT INTO activities (
                                id, source, external_id, name, sport_type, start_time,
                                local_date, distance_m, moving_time_s, elapsed_time_s,
                                elevation_gain_m, avg_hr, max_hr, fit_blob, fit_sha256,
                                created_at, updated_at
                             ) VALUES (?1, 'garmin_personal', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                                ?10, ?11, NULL, x'', ?12, ?13, ?13)",
                            rusqlite::params![
                                id,
                                external_id,
                                name,
                                sport_type,
                                start_time,
                                local_date,
                                distance_m,
                                moving_time_s,
                                moving_time_s + 60.0,
                                elevation_gain_m,
                                avg_hr,
                                fit_sha256,
                                crate_instant(now),
                            ],
                        )
                        .unwrap_or_else(|error| panic!("the run could not be seeded: {error}"));
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("the runs could not be seeded: {error}"));
    }

    if let Some(nights) = seed.get("nights").and_then(|value| value.as_array()) {
        for night in nights {
            let row = WellnessDaily {
                date: text(night, "date"),
                sleep_s: night.get("sleep_s").and_then(|value| value.as_f64()),
                deep_sleep_s: None,
                rem_s: None,
                light_sleep_s: None,
                awake_s: None,
                hrv_ms: night.get("hrv_ms").and_then(|value| value.as_f64()),
                resting_hr: night.get("resting_hr").and_then(|value| value.as_f64()),
                stress_avg: None,
                source: "garmin_personal".to_string(),
                updated_at: now,
            };
            store
                .upsert_wellness_daily(&row)
                .unwrap_or_else(|error| panic!("the night could not be seeded: {error}"));
        }
    }

    if with_plan {
        if let Some(sessions) = seed.get("plan_sessions").and_then(|value| value.as_array()) {
            let start = sessions
                .first()
                .and_then(|session| session.get("date"))
                .and_then(|value| value.as_str())
                .unwrap_or(FIXTURE_TODAY)
                .to_string();
            let block = NewPlanBlock {
                name: "Base block".to_string(),
                source: "prescribed".to_string(),
                start_date: start,
                end_date: None,
                goal: None,
                status: "active".to_string(),
                notes: None,
                created_at: now,
            };
            let block_id = store
                .upsert_plan_block(&block)
                .unwrap_or_else(|error| panic!("the plan block could not be seeded: {error}"));
            for session in sessions {
                let planned = NewPlanSession {
                    block_id,
                    date: text(session, "date"),
                    session_type: text(session, "session_type"),
                    title: session
                        .get("title")
                        .and_then(|value| value.as_str())
                        .map(|value| value.to_string()),
                    details: None,
                    target_distance_m: session
                        .get("target")
                        .and_then(|value| value.get("raw"))
                        .and_then(|value| value.as_f64()),
                    pace_low_s_km: None,
                    pace_high_s_km: None,
                };
                store.upsert_plan_session(&planned).unwrap_or_else(|error| {
                    panic!("the plan session could not be seeded: {error}")
                });
            }
        }
    }
}

fn text(value: &Value, name: &str) -> String {
    value
        .get(name)
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn raw(value: &Value, name: &str) -> f64 {
    value
        .get(name)
        .and_then(|value| value.get("raw"))
        .and_then(|value| value.as_f64())
        .unwrap_or_default()
}

fn crate_instant(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// The route a captured answer answers at.
fn route_to_path(route: &str) -> String {
    match route {
        // The export script asked the fixture for the pair (102, 101).
        "/compare" => "/compare?a=102&b=101".to_string(),
        // The zones are of one run; the fixture's first run is 101.
        "/fitness/zones" => "/fitness/zones?id=101".to_string(),
        other => other.to_string(),
    }
}

/// The router's answers, for the fixture's seed, keyed by route.
fn router_answers(router: &Router) -> serde_json::Map<String, Value> {
    let mut answers = serde_json::Map::new();
    let routes = [
        "/compare",
        "/activities",
        "/activities/101",
        "/physiology/vitals",
        "/physiology/sleep",
        "/fitness/zones",
        "/plan/week",
        "/insights/today",
        "/insights/progress",
        "/compare/candidates",
        "/sync/status",
        "/health",
        "/health/daily",
        "/auth/status",
        "/settings",
    ];
    for route in routes {
        let answer = router.call("GET", &route_to_path(route), None);
        answers.insert(route.to_string(), answer.body);
    }
    answers
}

/// The router's compare answers, for every pair.
fn router_compare_answers(router: &Router, ids: &[i64]) -> serde_json::Map<String, Value> {
    let mut answers = serde_json::Map::new();
    for a in ids {
        for b in ids {
            if a == b {
                continue;
            }
            let answer = router.call("GET", &format!("/compare?a={a}&b={b}"), None);
            answers.insert(format!("{a}:{b}"), answer.body);
        }
    }
    answers
}

/// Every answer the app renders is the router's answer for the same state.
#[test]
fn every_captured_answer_is_the_routers_answer() {
    let data = match read_json(&answers_path()) {
        Some(data) => data,
        None => return skip(&answers_path()),
    };
    let seed_value = data.get("seed").cloned().unwrap_or(Value::Null);
    let (_dir, router, store) = fixture_router();
    seed(&store, &seed_value, true);

    if let Ok(out) = std::env::var("STRIDE_FIXTURES_OUT") {
        // The prescribed variant.
        let answers = router_answers(&router);
        // The baseline variant: the same seed, no plan block, so the router
        // infers the baseline from the athlete's own history.
        let (_bdir, brouter, bstore) = fixture_router();
        seed(&bstore, &seed_value, false);
        let baseline = router_answers(&brouter);
        std::fs::write(
            &out,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "version": 1,
                    "answers": answers,
                    "baseline": baseline,
                    "seed": data.get("seed"),
                }))
                .unwrap_or_default()
            ),
        )
        .unwrap_or_else(|error| panic!("the answers could not be written to {out}: {error}"));
        eprintln!("wrote the router's answers to {out}");
        return;
    }

    let expected = data
        .get("answers")
        .and_then(|value| value.as_object())
        .unwrap_or_else(|| panic!("the captured answers have no answers map"));
    let mut disagreements = compare(&router, expected);

    // The baseline variant, when the fixture carries it.
    if let Some(expected_baseline) = data.get("baseline").and_then(|value| value.as_object()) {
        let (_bdir, brouter, bstore) = fixture_router();
        seed(&bstore, &seed_value, false);
        disagreements.extend(compare(&brouter, expected_baseline));
    }

    assert!(
        disagreements.is_empty(),
        "{} answer(s) disagreed:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
}

/// Compare the router's answers with the captured ones, route by route.
fn compare(router: &Router, expected: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut disagreements = Vec::new();
    for (route, wanted) in expected {
        let answer = router.call("GET", &route_to_path(route), None);
        if !answer.is_ok() {
            disagreements.push(format!(
                "{route}: the router did not answer: {}",
                answer.body
            ));
            continue;
        }
        let mine = serde_json::to_string(&answer.body).unwrap_or_default();
        let theirs = serde_json::to_string(wanted).unwrap_or_default();
        if mine != theirs {
            disagreements.push(format!("{route}: the answers differ"));
            eprintln!("--- {route} disagrees ---\nrouter: {mine}\napp:    {theirs}\n");
        }
    }
    disagreements
}

/// Every Compare pair the app can show is the router's answer.
#[test]
fn every_compare_pair_is_the_routers_answer() {
    let data = match read_json(&answers_path()) {
        Some(data) => data,
        None => return skip(&answers_path()),
    };
    let (_dir, router, store) = fixture_router();
    seed(&store, data.get("seed").unwrap_or(&Value::Null), true);
    let ids: Vec<i64> = data
        .get("seed")
        .and_then(|value| value.get("activities"))
        .and_then(|value| value.as_array())
        .map(|activities| {
            activities
                .iter()
                .filter_map(|activity| activity.get("id").and_then(|value| value.as_i64()))
                .collect()
        })
        .unwrap_or_default();

    if let Ok(out) = std::env::var("STRIDE_COMPARE_OUT") {
        std::fs::write(
            &out,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&router_compare_answers(&router, &ids))
                    .unwrap_or_default()
            ),
        )
        .unwrap_or_else(|error| {
            panic!("the compare answers could not be written to {out}: {error}")
        });
        eprintln!("wrote the router's compare answers to {out}");
        return;
    }

    let expected = match read_json(&compare_path()) {
        Some(value) => value,
        None => return skip(&compare_path()),
    };
    let mut disagreements = Vec::new();
    if let Some(object) = expected.as_object() {
        for (pair, wanted) in object {
            let (a, b) = match pair.split_once(':') {
                Some((a, b)) => (a, b),
                None => continue,
            };
            let answer = router.call("GET", &format!("/compare?a={a}&b={b}"), None);
            if !answer.is_ok() {
                disagreements.push(format!(
                    "{pair}: the router did not answer: {}",
                    answer.body
                ));
                continue;
            }
            let mine = serde_json::to_string(&answer.body).unwrap_or_default();
            let theirs = serde_json::to_string(wanted).unwrap_or_default();
            if mine != theirs {
                disagreements.push(format!("{pair}: the answers differ"));
                eprintln!("--- {pair} disagrees ---\nrouter: {mine}\napp:    {theirs}\n");
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "{} compare pair(s) disagreed:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
}

fn read_json(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn skip(path: &Path) {
    eprintln!(
        "the fixture set is not in the tree yet ({}) — it lands in the frontend's pull request, and this test will then hold the router to it value for value",
        path.display()
    );
}
