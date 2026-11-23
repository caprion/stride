//! The challenges: the athlete's own, measured week by week.
//!
//! A **challenge** is something the athlete set themselves and wants to
//! keep doing: a run of at least a certain distance, or a certain
//! distance across the week. There are two kinds, and both are measured
//! against a **week**:
//!
//! - **`week_distance`** — the week's own distance reached the target.
//!   The weekly goal is one of these, so the goal is already a
//!   challenge and needs nothing set up;
//! - **`run_distance`** — the week's **longest run** reached the target.
//!   "Complete a 10 km run", week by week.
//!
//! ## A week is met or it is not
//!
//! The answer is a fact: the week's distance was what it was, and the
//! longest run in it was what it was. There is no partial credit and no
//! grade — a week that reached the target was reached and a week that
//! did not was not. **A challenge that is not met is not a failure**:
//! the grid says what happened, and the reader draws the conclusion.
//!
//! ## The weeks are the calendar's
//!
//! The weeks are the same calendar weeks the rest of the product uses,
//! and the grid runs from the oldest to the newest. A week with no run
//! in it is a week the challenge was not met — and the record does not
//! cover it, so the reader cannot tell the two apart. **Absence is not
//! zero, and a gap in the record is not a rest week.** The grid draws
//! such a week differently, and says so.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::units;

/// How a challenge is measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeKind {
    /// The week's own distance reached the target.
    WeekDistance,
    /// The week's longest run reached the target.
    RunDistance,
}

impl ChallengeKind {
    /// The kind's name, in the core's own words.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WeekDistance => "week_distance",
            Self::RunDistance => "run_distance",
        }
    }

    /// The kind a name stands for, when it stands for one.
    pub fn of_str(value: &str) -> Option<Self> {
        match value {
            "week_distance" => Some(Self::WeekDistance),
            "run_distance" => Some(Self::RunDistance),
            _ => None,
        }
    }
}

/// One challenge of the athlete's own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Challenge {
    /// How it is measured.
    pub kind: ChallengeKind,
    /// The target, in metres.
    pub target_m: f64,
    /// The athlete's own words for it, when they gave any.
    pub label: Option<String>,
}

impl Challenge {
    /// The challenge, with its target in metres.
    pub fn new(kind: ChallengeKind, target_m: f64) -> Self {
        Self {
            kind,
            target_m,
            label: None,
        }
    }

    /// The name the core gives it: "10 km in the week", "a run of 10 km".
    ///
    /// It is a description of the measure, not a grade and not a
    /// suggestion.
    pub fn name(&self) -> String {
        let distance = units::render_distance(Some(self.target_m)).unwrap_or_default();
        match self.kind {
            ChallengeKind::WeekDistance => format!("{distance} in the week"),
            ChallengeKind::RunDistance => format!("a run of {distance}"),
        }
    }
}

/// What one week measured against one challenge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChallengeWeek {
    /// Whether the week reached the target.
    pub met: bool,
    /// What the week actually measured, in metres.
    pub measured_m: f64,
}

/// One week's own measures: the distance across it and its longest run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeekMeasure {
    /// The Monday the week starts on.
    pub week_start: Date,
    /// The week's distance, in metres.
    pub distance_m: f64,
    /// The week's longest run, in metres. `None` when the week holds no
    /// run at all.
    pub longest_run_m: Option<f64>,
}

/// Whether one week reached one challenge.
///
/// The rule is the whole of it: the week's own distance for a weekly
/// challenge, and the week's longest run for a run challenge. A week
/// with no run in it reaches neither.
pub fn challenge_met(kind: ChallengeKind, target_m: f64, measure: &WeekMeasure) -> ChallengeWeek {
    let measured_m = match kind {
        ChallengeKind::WeekDistance => measure.distance_m,
        ChallengeKind::RunDistance => measure.longest_run_m.unwrap_or(0.0),
    };
    ChallengeWeek {
        met: measured_m >= target_m && target_m > 0.0,
        measured_m,
    }
}

/// The grid: every challenge against every week.
///
/// The weeks are the ones given, oldest first, and the challenges are
/// the ones given, in the order they were set. The answer is the state
/// of the world — a grid of facts — and the drawing of it is the app's.
pub fn challenge_grid(challenges: &[Challenge], weeks: &[WeekMeasure]) -> Vec<Vec<ChallengeWeek>> {
    challenges
        .iter()
        .map(|challenge| {
            weeks
                .iter()
                .map(|week| challenge_met(challenge.kind, challenge.target_m, week))
                .collect()
        })
        .collect()
}

/// How many weeks in a row a challenge was met, at the end of the grid.
///
/// It is a count and not a grade: the streak is a fact about the weeks.
pub fn streak_of(grid: &[ChallengeWeek]) -> u32 {
    let mut streak = 0;
    for week in grid.iter().rev() {
        if !week.met {
            break;
        }
        streak += 1;
    }
    streak
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn week(start: &str, distance_m: f64, longest_m: Option<f64>) -> WeekMeasure {
        WeekMeasure {
            week_start: Date::parse(start).expect("a date"),
            distance_m,
            longest_run_m: longest_m,
        }
    }

    #[test]
    fn a_weekly_challenge_is_the_weeks_own_distance() {
        let challenge = Challenge::new(ChallengeKind::WeekDistance, 40_000.0);
        let met = challenge_met(
            challenge.kind,
            challenge.target_m,
            &week("2026-05-04", 42_000.0, Some(12_000.0)),
        );

        assert!(met.met);
        assert_eq!(met.measured_m, 42_000.0);
    }

    #[test]
    fn a_run_challenge_is_the_weeks_longest_run() {
        let challenge = Challenge::new(ChallengeKind::RunDistance, 10_000.0);
        let met = challenge_met(
            challenge.kind,
            challenge.target_m,
            &week("2026-05-04", 42_000.0, Some(9_400.0)),
        );

        // Forty-two kilometres across the week, but no single run of ten.
        assert!(!met.met);
        assert_eq!(met.measured_m, 9_400.0);
    }

    #[test]
    fn a_week_with_no_run_reaches_nothing() {
        let challenge = Challenge::new(ChallengeKind::RunDistance, 10_000.0);
        let met = challenge_met(
            challenge.kind,
            challenge.target_m,
            &week("2026-05-04", 0.0, None),
        );

        assert!(!met.met);
    }

    #[test]
    fn the_streak_counts_the_weeks_at_the_end() {
        let grid = vec![
            ChallengeWeek {
                met: true,
                measured_m: 1.0,
            },
            ChallengeWeek {
                met: false,
                measured_m: 0.0,
            },
            ChallengeWeek {
                met: true,
                measured_m: 1.0,
            },
            ChallengeWeek {
                met: true,
                measured_m: 1.0,
            },
        ];

        assert_eq!(streak_of(&grid), 2);
    }
}
