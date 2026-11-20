//! The pace bands: where a window's moving time went, by the pace it was
//! held at.
//!
//! The engine is a pure function of the runs handed to it: there is no clock
//! in these tests, no file is read and nothing is fetched. Every expected
//! number is worked out from the definition in the test itself.
//!
//! **The runs are invented.** The samples are half a kilometre apart, and
//! they mean nothing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::eddington::Period;
use stride_core::pace::{pace_bands, PacePoint, PaceRun};

/// The window every test measures over.
fn period() -> Period {
    Period::new(
        Date::parse("2026-05-01").expect("the first day is a day"),
        Date::parse("2026-05-10").expect("the last day is a day"),
    )
}

/// A run on a day, held at one pace.
fn run_at(day: &str, seconds_per_km: f64, kilometres: f64) -> PaceRun {
    let date = Date::parse(day).expect("the day is a day");
    let mut samples = Vec::new();
    let steps = (kilometres * 2.0).round() as i64;
    for step in 0..=steps {
        let kilometres = step as f64 * 0.5;
        samples.push(PacePoint::new(
            kilometres * seconds_per_km,
            kilometres * 1000.0,
        ));
    }
    PaceRun::new(date, samples)
}

#[test]
fn a_flat_series_lands_in_one_band() {
    // Five kilometres at 5:00 /km: one band, 1 500 seconds in it.
    let bands = pace_bands(&[run_at("2026-05-04", 300.0, 5.0)], period());

    assert_eq!(bands.bands.len(), 1, "one band holds the run");
    assert_eq!(bands.bands[0].from_s_per_km, 300.0);
    assert_eq!(bands.bands[0].to_s_per_km, 330.0);
    assert_eq!(bands.bands[0].time_s, 1_500.0);
    assert_eq!(bands.bands[0].distance_m, 5_000.0);
    assert_eq!(bands.total_s, 1_500.0);
    assert_eq!(bands.total_m, 5_000.0);
}

#[test]
fn a_rising_series_fills_the_bands_it_crosses() {
    // The first kilometre at 4:00 /km, the second at 6:00 /km.
    let date = Date::parse("2026-05-04").expect("the day is a day");
    let samples = vec![
        PacePoint::new(0.0, 0.0),
        PacePoint::new(240.0, 1_000.0),
        PacePoint::new(600.0, 2_000.0),
    ];
    let bands = pace_bands(&[PaceRun::new(date, samples)], period());

    assert_eq!(bands.bands.len(), 2, "two bands hold the run");
    assert_eq!(bands.bands[0].from_s_per_km, 240.0);
    assert_eq!(bands.bands[0].time_s, 240.0);
    assert_eq!(bands.bands[1].from_s_per_km, 360.0);
    assert_eq!(bands.bands[1].time_s, 360.0);
    assert_eq!(bands.total_s, 600.0);
}

#[test]
fn a_gap_leaves_the_rest_of_the_window_counted() {
    // A run with no samples at all, and a run outside the window: neither
    // is a band of zero, and the run with samples is still counted.
    let empty = PaceRun::new(
        Date::parse("2026-05-05").expect("the day is a day"),
        Vec::new(),
    );
    let outside = run_at("2026-04-01", 300.0, 5.0);
    let inside = run_at("2026-05-04", 300.0, 5.0);

    let bands = pace_bands(&[empty, outside, inside], period());

    assert_eq!(bands.bands.len(), 1, "one band holds the window");
    assert_eq!(bands.total_s, 1_500.0);
    assert_eq!(bands.total_m, 5_000.0);
}

#[test]
fn a_single_sample_carries_no_time() {
    let date = Date::parse("2026-05-04").expect("the day is a day");
    let samples = vec![PacePoint::new(0.0, 0.0)];

    let bands = pace_bands(&[PaceRun::new(date, samples)], period());

    assert!(bands.is_empty(), "a lone sample holds no time");
    assert!(bands.bands.is_empty(), "and so no band");
}

#[test]
fn an_empty_window_has_no_bands() {
    let bands = pace_bands(&[], period());

    assert!(bands.is_empty(), "an empty window holds no time");
    assert_eq!(bands.total_s, 0.0);
    assert_eq!(bands.total_m, 0.0);
}

#[test]
fn the_bands_are_named_in_the_core_s_words() {
    let bands = pace_bands(&[run_at("2026-05-04", 300.0, 5.0)], period());

    assert_eq!(
        bands.bands[0].label(),
        Some("5:00–5:30 /km".to_string()),
        "the band is named by the core"
    );
}

#[test]
fn the_bands_are_a_property_of_the_window() {
    // The same run, measured over a window that holds it and over one that
    // does not: two different answers.
    let run = run_at("2026-05-04", 300.0, 5.0);

    let inside = pace_bands(std::slice::from_ref(&run), period());
    let outside = pace_bands(
        &[run],
        Period::new(
            Date::parse("2026-06-01").expect("the first day is a day"),
            Date::parse("2026-06-30").expect("the last day is a day"),
        ),
    );

    assert_eq!(inside.total_s, 1_500.0);
    assert!(outside.is_empty(), "the window holds no time");
}
