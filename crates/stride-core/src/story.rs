//! The story engine: the one sentence every run carries.
//!
//! The plan is explicit: *"Never hide the story line — a row without a
//! sentence is a spreadsheet"*, and *"Every visible row has a non-empty story
//! sentence."* The sentence is the core's, like every other word the app says.
//!
//! What a story is:
//!
//! * **descriptive** — it says what the run *was*, not what it should have
//!   been. No verdict, no grade, no "good", no advice, and no comparison
//!   with a plan that is not there;
//! * **built from the same primitives the charts read** — the distance, the
//!   times, the heart rate, the splits — so a sentence can never contradict a
//!   chart;
//! * **one sentence** — never a paragraph, never two clauses fighting each
//!   other;
//! * **never invented** — a number that is missing is left out, and the
//!   sentence is still a sentence.
//!
//! The shapes, in the order the engine prefers them:
//!
//! 1. **against the athlete's own usual** — "10.2 km, 2.4 km more than your
//!    usual Wednesday";
//! 2. **the numbers** — "6.4 km in 41 m, at an average of 162 bpm";
//! 3. **nothing to say** — "nothing unusual about this one".
//!
//! And beside the sentence, the **why**: the one extra fact the detail knows
//! and the feed does not — the run's own shape, "the last 3 km were the
//! slowest of the run". The feed has no splits, so the why is absent there
//! and the detail says it.
//!
//! Every sentence comes out of the catalogue, so the lint holds it to the
//! voice: no imperative, no verdict, no score.

use serde::{Deserialize, Serialize};

use crate::band;
use crate::date::Date;
use crate::strings;
use crate::units;

/// The smallest difference, as a share of the usual, that is worth a sentence.
///
/// The same ten per cent the contract uses for the distance comparison: a run
/// within a tenth of the usual *is* usual, and saying so adds nothing.
pub const STORY_USUAL_THRESHOLD: f64 = 0.10;

/// The smallest difference, as a share of the run's average pace, that makes a
/// split worth mentioning.
///
/// Every run has a slowest split; saying which one it was is only worth a
/// sentence when it stands out from the rest of the run.
pub const STORY_SPLIT_THRESHOLD: f64 = 0.10;

/// The fewest prior runs on the same weekday that make a "usual" worth
/// saying.
///
/// The same floor the pattern engine uses: two runs are "twice so far", not a
/// usual.
pub const STORY_USUAL_SAMPLES: usize = 3;

/// One split of a run: how far, and how long it took.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Split {
    /// The split's distance, in metres.
    pub distance_m: f64,
    /// The split's moving time, in seconds.
    pub moving_time_s: f64,
}

/// What one run was, from the athlete's own numbers.
///
/// The same primitives the charts read, and nothing else: no verdict, no
/// score, no comparison with anything that is not there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunFacts {
    /// The day the run was on.
    pub date: Date,
    /// The distance, in metres.
    pub distance_m: Option<f64>,
    /// The moving time, in seconds.
    pub moving_time_s: Option<f64>,
    /// The average heart rate, in beats per minute.
    pub avg_hr: Option<f64>,
    /// The splits, in the order they were run.
    pub splits: Vec<Split>,
}

impl RunFacts {
    /// A run with nothing but a date.
    pub fn new(date: Date) -> Self {
        Self {
            date,
            distance_m: None,
            moving_time_s: None,
            avg_hr: None,
            splits: Vec::new(),
        }
    }

    /// The distance, in metres, when there is one worth saying.
    fn distance(&self) -> Option<f64> {
        self.distance_m
            .filter(|metres| metres.is_finite() && *metres > 0.0)
    }

    /// The moving time, in seconds, when there is one worth saying.
    fn duration(&self) -> Option<f64> {
        self.moving_time_s
            .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
    }

    /// The average heart rate, when there is one worth saying.
    fn heart_rate(&self) -> Option<f64> {
        self.avg_hr.filter(|bpm| bpm.is_finite() && *bpm > 0.0)
    }
}

