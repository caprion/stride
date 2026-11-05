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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

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
    hrv_failure_status: AtomicUsize,
    retry_after_seconds: AtomicUsize,
    filter_dates: AtomicBool,
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
            hrv_failure_status: AtomicUsize::new(0),
            retry_after_seconds: AtomicUsize::new(0),
            filter_dates: AtomicBool::new(false),
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
                .filter(|entry| {
                    if !self.filter_dates.load(Ordering::SeqCst) {
                        return true;
                    }
                    let date = &entry.start[..10];
                    params
                        .get("startDate")
                        .map_or(true, |from| date >= from.as_str())
                        && params.get("endDate").map_or(true, |to| date <= to.as_str())
                })
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

        let is_hrv = path.starts_with(&format!("{}/", stride_ingest::HRV_DAILY_PATH));
        if is_hrv && self.hrv_failure_status.load(Ordering::SeqCst) != 0 {
            let retry_after = self.retry_after_seconds.load(Ordering::SeqCst);
            return Ok(HttpResponse {
                status: self.hrv_failure_status.load(Ordering::SeqCst) as u16,
                headers: if retry_after > 0 {
                    vec![("Retry-After".to_string(), retry_after.to_string())]
                } else {
                    Vec::new()
                },
                body: Vec::new(),
            });
        }
        if is_hrv
            || [
                stride_ingest::garmin::SLEEP_DAILY_PATH,
                stride_ingest::garmin::USER_SUMMARY_PATH,
            ]
            .contains(&path.as_str())
        {
            return Ok(HttpResponse {
                status: 204,
                headers: Vec::new(),
                body: Vec::new(),
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
        Self::with_config(
            listed,
            fit,
            Config {
                max_activities_per_sync: budget,
                max_sync_requests: usize::MAX,
                max_sync_response_bytes: usize::MAX,
                ..Config::default()
            },
        )
    }

    fn with_config(listed: Vec<Listed>, fit: Vec<u8>, config: Config) -> Self {
        let dir = ScratchDir::new("sync-delta").unwrap_or_else(|_| panic!("scratch"));
        let key = DatabaseKey::from_text("the database key of a test").unwrap();
        let store = Arc::new(Store::open_and_migrate(&dir.file("stride.db"), &key).unwrap());
        let garmin = Arc::new(SyncGarmin::new(listed, fit));
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
        self.seed_with(
            id,
            distance_m,
            start,
            vec![
                0x0E, 0x10, 0x00, 0x00, 0x2E, 0x46, 0x49, 0x54, 0x00, 0x00, 0x00, 0x00,
            ],
        )
    }

    fn seed_without_file(&self, id: &str, distance_m: f64, start: DateTime<Utc>) {
        self.seed_with(id, distance_m, start, Vec::new())
    }

    fn seed_with(&self, id: &str, distance_m: f64, start: DateTime<Utc>, fit_blob: Vec<u8>) {
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
                fit_blob,
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

    fn backfill(&self, days: i64) -> Value {
        self.router
            .call(
                "POST",
                "/sync/run",
                Some(&format!("{{\"days\":{days},\"backfill\":true}}")),
            )
            .body
    }
}

#[test]
fn a_legacy_activity_cursor_survives_status_and_sync() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    let legacy_id = "12345678901234567890";
    let previous = instant() - chrono::Duration::days(1);
    harness
        .store
        .write_sync_state(&stride_store::SyncState {
            kind: "activities".to_string(),
            cursor: Some(legacy_id.to_string()),
            last_success_at: Some(previous),
            token_expires_at: None,
            updated_at: previous,
        })
        .unwrap();

    let status = harness.router.call("GET", "/sync/status", None);
    assert_eq!(status.status, 200);
    assert_eq!(status.body["cursor"], legacy_id);
    assert_eq!(status.body["pulled"], 0);
    assert!(status.body["history"].is_null());
    assert!(harness.garmin.requests().is_empty());
    let unchanged = harness.store.sync_state("activities").unwrap().unwrap();
    assert_eq!(unchanged.cursor.as_deref(), Some(legacy_id));
    assert_eq!(unchanged.last_success_at, Some(previous));

    let pulled = harness
        .router
        .call("POST", "/sync/run", Some("{\"days\":1}"));
    assert_eq!(pulled.status, 200);
    let state = harness.store.sync_state("activities").unwrap().unwrap();
    let cursor: Value = serde_json::from_str(state.cursor.as_deref().unwrap()).unwrap();
    assert_eq!(cursor["newest"], legacy_id);
    assert!(cursor["history"].is_object());
    assert_eq!(harness.status()["cursor"], legacy_id);
}

#[test]
fn a_malformed_activity_cursor_is_not_silently_reset() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    for cursor in ["not-an-id", "{}", "-123", "12.3"] {
        harness
            .store
            .write_sync_state(&stride_store::SyncState {
                kind: "activities".to_string(),
                cursor: Some(cursor.to_string()),
                last_success_at: Some(instant()),
                token_expires_at: None,
                updated_at: instant(),
            })
            .unwrap();
        for (method, path, body) in [
            ("GET", "/sync/status", None),
            ("POST", "/sync/run", Some("{\"days\":1}")),
        ] {
            let response = harness.router.call(method, path, body);
            assert_eq!(response.status, 500);
            assert_eq!(response.body["detail"], "the sync cursor could not be read");
        }
        let state = harness.store.sync_state("activities").unwrap().unwrap();
        assert_eq!(state.cursor.as_deref(), Some(cursor));
    }
    assert!(harness.garmin.requests().is_empty());
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
fn a_known_run_without_its_file_is_downloaded_once() {
    let (fit, distance, start) = decoded_truth();
    let harness = PullHarness::new(
        vec![PullHarness::listed("100", distance, &start.to_rfc3339())],
        fit,
    );
    // The importer carries a run across with no FIT file.
    harness.seed_without_file("100", distance, start);
    let stored = harness
        .store
        .activity_by_external("garmin_personal", "100")
        .unwrap_or(None)
        .unwrap_or_else(|| panic!("the run was not seeded"));
    assert!(stored.fit_blob.is_empty(), "the seed should carry no file");

    // The first pull fetches the file, because the file is the source of
    // truth and a run without one has no series.
    let body = harness.pull(30);
    assert_eq!(body["activities"].as_u64(), Some(1));
    assert_eq!(harness.garmin.downloads(), 1);
    let stored = harness
        .store
        .activity_by_external("garmin_personal", "100")
        .unwrap_or(None)
        .unwrap_or_else(|| panic!("the run was not kept"));
    assert!(!stored.fit_blob.is_empty(), "the run now has its file");

    // The second pull skips it: the file is held.
    let body = harness.pull(30);
    assert_eq!(body["activities"].as_u64(), Some(0));
    assert_eq!(body["skipped"].as_u64(), Some(1));
    assert_eq!(harness.garmin.downloads(), 1);
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
fn a_budget_stop_never_deletes_unseen_runs_or_advances_the_cursor() {
    let (fit, distance, start) = decoded_truth();
    let listed = (0..5)
        .map(|index| {
            PullHarness::listed(&format!("{}", 200 - index), distance, &start.to_rfc3339())
        })
        .collect();
    let harness = PullHarness::with_budget(listed, fit, 2);
    harness.seed("196", distance, start);
    harness.seed("195", distance, start);

    let result = harness.pull(30);
    assert_eq!(result["activities"].as_u64(), Some(2));
    assert_eq!(result["deleted"].as_u64(), Some(0));
    assert!(harness.status()["cursor"].is_null());
    for id in ["196", "195"] {
        assert!(harness
            .store
            .activity_by_external("garmin_personal", id)
            .unwrap()
            .is_some());
        assert!(!harness.store.activity_is_deleted(id).unwrap());
    }

    let complete = harness.pull(30);
    assert_eq!(complete["deleted"].as_u64(), Some(1));
    assert!(harness
        .store
        .activity_by_external("garmin_personal", "196")
        .unwrap()
        .is_some());
}

#[test]
fn a_provider_page_cap_does_not_authorize_deletion() {
    let (fit, distance, start) = decoded_truth();
    let listed = (0..300)
        .map(|index| {
            PullHarness::listed(&format!("{}", 1000 - index), distance, &start.to_rfc3339())
        })
        .collect();
    let harness = PullHarness::with_budget(listed, fit, 400);
    harness.seed("100", distance, start);

    let result = harness.pull(30);
    assert_eq!(result["deleted"].as_u64(), Some(0));
    assert!(harness
        .store
        .activity_by_external("garmin_personal", "100")
        .unwrap()
        .is_some());
    assert!(harness.status()["cursor"].is_null());
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

#[test]
fn a_partial_activity_walk_does_not_stamp_a_successful_scan() {
    let (fit, distance, start) = decoded_truth();
    let harness = PullHarness::with_budget(
        vec![
            PullHarness::listed("102", distance, &start.to_rfc3339()),
            PullHarness::listed("101", distance, &start.to_rfc3339()),
        ],
        fit,
        1,
    );
    harness.pull(30);
    let state = harness.store.sync_state("activities").unwrap().unwrap();
    assert!(state.last_success_at.is_none());
    assert_eq!(harness.status()["more_pending"], true);
    assert!(harness.status()["last_error"].is_null());

    harness.pull(30);
    harness.pull(30);
    let state = harness.store.sync_state("activities").unwrap().unwrap();
    assert_eq!(state.last_success_at, Some(instant()));
    assert!(harness.status()["last_error"].is_null());
}

#[test]
fn an_hrv_failure_preserves_the_previous_wellness_success_and_stops_requests() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    let previous = instant() - chrono::Duration::days(1);
    harness
        .store
        .write_sync_state(&stride_store::SyncState {
            kind: "wellness".to_string(),
            cursor: None,
            last_success_at: Some(previous),
            token_expires_at: None,
            updated_at: previous,
        })
        .unwrap();
    harness
        .garmin
        .hrv_failure_status
        .store(429, Ordering::SeqCst);
    harness.pull(30);
    let state = harness.store.sync_state("wellness").unwrap().unwrap();
    assert_eq!(state.last_success_at, Some(previous));
    assert!(harness.status()["last_error"].is_string());
    assert!(harness
        .garmin
        .requests()
        .iter()
        .any(|(path, _)| path.starts_with(stride_ingest::HRV_DAILY_PATH)));
    assert!(!harness
        .garmin
        .requests()
        .iter()
        .any(|(path, _)| path == stride_ingest::USER_SUMMARY_PATH));
    let activities = harness.store.sync_state("activities").unwrap().unwrap();
    assert_eq!(activities.last_success_at, Some(instant()));
}

#[test]
fn history_continues_across_api_recreation_without_refetching_recent_days() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    assert_eq!(harness.pull(365)["more_pending"], true);
    let status = harness.status();
    assert_eq!(
        status["history"]["activities"]["checked_from"],
        "2026-04-29"
    );
    assert_eq!(status["history"]["wellness"]["next_date"], "2026-05-09");
    assert_eq!(harness.store.count_wellness_daily().unwrap(), 0);

    let api = Arc::new(Api::with_transport(
        harness.store.clone(),
        Config::default(),
        Arc::new(FixedClock::new(instant())),
        harness.garmin.clone(),
    ));
    api.set_garmin_key(GARMIN_KEY);
    let restarted = Router::new(api);
    let body = restarted
        .call(
            "POST",
            "/sync/run",
            Some("{\"days\":365,\"backfill\":true}"),
        )
        .body;
    assert_eq!(body["more_pending"], true);
    let status = restarted.call("GET", "/sync/status", None).body;
    assert_eq!(status["history"]["activities"]["next_date"], "2026-03-31");
    assert_eq!(status["history"]["wellness"]["next_date"], "2026-04-25");
    assert_eq!(harness.store.count_wellness_daily().unwrap(), 0);
    let searches: Vec<_> = harness
        .garmin
        .requests()
        .into_iter()
        .filter(|(path, _)| path == ACTIVITIES_PATH)
        .collect();
    assert!(searches[1].1.contains("endDate=2026-04-28"));
}

#[test]
fn no_data_is_checked_to_the_target_not_treated_as_end_of_history() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    harness.pull(30);
    let first = harness.backfill(30);
    assert_eq!(first["complete"], false);
    assert_eq!(first["more_pending"], true);
    let second = harness.backfill(30);
    assert_eq!(second["complete"], true);
    assert_eq!(second["more_pending"], false);
    assert_eq!(
        harness.status()["history"]["wellness"]["checked_from"],
        "2026-04-12"
    );
    assert_eq!(harness.store.count_wellness_daily().unwrap(), 0);
}

