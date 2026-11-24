//! The effort model — a race time in, a fitness number and a cost curve out.
//!
//! The relations are **Daniels'**: the oxygen cost of running and the share
//! of VO₂max a pace holds, from *Running Formula*, 3rd ed. (Human Kinetics,
//! 2014), with the curves of Daniels & Gilbert, *Oxygen Power* (1979). The
//! card names them — `exploration/pace-engine.md` §2.1 — because this is the
//! **engine's own chain**: the book's own chain is its lookup tables
//! (Appendix B), and the two agree inside the book's thirty-second steps.
//!
//! ```text
//! v        = distance (m) ÷ t                 (t in minutes)
//! VO2      = −4.60 + 0.182258·v + 0.000104·v²     (ml/kg/min)
//! %max     = 0.8 + 0.1894393·e^(−0.012778·t) + 0.2989558·e^(−0.1932605·t)
//! VDOT     = VO2 ÷ %max
//! ```
//!
//! ## What this module is for
//!
//! * **the fitness number** — [`effort_from_race`] turns the athlete's own
//!   race into one number, and the surface prints it;
//! * **the equivalents** — [`equivalents`] gives the race time at another
//!   distance **twice**: once by the relations inverted
//!   ([`time_for_vdot`]) and once by **Riegel's form**
//!   ([`riegel_time`]). The two are independent models, and the gap between
//!   them is a fact the surface may print;
//! * **the cost curve** — [`cost_curve`] gives the share of VO₂max that a
//!   duration holds, and the pace that costs it, so any duration has a
//!   physiological cost and any pace has a cost.
//!
//! ## What it never does
//!
//! * It never invents a race time and never infers one from the training.
//!   A race is an observation the athlete typed (`exploration/pace-engine.md`
//!   §4).
//! * It never converts an age into a number, and it never holds a default.
//!   A missing input is a missing value, never a zero.
//! * It never draws the method's heart-rate bands on the cost curve:
//!   %maxHR and %maxVO₂ are not convertible without the athlete's lactate
//!   relation, which Stride does not have (`exploration/pace-engine.md`
//!   §8.2).

use serde::{Deserialize, Serialize};

use crate::units;

/// The intercept of the oxygen-cost relation.
pub const VO2_INTERCEPT: f64 = -4.60;
/// The linear term of the oxygen-cost relation.
pub const VO2_LINEAR: f64 = 0.182258;
/// The quadratic term of the oxygen-cost relation.
pub const VO2_QUADRATIC: f64 = 0.000104;

/// The share of VO₂max every effort costs, before the two decay terms.
pub const PCT_MAX_BASE: f64 = 0.8;
/// The first decay term of the percent-of-VO₂max relation.
pub const PCT_MAX_TERM: f64 = 0.1894393;
/// The decay rate of the first term, per minute.
pub const PCT_MAX_RATE: f64 = 0.012778;
/// The second decay term of the percent-of-VO₂max relation.
pub const PCT_MAX_TERM_FAST: f64 = 0.2989558;
/// The decay rate of the second term, per minute.
pub const PCT_MAX_RATE_FAST: f64 = 0.1932605;

/// Riegel's exponent: `T₂ = T₁·(D₂ ÷ D₁)^1.06`.
pub const RIEGEL_EXPONENT: f64 = 1.06;

/// The distances the equivalents are computed at, in metres.
pub const FIVE_K: f64 = 5_000.0;
/// 10 kilometres.
pub const TEN_K: f64 = 10_000.0;
/// 15 kilometres.
pub const FIFTEEN_K: f64 = 15_000.0;
/// 10 miles, in metres.
pub const TEN_MILE: f64 = 16_093.44;
/// A half marathon, in metres.
pub const HALF_MARATHON: f64 = 21_097.5;
/// A marathon, in metres.
pub const MARATHON: f64 = 42_195.0;

/// The shortest race the inversion will look at: one minute.
const MIN_RACE_S: f64 = 60.0;
/// The longest race the inversion will look at: a day.
const MAX_RACE_S: f64 = 24.0 * 3_600.0;

