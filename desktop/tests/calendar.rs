//! The rig's hours are the machine's hours.
//!
//! The rig reads its zone from the operating system and puts it into the
//! router, so the part of the day a run falls in and the athlete's own day
//! are read in the machine's own time — and not in UTC, which is what a
//! rig that never asked would answer.
//!
//! **The test is here because the bug was here.** The rig answered in UTC
//! for as long as it existed, and every early run in the athlete's history
//! was reported as a night run.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use stride_desktop::api::{ApiRouter, Host};
use stride_desktop::key::KeyStore;
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, Store};

/// The host the rig offers the router.
struct TestHost {
    store: Arc<Store>,
    keys: Arc<KeyStore>,
}

impl Host for TestHost {
    fn store(&self) -> Arc<Store> {
        Arc::clone(&self.store)
    }

    fn keys(&self) -> Arc<KeyStore> {
        Arc::clone(&self.keys)
    }
}

/// The rig over a scratch database.
fn rig() -> ApiRouter {
    let dir = ScratchDir::new("rig-calendar").unwrap_or_else(|_| panic!("the scratch directory"));
    let key =
        DatabaseKey::from_text("the database key of a test").unwrap_or_else(|_| panic!("the key"));
    let store = Arc::new(
        Store::open_and_migrate(&dir.file("stride.db"), &key)
            .unwrap_or_else(|_| panic!("the store")),
    );
    let host = TestHost {
        store,
        keys: Arc::new(KeyStore::with_persist_path(dir.file("keys"))),
    };
    ApiRouter::over(&host).unwrap_or_else(|_| panic!("the router"))
}

#[test]
fn the_rig_reads_its_hours_in_the_machines_own_zone() {
    let router = rig();
    let api = router.api();
    // The offset in force is the machine's own — the one the operating
    // system says — and not UTC.
    assert_eq!(
        api.utc_offset_minutes(),
        stride_api::state::machine_offset_minutes()
    );
    assert_eq!(
        api.host_utc_offset_minutes(),
        stride_api::state::machine_offset_minutes()
    );
    // And the athlete has stated no offset of their own, so the machine's
    // is the one in force.
    assert_eq!(api.utc_offset_override(), None);
    assert_eq!(api.utc_offset_source(), "machine");
}

#[test]
fn the_athletes_own_offset_is_in_force_over_the_machines_own() {
    let router = rig();
    let api = router.api();
    api.set_utc_offset_override(Some(330))
        .unwrap_or_else(|_| panic!("the offset"));
    // The athlete's own offset wins, and the machine's own is still the
    // machine's own.
    assert_eq!(api.utc_offset_minutes(), 330);
    assert_eq!(
        api.host_utc_offset_minutes(),
        stride_api::state::machine_offset_minutes()
    );
    assert_eq!(api.utc_offset_source(), "stated");
}
