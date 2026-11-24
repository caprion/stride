//! The plan's own reading: what the plan says about one week.
//!
//! Every sentence is the core's, in the runner's language, and every answer
//! names its source: the plan the athlete follows, the threshold it was
//! paced with, and the tier's entry requirement beside the athlete's own
//! mileage.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::plan::PlanSource;
use stride_core::plan_bases::{adopt, read_week, read_week_at, AdoptionInput, DateWords};
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
    }
}

#[test]
fn the_week_answers_with_its_seven_sessions_in_the_runner_s_language() {
    let plan = canonical_plan();
    let week = read_week(&plan, 3, &words()).expect("week three");
    assert_eq!(week.week, 3);
    assert_eq!(week.label, "3 weeks to go");
    assert_eq!(week.start_date, date("2026-12-28"));
    assert_eq!(week.end_date, date("2027-01-03"));
    assert_eq!(week.sessions.len(), 7);

    // The rest day: a name, and no distance.
    let monday = &week.sessions[0];
    assert_eq!(monday.weekday, "Monday");
    assert_eq!(monday.kind, "rest or cross-training");
    assert_eq!(
        monday.method_kind.as_deref(),
        Some("rest or cross-training")
    );
    assert_eq!(monday.distance, None);
    assert_eq!(monday.label, "rest or cross-training · no distance target");
    assert_eq!(monday.no_pace, None);

    // The general aerobic day: the runner's wording, and the method's own
    // name beside it.
    let tuesday = &week.sessions[1];
    assert_eq!(tuesday.kind, "easy");
    assert_eq!(tuesday.method_kind.as_deref(), Some("GA"));
    assert_eq!(tuesday.distance.as_deref(), Some("14.5 km"));
    assert_eq!(tuesday.label, "14.5 km · easy");
    // The band, and no pace: the reason says so.
    assert_eq!(tuesday.pace, None);
    assert_eq!(
        tuesday.no_pace.as_deref(),
        Some("the method fixes a heart-rate band for this run and no pace")
    );

    // The long run: the two ends, and the line that carries the first.
    let wednesday = &week.sessions[2];
    assert_eq!(wednesday.kind, "long run");
    assert_eq!(wednesday.method_kind.as_deref(), Some("endurance"));
    assert_eq!(wednesday.pace.as_deref(), Some("5:00–5:32 /km"));
    assert_eq!(wednesday.finish_pace.as_deref(), Some("4:59–5:37 /km"));
    assert_eq!(wednesday.label, "17.7 km · long run");
    assert_eq!(
        wednesday.line.as_deref(),
        Some("17.7 km · long run · 5:00–5:32 /km")
    );
    assert_eq!(wednesday.no_pace, None);

    // The intervals.
    let thursday = &week.sessions[3];
    assert_eq!(thursday.kind, "intervals");
    assert_eq!(thursday.method_kind.as_deref(), Some("VO₂max"));
    assert_eq!(thursday.pace.as_deref(), Some("3:59 /km"));
    assert_eq!(
        thursday.detail.as_deref(),
        Some("6 × 1,200 m @ 3K–5K race pace, jog 50–90 %")
    );

    // The recovery run.
    let friday = &week.sessions[4];
    assert_eq!(friday.kind, "recovery");
    assert_eq!(friday.pace.as_deref(), Some("5:36 /km"));

    // The general aerobic day with strides.
    let saturday = &week.sessions[5];
    assert_eq!(saturday.kind, "easy, with strides");
    assert_eq!(saturday.method_kind.as_deref(), Some("GA+speed"));
    assert_eq!(saturday.pace, None);

    // And the long run again.
    let sunday = &week.sessions[6];
    assert_eq!(sunday.kind, "long run");
    assert_eq!(sunday.pace.as_deref(), Some("5:00–5:32 /km"));
}

#[test]
fn the_week_names_the_plan_it_comes_from() {
    let plan = canonical_plan();
    let week = read_week(&plan, 3, &words()).expect("week three");
    assert_eq!(week.source.source, PlanSource::Prescribed);
    assert_eq!(week.source.line, "the Pfitzinger half marathon tier 2 plan");
    assert_eq!(week.source.template_id, "pfitzinger-half-marathon-tier-2");
    assert_eq!(week.source.template_version, "1");
}

#[test]
fn the_week_names_the_threshold_the_plan_was_paced_with() {
    let plan = canonical_plan();
    let week = read_week(&plan, 3, &words()).expect("week three");
    let threshold = week.pacing.threshold.as_ref().expect("a threshold");
    assert_eq!(threshold.value, "165 bpm");
    assert_eq!(threshold.date, date("2026-11-03"));
    assert_eq!(
        week.pacing.threshold_line.as_deref(),
        Some("paced with the threshold of 3 Nov 2026 · 165 bpm")
    );
    // And the rule the rest of the paces came from.
    let race = week.pacing.race.as_ref().expect("the race");
    assert_eq!(race.value, "10K");
    assert_eq!(race.date, date("2026-10-12"));
    assert_eq!(
        week.pacing.rule_line.as_deref(),
        Some("paced with the plan's rule on your 10K of 12 Oct 2026")
    );
    assert_eq!(week.pacing.missing, None);
}

