//! The delta pull.
//!
//! The pull is a delta: a known run is not downloaded twice, a changed run is
//! re-parsed, the pull continues where it stopped, Garmin's pages are walked,
//! the budget is spent on un-pulled runs, an interrupted pull resumes, and a
//! run Garmin no longer lists is marked and not resurrected.
//!
//! Every one of those is a test here, against a fake transport — never the
//! live API.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use stride_api::{Api, Config, FixedClock, Router};
use stride_ingest::{HttpRequest, HttpResponse, HttpTransport, TransportError};
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, NewActivity, Store};

const NOW: &str = "2026-05-12T06:30:00Z";
const GARMIN_KEY: &str = "stride-test-key-0000-not-a-credential";
const TOKEN_PATH: &str = "/personalapitoken-service/personal-api/personal-api-key/token";
const ACTIVITIES_PATH: &str = "/activitylist-service/activities/search/activities";
const DOWNLOAD_PATH: &str = "/download-service/files/activity";

/// The corpus FIT every download answers with, and its decoded truth.
///
/// The list entries carry the same distance and start time, so that a run
/// that was pulled is "unchanged" the next time the list is read.
fn decoded_truth() -> (Vec<u8>, f64, DateTime<Utc>) {
    let bytes = common::corpus_fit("real_garmin_run.fit");
    let decoded = stride_ingest::decode_fit(&bytes, "activity.fit")
        .unwrap_or_else(|_| panic!("the corpus FIT could not be decoded"));
    let start = decoded.start_time.unwrap_or_else(instant);
    (bytes, decoded.distance_m, start)
}

fn instant() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(NOW)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// One activity in the fake's list, newest first.
#[derive(Clone)]
struct Listed {
    id: String,
    distance_m: f64,
    start: String,
}

/// A fake Garmin that pages the activity search and serves FIT downloads.
struct SyncGarmin {
    listed: Mutex<Vec<Listed>>,
    fit: Vec<u8>,
    rate_limit_after_pages: AtomicUsize,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    requests: Vec<(String, String)>,
}

impl SyncGarmin {
    fn new(listed: Vec<Listed>, fit: Vec<u8>) -> Self {
        Self {
            listed: Mutex::new(listed),
            fit,
            rate_limit_after_pages: AtomicUsize::new(usize::MAX),
            inner: Mutex::new(Inner::default()),
        }
    }

    fn set_list(&self, listed: Vec<Listed>) {
        *self
            .listed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = listed;
    }

    fn rate_limit_after(&self, pages: usize) {
        self.rate_limit_after_pages.store(pages, Ordering::SeqCst);
    }

    fn requests(&self) -> Vec<(String, String)> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .requests
            .clone()
    }

    fn downloads(&self) -> usize {
        self.requests()
            .iter()
            .filter(|(path, _)| path.starts_with(DOWNLOAD_PATH))
            .count()
    }
}

impl HttpTransport for SyncGarmin {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let path = request.url.path().to_string();
        let query = request.url.query().unwrap_or_default().to_string();
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .requests
            .push((path.clone(), query.clone()));

        if path == TOKEN_PATH {
            return Ok(HttpResponse {
                status: 200,
                headers: Vec::new(),
                body: b"\"the-access-value\"".to_vec(),
            });
        }

        if path == ACTIVITIES_PATH {
            let params = parse_query(&query);
            let start = params
                .get("start")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = params
                .get("limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100);
            // A page past the rate-limit boundary is refused, the way a real
            // rate limit refuses mid-walk.
            let page_no = start / limit;
            if page_no >= self.rate_limit_after_pages.load(Ordering::SeqCst) {
                return Ok(HttpResponse {
                    status: 429,
                    headers: Vec::new(),
                    body: b"rate limited".to_vec(),
                });
            }
            let listed = self
                .listed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();
            let page: Vec<Value> = listed
                .iter()
                .skip(start)
                .take(limit)
                .map(|entry| {
                    json!({
                        "activityId": entry.id,
                        "activityName": format!("run {}", entry.id),
                        "distance": entry.distance_m,
                        "startTimeGMT": entry.start,
                    })
                })
                .collect();
            let body = serde_json::to_vec(&json!({ "activityList": page })).unwrap_or_default();
            return Ok(HttpResponse {
                status: 200,
                headers: Vec::new(),
                body,
            });
        }

        if path.starts_with(DOWNLOAD_PATH) {
            return Ok(HttpResponse {
                status: 200,
                headers: Vec::new(),
                body: self.fit.clone(),
            });
        }

        Ok(HttpResponse {
            status: 404,
            headers: Vec::new(),
            body: b"not found".to_vec(),
        })
    }
}

fn parse_query(query: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for part in query.split('&') {
        if let Some((name, value)) = part.split_once('=') {
            out.insert(name.to_string(), value.to_string());
        }
    }
    out
}