/// One race's effort: the oxygen cost it ran at, the share of VO₂max that
/// share was, and the fitness number they make together.
///
/// The three numbers are one fact in three forms: `vdot` is
/// `vo2_ml_kg_min ÷ percent_max`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Effort {
    /// The fitness number — VO₂max, in millilitres per kilogram per minute.
    pub vdot: f64,
    /// The oxygen cost of the race's own pace, in ml/kg/min.
    pub vo2_ml_kg_min: f64,
    /// The share of VO₂max the race held, as a fraction.
    pub percent_max: f64,
}

impl Effort {
    /// The fitness number, as the surface prints it: "49.6".
    pub fn vdot_text(&self) -> String {
        units::format_decimal(self.vdot, 1)
    }

    /// The share of VO₂max the race held, as the surface prints it: "91 %".
    pub fn percent_max_text(&self) -> String {
        units::format_decimal(self.percent_max * 100.0, 0)
    }
}

/// The oxygen cost of running at a speed, in millilitres per kilogram per
/// minute.
///
/// `speed_m_per_min` is the speed in metres per minute — the unit the
/// relation is written in.
pub fn oxygen_cost(speed_m_per_min: f64) -> f64 {
    VO2_INTERCEPT + VO2_LINEAR * speed_m_per_min + VO2_QUADRATIC * speed_m_per_min * speed_m_per_min
}

/// The share of VO₂max that a duration holds, as a fraction.
///
/// `minutes` is the duration in minutes. The relation decays towards the
/// base 0.8: a short effort costs nearly everything the athlete has, a long
/// one costs less. At ten minutes the relation reads above 1 — a ten-minute
/// effort is at or past the athlete's maximum — and that is the relation's
/// own answer, not a defect.
pub fn percent_max_at(minutes: f64) -> f64 {
    PCT_MAX_BASE
        + PCT_MAX_TERM * (-PCT_MAX_RATE * minutes).exp()
        + PCT_MAX_TERM_FAST * (-PCT_MAX_RATE_FAST * minutes).exp()
}

/// The effort of one race: the athlete's own race, and the fitness number it
/// makes.
///
/// `None` when the race cannot be measured — a distance or a time that is not
/// a positive number. Never a default fitness number.
pub fn effort_from_race(distance_m: f64, time_s: f64) -> Option<Effort> {
    if !distance_m.is_finite() || !time_s.is_finite() || distance_m <= 0.0 || time_s <= 0.0 {
        return None;
    }
    let minutes = time_s / 60.0;
    let speed = distance_m / minutes;
    let vo2 = oxygen_cost(speed);
    let percent_max = percent_max_at(minutes);
    if !vo2.is_finite() || !percent_max.is_finite() || percent_max <= 0.0 {
        return None;
    }
    Some(Effort {
        vdot: vo2 / percent_max,
        vo2_ml_kg_min: vo2,
        percent_max,
    })
}

/// The share of the athlete's own maximum that a pace costs, as a
/// percentage.
///
/// This is the other half of the cost curve: [`percent_max_at`] asks what a
/// **duration** costs, and this asks what a **pace** costs. The athlete's
/// own races sit on the curve at this number.
pub fn cost_of_pace(vdot: f64, pace_s_per_km: f64) -> Option<f64> {
    if !vdot.is_finite() || vdot <= 0.0 || !pace_s_per_km.is_finite() || pace_s_per_km <= 0.0 {
        return None;
    }
    let speed = 60_000.0 / pace_s_per_km;
    let vo2 = oxygen_cost(speed);
    if !vo2.is_finite() {
        return None;
    }
    Some(vo2 / vdot * 100.0)
}

/// The pace that costs a given share of the athlete's maximum, in seconds per
/// kilometre.
///
/// The inverse of [`cost_of_pace`]: the pace at which the athlete's own
/// maximum oxygen uptake is spent at `percent_max`.
pub fn pace_for_percent_max(vdot: f64, percent_max: f64) -> Option<f64> {
    if !vdot.is_finite() || vdot <= 0.0 || !percent_max.is_finite() {
        return None;
    }
    let target = vdot * percent_max;
    // oxygen_cost(v) = target  ⟹  a·v² + b·v + c = 0
    let c = VO2_INTERCEPT - target;
    let discriminant = VO2_LINEAR * VO2_LINEAR - 4.0 * VO2_QUADRATIC * c;
    if !discriminant.is_finite() || discriminant < 0.0 {
        return None;
    }
    let speed = (-VO2_LINEAR + discriminant.sqrt()) / (2.0 * VO2_QUADRATIC);
    if !speed.is_finite() || speed <= 0.0 {
        return None;
    }
    Some(60_000.0 / speed)
}

