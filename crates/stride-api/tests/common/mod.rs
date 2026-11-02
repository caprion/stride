//! The helpers the tests share.
//!
//! Every test gets its own scratch directory, its own database and its own
//! clock. The clock stands still at 2026-05-12 06:30 UTC, so every
//! `generated_at` and every date the router derives is the same on every
//! run.
//!
//! The Garmin transport is a fake: the tests never open a socket, and the
//! fake records every request so that a test can prove how many calls the
//! router made.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde_json::Value;
use stride_api::{Api, Config, FixedClock, Router};
use stride_ingest::{HttpRequest, HttpResponse, HttpTransport, TransportError};
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, NewActivity, NewPlanBlock, NewPlanSession, Store, WellnessDaily};

/// The instant the tests read the clock at.
pub const NOW: &str = "2026-05-12T06:30:00Z";

/// The day the tests read the clock on.
pub const TODAY: &str = "2026-05-12";

/// A Garmin key that is distinctive enough to be looked for.
///
/// It is not a credential: it opens nothing, it is refused by nothing, and
/// it is here so that a test can prove it never appears in an answer.
pub const GARMIN_KEY: &str = "stride-test-key-0000-not-a-credential";

/// The instant the tests read the clock at.
pub fn instant() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(NOW)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// The state, with a clock that stands still.
pub struct Harness {
    /// The directory the database lives in.
    pub dir: ScratchDir,
    /// The database.
    pub store: Arc<Store>,
    /// The shared state.
    pub api: Arc<Api>,
    /// The router.
    pub router: Router,
    /// The fake Garmin.
    pub garmin: Arc<FakeGarmin>,
}

impl Harness {
    /// The state, with an empty database.
    pub fn empty(prefix: &str) -> Harness {
        Harness::with_config(prefix, Config::default())
    }

    /// The state, with a configuration of the caller's choosing.
    ///
    /// The export routes need a directory the host chose, and this is how
    /// a test gives them one.
    pub fn with_config(prefix: &str, config: Config) -> Harness {
        let dir = match ScratchDir::new(prefix) {
            Ok(dir) => dir,
            Err(_) => panic!("the scratch directory could be made"),
        };
        let key = match DatabaseKey::from_text("the database key of a test") {
            Ok(key) => key,
            Err(_) => panic!("the database key could be made"),
        };
        let store = Arc::new(
            match Store::open_and_migrate(&dir.file("stride.db"), &key) {
                Ok(store) => store,
                Err(_) => panic!("the store could be opened"),
            },
        );
        let garmin = Arc::new(FakeGarmin::new());
        let api = Arc::new(Api::with_transport(
            store.clone(),
            config,
            Arc::new(FixedClock::new(instant())),
            garmin.clone(),
        ));
        Harness {
            dir,
            store,
            router: Router::new(api.clone()),
            api,
            garmin,
        }
    }

    /// The state, with a run, a night of wellness and a plan.
    pub fn seeded(prefix: &str) -> Harness {
        let harness = Harness::empty(prefix);
        harness.write_run(run("a-run-on-the-tenth", "2026-05-10", 10_000.0, 3_000.0));
        harness.write_wellness(wellness("2026-05-10"));
        harness.write_wellness(wellness("2026-05-11"));
        harness.write_wellness(wellness(TODAY));
        harness
    }

    /// Write a run.
    pub fn write_run(&self, activity: NewActivity) -> i64 {
        match self.store.upsert_activity(&activity) {
            Ok(id) => id,
            Err(_) => panic!("the run could be written"),
        }
    }

    /// Write a day of wellness.
    pub fn write_wellness(&self, row: WellnessDaily) {
        if let Err(error) = self.store.upsert_wellness_daily(&row) {
            panic!("the wellness could be written: {error}");
        }
    }

    /// Write a plan: a block and its sessions.
    ///
    /// The sessions are `(date, kind, target distance in metres)`.
    pub fn write_plan(
        &self,
        name: &str,
        source: &str,
        sessions: &[(&str, &str, Option<f64>)],
    ) -> i64 {
        let block = NewPlanBlock {
            name: name.to_string(),
            source: source.to_string(),
            start_date: sessions
                .first()
                .map(|session| session.0.to_string())
                .unwrap_or_else(|| TODAY.to_string()),
            end_date: None,
            goal: None,
            status: "active".to_string(),
            notes: None,
            created_at: instant(),
        };
        let block_id = match self.store.upsert_plan_block(&block) {
            Ok(id) => id,
            Err(_) => panic!("the plan block could not be written"),
        };
        for (date, kind, target) in sessions {
            let planned = NewPlanSession {
                block_id,
                date: date.to_string(),
                session_type: kind.to_string(),
                title: None,
                details: None,
                target_distance_m: *target,
                pace_low_s_km: None,
                pace_high_s_km: None,
            };
            if let Err(error) = self.store.upsert_plan_session(&planned) {
                panic!("the plan session could not be written: {error}");
            }
        }
        block_id
    }

    /// Call a route and read the body.
    pub fn get(&self, path: &str) -> Value {
        let answer = self.router.call("GET", path, None);
        answer.body
    }

    /// Call a route and read the body, with the answer checked.
    pub fn ok_get(&self, path: &str) -> Value {
        let answer = self.router.call("GET", path, None);
        if !answer.is_ok() {
            panic!("the route {path} did not answer: {}", answer.body);
        }
        answer.body
    }

    /// Call a route with a body and read the body.
    pub fn post(&self, path: &str, body: &str) -> Value {
        self.router.call("POST", path, Some(body)).body
    }
}

