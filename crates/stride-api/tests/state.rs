//! The application state: the credentials, the pairing code and the devices.
//!
//! The state is not in the database: the Python keeps it in `state.json` and
//! the phone keeps it in the Android preferences. These tests drive the
//! decorator that owns it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::Value;
use stride_api::{Api, MemBackend, StateBackend};

/// The router, over an in-memory store and a fresh state.
fn api() -> Api {
    let backend = StateBackend::new(Arc::new(MemBackend::new()));
    Api::new(Arc::new(backend))
}

/// The status the browser sees.
#[test]
fn the_status_carries_the_defaults() {
    let api = api();
    let status = api
        .call("GET", "/api/v1/auth/status", None)
        .expect("the status answers");
    assert_eq!(status["garmin_connected"], false);
    assert_eq!(status["garmin_method"], Value::Null);
    assert_eq!(status["athlete"], serde_json::json!({}));
    assert_eq!(status["has_personal_api_key"], false);
    assert_eq!(status["network_route"], "auto");
    assert_eq!(status["sync_days"], 90);
    assert_eq!(status["history_months"], 12);
    assert_eq!(status["auto_sync"], true);
    assert_eq!(status["demo_visible"], true);
}

/// A pairing code is minted, used once, and the device is listed.
#[test]
fn a_pairing_code_is_single_use() {
    let api = api();
    let minted = api
        .call("GET", "/api/v1/auth/pair-code", None)
        .expect("the code answers");
    let code = minted["code"].as_str().expect("a code");
    assert_eq!(code.len(), 9, "the code is XXXX-XXXX");
    assert_eq!(code.as_bytes()[4], b'-');
    assert_eq!(minted["expires_in"], 600);

    // The wrong code is refused.
    let refused = api
        .call(
            "POST",
            "/api/v1/auth/pair",
            Some(r#"{"code": "ZZZZ-ZZZZ"}"#),
        )
        .expect_err("401");
    assert_eq!(refused.status, 401);
    assert_eq!(
        refused.detail,
        "That code is wrong, expired, or already used."
    );

    // The right code pairs a device.
    let paired = api
        .call(
            "POST",
            "/api/v1/auth/pair",
            Some(&format!(r#"{{"code": "{code}"}}"#)),
        )
        .expect("the pair answers");
    assert_eq!(paired["ok"], true);
    let device_id = paired["device_id"].as_str().expect("an id");
    assert!(!paired["token"].as_str().expect("a token").is_empty());

    // The device is listed, without its token.
    let devices = api
        .call("GET", "/api/v1/auth/devices", None)
        .expect("the devices answer");
    assert_eq!(devices["devices"].as_array().map(Vec::len), Some(1));
    assert_eq!(devices["devices"][0]["id"], device_id);
    assert!(devices["devices"][0].get("token").is_none());

    // The code is spent: the same code does not pair a second device.
    let again = api
        .call(
            "POST",
            "/api/v1/auth/pair",
            Some(&format!(r#"{{"code": "{code}"}}"#)),
        )
        .expect_err("401");
    assert_eq!(again.status, 401);

    // The device can be revoked.
    let revoked = api
        .call("DELETE", &format!("/api/v1/auth/devices/{device_id}"), None)
        .expect("the revoke answers");
    assert_eq!(revoked["ok"], true);
    assert_eq!(revoked["remaining"], 0);
}

/// The sync settings are the state's, and they change.
#[test]
fn the_settings_change() {
    let api = api();
    let status = api
        .call(
            "PATCH",
            "/api/v1/sync/settings",
            Some(r#"{"sync_days": 30, "auto_sync": false, "sync_interval_minutes": 15}"#),
        )
        .expect("the settings answer");
    assert_eq!(status["sync_days"], 30);
    assert_eq!(status["auto_sync"], false);
    assert_eq!(status["sync_interval_minutes"], 15);

    // The values are bounded, as the reference bounds them.
    let bounded = api
        .call(
            "PATCH",
            "/api/v1/sync/settings",
            Some(r#"{"sync_days": 9000, "sync_interval_minutes": 1}"#),
        )
        .expect("the settings answer");
    assert_eq!(bounded["sync_days"], 3650);
    assert_eq!(bounded["sync_interval_minutes"], 15);

    // An unknown route is still a 400.
    let refused = api
        .call(
            "PATCH",
            "/api/v1/sync/settings",
            Some(r#"{"network_route": "somewhere"}"#),
        )
        .expect_err("400");
    assert_eq!(refused.status, 400);
    assert_eq!(refused.detail, "Unknown network route.");
}

/// The state and the router agree about the demo data.
#[test]
fn the_demo_data_can_be_hidden() {
    let api = api();
    let status = api
        .call(
            "PATCH",
            "/api/v1/sync/settings",
            Some(r#"{"demo_visible": false}"#),
        )
        .expect("the settings answer");
    assert_eq!(status["demo_visible"], false);
    let fresh = api
        .call("GET", "/api/v1/auth/status", None)
        .expect("the status answers");
    assert_eq!(fresh["demo_visible"], false);
}

/// The network route is read from the machine, and the route is validated
/// before anything is changed.
#[test]
fn the_network_status_answers() {
    use stride_api::NetworkBackend;

    let backend = NetworkBackend::new(Arc::new(MemBackend::new()));
    let api = Api::new(Arc::new(backend));

    let status = api
        .call("GET", "/api/v1/network/status", None)
        .expect("the status answers");
    for key in [
        "exit_node_tooling_available",
        "exit_node",
        "exit_node_active",
        "public_ip",
        "public_org",
        "route",
        "public_identity",
        "effective_route",
        "nodes",
        "resolved_exit_node",
    ] {
        assert!(status.get(key).is_some(), "the status is missing {key}");
    }
    // The route is the machine's, and the preference is the state's.
    assert!(status["route"].is_string());
    assert!(status["effective_route"].is_string());

    // An unknown route is refused before the machine is touched.
    let refused = api
        .call(
            "POST",
            "/api/v1/network/route",
            Some(r#"{"route": "somewhere"}"#),
        )
        .expect_err("400");
    assert_eq!(refused.status, 400);
    assert_eq!(refused.detail, "Route must be 'direct' or 'exit_node'.");
}

/// A sync without a Garmin key says so, instead of pretending to run.
#[test]
fn a_sync_without_a_key_says_so() {
    let api = api();
    let err = api
        .call(
            "POST",
            "/api/v1/sync/run",
            Some(r#"{"days": 30, "background": false}"#),
        )
        .expect_err("400");
    assert_eq!(err.status, 400);
    assert_eq!(err.detail, "There is no Garmin key to sync with.");

    // The status is still answerable: nothing is running.
    let status = api
        .call("GET", "/api/v1/sync/status", None)
        .expect("the status answers");
    assert_eq!(status["running"], false);
}

/// The sync status falls back to the log when this process has not synced.
///
/// The reference keeps the start time and the message of the last sync after
/// it has finished, and the log is what knows them after a restart.
#[test]
fn the_sync_status_falls_back_to_the_log() {
    let path = std::env::temp_dir().join(format!("stride-api-log-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let store = stride_store::Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    let rows = stride_api::store::backend(Arc::new(store));
    rows.upsert_sync_run(&stride_api::backend::SyncRunRow {
        source: "personal".to_string(),
        started_at: Some("2026-10-29T04:01:09+00:00".to_string()),
        finished_at: Some("2026-10-29T04:02:23+00:00".to_string()),
        status: "ok".to_string(),
        days: 90,
        activities_found: 29,
        activities_new: 0,
        activities_updated: 29,
        message: Some(
            "Synced 0 new and 29 updated activities, plus 30 days of health data.".to_string(),
        ),
        error: None,
        ..stride_api::backend::SyncRunRow::default()
    })
    .expect("the row is written");

    let state = StateBackend::new(rows);
    let api = Api::new(Arc::new(stride_api::CoreBackend::new(Arc::new(state))));
    let status = api
        .call("GET", "/api/v1/sync/status", None)
        .expect("the status answers");
    assert_eq!(status["running"], false);
    assert_eq!(
        status["message"],
        "Synced 0 new and 29 updated activities, plus 30 days of health data."
    );
    assert_eq!(
        status["started_at"].as_str().unwrap_or_default(),
        "2026-10-29T04:01:09+00:00"
    );
    assert_eq!(
        status["last_sync_at"].as_str().unwrap_or_default(),
        "2026-10-29T04:02:23+00:00"
    );
}

/// The trust configuration is the deployment's, and it is not in the state.
#[test]
fn the_trust_configuration_is_the_deployments() {
    use stride_api::network::{Cidr, TrustConfig};

    let mut state = stride_api::State::default();
    assert!(state.trust.is_empty(), "nothing is trusted by default");
    state.trust = TrustConfig {
        remote: true,
        networks: vec![Cidr::parse("100.64.0.0/10").expect("the network")],
    };
    let backend = StateBackend::with_state(Arc::new(MemBackend::new()), state);
    let inside: std::net::IpAddr = "100.89.222.50".parse().expect("an address");
    let outside: std::net::IpAddr = "192.0.2.7".parse().expect("an address");
    assert!(backend.trusts(&inside));
    assert!(!backend.trusts(&outside));
}
