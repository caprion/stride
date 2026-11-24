//! The ladder: the training paces, each with the rule it comes from.
//!
//! The ladder is the method's own chain, read from the athlete's own
//! facts: **a race the athlete typed → a fitness number → the equivalent
//! race times → the rungs**. Every rung carries the rule it comes from, the
//! citation beside it, and the reason it has no value where it has none.
//!
//! ## What it answers
//!
//! * the **rungs**, fastest first — each with its pace, its heart-rate
//!   band, its rule and its citation;
//! * the **anchors** — what each rung's numbers hang on. The goal race
//!   pace and the hour-pace band are named, and never mixed;
//! * the **source lines** — the race the paces came from and the zone
//!   model the bands were measured on: one line hoisted for the screen to
//!   name, and the rest behind a click;
//! * the **missing-value reasons** — in the core's own words.
//!
//! ## What it never does
//!
//! It never guesses a pace for a rung the method leaves open, never turns
//! a heart rate into a pace or a pace into a heart rate, and never invents
//! a race. A rung with no pace says so, and the other rungs still answer.

use serde_json::Value;
use stride_core::training_paces::{Anchor, HeartRateBand, HeartRateValue, Rung, RungKind};

use crate::dto::{HeartRateRow, PacesInsight, RuleRow, RungRow};
use crate::error::ApiResult;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The ladder, over the Progress window.
///
/// The window is named, and the ladder's own numbers do not depend on it:
/// the paces come from the athlete's own race, and a race is a fact about
/// a day rather than a period.
pub fn paces(api: &Api, request: &Request) -> ApiResult<Value> {
    let (window, _, today) = super::insight_window(api, request)?;
    let facts = crate::athlete::Facts::read(api, today)?;
    let ladder = facts.ladder();

    // The source lines: the race the paces came from, and the zone model
    // the bands were measured on. One line is hoisted; the rest are behind
    // a click.
    let (source, sources) = super::source_lines(&facts, &ladder);

    let answer = PacesInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window,
        source,
        sources,
        chain: ladder.chain_line(),
        race: facts.stored_race.as_ref().map(super::race_row),
        equivalents: super::equivalents_of(&ladder),
        rungs: ladder.rungs.iter().map(rung_row).collect(),
        unavailable: if ladder.vdot.is_some() {
            None
        } else {
            stride_core::training_paces::MissingInput::NoRaceTime.reason()
        },
    };
    ok(&answer)
}

/// One rung, as the answer carries it.
fn rung_row(rung: &Rung) -> RungRow {
    RungRow {
        kind: kind_of(rung.kind).to_string(),
        label: rung.label().to_string(),
        rule: RuleRow {
            id: rung.rule.id.to_string(),
            citation: rung.rule.citation.to_string(),
            line: rung.rule.line(),
        },
        anchor: anchor_of(rung.anchor).to_string(),
        pace: super::pace_row(&rung.pace),
        finish_pace: super::pace_row(&rung.finish_pace),
        heart_rate: heart_rate_row(&rung.heart_rate),
        heart_rate_reserve: heart_rate_row(&rung.heart_rate_reserve),
    }
}

/// The key the app reads a rung by.
fn kind_of(kind: RungKind) -> &'static str {
    match kind {
        RungKind::GeneralAerobic => "general_aerobic",
        RungKind::Recovery => "recovery",
        RungKind::LactateThreshold => "lactate_threshold",
        RungKind::LongRun => "long_run",
        RungKind::GoalRace => "goal_race",
    }
}

/// The key the app reads an anchor by.
///
/// **The two anchors are named and never mixed**: the goal race pace and
/// the hour-pace band are two different numbers, and each rung says which
/// one its numbers hang on.
fn anchor_of(anchor: Anchor) -> &'static str {
    match anchor {
        Anchor::GoalRace => "goal_race",
        Anchor::ThresholdBand => "threshold_band",
        Anchor::RacePace => "race_pace",
        Anchor::TenKPace => "ten_k_pace",
        Anchor::None => "none",
    }
}

/// A heart-rate band, or the reason there is none.
fn heart_rate_row(value: &HeartRateValue) -> HeartRateRow {
    match value {
        HeartRateValue::Known(band) => {
            let (from_bpm, to_bpm, below_bpm, max_hr, resting_hr) = match band {
                HeartRateBand::Range {
                    from_bpm,
                    to_bpm,
                    basis,
                } => (
                    Some(*from_bpm),
                    Some(*to_bpm),
                    None,
                    basis_max_hr(basis),
                    basis_resting_hr(basis),
                ),
                HeartRateBand::Below { below_bpm, basis } => (
                    None,
                    None,
                    Some(*below_bpm),
                    basis_max_hr(basis),
                    basis_resting_hr(basis),
                ),
            };
            HeartRateRow {
                value: band.render(),
                from_bpm,
                to_bpm,
                below_bpm,
                max_heart_rate: max_hr,
                resting_heart_rate: resting_hr,
                missing: None,
            }
        }
        HeartRateValue::NoneStated => HeartRateRow {
            value: None,
            from_bpm: None,
            to_bpm: None,
            below_bpm: None,
            max_heart_rate: None,
            resting_heart_rate: None,
            missing: super::core_line("threshold.none_stated"),
        },
        HeartRateValue::Missing(missing) => HeartRateRow {
            value: None,
            from_bpm: None,
            to_bpm: None,
            below_bpm: None,
            max_heart_rate: None,
            resting_heart_rate: None,
            missing: missing.reason(),
        },
    }
}

/// The maximum heart rate a band was computed on.
fn basis_max_hr(basis: &stride_core::training_paces::BandBasis) -> Option<f64> {
    match basis {
        stride_core::training_paces::BandBasis::MaxHeartRate { max_hr } => Some(*max_hr),
        stride_core::training_paces::BandBasis::Reserve { max_hr, .. } => Some(*max_hr),
    }
}

/// The resting heart rate a band was computed on.
fn basis_resting_hr(basis: &stride_core::training_paces::BandBasis) -> Option<f64> {
    match basis {
        stride_core::training_paces::BandBasis::MaxHeartRate { .. } => None,
        stride_core::training_paces::BandBasis::Reserve { resting_hr, .. } => Some(*resting_hr),
    }
}