/// A run to write.
pub fn run(
    external_id: &str,
    local_date: &str,
    distance_m: f64,
    moving_time_s: f64,
) -> NewActivity {
    NewActivity {
        source: "garmin_personal".to_string(),
        external_id: external_id.to_string(),
        name: "a run".to_string(),
        sport_type: "running".to_string(),
        start_time: instant(),
        local_date: local_date.to_string(),
        distance_m,
        moving_time_s,
        elapsed_time_s: moving_time_s + 60.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        fit_blob: Vec::new(),
        created_at: instant(),
        updated_at: instant(),
    }
}

/// A run with no heart rate and no elevation.
pub fn run_without_readings(
    external_id: &str,
    local_date: &str,
    distance_m: f64,
    moving_time_s: f64,
) -> NewActivity {
    let mut activity = run(external_id, local_date, distance_m, moving_time_s);
    activity.avg_hr = None;
    activity.max_hr = None;
    activity.elevation_gain_m = 0.0;
    activity
}

/// A day of wellness.
pub fn wellness(date: &str) -> WellnessDaily {
    WellnessDaily {
        date: date.to_string(),
        sleep_s: Some(22_200.0),
        deep_sleep_s: Some(5_400.0),
        rem_s: Some(4_800.0),
        light_sleep_s: Some(12_000.0),
        awake_s: Some(1_800.0),
        hrv_ms: Some(48.0),
        resting_hr: Some(63.0),
        stress_avg: Some(28.0),
        source: "garmin_personal".to_string(),
        updated_at: instant(),
    }
}

/// A day of wellness, with the readings the test wants.
pub fn wellness_with(
    date: &str,
    sleep_s: Option<f64>,
    hrv_ms: Option<f64>,
    resting_hr: Option<f64>,
) -> WellnessDaily {
    let mut row = wellness(date);
    row.sleep_s = sleep_s;
    row.hrv_ms = hrv_ms;
    row.resting_hr = resting_hr;
    row
}

/// A day of wellness with nothing measured.
pub fn wellness_without_readings(date: &str) -> WellnessDaily {
    let mut row = wellness(date);
    row.sleep_s = None;
    row.deep_sleep_s = None;
    row.rem_s = None;
    row.light_sleep_s = None;
    row.awake_s = None;
    row.hrv_ms = None;
    row.resting_hr = None;
    row.stress_avg = None;
    row
}

/// The Garmin transport the tests use: a fake that records every request.
pub struct FakeGarmin {
    inner: Mutex<FakeInner>,
}

#[derive(Default)]
struct FakeInner {
    answers: HashMap<String, HttpResponse>,
    requests: Vec<(String, Option<String>)>,
}

impl FakeGarmin {
    /// A fake that refuses everything.
    pub fn new() -> FakeGarmin {
        FakeGarmin {
            inner: Mutex::new(FakeInner::default()),
        }
    }

    /// Answer a path with a body.
    pub fn with_json(&self, path: &str, body: &str) {
        self.with_response(
            path,
            HttpResponse {
                status: 200,
                headers: Vec::new(),
                body: body.as_bytes().to_vec(),
            },
        );
    }

    /// Answer a path with a status and a body.
    pub fn with_status(&self, path: &str, status: u16, body: &str) {
        self.with_response(
            path,
            HttpResponse {
                status,
                headers: Vec::new(),
                body: body.as_bytes().to_vec(),
            },
        );
    }

    /// Answer a path with a response.
    ///
    /// The answer is the one every request to that path gets. A path that
    /// carries the day in it — Garmin's HRV endpoint does — is answered by
    /// the longest prefix that matches, so one answer covers every day.
    pub fn with_response(&self, path: &str, response: HttpResponse) {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.answers.insert(path.to_string(), response);
    }

    /// Every request the fake was asked for, with the path and the
    /// authorization header.
    pub fn requests(&self) -> Vec<(String, Option<String>)> {
        let inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.requests.clone()
    }

    /// How many requests the fake was asked for.
    pub fn request_count(&self) -> usize {
        self.requests().len()
    }
}

impl HttpTransport for FakeGarmin {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let path = request.url.path().to_string();
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner
            .requests
            .push((path.clone(), request.auth_value.clone()));
        // The longest prefix that matches answers the path.
        let mut best: Option<&String> = None;
        for candidate in inner.answers.keys() {
            if path.starts_with(candidate.as_str())
                && best
                    .map(|best| candidate.len() > best.len())
                    .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
        let answer = match best.and_then(|candidate| inner.answers.get(candidate)) {
            Some(answer) => HttpResponse {
                status: answer.status,
                headers: answer.headers.clone(),
                body: answer.body.clone(),
            },
            None => HttpResponse {
                status: 404,
                headers: Vec::new(),
                body: b"no answer was set for this path".to_vec(),
            },
        };
        Ok(answer)
    }
}

/// The path of the token exchange.
pub const TOKEN_PATH: &str = "/personalapitoken-service/personal-api/personal-api-key/token";

/// Encode bytes as base64.
///
/// The import route carries a file as text, so the test has to be able to
/// write that text.
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let first = chunk[0] as u32;
        let second = *chunk.get(1).unwrap_or(&0) as u32;
        let third = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (first << 16) | (second << 8) | third;
        out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// A corpus FIT file, read from the repository.
pub fn corpus_fit(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("fit")
        .join(name);
    match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => panic!("the corpus file {name} could not be read: {error}"),
    }
}

/// The keys of a JSON object, sorted.
pub fn keys(value: &Value) -> Vec<String> {
    match value.as_object() {
        Some(object) => {
            let mut names: Vec<String> = object.keys().cloned().collect();
            names.sort();
            names
        }
        None => Vec::new(),
    }
}
