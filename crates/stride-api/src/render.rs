//! The rendered value: the core's words, and the number for a chart.
//!
//! The contract's §8: *the DTOs carry the rendered value*. A domain value
//! crosses the wire as **both** — the text the core rendered ("8.4 km",
//! "42 m", "148 bpm") and the raw number a chart draws. There is no
//! `format:` hint: a hint tells the frontend *how* to render, which is the
//! hole §8 closes.
//!
//! ## This module is the only place that renders a domain value
//!
//! Every renderer here goes to `stride-core` for its words: the core's
//! types (`Distance::to_human`, `Duration::to_human`), the core's units
//! (`units::format_whole`) and the core's catalogue (the templates that
//! carry a unit). Nothing here writes a number into a sentence.
//!
//! **A value the core does not render is absent.** The core's renderers
//! for the pace and for an altitude are still to come; until they land the
//! field is `null` and the app shows nothing, because a second
//! implementation of "how do we say 5:02" is the same disease as a second
//! FIT decoder. When the core grows a renderer, this module is the one
//! line that changes.

use stride_core::strings;
use stride_core::types::{Distance, Duration};
use stride_core::units;

use crate::dto::RenderedValue;

/// A distance, as the core renders it.
///
/// The core renders a distance in kilometres: "8.4 km".
pub fn distance(metres: Option<f64>) -> Option<RenderedValue> {
    let metres = metres?;
    let distance = Distance::from_metres(metres)?;
    Some(RenderedValue {
        text: distance.to_human(),
        raw: Some(distance.metres()),
    })
}

/// A duration, as the core renders it.
///
/// The core renders a duration the way a person reads it: "6 h 10 m",
/// "42 m".
pub fn duration(seconds: Option<f64>) -> Option<RenderedValue> {
    let seconds = seconds?;
    let duration = Duration::from_seconds(seconds)?;
    Some(RenderedValue {
        text: duration.to_human(),
        raw: Some(duration.seconds()),
    })
}

/// A heart rate, as the core renders it.
///
/// The core's catalogue carries the unit — "{value} bpm" — and the core's
/// units carry the number. When the core grows a heart-rate renderer of
/// its own, this is the one line that changes.
pub fn heart_rate(bpm: Option<f64>) -> Option<RenderedValue> {
    let bpm = bpm?;
    let text = strings::render(
        "vital.resting_hr.value",
        &[("value", &units::format_whole(bpm))],
    )?;
    Some(RenderedValue {
        text,
        raw: Some(bpm),
    })
}

/// A heart-rate variability, as the core renders it.
///
/// The catalogue's own template: "{value} ms".
pub fn hrv(milliseconds: Option<f64>) -> Option<RenderedValue> {
    let milliseconds = milliseconds?;
    let text = strings::render(
        "vital.hrv.value",
        &[("value", &units::format_whole(milliseconds))],
    )?;
    Some(RenderedValue {
        text,
        raw: Some(milliseconds),
    })
}

/// A pace, as the core renders it.
///
/// **The core does not render a pace yet.** The contract's pace form is
/// "5:02 /km", and it belongs to the core's units with the rest of the
/// rounding table. Until the core has it, the pace is absent: the app
/// shows nothing rather than a second implementation of the core's
/// rounding.
pub fn pace(_seconds_per_km: Option<f64>) -> Option<RenderedValue> {
    None
}

/// An altitude, as the core renders it.
///
/// **The core does not render an altitude yet.** A climb is read in
/// metres, and the core's distance renderer reads kilometres — so there
/// is nothing to borrow, and the climb is absent until the core has a
/// renderer of its own.
pub fn altitude(_metres: Option<f64>) -> Option<RenderedValue> {
    None
}

/// A cadence, as the core renders it.
///
/// **The core does not render a cadence yet.**
pub fn cadence(_steps_per_minute: Option<f64>) -> Option<RenderedValue> {
    None
}

/// A number the core reads as a whole number: "48".
///
/// A count is not a measurement — it is a fact about how many things there
/// are — and the core's units read it.
pub fn whole(number: Option<f64>) -> Option<RenderedValue> {
    let number = number?;
    Some(RenderedValue {
        text: units::format_whole(number),
        raw: Some(number),
    })
}

/// A difference between two values, as the core renders it.
///
/// **The core does not render a difference yet.** A difference is signed,
/// and the core's renderers read a magnitude: there is nothing to borrow,
/// so the difference is absent until the core has a renderer of its own.
pub fn difference(_difference: Option<f64>) -> Option<RenderedValue> {
    None
}