/// The router over a fake Garmin and a fresh store.
struct PullHarness {
    router: Router,
    store: Arc<Store>,
    garmin: Arc<SyncGarmin>,
}

impl PullHarness {
    fn with_budget(listed: Vec<Listed>, fit: Vec<u8>, budget: usize) -> Self {
        let dir = ScratchDir::new("sync-delta").unwrap_or_else(|_| panic!("scratch"));
        let key = DatabaseKey::from_text("the database key of a test").unwrap();
        let store = Arc::new(Store::open_and_migrate(&dir.file("stride.db"), &key).unwrap());
        let garmin = Arc::new(SyncGarmin::new(listed, fit));
        let config = Config {
            max_activities_per_sync: budget,
            ..Config::default()
        };
        let api = Arc::new(Api::with_transport(
            store.clone(),
            config,
            Arc::new(FixedClock::new(instant())),
            garmin.clone(),
        ));
        api.set_garmin_key(GARMIN_KEY);
        Self {
            router: Router::new(api),
            store,
            garmin,
        }
    }

    fn new(listed: Vec<Listed>, fit: Vec<u8>) -> Self {
        Self::with_budget(listed, fit, 20)
    }

    fn listed(id: &str, distance_m: f64, start: &str) -> Listed {
        Listed {
            id: id.to_string(),
            distance_m,
            start: start.to_string(),
        }
    }

    fn seed(&self, id: &str, distance_m: f64, start: DateTime<Utc>) {
        self.store
            .upsert_activity(&NewActivity {
                source: "garmin_personal".to_string(),
                external_id: id.to_string(),
                name: format!("run {id}"),
                sport_type: "running".to_string(),
                start_time: start,
                local_date: "2026-05-12".to_string(),
                distance_m,
                moving_time_s: 3_000.0,
                elapsed_time_s: 3_060.0,
                elevation_gain_m: 0.0,
                avg_hr: Some(150.0),
                max_hr: Some(170.0),
                fit_blob: Vec::new(),
                created_at: instant(),
                updated_at: instant(),
            })
            .unwrap_or_else(|_| panic!("the run could not be seeded"));
    }

    fn pull(&self, days: i64) -> Value {
        let body = format!("{{\"days\": {days}}}");
        self.router.call("POST", "/sync/run", Some(&body)).body
    }

    fn status(&self) -> Value {
        self.router.call("GET", "/sync/status", None).body
    }
}

#[test]
fn a_known_run_is_not_downloaded_twice() {
    let (fit, distance, start) = decoded_truth();
    let harness = PullHarness::new(
        vec![PullHarness::listed("100", distance, &start.to_rfc3339())],
        fit,
    );
    harness.seed("100", distance, start);

    let body = harness.pull(30);
    assert_eq!(body["activities"].as_u64(), Some(0));
    assert_eq!(body["skipped"].as_u64(), Some(1));
    assert_eq!(harness.garmin.downloads(), 0);
}

#[test]
fn a_changed_run_is_re_parsed_and_replaced() {
    let (fit, distance, start) = decoded_truth();
    // The list carries a distance the stored run does not have: it changed.
    let harness = PullHarness::new(
        vec![PullHarness::listed(
            "100",
            distance + 1_000.0,
            &start.to_rfc3339(),
        )],
        fit,
    );
    harness.seed("100", distance, start);

    let body = harness.pull(30);
    assert_eq!(body["activities"].as_u64(), Some(1));
    assert_eq!(body["skipped"].as_u64(), Some(0));
    assert_eq!(harness.garmin.downloads(), 1);

    // The run now carries the file's numbers, not the stale ones.
    let stored = harness
        .store
        .activity_by_external("garmin_personal", "100")
        .unwrap_or(None)
        .unwrap_or_else(|| panic!("the run was not replaced"));
    assert_eq!(stored.distance_m, distance);
}

#[test]
fn the_pull_walks_garmins_pages() {
    let (fit, distance, start) = decoded_truth();
    let mut listed = Vec::new();
    for index in 0..250u32 {
        listed.push(PullHarness::listed(
            &format!("{}", 1000 - index),
            distance,
            &start.to_rfc3339(),
        ));
    }
    // A budget large enough to cover the whole 250-run list, across three
    // pages of 100.
    let harness = PullHarness::with_budget(listed, fit, 300);
    let body = harness.pull(30);
    assert_eq!(body["activities"].as_u64(), Some(250));
    assert_eq!(harness.garmin.downloads(), 250);
    // Three list pages were fetched, not one.
    let searches = harness
        .garmin
        .requests()
        .iter()
        .filter(|(path, _)| path == ACTIVITIES_PATH)
        .count();
    assert_eq!(searches, 3);
}

