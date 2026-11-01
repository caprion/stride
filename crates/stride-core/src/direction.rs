//! The direction engine: a vital, its band, and the words the app shows.
//!
//! The contract (`docs/contract/core-contract.md` §4) fixes the shape of the
//! answer:
//!
//! * inside the band → "about usual"; outside it → "better than usual" or
//!   "could be better", depending on which side is the better side;
//! * the word comes from a **closed set** — three words, no synonyms, no
//!   extensions;
//! * beside it sit the muted arithmetic ("1 h 10 m less than your usual") and
//!   the direction of travel ("3rd night down", "steady"), both descriptive,
//!   never a verdict.
//!
//! "Not enough data for a comparison yet" is a legitimate answer, and it never
//! degrades into a number: without a band there is no comparison, so there is
//! no deviation either — only the raw value.

use serde::{Deserialize, Serialize};

use crate::band::{self, Band};
use crate::date::Date;
use crate::strings;
use crate::types::{BetterSide, VitalKind, WellnessDay};
use crate::units;

/// The direction word. The set is closed: three words, and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// On the better side of the band.
    BetterThanUsual,
    /// Inside the band.
    AboutUsual,
    /// On the worse side of the band.
    CouldBeBetter,
}

impl Direction {
    /// The catalogue id of this word.
    pub fn id(&self) -> &'static str {
        match self {
            Direction::BetterThanUsual => "direction.better_than_usual",
            Direction::AboutUsual => "direction.about_usual",
            Direction::CouldBeBetter => "direction.could_be_better",
        }
    }

    /// The word itself.
    pub fn as_str(&self) -> &'static str {
        strings::text(self.id()).unwrap_or("")
    }

    /// The direction word with this text, if it is one of the three.
    pub fn from_text(text: &str) -> Option<Self> {
        match text {
            strings::BETTER_THAN_USUAL => Some(Direction::BetterThanUsual),
            strings::ABOUT_USUAL => Some(Direction::AboutUsual),
            strings::COULD_BE_BETTER => Some(Direction::CouldBeBetter),
            _ => None,
        }
    }
}

/// Which side of the usual a measurement sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviationSide {
    /// Below the usual.
    Below,
    /// Above the usual.
    Above,
}

impl DeviationSide {
    /// The word the templates use: "below", "above".
    pub fn word(&self) -> &'static str {
        match self {
            DeviationSide::Below => "below",
            DeviationSide::Above => "above",
        }
    }
}

/// Where a measurement sits relative to the band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BandSide {
    /// Below the band.
    Below,
    /// Inside the band.
    Within,
    /// Above the band.
    Above,
}

/// How a vital has moved over the last few days.
///
/// Descriptive, never a verdict: "3rd night down", "steady".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Travel {
    /// The last two readings are the same.
    Steady,
    /// The last `nights` readings each went up.
    Up {
        /// How many nights in a row.
        nights: usize,
    },
    /// The last `nights` readings each went down.
    Down {
        /// How many nights in a row.
        nights: usize,
    },
}

/// One vital's direction, with everything the app shows beside it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VitalDirection {
    /// Which vital this is.
    pub vital: VitalKind,
    /// The measured value, in the vital's own scale.
    pub value: Option<f64>,
    /// The direction word, or `None` when there is no comparison to make.
    pub direction: Option<Direction>,
    /// The difference from the usual, in the vital's own scale.
    pub deviation: Option<f64>,
    /// The band the usual was drawn from.
    pub band: Option<Band>,
    /// How the vital has moved over the last few days.
    pub travel: Option<Travel>,
}

impl VitalDirection {
    /// The value as the athlete reads it: "6 h 10 m", "48 ms", "63 bpm".
    ///
    /// The wording is the catalogue's and the rounding is `units.rs`'s; this
    /// module only chooses which vital it is.
    pub fn value_text(&self) -> Option<String> {
        let value = self.value?;
        match self.vital {
            VitalKind::Sleep => units::render_duration(Some(value)),
            VitalKind::Hrv => {
                let number = units::format_whole(value);
                strings::render("vital.hrv.value", &[("value", &number)])
            }
            VitalKind::RestingHr => {
                let number = units::format_truncated(value);
                strings::render("vital.resting_hr.value", &[("value", &number)])
            }
        }
    }

