//! The golden tests: the FIT importer against the reference implementation.
//!
//! Every case in `backend/tests/golden/cases.json` that involves a FIT file
//! is parsed here and compared with the golden file the Python reference
//! produced. The golden files decide: they are the definition of correct.
//!
//! The comparison follows `docs/core-contract.md` §8: strings, booleans,
//! `null` and integers must be exactly equal, and floats may differ by at
//! most `1e-9 × max(1, |golden|)`.
//!
//! A test may panic: it is a test, not a library, and a failing assertion is
//! the point.

// The workspace denies `unwrap`, `expect` and `panic` in library code; a test
// is allowed to fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_fit::SegmentKind;

use common::{compare, golden, import_section, parse};

/// Parse a fixture and compare it with its golden file.
fn check_case(case: &str) {
    let activity = parse(case);
    let golden = golden(case);
    let expected = golden
        .get("import")
        .unwrap_or_else(|| panic!("{case}: the golden file has no import section"));
    compare(
        &format!("{case}.import"),
        expected,
        &import_section(&activity),
    );
}

/// A run in the shape a COROS export has.
///
/// The file names COROS, in the `file_id` message and in the
/// `device_info` message, and the `device_info` message carries the product
/// name of the watch. The run is attributed to that watch — not to the
/// manufacturer the importer was written against.
#[test]
fn golden_vendor_coros_export() {
    check_case("vendor_coros_export");
    let activity = parse("vendor_coros_export");
    assert_eq!(activity.distance_m, 1266.0);
    assert_eq!(activity.moving_time_s, 420.0);
    assert_eq!(activity.elapsed_time_s, 420.0);
    assert_eq!(activity.avg_hr, Some(135.0));
    assert_eq!(activity.max_hr, Some(141.0));
    // The walking warm-up is a walk, and it is not running time.
    assert_eq!(activity.walking_time_s, Some(60.0));
    assert_eq!(activity.running_time_s, Some(359.0));
    assert_eq!(activity.device.as_deref(), Some("COROS DURA"));
}

#[test]
fn golden_cycling() {
    check_case("cycling");
}

#[test]
fn golden_gaps() {
    check_case("gaps");
}

#[test]
fn golden_hilly() {
    check_case("hilly");
}

#[test]
fn golden_hr_gps_run() {
    check_case("hr_gps_run");
}

#[test]
fn golden_low_resolution() {
    check_case("low_resolution");
}

#[test]
fn golden_mixed_run() {
    check_case("mixed_run");
}

#[test]
fn golden_no_cadence() {
    check_case("no_cadence");
}

#[test]
fn golden_short_effort() {
    check_case("short_effort");
}

#[test]
fn golden_treadmill() {
    check_case("treadmill");
}

/// A run in the shape a Strava export has.
///
/// Strava re-encodes the file it serves: its own manufacturer is in the
/// `file_id` message, and the watch that recorded the run is not in the
/// file at all. So the run has no watch to name, and it must not borrow the
/// name of a watch that is not in the file.
#[test]
fn golden_vendor_strava_export() {
    check_case("vendor_strava_export");
    let activity = parse("vendor_strava_export");
    assert_eq!(activity.distance_m, 2040.0);
    assert_eq!(activity.moving_time_s, 600.0);
    assert_eq!(activity.elapsed_time_s, 600.0);
    assert_eq!(activity.avg_hr, Some(142.0));
    assert_eq!(activity.max_hr, Some(147.0));
    assert_eq!(activity.device, None);
}

/// A run in the shape a Wahoo export has.
///
/// Wahoo writes the manufacturer and no product name, so the watch is named
/// by its manufacturer alone — and it must be named the way the reference
/// decoder names it.
#[test]
fn golden_vendor_wahoo_export() {
    check_case("vendor_wahoo_export");
    let activity = parse("vendor_wahoo_export");
    assert_eq!(activity.distance_m, 1920.0);
    assert_eq!(activity.moving_time_s, 600.0);
    assert_eq!(activity.elapsed_time_s, 600.0);
    assert_eq!(activity.avg_hr, Some(147.0));
    assert_eq!(activity.max_hr, Some(152.0));
    assert_eq!(activity.device.as_deref(), Some("wahoo_fitness"));
}

/// The numbers the report quotes, asserted so they cannot drift silently.
#[test]
fn the_headline_numbers() {
    let mixed = parse("mixed_run");
    assert_eq!(mixed.running_time_s, Some(139.0));
    assert_eq!(mixed.walking_time_s, Some(10.0));
    assert_eq!(mixed.stopped_time_s, Some(20.0));
    assert_eq!(mixed.running_distance_m, Some(426.8));
    assert_eq!(mixed.distance_m, 443.0);
    assert_eq!(mixed.moving_time_s, 150.0);
    assert_eq!(mixed.elapsed_time_s, 170.0);
    assert_eq!(mixed.avg_cadence, Some(78.0));
    assert_eq!(mixed.avg_hr, Some(142.0));
    assert_eq!(mixed.max_hr, Some(155.0));
    assert_eq!(mixed.sport_type, "running");
    assert_eq!(mixed.segments.len(), 4);
    assert_eq!(mixed.segments[0].kind, SegmentKind::Walk);
    assert_eq!(mixed.segments[1].kind, SegmentKind::Run);
    assert_eq!(mixed.segments[2].kind, SegmentKind::Stand);
    assert_eq!(mixed.segments[3].kind, SegmentKind::Run);

    // A cycling cadence of 90 rpm stays 90: the halving correction is for
    // running sports only.
    let cycling = parse("cycling");
    assert_eq!(cycling.sport_type, "cycling");
    assert_eq!(cycling.avg_cadence, Some(90.0));
    assert_eq!(cycling.laps[0].avg_cadence, Some(90.0));
    assert_eq!(cycling.laps[0].max_cadence, Some(90.0));

    // A watch that records every five seconds: the intervals are five
    // seconds wide.
    let low = parse("low_resolution");
    assert_eq!(low.segments[0].samples, 41);
    assert_eq!(low.segments[0].duration_s, 200.0);
}
