//! Garmin client tests against an in-memory transport.
//!
//! No test opens a socket and no test uses a real Personal API key.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::VecDeque;
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use stride_ingest::{
    GarminClient, GarminError, HttpMethod, HttpRequest, HttpResponse, HttpTransport, TransportError,
};

const PERSONAL_KEY: &str = "unit-test-personal-value";
const FIRST_ACCESS: &str = "first-access-value";

#[derive(Debug, Clone)]
struct RecordedRequest {
    method: HttpMethod,
    url: String,
    auth_value: Option<String>,
}

struct MockTransport {
    responses: Mutex<VecDeque<HttpResponse>>,
    requests: Mutex<Vec<RecordedRequest>>,
}

impl MockTransport {
    fn new(responses: Vec<HttpResponse>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn request_count(&self) -> usize {
        self.requests.lock().expect("request lock").len()
    }

    fn requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().expect("request lock").clone()
    }
}

impl HttpTransport for MockTransport {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        self.requests
            .lock()
            .expect("request lock")
            .push(RecordedRequest {
                method: request.method,
                url: request.url.to_string(),
                auth_value: request.auth_value.clone(),
            });
        self.responses
            .lock()
            .expect("response lock")
            .pop_front()
            .ok_or_else(|| TransportError::new("the mock has no response left"))
    }
}

fn response(status: u16, body: impl Into<Vec<u8>>) -> HttpResponse {
    HttpResponse {
        status,
        headers: Vec::new(),
        body: body.into(),
    }
}

fn json_response(status: u16, value: serde_json::Value) -> HttpResponse {
    response(
        status,
        serde_json::to_vec(&value).expect("the mock body is JSON"),
    )
}

fn redirect(location: &str) -> HttpResponse {
    HttpResponse {
        status: 302,
        headers: vec![("Location".to_owned(), location.to_owned())],
        body: Vec::new(),
    }
}

fn client(responses: Vec<HttpResponse>) -> (GarminClient, Arc<MockTransport>) {
    let transport = Arc::new(MockTransport::new(responses));
    let client = GarminClient::with_transport(PERSONAL_KEY, transport.clone());
    (client, transport)
}

#[test]
fn the_key_is_exchanged_once_and_the_access_value_is_reused() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        json_response(200, json!({"displayName": "ATHLETE"})),
        json_response(200, json!({"displayName": "ATHLETE"})),
    ]);

    client.profile().expect("the first read succeeds");
    client.profile().expect("the second read succeeds");

    assert_eq!(client.exchange_count(), 1);
    assert_eq!(client.request_count(), 3);
    let requests = transport.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, HttpMethod::Post);
    assert_eq!(
        requests[0].auth_value.as_deref(),
        Some(format!("ApiKey {PERSONAL_KEY}").as_str())
    );
    assert_eq!(
        requests[1].auth_value.as_deref(),
        Some("Bearer first-access-value")
    );
    assert_eq!(
        requests[2].auth_value.as_deref(),
        Some("Bearer first-access-value")
    );
}

#[test]
fn a_401_returns_without_re_exchange_or_retry() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        response(401, "expired"),
        // These responses prove the client does not consume a second
        // exchange or a second data attempt.
        response(200, "unused-access-value"),
        json_response(200, json!({"displayName": "ATHLETE"})),
    ]);

    let error = client
        .profile()
        .expect_err("the 401 is returned to the shell");
    assert!(matches!(error, GarminError::NeedsReconnect));
    assert_eq!(client.exchange_count(), 1);
    assert_eq!(client.request_count(), 2);
    assert_eq!(transport.request_count(), 2);
    let requests = transport.requests();
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == HttpMethod::Post)
            .count(),
        1
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == HttpMethod::Get)
            .count(),
        1
    );
}

#[test]
fn a_successful_call_after_a_fresh_exchange_is_not_retried() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        json_response(200, json!({"displayName": "ATHLETE"})),
    ]);

    client.profile().expect("the call succeeds");

    assert_eq!(client.exchange_count(), 1);
    assert_eq!(client.request_count(), 2);
    assert_eq!(transport.request_count(), 2);
}

