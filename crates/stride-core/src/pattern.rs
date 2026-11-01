//! The pattern engine: what usually happened to this athlete.
//!
//! `docs/contract/voice.md`: *a `pattern` string must carry its sample size
//! **n**; never emitted from fewer than **three** comparable days (two is
//! "twice so far", not a pattern).*
//!
//! A pattern is a fact about the past, stated as such:
//!
//! * a **streak** — "your HRV has been below your usual for 3 nights";
//! * a **comparable outcome** — "the 5 other times your sleep was below your
//!   usual, you ran 8.4 km or less".
//!
//! It is not a forecast and not a rule about what to do, and it never lives
//! beside a suggestion: it belongs to Body, over the visible history.

use serde::{Deserialize, Serialize};

use crate::band::{self};
use crate::date::Date;
use crate::direction::{BandSide, DeviationSide};
use crate::strings;
use crate::types::{VitalKind, Vitals, WellnessDay};
use crate::units;

/// One day of the athlete's history, as the pattern engine sees it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryDay {
    /// The athlete-local day.
    pub date: Date,
    /// The three vitals.
    pub vitals: Vitals,
    /// The distance run that day, in metres, if there was one.
    pub distance_m: Option<f64>,
}

impl HistoryDay {
    /// A day of history.
    pub fn new(date: Date, vitals: Vitals, distance_m: Option<f64>) -> Self {
        Self {
            date,
            vitals,
            distance_m,
        }
    }
}

/// What kind of pattern this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    /// A run of consecutive nights on one side of the usual.
    UsualStreak {
        /// The vital the streak is about.
        vital: VitalKind,
        /// Which side of the usual the streak is on.
        side: DeviationSide,
    },
    /// What usually happened on the other comparable days.
    ComparableOutcome {
        /// The vital the comparable days share.
        vital: VitalKind,
        /// Which side of the usual they are on.
        side: DeviationSide,
    },
}

/// A pattern, with the sample size it was read from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern {
    /// What kind of pattern this is.
    pub kind: PatternKind,
    /// How many comparable days the statement is read from.
    pub sample_size: usize,
    /// The sentence the app shows.
    pub text: String,
}

/// The patterns the athlete's history shows today.
///
/// The history must include today: a pattern is about a state the athlete is
/// in now, compared with the days that looked the same.
pub fn patterns(history: &[HistoryDay], today: Date) -> Vec<Pattern> {
    // The band engine reads wellness days; the pattern engine reads history
    // days. The conversion happens once.
    let wellness: Vec<WellnessDay> = history
        .iter()
        .map(|entry| WellnessDay::new(entry.date, entry.vitals))
        .collect();
    let mut found: Vec<Pattern> = Vec::new();
    for vital in VitalKind::ALL {
        let side = match side_of_day(&wellness, today, vital) {
            Some(BandSide::Below) => DeviationSide::Below,
            Some(BandSide::Above) => DeviationSide::Above,
            // Inside the band there is no "this low" state to compare.
            Some(BandSide::Within) | None => continue,
        };
        let band_side = match side {
            DeviationSide::Below => BandSide::Below,
            DeviationSide::Above => BandSide::Above,
        };

        let nights = streak(&wellness, today, vital, band_side);
        if nights >= strings::MIN_PATTERN_SAMPLE {
            if let Some(pattern) = streak_pattern(vital, side, nights) {
                found.push(pattern);
            }
        }

        let comparable: Vec<&HistoryDay> = history
            .iter()
            .filter(|entry| entry.date < today)
            .filter(|entry| side_of_day(&wellness, entry.date, vital) == Some(band_side))
            .collect();
        let distances: Vec<f64> = comparable
            .iter()
            .filter_map(|entry| entry.distance_m)
            .collect();
        if distances.len() >= strings::MIN_PATTERN_SAMPLE {
            if let Some(pattern) = outcome_pattern(vital, side, &distances) {
                found.push(pattern);
            }
        }
    }
    found
}

/// Where a day's vital sits relative to its own band.
fn side_of_day(wellness: &[WellnessDay], day: Date, vital: VitalKind) -> Option<BandSide> {
    let entry = wellness.iter().find(|entry| entry.date == day)?;
    let value = vital.value(&entry.vitals)?;
    let band = band::band_for(wellness, day, vital)?;
    Some(crate::direction::band_side(value, &band))
}

/// The run of consecutive days, ending today, on one side of the usual.
fn streak(wellness: &[WellnessDay], today: Date, vital: VitalKind, side: BandSide) -> usize {
    let mut nights = 0usize;
    let mut day = today;
    // The window bounds the walk: a streak longer than the band window is not
    // a streak the engine will claim.
    let limit = band::BAND_WINDOW_NIGHTS as usize;
    while nights < limit {
        match side_of_day(wellness, day, vital) {
            Some(found) if found == side => nights += 1,
            _ => break,
        }
        day = day.add_days(-1);
    }
    nights
}

/// The streak pattern.
fn streak_pattern(vital: VitalKind, side: DeviationSide, nights: usize) -> Option<Pattern> {
    let count = nights.to_string();
    let text = strings::render(
        "pattern.usual_streak",
        &[
            ("vital", vital.phrase()),
            ("side", side.word()),
            ("n", &count),
        ],
    )?;
    Some(Pattern {
        kind: PatternKind::UsualStreak { vital, side },
        sample_size: nights,
        text,
    })
}

