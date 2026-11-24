//! The threshold — four candidates, one choice, a dated history.
//!
//! The app **computes candidates**; the athlete **chooses one**. The choice
//! is stored as a dated fact with its source, and the app never says which
//! candidate is right (`exploration/threshold-setup.md` §0).
//!
//! | candidate | what it is | what it yields | the rule |
//! |---|---|---|---|
//! | the watch's own zone model | the floor of the fifth zone | a heart rate | the product's own convention |
//! | from a race | the hour-pace band | a pace band | the book (ch. 1, ch. 7) |
//! | from a field test | the estimate from a run | a heart rate and a pace | the book's 45-minute time trial (ch. 1, ch. 7) |
//! | typed | the athlete's own number | a heart rate, a pace band, or both | none |
//!
//! ## What each candidate cannot know
//!
//! That line **lives in the notes**, not in the number: a candidate carries
//! its value, its date and its rule, and the limit on it is an annotation
//! the surface may show under the card (`exploration/threshold-setup.md`
//! §15). The core holds the note's wording so that the app never writes one
//! of its own.
//!
//! ## What this module never does
//!
//! * It never computes a threshold or a maximum heart rate from an age.
//! * It never converts a pace into a heart rate, or a heart rate into a
//!   pace: the two kinds sit in two lanes.
//! * It never collapses a pace band to a midpoint — the method's anchor is a
//!   band, and a midpoint is a number the method does not state.
//! * It never picks: the candidates are four **sources**, not four rules
//!   held against each other.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::strings;
use crate::training_paces::{PaceBand, Race, Rule};
use crate::units;
use crate::vo2;

/// The book's 45-minute time trial: the window it reads is the **last 30
/// minutes** of the trial.
pub const FIELD_TEST_MINUTES: f64 = 45.0;
/// The window the book's time trial reads: the last 30 minutes.
pub const FIELD_TEST_WINDOW_S: f64 = 1_800.0;

/// Where a threshold came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdSource {
    /// The floor of the fifth zone of the athlete's own watch configuration.
    WatchZoneModel,
    /// The hour-pace band, computed from a race the athlete typed.
    Race,
    /// An estimate from a run the athlete pointed at.
    FieldTest,
    /// A number the athlete brought.
    Typed,
}

impl ThresholdSource {
    /// The name of the source, as the card prints it.
    pub fn key(&self) -> &'static str {
        match self {
            ThresholdSource::WatchZoneModel => "watch_zone_model",
            ThresholdSource::Race => "race",
            ThresholdSource::FieldTest => "field_test",
            ThresholdSource::Typed => "typed",
        }
    }

    /// Where the threshold came from, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            ThresholdSource::WatchZoneModel => "your watch's zones",
            ThresholdSource::Race => "from a race",
            ThresholdSource::FieldTest => "from a field test",
            ThresholdSource::Typed => "entered by you",
        }
    }
}

/// The base a choice was made for.
///
/// The two bases read the threshold differently — the Pfitzinger base reads
/// it as a pace, the 80/20 base as a heart rate — so a choice names the base
/// it was made for, and the app holds one at a time (D14).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdBase {
    /// The Pfitzinger plan (the first drop).
    Pfitzinger,
    /// The polarised 80/20 plan.
    EightyTwenty,
}

impl ThresholdBase {
    /// The base's name, as the card prints it.
    pub fn key(&self) -> &'static str {
        match self {
            ThresholdBase::Pfitzinger => "pfitzinger",
            ThresholdBase::EightyTwenty => "eighty_twenty",
        }
    }
}

/// The label a typed number carries — a **closed list**.
///
/// The app stores the label and shows it; it never parses free text
/// (`exploration/threshold-setup.md` §5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedSource {
    /// A laboratory test.
    LabTest,
    /// A coach.
    Coach,
    /// Another app's estimate.
    AnotherApp,
    /// A book or a table.
    BookOrTable,
    /// The athlete's own number.
    MyOwnNumber,
}

