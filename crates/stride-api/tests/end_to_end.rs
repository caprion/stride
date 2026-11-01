//! The wiring, end to end.
//!
//! **A real encrypted database, a real router, and a real number in the
//! answer.** Nothing here is stubbed and nothing is faked: the database is
//! the SQLCipher file the rig opens, opened with a key the caller holds,
//! and the answer is the envelope the hosts read — the same envelope the
//! desktop's HTTP layer puts on the wire and the phone's bridge hands
//! across.
//!
//! The rig's own half — the HTTP layer, the trust model, the key store —
//! is the desktop's crate, and its own test is the desktop's. This file is
//! the half that is the router's: from the file on the disk to a number in
//! the envelope, with the store, the clock and the answers all real.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::json;
use stride_api::{Api, Config, FixedClock, Router};
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, NewActivity, Store};

/// The instant the test reads the clock at.
fn instant() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-05-12T06:30:00Z")
        .map(|value| value.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

/// A run to write.
fn run() -> NewActivity {
    NewActivity {
        source: "garmin_personal".to_string(),
        external_id: "a-run".to_string(),
        name: "a run".to_string(),
        sport_type: "running".to_string(),
        start_time: instant(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        fit_blob: Vec::new(),
        created_at: instant(),
        updated_at: instant(),
    }
}

/// The router over a real database, the way a host builds it.
fn router_over(store: Arc<Store>) -> Router {
    let api = Arc::new(
        Api::new(
            store,
            Config::default(),
            Arc::new(FixedClock::new(instant())),
        )
        .expect("the state could be built"),
    );
    Router::new(api)
}

#[test]
fn a_real_database_answers_with_a_real_number() {
    // The directory and the key, the way the rig has them.
    let dir = ScratchDir::new("end-to-end").expect("the scratch directory could be made");
    let key =
        DatabaseKey::from_text("the key of an end-to-end test").expect("the key could be made");
    let path = dir.file("stride.db");

    // The database: opened, migrated and written to.
    let store =
        Arc::new(Store::open_and_migrate(&path, &key).expect("the database could be opened"));
    store
        .upsert_activity(&run())
        .expect("the run could be written");

    // The file on the disk is encrypted: it is not a plain SQLite file.
    let bytes = std::fs::read(&path).expect("the database file could be read");
    assert!(
        !bytes.starts_with(b"SQLite format 3"),
        "the database on the disk is a plain SQLite file"
    );

    // The host's call: the envelope, with the path the app asks for.
    let router = router_over(store);
    let answer = router.envelope("GET", "/api/v1/activities", None);
    assert_eq!(answer["ok"], json!(true));
    assert_eq!(answer["status"], json!(200));

    // A real number, in the core's words and beside them.
    let activities = answer["body"]["activities"]
        .as_array()
        .expect("the feed is a list");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0]["distance"]["text"], json!("10 km"));
    // The number is compared by *value*, not by spelling: the contract says
    // the two hosts hand over the same answer, and a number's spelling can
    // change in transit (10000.0 may arrive as 10000).
    assert_eq!(
        activities[0]["distance"]["raw"].as_f64(),
        Some(10_000.0),
        "the distance's raw number is not 10000"
    );
    assert_eq!(activities[0]["duration"]["text"], json!("50 m"));
    assert_eq!(activities[0]["heart_rate"]["text"], json!("148 bpm"));

    // And the same answer from a second router over the same file: the
    // number is on the disk, not in the process.
    let store =
        Arc::new(Store::open_and_migrate(&path, &key).expect("the database could be opened again"));
    let again = router_over(store);
    assert_eq!(again.envelope("GET", "/api/v1/activities", None), answer);
}

#[test]
fn a_route_that_is_not_there_is_a_failure_and_not_a_number() {
    let dir = ScratchDir::new("end-to-end-failure").expect("the scratch directory could be made");
    let key =
        DatabaseKey::from_text("the key of an end-to-end test").expect("the key could be made");
    let store = Arc::new(
        Store::open_and_migrate(&dir.file("stride.db"), &key)
            .expect("the database could be opened"),
    );
    let router = router_over(store);

    let answer = router.envelope("GET", "/api/v1/nothing/here", None);
    assert_eq!(answer["ok"], json!(false));
    assert_eq!(answer["status"], json!(404));
    assert_eq!(answer["error"]["code"], json!("not_found"));
    assert!(answer["body"].is_null(), "a failure is not a body");
}