#[test]
fn a_request_budget_pauses_before_another_attempt_and_retries_the_unchecked_day() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::with_config(
        Vec::new(),
        fit,
        Config {
            max_sync_requests: 4,
            ..Config::default()
        },
    );
    let result = harness.pull(365);
    assert_eq!(harness.garmin.requests().len(), 4);
    assert_eq!(result["more_pending"], true);
    assert!(result["failure"].is_null());
    assert_eq!(
        harness.status()["history"]["wellness"]["next_date"],
        "2026-05-12"
    );
    assert!(harness
        .store
        .sync_state("wellness")
        .unwrap()
        .unwrap()
        .last_success_at
        .is_none());
    assert_eq!(
        harness.status()["history"]["activities"]["next_date"],
        "2026-04-28"
    );
    harness.backfill(365);
    assert_eq!(harness.garmin.requests().len(), 8);
    assert_eq!(
        harness.status()["history"]["wellness"]["next_date"],
        "2026-05-12"
    );
}

#[test]
fn a_byte_budget_stops_after_the_bounded_in_flight_response() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::with_config(
        Vec::new(),
        fit,
        Config {
            max_sync_response_bytes: 1,
            ..Config::default()
        },
    );
    let result = harness.pull(365);
    assert_eq!(result["more_pending"], true);
    assert!(result["failure"].is_null());
    assert_eq!(harness.garmin.requests().len(), 1);
    assert!(harness.status()["history"]["activities"]["checked_from"].is_null());
}