#[test]
fn an_expired_cached_access_is_refreshed_before_the_next_call() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        json_response(200, json!({"displayName": "ATHLETE"})),
        response(200, "fresh-access-value"),
        json_response(200, json!({"displayName": "ATHLETE"})),
    ]);
    let client = client.with_access_ttl(Duration::ZERO);

    client.profile().expect("the first call succeeds");
    client.profile().expect("the second call succeeds");

    // The refresh happens before the second call, not as a retry of the first.
    assert_eq!(client.exchange_count(), 2);
    assert_eq!(client.request_count(), 4);
    assert_eq!(transport.request_count(), 4);
}

#[test]
fn a_403_does_not_re_exchange_the_key() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        response(403, "scope denied"),
    ]);

    let error = client.profile().expect_err("a scope failure is fatal");
    assert!(matches!(error, GarminError::Forbidden));
    assert_eq!(client.exchange_count(), 1);
    assert_eq!(transport.request_count(), 2);
}

#[test]
fn the_key_never_appears_in_debug_or_an_error() {
    let (client, _) = client(vec![
        response(200, FIRST_ACCESS),
        response(500, format!("the value {PERSONAL_KEY} was rejected")),
    ]);
    let error = client.profile().expect_err("Garmin returns an error");
    assert!(!error.to_string().contains(PERSONAL_KEY));
    assert!(!format!("{error:?}").contains(PERSONAL_KEY));
    assert!(!format!("{client:?}").contains(PERSONAL_KEY));
}

#[test]
fn a_redirect_outside_garmin_is_refused() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        redirect("https://outside.example/collect"),
    ]);

    let error = client.profile().expect_err("the redirect is refused");
    assert!(matches!(error, GarminError::UntrustedHost { .. }));
    assert_eq!(transport.request_count(), 2);
}

#[test]
fn a_redirect_inside_the_allowed_set_is_followed_without_the_old_header() {
    let (client, transport) = client(vec![
        response(200, FIRST_ACCESS),
        redirect("https://connect.garmin.com/profile"),
        json_response(200, json!({"displayName": "ATHLETE"})),
    ]);

    client.profile().expect("the allowed redirect is followed");
    let requests = transport.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[2].url, "https://connect.garmin.com/profile");
    assert_eq!(requests[2].auth_value, None);
}

#[test]
fn a_response_over_five_mebibytes_is_refused() {
    let oversized = vec![0_u8; 5 * 1024 * 1024 + 1];
    let (client, _) = client(vec![response(200, FIRST_ACCESS), response(200, oversized)]);

    let error = client
        .profile()
        .expect_err("the oversized response is refused");
    assert!(matches!(error, GarminError::ResponseTooLarge { .. }));
}

#[test]
fn invalid_activity_ids_never_reach_the_transport() {
    let (client, transport) = client(Vec::new());
    let too_long = "x".repeat(21);
    for invalid in ["../etc/passwd", "12a", "", too_long.as_str()] {
        let error = client
            .activity_detail(invalid)
            .expect_err("the invalid id is refused");
        assert!(matches!(error, GarminError::InvalidActivityId(_)));
        let error = client
            .activity_fit(invalid)
            .expect_err("the invalid id is refused");
        assert!(matches!(error, GarminError::InvalidActivityId(_)));
    }
    assert_eq!(transport.request_count(), 0);
}