impl TypedSource {
    /// The label, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            TypedSource::LabTest => "a lab test",
            TypedSource::Coach => "a coach",
            TypedSource::AnotherApp => "another app",
            TypedSource::BookOrTable => "a book or a table",
            TypedSource::MyOwnNumber => "your own number",
        }
    }

    /// The label's identifier, as it is stored.
    pub fn key(&self) -> &'static str {
        match self {
            TypedSource::LabTest => "lab_test",
            TypedSource::Coach => "coach",
            TypedSource::AnotherApp => "another_app",
            TypedSource::BookOrTable => "book_or_table",
            TypedSource::MyOwnNumber => "my_own_number",
        }
    }
}

/// The watch's own zone model, with the day it was read.
///
/// This is the athlete's **configuration**, read once and dated — not a
/// measurement and not a series derived from a run.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchZoneModel {
    /// The model's five floors.
    pub floors: crate::zones::ZoneFloors,
    /// The day the model was read from the watch.
    pub read_on: Date,
}

/// One sample of a field test: the run's own record.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldTestSample {
    /// Seconds from the start of the run.
    pub elapsed_s: f64,
    /// Whether the athlete was moving when the sample was taken.
    pub moving: bool,
    /// The heart rate, in beats per minute.
    pub heart_rate_bpm: Option<f64>,
    /// The pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
}

/// A run the athlete pointed at, as the field-test rule reads it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldTest {
    /// The day of the run.
    pub date: Date,
    /// The whole run, in seconds — the elapsed time the trial took.
    ///
    /// The rule reads the **last 30 minutes** of the run, and the run's own
    /// length is what says where that window begins. When the length is not
    /// known, the last sample's own time is used instead.
    pub duration_s: Option<f64>,
    /// The run's own samples.
    pub samples: Vec<FieldTestSample>,
}

/// What the book's 45-minute time trial yields.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldTestEstimate {
    /// The mean heart rate over the window.
    pub heart_rate_bpm: f64,
    /// The pace over the window, in seconds per kilometre.
    pub pace_s_per_km: f64,
    /// The heart rate's drift across the window: the second half's mean minus
    /// the first half's mean.
    pub drift_bpm: f64,
    /// The rule's window, in seconds.
    pub window_s: f64,
    /// How much of the window the run actually covered.
    pub covered_s: f64,
}