/// The story of one run: one sentence, and never an empty one.
///
/// `history` is the athlete's own earlier runs — the story compares a run with
/// the athlete's history and with nothing else. Runs on the same day as the
/// run itself are ignored, so a run can never be compared with itself.
///
/// This is the feed's line: the sentence alone. The detail knows more — the
/// run's splits — and says the extra fact in [`story_with_why`].
pub fn story(run: &RunFacts, history: &[RunFacts]) -> String {
    sentence(run, history)
}

/// The sentence, and the one extra fact the detail knows.
///
/// The sentence is what the feed shows. The **why** is the run's own shape —
/// the slowest split, the fastest one — the fact the detail knows and the
/// feed does not, because the feed has no splits. It is `None` when there is
/// nothing extra to say.
pub fn story_with_why(run: &RunFacts, history: &[RunFacts]) -> (String, Option<String>) {
    (sentence(run, history), why(run))
}

/// The sentence: what the run was, and how it sat against the athlete's own
/// usual.
fn sentence(run: &RunFacts, history: &[RunFacts]) -> String {
    if let Some(sentence) = usual_sentence(run, history) {
        return sentence;
    }
    plain_sentence(run)
}

/// The why: the run's own shape.
fn why(run: &RunFacts) -> Option<String> {
    split_sentence(run)
}

/// The story against the athlete's own usual on that weekday.
///
/// "10.2 km, 2.4 km more than your usual Wednesday".
fn usual_sentence(run: &RunFacts, history: &[RunFacts]) -> Option<String> {
    let distance = run.distance()?;
    let usual = usual_on_weekday(run, history)?;
    let difference = (distance - usual).abs();
    if difference / usual <= STORY_USUAL_THRESHOLD {
        return None;
    }
    let distance_text = units::render_distance(Some(distance))?;
    let gap = units::render_distance(Some(difference))?;
    let weekday = run.date.weekday().name();
    let id = if distance >= usual {
        "story.usual.more"
    } else {
        "story.usual.less"
    };
    strings::render(
        id,
        &[
            ("distance", &distance_text),
            ("gap", &gap),
            ("weekday", weekday),
        ],
    )
}

/// The usual distance on the run's weekday: the median of the athlete's
/// earlier runs on that weekday.
///
/// The run's own day is excluded, and so are days after it: a usual is
/// something the athlete has already done.
fn usual_on_weekday(run: &RunFacts, history: &[RunFacts]) -> Option<f64> {
    let weekday = run.date.weekday();
    let distances: Vec<f64> = history
        .iter()
        .filter(|other| other.date < run.date && other.date.weekday() == weekday)
        .filter_map(|other| other.distance())
        .collect();
    if distances.len() < STORY_USUAL_SAMPLES {
        return None;
    }
    band::median(&distances)
}

/// The story from the run's own shape: the slowest or the fastest split.
///
/// "the last 3 km were the slowest of the run". This is the **why**: the fact
/// the detail knows and the feed does not, because the feed has no splits.
fn split_sentence(run: &RunFacts) -> Option<String> {
    let paces: Vec<f64> = run
        .splits
        .iter()
        .filter_map(|split| split_pace(*split))
        .collect();
    if paces.len() < 2 {
        return None;
    }
    let average = run_pace(run)?;
    let mut slowest = 0usize;
    let mut fastest = 0usize;
    for index in 1..paces.len() {
        if paces[index] > paces[slowest] {
            slowest = index;
        }
        if paces[index] < paces[fastest] {
            fastest = index;
        }
    }
    let last = paces.len() - 1;
    if paces[slowest] >= average * (1.0 + STORY_SPLIT_THRESHOLD)
        && (slowest == 0 || slowest == last)
    {
        return split_phrase(run.splits[slowest], Position::Last, Speed::Slowest);
    }
    if paces[fastest] <= average * (1.0 - STORY_SPLIT_THRESHOLD)
        && (fastest == 0 || fastest == last)
    {
        return split_phrase(run.splits[fastest], Position::First, Speed::Fastest);
    }
    None
}