#[test]
fn every_required_endpoint_uses_the_verified_path() {
    let mut responses = vec![response(200, FIRST_ACCESS)];
    for _ in 0..13 {
        responses.push(json_response(200, json!({})));
    }
    let (client, transport) = client(responses);

    client
        .activity_search(0, 20, Some("2026-03-01"), Some("2026-03-07"))
        .expect("activity search succeeds");
    client
        .activity_detail("42")
        .expect("activity detail succeeds");
    client.sleep_daily("2026-03-07").expect("sleep succeeds");
    client
        .sleep_summary("2026-03-07")
        .expect("sleep summary succeeds");
    client.hrv_daily("2026-03-07").expect("daily HRV succeeds");
    client
        .hrv_range("2026-03-01", "2026-03-07")
        .expect("HRV range succeeds");
    client
        .wellness_daily_heart_rate("2026-03-07")
        .expect("daily heart rate succeeds");
    client
        .body_battery("2026-03-01", "2026-03-07")
        .expect("body battery succeeds");
    client
        .user_summary_daily("2026-03-07")
        .expect("daily summary succeeds");
    client
        .stress("2026-03-01", "2026-03-07")
        .expect("stress succeeds");
    client
        .steps("2026-03-01", "2026-03-07")
        .expect("steps succeeds");
    client.heart_rate_zones().expect("zones succeed");
    client.profile().expect("profile succeeds");

    let paths: Vec<String> = transport
        .requests()
        .into_iter()
        .map(|request| request.url)
        .collect();
    assert!(paths
        .iter()
        .any(|path| path.contains("activitylist-service/activities/search/activities")));
    assert!(paths
        .iter()
        .any(|path| path.contains("activity-service/activity/42")));
    assert!(paths
        .iter()
        .any(|path| path.contains("sleep-service/sleep/dailySleep")));
    assert!(paths
        .iter()
        .any(|path| path.contains("sleep-service/sleep/dailySleepSummariesByDate")));
    assert!(paths
        .iter()
        .any(|path| path.contains("hrv-service/hrv/2026-03-07")));
    assert!(paths
        .iter()
        .any(|path| path.contains("hrv-service/hrv/daily/2026-03-01/2026-03-07")));
    assert!(paths
        .iter()
        .any(|path| path.contains("wellness-service/wellness/dailyHeartRate")));
    assert!(paths
        .iter()
        .any(|path| path.contains("wellness-service/wellness/bodyBattery/reports/daily")));
    assert!(paths
        .iter()
        .any(|path| path.contains("usersummary-service/usersummary/daily")));
    assert!(paths
        .iter()
        .any(|path| path.contains("usersummary-service/stats/stress/daily")));
    assert!(paths
        .iter()
        .any(|path| path.contains("usersummary-service/stats/steps/daily")));
    assert!(paths
        .iter()
        .any(|path| path.contains("biometric-service/heartRateZones/v2")));
    assert!(paths
        .iter()
        .any(|path| path.contains("userprofile-service/userprofile/settings")));
}

fn zip_with(entry_name: &str, content: &[u8]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(entry_name, options)
            .expect("the member starts");
        writer.write_all(content).expect("the member is written");
        writer.finish().expect("the archive is finished");
    }
    buffer.into_inner()
}

#[test]
fn a_fit_member_is_extracted_from_the_download_archive() {
    let fit = [0x0e, 0x10, 0x00, 0x01, 0x02, 0x03];
    let (client, _) = client(vec![
        response(200, FIRST_ACCESS),
        response(200, zip_with("42_ACTIVITY.fit", &fit)),
    ]);

    let bytes = client.activity_fit("42").expect("the FIT member is found");
    assert_eq!(bytes, fit);
}

#[test]
fn an_archive_with_an_unsafe_member_name_is_refused() {
    let (client, _) = client(vec![
        response(200, FIRST_ACCESS),
        response(200, zip_with("../escape.fit", &[0x0e, 0x10])),
    ]);

    let error = client
        .activity_fit("42")
        .expect_err("the archive is refused");
    assert!(matches!(error, GarminError::FitUnavailable { .. }));
}

#[test]
fn an_archive_with_too_many_members_is_refused() {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for index in 0..9 {
            writer
                .start_file(format!("member-{index}.txt"), options)
                .expect("the member starts");
            writer.write_all(b"x").expect("the member is written");
        }
        writer.finish().expect("the archive is finished");
    }
    let (client, _) = client(vec![
        response(200, FIRST_ACCESS),
        response(200, buffer.into_inner()),
    ]);

    let error = client
        .activity_fit("42")
        .expect_err("the archive is refused");
    assert!(matches!(error, GarminError::FitUnavailable { .. }));
}