/// The time a race of that distance would take at this fitness number, in
/// seconds.
///
/// The relations **inverted**: the answer is the `t` at which the pace's
/// oxygen cost equals `vdot × %max(t)`. The equation is monotone in `t` — a
/// longer race is slower and holds a smaller share — so the answer is found
/// by bisection.
///
/// The inversion **reproduces the entered race exactly**: the fitness number
/// came from that race, and the same relations read it back. A round trip
/// that does not return the entered time is a bug, not a rounding artefact.
pub fn time_for_vdot(vdot: f64, distance_m: f64) -> Option<f64> {
    if !vdot.is_finite() || vdot <= 0.0 || !distance_m.is_finite() || distance_m <= 0.0 {
        return None;
    }
    let residual = |time_s: f64| {
        let minutes = time_s / 60.0;
        oxygen_cost(distance_m / minutes) - vdot * percent_max_at(minutes)
    };
    let mut low = MIN_RACE_S;
    let mut high = MAX_RACE_S;
    if residual(low) <= 0.0 || residual(high) >= 0.0 {
        // The answer is outside the range the relation is read over.
        return None;
    }
    for _ in 0..200 {
        let middle = (low + high) / 2.0;
        if residual(middle) > 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some((low + high) / 2.0)
}

/// The pace a race of that distance would be run at this fitness number, in
/// seconds per kilometre.
pub fn pace_for_vdot(vdot: f64, distance_m: f64) -> Option<f64> {
    let time_s = time_for_vdot(vdot, distance_m)?;
    if !time_s.is_finite() || time_s <= 0.0 {
        return None;
    }
    Some(time_s / (distance_m / 1_000.0))
}

/// The time a race of that distance would take, by **Riegel's form**:
/// `T₂ = T₁·(D₂ ÷ D₁)^1.06`.
///
/// This is the second model, and it is independent of Daniels' relations: it
/// is a power law over the athlete's own race, with no physiology in it. The
/// two are compared, never added.
pub fn riegel_time(time_s: f64, from_m: f64, to_m: f64) -> Option<f64> {
    if !time_s.is_finite() || !from_m.is_finite() || !to_m.is_finite() {
        return None;
    }
    if time_s <= 0.0 || from_m <= 0.0 || to_m <= 0.0 {
        return None;
    }
    let time = time_s * (to_m / from_m).powf(RIEGEL_EXPONENT);
    if time.is_finite() && time > 0.0 {
        Some(time)
    } else {
        None
    }
}

/// One equivalent race time, and the two ways it was reached.
///
/// The two models are kept side by side and never merged: the gap between
/// them is a fact about two models, and the surface may print it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equivalent {
    /// The distance the equivalent is for, in metres.
    pub distance_m: f64,
    /// The time by Daniels' relations inverted at the athlete's fitness
    /// number.
    pub vdot_time_s: Option<f64>,
    /// The time by Riegel's form from the athlete's own race.
    pub riegel_time_s: Option<f64>,
    /// The gap between the two, in seconds: the inversion minus Riegel.
    pub difference_s: Option<f64>,
}

impl Equivalent {
    /// The pace of the Riegel form, in seconds per kilometre.
    pub fn riegel_pace_s_per_km(&self) -> Option<f64> {
        let time = self.riegel_time_s?;
        Some(time / (self.distance_m / 1_000.0))
    }

    /// The pace of the inversion, in seconds per kilometre.
    pub fn vdot_pace_s_per_km(&self) -> Option<f64> {
        let time = self.vdot_time_s?;
        Some(time / (self.distance_m / 1_000.0))
    }
}