/// Where a split sits in the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Position {
    /// The opening split.
    First,
    /// The closing split.
    Last,
}

/// Whether a split was the fastest or the slowest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Speed {
    /// The quickest split.
    Fastest,
    /// The slowest split.
    Slowest,
}

/// The sentence about one split.
fn split_phrase(split: Split, position: Position, speed: Speed) -> Option<String> {
    let distance = units::render_distance(Some(split.distance_m))?;
    // "3 km were the slowest" and "1 km was the slowest": a distance of one
    // kilometre or less is one distance, not several.
    let one = split.distance_m <= 1000.0;
    let id = match (position, speed, one) {
        (Position::Last, Speed::Slowest, false) => "why.slowest_split",
        (Position::Last, Speed::Slowest, true) => "why.slowest_split_one",
        (Position::First, Speed::Fastest, false) => "why.fastest_split",
        (Position::First, Speed::Fastest, true) => "why.fastest_split_one",
        _ => return None,
    };
    strings::render(id, &[("distance", &distance)])
}

/// The story from the run's numbers: "6.4 km in 41 m, at an average of 162
/// bpm".
///
/// A number that is missing is left out; what is left is still a sentence.
fn plain_sentence(run: &RunFacts) -> String {
    let distance = run
        .distance()
        .and_then(|metres| units::render_distance(Some(metres)));
    let duration = run
        .duration()
        .and_then(|seconds| units::render_duration(Some(seconds)));
    let heart_rate = run
        .heart_rate()
        .and_then(|bpm| units::render_heart_rate(Some(bpm)));

    let rendered = match (&distance, &duration, &heart_rate) {
        (Some(distance), Some(duration), Some(heart_rate)) => strings::render(
            "story.plain",
            &[
                ("distance", distance),
                ("duration", duration),
                ("heart_rate", heart_rate),
            ],
        ),
        (Some(distance), Some(duration), None) => strings::render(
            "story.plain_no_heart_rate",
            &[("distance", distance), ("duration", duration)],
        ),
        (Some(distance), None, Some(heart_rate)) => strings::render(
            "story.plain_heart_rate_only",
            &[("distance", distance), ("heart_rate", heart_rate)],
        ),
        (Some(distance), None, None) => {
            strings::render("story.plain_distance_only", &[("distance", distance)])
        }
        (None, Some(duration), Some(heart_rate)) => strings::render(
            "story.plain_no_distance",
            &[("duration", duration), ("heart_rate", heart_rate)],
        ),
        (None, Some(duration), None) => {
            strings::render("story.plain_duration_only", &[("duration", duration)])
        }
        (None, None, Some(heart_rate)) => {
            strings::render("story.plain_heart_rate_only", &[("heart_rate", heart_rate)])
        }
        (None, None, None) => None,
    };
    rendered.unwrap_or_else(|| {
        strings::text("story.nothing_unusual")
            .unwrap_or("nothing unusual about this one")
            .to_string()
    })
}

/// The run's average pace, in seconds per kilometre.
fn run_pace(run: &RunFacts) -> Option<f64> {
    let distance = run.distance()?;
    let duration = run.duration()?;
    let pace = duration / (distance / 1000.0);
    if pace.is_finite() && pace > 0.0 {
        Some(pace)
    } else {
        None
    }
}

/// One split's pace, in seconds per kilometre.
fn split_pace(split: Split) -> Option<f64> {
    if !split.distance_m.is_finite() || split.distance_m <= 0.0 {
        return None;
    }
    if !split.moving_time_s.is_finite() || split.moving_time_s <= 0.0 {
        return None;
    }
    let pace = split.moving_time_s / (split.distance_m / 1000.0);
    if pace.is_finite() && pace > 0.0 {
        Some(pace)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_with_nothing_to_say_says_that() {
        let date = match Date::parse("2026-05-12") {
            Some(date) => date,
            None => return,
        };
        let run = RunFacts::new(date);
        assert_eq!(story(&run, &[]), "nothing unusual about this one");
    }
}