/// The estimate the book's 45-minute time trial yields.
///
/// The rule reads the **last 30 minutes** of the run: the pace and the heart
/// rate over that window (ch. 1, ch. 7). A run shorter than the window still
/// answers — with the window it did cover — and the estimate says so.
///
/// `None` when the window holds no heart rate or no movement: there is
/// nothing to measure, and that is never a zero.
pub fn field_test_estimate(test: &FieldTest) -> Option<FieldTestEstimate> {
    let last_sample = test
        .samples
        .iter()
        .map(|sample| sample.elapsed_s)
        .fold(f64::NEG_INFINITY, f64::max);
    if !last_sample.is_finite() {
        return None;
    }
    // The window is the last 30 minutes of the **run**: the run's own length
    // says where it begins, and the last sample is the fallback.
    let end = match test.duration_s {
        Some(duration_s) if duration_s.is_finite() && duration_s > 0.0 => duration_s,
        _ => last_sample,
    };
    let start = (end - FIELD_TEST_WINDOW_S).max(0.0);

    let in_window: Vec<&FieldTestSample> = test
        .samples
        .iter()
        .filter(|sample| sample.elapsed_s >= start && sample.elapsed_s <= end)
        .collect();
    if in_window.is_empty() {
        return None;
    }

    // The distance and the moving time the window holds. A sample owns the
    // span until the next one, so the spans are the samples' own.
    let mut distance_m = 0.0;
    let mut moving_s = 0.0;
    for pair in in_window.windows(2) {
        let sample = pair[0];
        let next = pair[1];
        let span = next.elapsed_s - sample.elapsed_s;
        if !span.is_finite() || span <= 0.0 || !sample.moving {
            continue;
        }
        if let Some(pace) = sample.pace_s_per_km {
            if pace.is_finite() && pace > 0.0 {
                // The pace is seconds per kilometre, so the distance the span
                // covered is the span divided by the pace.
                distance_m += span / pace * 1_000.0;
                moving_s += span;
            }
        }
    }
    if moving_s <= 0.0 || distance_m <= 0.0 {
        return None;
    }
    // The pace over the window: the moving time over the distance it covered.
    let pace_s_per_km = moving_s * 1_000.0 / distance_m;

    // The heart rate over the window: the mean of the moving samples that
    // carry one.
    let rates: Vec<f64> = in_window
        .iter()
        .filter(|sample| sample.moving)
        .filter_map(|sample| sample.heart_rate_bpm)
        .filter(|rate| rate.is_finite())
        .collect();
    if rates.is_empty() {
        return None;
    }
    let heart_rate_bpm = rates.iter().sum::<f64>() / rates.len() as f64;

    // The drift across the window: the second half's mean minus the first
    // half's mean. It is a fact about the samples, not a verdict.
    let middle = rates.len() / 2;
    let first = &rates[..middle];
    let second = &rates[middle..];
    let mean = |values: &[f64]| -> f64 {
        if values.is_empty() {
            f64::NAN
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    };
    let drift_bpm = if first.is_empty() || second.is_empty() {
        0.0
    } else {
        mean(second) - mean(first)
    };

    Some(FieldTestEstimate {
        heart_rate_bpm,
        pace_s_per_km,
        drift_bpm,
        window_s: FIELD_TEST_WINDOW_S,
        covered_s: end - start,
    })
}

/// A number the athlete brought.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypedThreshold {
    /// The heart rate, in beats per minute.
    pub heart_rate_bpm: Option<f64>,
    /// The pace band, in seconds per kilometre.
    pub pace: Option<PaceBand>,
    /// The label the athlete picked, from the closed list.
    pub label: TypedSource,
    /// The day the number was entered.
    pub entered_on: Date,
}

/// What a candidate carries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CandidateValue {
    /// A heart rate, in beats per minute.
    HeartRate(f64),
    /// A pace band, in seconds per kilometre.
    Pace(PaceBand),
    /// A heart rate and a pace band together — what a lab test brings.
    Both {
        /// The heart rate, in beats per minute.
        heart_rate_bpm: f64,
        /// The pace band, in seconds per kilometre.
        pace: PaceBand,
    },
    /// No value, and why.
    Missing(MissingInput),
}

impl CandidateValue {
    /// The heart rate, when the candidate carries one.
    pub fn heart_rate_bpm(&self) -> Option<f64> {
        match self {
            CandidateValue::HeartRate(bpm) => Some(*bpm),
            CandidateValue::Both { heart_rate_bpm, .. } => Some(*heart_rate_bpm),
            _ => None,
        }
    }

    /// The pace band, when the candidate carries one.
    pub fn pace(&self) -> Option<PaceBand> {
        match self {
            CandidateValue::Pace(band) => Some(*band),
            CandidateValue::Both { pace, .. } => Some(*pace),
            _ => None,
        }
    }

    /// The reason the value is missing, when it is.
    pub fn missing(&self) -> Option<MissingInput> {
        match self {
            CandidateValue::Missing(missing) => Some(*missing),
            _ => None,
        }
    }
}

/// An input a candidate is missing, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingInput {
    /// No zone model has been read from the watch.
    NoZoneModel,
    /// The stored zone model's floors cannot be used.
    UnusableZoneModel,
    /// No race time has been entered.
    NoRaceTime,
    /// No run has been pointed at for the estimate.
    NoTestRun,
    /// The run has no samples the rule's window can read.
    NoTestWindow,
    /// No number has been entered.
    NoTypedValue,
}

