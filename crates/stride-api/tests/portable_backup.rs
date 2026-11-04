#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;
use stride_api::{ffi, Api, Config, FixedClock, Router};
use stride_store::{DatabaseKey, Store};

#[test]
fn backup_restores_without_original_device_key_and_refuses_wrong_or_corrupt_input() {
    let source = common::Harness::seeded("portable-backup-source");
    ffi::install(Router::new(Arc::clone(&source.api)));
    let json_export = ffi::history_export_value("2026-05-10", "2026-05-11", "json");
    let observations: serde_json::Value =
        serde_json::from_str(json_export["body"]["content"].as_str().unwrap()).unwrap();
    assert_eq!(observations["observations"].as_array().unwrap().len(), 3);
    assert!(observations["observations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["date"].as_str().unwrap() <= "2026-05-11"));
    let csv = ffi::history_export_value("2026-05-10", "2026-05-11", "csv");
    assert!(csv["body"]["content"]
        .as_str()
        .unwrap()
        .starts_with("kind,date,distance_m,moving_time_s"));
    assert_eq!(
        ffi::history_export_value("2026-05-12", "2026-05-10", "csv")["ok"],
        false
    );
    assert_eq!(
        ffi::history_export_value("2026-05-10", "2026-05-11", "../../file")["ok"],
        false
    );
    let archive = source.dir.file("recovery");
    let recovery_key = DatabaseKey::from_bytes([17; 32]).unwrap();
    assert_eq!(
        ffi::backup_value(&archive, &recovery_key, false)["ok"],
        true
    );
    let count = source.store.count_activities().unwrap();
    let target_dir = stride_store::scratch::ScratchDir::new("portable-backup-target").unwrap();
    let different_device_key = DatabaseKey::from_bytes([29; 32]).unwrap();
    let target = Arc::new(
        Store::open_and_migrate(&target_dir.file("stride.db"), &different_device_key).unwrap(),
    );
    let api = Arc::new(
        Api::new(
            Arc::clone(&target),
            Config::default(),
            Arc::new(FixedClock::new(common::instant())),
        )
        .unwrap(),
    );
    ffi::install(Router::new(api));
    let wrong_key = DatabaseKey::from_bytes([30; 32]).unwrap();
    assert_eq!(ffi::backup_value(&archive, &wrong_key, true)["ok"], false);
    assert_eq!(target.count_activities().unwrap(), 0);
    assert_eq!(ffi::backup_value(&archive, &recovery_key, true)["ok"], true);
    assert_eq!(target.count_activities().unwrap(), count);
    assert_eq!(
        target.count_wellness_daily().unwrap(),
        source.store.count_wellness_daily().unwrap()
    );
    assert_eq!(ffi::backup_value(&archive, &recovery_key, true)["ok"], true);
    assert_eq!(target.count_activities().unwrap(), count);
    let database = archive.join("stride.db");
    let mut bytes = std::fs::read(&database).unwrap();
    bytes[100] ^= 1;
    std::fs::write(&database, bytes).unwrap();
    assert_eq!(
        ffi::backup_value(&archive, &recovery_key, true)["ok"],
        false
    );
    assert_eq!(target.count_activities().unwrap(), count);
}