/// The equivalent race times at a set of distances, by both models.
///
/// `anchor_time_s` and `anchor_distance_m` are the athlete's own race — the
/// one Riegel's form is applied to, and the one the fitness number came
/// from.
pub fn equivalents(
    vdot: f64,
    anchor_time_s: f64,
    anchor_distance_m: f64,
    distances_m: &[f64],
) -> Vec<Equivalent> {
    distances_m
        .iter()
        .map(|distance_m| {
            let vdot_time_s = time_for_vdot(vdot, *distance_m);
            let riegel_time_s = riegel_time(anchor_time_s, anchor_distance_m, *distance_m);
            let difference_s = match (vdot_time_s, riegel_time_s) {
                (Some(a), Some(b)) => Some(a - b),
                _ => None,
            };
            Equivalent {
                distance_m: *distance_m,
                vdot_time_s,
                riegel_time_s,
                difference_s,
            }
        })
        .collect()
}

/// One point of the physiological-cost curve.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostPoint {
    /// The duration, in minutes.
    pub minutes: f64,
    /// The share of VO₂max the duration holds, as a fraction.
    pub percent_max: f64,
    /// The pace that costs it, in seconds per kilometre — the athlete's own,
    /// at their own fitness number.
    pub pace_s_per_km: Option<f64>,
}

impl CostPoint {
    /// The share of VO₂max, as the surface prints it: "91 %".
    pub fn percent_max_text(&self) -> String {
        units::format_decimal(self.percent_max * 100.0, 0)
    }
}

