//! The Eddington number on the Progress route.
//!
//! The number is the largest *E* such that at least *E* days in the window
//! covered at least *E* kilometres. It is counted by the core
//! (`stride-core`'s `eddington` engine); this file holds the **route** to
//! it: the window it covers, the unit it is counted in, and the rule that
//! a window with no distance carries no number at all.
//!
//! Every value here is synthetic: invented distances on invented days.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

/// The day the harness reads the clock on.
const TODAY: &str = common::TODAY;

/// A history with a known number.
///
/// The daily distances are 12, 10, 8, 6, 4 and 2 kilometres: the largest
/// *E* with *E* days at or above *E* kilometres is **4** (four days at or
/// above 4 km, and the fifth-largest day — 4 km — is short of 5), and one
/// more day at 5 km or more would make it 5.
fn stepped_history(harness: &common::Harness) {
    for (index, (date, distance)) in [
        ("2026-05-04", 12_000.0),
        ("2026-05-05", 10_000.0),
        ("2026-05-06", 8_000.0),
        ("2026-05-07", 6_000.0),
        ("2026-05-08", 4_000.0),
        ("2026-05-09", 2_000.0),
    ]
    .iter()
    .enumerate()
    {
        harness.write_run(common::run(
            &format!("stepped-{index}"),
            date,
            *distance,
            1_800.0,
        ));
    }
}

#[test]
fn the_number_is_the_core_s_eddington_number_over_the_selected_window() {
    let harness = common::Harness::empty("eddington-number");
    stepped_history(&harness);

    let body = harness.ok_get("/insights/progress?weeks=2");
    let number = &body["eddington"];
    assert_eq!(number["number"], 4);
    assert_eq!(number["days_needed_for_next"], 1);
    assert_eq!(number["longest_day"]["text"], "12 km");
    assert_eq!(number["longest_day"]["raw"], 12_000.0);
}

#[test]
fn a_window_with_no_distance_has_no_number() {
    let harness = common::Harness::empty("eddington-empty-window");
    // A run outside the window: the window itself holds nothing.
    harness.write_run(common::run("older", "2026-04-01", 10_000.0, 3_600.0));

    let body = harness.ok_get("/insights/progress?weeks=1");
    assert!(
        body["eddington"].is_null(),
        "a window with no distance carries a number: {}",
        body["eddington"]
    );
    // The totals beside it are honest too: the run is outside the window.
    assert_eq!(body["period"]["sessions"], 0);
}

#[test]
fn a_store_with_no_runs_at_all_has_no_number() {
    let harness = common::Harness::empty("eddington-empty-store");
    let body = harness.ok_get("/insights/progress");
    assert!(body["eddington"].is_null());
}

#[test]
fn the_number_changes_with_the_window() {
    let harness = common::Harness::empty("eddington-window");
    stepped_history(&harness);
    // A long run on the Monday that opens the four-week window, and
    // outside the two-week one.
    harness.write_run(common::run("the-long-one", "2026-04-20", 30_000.0, 7_200.0));

    let two_weeks = harness.ok_get("/insights/progress?weeks=2");
    assert_eq!(two_weeks["eddington"]["number"], 4);
    assert_eq!(two_weeks["eddington"]["longest_day"]["text"], "12 km");
    assert_eq!(two_weeks["period"]["from"], "2026-05-04");

    let four_weeks = harness.ok_get("/insights/progress?weeks=4");
    assert_eq!(four_weeks["eddington"]["number"], 5);
    assert_eq!(four_weeks["eddington"]["longest_day"]["text"], "30 km");
    assert_eq!(four_weeks["period"]["from"], "2026-04-20");
}

#[test]
fn the_number_is_counted_in_kilometres() {
    let harness = common::Harness::empty("eddington-unit");
    // Five days of exactly one kilometre: in kilometres the number is 1
    // (one day at or above 1 km), and in miles it would be 0 — a kilometre
    // is a little over half a mile.
    for index in 0..5 {
        harness.write_run(common::run(
            &format!("one-kilometre-{index}"),
            &format!("2026-05-0{}", index + 4),
            1_000.0,
            600.0,
        ));
    }

    let body = harness.ok_get("/insights/progress?weeks=2");
    assert_eq!(body["eddington"]["number"], 1);
}

#[test]
fn the_days_are_the_daily_totals() {
    let harness = common::Harness::empty("eddington-daily-totals");
    // Two runs on one day: the day is 6 km, not two days of 3 km.
    harness.write_run(common::run("first-half", "2026-05-04", 3_000.0, 900.0));
    harness.write_run(common::run("second-half", "2026-05-04", 3_000.0, 900.0));
    for index in 0..3 {
        harness.write_run(common::run(
            &format!("short-{index}"),
            &format!("2026-05-0{}", index + 5),
            1_000.0,
            400.0,
        ));
    }

    let body = harness.ok_get("/insights/progress?weeks=2");
    // 6, 1, 1, 1 — one day at or above 1 km is enough for a number of 1,
    // and the second day never reaches 2 km.
    assert_eq!(body["eddington"]["number"], 1);
    assert_eq!(body["eddington"]["longest_day"]["text"], "6 km");
    assert_eq!(body["eddington"]["days_needed_for_next"], 1);
}

#[test]
fn the_number_covers_the_window_the_route_reports() {
    let harness = common::Harness::empty("eddington-window-agrees");
    stepped_history(&harness);

    let body = harness.ok_get("/insights/progress?weeks=3");
    // The window the totals name and the window the number counts are the
    // same one: the run on the 9th is inside the three-week window, and
    // the number counts it.
    assert_eq!(body["period"]["from"], "2026-04-27");
    assert_eq!(body["period"]["to"], TODAY);
    assert_eq!(body["eddington"]["number"], 4);
}
