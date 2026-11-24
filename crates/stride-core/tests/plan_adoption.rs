//! The adoption: a programme the athlete adopts, dated and paced once.
//!
//! The canonical athlete is the one the whole rig is written against
//! (`exploration/athlete.js`): the 10K in 41:36 on 12 October 2026, the
//! watch's own threshold of 165 bpm read on 3 November 2026, and the
//! half-marathon plan the athlete adopts on 4 November 2026 for a race on
//! Sunday 24 January 2027.
//!
//! Every date is an argument: there is no clock in these tests.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::plan::SessionKind;
use stride_core::plan_bases::{adopt, AdoptedPlan, AdoptionInput, AdoptionRefusal, Pacing};
use stride_core::threshold::{ThresholdBase, ThresholdChoice, ThresholdSource};
use stride_core::training_paces::Race;

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

/// The canonical athlete's own race: the 10K in 41:36 on 12 October 2026.
fn canonical_race() -> Race {
    Race::new(10_000.0, 2_496.0, date("2026-10-12"))
}

/// The threshold the athlete chose from the watch's own zones: 165 bpm,
/// read on 3 November 2026.
fn canonical_threshold() -> ThresholdChoice {
    ThresholdChoice {
        source: ThresholdSource::WatchZoneModel,
        heart_rate_bpm: Some(165.0),
        pace: None,
        rule_id: None,
        base: ThresholdBase::Pfitzinger,
        origin_date: Some(date("2026-11-03")),
        origin_ref: None,
        label: None,
        entered_on: date("2026-11-03"),
        updated_on: date("2026-11-03"),
        in_force: true,
    }
}

/// The canonical athlete's own recent weekly mileage, in metres: the four
/// weeks before the adoption, from the synthetic athlete's own run table.
fn canonical_mileage() -> Vec<f64> {
    vec![60_800.0, 61_100.0, 50_800.0, 6_000.0]
}

/// The canonical adoption.
fn canonical_input() -> AdoptionInput {
    AdoptionInput {
        programme_id: "pfitzinger-half-marathon-tier-2".to_string(),
        goal_distance_m: 21_097.5,
        race_date: date("2027-01-24"),
        adopted_on: date("2026-11-04"),
        start_date: Some(date("2026-11-02")),
        threshold: Some(canonical_threshold()),
        race: Some(canonical_race()),
        weekly_mileage_m: canonical_mileage(),
    }
}

/// The canonical adopted plan.
fn canonical() -> AdoptedPlan {
    adopt(&canonical_input()).expect("the canonical adoption")
}

#[test]
fn the_canonical_athlete_adopts_the_half_marathon_tier_two() {
    let plan = canonical();
    assert_eq!(plan.template_id, "pfitzinger-half-marathon-tier-2");
    assert_eq!(plan.template_version, "1");
    assert_eq!(plan.label, "half marathon tier 2");
    assert_eq!(plan.method, "Pfitzinger");
    // The race anchors the plan: the race week is the twelfth week, and
    // the plan begins eleven weeks before it.
    assert_eq!(plan.race_date, date("2027-01-24"));
    assert_eq!(plan.start_date, date("2026-11-02"));
    assert_eq!(plan.end_date, date("2027-02-07"));
    assert_eq!(plan.weeks.len(), 14);
    assert_eq!(plan.weeks[0].week, 11);
    assert_eq!(plan.weeks[11].week, 0);
    assert_eq!(plan.weeks[12].week, -1);
    assert_eq!(plan.weeks[13].week, -2);
    // And it is a prescription: the athlete chose it.
    assert_eq!(plan.source, stride_core::plan::PlanSource::Prescribed);
}

