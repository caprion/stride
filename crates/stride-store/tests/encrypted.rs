//! The database on disk has to be encrypted.
//!
//! An unencrypted database that claims to be encrypted is the worst outcome in
//! this project: a year of sleep, heart rate, stress and GPS routes, in the
//! clear, on a laptop.

// An integration test is its own crate, so clippy's allow-in-tests
// configuration does not cover it: the test code opts out here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::{Activity, Store, StoreError};

/// Does this haystack contain that needle?
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn the_bytes_on_disk_do_not_contain_what_was_written() {
    let dir = common::temp_dir("encrypted");
    let path = dir.join("stride.db");
    let key = common::key();

    let store = Store::open_and_migrate(&path, &key).expect("could not open");

    let sentinel = "ZEBRA-SENTINEL-9001";
    let secret = "the-athlete-is-called-Sumit";

    let activity = Activity {
        source: "garmin_fit".to_string(),
        external_id: sentinel.to_string(),
        name: sentinel.to_string(),
        description: Some(secret.to_string()),
        device: Some(secret.to_string()),
        geo_points: Some(serde_json::json!([[28.6139, 77.2090]])),
        ..Activity::default()
    };
    store.upsert_activity(&activity).expect("could not write");

    // Put the write-ahead log back into the main file, so the bytes on disk
    // are everything the database holds.
    store
        .with_connection(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|source| StoreError::query("wal_checkpoint".to_string(), source))
        })
        .expect("could not checkpoint");

    drop(store);

    let bytes = std::fs::read(&path).expect("could not read the database file");
    assert!(
        bytes.len() > 4096,
        "the database file is only {} bytes",
        bytes.len()
    );
    assert!(
        !contains(&bytes, b"SQLite format 3"),
        "the file starts with the plain SQLite header: it is not encrypted"
    );
    assert!(
        !contains(&bytes, sentinel.as_bytes()),
        "the sentinel is in the file in the clear"
    );
    assert!(
        !contains(&bytes, secret.as_bytes()),
        "the secret is in the file in the clear"
    );
    assert!(
        !contains(&bytes, b"garmin_fit"),
        "the source is in the file in the clear"
    );
    assert!(
        !contains(&bytes, b"28.6139"),
        "the route is in the file in the clear"
    );

    // The write-ahead log must not leak either.
    let wal = dir.join("stride.db-wal");
    if wal.exists() {
        let wal_bytes = std::fs::read(&wal).expect("could not read the write-ahead log");
        assert!(
            !contains(&wal_bytes, sentinel.as_bytes()),
            "the sentinel is in the write-ahead log in the clear"
        );
    }
}

#[test]
fn the_wrong_key_does_not_open_the_database() {
    let dir = common::temp_dir("wrong-key");
    let path = dir.join("stride.db");
    let key = common::key();

    let store = Store::open_and_migrate(&path, &key).expect("could not open");
    store
        .upsert_activity(&Activity {
            source: "garmin".to_string(),
            external_id: "1".to_string(),
            ..Activity::default()
        })
        .expect("could not write");
    drop(store);

    let error = Store::open(&path, &common::other_key()).expect_err("the wrong key opened it");
    match error {
        StoreError::InvalidKey { .. } => {}
        other => panic!("the wrong key gave {other:?}, not an invalid-key error"),
    }
}

#[test]
fn an_empty_key_is_refused() {
    let dir = common::temp_dir("empty-key");
    let error = Store::open(&dir.join("stride.db"), b"").expect_err("the empty key opened it");
    match error {
        StoreError::InvalidKey { .. } => {}
        other => panic!("the empty key gave {other:?}, not an invalid-key error"),
    }
}

#[test]
fn a_passphrase_key_works_too() {
    let dir = common::temp_dir("passphrase");
    let path = dir.join("stride.db");
    let passphrase = b"a passphrase with a ' in it";

    {
        let store = Store::open_and_migrate(&path, passphrase).expect("could not open");
        store
            .upsert_activity(&Activity {
                source: "garmin".to_string(),
                external_id: "1".to_string(),
                ..Activity::default()
            })
            .expect("could not write");
    }

    let store = Store::open(&path, passphrase).expect("the passphrase did not open it again");
    let activity = store.activity(1).expect("could not read").expect("gone");
    assert_eq!(activity.external_id, "1");
}
