//! The golden assertions of the plan engine.
//!
//! The engine is a pure function of the history and the facts handed to it.
//! There is no clock in these tests: the day a week belongs to is explicit.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::plan::{
    adherence_of, baseline_from_history, baseline_not_enough_history, day_label,
    distance_comparison, distance_comparison_line, results_of, session_label, source_banner,
    today_of, week_of, Adherence, PlanBlock, PlanSession, PlanSource, RunFact, SessionKind,
    SessionStatus,
};
use stride_core::strings::{self, StringKind};

fn date(text: &str) -> Date {
    Date::parse(text).expect("a test date")
}

/// Eight weeks of the same three-session shape.
fn usual_history() -> Vec<RunFact> {
    let start = date("2025-12-29");
    let mut runs = Vec::new();
    for week in 0..8i64 {
        runs.push(RunFact::new(start.add_days(week * 7 + 1), Some(10_000.0)));
        runs.push(RunFact::new(start.add_days(week * 7 + 3), Some(8_000.0)));
        runs.push(RunFact::new(start.add_days(week * 7 + 6), Some(15_000.0)));
    }
    runs
}

#[test]
fn a_baseline_is_a_known_week_and_states_the_history_it_used() {
    let baseline = baseline_from_history(&usual_history(), 8).expect("a baseline");
    assert_eq!(baseline.source, PlanSource::Baseline);
    assert_eq!(
        baseline.name,
        "your usual week, inferred from your history (8 weeks)"
    );
    assert_eq!(baseline.sessions.len(), 3);

    assert_eq!(baseline.sessions[0].date, date("2026-02-17"));
    assert_eq!(baseline.sessions[0].kind, SessionKind::Quality);
    assert_eq!(baseline.sessions[0].target_distance_m, Some(10_000.0));
    assert_eq!(baseline.sessions[0].label, "10 km · quality");

    assert_eq!(baseline.sessions[1].date, date("2026-02-19"));
    assert_eq!(baseline.sessions[1].kind, SessionKind::Easy);
    assert_eq!(baseline.sessions[1].target_distance_m, Some(8_000.0));
    assert_eq!(baseline.sessions[1].label, "8 km · easy");

    assert_eq!(baseline.sessions[2].date, date("2026-02-22"));
    assert_eq!(baseline.sessions[2].kind, SessionKind::Long);
    assert_eq!(baseline.sessions[2].target_distance_m, Some(15_000.0));
    assert_eq!(baseline.sessions[2].label, "15 km · long");
}

#[test]
fn a_thin_history_infers_nothing_and_says_so() {
    let mut history = usual_history();
    history.truncate(12); // Four weeks, twelve runs.
    assert!(baseline_from_history(&history, 8).is_none());
    assert_eq!(
        baseline_not_enough_history(),
        "not enough history to infer a usual week"
    );
}

#[test]
fn the_week_is_seven_days_in_order_with_the_right_intent() {
    let start = date("2026-05-11");
    let kinds = [
        SessionKind::Easy,
        SessionKind::Long,
        SessionKind::Quality,
        SessionKind::Rest,
        SessionKind::Other("trail".to_string()),
        SessionKind::Easy,
        SessionKind::Long,
    ];
    let sessions: Vec<PlanSession> = kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| {
            PlanSession::new(
                start.add_days(index as i64),
                kind.clone(),
                Some((index as f64 + 1.0) * 1_000.0),
            )
        })
        .collect();
    let block = PlanBlock::new(PlanSource::Prescribed, "A plan".to_string(), sessions);

    let week = week_of(&block, date("2026-05-13"));
    assert_eq!(week.len(), 7);
    for (index, session) in week.iter().enumerate() {
        assert_eq!(session.date, start.add_days(index as i64));
        assert_eq!(session.kind, kinds[index]);
        assert_eq!(
            session.target_distance_m,
            Some((index as f64 + 1.0) * 1_000.0)
        );
    }
    assert_eq!(
        today_of(&block, date("2026-05-13")).map(|session| session.kind.clone()),
        Some(SessionKind::Quality)
    );
    assert!(today_of(&block, date("2026-05-18")).is_none());
}

#[test]
fn a_session_without_a_target_says_that_it_has_no_target() {
    let session = PlanSession::new(date("2026-05-11"), SessionKind::Easy, None);
    assert_eq!(session.target_distance_m, None);
    assert_eq!(session.label, "easy · no distance target");
    assert_eq!(session_label(&session), "easy · no distance target");
}

