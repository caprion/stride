//! The application state: the credentials, the pairing code and the devices.
//!
//! The state is not in the database: the Python keeps it in `state.json` and
//! the phone keeps it in the Android preferences. These tests drive the
//! decorator that owns it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::Value;
use stride_api::{Api, MemBackend, StateBackend};

/// A pairing code is the operator's console secret, and it is minted on the
/// machine that runs Stride.
///
/// The reference refuses anybody who is not on the machine, and the
/// security gate checks exactly that over the TLS listener, which is bound
/// to the operator's address and not to loopback. The router does not know
/// who is asking: the host does, and it passes the caller's address and
/// the name it used with the request. A caller that does not say where it
/// is gets no code at all.
#[test]
fn a_caller_that_does_not_say_where_it_is_gets_no_pairing_code() {
    // The desktop's stack: the rows, the state, the network, the domain.
    let api = desktop_api();

    let refused = api
        .call("GET", "/api/v1/auth/pair-code", None)
        .expect_err("403");
    assert_eq!(refused.status, 403);
    assert_eq!(
        refused.detail,
        "A pairing code can only be requested on the machine running Stride."
    );
}

/// The desktop's answer: a caller on the machine may mint a code, and a
/// caller on the network may not.
///
/// The desktop is the host: it has the caller's address and the name it
/// used. The rule is the one the security gate checks over the TLS
/// listener — a pairing code cannot be minted from another device — and
/// the one the operator's console needs: a code can be minted here, on the
/// machine, for the phone to pair with.
#[test]
fn a_caller_on_the_machine_may_mint_a_pairing_code() {
    let api = desktop_api();
    let caller = stride_api::Caller {
        peer: Some("127.0.0.1".parse().expect("an address")),
        host: Some("127.0.0.1:8000".to_string()),
    };
    let minted = api
        .call_response_from(&caller, "GET", "/api/v1/auth/pair-code", None)
        .expect("the code answers");
    assert!(
        minted.to_value()["code"]
            .as_str()
            .is_some_and(|code| code.len() == 9),
        "the code was minted"
    );
}

/// A caller on the network is refused, whatever it calls itself.
#[test]
fn a_caller_on_the_network_may_not_mint_a_pairing_code() {
    let api = desktop_api();
    for (peer, host) in [
        ("100.89.222.50", "100.89.222.50:8443"),
        ("127.0.0.1", "stride.example.com"),
        ("100.89.222.50", "localhost:8000"),
    ] {
        let caller = stride_api::Caller {
            peer: Some(peer.parse().expect("an address")),
            host: Some(host.to_string()),
        };
        let refused = api
            .call_response_from(&caller, "GET", "/api/v1/auth/pair-code", None)
            .expect_err("403");
        assert_eq!(refused.status, 403, "{peer} / {host} was refused");
    }
}

/// The desktop's stack, over a temporary database.
fn desktop_api() -> Api {
    let directory = tempdir::TempDir::new("stride-api-pairing").expect("a temporary directory");
    let path = directory.path().join("stride.db");
    let store = stride_store::Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    let rows = stride_api::store::backend(Arc::new(store));
    let state = StateBackend::new(rows);
    Api::new(Arc::new(state))
}

/// The console \u2014 the in-memory backend \u2014 mints one, because it holds
/// nothing that is not on this machine.
#[test]
fn the_console_may_mint_a_pairing_code() {
    let api = api();
    let minted = api
        .call("GET", "/api/v1/auth/pair-code", None)
        .expect("the code answers");
    assert!(minted["code"].as_str().is_some_and(|code| code.len() == 9));
}

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
    // The demo flag is gone with the demo dataset: the status does not
    // carry it, and the reference does not answer it.
    assert_eq!(status.get("demo_visible"), None);
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

/// The demo data is gone, and so is the flag that hid it.
#[test]
fn the_demo_flag_is_gone() {
    let api = api();
    let status = api
        .call(
            "PATCH",
            "/api/v1/sync/settings",
            Some(r#"{"demo_visible": false}"#),
        )
        .expect("the settings answer");
    assert_eq!(status.get("demo_visible"), None);
    let fresh = api
        .call("GET", "/api/v1/auth/status", None)
        .expect("the status answers");
    assert_eq!(fresh.get("demo_visible"), None);
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

/// The plan sources: a baseline and a prescription, each with its own ruler.
///
/// The route is new, and the existing shapes do not change: it is a route of
/// its own, and it carries the two plans the screen offers.
#[test]
fn the_plan_sources_answer_with_both_plans() {
    let api = api();
    let sources = api
        .call("GET", "/api/v1/plan/sources", None)
        .expect("the sources answer");
    for key in ["inferred", "garmin"] {
        let source = sources
            .get(key)
            .unwrap_or_else(|| panic!("the sources are missing {key}"));
        assert!(source.get("source").is_some(), "{key} has no source");
        assert!(
            source.get("prescribed").is_some(),
            "{key} does not say whether it is a prescription"
        );
        assert!(source.get("sessions").is_some(), "{key} has no sessions");
    }
    // The Garmin source needs a key; without one it says so rather than
    // pretending to be empty.
    assert_eq!(sources["garmin"]["available"], false);
    assert!(sources["garmin"]["reason"].is_string());
    // The inferred plan is a baseline: it is never a prescription.
    assert_eq!(sources["inferred"]["prescribed"], false);
}

/// A tiny temporary-directory helper, so the test does not need a crate.
mod tempdir {
    use std::path::{Path, PathBuf};

    /// A directory that removes itself when it is dropped.
    pub struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        /// Make a directory with a name nobody else is using.
        pub fn new(prefix: &str) -> std::io::Result<Self> {
            let mut path = std::env::temp_dir();
            let unique = format!(
                "{prefix}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|since| since.as_nanos())
                    .unwrap_or_default()
            );
            path.push(unique);
            std::fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        /// The directory itself.
        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
