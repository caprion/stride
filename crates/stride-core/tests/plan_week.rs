//! The plan's week against the week that happened.
//!
//! The card answers one question — how is this week against the plan? —
//! and it answers it as fact: a session is done or not done, a distance
//! is compared with the distance that was planned, and a pace is compared
//! with the plan's pace **only when both of them exist**.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::plan::SessionStatus;
use stride_core::plan_bases::{adopt, AdoptionInput, DateWords};
use stride_core::plan_week::{compare_week, WeekFact, WeekState};
use stride_core::strings;

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

/// The canonical athlete's adoption: the half-marathon tier 2, the race on
/// Sunday 24 January 2027, and the watch's own threshold of 165 bpm read on
/// 3 November 2026.
fn canonical_plan() -> stride_core::plan_bases::AdoptedPlan {
    let input = AdoptionInput {
        programme_id: "pfitzinger-half-marathon-tier-2".to_string(),
        goal_distance_m: 21_097.5,
        race_date: date("2027-01-24"),
        adopted_on: date("2026-11-04"),
        start_date: Some(date("2026-11-02")),
        threshold: Some(stride_core::threshold::ThresholdChoice {
            source: stride_core::threshold::ThresholdSource::WatchZoneModel,
            heart_rate_bpm: Some(165.0),
            pace: None,
            rule_id: None,
            base: stride_core::threshold::ThresholdBase::Pfitzinger,
            origin_date: Some(date("2026-11-03")),
            origin_ref: None,
            label: None,
            entered_on: date("2026-11-03"),
            updated_on: date("2026-11-03"),
            in_force: true,
        }),
        race: Some(stride_core::training_paces::Race::new(
            10_000.0,
            2_496.0,
            date("2026-10-12"),
        )),
        weekly_mileage_m: vec![60_800.0, 61_100.0, 50_800.0, 6_000.0],
    };
    adopt(&input).expect("the canonical adoption")
}

/// The caller's own words for the days the plan's lines carry.
fn words() -> DateWords {
    DateWords {
        threshold: Some("3 Nov 2026".to_string()),
        race: Some("12 Oct 2026".to_string()),
        start: Some("2 Nov 2026".to_string()),
        end: Some("7 Feb 2027".to_string()),
    }
}

/// The week that happened: a run on the Tuesday and a run on the Wednesday
/// of the plan's third week, and nothing on the other days.
fn the_week_that_happened() -> Vec<WeekFact> {
    vec![
        WeekFact::new(date("2026-12-29"), Some(14_500.0), Some(330.0)),
        WeekFact::new(date("2026-12-30"), Some(15_000.0), Some(335.0)),
    ]
}

/// The Friday of the plan's third week: two sessions done, one not done,
/// and the rest of the week still to come.
fn friday() -> Date {
    date("2027-01-01")
}

#[test]
fn no_plan_adopted_says_so() {
    let comparison = compare_week(None, friday(), &[], &words());
    assert_eq!(comparison.state, WeekState::None);
    assert_eq!(
        comparison.state_line.as_deref(),
        Some("no plan has been adopted")
    );
    assert!(comparison.sessions.is_empty());
    assert!(comparison.totals.is_none());
    // And there is no plan to name.
    assert!(comparison.source.is_none());
    assert!(comparison.pacing.is_none());
    assert!(comparison.entry.is_none());
}

#[test]
fn a_plan_that_has_not_started_says_when_it_starts() {
    let plan = canonical_plan();
    // The day before the plan's first day.
    let comparison = compare_week(Some(&plan), date("2026-11-01"), &[], &words());
    assert_eq!(comparison.state, WeekState::Before);
    assert_eq!(
        comparison.state_line.as_deref(),
        Some("your plan starts on 2 Nov 2026 — this week comes before it")
    );
    // The plan is still named: it exists, and it has not begun.
    assert_eq!(
        comparison.source.map(|source| source.line),
        Some("the Pfitzinger half marathon tier 2 plan".to_string())
    );
    assert!(comparison.week.is_none());
    assert!(comparison.totals.is_none());
}

#[test]
fn a_plan_that_has_passed_says_when_it_ended() {
    let plan = canonical_plan();
    let comparison = compare_week(Some(&plan), date("2027-02-08"), &[], &words());
    assert_eq!(comparison.state, WeekState::After);
    assert_eq!(
        comparison.state_line.as_deref(),
        Some("your plan's last week ended on 7 Feb 2027 — this week comes after it")
    );
    assert!(comparison.week.is_none());
}

#[test]
fn every_planned_session_is_done_or_not_done_and_never_a_grade() {
    let plan = canonical_plan();
    let comparison = compare_week(Some(&plan), friday(), &the_week_that_happened(), &words());
    assert_eq!(comparison.state, WeekState::Running);
    assert_eq!(
        comparison.week.map(|week| week.label),
        Some("3 weeks to go".to_string())
    );
    assert_eq!(comparison.sessions.len(), 7);

    // The rest day: neither done nor not done.
    let monday = &comparison.sessions[0];
    assert_eq!(monday.date, date("2026-12-28"));
    assert_eq!(monday.status, None);

    // The easy run: done.
    let tuesday = &comparison.sessions[1];
    assert_eq!(tuesday.status, Some(SessionStatus::Done));
    assert_eq!(tuesday.actual_distance_m, Some(14_500.0));
    // Within ten per cent of the planned distance: there is no difference
    // to state.
    assert_eq!(tuesday.difference, None);

    // The long run: done, short of the plan, and slower than the plan's
    // pace.
    let wednesday = &comparison.sessions[2];
    assert_eq!(wednesday.status, Some(SessionStatus::Done));
    assert_eq!(
        wednesday.difference.as_deref(),
        Some("2.7 km less than the 17.7 km target")
    );
    assert_eq!(
        wednesday.pace.as_deref(),
        Some("5:35 /km · 3 s/km slower than the plan's 5:00–5:32 /km")
    );

    // The intervals: the day went by and the session was not run.
    let thursday = &comparison.sessions[3];
    assert_eq!(thursday.status, Some(SessionStatus::NotDone));
    assert_eq!(thursday.actual_distance_m, None);
    assert_eq!(thursday.difference, None);
    assert_eq!(thursday.pace, None);

    // The days still to come are planned, not not done.
    let friday = &comparison.sessions[4];
    assert_eq!(friday.status, Some(SessionStatus::Planned));
    let sunday = &comparison.sessions[6];
    assert_eq!(sunday.status, Some(SessionStatus::Planned));
}

