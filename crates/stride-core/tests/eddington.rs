//! The Eddington number and its companions.
//!
//! The engine is a pure function of the daily distances and the period handed
//! to it: there is no clock in these tests, and every expected number is
//! worked out from the definition in the test itself, not copied from the
//! implementation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::eddington::{
    days_needed_for_next, eddington, DailyDistance, EddingtonUnit, Period,
};

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

fn day(text: &str, distance_km: f64) -> DailyDistance {
    DailyDistance::new(date(text), distance_km * 1000.0)
}

fn period(from: &str, to: &str) -> Period {
    Period::new(date(from), date(to))
}

#[test]
fn a_flat_history_counts_every_day() {
    // Ten days of exactly 5 km: five days reach 5 km, and no day reaches
    // 6 km, so the number is 5 and the next number is six days away.
    let days: Vec<DailyDistance> = (0..10)
        .map(|offset| day(&format!("2026-01-{:02}", offset + 1), 5.0))
        .collect();
    let result = eddington(
        &days,
        period("2026-01-01", "2026-01-10"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 5);
    assert_eq!(result.days_needed_for_next, 6);
    assert_eq!(result.longest_day_m, Some(5000.0));
}

#[test]
fn a_history_that_steps_up_has_a_higher_number() {
    // 1,1,2,2,3,3,4,4,5,5 km: four days reach 4 km, but only two reach 5 km.
    let distances_km = [1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0];
    let days: Vec<DailyDistance> = distances_km
        .iter()
        .enumerate()
        .map(|(index, km)| day(&format!("2026-01-{:02}", index + 1), *km))
        .collect();
    let result = eddington(
        &days,
        period("2026-01-01", "2026-01-10"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 4);
    // Five days at or above 5 km are needed; only two exist.
    assert_eq!(result.days_needed_for_next, 3);
    assert_eq!(result.longest_day_m, Some(5000.0));
}

#[test]
fn a_gap_does_not_count_as_a_day() {
    // Three 10 km days, a six-day gap, then three more 10 km days: the six
    // present days make the number, and the absent days contribute nothing.
    let mut days = Vec::new();
    for text in [
        "2026-01-01",
        "2026-01-02",
        "2026-01-03",
        "2026-01-10",
        "2026-01-11",
        "2026-01-12",
    ] {
        days.push(day(text, 10.0));
    }
    let result = eddington(
        &days,
        period("2026-01-01", "2026-01-12"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 6);
    // Seven days at or above 7 km are needed; six exist.
    assert_eq!(result.days_needed_for_next, 1);
    assert_eq!(result.longest_day_m, Some(10_000.0));
}

#[test]
fn a_single_day_has_an_eddington_number_of_one() {
    let days = vec![day("2026-01-01", 10.0)];
    let result = eddington(
        &days,
        period("2026-01-01", "2026-01-01"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 1);
    // One more day at or above 2 km reaches 2.
    assert_eq!(result.days_needed_for_next, 1);
    assert_eq!(result.longest_day_m, Some(10_000.0));
}

#[test]
fn an_empty_history_has_number_zero_and_no_longest_day() {
    let result = eddington(
        &[],
        period("2026-01-01", "2026-01-31"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 0);
    // One day at or above 1 km reaches 1.
    assert_eq!(result.days_needed_for_next, 1);
    assert_eq!(result.longest_day_m, None);
}

#[test]
fn only_days_inside_the_period_count() {
    let days = vec![
        day("2025-12-31", 10.0),
        day("2026-01-01", 10.0),
        day("2026-01-02", 10.0),
        day("2026-01-03", 10.0),
    ];
    let result = eddington(
        &days,
        period("2026-01-01", "2026-01-02"),
        EddingtonUnit::Kilometres,
    );
    assert_eq!(result.number, 2);
    assert_eq!(result.longest_day_m, Some(10_000.0));
}

#[test]
fn the_days_still_needed_are_zero_when_the_next_number_is_already_reached() {
    let days = vec![day("2026-01-01", 10.0), day("2026-01-02", 10.0)];
    let period = period("2026-01-01", "2026-01-02");

    // From 0 to 1: a day at or above 1 km already exists.
    assert_eq!(
        days_needed_for_next(&days, period, EddingtonUnit::Kilometres, 0),
        0
    );
    // From 1 to 2: two days at or above 2 km already exist.
    assert_eq!(
        days_needed_for_next(&days, period, EddingtonUnit::Kilometres, 1),
        0
    );
    // From 2 to 3: two days at or above 3 km already exist, so one more is
    // needed.
    assert_eq!(
        days_needed_for_next(&days, period, EddingtonUnit::Kilometres, 2),
        1
    );
}

#[test]
fn the_number_is_counted_in_the_chosen_unit() {
    // Ten days of exactly 8 km.
    let days: Vec<DailyDistance> = (0..10)
        .map(|offset| day(&format!("2026-01-{:02}", offset + 1), 8.0))
        .collect();
    let period = period("2026-01-01", "2026-01-10");

    let kilometres = eddington(&days, period, EddingtonUnit::Kilometres);
    assert_eq!(kilometres.number, 8);
    // Nine days at or above 9 km are needed; none exist.
    assert_eq!(kilometres.days_needed_for_next, 9);

    let miles = eddington(&days, period, EddingtonUnit::Miles);
    // 8 km is about 4.97 miles: ten days reach 4 miles, none reach 5.
    assert_eq!(miles.number, 4);
    // Five days at or above 5 miles are needed; none exist.
    assert_eq!(miles.days_needed_for_next, 5);

    // The longest day is a distance and stays in metres in either unit.
    assert_eq!(kilometres.longest_day_m, Some(8000.0));
    assert_eq!(miles.longest_day_m, Some(8000.0));
}
