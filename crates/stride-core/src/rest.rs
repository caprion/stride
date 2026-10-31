//! The "Rest" line: a count and a direction.
//!
//! Contract §4: *The "Rest" line on Today is a **count + direction**, not a
//! conclusion: it is a summary of the three vitals ("2 of 3 signals below
//! your usual"), and its direction word is a pure function of that count,
//! never an editorial judgment.*
//!
//! So the line counts, and the word follows from the count. It never combines
//! the vitals into a verdict about the athlete — the three rows are always
//! shown beneath it, and the athlete reads the combination.
//!
//! The count is over the signals that are on the **worse** side of their own
//! band: "below your usual" for sleep and HRV, and — for a resting heart rate,
//! where lower is better — "above your usual". The wording names the side the
//! counted signals are literally on, so the sentence is true whichever vital
//! it counts.

use serde::{Deserialize, Serialize};

use crate::direction::{Direction, VitalDirection};
use crate::strings;

/// Which sides the counted signals sit on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalSides {
    /// All of them are below their usual.
    Below,
    /// All of them are above their usual.
    Above,
    /// Some are below and some are above.
    Both,
    /// All of them are inside their band.
    About,
}

impl SignalSides {
    /// The catalogue id of the phrase that names these sides.
    pub fn id(&self) -> &'static str {
        match self {
            SignalSides::Below => "rest.count.below",
            SignalSides::Above => "rest.count.above",
            SignalSides::Both => "rest.count.outside",
            SignalSides::About => "rest.count.about",
        }
    }
}

/// The summary of the three vitals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestSummary {
    /// How many signals could be compared at all.
    pub considered: usize,
    /// How many the sentence counts.
    pub counted: usize,
    /// Which sides the counted signals sit on.
    pub sides: SignalSides,
    /// The direction word, or `None` when nothing could be compared.
    pub direction: Option<Direction>,
}

impl RestSummary {
    /// The count: "2 of 3 signals below your usual".
    pub fn phrase(&self) -> Option<String> {
        let count = self.counted.to_string();
        let total = self.considered.to_string();
        strings::render(self.sides.id(), &[("count", &count), ("total", &total)])
    }

    /// The direction word.
    pub fn direction_text(&self) -> Option<&'static str> {
        self.direction.map(|direction| direction.as_str())
    }

    /// The whole line: the direction, then the count.
    pub fn line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(direction) = self.direction_text() {
            parts.push(direction.to_string());
        }
        if let Some(phrase) = self.phrase() {
            parts.push(phrase);
        }
        parts.join(" · ")
    }

    /// What the line says when nothing could be compared.
    pub fn status_text(&self) -> Option<String> {
        if self.direction.is_some() {
            return None;
        }
        strings::render("status.not_enough_data", &[])
    }
}

/// Summarize the three vitals into the "Rest" line.
pub fn summarize(vitals: &[VitalDirection]) -> RestSummary {
    let considered: Vec<&VitalDirection> = vitals
        .iter()
        .filter(|vital| vital.direction.is_some())
        .collect();
    let worse: Vec<&VitalDirection> = considered
        .iter()
        .copied()
        .filter(|vital| vital.direction == Some(Direction::CouldBeBetter))
        .collect();
    let better: Vec<&VitalDirection> = considered
        .iter()
        .copied()
        .filter(|vital| vital.direction == Some(Direction::BetterThanUsual))
        .collect();

    let direction = if considered.is_empty() {
        None
    } else if worse.is_empty() && better.is_empty() {
        // Every signal is inside its band.
        Some(Direction::AboutUsual)
    } else if worse.is_empty() {
        // Nothing is on the worse side.
        Some(Direction::BetterThanUsual)
    } else if better.is_empty() {
        // Nothing is on the better side.
        Some(Direction::CouldBeBetter)
    } else if worse.len() == better.len() {
        // As many on one side as on the other.
        Some(Direction::AboutUsual)
    } else if worse.len() > better.len() {
        Some(Direction::CouldBeBetter)
    } else {
        Some(Direction::BetterThanUsual)
    };

    // The counted group: the signals on the worse side if there are any, else
    // the ones on the better side, else the ones inside their band.
    let all_about = worse.is_empty() && better.is_empty();
    let counted_group: Vec<&VitalDirection> = if !worse.is_empty() {
        worse
    } else if !better.is_empty() {
        better
    } else {
        considered.clone()
    };

    let sides = sides_of(&counted_group, all_about);

    RestSummary {
        considered: considered.len(),
        counted: counted_group.len(),
        sides,
        direction,
    }
}