impl MissingInput {
    /// The reason, in the core's own words.
    pub fn reason(&self) -> Option<String> {
        let id = match self {
            MissingInput::NoZoneModel => "zones.no_model",
            MissingInput::UnusableZoneModel => "zones.unusable_model",
            MissingInput::NoRaceTime => "pace.missing.race",
            MissingInput::NoTestRun => "threshold.missing.test",
            MissingInput::NoTestWindow => "threshold.missing.window",
            MissingInput::NoTypedValue => "threshold.missing.typed",
        };
        strings::text(id).map(|text| text.to_string())
    }
}

/// The watch's own rule: the floor of the fifth zone.
///
/// This is the product's **own convention** — the load model reads the same
/// floor as the threshold (`crates/stride-core/src/load.rs`) — and not the
/// watch's estimate of one.
pub const RULE_WATCH: Rule = Rule {
    id: "zone_model.zone5_floor",
    citation: "the product's own convention",
    string_id: "threshold.rule.watch",
};

/// The book's hour-pace band, computed from a race.
pub const RULE_RACE: Rule = Rule {
    id: "book.lt_pace_band",
    citation: "ch. 1, ch. 7",
    string_id: "threshold.rule.race",
};

/// The book's 45-minute time trial.
pub const RULE_FIELD_TEST: Rule = Rule {
    id: "book.45min_tt",
    citation: "ch. 1, ch. 7",
    string_id: "threshold.rule.test",
};

/// One candidate: a value, a date, and the rule that computed it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Candidate {
    /// Where the candidate comes from.
    pub source: ThresholdSource,
    /// The value — or the reason there is none.
    pub value: CandidateValue,
    /// The day of the fact the value came from: the model's read day, the
    /// race's day, the run's day, the day it was entered.
    pub date: Option<Date>,
    /// The rule that computed it, where a rule did.
    pub rule: Option<Rule>,
    /// The note that says what the candidate cannot know. It lives in the
    /// notes, not in the number.
    pub note_id: Option<&'static str>,
}

impl Candidate {
    /// The candidate's value, rendered: "165 bpm", "4:16–4:17 /km", or both.
    pub fn render(&self) -> Option<String> {
        let heart_rate = self
            .value
            .heart_rate_bpm()
            .and_then(|bpm| units::render_heart_rate(Some(bpm)));
        let pace = self.value.pace().and_then(|band| band.render());
        match (heart_rate, pace) {
            (Some(rate), Some(pace)) => Some(format!("{rate} · {pace}")),
            (Some(rate), None) => Some(rate),
            (None, Some(pace)) => Some(pace),
            (None, None) => None,
        }
    }

    /// The note that says what this candidate cannot know.
    pub fn note(&self) -> Option<String> {
        let id = self.note_id?;
        strings::text(id).map(|text| text.to_string())
    }
}

/// Everything the four candidates are computed from.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThresholdInputs {
    /// The watch's own zone model.
    pub watch: Option<WatchZoneModel>,
    /// The race the athlete typed.
    pub race: Option<Race>,
    /// The run the athlete pointed at for the estimate.
    pub field_test: Option<FieldTest>,
    /// The number the athlete brought.
    pub typed: Option<TypedThreshold>,
}

/// The four candidates, in the product's own order.
///
/// A candidate with a missing input is **no value with a reason** — never a
/// zero — and the other three still stand.
pub fn candidates(inputs: &ThresholdInputs) -> Vec<Candidate> {
    vec![
        watch_candidate(inputs),
        race_candidate(inputs),
        field_test_candidate(inputs),
        typed_candidate(inputs),
    ]
}

/// The watch's own zone model: the floor of the fifth zone.
fn watch_candidate(inputs: &ThresholdInputs) -> Candidate {
    let (value, date) = match inputs.watch {
        Some(model) if model.floors.is_usable() => (
            CandidateValue::HeartRate(model.floors.zone5),
            Some(model.read_on),
        ),
        Some(_) => (
            CandidateValue::Missing(MissingInput::UnusableZoneModel),
            None,
        ),
        None => (CandidateValue::Missing(MissingInput::NoZoneModel), None),
    };
    Candidate {
        source: ThresholdSource::WatchZoneModel,
        value,
        date,
        rule: Some(RULE_WATCH),
        note_id: Some("threshold.limit.watch"),
    }
}

