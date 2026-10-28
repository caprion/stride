//! Shared helpers for the tests.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A 32-byte key, the shape a Keystore or a Key Vault secret has.
pub fn key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = (index as u8).wrapping_mul(7).wrapping_add(3);
    }
    key
}

/// A different 32-byte key, for the "the wrong key does not open it" test.
pub fn other_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = (index as u8).wrapping_add(200);
    }
    key
}

/// A fresh directory for one test.
pub fn temp_dir(name: &str) -> PathBuf {
    let serial = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "stride-store-test-{name}-{}-{serial}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("could not create the test directory");
    dir
}
