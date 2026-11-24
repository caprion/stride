//! The one movement rule: how the athlete was moving.
//!
//! A record stream and a lap's own means answer the same question — **was
//! the athlete running, walking or standing?** — and the answer used to
//! live in two places with two different sets of thresholds. The ingest
//! decided it for every sample of a run; the fingerprint decided it again
//! for every lap, and the two disagreed: a lap at 100 steps a minute and
//! 2.0 metres a second was **walked** at the lap level and **run** at the
//! sample level.
//!
//! So the rule lives **here**, in the core, and both read it:
//!
//! * the **ingest classifies** — it calls [`movement_of`] for every sample
//!   of a record stream and keeps only the stream logic: which sample owns
//!   which span;
//! * the **core reads** — the fingerprint calls the same function with a
//!   lap's own means and keeps the lap logic: which laps are long enough
//!   to measure.
//!
//! ## Two views of one rule
//!
//! The rule is one rule; the **grain** is what differs. The sample view
//! asks whether the athlete was moving in this instant; the lap view asks
//! whether this lap was run or walked, computed from the lap's own means.
//! One threshold set, two views of it.
//!
//! ## The thresholds
//!
//! They are the **ingest's own**, kept: the reference implementation's
//! rules, in one place. The speed below which a stretch is standing still
//! is 0.30 m/s; the cadence below which a stretch is a walk is 60 steps a
//! minute, and it only counts when the stretch is slow as well (below
//! 2.2 m/s); and any stretch slower than 1.55 m/s is a walk whatever its
//! cadence says.

use serde::{Deserialize, Serialize};

/// The speed below which a stretch is **standing still**, in metres per
/// second.
pub const STAND_SPEED_MPS: f64 = 0.30;

/// The cadence below which a stretch is **walking**, in steps per minute.
///
/// The stored cadence is deliberately not corrected before the comparison:
/// a watch that stores running cadence halved writes about `80`, and that
/// value is what distinguishes a run from a walk.
pub const WALKING_CADENCE_SPM: f64 = 60.0;

/// The speed below which a walk is a walk, in metres per second.
///
/// The cadence rule counts only when the stretch is slow as well: a slow
/// runner with short strides is still running.
pub const WALKING_SPEED_MPS: f64 = 2.2;

/// The speed below which a stretch is a **walk** whatever its cadence
/// says, in metres per second.
pub const WALK_SPEED_MPS: f64 = 1.55;

/// How the athlete was moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Movement {
    /// The athlete was running.
    Run,
    /// The athlete was walking.
    Walk,
    /// The athlete was standing still.
    Stand,
}

impl Movement {
    /// The kind's name, as the contract spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Walk => "walk",
            Self::Stand => "stand",
        }
    }
}

/// How the athlete was moving, from a speed and a cadence.
///
/// The rules, in order:
///
/// * a stretch slower than [`STAND_SPEED_MPS`] is **standing still**;
/// * a stretch whose stored cadence is zero is **standing still** — a
///   watch that records no steps is not recording a walk;
/// * a stretch with a cadence below [`WALKING_CADENCE_SPM`] **and** a
///   speed below [`WALKING_SPEED_MPS`] is **walking**;
/// * a stretch slower than [`WALK_SPEED_MPS`] is **walking**, whatever its
///   cadence says;
/// * everything else is **running**.
///
/// A missing reading is not a reading: a stretch with no cadence is judged
/// on its speed alone, and a stretch with no speed is judged on its
/// cadence alone.
pub fn movement_of(speed: Option<f64>, cadence: Option<f64>) -> Movement {
    if speed.is_some_and(|speed| speed < STAND_SPEED_MPS) {
        return Movement::Stand;
    }
    if cadence.is_some_and(|cadence| cadence == 0.0) {
        return Movement::Stand;
    }
    if cadence.is_some_and(|cadence| cadence < WALKING_CADENCE_SPM)
        && speed.map_or(true, |speed| speed < WALKING_SPEED_MPS)
    {
        return Movement::Walk;
    }
    if speed.is_some_and(|speed| speed < WALK_SPEED_MPS) {
        return Movement::Walk;
    }
    Movement::Run
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stretch_that_is_too_slow_to_move_is_standing_still() {
        assert_eq!(movement_of(Some(0.0), Some(80.0)), Movement::Stand);
        assert_eq!(movement_of(Some(0.1), Some(0.0)), Movement::Stand);
    }

    #[test]
    fn a_stretch_with_no_cadence_and_no_speed_is_running() {
        // Nothing is known, and nothing is invented: it is not a walk.
        assert_eq!(movement_of(None, None), Movement::Run);
    }

    #[test]
    fn a_slow_stretch_is_walking_whatever_its_cadence_says() {
        assert_eq!(movement_of(Some(1.0), None), Movement::Walk);
        assert_eq!(movement_of(Some(1.3), Some(30.0)), Movement::Walk);
        assert_eq!(movement_of(Some(1.3), Some(170.0)), Movement::Walk);
    }

    #[test]
    fn a_short_strided_stretch_is_walking_only_when_it_is_slow() {
        assert_eq!(movement_of(Some(2.0), Some(30.0)), Movement::Walk);
        assert_eq!(movement_of(Some(2.5), Some(30.0)), Movement::Run);
        assert_eq!(movement_of(Some(3.0), Some(78.0)), Movement::Run);
    }
}