/// From a race: the hour-pace band, on the athlete's own race.
///
/// The band is the **15K-to-10-mile race pace** — the book's approximation
/// of "the pace you could race for about an hour" (ch. 1, ch. 7). It is
/// the same rule as the ladder's threshold rung, on a different anchor: the
/// ladder's band is on the goal distance's span, and this one is on the race
/// the athlete typed. Neither is ever quoted as the other.
fn race_candidate(inputs: &ThresholdInputs) -> Candidate {
    let band = inputs.race.as_ref().and_then(|race| {
        let effort = vo2::effort_from_race(race.distance_m, race.time_s)?;
        let equivalents = vo2::equivalents(
            effort.vdot,
            race.time_s,
            race.distance_m,
            &[vo2::FIFTEEN_K, vo2::TEN_MILE],
        );
        let fifteen_k = equivalents
            .iter()
            .find(|equivalent| (equivalent.distance_m - vo2::FIFTEEN_K).abs() < 1.0)
            .and_then(|equivalent| equivalent.riegel_pace_s_per_km());
        let ten_mile = equivalents
            .iter()
            .find(|equivalent| (equivalent.distance_m - vo2::TEN_MILE).abs() < 1.0)
            .and_then(|equivalent| equivalent.riegel_pace_s_per_km());
        Some(PaceBand::between(fifteen_k?, ten_mile?))
    });
    let (value, date) = match band {
        Some(band) => (
            CandidateValue::Pace(band),
            inputs.race.as_ref().map(|r| r.date),
        ),
        None => (CandidateValue::Missing(MissingInput::NoRaceTime), None),
    };
    Candidate {
        source: ThresholdSource::Race,
        value,
        date,
        rule: Some(RULE_RACE),
        note_id: Some("threshold.limit.race"),
    }
}

/// From a field test: the estimate from the run the athlete pointed at.
fn field_test_candidate(inputs: &ThresholdInputs) -> Candidate {
    let estimate = inputs.field_test.as_ref().and_then(field_test_estimate);
    let (value, date) = match (estimate, inputs.field_test.as_ref()) {
        (Some(estimate), Some(test)) => (
            CandidateValue::Both {
                heart_rate_bpm: estimate.heart_rate_bpm,
                pace: PaceBand::single(estimate.pace_s_per_km),
            },
            Some(test.date),
        ),
        (Some(_), None) => (CandidateValue::Missing(MissingInput::NoTestRun), None),
        (None, Some(test)) => (
            CandidateValue::Missing(MissingInput::NoTestWindow),
            Some(test.date),
        ),
        (None, None) => (CandidateValue::Missing(MissingInput::NoTestRun), None),
    };
    Candidate {
        source: ThresholdSource::FieldTest,
        value,
        date,
        rule: Some(RULE_FIELD_TEST),
        note_id: Some("threshold.limit.test"),
    }
}

/// Typed by the athlete: a value, or a pair.
fn typed_candidate(inputs: &ThresholdInputs) -> Candidate {
    let (value, date) = match inputs.typed.as_ref() {
        Some(typed) => {
            let value = match (typed.heart_rate_bpm, typed.pace) {
                (Some(bpm), Some(band)) => CandidateValue::Both {
                    heart_rate_bpm: bpm,
                    pace: band,
                },
                (Some(bpm), None) => CandidateValue::HeartRate(bpm),
                (None, Some(band)) => CandidateValue::Pace(band),
                (None, None) => CandidateValue::Missing(MissingInput::NoTypedValue),
            };
            (value, Some(typed.entered_on))
        }
        None => (CandidateValue::Missing(MissingInput::NoTypedValue), None),
    };
    Candidate {
        source: ThresholdSource::Typed,
        value,
        date,
        rule: None,
        note_id: Some("threshold.limit.typed"),
    }
}

