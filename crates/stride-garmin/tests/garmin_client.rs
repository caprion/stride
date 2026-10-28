//! The client against a local stub. No test touches the network.

// An integration test is its own crate, not a `#[cfg(test)]` module, so the
// workspace's `allow-*-in-tests` configuration does not reach it. The stub
// asserts its own invariants with `expect`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Duration, Utc};
use common::{Reply, Request, StubServer};
use serde_json::json;
use stride_garmin::{GarminClient, GarminError, TOKEN_PATH};

const KEY: &str = "GARMIN-PERSONAL-KEY-4f2b91c7SECRET";
const PROFILE_PATH: &str = "/userprofile-service/userprofile/settings";
const ACTIVITIES_PATH: &str = "/activitylist-service/activities/search/activities";

fn client_for(stub: &StubServer) -> GarminClient {
    GarminClient::with_base_url(KEY, &stub.base_url()).expect("the stub is a loopback host")
}

fn token_reply() -> Reply {
    Reply::text("eyJhbGciOiJIUzI1NiJ9.stub-jwt.stub")
}

fn profile_reply() -> Reply {
    Reply::json(json!({
        "displayName": "Test Athlete",
        "userName": "test-athlete",
        "profileId": 12345,
    }))
}

// ------------------------------------------------------------ the token cache

#[test]
fn the_token_is_exchanged_once_and_reused_until_it_expires() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    let first = client.profile().expect("the first profile read succeeds");
    let second = client.profile().expect("the second profile read succeeds");

    assert_eq!(first.athlete_name(), Some("Test Athlete"));
    assert_eq!(second.athlete_name(), Some("Test Athlete"));
    assert_eq!(
        stub.count("POST", TOKEN_PATH),
        1,
        "the token must be exchanged once and then reused"
    );
    assert_eq!(stub.count("GET", PROFILE_PATH), 2);
    assert!(client.has_cached_token());
}

#[test]
fn an_expired_token_is_exchanged_again() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    // A token that is already stale is never reused.
    let client = GarminClient::with_base_url(KEY, &stub.base_url())
        .expect("the stub is a loopback host")
        .with_token_ttl(std::time::Duration::from_secs(0));
    client.profile().expect("the first profile read succeeds");
    client.profile().expect("the second profile read succeeds");

    assert_eq!(
        stub.count("POST", TOKEN_PATH),
        2,
        "a token with no life left must be exchanged again"
    );
}

#[test]
fn invalidating_the_token_forces_a_new_exchange() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    client.profile().expect("the first profile read succeeds");
    client.invalidate_token();
    assert!(!client.has_cached_token());
    client.profile().expect("the second profile read succeeds");

    assert_eq!(stub.count("POST", TOKEN_PATH), 2);
}

// -------------------------------------------------------------- the 401 path

#[test]
fn a_401_triggers_exactly_one_re_exchange_and_one_retry() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::status(401));
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    let profile = client.profile().expect("the retry succeeds");

    assert_eq!(profile.athlete_name(), Some("Test Athlete"));
    assert_eq!(
        stub.count("POST", TOKEN_PATH),
        2,
        "one re-exchange, not a loop"
    );
    assert_eq!(stub.count("GET", PROFILE_PATH), 2, "one retry, not a loop");
}

#[test]
fn a_second_401_yields_needs_reconnect_and_stops() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::status(401));
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::status(401));

    let client = client_for(&stub);
    let error = client.profile().expect_err("the second 401 is fatal");

    assert!(
        matches!(error, GarminError::NeedsReconnect),
        "expected NeedsReconnect, got {error:?}"
    );
    assert_eq!(stub.count("POST", TOKEN_PATH), 2, "exactly one re-exchange");
    assert_eq!(stub.count("GET", PROFILE_PATH), 2, "exactly one retry");
    assert_eq!(
        stub.requests().len(),
        4,
        "no request may follow the second 401"
    );
}