#[test]
fn adherence_is_a_count_and_a_baseline_never_has_a_shortfall() {
    let sessions = vec![
        PlanSession::new(date("2026-05-01"), SessionKind::Easy, Some(8_000.0)),
        PlanSession::new(date("2026-05-02"), SessionKind::Long, Some(15_000.0)),
        PlanSession::new(date("2026-05-03"), SessionKind::Quality, Some(10_000.0)),
    ];
    let actual = vec![
        RunFact::new(date("2026-05-01"), Some(8_000.0)),
        RunFact::new(date("2026-05-03"), Some(10_200.0)),
    ];

    let prescribed = PlanBlock::new(
        PlanSource::Prescribed,
        "A plan".to_string(),
        sessions.clone(),
    );
    let adherence = adherence_of(&prescribed, &actual);
    assert_eq!(adherence.planned, 3);
    assert_eq!(adherence.done, 2);
    assert_eq!(adherence.not_done, 1);
    assert_eq!(adherence.line().as_deref(), Some("2 of 3 sessions run"));

    let baseline = PlanBlock::new(PlanSource::Baseline, "Usual".to_string(), sessions);
    let adherence = adherence_of(&baseline, &actual);
    assert_eq!(adherence.planned, 3);
    assert_eq!(adherence.done, 2);
    assert_eq!(adherence.not_done, 0);
    assert_eq!(
        adherence.line_for(PlanSource::Baseline).as_deref(),
        Some("2 sessions in your usual week")
    );
}

#[test]
fn a_result_states_the_actual_and_the_target_neutrally() {
    let session = PlanSession::new(date("2026-05-01"), SessionKind::Easy, Some(10_000.0));
    let block = PlanBlock::new(PlanSource::Prescribed, "A plan".to_string(), vec![session]);
    let actual = vec![RunFact::new(date("2026-05-01"), Some(8_400.0))];
    let results = results_of(&block, &actual);
    assert_eq!(results.len(), 1);
    assert!(results[0].ran);
    assert_eq!(results[0].actual, Some(8_400.0));
    assert_eq!(results[0].line.as_deref(), Some("8.4 km of a 10 km target"));
    assert!(results[0].difference.is_none());
    assert_eq!(results[0].status, SessionStatus::Done);
}

#[test]
fn a_baseline_result_never_says_not_done() {
    let session = PlanSession::new(date("2026-05-01"), SessionKind::Easy, Some(10_000.0));
    let block = PlanBlock::new(PlanSource::Baseline, "Usual".to_string(), vec![session]);
    let results = results_of(&block, &[]);
    assert_eq!(results[0].status, SessionStatus::Planned);
    assert!(!results[0].ran);
}

#[test]
fn the_distance_comparison_is_only_news_beyond_ten_per_cent() {
    assert_eq!(
        distance_comparison(Some(12_400.0), Some(10_000.0)).as_deref(),
        Some("2.4 km more than the 10 km target")
    );
    assert_eq!(
        distance_comparison_line(Some(12_400.0), Some(10_000.0)).as_deref(),
        Some("12.4 km · 2.4 km more than the 10 km target")
    );
    assert!(distance_comparison(Some(10_200.0), Some(10_000.0)).is_none());
    assert!(distance_comparison(Some(8_000.0), Some(10_000.0)).is_none());
    assert!(distance_comparison(None, Some(10_000.0)).is_none());
    assert!(distance_comparison(Some(12_400.0), None).is_none());
    assert!(distance_comparison(Some(12_400.0), Some(0.0)).is_none());
}

#[test]
fn every_plan_sentence_is_catalogued_and_carries_no_hole() {
    for id in [
        "statement.plan.source.baseline",
        "statement.plan.source.prescribed",
        "statement.plan.source.typed",
        "statement.plan.baseline.description",
        "statement.plan.baseline.not_enough_history",
        "statement.plan.kind.easy",
        "statement.plan.kind.long",
        "statement.plan.kind.quality",
        "statement.plan.kind.rest",
        "statement.plan.kind.other",
        "statement.plan.session.label",
        "statement.plan.session.no_target",
        "statement.plan.day.monday",
        "statement.plan.day.tuesday",
        "statement.plan.day.wednesday",
        "statement.plan.day.thursday",
        "statement.plan.day.friday",
        "statement.plan.day.saturday",
        "statement.plan.day.sunday",
        "statement.plan.adherence",
        "statement.plan.adherence.baseline",
        "statement.plan.result.target",
        "statement.plan.result.no_target",
        "statement.plan.result.no_run",
        "statement.plan.result.empty",
        "statement.plan.status.done",
        "statement.plan.status.planned",
        "statement.plan.status.not_done",
    ] {
        let entry = strings::entry(id).expect("a catalogue entry");
        assert_eq!(entry.kind, StringKind::Statement, "{id}");
        assert!(strings::lint::check_entry(entry).is_empty(), "{id}");
    }

    let source = source_banner(PlanSource::Baseline);
    assert_eq!(source, "your usual week, inferred from your history");
    assert!(strings::lint::check_rendered_statement(&source).is_empty());
    assert_eq!(day_label(date("2026-05-11")), "Monday");
    assert!(strings::lint::check_rendered_statement("Monday").is_empty());

    let adherence = Adherence {
        planned: 8,
        done: 7,
        not_done: 1,
    };
    let line = adherence.line().expect("an adherence line");
    assert_eq!(line, "7 of 8 sessions run");
    assert!(strings::lint::check_rendered_statement(&line).is_empty());
}
