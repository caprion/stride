//! The physiological-cost curve: the share of the maximum against time.
//!
//! The curve is the engine's own relation — **the share of VO₂max that a
//! duration holds**, against the time it is held for. It is a **second
//! reading** of the athlete, not a second ladder: no rungs, and no pace
//! bands repeated.
//!
//! ## What is on the curve
//!
//! * the **curve** — the relation itself, read over the durations the
//!   surface draws;
//! * the **athlete's own races** — the races the athlete typed, each at
//!   its own time and the share of their maximum that time held;
//! * the **method's own sessions** — the threshold session and the
//!   recovery run, each as a band over its duration and its prescribed
//!   pace. The long run's length comes with the plan's own weeks, and is
//!   **no value** until they land;
//! * the **field test's marker** — forty-five minutes, the test's own
//!   place in time;
//! * the **pace equivalents** — the equivalent race times, along the
//!   bottom.
//!
//! ## What it never does
//!
//! It never judges: nothing here says a session is too hard, nothing
//! ranks the athlete's races against the method's sessions, and nothing
//! draws a conclusion about the athlete's readiness for a distance. And it
//! never turns the method's heart-rate bands into %maxVO₂: the two are
//! different numbers, and converting one into the other needs the
//! athlete's lactate relation, which the app does not have.

use serde_json::Value;
use stride_core::training_paces::method_sessions;
use stride_core::vo2;

use crate::dto::{CurveInsight, CurvePointRow, CurveRaceRow, CurveSessionRow, RuleRow};
use crate::error::ApiResult;
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The durations the curve is read over, in minutes.
///
/// The curve's own points are these, and the field test's marker is one of
/// them.
const MINUTES: [f64; 9] = [5.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0];

/// The book's forty-five-minute time trial, in minutes.
const FIELD_TEST_MINUTES: f64 = 45.0;

/// The cost curve, over the Progress window.
pub fn pacing_curve(api: &Api, request: &Request) -> ApiResult<Value> {
    let (window, _, today) = super::insight_window(api, request)?;
    let facts = crate::athlete::Facts::read(api, today)?;
    let ladder = facts.ladder();
    let vdot = ladder.vdot;

    // The curve: the relation, read over the durations the surface draws.
    let curve: Vec<CurvePointRow> = match vdot {
        Some(vdot) => vo2::cost_curve(vdot, &MINUTES)
            .iter()
            .map(|point| CurvePointRow {
                minutes: point.minutes,
                percent_max: render::percent(Some(point.percent_max * 100.0)).unwrap_or(
                    crate::dto::RenderedValue {
                        text: String::new(),
                        raw: Some(point.percent_max * 100.0),
                    },
                ),
                pace: render::pace(point.pace_s_per_km),
            })
            .collect(),
        None => Vec::new(),
    };

    // The athlete's own races: every race the athlete typed, at its own
    // time and the share of their maximum that time held.
    let stored = api
        .store()
        .races()
        .map_err(|error| api.store_error("the races could not be read", &error))?;
    let races: Vec<CurveRaceRow> = stored
        .iter()
        .map(|race| {
            let minutes = race.time_s / 60.0;
            let pace = match race.distance_m > 0.0 && race.time_s > 0.0 {
                true => Some(race.time_s / (race.distance_m / 1_000.0)),
                false => None,
            };
            CurveRaceRow {
                label: race.label.clone(),
                time: stride_core::units::format_race_time(race.time_s),
                minutes,
                percent_max: render::percent(Some(vo2::percent_max_at(minutes) * 100.0)).unwrap_or(
                    crate::dto::RenderedValue {
                        text: String::new(),
                        raw: Some(vo2::percent_max_at(minutes) * 100.0),
                    },
                ),
                pace: render::pace(pace),
                date: race.race_date.clone(),
                date_label: super::core_date(&race.race_date)
                    .map(super::day_label)
                    .unwrap_or_default(),
            }
        })
        .collect();

    // The method's own sessions: the threshold session and the recovery
    // run, and the long run whose length comes with the plan's weeks.
    let sessions: Vec<CurveSessionRow> = method_sessions(&ladder)
        .iter()
        .map(|session| {
            // The share of the maximum the session's pace costs, at its
            // two ends: the curve's vertical axis, and the core's own
            // relation. **No value when the athlete's own fitness number
            // is not there** — every point on the curve is measured
            // against it.
            let (from_percent_max, to_percent_max) = match (vdot, session.pace.as_ref()) {
                (Some(vdot), Some(band)) => (
                    vo2::cost_of_pace(vdot, band.fast_s_per_km)
                        .and_then(|percent| render::percent(Some(percent))),
                    vo2::cost_of_pace(vdot, band.slow_s_per_km)
                        .and_then(|percent| render::percent(Some(percent))),
                ),
                _ => (None, None),
            };
            CurveSessionRow {
                kind: session.kind.key().to_string(),
                label: session.label.to_string(),
                from_minutes: session.minutes.map(|minutes| minutes.0),
                to_minutes: session.minutes.map(|minutes| minutes.1),
                pace: session.pace.as_ref().map(super::pace_band_row),
                from_percent_max,
                to_percent_max,
                rule: RuleRow {
                    id: session.rule.id.to_string(),
                    citation: session.rule.citation.to_string(),
                    line: session.rule.line(),
                },
                missing: if session.minutes.is_some() {
                    None
                } else {
                    super::core_line("curve.long_run.no_length")
                },
            }
        })
        .collect();

    // The field test's marker: forty-five minutes, the test's own place in
    // time.
    let field_test = vdot.map(|vdot| {
        let point = vo2::cost_curve(vdot, &[FIELD_TEST_MINUTES]);
        point.first().map(|point| CurvePointRow {
            minutes: point.minutes,
            percent_max: render::percent(Some(point.percent_max * 100.0)).unwrap_or(
                crate::dto::RenderedValue {
                    text: String::new(),
                    raw: Some(point.percent_max * 100.0),
                },
            ),
            pace: render::pace(point.pace_s_per_km),
        })
    });

    let (source, sources) = super::source_lines(&facts, &ladder);

    let answer = CurveInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window,
        source,
        sources,
        curve,
        races,
        sessions,
        field_test: field_test.flatten(),
        equivalents: super::equivalents_of(&ladder),
        unavailable: if vdot.is_some() {
            None
        } else {
            stride_core::training_paces::MissingInput::NoRaceTime.reason()
        },
    };
    ok(&answer)
}