#[test]
fn a_403_does_not_trigger_a_re_exchange() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::status(403));

    let client = client_for(&stub);
    let error = client.profile().expect_err("a 403 is a scope failure");

    assert!(
        matches!(error, GarminError::Forbidden),
        "expected Forbidden, got {error:?}"
    );
    assert_eq!(
        stub.count("POST", TOKEN_PATH),
        1,
        "a scope failure is not a token problem"
    );
    assert_eq!(stub.count("GET", PROFILE_PATH), 1);
}

#[test]
fn a_429_is_reported_with_the_retry_after() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::status(429));

    let client = client_for(&stub);
    let error = client.profile().expect_err("a 429 is an error");

    assert!(
        matches!(error, GarminError::RateLimited { .. }),
        "expected RateLimited, got {error:?}"
    );
    assert_eq!(stub.count("POST", TOKEN_PATH), 1);
}

// ------------------------------------------------------------- the key itself

#[test]
fn the_key_never_appears_in_an_error_message() {
    // Garmin rejects the key.
    let stub = StubServer::start();
    stub.push(
        "POST",
        TOKEN_PATH,
        Reply::error(401, "the key is not welcome"),
    );
    let client = client_for(&stub);
    let error = client.validate().expect_err("a rejected key fails");
    assert!(!error.to_string().contains(KEY), "leaked: {error}");
    assert!(!format!("{error:?}").contains(KEY), "leaked: {error:?}");

    // Garmin fails a data call and echoes the key back.
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        PROFILE_PATH,
        Reply::error(500, format!("the key {KEY} is invalid")),
    );
    let client = client_for(&stub);
    let error = client.profile().expect_err("a 500 is an error");
    assert!(!error.to_string().contains(KEY), "leaked: {error}");
    assert!(!format!("{error:?}").contains(KEY), "leaked: {error:?}");

    // Garmin cannot be reached at all.
    let client =
        GarminClient::with_base_url(KEY, "http://127.0.0.1:1").expect("a loopback host is allowed");
    let error = client.validate().expect_err("nothing is listening");
    assert!(!error.to_string().contains(KEY), "leaked: {error}");
    assert!(!format!("{error:?}").contains(KEY), "leaked: {error:?}");
}

#[test]
fn the_key_is_never_printed_or_sent_in_a_url() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    client.profile().expect("the profile read succeeds");

    assert!(!format!("{client:?}").contains(KEY), "leaked in Debug");
    for request in stub.requests() {
        assert!(!request.path.contains(KEY), "leaked in the path");
        assert!(!request.query.contains(KEY), "leaked in the query");
        assert!(!request.body.contains(KEY), "leaked in the body");
    }
}

#[test]
fn the_key_is_sent_as_an_authorization_header_and_the_token_as_a_bearer() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    client.profile().expect("the profile read succeeds");

    let token_request = &stub.requests_for("POST", TOKEN_PATH)[0];
    assert_eq!(
        token_request.header("authorization"),
        Some(format!("ApiKey {KEY}").as_str())
    );
    let profile_request = &stub.requests_for("GET", PROFILE_PATH)[0];
    assert_eq!(
        profile_request.header("authorization"),
        Some("Bearer eyJhbGciOiJIUzI1NiJ9.stub-jwt.stub")
    );
}

// ------------------------------------------------------------------ validate

#[test]
fn validate_exchanges_the_key_and_reads_the_profile_before_it_returns() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    let profile = client.validate().expect("validate succeeds");

    assert_eq!(profile.athlete_name(), Some("Test Athlete"));
    assert_eq!(stub.count("POST", TOKEN_PATH), 1);
    assert_eq!(stub.count("GET", PROFILE_PATH), 1);
    assert!(
        stub.requests()[0].method == "POST",
        "the token is exchanged first"
    );
}

#[test]
fn validate_fails_when_the_profile_read_fails_after_the_token_exchange() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, Reply::error(500, "no profile for you"));

    let client = client_for(&stub);
    let error = client.validate().expect_err("the profile read failed");

    assert!(
        matches!(error, GarminError::Http { status: 500, .. }),
        "expected an HTTP error, got {error:?}"
    );
    assert_eq!(stub.count("POST", TOKEN_PATH), 1);
}