#[test]
fn the_canonical_week_three_is_seven_sessions_with_their_paces() {
    let plan = canonical();
    let week = plan
        .weeks
        .iter()
        .find(|week| week.week == 3)
        .expect("week three");
    assert_eq!(week.start_date, date("2026-12-28"));
    assert_eq!(week.end_date, date("2027-01-03"));
    assert_eq!(week.label, "3 weeks to go");
    assert_eq!(week.sessions.len(), 7);

    // Monday: a rest day, and not a run.
    let monday = &week.sessions[0];
    assert_eq!(monday.date, date("2026-12-28"));
    assert_eq!(monday.kind, SessionKind::RestCross);
    assert_eq!(monday.distance_m, None);
    assert_eq!(
        monday.pace,
        stride_core::training_paces::PaceValue::NoneStated
    );

    // Tuesday: the general aerobic day — the band, and no pace.
    let tuesday = &week.sessions[1];
    assert_eq!(tuesday.date, date("2026-12-29"));
    assert_eq!(tuesday.kind, SessionKind::GeneralAerobic);
    assert!((tuesday.distance_m.unwrap() - 9.0 * 1_609.344).abs() < 0.1);
    assert_eq!(
        tuesday.pace,
        stride_core::training_paces::PaceValue::NoneStated
    );

    // Wednesday: the long run, from the start pace to the finish pace.
    let wednesday = &week.sessions[2];
    assert_eq!(wednesday.date, date("2026-12-30"));
    assert_eq!(wednesday.kind, SessionKind::Endurance);
    assert!((wednesday.distance_m.unwrap() - 11.0 * 1_609.344).abs() < 0.1);
    assert_eq!(wednesday.pace.render(), Some("5:00–5:32 /km".to_string()));
    assert_eq!(
        wednesday.finish_pace.render(),
        Some("4:59–5:37 /km".to_string())
    );

    // Thursday: the intervals, at the 3K-to-5K race pace.
    let thursday = &week.sessions[3];
    assert_eq!(thursday.date, date("2026-12-31"));
    assert_eq!(thursday.kind, SessionKind::Vo2Max);
    assert!((thursday.distance_m.unwrap() - 11.0 * 1_609.344).abs() < 0.1);
    assert_eq!(thursday.pace.render(), Some("3:59 /km".to_string()));

    // Friday: the recovery run, at the recovery pace.
    let friday = &week.sessions[4];
    assert_eq!(friday.date, date("2027-01-01"));
    assert_eq!(friday.kind, SessionKind::Recovery);
    assert!((friday.distance_m.unwrap() - 6.0 * 1_609.344).abs() < 0.1);
    assert_eq!(friday.pace.render(), Some("5:36 /km".to_string()));

    // Saturday: the general aerobic day with strides — the band, and no
    // pace.
    let saturday = &week.sessions[5];
    assert_eq!(saturday.date, date("2027-01-02"));
    assert_eq!(saturday.kind, SessionKind::GeneralAerobicSpeed);
    assert_eq!(
        saturday.pace,
        stride_core::training_paces::PaceValue::NoneStated
    );

    // Sunday: the long run again.
    let sunday = &week.sessions[6];
    assert_eq!(sunday.date, date("2027-01-03"));
    assert_eq!(sunday.kind, SessionKind::Endurance);
    assert!((sunday.distance_m.unwrap() - 16.0 * 1_609.344).abs() < 0.1);
    assert_eq!(sunday.pace.render(), Some("5:00–5:32 /km".to_string()));
}

#[test]
fn the_plan_keeps_the_threshold_it_was_paced_with() {
    let plan = canonical();
    let threshold = plan.pacing.threshold.as_ref().expect("a threshold");
    assert_eq!(threshold.value, "165 bpm");
    assert_eq!(threshold.date, date("2026-11-03"));
    // And the race the plan's own rule was read from.
    let race = plan.pacing.race.as_ref().expect("the race");
    assert_eq!(race.value, "10K");
    assert_eq!(race.date, date("2026-10-12"));
}

