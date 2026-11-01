//! The golden assertions of the domain values' rendering.
//!
//! The values cross the wire as the text the core rendered and as the raw
//! number for a chart (contract §8), so the *text* is the core's: one
//! implementation of "how do we say 30 seconds", and no `format:` hint for
//! the frontend to act on.

use stride_core::units;

#[test]
fn a_distance_renders_in_the_apps_own_form() {
    // One decimal, with the decimals a person leaves out dropped.
    assert_eq!(
        units::render_distance(Some(8400.0)),
        Some("8.4 km".to_string())
    );
    assert_eq!(
        units::render_distance(Some(5000.0)),
        Some("5 km".to_string())
    );
}

#[test]
fn a_duration_renders_as_a_person_reads_it() {
    assert_eq!(
        units::render_duration(Some(2520.0)),
        Some("42 m".to_string())
    );
    assert_eq!(
        units::render_duration(Some(6.0 * 3600.0 + 10.0 * 60.0)),
        Some("6 h 10 m".to_string())
    );
    assert_eq!(
        units::render_duration(Some(3600.0)),
        Some("1 h".to_string())
    );
}

#[test]
fn a_heart_rate_is_truncated_never_rounded() {
    assert_eq!(
        units::render_heart_rate(Some(148.7)),
        Some("148 bpm".to_string())
    );
}

#[test]
fn a_pace_renders_as_a_clock() {
    assert_eq!(
        units::render_pace(Some(302.0)),
        Some("5:02 /km".to_string())
    );
    assert_eq!(
        units::render_pace(Some(324.0)),
        Some("5:24 /km".to_string())
    );
}

#[test]
fn an_elevation_renders_in_whole_metres() {
    assert_eq!(
        units::render_elevation(Some(128.0)),
        Some("128 m".to_string())
    );
}

#[test]
fn a_missing_value_renders_nothing_at_all() {
    assert_eq!(units::render_distance(None), None);
    assert_eq!(units::render_duration(None), None);
    assert_eq!(units::render_heart_rate(None), None);
    assert_eq!(units::render_pace(None), None);
    assert_eq!(units::render_elevation(None), None);
    // And a pace of zero is no value, never "0:00".
    assert_eq!(units::render_pace(Some(0.0)), None);
}

#[test]
fn a_rendered_value_is_a_catalogued_string() {
    // Every rendered value is a catalogue entry with the value already in it:
    // the API sends "8.4 km", not a template with a hole in it.
    assert_eq!(
        stride_core::strings::text("value.distance"),
        Some("{value} km")
    );
    assert_eq!(
        stride_core::strings::text("value.duration"),
        Some("{value}")
    );
    assert_eq!(
        stride_core::strings::text("value.heart_rate"),
        Some("{value} bpm")
    );
    assert_eq!(
        stride_core::strings::text("value.pace"),
        Some("{value} /km")
    );
    assert_eq!(
        stride_core::strings::text("value.elevation"),
        Some("{value} m")
    );
}