/// The cost curve: the share of VO₂max against time, with the pace each
/// duration costs.
///
/// The curve is the engine's own relation — the same one the fitness number
/// came from — and it is a **second reading** of the athlete, not a second
/// ladder: no rungs, no pace bands (`exploration/pace-engine.md` §8).
///
/// The minutes are the caller's: the curve is read over the durations the
/// surface draws.
pub fn cost_curve(vdot: f64, minutes: &[f64]) -> Vec<CostPoint> {
    minutes
        .iter()
        .map(|minutes| {
            let percent_max = percent_max_at(*minutes);
            CostPoint {
                minutes: *minutes,
                percent_max,
                pace_s_per_km: pace_for_percent_max(vdot, percent_max),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical athlete's race: a 10K in 41:36 on 12 October 2026
    /// (`exploration/athlete.js`).
    const TEN_K_TIME_S: f64 = 2_496.0;

    fn canonical_effort() -> Effort {
        effort_from_race(TEN_K, TEN_K_TIME_S).unwrap_or_default()
    }

    #[test]
    fn the_oxygen_cost_follows_daniels_relation() {
        // v = 240.3846 m/min (10 000 m in 41:36) → 45.2216 ml/kg/min.
        let speed = TEN_K / (TEN_K_TIME_S / 60.0);
        let vo2 = oxygen_cost(speed);
        assert!((vo2 - 45.2216).abs() < 0.001, "VO2 was {vo2}");
    }

    #[test]
    fn the_share_of_max_at_the_canonical_race_is_the_relations_own() {
        let effort = canonical_effort();
        assert!((effort.percent_max - 0.91143).abs() < 0.0001);
        assert!(
            (effort.vdot - 49.6163).abs() < 0.001,
            "VDOT was {}",
            effort.vdot
        );
        // And the two are one fact: VDOT = VO2 ÷ %max.
        assert!((effort.vo2_ml_kg_min / effort.percent_max - effort.vdot).abs() < 1e-9);
    }

    #[test]
    fn a_race_that_cannot_be_measured_is_no_value() {
        assert!(effort_from_race(0.0, TEN_K_TIME_S).is_none());
        assert!(effort_from_race(TEN_K, 0.0).is_none());
        assert!(effort_from_race(f64::NAN, TEN_K_TIME_S).is_none());
        assert!(effort_from_race(TEN_K, -1.0).is_none());
    }

    #[test]
    fn the_inversion_reproduces_the_entered_race_exactly() {
        let effort = canonical_effort();
        let time = time_for_vdot(effort.vdot, TEN_K);
        assert!((time.unwrap_or_default() - TEN_K_TIME_S).abs() < 0.01);
    }

    #[test]
    fn the_two_models_agree_within_the_book_own_table_step() {
        // The canonical athlete: Riegel gives 15K 1:03:56, half 1:31:47,
        // marathon 3:11:22; the inversion, at the fitness number the file
        // carries, gives 1:03:53, 1:32:00 and 3:11:38 — three, twelve and
        // fifteen seconds apart (`exploration/pace-engine.md` §2.2).
        let vdot = 49.7;
        let rows = equivalents(
            vdot,
            TEN_K_TIME_S,
            TEN_K,
            &[FIFTEEN_K, HALF_MARATHON, MARATHON],
        );

        let fifteen = rows[0];
        assert!((fifteen.riegel_time_s.unwrap_or_default() - 3_836.2).abs() < 1.0);
        assert!((fifteen.vdot_time_s.unwrap_or_default() - 3_833.5).abs() < 1.0);
        assert!((fifteen.difference_s.unwrap_or_default().abs() - 2.7).abs() < 1.0);

        let half = rows[1];
        assert!((half.riegel_time_s.unwrap_or_default() - 5_507.2).abs() < 1.0);
        assert!((half.vdot_time_s.unwrap_or_default() - 5_519.9).abs() < 1.0);
        assert!((half.difference_s.unwrap_or_default().abs() - 12.7).abs() < 1.0);

        let marathon = rows[2];
        assert!((marathon.riegel_time_s.unwrap_or_default() - 11_482.1).abs() < 1.0);
        assert!((marathon.vdot_time_s.unwrap_or_default() - 11_497.8).abs() < 1.0);
        assert!((marathon.difference_s.unwrap_or_default().abs() - 15.7).abs() < 1.0);
    }

    #[test]
    fn riegel_is_the_power_law_it_says_it_is() {
        // T₂ = T₁·(D₂ ÷ D₁)^1.06 — from the canonical 10K in 41:36.
        let half = riegel_time(TEN_K_TIME_S, TEN_K, HALF_MARATHON);
        assert!((half.unwrap_or_default() - 5_507.2).abs() < 1.0);
        // And the same distance returns the same time.
        assert_eq!(riegel_time(TEN_K_TIME_S, TEN_K, TEN_K), Some(TEN_K_TIME_S));
    }

    #[test]
    fn riegel_of_nothing_is_no_value() {
        assert!(riegel_time(0.0, TEN_K, HALF_MARATHON).is_none());
        assert!(riegel_time(TEN_K_TIME_S, 0.0, HALF_MARATHON).is_none());
        assert!(riegel_time(TEN_K_TIME_S, TEN_K, 0.0).is_none());
    }

    #[test]
    fn the_cost_of_a_pace_is_the_share_of_the_athletes_own_maximum() {
        let effort = canonical_effort();
        let pace = TEN_K_TIME_S / (TEN_K / 1_000.0);
        let cost = cost_of_pace(effort.vdot, pace);
        // The race's own pace cost the share the relation gives for its own
        // duration: the same number, read the other way round.
        assert!((cost.unwrap_or_default() - 91.1427).abs() < 0.01);
    }

    #[test]
    fn the_cost_curve_falls_as_the_duration_grows() {
        let effort = canonical_effort();
        let points = cost_curve(effort.vdot, &[10.0, 20.0, 45.0, 60.0, 120.0]);
        assert_eq!(points.len(), 5);
        for pair in points.windows(2) {
            assert!(pair[1].percent_max < pair[0].percent_max);
            assert!(
                pair[1].pace_s_per_km.unwrap_or_default()
                    > pair[0].pace_s_per_km.unwrap_or_default()
            );
        }
        // The 45-minute mark is the book's own time trial, and the curve
        // reads 90.7 % of the maximum there.
        let forty_five = points[2];
        assert!((forty_five.percent_max - 0.9072).abs() < 0.001);
    }

    #[test]
    fn a_cost_point_with_nothing_to_solve_has_no_pace() {
        // No fitness number to spend, and no pace to speak of.
        assert!(pace_for_percent_max(0.0, 0.9).is_none());
        assert!(pace_for_percent_max(f64::NAN, 0.9).is_none());
        assert!(cost_of_pace(49.6, 0.0).is_none());
        assert!(cost_of_pace(49.6, f64::NAN).is_none());
    }
}
