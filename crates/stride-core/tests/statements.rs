//! The golden assertions of the statements.
//!
//! Three sentences the app shows that are neither vitals, nor directions, nor
//! options, nor patterns: the distance comparison, the Progress headline and
//! the Compare basis. They are facts about the data, rendered by the core.

use stride_core::statement::{
    compare_basis, distance_comparison, distance_comparison_line, progress_headline, CompareBasis,
    PaceAtHr,
};
use stride_core::types::Distance;

fn kilometres(value: f64) -> Option<Distance> {
    Distance::from_kilometres(value)
}

#[test]
fn a_gap_beyond_ten_per_cent_is_stated() {
    let actual = match kilometres(12.4) {
        Some(distance) => distance,
        None => return,
    };
    let target = match kilometres(10.0) {
        Some(distance) => distance,
        None => return,
    };
    assert_eq!(
        distance_comparison(actual, target),
        Some("2.4 km more than the 10 km target".to_string())
    );
    assert_eq!(
        distance_comparison_line(actual, target),
        Some("12.4 km · 2.4 km more than the 10 km target".to_string())
    );
}

#[test]
fn a_gap_within_ten_per_cent_is_not_news() {
    let actual = match kilometres(10.2) {
        Some(distance) => distance,
        None => return,
    };
    let target = match kilometres(10.0) {
        Some(distance) => distance,
        None => return,
    };
    assert_eq!(distance_comparison(actual, target), None);
    assert_eq!(distance_comparison_line(actual, target), None);
}

#[test]
fn the_basis_names_what_is_actually_being_compared() {
    assert_eq!(
        compare_basis(true, false, 21).sentence(),
        "Same distance, 3 weeks apart"
    );
    assert_eq!(
        compare_basis(false, true, 21).sentence(),
        "Same route, different week"
    );
    assert_eq!(
        compare_basis(false, false, 21).sentence(),
        "Different distances — read with care"
    );
    assert_eq!(compare_basis(true, true, 21), CompareBasis::SameRoute);
}

#[test]
fn the_progress_headline_states_the_trend_and_its_evidence() {
    let headline = progress_headline(Some(PaceAtHr {
        now_s_per_km: 312.0,
        then_s_per_km: 336.0,
        weeks: 8,
    }));
    assert_eq!(headline.headline, "fitter than 8 weeks ago");
    assert_eq!(
        headline.evidence,
        Some("same heart rate, 24 s/km faster".to_string())
    );

    // Without a comparison, the headline says so plainly.
    let empty = progress_headline(None);
    assert_eq!(empty.headline, "not enough data yet");
    assert_eq!(empty.evidence, None);
}