#[test]
fn the_pull_is_bounded_by_the_budget() {
    let (fit, distance, start) = decoded_truth();
    let mut listed = Vec::new();
    for index in 0..50u32 {
        listed.push(PullHarness::listed(
            &format!("{}", 200 - index),
            distance,
            &start.to_rfc3339(),
        ));
    }
    let harness = PullHarness::new(listed, fit);

    let body = harness.pull(30);
    // The default budget is 20: no more than twenty FIT files came down.
    assert_eq!(body["activities"].as_u64(), Some(20));
    assert_eq!(harness.garmin.downloads(), 20);
}

#[test]
fn no_starvation_the_budget_goes_to_unpulled_runs() {
    let (fit, distance, start) = decoded_truth();
    let mut listed = Vec::new();
    for index in 0..50u32 {
        listed.push(PullHarness::listed(
            &format!("{}", 250 - index),
            distance,
            &start.to_rfc3339(),
        ));
    }
    let harness = PullHarness::new(listed, fit);

    // Three pulls, a budget of twenty each: fifty runs are pulled, and
    // nothing is downloaded twice.
    assert_eq!(harness.pull(30)["activities"].as_u64(), Some(20));
    assert_eq!(harness.pull(30)["activities"].as_u64(), Some(20));
    assert_eq!(harness.pull(30)["activities"].as_u64(), Some(10));
    assert_eq!(harness.garmin.downloads(), 50);
    assert_eq!(harness.store.count_activities().unwrap_or(0), 50);

    // A fourth pull downloads nothing: the history is fully pulled.
    let fourth = harness.pull(30);
    assert_eq!(fourth["activities"].as_u64(), Some(0));
    assert_eq!(fourth["skipped"].as_u64(), Some(50));
    assert_eq!(harness.garmin.downloads(), 50);
}

#[test]
fn an_interrupted_pull_resumes_without_downloading_again() {
    let (fit, distance, start) = decoded_truth();
    let mut listed = Vec::new();
    for index in 0..200u32 {
        listed.push(PullHarness::listed(
            &format!("{}", 300 - index),
            distance,
            &start.to_rfc3339(),
        ));
    }
    // A budget larger than one page, so the walk reaches the second page,
    // which is refused.
    let harness = PullHarness::with_budget(listed, fit, 150);
    harness.garmin.rate_limit_after(1);

    let first = harness.pull(30);
    // The first page was walked and its hundred runs pulled; the second page
    // was refused, so the pull stopped early.
    assert_eq!(first["activities"].as_u64(), Some(100));
    assert_eq!(harness.garmin.downloads(), 100);
    // The cursor did not advance: the pull did not finish.
    assert!(harness.status()["cursor"].is_null());

    // The rate limit clears; the next pull skips the hundred it has and
    // pulls the hundred it has not.
    harness.garmin.rate_limit_after(usize::MAX);
    let second = harness.pull(30);
    assert_eq!(second["skipped"].as_u64(), Some(100));
    assert_eq!(second["activities"].as_u64(), Some(100));
    assert_eq!(harness.garmin.downloads(), 200);
    assert_eq!(harness.store.count_activities().unwrap_or(0), 200);
}

#[test]
fn a_run_garmin_no_longer_lists_is_marked_and_not_resurrected() {
    let (fit, distance, start) = decoded_truth();
    let harness = PullHarness::new(
        vec![PullHarness::listed("100", distance, &start.to_rfc3339())],
        fit,
    );
    harness.seed("100", distance, start);

    // Garmin no longer lists it: the run is marked and removed.
    harness.garmin.set_list(Vec::new());
    let gone = harness.pull(30);
    assert_eq!(gone["deleted"].as_u64(), Some(1));
    assert!(
        harness
            .store
            .activity_by_external("garmin_personal", "100")
            .unwrap_or(None)
            .is_none(),
        "the run Garmin no longer lists is gone"
    );

    // It reappears in a later list, and it is not resurrected.
    harness.garmin.set_list(vec![PullHarness::listed(
        "100",
        distance,
        &start.to_rfc3339(),
    )]);
    let again = harness.pull(30);
    assert_eq!(again["activities"].as_u64(), Some(0));
    assert!(
        harness
            .store
            .activity_by_external("garmin_personal", "100")
            .unwrap_or(None)
            .is_none(),
        "a deleted run was resurrected"
    );
}

#[test]
fn the_cursor_is_the_newest_activity_pulled() {
    let (fit, distance, start) = decoded_truth();
    let harness = PullHarness::new(
        vec![
            PullHarness::listed("102", distance, &start.to_rfc3339()),
            PullHarness::listed("101", distance, &start.to_rfc3339()),
            PullHarness::listed("100", distance, &start.to_rfc3339()),
        ],
        fit,
    );
    harness.pull(30);

    let status = harness.status();
    assert_eq!(status["cursor"].as_str(), Some("102"));
    assert_eq!(status["pulled"].as_u64(), Some(3));
    assert_eq!(status["skipped"].as_u64(), Some(0));
}