/// A choice the athlete made: the threshold in use, with its provenance.
///
/// One choice is in force at a time; the earlier choices stay in the dated
/// history with their base named (`exploration/threshold-setup.md` §6.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdChoice {
    /// Where the choice came from.
    pub source: ThresholdSource,
    /// The heart rate, when the choice carries one.
    pub heart_rate_bpm: Option<f64>,
    /// The pace band, when the choice carries one.
    pub pace: Option<PaceBand>,
    /// The rule that computed the value, where one did.
    pub rule_id: Option<String>,
    /// The base the choice was made for.
    pub base: ThresholdBase,
    /// The day of the fact the value came from.
    pub origin_date: Option<Date>,
    /// What the value came from — the model's, the race's or the run's id.
    pub origin_ref: Option<String>,
    /// The label the athlete picked, for a typed choice.
    pub label: Option<TypedSource>,
    /// The day the choice was made.
    pub entered_on: Date,
    /// The day the choice was last changed.
    pub updated_on: Date,
    /// Whether this choice is the one in force.
    pub in_force: bool,
}

impl ThresholdChoice {
    /// The choice's value, rendered: "165 bpm", "4:16–4:17 /km", or both.
    pub fn render(&self) -> Option<String> {
        let heart_rate = self
            .heart_rate_bpm
            .and_then(|bpm| units::render_heart_rate(Some(bpm)));
        let pace = self.pace.and_then(|band| band.render());
        match (heart_rate, pace) {
            (Some(rate), Some(pace)) => Some(format!("{rate} · {pace}")),
            (Some(rate), None) => Some(rate),
            (None, Some(pace)) => Some(pace),
            (None, None) => None,
        }
    }
}

/// The choice in force — the one every dependent answer reads.
///
/// `None` when no choice has been made: the setup may be skipped, and every
/// answer that depends on a threshold is then a no value with a reason.
pub fn in_force(choices: &[ThresholdChoice]) -> Option<&ThresholdChoice> {
    choices.iter().find(|choice| choice.in_force)
}

/// The choices in their dated history, newest first.
///
/// The earlier choices are **not** deleted and **not** edited: the history is
/// a list of facts ("your threshold on 3 November 2026 was 165 bpm").
pub fn history(choices: &[ThresholdChoice]) -> Vec<&ThresholdChoice> {
    let mut dated: Vec<&ThresholdChoice> = choices.iter().collect();
    dated.sort_by_key(|choice| std::cmp::Reverse(choice.entered_on));
    dated
}

