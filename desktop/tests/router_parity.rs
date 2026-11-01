//! The rig and the router, byte for byte.
//!
//! **The rig is a host.** It opens the door, it decides who is let in, and
//! it serves the app. It does not have an opinion about a number, and it
//! must not reshape one.
//!
//! So the bytes the rig puts on the wire are the router's bytes: the same
//! DTO, serialized once, by one implementation. The phone's answer is the
//! same router's answer — and `stride-ffi`'s own test holds the phone to
//! the same rule. Between the two, the two hosts cannot disagree about a
//! number, because there is only one of it.
//!
//! This test also holds the **key** to its promise: with a distinctive key
//! in play, no response and no log line carries it.

use std::sync::Arc;

use stride_api::{Api, Config, FixedClock, Router as CoreRouter};
use stride_desktop::api::{ApiRouter, Request, Router};
use stride_desktop::key::KeyStore;
use stride_desktop::server::{DesktopHost, Server};
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;
use stride_store::{DatabaseKey, Store};

/// A key that is recognisable if it ever turns up anywhere.
///
/// It is not credential-shaped on purpose: this is a test fixture, and the
/// repository's secret scanner would (rightly) refuse a string that looked
/// like a real Garmin key.
const THE_GARMIN_KEY: &str = "stride-garmin-key-4f2c9a7e1b6d8305-do-not-print";

/// The result of a test: anything that went wrong is an error.
type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The instant the clock stands at, so that every answer is the same.
fn the_instant() -> TestResult<chrono::DateTime<chrono::Utc>> {
    Ok(chrono::DateTime::parse_from_rfc3339("2026-05-12T06:30:00Z")?.into())
}

/// A store in a scratch directory.
///
/// The name is unique per call: two tests that opened the same file at the
/// same time would fight over it, and the migration is not a thing to race.
fn store(name: &str) -> TestResult<Arc<Store>> {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let directory = std::env::temp_dir().join(format!("stride-{name}-{unique}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    let database = directory.join("stride.db");
    let key = DatabaseKey::from_bytes([7u8; 32])?;
    Ok(Arc::new(Store::open_and_migrate(&database, &key)?))
}

/// The router, over a store, and the store itself.
fn router(name: &str) -> TestResult<(CoreRouter, Arc<Store>)> {
    let store = store(name)?;
    let api = Api::new(
        Arc::clone(&store),
        Config::default(),
        Arc::new(FixedClock::new(the_instant()?)),
    )
    .map_err(|_| stride_desktop::error::Error::config("the Garmin client could not be built"))?;
    Ok((CoreRouter::new(Arc::new(api)), store))
}

/// The rig's answer, and the router's answer, for one request.
///
/// Both are **documents**: the router's body as it is, and the rig's body
/// parsed back out of the bytes it put on the wire. Comparing the two
/// documents is comparing the answers — the fields, the names, the
/// nullability and the values.
fn both_answers(
    method: &str,
    path: &str,
    body: &[u8],
) -> TestResult<(serde_json::Value, serde_json::Value)> {
    let (core, store) = router("router-parity")?;
    let keys = Arc::new(KeyStore::empty());
    let host = DesktopHost::new(Arc::clone(&store), Arc::clone(&keys));

    let core_answer = core.call(method, path, Some(std::str::from_utf8(body)?));

    let rig = ApiRouter::with_clock(&host, Arc::new(FixedClock::new(the_instant()?)))?;
    let rig_answer = rig.call(&Request {
        method,
        path,
        query: "",
        body,
    });
    let rig_document: serde_json::Value = serde_json::from_slice(&rig_answer.body)?;

    Ok((core_answer.body, rig_document))
}

#[test]
fn the_rig_answers_the_router_answer() -> TestResult<()> {
    let (core_body, rig_body) = both_answers("GET", "/api/v1/activities", b"")?;
    assert_eq!(rig_body, core_body, "the rig reshaped the router's answer");
    Ok(())
}

#[test]
fn a_failure_is_the_router_failure() -> TestResult<()> {
    let (core_body, rig_body) = both_answers("GET", "/api/v1/there-is-no-such-route", b"")?;
    assert_eq!(rig_body, core_body);
    Ok(())
}

#[test]
fn the_key_is_in_no_answer_and_no_log_line() -> TestResult<()> {
    let lines: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink_lines = Arc::clone(&lines);
    stride_desktop::log::set_sink(Arc::new(move |line: &str| {
        if let Ok(mut guard) = sink_lines.lock() {
            guard.push(line.to_string());
        }
    }));

    let (_, store) = router("router-parity-key")?;
    let keys = Arc::new(KeyStore::empty());
    keys.set(THE_GARMIN_KEY)?;
    let host = DesktopHost::new(Arc::clone(&store), Arc::clone(&keys));
    let rig = Arc::new(ApiRouter::over(&host)?);

    let server = Server::new(Trust::loopback_only(), rig, Spa::new(None));

    // Every kind of request the rig answers, with the key in play.
    let requests = [
        ("GET", "/api/v1/activities", "".to_string()),
        ("GET", "/api/v1/auth/status", "".to_string()),
        (
            "POST",
            "/api/v1/auth/garmin/personal-key",
            format!("{{\"key\":\"{THE_GARMIN_KEY}\"}}"),
        ),
        (
            "GET",
            &format!("/api/v1/auth/status?key={THE_GARMIN_KEY}"),
            "".to_string(),
        ),
    ];
    for (method, path, body) in requests {
        let request = stride_desktop::http::Request {
            method: method.to_string(),
            target: path.to_string(),
            path: path.to_string(),
            query: "".to_string(),
            headers: Vec::new(),
            body: body.into_bytes(),
        };
        let response = server.handle(&request, std::net::IpAddr::from([127, 0, 0, 1]));
        let text = String::from_utf8_lossy(&response.body).into_owned();
        assert!(
            !text.contains(THE_GARMIN_KEY),
            "an answer carried the key: {text}"
        );
    }

    if let Ok(guard) = lines.lock() {
        for line in guard.iter() {
            assert!(
                !line.contains(THE_GARMIN_KEY),
                "a log line carried the key: {line}"
            );
        }
    }
    stride_desktop::log::clear_sink();
    Ok(())
}
