//! The FIT privacy gate: no committed corpus FIT file may carry a coordinate
//! that is not the synthetic anchor (-35.0, -140.0). This is the Rust-side
//! counterpart of the old repo's `backend/tests/golden/test_fixture_privacy.py`.
//!
//! Test code is the one place the workspace lints are deliberately relaxed:
//! `unwrap`/`expect`/`panic` are how a test reports a failure. Library code
//! keeps the deny; tests opt out here (see CONTRIBUTING.md).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use stride_testkit::fit_position;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn every_corpus_fit_position_is_the_synthetic_anchor() {
    let fit_dir = fixtures_dir().join("fit");
    let files: Vec<PathBuf> = std::fs::read_dir(&fit_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", fit_dir.display()))
        .map(|e| e.expect("read_dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "fit"))
        .collect();

    assert!(!files.is_empty(), "no corpus FIT files found");

    for path in &files {
        let positions = fit_position::positions_in(path)
            .unwrap_or_else(|e| panic!("{}: decode failed: {e}", path.display()));
        for &(lat, lon) in &positions {
            assert!(
                fit_position::within_anchor(lat, lon),
                "{}: position ({lat}, {lon}) is not the synthetic anchor",
                path.display()
            );
        }
    }
}

#[test]
fn the_bad_fit_fixture_is_detected() {
    // The self-test: prove the reader actually reads positions, by feeding it a
    // file whose position is deliberately not the anchor. If this test passes,
    // a real route written into a corpus file cannot slip past the gate above.
    let bad = fixtures_dir().join("scan-tests/bad.fit");
    let positions = fit_position::positions_in(&bad)
        .unwrap_or_else(|e| panic!("{}: decode failed: {e}", bad.display()));
    assert!(!positions.is_empty(), "bad.fit carries no positions");
    for &(lat, lon) in &positions {
        assert!(
            !fit_position::within_anchor(lat, lon),
            "bad.fit position ({lat}, {lon}) is inside the anchor — the reader is vacuous"
        );
    }
}
