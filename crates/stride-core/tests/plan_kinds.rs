//! The kinds of run: the method's own vocabulary, in the runner's wording.
//!
//! Two names live side by side and never both lead: the runner's wording is
//! what the surface prints, and the method's own name rides beside it. The
//! kind itself travels as data, because the ledger compares a session's kind
//! against the watch's own evidence.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::plan::SessionKind;
use stride_core::strings;

/// Every kind the catalogue holds, with the runner's wording and the
/// method's own name.
const KINDS: &[(SessionKind, &str, &str, &str)] = &[
    (
        SessionKind::RestCross,
        "rest_cross",
        "rest or cross-training",
        "rest or cross-training",
    ),
    (SessionKind::GeneralAerobic, "ga", "easy", "GA"),
    (
        SessionKind::GeneralAerobicSpeed,
        "ga_speed",
        "easy, with strides",
        "GA+speed",
    ),
    (SessionKind::Endurance, "endurance", "long run", "endurance"),
    (
        SessionKind::ProgressionLongRun,
        "progression_long_run",
        "long run, finishing fast",
        "progression long run",
    ),
    (SessionKind::Threshold, "lt", "tempo", "LT"),
    (SessionKind::Vo2Max, "vo2max", "intervals", "VO₂max"),
    (
        SessionKind::SpeedRepetitions,
        "speed_repetitions",
        "speed repetitions",
        "speed repetitions",
    ),
    (SessionKind::Recovery, "recovery", "recovery", "recovery"),
    (SessionKind::RacePace, "race_pace", "race pace", "race pace"),
    (
        SessionKind::RacePaceSpeed,
        "race_pace_speed",
        "race pace, with strides",
        "race pace",
    ),
    (
        SessionKind::TuneUpRace,
        "tune_up_race",
        "race",
        "the tune-up race",
    ),
    (SessionKind::GoalRace, "goal_race", "race", "the goal race"),
];

#[test]
fn every_kind_speaks_in_the_runner_s_language_and_names_the_method_beside_it() {
    for (kind, key, wording, method) in KINDS {
        assert_eq!(kind.word(), *wording, "{key}");
        assert_eq!(
            kind.method_name().as_deref(),
            Some(*method),
            "{key} names the method"
        );
        assert_eq!(kind.key(), *key);
    }
}

#[test]
fn a_kind_travels_under_its_key_and_comes_back_from_it() {
    for (kind, key, _, _) in KINDS {
        assert_eq!(SessionKind::from_key(key), Some(kind.clone()));
    }
    // A key the core does not know is no kind at all — never a kind the
    // athlete never chose.
    assert_eq!(SessionKind::from_key("tempo"), None);
    assert_eq!(SessionKind::from_key("unclassified"), None);
}

#[test]
fn the_apps_own_kinds_have_no_method_name() {
    assert_eq!(SessionKind::Easy.method_name(), None);
    assert_eq!(SessionKind::Long.method_name(), None);
    assert_eq!(SessionKind::Quality.method_name(), None);
    assert_eq!(SessionKind::Rest.method_name(), None);
    assert_eq!(SessionKind::Other("trail".to_string()).method_name(), None);
}

#[test]
fn only_the_rest_days_are_not_runs() {
    assert!(!SessionKind::Rest.is_run());
    assert!(!SessionKind::RestCross.is_run());
    for (kind, _, _, _) in KINDS {
        if kind.key() != "rest_cross" {
            assert!(kind.is_run(), "{}", kind.key());
        }
    }
}

#[test]
fn every_kind_word_is_a_catalogued_string_that_breaks_no_rule() {
    for (kind, key, _, _) in KINDS {
        let word = kind.word();
        assert!(!word.is_empty(), "{key}");
        assert!(
            strings::lint::check_rendered_statement(&word).is_empty(),
            "{key}"
        );
        let method = kind.method_name().expect("a method name");
        assert!(
            strings::lint::check_rendered_statement(&method).is_empty(),
            "{key}"
        );
    }
}

#[test]
fn the_catalogue_carries_every_kind_word_and_method_name() {
    for (_, key, _, _) in KINDS {
        let word_id = format!("statement.plan.kind.{key}");
        let method_id = format!("statement.plan.method.{key}");
        assert!(strings::entry(&word_id).is_some(), "{word_id}");
        assert!(strings::entry(&method_id).is_some(), "{method_id}");
    }
}