#[test]
fn a_partial_failure_is_explicit_and_survives_api_recreation() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    harness
        .garmin
        .hrv_failure_status
        .store(429, Ordering::SeqCst);
    let result = harness.pull(365);
    assert_eq!(result["failure"]["status"], 429);
    assert_eq!(result["complete"], false);
    let api = Arc::new(Api::with_transport(
        harness.store.clone(),
        Config::default(),
        Arc::new(FixedClock::new(instant())),
        harness.garmin.clone(),
    ));
    assert!(Router::new(api).call("GET", "/sync/status", None).body["last_error"].is_string());
}

#[test]
fn retry_after_survives_restart_and_blocks_another_provider_attempt() {
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    harness
        .garmin
        .hrv_failure_status
        .store(429, Ordering::SeqCst);
    harness
        .garmin
        .retry_after_seconds
        .store(120, Ordering::SeqCst);
    let result = harness.pull(30);
    assert_eq!(result["failure"]["retry_after_seconds"], 120);
    let before = harness.garmin.requests().len();
    let api = Arc::new(Api::with_transport(
        harness.store.clone(),
        Config::default(),
        Arc::new(FixedClock::new(instant())),
        harness.garmin.clone(),
    ));
    api.set_garmin_key(GARMIN_KEY);
    let answer = Router::new(api).call("POST", "/sync/run", Some("{\"days\":30}"));
    assert_eq!(answer.status, 429);
    assert_eq!(answer.body["code"], "rate_limited");
    assert_eq!(harness.garmin.requests().len(), before);
}