#[test]
fn a_later_threshold_does_not_re_pace_the_plan_that_is_running() {
    // The plan was adopted on 4 November 2026, and the threshold in force
    // that day was the watch's own, read on 3 November.
    let plan = canonical();
    assert_eq!(plan.adopted_on, date("2026-11-04"));
    assert_eq!(
        plan.pacing.threshold.as_ref().map(|source| source.date),
        Some(date("2026-11-03"))
    );
    // A threshold chosen afterwards is a different fact, and reaching it
    // is an explicit act: the core reads the choice in force on the day
    // the plan was adopted.
    let choices = vec![
        canonical_threshold(),
        ThresholdChoice {
            source: ThresholdSource::Typed,
            heart_rate_bpm: Some(168.0),
            pace: None,
            rule_id: None,
            base: ThresholdBase::Pfitzinger,
            origin_date: Some(date("2026-11-20")),
            origin_ref: None,
            label: None,
            entered_on: date("2026-11-20"),
            updated_on: date("2026-11-20"),
            in_force: true,
        },
    ];
    let in_force = stride_core::threshold::in_force_at(&choices, date("2026-11-04"))
        .expect("a threshold in force");
    assert_eq!(in_force.entered_on, date("2026-11-03"));
    // And the plan's own paces are the ones worked out then.
    let week = plan.weeks.iter().find(|week| week.week == 3).unwrap();
    assert_eq!(week.sessions[3].pace.render(), Some("3:59 /km".to_string()));
}

#[test]
fn a_race_day_carries_the_races_own_distance_and_not_the_printed_miles() {
    let plan = canonical();
    let week = plan.weeks.iter().find(|week| week.week == 0).unwrap();
    let race = week
        .sessions
        .iter()
        .find(|session| session.kind == SessionKind::GoalRace)
        .expect("the goal race");
    assert_eq!(race.date, date("2027-01-24"));
    // The book prints 17 miles beside the race: that is the mileage the
    // day counts for in the week's total, not the race's distance.
    assert_eq!(race.distance_m, Some(21_097.5));
    // And the tune-up race, two weeks before, is the 10K the book prints.
    let week = plan.weeks.iter().find(|week| week.week == 2).unwrap();
    let tune_up = week
        .sessions
        .iter()
        .find(|session| session.kind == SessionKind::TuneUpRace)
        .expect("the tune-up race");
    assert_eq!(tune_up.distance_m, Some(10_000.0));
}

#[test]
fn a_race_on_another_day_moves_the_race_week_alone() {
    // The half-marathon programme's race is on the Sunday; this one is on
    // the Saturday.
    let mut input = canonical_input();
    input.race_date = date("2027-01-23");
    input.start_date = None;
    let plan = adopt(&input).expect("the adoption");
    // Every week but the race week stands where it was.
    for week in plan.weeks.iter().filter(|week| week.week != 0) {
        assert!(week.shift.is_none(), "week {} moved", week.week);
    }
    // The race week moved a day earlier, and the goal race fell on the
    // Saturday the athlete races.
    let race_week = plan.weeks.iter().find(|week| week.week == 0).unwrap();
    let shift = race_week.shift.as_ref().expect("a shift");
    assert_eq!(shift.days, -1);
    assert_eq!(shift.race_weekday, "Saturday");
    // The Monday's rest day fell outside the week, and the week says so.
    assert_eq!(shift.dropped.len(), 1);
    assert_eq!(shift.dropped[0].weekday, "Monday");
    assert_eq!(shift.dropped[0].kind, "rest or cross-training");
    let race = race_week
        .sessions
        .iter()
        .find(|session| session.kind == SessionKind::GoalRace)
        .expect("the goal race");
    assert_eq!(race.date, date("2027-01-23"));
    // The other weeks did not move: the plan still begins on the Monday
    // eleven weeks before the race week.
    assert_eq!(plan.start_date, date("2026-11-02"));
}

#[test]
fn the_adoption_is_refused_when_the_dates_cannot_both_be_true() {
    let mut input = canonical_input();
    input.start_date = Some(date("2026-10-26"));
    let refusal = adopt(&input).expect_err("the adoption is refused");
    match &refusal {
        AdoptionRefusal::StartDate {
            expected,
            given,
            race,
        } => {
            assert_eq!(*expected, date("2026-11-02"));
            assert_eq!(*given, date("2026-10-26"));
            assert_eq!(*race, date("2027-01-24"));
        }
        other => panic!("the wrong refusal: {other:?}"),
    }
    let reason = refusal.reason().expect("a reason");
    assert!(reason.contains("2026-11-02"), "{reason}");
    assert!(reason.contains("2026-10-26"), "{reason}");
}