#[test]
fn the_week_states_the_entry_requirement_beside_the_athletes_own_mileage() {
    let plan = canonical_plan();
    let week = read_week(&plan, 3, &words()).expect("week three");
    // The tier's entry requirement, as the book prints it.
    assert_eq!(
        week.entry.requirement.as_deref(),
        Some("this level asks for 64.4-80.5 km a week")
    );
    // The plan's own band.
    assert_eq!(
        week.entry.plan_band.as_deref(),
        Some("this plan runs 74-101.4 km a week")
    );
    // And the athlete's own mileage, unranked beside them.
    assert_eq!(
        week.entry.athlete.as_deref(),
        Some("your last 4 weeks: 60.8, 61.1, 50.8, 6 km")
    );
    // The three facts sit side by side, and none of them ranks another.
    let line = format!(
        "{} {} {}",
        week.entry.requirement.as_deref().unwrap_or_default(),
        week.entry.plan_band.as_deref().unwrap_or_default(),
        week.entry.athlete.as_deref().unwrap_or_default()
    );
    for banned in ["should", "recommend", "better", "need to", "fits"] {
        assert!(!line.contains(banned), "{line}");
    }
}

#[test]
fn the_week_states_its_own_mileage_and_the_reconciliation_of_its_days() {
    let plan = canonical_plan();
    let week = read_week(&plan, 3, &words()).expect("week three");
    // The book prints 63 miles; the core's own unit is the kilometre.
    assert_eq!(week.printed_volume.as_deref(), Some("101.4 km"));
    assert_eq!(
        week.volume.as_deref(),
        Some("the week's seven days add up to 101.4 km")
    );
}

#[test]
fn a_week_is_read_by_its_number_and_by_the_day_it_holds() {
    let plan = canonical_plan();
    let by_number = read_week(&plan, 0, &words()).expect("race week");
    let by_day = read_week_at(&plan, date("2027-01-24"), &words()).expect("race week");
    assert_eq!(by_number.week, 0);
    assert_eq!(by_day.week, 0);
    assert_eq!(by_number.label, "race week");
    assert_eq!(by_number.start_date, date("2027-01-18"));
    assert_eq!(by_number.end_date, date("2027-01-24"));
    // A day that is not in the plan has no week.
    assert!(read_week_at(&plan, date("2027-03-01"), &words()).is_none());
    assert!(read_week(&plan, 42, &words()).is_none());
}

#[test]
fn the_race_week_says_when_it_moved_and_what_is_not_in_it() {
    let mut input = AdoptionInput {
        programme_id: "pfitzinger-half-marathon-tier-2".to_string(),
        goal_distance_m: 21_097.5,
        race_date: date("2027-01-23"),
        adopted_on: date("2026-11-04"),
        start_date: None,
        threshold: None,
        race: Some(stride_core::training_paces::Race::new(
            10_000.0,
            2_496.0,
            date("2026-10-12"),
        )),
        weekly_mileage_m: Vec::new(),
    };
    input.start_date = None;
    let plan = adopt(&input).expect("the adoption");
    let week = read_week(&plan, 0, &words()).expect("race week");
    let shift = week.shift.as_ref().expect("a shift");
    assert_eq!(shift.days, -1);
    assert_eq!(
        shift.line.as_deref(),
        Some("your race is on Saturday · this week's sessions are earlier a day")
    );
    assert_eq!(
        shift.dropped,
        vec!["the Monday session is not in this week: rest or cross-training".to_string()]
    );
    // And no other week moved.
    let other = read_week(&plan, 3, &words()).expect("week three");
    assert!(other.shift.is_none());
}

#[test]
fn every_line_the_reading_renders_is_the_core_s_own_and_breaks_no_rule() {
    let plan = canonical_plan();
    for week in plan.weeks.iter() {
        let reading = read_week(&plan, week.week, &words()).expect("a week");
        let mut lines = vec![reading.source.line.clone()];
        lines.extend(reading.sessions.iter().map(|session| session.label.clone()));
        lines.extend(
            reading
                .sessions
                .iter()
                .filter_map(|session| session.line.clone()),
        );
        lines.extend(reading.sessions.iter().filter_map(|s| s.no_pace.clone()));
        lines.extend(reading.sessions.iter().filter_map(|s| s.lt_last.clone()));
        lines.extend(reading.sessions.iter().filter_map(|s| s.detail.clone()));
        lines.extend(reading.pacing.threshold_line.clone());
        lines.extend(reading.pacing.rule_line.clone());
        lines.extend(reading.entry.requirement.clone());
        lines.extend(reading.entry.plan_band.clone());
        lines.extend(reading.entry.athlete.clone());
        lines.extend(reading.volume.clone());
        lines.extend(reading.shift.as_ref().and_then(|s| s.line.clone()));
        lines.extend(
            reading
                .shift
                .as_ref()
                .map(|shift| shift.dropped.clone())
                .unwrap_or_default(),
        );
        for line in lines {
            assert!(
                strings::lint::check_rendered_statement(&line).is_empty(),
                "{line}"
            );
        }
    }
}
