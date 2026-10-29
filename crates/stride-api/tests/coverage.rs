//! Every route, against the production stack.
//!
//! The stack the desktop and the phone build is
//! `Api → CoreBackend → TileBackend → NetworkBackend → StateBackend → Store`.
//! This test drives **all 65 routes** through it and checks that none of
//! them answers "not implemented" — the failure mode that hides a missing
//! piece of the app behind a 501 nobody looks at.
//!
//! Four routes are excluded, and the reason is in the list: they need the
//! Garmin OAuth1 client and the Garmin account client, which do not exist in
//! Rust and which the phone does not need — it connects with a Personal API
//! key, and that path is implemented.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::Value;

use stride_api::{Api, CoreBackend, NetworkBackend, StateBackend, TileBackend};

/// The routes that need a client this build does not have.
const EXCEPTIONS: [(&str, &str); 4] = [
    (
        "/api/v1/auth/garmin/developer-key",
        "the Garmin developer-key flow needs the OAuth1 client",
    ),
    (
        "/api/v1/auth/garmin/exchange",
        "the Garmin exchange flow needs the OAuth1 client",
    ),
    (
        "/api/v1/auth/garmin/login",
        "the Garmin login flow needs the account client",
    ),
    (
        "/api/v1/auth/garmin/mfa",
        "the Garmin MFA flow needs the account client",
    ),
];

/// A cache directory of this test's own.
///
/// The tests run in parallel, and two of them writing the same tile cache is
/// a race.
fn cache_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("stride-api-tiles-{}-{name}", std::process::id()))
}