#[test]
fn the_adoption_is_refused_when_the_goal_race_is_not_one_the_programme_prints() {
    let mut input = canonical_input();
    input.goal_distance_m = 42_195.0;
    let refusal = adopt(&input).expect_err("the adoption is refused");
    match &refusal {
        AdoptionRefusal::GoalDistance { printed, given_m } => {
            assert_eq!(printed, &vec!["half marathon".to_string()]);
            assert_eq!(*given_m, 42_195.0);
        }
        other => panic!("the wrong refusal: {other:?}"),
    }
    assert!(refusal
        .reason()
        .expect("a reason")
        .contains("half marathon"));
}

#[test]
fn the_adoption_is_refused_when_the_programme_is_not_in_the_catalogue() {
    let mut input = canonical_input();
    input.programme_id = "pfitzinger-eighty-twenty-tier-1".to_string();
    let refusal = adopt(&input).expect_err("the adoption is refused");
    assert!(matches!(refusal, AdoptionRefusal::UnknownProgramme { .. }));
    assert!(refusal.reason().expect("a reason").contains("catalogue"));
}

#[test]
fn without_a_race_the_sessions_have_no_pace_and_say_why() {
    let mut input = canonical_input();
    input.race = None;
    let plan = adopt(&input).expect("the adoption");
    let week = plan.weeks.iter().find(|week| week.week == 3).unwrap();
    // The threshold pace is not there either: the threshold carries a
    // heart rate and no pace, and a heart rate is not a pace.
    let wednesday = &week.sessions[2];
    assert_eq!(
        wednesday.pace,
        stride_core::training_paces::PaceValue::Missing(
            stride_core::training_paces::MissingInput::NoRaceTime
        )
    );
    // And the plan says why.
    let missing = plan.pacing.missing.as_ref().expect("a reason");
    assert!(missing.contains("no race time entered"), "{missing}");
}

#[test]
fn a_threshold_that_carries_a_pace_paces_the_threshold_session() {
    let mut input = canonical_input();
    input.threshold = Some(ThresholdChoice {
        source: ThresholdSource::Race,
        heart_rate_bpm: None,
        pace: Some(stride_core::training_paces::PaceBand::between(250.0, 258.0)),
        rule_id: Some("book.lt_pace_band".to_string()),
        base: ThresholdBase::Pfitzinger,
        origin_date: Some(date("2026-10-12")),
        origin_ref: None,
        label: None,
        entered_on: date("2026-10-12"),
        updated_on: date("2026-10-12"),
        in_force: true,
    });
    let plan = adopt(&input).expect("the adoption");
    // The threshold session is paced at the threshold's own band. Week 11
    // is the tempo run.
    let week = plan.weeks.iter().find(|week| week.week == 11).unwrap();
    let tempo = week
        .sessions
        .iter()
        .find(|session| session.kind == SessionKind::Threshold)
        .expect("the tempo run");
    assert_eq!(tempo.pace.render(), Some("4:10–4:18 /km".to_string()));
    // And the progression long run finishes at the same band.
    let week = plan.weeks.iter().find(|week| week.week == 9).unwrap();
    let long = week
        .sessions
        .iter()
        .find(|session| session.kind == SessionKind::ProgressionLongRun)
        .expect("the progression long run");
    assert_eq!(long.finish_pace.render(), Some("4:10–4:18 /km".to_string()));
    assert_eq!(long.lt_last_m, Some(2.0 * 1_609.344));
    // And the plan says which threshold it was paced with.
    let threshold = plan.pacing.threshold.as_ref().expect("a threshold");
    assert_eq!(threshold.value, "4:10–4:18 /km");
    assert_eq!(threshold.date, date("2026-10-12"));
}

#[test]
fn the_pacing_record_names_both_the_threshold_and_the_rule() {
    let plan = canonical();
    let pacing: &Pacing = &plan.pacing;
    assert!(pacing.threshold.is_some());
    assert!(pacing.race.is_some());
    assert!(pacing.missing.is_none());
}