#[test]
fn a_capped_listing_narrows_the_dated_window_until_it_can_finish() {
    let (fit, distance, _) = decoded_truth();
    let listed = (0..300)
        .map(|index| {
            PullHarness::listed(
                &format!("{}", 1000 - index),
                distance,
                "2026-05-01T06:00:00Z",
            )
        })
        .collect();
    let harness = PullHarness::with_budget(listed, fit, 400);
    harness.garmin.filter_dates.store(true, Ordering::SeqCst);
    harness.pull(30);
    assert!(harness.status()["history"]["activities"]["checked_from"].is_null());
    harness.backfill(30);
    assert_eq!(
        harness.status()["history"]["activities"]["checked_from"],
        "2026-05-06"
    );
    let searches: Vec<_> = harness
        .garmin
        .requests()
        .into_iter()
        .filter(|(path, _)| path == ACTIVITIES_PATH)
        .collect();
    assert!(searches.last().unwrap().1.contains("startDate=2026-05-06"));
}

#[test]
fn a_single_day_that_cannot_be_enumerated_is_an_explicit_failure() {
    let (fit, distance, _) = decoded_truth();
    let listed = (0..300)
        .map(|index| {
            PullHarness::listed(
                &format!("{}", 1000 - index),
                distance,
                "2026-05-01T06:00:00Z",
            )
        })
        .collect();
    let harness = PullHarness::with_budget(listed, fit, 400);
    harness.garmin.filter_dates.store(true, Ordering::SeqCst);
    harness
        .store
        .write_sync_state(&stride_store::SyncState {
            kind: "activities".to_string(),
            cursor: Some(
                json!({
                    "newest": null, "pulled": 0, "skipped": 0, "deleted": 0,
                    "history": {
                        "target_from": "2026-04-12", "next_date": "2026-05-01",
                        "checked_from": "2026-05-02", "checked_to": "2026-05-12", "window_days": 1
                    }
                })
                .to_string(),
            ),
            last_success_at: None,
            token_expires_at: None,
            updated_at: instant(),
        })
        .unwrap();
    let result = harness.backfill(30);
    assert_eq!(result["failure"]["status"], 400);
    assert_eq!(result["complete"], false);
    assert_eq!(
        harness.status()["history"]["activities"]["next_date"],
        "2026-05-01"
    );
    assert_eq!(result["deleted"], 0);
}