/// The router over the whole stack.
///
/// Each test gets its own database: the tests run in parallel, and two of
/// them migrating the same file at the same time is a race.
fn api(cache: &std::path::Path) -> Api {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let number = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!(
        "stride-api-coverage-{}-{number}.db",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = stride_store::Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    let backend = stride_api::store::backend(Arc::new(store));
    let state = StateBackend::new(backend);
    let network = NetworkBackend::new(Arc::new(state));
    let tiles = TileBackend::new(Arc::new(network), cache);
    let core = CoreBackend::new(Arc::new(tiles));
    Api::new(Arc::new(core))
}

/// A path with every parameter filled in.
fn concrete(pattern: &str) -> String {
    pattern
        .replace("{activity_id}", "1")
        .replace("{id}", "1")
        .replace("{device_id}", "abc")
        .replace("{block_id}", "1")
        .replace("{phase_id}", "1")
        .replace("{week_id}", "1")
        .replace("{kind}", "hr")
        .replace("{z}", "12")
        .replace("{x}", "2048")
        .replace("{y}", "2048")
}

/// A body the mutating routes accept.
fn body(method: &str) -> Option<&'static str> {
    match method {
        "POST" => Some(
            r#"{"content": "x", "format": "markdown", "api_key": "0123456789012345678901234567890123456789", "code": "ABCD-EFGH", "route": "direct"}"#,
        ),
        "PATCH" => Some(r#"{"name": "x"}"#),
        _ => None,
    }
}

/// Every route answers, and none of them says "not implemented".
#[test]
fn every_route_answers() {
    let cache = cache_dir("coverage");
    let api = api(&cache);
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for route in stride_api::router::ROUTES {
        let method = route.method;
        let path = concrete(route.pattern);
        if EXCEPTIONS
            .iter()
            .any(|(pattern, _)| *pattern == route.pattern)
        {
            continue;
        }
        match api.call(method, &path, body(method)) {
            Ok(_) => {}
            Err(err) => {
                if err.status == 501 {
                    missing.push(format!("{method} {path} -> {}", err.detail));
                }
            }
        }
        checked += 1;
    }
    assert!(checked >= 60, "only {checked} routes were checked");
    assert!(
        missing.is_empty(),
        "{} routes are not implemented:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// The tile policy is the reference's, and the cache is where it says.
#[test]
fn the_tile_policy_is_the_reference_s() {
    let cache = cache_dir("policy");
    let api = api(&cache);
    let policy = api
        .call("GET", "/api/v1/tiles/policy", None)
        .expect("the policy answers");
    assert_eq!(policy["upstream"], "openstreetmap.org");
    assert_eq!(policy["proxied"], true);
    assert_eq!(
        policy["user_agent"],
        "Stride/1.0 (private running log; https://github.com/local/stride)"
    );
    assert!(policy["reason"]
        .as_str()
        .expect("a reason")
        .contains("never reveals the athlete's IP address"));
    assert_eq!(
        policy["cache_dir"].as_str().unwrap_or_default(),
        cache.to_string_lossy().as_ref()
    );
}

/// A cached tile is served from the disk, without touching the network.
#[test]
fn a_cached_tile_is_served_from_the_disk() {
    let cache = cache_dir("cached");
    let _ = std::fs::remove_dir_all(&cache);
    std::fs::create_dir_all(&cache).expect("the cache directory is made");
    // The name the backend uses for the tile 12/2048/2048.
    let key = stride_api::tiles::tile_key(12, 2048, 2048);
    std::fs::write(cache.join(format!("{key}.png")), b"a png").expect("the tile is written");

    let api = api(&cache);
    let response = api
        .call_response("GET", "/api/v1/tiles/12/2048/2048.png", None)
        .expect("the tile answers");
    match response {
        stride_api::Response::Binary {
            content_type,
            bytes,
        } => {
            assert_eq!(content_type, "image/png");
            assert_eq!(bytes, b"a png");
        }
        other => panic!("the tile is not binary: {other:?}"),
    }
}

/// The sync status answers on the desktop's own stack.
///
/// The desktop builds `Api → CoreBackend → Store` — no state layer — so the
/// sync status has to come out of the database, where the sync log is. The
/// dashboard's "Synced 2 days ago" reads this route, so a 501 here is a
/// visible gap and not a theoretical one.
#[test]
fn the_sync_status_answers_without_the_state_layer() {
    let path = std::env::temp_dir().join(format!("stride-api-sync-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let store = stride_store::Store::open_and_migrate(&path, b"0123456789abcdef0123456789abcdef")
        .expect("the store opens");
    let backend = stride_api::store::backend(Arc::new(store));
    // The desktop's stack: the rows and the domain, and nothing else.
    let api = Api::new(Arc::new(CoreBackend::new(backend)));

    let status = api
        .call("GET", "/api/v1/sync/status", None)
        .expect("the status answers");
    for key in [
        "running",
        "started_at",
        "message",
        "last_sync_at",
        "connected",
        "method",
    ] {
        assert!(status.get(key).is_some(), "the status is missing {key}");
    }
    assert_eq!(status["running"], false);
    assert_eq!(status["connected"], false);
    assert_eq!(status["method"], Value::Null);
}

/// A tile that cannot be fetched is a 502, the way the reference answers it.
#[test]
fn a_tile_that_cannot_be_fetched_is_a_502() {
    use stride_api::TileBackend;

    let cache = cache_dir("dead");
    let _ = std::fs::remove_dir_all(&cache);
    let backend = stride_api::store::backend(Arc::new(
        stride_store::Store::open_and_migrate(
            &std::env::temp_dir().join(format!("stride-api-dead-{}.db", std::process::id())),
            b"0123456789abcdef0123456789abcdef",
        )
        .expect("the store opens"),
    ));
    // Nothing is listening on this port, so the fetch fails.
    let tiles = TileBackend::with_host(backend, &cache, "127.0.0.1:1");
    let api = Api::new(Arc::new(tiles));
    let err = api
        .call("GET", "/api/v1/tiles/12/2048/2048.png", None)
        .expect_err("502");
    assert_eq!(err.status, 502);
    assert!(
        err.detail
            .as_str()
            .expect("a detail")
            .starts_with("Map tile unavailable"),
        "the detail is {:?}",
        err.detail
    );
}

/// A tile that is not on the grid is refused, the way the reference refuses it.
#[test]
fn a_tile_off_the_grid_is_refused() {
    let cache = cache_dir("off-grid");
    let api = api(&cache);
    let err = api
        .call("GET", "/api/v1/tiles/12/9999/9999.png", None)
        .expect_err("400");
    assert_eq!(err.status, 400);
    assert_eq!(err.detail, "Invalid tile coordinates.");
}