#[test]
fn a_pace_is_compared_only_when_both_of_them_exist() {
    let plan = canonical_plan();
    let comparison = compare_week(Some(&plan), friday(), &the_week_that_happened(), &words());
    // The general aerobic day: the method fixes a band and states no
    // pace, so there is no pace to compare the run's pace against.
    let tuesday = &comparison.sessions[1];
    assert_eq!(tuesday.pace, None);
    // The run had a pace; the plan did not. Nothing is compared.
    let saturday = &comparison.sessions[5];
    assert_eq!(saturday.pace, None);
    assert_eq!(saturday.status, Some(SessionStatus::Planned));
}

#[test]
fn a_pace_inside_the_plans_band_says_so() {
    let plan = canonical_plan();
    // The long run at 5:10 /km, inside the plan's 5:00–5:32.
    let facts = vec![WeekFact::new(
        date("2026-12-30"),
        Some(17_700.0),
        Some(310.0),
    )];
    let comparison = compare_week(Some(&plan), friday(), &facts, &words());
    let wednesday = &comparison.sessions[2];
    assert_eq!(
        wednesday.pace.as_deref(),
        Some("5:10 /km · within the plan's 5:00–5:32 /km")
    );
    // And a pace faster than the plan's band.
    let facts = vec![WeekFact::new(
        date("2026-12-30"),
        Some(17_700.0),
        Some(290.0),
    )];
    let comparison = compare_week(Some(&plan), friday(), &facts, &words());
    let wednesday = &comparison.sessions[2];
    assert_eq!(
        wednesday.pace.as_deref(),
        Some("4:50 /km · 10 s/km faster than the plan's 5:00–5:32 /km")
    );
}

#[test]
fn the_totals_are_the_days_that_have_gone_on_both_sides() {
    let plan = canonical_plan();
    let comparison = compare_week(Some(&plan), friday(), &the_week_that_happened(), &words());
    let totals = comparison.totals.expect("the week's totals");
    // The plan asked for 9 miles on the Tuesday, 11 on the Wednesday and
    // 11 on the Thursday: the days that have gone. The Friday is still
    // running, and it is not counted against the athlete.
    assert!(
        (totals.planned_m - 49_889.664).abs() < 0.001,
        "{}",
        totals.planned_m
    );
    // The athlete ran 14.5 km and 15 km.
    assert_eq!(totals.actual_m, 29_500.0);
    assert!(
        (totals.difference_m + 20_389.664).abs() < 0.001,
        "{}",
        totals.difference_m
    );
    assert_eq!(
        totals.line.as_deref(),
        Some("29.5 km against the plan's 49.9 km")
    );
}

#[test]
fn the_card_names_the_plan_the_threshold_and_the_entry_requirement() {
    let plan = canonical_plan();
    let comparison = compare_week(Some(&plan), friday(), &the_week_that_happened(), &words());
    assert_eq!(
        comparison.source.map(|source| source.line),
        Some("the Pfitzinger half marathon tier 2 plan".to_string())
    );
    let pacing = comparison.pacing.expect("the plan's pacing");
    assert_eq!(
        pacing.threshold_line.as_deref(),
        Some("paced with the threshold of 3 Nov 2026 · 165 bpm")
    );
    let entry = comparison.entry.expect("the entry requirement");
    assert_eq!(
        entry.requirement.as_deref(),
        Some("this level asks for 64.4-80.5 km a week")
    );
    assert_eq!(
        entry.athlete.as_deref(),
        Some("your last 4 weeks: 60.8, 61.1, 50.8, 6 km")
    );
}

#[test]
fn every_line_the_card_prints_is_the_core_s_own_and_breaks_no_rule() {
    let plan = canonical_plan();
    for day in [
        date("2026-11-01"),
        date("2026-12-30"),
        friday(),
        date("2027-02-08"),
    ] {
        let comparison = compare_week(Some(&plan), day, &the_week_that_happened(), &words());
        let mut lines = vec![comparison.state_line.clone().unwrap_or_default()];
        lines.extend(
            comparison
                .totals
                .iter()
                .filter_map(|totals| totals.line.clone()),
        );
        lines.extend(
            comparison
                .sessions
                .iter()
                .filter_map(|session| session.difference.clone()),
        );
        lines.extend(
            comparison
                .sessions
                .iter()
                .filter_map(|session| session.pace.clone()),
        );
        for line in lines {
            assert!(
                strings::lint::check_rendered_statement(&line).is_empty(),
                "{line}"
            );
            for banned in ["missed", "should", "score", "%", "good", "bad"] {
                assert!(!line.contains(banned), "{line}");
            }
        }
    }
}