#[test]
fn overlapping_callers_cannot_start_a_second_sync() {
    struct BlockingTransport {
        inner: Arc<SyncGarmin>,
        entered: Arc<Barrier>,
        release: Arc<Barrier>,
    }
    impl HttpTransport for BlockingTransport {
        fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
            if request.url.path() == TOKEN_PATH {
                self.entered.wait();
                self.release.wait();
            }
            self.inner.execute(request)
        }
    }
    let (fit, _, _) = decoded_truth();
    let harness = PullHarness::new(Vec::new(), fit);
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let api = Arc::new(Api::with_transport(
        harness.store.clone(),
        Config::default(),
        Arc::new(FixedClock::new(instant())),
        Arc::new(BlockingTransport {
            inner: harness.garmin.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
    ));
    api.set_garmin_key(GARMIN_KEY);
    let router = Arc::new(Router::new(api));
    let first = router.clone();
    let owner = std::thread::spawn(move || first.call("POST", "/sync/run", Some("{\"days\":30}")));
    entered.wait();
    let duplicate = router.call("POST", "/sync/run", Some("{\"days\":30}"));
    release.wait();
    assert_eq!(duplicate.status, 409);
    assert_eq!(duplicate.body["code"], "sync_busy");
    assert!(owner.join().unwrap().is_ok());
}
