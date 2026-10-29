//! The plan importer, against the reference.
//
//! The fixture holds five documents and what the Python importer made of
//! them, recorded from `backend/app/ingest/plan_importer.py`. The port has to
//! agree with it exactly — the quirks included, because the importer is what
//! the athlete's plan becomes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::Value;
use stride_api::plan_import::import_plan;

/// The recorded documents and the reference's answers.
const FIXTURE: &str = include_str!("fixtures/plan_import.json");

/// Every document imports the way the reference imports it.
#[test]
fn the_importer_agrees_with_the_reference() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture parses");
    let cases = fixture["cases"].as_array().expect("the cases");
    assert!(cases.len() >= 5, "the fixture holds five documents");
    for case in cases {
        let name = case["name"].as_str().expect("a name");
        let format = case["format"].as_str().expect("a format");
        let content = case["content"].as_str().expect("the document");
        let expected = &case["expected"];
        let actual = import_plan(content, format, None, 2026).expect("the document imports");
        assert_eq!(actual, *expected, "the {name} document imports differently");
    }
}

/// An unknown format is refused, the way the reference refuses it.
#[test]
fn an_unknown_format_is_refused() {
    let err = import_plan("whatever", "yaml", None, 2026).expect_err("it is refused");
    assert_eq!(err, "format must be markdown, csv, or json");
}

/// The paces and the distances, the two parsers the sessions depend on.
#[test]
fn the_paces_and_the_distances_are_read() {
    use stride_api::plan_import::{parse_distance, parse_pace};

    assert_eq!(parse_pace(Some("4:30-4:45/km")), (Some(270), Some(285)));
    assert_eq!(parse_pace(Some("4:30/km")), (Some(270), Some(270)));
    assert_eq!(parse_pace(Some("no pace here")), (None, None));
    assert_eq!(parse_pace(None), (None, None));

    assert_eq!(parse_distance(Some("12 km")), Some(12_000.0));
    assert_eq!(parse_distance(Some("6-8km")), Some(6_000.0));
    assert_eq!(parse_distance(Some("5k")), Some(5_000.0));
    assert_eq!(parse_distance(Some("half marathon")), Some(21_097.5));
    assert_eq!(parse_distance(Some("4:45-5:00/km")), None);
    assert_eq!(parse_distance(None), None);
}
