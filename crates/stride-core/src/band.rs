//! "Usual": the median of the last 30 nights, and the band around it.
//!
//! Contract §4, verbatim: *Usual* = the median of the last **30 nights** (the
//! day itself excluded) · **Band** = median ± **1.5 × MAD** · inside the band
//! is "about usual".
//!
//! The band is drawn on every chart, so the athlete can see where the line is
//! and disagree with it. Which side of the band is the better side is a fact
//! about the measurement — more sleep, higher HRV, lower resting heart rate —
//! and it lives in [`crate::types::VitalKind::better_side`], not here.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::types::{VitalKind, WellnessDay};

/// How many nights the band looks back at.
pub const BAND_WINDOW_NIGHTS: i64 = 30;

/// How many median absolute deviations wide the band is.
pub const BAND_SCALE: f64 = 1.5;

/// The fewest nights a band can be drawn from.
///
/// The contract fixes the window (30 nights) and the scale (1.5 × MAD) but not
/// the smallest sample the band may be drawn from. Three is the house
/// minimum — the same floor the pattern engine uses — because a median of one
/// night is that night, and a band around it has no width at all.
pub const MIN_BAND_SAMPLES: usize = 3;

/// The athlete's usual band for one vital.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Band {
    usual: f64,
    mad: f64,
    lower: f64,
    upper: f64,
    samples: usize,
}

impl Band {
    /// The usual value: the median of the nights in the window.
    pub fn usual(&self) -> f64 {
        self.usual
    }

    /// The median absolute deviation of the nights in the window.
    pub fn mad(&self) -> f64 {
        self.mad
    }

    /// The lower edge of the band.
    pub fn lower(&self) -> f64 {
        self.lower
    }

    /// The upper edge of the band.
    pub fn upper(&self) -> f64 {
        self.upper
    }

    /// How many nights the band was drawn from.
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// How wide the band is, on either side of the usual.
    pub fn scale(&self) -> f64 {
        BAND_SCALE * self.mad
    }

    /// Whether a value sits inside the band.
    ///
    /// The edges are inside: a value exactly on the edge is still "about
    /// usual".
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }
}

/// The median of a sample, or `None` for an empty one.
///
/// An even-sized sample averages the two middle values, like
/// `statistics.median`.
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[middle])
    } else {
        Some((sorted[middle - 1] + sorted[middle]) / 2.0)
    }
}

/// The median absolute deviation of a sample around its median.
pub fn median_absolute_deviation(values: &[f64], usual: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let deviations: Vec<f64> = values.iter().map(|value| (value - usual).abs()).collect();
    median(&deviations).unwrap_or(0.0)
}

/// The band for a sample of values, in the vital's own scale.
///
/// `None` when the sample is too small to draw a band from.
pub fn band(values: &[f64]) -> Option<Band> {
    if values.len() < MIN_BAND_SAMPLES {
        return None;
    }
    let usual = median(values)?;
    let mad = median_absolute_deviation(values, usual);
    let scale = BAND_SCALE * mad;
    Some(Band {
        usual,
        mad,
        lower: usual - scale,
        upper: usual + scale,
        samples: values.len(),
    })
}

/// The values of one vital over the trailing window that ends the day before
/// `day`.
///
/// The window is the thirty calendar days before the day itself: the day's
/// own measurement is excluded, and so is anything older than the window.
pub fn window_values(history: &[WellnessDay], day: Date, vital: VitalKind) -> Vec<f64> {
    let start = day.add_days(-BAND_WINDOW_NIGHTS);
    let mut days: Vec<&WellnessDay> = history
        .iter()
        .filter(|entry| entry.date >= start && entry.date < day)
        .collect();
    days.sort_by_key(|entry| entry.date);
    days.iter()
        .filter_map(|entry| vital.value(&entry.vitals))
        .collect()
}

/// The band for one vital on one day, drawn from the nights before it.
pub fn band_for(history: &[WellnessDay], day: Date, vital: VitalKind) -> Option<Band> {
    band(&window_values(history, day, vital))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Hrv, Vitals};

    fn day(date: &str, hrv: Option<f64>) -> Option<WellnessDay> {
        let date = Date::parse(date)?;
        Some(WellnessDay::new(
            date,
            Vitals {
                hrv: hrv.and_then(Hrv::from_ms),
                ..Vitals::default()
            },
        ))
    }

    #[test]
    fn the_median_of_an_even_sample_is_the_middle_pair() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[3.0]), Some(3.0));
        assert_eq!(median(&[1.0, 3.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn the_band_is_the_median_plus_and_minus_one_and_a_half_mads() {
        // A tidy sample: median 10, deviations 0, 0, 2, 4 → MAD 1 → band 8.5–11.5.
        let values = [10.0, 10.0, 12.0, 6.0];
        let band = band(&values);
        assert!(band.is_some());
        let band = match band {
            Some(band) => band,
            None => return,
        };
        assert_eq!(band.usual(), 10.0);
        assert_eq!(band.mad(), 1.0);
        assert_eq!(band.lower(), 8.5);
        assert_eq!(band.upper(), 11.5);
        assert_eq!(band.samples(), 4);
        assert!(band.contains(10.0));
        assert!(band.contains(8.5));
        assert!(band.contains(11.5));
        assert!(!band.contains(8.4));
        assert!(!band.contains(11.6));
    }

    #[test]
    fn a_band_needs_three_nights_to_be_drawn_at_all() {
        assert!(band(&[]).is_none());
        assert!(band(&[10.0]).is_none());
        assert!(band(&[10.0, 11.0]).is_none());
        assert!(band(&[10.0, 11.0, 12.0]).is_some());
    }

    #[test]
    fn the_window_is_the_thirty_nights_before_the_day_and_excludes_the_day() {
        let history = vec![
            match day("2026-01-01", Some(1.0)) {
                Some(entry) => entry,
                None => return,
            },
            match day("2026-02-01", Some(50.0)) {
                Some(entry) => entry,
                None => return,
            },
            match day("2026-02-02", Some(51.0)) {
                Some(entry) => entry,
                None => return,
            },
            match day("2026-02-03", Some(52.0)) {
                Some(entry) => entry,
                None => return,
            },
            match day("2026-02-20", Some(53.0)) {
                Some(entry) => entry,
                None => return,
            },
            match day("2026-02-25", Some(999.0)) {
                Some(entry) => entry,
                None => return,
            },
        ];
        let today = match Date::parse("2026-02-25") {
            Some(date) => date,
            None => return,
        };
        let values = window_values(&history, today, VitalKind::Hrv);
        // 2026-01-01 is older than the window; 2026-02-25 is the day itself.
        assert_eq!(values, vec![50.0, 51.0, 52.0, 53.0]);
    }
}
