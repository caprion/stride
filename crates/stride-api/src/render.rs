//! The rendered value: the core's words, and the number for a chart.
//!
//! The contract's §8: *the DTOs carry the rendered value*. A domain value
//! crosses the wire as **both** — the text the core rendered ("8.4 km",
//! "42 m", "148 bpm", "5:02 /km") and the raw number a chart draws. There
//! is no `format:` hint: a hint tells the frontend *how* to render, which
//! is the hole §8 closes.
//!
//! ## This module is the only place that renders a domain value
//!
//! Every renderer here is the **core's**: `stride-core`'s
//! `units::render_distance`, `render_duration`, `render_heart_rate`,
//! `render_pace` and `render_elevation` — the core's own functions, with
//! the core's rounding and the core's catalogue behind them. This crate
//! does not build a display string out of a number and a unit it added
//! itself: that is the second implementation the review found, and it is
//! the disease the contract is about.
//!
//! A value the core does not render is **absent**: the field is `null` and
//! the app shows nothing. Two values are in that state today — the
//! cadence and the difference between two values — and each is one line
//! here when the core grows its renderer.

use stride_core::strings;
use stride_core::units;

use crate::dto::RenderedValue;

/// A distance, as the core renders it: "8.4 km", "5 km".
pub fn distance(metres: Option<f64>) -> Option<RenderedValue> {
    let text = units::render_distance(metres)?;
    Some(RenderedValue { text, raw: metres })
}

/// A duration, as the core renders it: "6 h 10 m", "42 m".
pub fn duration(seconds: Option<f64>) -> Option<RenderedValue> {
    let text = units::render_duration(seconds)?;
    Some(RenderedValue { text, raw: seconds })
}

/// A heart rate, as the core renders it: "148 bpm".
///
/// The core truncates a heart rate to an integer (contract §3), and the
/// number beside the text is that integer.
pub fn heart_rate(bpm: Option<f64>) -> Option<RenderedValue> {
    let text = units::render_heart_rate(bpm)?;
    let raw = bpm.map(|bpm| units::truncate_heart_rate(bpm) as f64);
    Some(RenderedValue { text, raw })
}

/// A pace, as the core renders it: "5:02 /km".
pub fn pace(seconds_per_km: Option<f64>) -> Option<RenderedValue> {
    let text = units::render_pace(seconds_per_km)?;
    Some(RenderedValue {
        text,
        raw: seconds_per_km,
    })
}

/// An elevation, as the core renders it: "128 m".
pub fn elevation(metres: Option<f64>) -> Option<RenderedValue> {
    let text = units::render_elevation(metres)?;
    Some(RenderedValue { text, raw: metres })
}

/// A heart-rate variability, as the core renders it: "48 ms".
///
/// The core's catalogue carries the unit — "{value} ms" — and the core's
/// units carry the number, which is how the core's own renderers are
/// built. The core has no renderer of its own for a variability yet, and
/// this is the one line that changes when it has one.
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

/// A cadence, as the core renders it.
///
/// **The core does not render a cadence yet**, so the cadence is absent.
pub fn cadence(_steps_per_minute: Option<f64>) -> Option<RenderedValue> {
    None
}

/// The difference between two values, as the core renders it.
///
/// **The core does not render a difference yet.** A difference is signed,
/// and the core's renderers read a magnitude: there is nothing to borrow,
/// so the difference is absent until the core has a renderer of its own.
pub fn difference(_difference: Option<f64>) -> Option<RenderedValue> {
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
