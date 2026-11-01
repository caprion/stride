//! Every sentence this crate shows is a sentence the voice allows.
//!
//! `docs/contract/voice.md` is the contract for the words, and
//! `stride-core`'s lint is the machine-checkable half of it. This crate
//! writes a few sentences of its own — the error details, the summary of a
//! planned session, the distance comparison, the Progress headline — and
//! every one of them goes through the core's lint here.
//!
//! The lint is the core's, not this crate's: a second list of banned words
//! is a second rule, and two rules drift.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_core::strings::lint::{self, Rule};
use stride_core::strings::StringKind;

/// Every sentence in the crate's own source is checked.
///
/// The check reads the crate's source and takes out every string literal
/// that reads like a sentence. A literal that breaks the voice's rules
/// fails here, whether it is an error detail, a label or a sentence the
/// router writes.
#[test]
fn every_sentence_in_the_crate_breaks_no_rule() {
    let sources = source_files();
    assert!(!sources.is_empty(), "the crate's source could not be read");
    for (path, text) in sources {
        for literal in literals(&text) {
            if !reads_like_a_sentence(&literal) {
                continue;
            }
            let violations = lint::check_text(StringKind::Status, None, &literal);
            assert!(
                violations.is_empty(),
                "the sentence {literal:?} in {path} breaks the voice: {violations:?}"
            );
        }
    }
}

/// The error details the router can produce break no rule.
#[test]
fn the_error_details_break_no_rule() {
    let harness = common::Harness::empty("voice-errors");
    let failures: Vec<(&str, &str, Option<&str>)> = vec![
        ("GET", "/activities/9999", None),
        ("GET", "/activities?cursor=not-a-cursor", None),
        ("GET", "/activities?limit=many", None),
        ("POST", "/plan/import", Some("{}")),
        ("POST", "/auth/garmin/personal-key", Some("{}")),
        ("POST", "/auth/garmin/personal-key/test", Some("{}")),
        ("POST", "/auth/garmin/fit-import", Some("{}")),
        ("GET", "/nothing/here", None),
        ("POST", "/activities", Some("{}")),
    ];
    for (method, path, body) in failures {
        let answer = harness.router.call(method, path, body);
        assert!(!answer.is_ok(), "the route {path} answered");
        let detail = answer.body["detail"].as_str().unwrap_or_default();
        let violations = lint::check_text(StringKind::Status, None, detail);
        assert!(
            violations.is_empty(),
            "the detail {detail:?} of {path} breaks the voice: {violations:?}"
        );
    }
}

/// The sentences the router writes for the plan break no rule.
#[test]
fn the_plan_sentences_break_no_rule() {
    let harness = common::Harness::empty("voice-plan");
    let body = harness.ok_get("/plan/week");
    let sessions = body["sessions"].as_array().unwrap();
    for session in sessions.iter() {
        let sentence = session["sentence"].as_str().unwrap_or_default();
        let violations = lint::check_text(StringKind::Option, None, sentence);
        assert!(
            violations.is_empty(),
            "the session sentence {sentence:?} breaks the voice: {violations:?}"
        );
    }
}

/// The distance comparison breaks no rule.
#[test]
fn the_distance_comparison_breaks_no_rule() {
    let harness = common::Harness::empty("voice-comparison");
    let body = harness.ok_get("/insights/today");
    if let Some(comparison) = body["distance_comparison"].as_str() {
        let violations = lint::check_text(StringKind::Status, None, comparison);
        assert!(
            violations.is_empty(),
            "the distance comparison {comparison:?} breaks the voice: {violations:?}"
        );
    }
}

/// The Progress headline breaks no rule.
#[test]
fn the_progress_headline_breaks_no_rule() {
    let harness = common::Harness::empty("voice-progress");
    let body = harness.ok_get("/insights/progress");
    for field in ["headline", "headline_note"] {
        if let Some(text) = body[field].as_str() {
            let violations = lint::check_text(StringKind::Status, None, text);
            assert!(
                violations.is_empty(),
                "the {field} {text:?} breaks the voice: {violations:?}"
            );
        }
    }
}

/// The direction words that travel are the closed set.
#[test]
fn the_direction_words_are_the_closed_set() {
    let harness = common::Harness::empty("voice-direction");
    for date in [
        "2026-05-07",
        "2026-05-08",
        "2026-05-09",
        "2026-05-10",
        "2026-05-11",
    ] {
        harness.write_wellness(common::wellness_with(
            date,
            Some(22_200.0),
            Some(48.0),
            Some(63.0),
        ));
    }
    harness.write_wellness(common::wellness_with(
        common::TODAY,
        Some(22_200.0),
        Some(48.0),
        Some(63.0),
    ));
    let body = harness.ok_get("/physiology/vitals");
    for vital in body["vitals"].as_array().unwrap().iter() {
        if let Some(word) = vital["direction_text"].as_str() {
            assert!(
                stride_core::strings::DIRECTION_SET.contains(&word),
                "{word:?} is not one of the three direction words"
            );
            let violations = lint::check_text(
                StringKind::Direction,
                Some(stride_core::strings::DirectionRole::Primary),
                word,
            );
            assert!(
                violations.is_empty(),
                "the direction word {word:?} breaks the voice: {violations:?}"
            );
        }
    }
}

/// The rules the lint knows, so that the test above is not vacuous.
#[test]
fn the_lint_is_armed() {
    let violations = lint::check_text(StringKind::Status, None, "you should rest today");
    assert!(
        violations
            .iter()
            .any(|violation| violation.rule == Rule::BannedWord),
        "the lint did not catch a banned phrase"
    );
}

/// The crate's source files.
fn source_files() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out: Vec<(String, String)> = Vec::new();
    let mut stack = vec![root];
    while let Some(directory) = stack.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let text = match std::fs::read_to_string(&path) {
                Ok(text) => text,
                Err(_) => continue,
            };
            out.push((path.display().to_string(), text));
        }
    }
    out.sort();
    out
}

/// Every string literal in a text.
fn literals(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut inside = false;
    let mut escaped = false;
    for character in text.chars() {
        if inside {
            if escaped {
                current.push(character);
                escaped = false;
                continue;
            }
            match character {
                '\\' => {
                    current.push(character);
                    escaped = true;
                }
                '"' => {
                    out.push(std::mem::take(&mut current));
                    inside = false;
                }
                other => current.push(other),
            }
            continue;
        }
        if character == '"' {
            inside = true;
            current.clear();
        }
    }
    out
}

/// Whether a literal reads like a sentence: a space, a letter, and no
/// path or key in it.
fn reads_like_a_sentence(literal: &str) -> bool {
    if !literal.contains(' ') {
        return false;
    }
    if !literal.chars().any(|character| character.is_alphabetic()) {
        return false;
    }
    if literal.contains('/') && !literal.contains(' ') {
        return false;
    }
    true
}