    /// The direction word.
    pub fn direction_text(&self) -> Option<&'static str> {
        self.direction.map(|direction| direction.as_str())
    }

    /// The muted arithmetic: "14 below your usual", "1 h 10 m less than your
    /// usual".
    ///
    /// `None` when there is no comparison, and `None` when the difference
    /// rounds to nothing — a difference of zero is not news.
    pub fn deviation_text(&self) -> Option<String> {
        let deviation = self.deviation?;
        deviation_phrase(self.vital, deviation)
    }

    /// The direction of travel: "3rd night down", "steady".
    ///
    /// The travel is a *direction*, so it needs two nights to have one: a
    /// single night of movement is not a direction of travel. And "steady" is
    /// worth saying only when the reading is unusual — a usual night that
    /// did not move needs no adjective.
    ///
    /// Without a comparison there is no travel either: the row says it has no
    /// data, and that is the whole of the row.
    pub fn travel_text(&self) -> Option<String> {
        let travel = self.travel?;
        let direction = self.direction?;
        match travel {
            Travel::Steady => {
                if direction == Direction::AboutUsual {
                    return None;
                }
                travel_text(travel)
            }
            Travel::Up { nights } | Travel::Down { nights } => {
                if nights < 2 {
                    return None;
                }
                travel_text(travel)
            }
        }
    }

    /// Where the value sits relative to the band.
    pub fn side(&self) -> Option<BandSide> {
        match (self.value, self.band) {
            (Some(value), Some(band)) => Some(band_side(value, &band)),
            _ => None,
        }
    }

    /// The whole line after the value: the direction, the arithmetic and the
    /// travel, separated by the middot the product uses.
    pub fn line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(direction) = self.direction_text() {
            parts.push(direction.to_string());
        }
        if let Some(deviation) = self.deviation_text() {
            parts.push(deviation);
        }
        if let Some(travel) = self.travel_text() {
            parts.push(travel);
        }
        parts.join(" · ")
    }

    /// The whole row: the value, then the line — or the status, when there is
    /// no comparison to make.
    ///
    /// The house style is label · value · direction, and the label is the
    /// catalogue's ("Sleep", "HRV", "Resting HR"), so the row starts at the
    /// value.
    pub fn row_text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(value) = self.value_text() {
            parts.push(value);
        }
        let line = self.line();
        if !line.is_empty() {
            parts.push(line);
        } else if let Some(status) = self.status_text() {
            parts.push(status);
        }
        parts.join(" · ")
    }

    /// What the row says when there is no comparison to make.
    pub fn status_text(&self) -> Option<String> {
        if self.direction.is_some() {
            return None;
        }
        strings::render("status.not_enough_data", &[])
    }
}

/// Where a value sits relative to a band.
pub fn band_side(value: f64, band: &Band) -> BandSide {
    if band.contains(value) {
        BandSide::Within
    } else if value < band.usual() {
        BandSide::Below
    } else {
        BandSide::Above
    }
}

/// The direction word for a value, its band, and which side is the better one.
pub fn classify(value: f64, band: &Band, better: BetterSide) -> Direction {
    match band_side(value, band) {
        BandSide::Within => Direction::AboutUsual,
        BandSide::Below => match better {
            BetterSide::Lower => Direction::BetterThanUsual,
            BetterSide::Higher => Direction::CouldBeBetter,
        },
        BandSide::Above => match better {
            BetterSide::Higher => Direction::BetterThanUsual,
            BetterSide::Lower => Direction::CouldBeBetter,
        },
    }
}

/// The muted arithmetic for a deviation, in the vital's own scale.
fn deviation_phrase(vital: VitalKind, deviation: f64) -> Option<String> {
    let below = deviation < 0.0;
    let magnitude = deviation.abs();
    match vital {
        // A duration is read as a duration: "1 h 10 m less than your usual".
        VitalKind::Sleep => {
            let value = units::format_duration(magnitude);
            if value == "0 m" {
                return None;
            }
            let id = if below {
                "direction.less_than_usual"
            } else {
                "direction.more_than_usual"
            };
            strings::render(id, &[("value", &value)])
        }
        // A count is read as a count: "14 below your usual".
        VitalKind::Hrv | VitalKind::RestingHr => {
            let value = units::format_whole(magnitude);
            if value == "0" {
                return None;
            }
            let id = if below {
                "direction.below_usual"
            } else {
                "direction.above_usual"
            };
            strings::render(id, &[("value", &value)])
        }
    }
}