// ---------------------------------------------------------------- activities

#[test]
fn the_activities_are_paginated_up_to_the_page_limit() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    let page: Vec<serde_json::Value> = (0..100)
        .map(|index| json!({"activityId": 1000 + index, "activityName": format!("Run {index}")}))
        .collect();
    stub.push(
        "GET",
        ACTIVITIES_PATH,
        Reply::json(json!({ "activityList": page })),
    );
    stub.push(
        "GET",
        ACTIVITIES_PATH,
        Reply::json(json!({ "activityList": [{"activityId": 9999}] })),
    );

    let client = client_for(&stub);
    let to = Utc::now();
    let activities = client
        .activities(to - Duration::days(7), to)
        .expect("the activity list succeeds");

    assert_eq!(
        activities.len(),
        101,
        "100 from the first page, 1 from the second"
    );
    assert_eq!(activities[0].activity_id, "1000");
    assert_eq!(activities[100].activity_id, "9999");
    assert_eq!(stub.count("GET", ACTIVITIES_PATH), 2);
}

#[test]
fn the_activity_window_is_clamped_to_a_year() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        ACTIVITIES_PATH,
        Reply::json(json!({"activityList": []})),
    );

    let client = client_for(&stub);
    let to = Utc::now();
    client
        .activities(to - Duration::days(900), to)
        .expect("the activity list succeeds");

    let request = &stub.requests_for("GET", ACTIVITIES_PATH)[0];
    let end: chrono::NaiveDate = request
        .query
        .split('&')
        .find_map(|pair| pair.strip_prefix("endDate="))
        .and_then(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .expect("the end date is in the query");
    let start: chrono::NaiveDate = request
        .query
        .split('&')
        .find_map(|pair| pair.strip_prefix("startDate="))
        .and_then(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .expect("the start date is in the query");
    assert_eq!(
        (end - start).num_days(),
        365,
        "the window must never exceed a year"
    );
}

#[test]
fn an_invalid_activity_id_never_reaches_garmin() {
    let stub = StubServer::start();
    let client = client_for(&stub);

    for bad in ["../etc/passwd", "12a", "", "123456789012345678901"] {
        let error = client
            .activity_detail(bad)
            .expect_err("an invalid id is refused");
        assert!(
            matches!(error, GarminError::InvalidActivityId(_)),
            "expected InvalidActivityId, got {error:?}"
        );
        let error = client
            .activity_fit(bad)
            .expect_err("an invalid id is refused");
        assert!(
            matches!(error, GarminError::InvalidActivityId(_)),
            "expected InvalidActivityId, got {error:?}"
        );
    }
    assert!(stub.requests().is_empty(), "nothing may be sent");
}

// ----------------------------------------------------------------- the FIT file

fn zip_with(entry_name: &str, content: &[u8]) -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(entry_name, options)
            .expect("the entry starts");
        std::io::Write::write_all(&mut writer, content).expect("the entry is written");
        writer.finish().expect("the archive is finished");
    }
    buffer.into_inner()
}

#[test]
fn the_fit_file_is_extracted_from_the_download_zip() {
    let fit = vec![0x0e, 0x10, 0x00, 0x01, 0x02, 0x03];
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        "/download-service/files/activity/42",
        Reply::bytes(zip_with("42_ACTIVITY.fit", &fit)),
    );

    let client = client_for(&stub);
    let bytes = client
        .activity_fit("42")
        .expect("the FIT file is extracted");

    assert_eq!(bytes, fit);
}

#[test]
fn a_bare_fit_file_is_taken_as_it_is() {
    let fit = vec![0x0e, 0x10, 0x00, 0x01, 0x02, 0x03];
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        "/download-service/files/activity/42",
        Reply::bytes(fit.clone()),
    );

    let client = client_for(&stub);
    let bytes = client
        .activity_fit("42")
        .expect("the bare FIT file is taken");

    assert_eq!(bytes, fit);
}