#[test]
fn the_plan_keeps_the_entry_requirement_as_it_was_at_the_adoption() {
    let plan = canonical();
    let entry = plan.entry_mileage.as_ref().expect("an entry requirement");
    // The book prints 40–50 miles a week; the core's own unit is metres.
    assert!((entry.min_m - 40.0 * 1_609.344).abs() < 0.1);
    assert!((entry.max_m - 50.0 * 1_609.344).abs() < 0.1);
    assert_eq!(entry.printed, None);
    // And the athlete's own mileage is the plan's own fact.
    assert_eq!(plan.athlete_mileage_m, canonical_mileage());
}

#[test]
fn the_five_k_and_the_mile_are_the_ones_the_method_names() {
    // The VO₂max pace is the 3K-to-5K race pace, and the speed
    // repetitions are at the 800 m-to-mile race pace.
    let plan = canonical();
    let week = plan.weeks.iter().find(|week| week.week == 3).unwrap();
    assert_eq!(week.sessions[3].pace.render(), Some("3:59 /km".to_string()));
    // The five-kilometre equivalent of the canonical 10K is 19:57, which
    // is the 3:59 /km the intervals carry.
    assert!((stride_core::vo2::FIVE_K - 5_000.0).abs() < 0.1);
}

#[test]
fn the_catalogue_names_the_tier_of_every_programme() {
    use stride_core::plan_bases::programmes;
    let tiers: Vec<String> = programmes()
        .iter()
        .map(|programme| programme.tier())
        .collect();
    assert_eq!(
        tiers,
        vec![
            "tier 1".to_string(),
            "tier 2".to_string(),
            "tier 1".to_string(),
            "tier 2".to_string(),
        ]
    );
}

#[test]
fn a_programme_is_named_by_its_id_and_its_tier() {
    use stride_core::plan_bases::{programme_of, AdoptionRefusal};

    // The id on its own.
    let programme = programme_of("pfitzinger-half-marathon-tier-2", None).expect("a programme");
    assert_eq!(programme.id, "pfitzinger-half-marathon-tier-2");

    // The id with the tier beside it.
    let programme = programme_of("pfitzinger-half-marathon", Some("tier 2")).expect("a programme");
    assert_eq!(programme.id, "pfitzinger-half-marathon-tier-2");

    // The tier as a number.
    let programme = programme_of("pfitzinger-half-marathon", Some("2")).expect("a programme");
    assert_eq!(programme.id, "pfitzinger-half-marathon-tier-2");

    // A tier the programme does not have.
    let refusal = programme_of("pfitzinger-half-marathon-tier-2", Some("tier 3"))
        .expect_err("the tier is refused");
    match &refusal {
        AdoptionRefusal::Tier { id, tier } => {
            assert_eq!(id, "pfitzinger-half-marathon-tier-2");
            assert_eq!(tier, "tier 2");
        }
        other => panic!("the wrong refusal: {other:?}"),
    }
    let reason = refusal.reason().expect("a reason");
    assert_eq!(
        reason,
        "pfitzinger-half-marathon-tier-2 is the tier 2 programme"
    );

    // A programme the catalogue does not hold.
    let refusal =
        programme_of("pfitzinger-80-20-tier-1", None).expect_err("the programme is refused");
    let reason = refusal.reason().expect("a reason");
    assert_eq!(
        reason,
        "there is no programme called pfitzinger-80-20-tier-1 in the catalogue"
    );
}

#[test]
fn a_stored_session_keeps_the_pace_it_was_worked_out_with() {
    use stride_core::plan_bases::stored_pace;

    // A session with a pace: the band it was stored with.
    let pace = stored_pace(&SessionKind::Threshold, Some(239.0), Some(261.0));
    assert_eq!(pace.render(), Some("3:59–4:21 /km".to_string()));

    // The general aerobic day: the method fixes a band and states no pace.
    let pace = stored_pace(&SessionKind::GeneralAerobic, None, None);
    assert!(matches!(
        pace,
        stride_core::training_paces::PaceValue::NoneStated
    ));

    // A run whose pace could not be worked out: a value with a reason.
    let pace = stored_pace(&SessionKind::Endurance, None, None);
    assert!(matches!(
        pace,
        stride_core::training_paces::PaceValue::Missing(_)
    ));
}