/// The travel phrase.
fn travel_text(travel: Travel) -> Option<String> {
    match travel {
        Travel::Steady => strings::render("direction.travel.steady", &[]),
        Travel::Up { nights } => {
            let ordinal = units::ordinal(nights);
            strings::render("direction.travel.up", &[("ordinal", &ordinal)])
        }
        Travel::Down { nights } => {
            let ordinal = units::ordinal(nights);
            strings::render("direction.travel.down", &[("ordinal", &ordinal)])
        }
    }
}

/// The direction of travel over a series of readings, oldest first.
///
/// The streak counts the *changes*: three nights each lower than the one
/// before is a third night down.
pub fn travel(values: &[f64]) -> Option<Travel> {
    let last = values.last()?;
    let previous = values.get(values.len().checked_sub(2)?)?;
    if last == previous {
        return Some(Travel::Steady);
    }
    let rising = last > previous;
    let mut nights = 0usize;
    for index in (1..values.len()).rev() {
        let before = values[index - 1];
        let after = values[index];
        let step_up = after > before;
        let step_down = after < before;
        if (rising && step_up) || (!rising && step_down) {
            nights += 1;
        } else {
            break;
        }
    }
    Some(if rising {
        Travel::Up { nights }
    } else {
        Travel::Down { nights }
    })
}

/// The direction of travel for one vital, over the trailing window that ends
/// with the day itself.
pub fn travel_for(history: &[WellnessDay], day: Date, vital: VitalKind) -> Option<Travel> {
    let start = day.add_days(-band::BAND_WINDOW_NIGHTS);
    let mut days: Vec<&WellnessDay> = history
        .iter()
        .filter(|entry| entry.date >= start && entry.date <= day)
        .collect();
    days.sort_by_key(|entry| entry.date);
    let values: Vec<f64> = days
        .iter()
        .filter_map(|entry| vital.value(&entry.vitals))
        .collect();
    travel(&values)
}

/// One vital's direction on one day.
pub fn vital_direction(
    history: &[WellnessDay],
    day: Date,
    vital: VitalKind,
    vitals: crate::types::Vitals,
) -> VitalDirection {
    let value = vital.value(&vitals);
    let band = band::band_for(history, day, vital);
    let direction = match (value, band) {
        (Some(value), Some(band)) => Some(classify(value, &band, vital.better_side())),
        _ => None,
    };
    let deviation = match (value, band) {
        (Some(value), Some(band)) => Some(value - band.usual()),
        _ => None,
    };
    VitalDirection {
        vital,
        value,
        direction,
        deviation,
        band,
        travel: travel_for(history, day, vital),
    }
}

/// Everything Today and Body show about one day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayDirections {
    /// The day.
    pub date: Date,
    /// The three vitals, in the order the app shows them.
    pub vitals: Vec<VitalDirection>,
    /// The "Rest" line: a count and a direction.
    pub rest: crate::rest::RestSummary,
}