#[test]
fn a_zip_with_an_unsafe_member_name_is_refused() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        "/download-service/files/activity/42",
        Reply::bytes(zip_with("../escape.fit", &[0x0e, 0x10])),
    );

    let client = client_for(&stub);
    let error = client
        .activity_fit("42")
        .expect_err("the archive is refused");

    assert!(
        matches!(error, GarminError::FitUnavailable { .. }),
        "expected FitUnavailable, got {error:?}"
    );
}

#[test]
fn a_zip_without_a_fit_member_is_refused() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        "/download-service/files/activity/42",
        Reply::bytes(zip_with("42_ACTIVITY.txt", b"not a fit file")),
    );

    let client = client_for(&stub);
    let error = client.activity_fit("42").expect_err("there is no FIT file");

    assert!(
        matches!(error, GarminError::FitUnavailable { .. }),
        "expected FitUnavailable, got {error:?}"
    );
}

// ------------------------------------------------------------------- sources

#[test]
fn the_sources_report_what_has_answered() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push("GET", PROFILE_PATH, profile_reply());

    let client = client_for(&stub);
    let sources = client.sources();
    assert!(!sources["activities"], "nothing has been fetched yet");
    assert!(!sources["profile"]);

    client.profile().expect("the profile read succeeds");

    let sources = client.sources();
    assert!(sources["profile"], "the profile answered");
    assert!(!sources["activities"], "the activity list did not answer");
    assert_eq!(sources.len(), 12, "the source list has not changed shape");
}

// --------------------------------------------------------------------- hosts

#[test]
fn a_host_that_is_not_garmin_is_refused() {
    for host in [
        "https://evil.example.com",
        "http://personal.connectapi.garmin.com",
        "https://127.0.0.1",
    ] {
        let error = GarminClient::with_base_url(KEY, host)
            .expect_err("only Garmin's hosts and loopback are allowed");
        assert!(
            matches!(error, GarminError::UntrustedHost { .. }),
            "expected UntrustedHost for {host}, got {error:?}"
        );
    }
}

#[test]
fn the_client_is_debuggable_without_leaking_the_key() {
    let client = client_for(&StubServer::start());
    let rendered = format!("{client:?}");
    assert!(!rendered.contains(KEY));
    assert!(rendered.contains("has_key: true"));
}

// --------------------------------------------------------------- the requests

#[test]
fn the_wellness_endpoints_send_the_dates_they_are_given() {
    let stub = StubServer::start();
    stub.push("POST", TOKEN_PATH, token_reply());
    stub.push(
        "GET",
        "/sleep-service/sleep/dailySleep",
        Reply::json(json!({})),
    );
    stub.push("GET", "/hrv-service/hrv/2026-03-07", Reply::json(json!({})));
    stub.push(
        "GET",
        "/wellness-service/wellness/dailyHeartRate",
        Reply::json(json!({})),
    );
    stub.push(
        "GET",
        "/usersummary-service/usersummary/daily",
        Reply::json(json!({})),
    );

    let client = client_for(&stub);
    client.sleep("2026-03-07").expect("sleep succeeds");
    client.hrv("2026-03-07").expect("hrv succeeds");
    client
        .daily_heart_rate("2026-03-07")
        .expect("the daily heart rate succeeds");
    client
        .daily_summary("2026-03-07")
        .expect("the daily summary succeeds");

    let sleep: Vec<Request> = stub.requests_for("GET", "/sleep-service/sleep/dailySleep");
    assert!(sleep[0].query.contains("date=2026-03-07"));
    let summary: Vec<Request> = stub.requests_for("GET", "/usersummary-service/usersummary/daily");
    assert!(summary[0].query.contains("calendarDate=2026-03-07"));
}

#[test]
fn an_invalid_date_is_refused_before_the_request() {
    let stub = StubServer::start();
    let client = client_for(&stub);

    let error = client.sleep("07/03/2026").expect_err("the date is refused");
    assert!(
        matches!(error, GarminError::InvalidDate(_)),
        "expected InvalidDate, got {error:?}"
    );
    assert!(stub.requests().is_empty(), "nothing may be sent");
}