/// Which sides a group of signals sits on.
fn sides_of(group: &[&VitalDirection], all_about: bool) -> SignalSides {
    if all_about {
        return SignalSides::About;
    }
    let mut below = false;
    let mut above = false;
    for vital in group {
        match vital.deviation {
            Some(deviation) if deviation < 0.0 => below = true,
            Some(deviation) if deviation > 0.0 => above = true,
            _ => {}
        }
    }
    match (below, above) {
        (true, false) => SignalSides::Below,
        (false, true) => SignalSides::Above,
        (true, true) => SignalSides::Both,
        (false, false) => SignalSides::About,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VitalKind;

    fn vital(
        kind: VitalKind,
        direction: Option<Direction>,
        deviation: Option<f64>,
    ) -> VitalDirection {
        VitalDirection {
            vital: kind,
            value: Some(50.0),
            direction,
            deviation,
            band: None,
            travel: None,
        }
    }

    #[test]
    fn the_frozen_example_reads_exactly_as_the_contract_writes_it() {
        // Sleep 6 h 10 m (1 h 10 m less than usual), HRV 48 ms (14 below),
        // resting HR 63 bpm (5 above, inside the band).
        let summary = summarize(&[
            vital(
                VitalKind::Sleep,
                Some(Direction::CouldBeBetter),
                Some(-70.0 * 60.0),
            ),
            vital(VitalKind::Hrv, Some(Direction::CouldBeBetter), Some(-14.0)),
            vital(VitalKind::RestingHr, Some(Direction::AboutUsual), Some(5.0)),
        ]);
        assert_eq!(summary.considered, 3);
        assert_eq!(summary.counted, 2);
        assert_eq!(summary.sides, SignalSides::Below);
        assert_eq!(summary.direction, Some(Direction::CouldBeBetter));
        assert_eq!(
            summary.phrase(),
            Some("2 of 3 signals below your usual".to_string())
        );
        assert_eq!(
            summary.line(),
            "could be better · 2 of 3 signals below your usual"
        );
    }

    #[test]
    fn the_direction_follows_from_the_count() {
        // Two of three on the worse side.
        let summary = summarize(&[
            vital(VitalKind::Sleep, Some(Direction::CouldBeBetter), Some(-1.0)),
            vital(VitalKind::Hrv, Some(Direction::CouldBeBetter), Some(-1.0)),
            vital(VitalKind::RestingHr, Some(Direction::AboutUsual), Some(1.0)),
        ]);
        assert_eq!(summary.direction, Some(Direction::CouldBeBetter));

        // One of three on the worse side, one on the better side: a tie.
        let summary = summarize(&[
            vital(VitalKind::Sleep, Some(Direction::CouldBeBetter), Some(-1.0)),
            vital(VitalKind::Hrv, Some(Direction::BetterThanUsual), Some(1.0)),
            vital(VitalKind::RestingHr, Some(Direction::AboutUsual), Some(0.0)),
        ]);
        assert_eq!(summary.direction, Some(Direction::AboutUsual));

        // None on the worse side.
        let summary = summarize(&[
            vital(
                VitalKind::Sleep,
                Some(Direction::BetterThanUsual),
                Some(1.0),
            ),
            vital(VitalKind::Hrv, Some(Direction::AboutUsual), Some(0.0)),
            vital(VitalKind::RestingHr, Some(Direction::AboutUsual), Some(0.0)),
        ]);
        assert_eq!(summary.direction, Some(Direction::BetterThanUsual));
        assert_eq!(
            summary.phrase(),
            Some("1 of 3 signals above your usual".to_string())
        );
    }

    #[test]
    fn a_resting_heart_rate_above_its_usual_is_counted_as_above() {
        let summary = summarize(&[
            vital(VitalKind::Sleep, Some(Direction::AboutUsual), Some(0.0)),
            vital(VitalKind::Hrv, Some(Direction::AboutUsual), Some(0.0)),
            vital(
                VitalKind::RestingHr,
                Some(Direction::CouldBeBetter),
                Some(6.0),
            ),
        ]);
        assert_eq!(summary.counted, 1);
        assert_eq!(summary.sides, SignalSides::Above);
        assert_eq!(
            summary.phrase(),
            Some("1 of 3 signals above your usual".to_string())
        );
    }

    #[test]
    fn signals_on_both_sides_are_counted_as_outside() {
        let summary = summarize(&[
            vital(
                VitalKind::Sleep,
                Some(Direction::CouldBeBetter),
                Some(-60.0),
            ),
            vital(VitalKind::Hrv, Some(Direction::CouldBeBetter), Some(-10.0)),
            vital(
                VitalKind::RestingHr,
                Some(Direction::CouldBeBetter),
                Some(6.0),
            ),
        ]);
        assert_eq!(summary.sides, SignalSides::Both);
        assert_eq!(
            summary.phrase(),
            Some("3 of 3 signals outside your usual".to_string())
        );
    }

    #[test]
    fn without_a_comparison_the_line_says_so_and_invents_nothing() {
        let summary = summarize(&[
            vital(VitalKind::Sleep, None, None),
            vital(VitalKind::Hrv, None, None),
            vital(VitalKind::RestingHr, None, None),
        ]);
        assert_eq!(summary.considered, 0);
        assert_eq!(summary.direction, None);
        assert_eq!(
            summary.status_text(),
            Some("not enough data for a comparison yet".to_string())
        );
        assert_eq!(
            summary.phrase(),
            Some("0 of 0 signals about your usual".to_string())
        );
    }

    #[test]
    fn the_denominator_is_the_number_of_signals_that_could_be_compared() {
        let summary = summarize(&[
            vital(
                VitalKind::Sleep,
                Some(Direction::CouldBeBetter),
                Some(-60.0),
            ),
            vital(VitalKind::Hrv, None, None),
            vital(VitalKind::RestingHr, Some(Direction::AboutUsual), Some(1.0)),
        ]);
        assert_eq!(summary.considered, 2);
        assert_eq!(
            summary.phrase(),
            Some("1 of 2 signals below your usual".to_string())
        );
    }
}