/// The choice in force **on a day**.
///
/// The history is a list of dated facts, and the one in force on a day is
/// the last choice made on or before it. A plan adopted on a day is paced
/// by the choice that was in force **that day** — and a choice made later
/// does not reach back into it.
pub fn in_force_at(choices: &[ThresholdChoice], day: Date) -> Option<&ThresholdChoice> {
    choices
        .iter()
        .filter(|choice| choice.entered_on <= day)
        .max_by_key(|choice| choice.entered_on)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::zones::ZoneFloors;

    /// The canonical athlete's watch model, read on 3 November 2026.
    fn watch_model() -> WatchZoneModel {
        WatchZoneModel {
            floors: ZoneFloors {
                zone1: 115.0,
                zone2: 128.0,
                zone3: 143.0,
                zone4: 159.0,
                zone5: 165.0,
            },
            read_on: Date::from_ymd(2026, 11, 3).expect("a date"),
        }
    }

    /// The canonical athlete's race: a 10K in 41:36 on 12 October 2026.
    fn race() -> Race {
        Race::new(
            vo2::TEN_K,
            2_496.0,
            Date::from_ymd(2026, 10, 12).expect("a date"),
        )
    }

    /// A field test: a 45-minute run whose last 30 minutes were held at
    /// 166 bpm and 4:22 /km.
    fn field_test() -> FieldTest {
        let pace = 262.0;
        let mut samples = Vec::new();
        // The first fifteen minutes: the same pace, a lower heart rate — and
        // outside the rule's window.
        for step in 0..15 {
            samples.push(FieldTestSample {
                elapsed_s: step as f64 * 60.0,
                moving: true,
                heart_rate_bpm: Some(160.0),
                pace_s_per_km: Some(pace),
            });
        }
        // The last thirty minutes: the window the rule reads. The first
        // half of it at 164 and the second at 168, so the window's mean is
        // 166 and the drift across it is 4.
        for step in 15..30 {
            samples.push(FieldTestSample {
                elapsed_s: step as f64 * 60.0,
                moving: true,
                heart_rate_bpm: Some(164.0),
                pace_s_per_km: Some(pace),
            });
        }
        for step in 30..45 {
            samples.push(FieldTestSample {
                elapsed_s: step as f64 * 60.0,
                moving: true,
                heart_rate_bpm: Some(168.0),
                pace_s_per_km: Some(pace),
            });
        }
        FieldTest {
            date: Date::from_ymd(2026, 3, 3).expect("a date"),
            duration_s: Some(45.0 * 60.0),
            samples,
        }
    }

    fn inputs() -> ThresholdInputs {
        ThresholdInputs {
            watch: Some(watch_model()),
            race: Some(race()),
            field_test: Some(field_test()),
            typed: Some(TypedThreshold {
                heart_rate_bpm: Some(168.0),
                pace: Some(PaceBand::between(246.0, 252.0)),
                label: TypedSource::LabTest,
                entered_on: Date::from_ymd(2026, 3, 3).expect("a date"),
            }),
        }
    }

    #[test]
    fn the_watch_candidate_is_the_fifth_zone_s_floor() {
        let candidates = candidates(&inputs());
        let watch = &candidates[0];
        assert_eq!(watch.source, ThresholdSource::WatchZoneModel);
        assert_eq!(watch.value.heart_rate_bpm(), Some(165.0));
        assert_eq!(watch.render(), Some("165 bpm".to_string()));
        assert_eq!(
            watch.date,
            Some(Date::from_ymd(2026, 11, 3).expect("a date"))
        );
    }

    #[test]
    fn the_race_candidate_is_the_hour_pace_band_on_the_typed_race() {
        let candidates = candidates(&inputs());
        let race = &candidates[1];
        let band = race.value.pace().expect("a band");
        // The 15K pace (4:16) to the 10-mile pace (4:17) — the book's own
        // anchor, and not the ladder's band on the goal distance.
        assert!((band.fast_s_per_km - 255.75).abs() < 0.2);
        assert!((band.slow_s_per_km - 256.83).abs() < 0.2);
        assert_eq!(race.render(), Some("4:16–4:17 /km".to_string()));
    }

    #[test]
    fn the_field_test_candidate_reads_the_last_thirty_minutes() {
        let candidates = candidates(&inputs());
        let test = &candidates[2];
        assert_eq!(test.value.heart_rate_bpm(), Some(166.0));
        let band = test.value.pace().expect("a band");
        assert!((band.fast_s_per_km - 262.0).abs() < 0.01);
        assert_eq!(test.render(), Some("166 bpm · 4:22 /km".to_string()));
    }

    #[test]
    fn the_field_test_reports_the_drift_across_its_window() {
        let estimate = field_test_estimate(&field_test()).expect("an estimate");
        // The window's first half ran at 164 and its second half at 168.
        assert!((estimate.drift_bpm - 4.0).abs() < 0.01);
        assert!((estimate.covered_s - 1_800.0).abs() < 0.01);
    }

    #[test]
    fn a_run_shorter_than_the_window_still_answers() {
        let mut test = field_test();
        test.samples.truncate(20);
        test.duration_s = Some(20.0 * 60.0);
        let estimate = field_test_estimate(&test).expect("an estimate");
        // The window the rule names, and the twenty minutes the run covered.
        assert!((estimate.covered_s - 1_200.0).abs() < 0.01);
        assert!((estimate.window_s - 1_800.0).abs() < 0.01);
    }

    #[test]
    fn the_typed_candidate_carries_a_pair_and_its_label() {
        let candidates = candidates(&inputs());
        let typed = &candidates[3];
        assert_eq!(typed.source, ThresholdSource::Typed);
        assert_eq!(typed.value.heart_rate_bpm(), Some(168.0));
        assert_eq!(typed.render(), Some("168 bpm · 4:06–4:12 /km".to_string()));
        // A typed number has no rule: it is the athlete's own.
        assert!(typed.rule.is_none());
    }

    #[test]
    fn a_candidate_with_a_missing_input_is_no_value_with_a_reason() {
        let empty = ThresholdInputs::default();
        let candidates = candidates(&empty);
        assert_eq!(
            candidates[0].value.missing(),
            Some(MissingInput::NoZoneModel)
        );
        assert_eq!(
            candidates[1].value.missing(),
            Some(MissingInput::NoRaceTime)
        );
        assert_eq!(candidates[2].value.missing(), Some(MissingInput::NoTestRun));
        assert_eq!(
            candidates[3].value.missing(),
            Some(MissingInput::NoTypedValue)
        );
        // And the reason is the core's own sentence.
        assert!(MissingInput::NoZoneModel
            .reason()
            .unwrap_or_default()
            .contains("zones"));
    }

    #[test]
    fn an_unusable_zone_model_is_not_a_threshold() {
        let mut inputs = inputs();
        let mut model = watch_model();
        model.floors.zone4 = 200.0;
        inputs.watch = Some(model);
        let candidates = candidates(&inputs);
        assert_eq!(
            candidates[0].value.missing(),
            Some(MissingInput::UnusableZoneModel)
        );
    }

    #[test]
    fn each_candidate_carries_the_note_that_says_what_it_cannot_know() {
        let candidates = candidates(&inputs());
        for candidate in &candidates {
            assert!(candidate.note().is_some());
        }
        assert_eq!(
            candidates[0].note(),
            Some("the app cannot check what was put into the watch".to_string())
        );
        assert_eq!(
            candidates[1].note(),
            Some(
                "a race is a race, not a test — the pace carries that day's conditions".to_string()
            )
        );
    }

    #[test]
    fn one_choice_is_in_force_and_the_history_stays() {
        let march = ThresholdChoice {
            source: ThresholdSource::Typed,
            heart_rate_bpm: Some(172.0),
            pace: None,
            rule_id: None,
            base: ThresholdBase::Pfitzinger,
            origin_date: None,
            origin_ref: None,
            label: Some(TypedSource::Coach),
            entered_on: Date::from_ymd(2026, 3, 3).expect("a date"),
            updated_on: Date::from_ymd(2026, 3, 3).expect("a date"),
            in_force: false,
        };
        let november = ThresholdChoice {
            source: ThresholdSource::WatchZoneModel,
            heart_rate_bpm: Some(165.0),
            pace: None,
            rule_id: Some("zone_model.zone5_floor".to_string()),
            base: ThresholdBase::Pfitzinger,
            origin_date: Some(Date::from_ymd(2026, 11, 3).expect("a date")),
            origin_ref: None,
            label: None,
            entered_on: Date::from_ymd(2026, 11, 3).expect("a date"),
            updated_on: Date::from_ymd(2026, 11, 3).expect("a date"),
            in_force: true,
        };
        let choices = vec![march.clone(), november.clone()];

        let in_use = in_force(&choices).expect("a choice in force");
        assert_eq!(in_use.heart_rate_bpm, Some(165.0));

        let history = history(&choices);
        assert_eq!(history.len(), 2);
        // Newest first.
        assert_eq!(history[0].entered_on, november.entered_on);
        assert_eq!(history[1].entered_on, march.entered_on);
        // And the earlier choice keeps its base and its label.
        assert_eq!(history[1].base, ThresholdBase::Pfitzinger);
        assert_eq!(history[1].label, Some(TypedSource::Coach));
    }

    #[test]
    fn no_choice_at_all_is_no_value_and_not_a_default() {
        assert!(in_force(&[]).is_none());
    }
}
