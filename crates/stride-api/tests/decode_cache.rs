// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The decode cache: one decode per run, and a bounded number of them.
//!
//! **The cost this buys back.** A twelve-week window is about sixty runs,
//! and every run's file was decoded again on every read: the pace page,
//! the rolling view, the year, the heatmap and the ledger all read their
//! runs' files. The cache is the one place a file is decoded, and it is
//! bounded — so a read is cheap and the memory is not.

mod common;

use common::{run_with_fit, Harness};
use stride_api::Config;

/// The decoded run, through the router's one choke point.
fn decoded(harness: &Harness, id: i64) -> bool {
    stride_api::routes::series::decoded(&harness.api, id).is_some()
}

#[test]
fn the_first_read_decodes_and_the_second_is_served_from_the_cache() {
    let harness = Harness::empty("decode-cache");
    let id = harness.write_run(run_with_fit("a-run", "2026-05-10", "hr_gps_run.fit"));

    assert!(decoded(&harness, id));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.hits, 0, "the first read was served from the cache");
    assert_eq!(stats.misses, 1, "the first read did not decode");

    assert!(decoded(&harness, id));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(
        stats.hits, 1,
        "the second read was not served from the cache"
    );
    assert_eq!(stats.misses, 1, "the second read decoded the file again");
}

#[test]
fn a_route_that_reads_a_run_decodes_it_once() {
    let harness = Harness::empty("decode-cache-route");
    let id = harness.write_run(run_with_fit("a-run", "2026-05-10", "hr_gps_run.fit"));

    harness.ok_get(&format!("/activities/{id}"));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.misses, 1, "the route decoded the file more than once");
    assert_eq!(stats.hits, 0);

    harness.ok_get(&format!("/activities/{id}"));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.misses, 1, "the second read decoded the file again");
    assert_eq!(
        stats.hits, 1,
        "the second read was not served from the cache"
    );
}

#[test]
fn the_cache_holds_a_bounded_number_of_runs() {
    let config = Config {
        decode_cache_capacity: 2,
        ..Config::default()
    };
    let harness = Harness::with_config("decode-cache-bound", config);

    let first = harness.write_run(run_with_fit("run-one", "2026-05-10", "hr_gps_run.fit"));
    let second = harness.write_run(run_with_fit("run-two", "2026-05-09", "hilly.fit"));
    let third = harness.write_run(run_with_fit("run-three", "2026-05-08", "mixed_run.fit"));

    for id in [first, second, third] {
        assert!(decoded(&harness, id));
    }

    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.entries, 2, "the cache grew past its bound");
    assert_eq!(stats.capacity, 2);
    assert_eq!(stats.misses, 3, "each run was decoded once");

    // The oldest run was thrown away: it is decoded again, and the two
    // that are still held are served from the cache.
    assert!(decoded(&harness, first));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.misses, 4, "the thrown-away run was not decoded again");
    assert!(decoded(&harness, third));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(
        stats.hits, 1,
        "the run that was still held was decoded again"
    );
}

#[test]
fn a_file_that_was_imported_again_invalidates_its_own_entry() {
    let harness = Harness::empty("decode-cache-reimport");
    let id = harness.write_run(run_with_fit("a-run", "2026-05-10", "hr_gps_run.fit"));
    assert!(decoded(&harness, id));
    assert!(decoded(&harness, id));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.hits, 1);

    // The same run, with a different file: the digest changes, and the
    // decode that is held is of a file that is not there any more.
    harness.write_run(run_with_fit("a-run", "2026-05-10", "hilly.fit"));
    assert!(decoded(&harness, id));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(
        stats.misses, 2,
        "the re-imported file was served from the cache"
    );
    assert_eq!(stats.hits, 1);
}

#[test]
fn a_run_with_no_file_has_no_decode_and_no_cache_entry() {
    let harness = Harness::empty("decode-cache-empty");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    assert!(!decoded(&harness, id));
    let stats = harness.api.decoded_cache().stats();
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.hits, 0);
}