/// The comparable-days pattern.
///
/// On the worse side the days ran *less* (the longest of them is the claim);
/// on the better side they ran *more* (the shortest of them is the claim).
fn outcome_pattern(vital: VitalKind, side: DeviationSide, distances: &[f64]) -> Option<Pattern> {
    if distances.len() < strings::MIN_PATTERN_SAMPLE {
        return None;
    }
    let claim = match side {
        DeviationSide::Below => distances.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        DeviationSide::Above => distances.iter().copied().fold(f64::INFINITY, f64::min),
    };
    let distance = format!("{} km", units::format_distance_km_short(claim));
    let count = distances.len().to_string();
    let id = match side {
        DeviationSide::Below => "pattern.comparable_outcome",
        DeviationSide::Above => "pattern.comparable_outcome.at_least",
    };
    let text = strings::render(
        id,
        &[
            ("n", &count),
            ("vital", vital.phrase()),
            ("side", side.word()),
            ("distance", &distance),
        ],
    )?;
    Some(Pattern {
        kind: PatternKind::ComparableOutcome { vital, side },
        sample_size: distances.len(),
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Hrv, Sleep};

    fn date(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    /// A night with an HRV and a run distance.
    fn day(text: &str, hrv: f64, distance_m: Option<f64>) -> Option<HistoryDay> {
        let date = date(text)?;
        let total = crate::types::Duration::from_hours(7.0)?;
        let hrv = Hrv::from_ms(hrv)?;
        Some(HistoryDay::new(
            date,
            Vitals {
                sleep: Some(Sleep::new(total)),
                hrv: Some(hrv),
                ..Vitals::default()
            },
            distance_m,
        ))
    }

    fn history() -> Option<Vec<HistoryDay>> {
        let days = vec![
            day("2026-02-19", 62.0, Some(7000.0))?,
            day("2026-02-20", 62.0, Some(8000.0))?,
            day("2026-02-21", 62.0, Some(9000.0))?,
            day("2026-02-22", 55.0, Some(6000.0))?,
            day("2026-02-23", 52.0, Some(5500.0))?,
            day("2026-02-24", 49.0, Some(5000.0))?,
            day("2026-02-25", 48.0, Some(4000.0))?,
        ];
        Some(days)
    }

    #[test]
    fn a_pattern_is_never_read_from_fewer_than_three_days() {
        let history = match history() {
            Some(history) => history,
            None => return,
        };
        let today = match date("2026-02-25") {
            Some(day) => day,
            None => return,
        };
        let found = patterns(&history, today);
        for pattern in &found {
            assert!(
                pattern.sample_size >= strings::MIN_PATTERN_SAMPLE,
                "a pattern was read from {} days",
                pattern.sample_size
            );
        }
    }

    #[test]
    fn a_streak_of_four_nights_is_a_pattern_and_two_are_not() {
        let history = match history() {
            Some(history) => history,
            None => return,
        };
        // 22, 23, 24 and 25 February are below the usual: four nights.
        let today = match date("2026-02-25") {
            Some(day) => day,
            None => return,
        };
        let found = patterns(&history, today);
        let streak = found
            .iter()
            .find(|pattern| matches!(pattern.kind, PatternKind::UsualStreak { .. }));
        assert!(streak.is_some());
        let streak = match streak {
            Some(pattern) => pattern,
            None => return,
        };
        assert_eq!(streak.sample_size, 4);
        assert!(streak.text.contains("4 nights"));
        assert!(streak.text.contains("HRV"));
        assert!(streak.text.contains("below your usual"));

        // On 2026-02-23 the streak is two nights, which is not a pattern.
        let today = match date("2026-02-23") {
            Some(day) => day,
            None => return,
        };
        let found = patterns(&history, today);
        assert!(found
            .iter()
            .all(|pattern| !matches!(pattern.kind, PatternKind::UsualStreak { .. })));
    }

    #[test]
    fn a_comparable_outcome_carries_its_sample_size() {
        let history = match history() {
            Some(history) => history,
            None => return,
        };
        let today = match date("2026-02-25") {
            Some(day) => day,
            None => return,
        };
        let found = patterns(&history, today);
        let outcome = found
            .iter()
            .find(|pattern| matches!(pattern.kind, PatternKind::ComparableOutcome { .. }));
        assert!(outcome.is_some());
        let outcome = match outcome {
            Some(pattern) => pattern,
            None => return,
        };
        // The other three nights below the usual, and the longest of them was
        // 6 km.
        assert_eq!(outcome.sample_size, 3);
        assert!(outcome.text.contains("3 other times"));
        assert!(outcome.text.contains("6 km or less"));
    }

    #[test]
    fn a_pattern_is_never_emitted_without_its_sample_size() {
        let history = match history() {
            Some(history) => history,
            None => return,
        };
        let today = match date("2026-02-25") {
            Some(day) => day,
            None => return,
        };
        for pattern in patterns(&history, today) {
            let violations =
                strings::lint::check_rendered_pattern(&pattern.text, pattern.sample_size);
            assert!(
                violations.is_empty(),
                "{:?} breaks the pattern rule: {violations:?}",
                pattern.text
            );
        }
    }

    #[test]
    fn a_day_inside_its_band_has_no_pattern() {
        let history = match history() {
            Some(history) => history,
            None => return,
        };
        // On 2026-02-22 there is one night below the usual and no comparable
        // day before it: nothing to say.
        let today = match date("2026-02-22") {
            Some(day) => day,
            None => return,
        };
        let found = patterns(&history, today);
        assert!(found.is_empty());
    }
}