/// The directions for one day, read from the history of wellness days.
pub fn day_directions(history: &[WellnessDay], day: Date) -> DayDirections {
    let vitals = history
        .iter()
        .find(|entry| entry.date == day)
        .map(|entry| entry.vitals)
        .unwrap_or_default();
    let directions: Vec<VitalDirection> = VitalKind::ALL
        .iter()
        .map(|vital| vital_direction(history, day, *vital, vitals))
        .collect();
    let rest = crate::rest::summarize(&directions);
    DayDirections {
        date: day,
        vitals: directions,
        rest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::band::band;
    use crate::types::{Hrv, RestingHr, Sleep, Vitals};

    fn date(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    fn sleep_day(text: &str, hours: f64) -> Option<WellnessDay> {
        let date = date(text)?;
        let total = crate::types::Duration::from_hours(hours)?;
        Some(WellnessDay::new(
            date,
            Vitals {
                sleep: Some(Sleep::new(total)),
                ..Vitals::default()
            },
        ))
    }

    /// A band around `centre`, 1.5 wide on either side.
    fn band_around(centre: f64) -> Option<Band> {
        // The median of three readings is the middle one; the deviations are
        // 1, 0, 1, so the MAD is 1 and the band is 1.5 either side.
        band(&[centre - 1.0, centre, centre + 1.0])
    }

    #[test]
    fn the_closed_set_is_three_words() {
        assert_eq!(Direction::BetterThanUsual.as_str(), "better than usual");
        assert_eq!(Direction::AboutUsual.as_str(), "about usual");
        assert_eq!(Direction::CouldBeBetter.as_str(), "could be better");
        assert_eq!(
            Direction::from_text("about usual"),
            Some(Direction::AboutUsual)
        );
        assert_eq!(Direction::from_text("much worse"), None);
        // The word comes from the catalogue, not from a literal in the code.
        assert_eq!(
            Direction::CouldBeBetter.as_str(),
            strings::text("direction.could_be_better").unwrap_or("")
        );
    }

    #[test]
    fn inside_the_band_is_about_usual() {
        let band = match band_around(60.0) {
            Some(band) => band,
            None => return,
        };
        assert_eq!(
            classify(60.0, &band, BetterSide::Higher),
            Direction::AboutUsual
        );
        assert_eq!(
            classify(59.5, &band, BetterSide::Higher),
            Direction::AboutUsual
        );
        assert_eq!(
            classify(60.5, &band, BetterSide::Higher),
            Direction::AboutUsual
        );
    }

    #[test]
    fn the_better_side_is_a_fact_about_the_measurement() {
        let band = match band_around(60.0) {
            Some(band) => band,
            None => return,
        };
        // More sleep and a higher HRV are better; a lower resting heart rate
        // is better.
        assert_eq!(
            classify(70.0, &band, BetterSide::Higher),
            Direction::BetterThanUsual
        );
        assert_eq!(
            classify(50.0, &band, BetterSide::Higher),
            Direction::CouldBeBetter
        );
        assert_eq!(
            classify(50.0, &band, BetterSide::Lower),
            Direction::BetterThanUsual
        );
        assert_eq!(
            classify(70.0, &band, BetterSide::Lower),
            Direction::CouldBeBetter
        );
    }

    #[test]
    fn the_travel_counts_the_nights_not_the_readings() {
        assert_eq!(
            travel(&[62.0, 55.0, 50.0, 48.0]),
            Some(Travel::Down { nights: 3 })
        );
        assert_eq!(
            travel(&[48.0, 50.0, 55.0, 62.0]),
            Some(Travel::Up { nights: 3 })
        );
        assert_eq!(travel(&[50.0, 50.0]), Some(Travel::Steady));
        assert_eq!(travel(&[50.0]), None);
        assert_eq!(travel(&[]), None);
    }

    #[test]
    fn the_deviation_is_read_in_the_vitals_own_scale() {
        let sleep = VitalDirection {
            vital: VitalKind::Sleep,
            value: Some(6.0 * 3600.0 + 10.0 * 60.0),
            direction: Some(Direction::CouldBeBetter),
            deviation: Some(-(70.0 * 60.0)),
            band: None,
            travel: None,
        };
        assert_eq!(
            sleep.deviation_text(),
            Some("1 h 10 m less than your usual".to_string())
        );
        let hrv = VitalDirection {
            vital: VitalKind::Hrv,
            value: Some(48.0),
            direction: Some(Direction::CouldBeBetter),
            deviation: Some(-14.0),
            band: None,
            travel: None,
        };
        assert_eq!(
            hrv.deviation_text(),
            Some("14 below your usual".to_string())
        );
        let resting = VitalDirection {
            vital: VitalKind::RestingHr,
            value: Some(63.0),
            direction: Some(Direction::AboutUsual),
            deviation: Some(5.0),
            band: None,
            travel: None,
        };
        assert_eq!(
            resting.deviation_text(),
            Some("5 above your usual".to_string())
        );
    }

    #[test]
    fn a_difference_of_zero_is_not_shown() {
        let hrv = VitalDirection {
            vital: VitalKind::Hrv,
            value: Some(48.0),
            direction: Some(Direction::AboutUsual),
            deviation: Some(0.2),
            band: None,
            travel: None,
        };
        assert_eq!(hrv.deviation_text(), None);
    }

    #[test]
    fn without_a_band_there_is_no_comparison_and_no_number() {
        // Two nights of history are not enough for a band.
        let history = vec![
            match sleep_day("2026-03-01", 7.0) {
                Some(day) => day,
                None => return,
            },
            match sleep_day("2026-03-02", 7.0) {
                Some(day) => day,
                None => return,
            },
            match sleep_day("2026-03-03", 6.0) {
                Some(day) => day,
                None => return,
            },
        ];
        let day = match date("2026-03-03") {
            Some(day) => day,
            None => return,
        };
        let directions = day_directions(&history, day);
        let sleep = directions.vitals[0];
        assert_eq!(sleep.direction, None);
        assert_eq!(sleep.deviation, None);
        assert_eq!(sleep.deviation_text(), None);
        assert_eq!(
            sleep.status_text(),
            Some("not enough data for a comparison yet".to_string())
        );
        // The value is still shown: the raw value alone.
        assert_eq!(sleep.value_text(), Some("6 h".to_string()));
    }

    #[test]
    fn a_day_reads_the_same_way_every_time_it_is_read() {
        // Today and Body read the same values: the core owns them.
        let history = vec![
            match sleep_day("2026-02-26", 7.5) {
                Some(day) => day,
                None => return,
            },
            match sleep_day("2026-02-27", 7.0) {
                Some(day) => day,
                None => return,
            },
            match sleep_day("2026-02-28", 7.2) {
                Some(day) => day,
                None => return,
            },
            match sleep_day("2026-03-01", 6.0) {
                Some(day) => day,
                None => return,
            },
        ];
        let day = match date("2026-03-01") {
            Some(day) => day,
            None => return,
        };
        let first = day_directions(&history, day);
        let second = day_directions(&history, day);
        assert_eq!(first, second);
    }

    #[test]
    fn the_vitals_are_read_in_the_order_the_app_shows_them() {
        let day = match date("2026-03-01") {
            Some(day) => day,
            None => return,
        };
        let directions = day_directions(&[], day);
        let kinds: Vec<VitalKind> = directions.vitals.iter().map(|vital| vital.vital).collect();
        assert_eq!(
            kinds,
            vec![VitalKind::Sleep, VitalKind::Hrv, VitalKind::RestingHr]
        );
        // Nothing to compare with, so nothing is invented.
        for vital in &directions.vitals {
            assert_eq!(vital.direction, None);
            assert_eq!(vital.band, None);
        }
    }

    #[test]
    fn the_value_reads_as_the_athlete_reads_it() {
        let hrv = VitalDirection {
            vital: VitalKind::Hrv,
            value: Some(48.0),
            direction: Some(Direction::CouldBeBetter),
            deviation: Some(-14.0),
            band: None,
            travel: None,
        };
        assert_eq!(hrv.value_text(), Some("48 ms".to_string()));
        let resting = VitalDirection {
            vital: VitalKind::RestingHr,
            value: Some(63.0),
            direction: Some(Direction::AboutUsual),
            deviation: Some(5.0),
            band: None,
            travel: None,
        };
        assert_eq!(resting.value_text(), Some("63 bpm".to_string()));
        let sleep = VitalDirection {
            vital: VitalKind::Sleep,
            value: Some(6.0 * 3600.0 + 10.0 * 60.0),
            direction: Some(Direction::CouldBeBetter),
            deviation: Some(-(70.0 * 60.0)),
            band: None,
            travel: None,
        };
        assert_eq!(sleep.value_text(), Some("6 h 10 m".to_string()));
    }

    #[test]
    fn the_line_reads_direction_then_arithmetic_then_travel() {
        let hrv = VitalDirection {
            vital: VitalKind::Hrv,
            value: Some(48.0),
            direction: Some(Direction::CouldBeBetter),
            deviation: Some(-14.0),
            band: None,
            travel: Some(Travel::Down { nights: 3 }),
        };
        assert_eq!(
            hrv.line(),
            "could be better · 14 below your usual · 3rd night down"
        );
        assert_eq!(
            hrv.row_text(),
            "48 ms · could be better · 14 below your usual · 3rd night down"
        );
    }

    #[test]
    fn a_row_without_a_comparison_says_so_instead_of_saying_nothing() {
        let hrv = VitalDirection {
            vital: VitalKind::Hrv,
            value: Some(48.0),
            direction: None,
            deviation: None,
            band: None,
            travel: None,
        };
        assert_eq!(hrv.line(), "");
        assert_eq!(
            hrv.row_text(),
            "48 ms · not enough data for a comparison yet"
        );
    }

    #[test]
    fn an_hrv_reads_its_own_value() {
        let hrv = Hrv::from_ms(48.0);
        assert!(hrv.is_some());
        let resting = RestingHr::from_bpm(63.0);
        assert!(resting.is_some());
    }
}
