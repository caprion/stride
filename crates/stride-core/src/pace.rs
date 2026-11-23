//! The pace bands: where the moving time went, by the pace it was held at.
//!
//! A running log knows two things about a run: how far it went and how long
//! it moved. The third thing — **how fast it was when it was running** — is a
//! distribution, not a number: an hour at 5:00 /km and an hour at 6:30 /km
//! are the same hour of running and a very different hour of training. The
//! bands are that distribution, bucketed by the pace the time was held at.
//!
//! ## How a span is measured
//!
//! The bands are measured over the run's own **moving samples**, the same
//! filter the time-in-zone computation has always used: the standing samples
//! are left out, so the numbers stay comparable with what the zones route
//! answered before.
//!
//! A sample carries no time of its own: **the time between two samples is
//! the time of the first**, and the pace of that time is the distance covered
//! over it, divided by the time it took. The last sample of a run carries no
//! span, so it is in no band — the same rule the time-in-zone computation
//! uses, and for the same reason: a span that has not happened yet is not a
//! measurement.
//!
//! ## The grid
//!
//! The bands sit on a fixed grid: **thirty seconds a kilometre**, the width a
//! person reads a pace in. A band's lower edge is always a whole number of
//! bands from zero, so the same pace is always in the same band.
//!
//! The boundary rule is the zone model's rule, word for word: **a pace
//! exactly on an edge belongs to the band below it**, so the interval of a
//! band is `[edge, edge + width)`.
//!
//! ## The bands the answer holds
//!
//! The answer holds **the bands the window's own time fills**, fastest
//! first. It does not cut the distribution at a fixed pace and gather the
//! ends into two rows: a range cut at somebody else's paces is a claim
//! about the athlete, and it hides most of the time in one row. Every band
//! the window fills is here, and the times in them add up to the window's
//! moving time.
//!
//! ## The window
//!
//! **The bands are a property of the window.** The same bands over twelve
//! weeks and over a year are two different answers, and the caller says which
//! window it is: the runs outside the period are not counted, and the answer
//! names the period it covers.
//!
//! ## No value
//!
//! A window with no moving samples has **no bands**: not a band of zero, and
//! not an empty distribution that reads like one. The answer is "no value",
//! and the caller says why in the core's own words.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::eddington::Period;
use crate::units;

/// The width of one pace band, in seconds per kilometre.
///
/// Thirty seconds a kilometre is the width a person reads a pace in, and the
/// grid is the same everywhere: a band's lower edge is always a whole number
/// of bands from zero.
pub const BAND_WIDTH_S_PER_KM: f64 = 30.0;

/// One moving sample of a run: where the run was, in time and in distance.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PacePoint {
    /// Seconds since the start of the run.
    pub elapsed_s: f64,
    /// The run's cumulative distance at the sample, in metres.
    pub distance_m: f64,
    /// Whether the athlete was **moving** when this point was taken.
    ///
    /// A point owns the span until the next one, so a point taken while
    /// the athlete stood still owns the time they stood still. Dropping
    /// the stopped points from the list does not drop the time: it
    /// merges the stopped span into the neighbouring running one, and the
    /// run then claims more time at a pace than it spent moving.
    pub moving: bool,
}

impl PacePoint {
    /// One moving sample.
    pub fn new(elapsed_s: f64, distance_m: f64) -> Self {
        Self {
            elapsed_s,
            distance_m,
            moving: true,
        }
    }

    /// A point of a stretch the athlete was not moving through.
    pub fn stopped(elapsed_s: f64, distance_m: f64) -> Self {
        Self {
            elapsed_s,
            distance_m,
            moving: false,
        }
    }
}

/// One run, as the pace bands read it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaceRun {
    /// The athlete-local day the run belongs to.
    pub date: Date,
    /// The run's moving samples, in the order the file carries them.
    pub samples: Vec<PacePoint>,
}

impl PaceRun {
    /// One run, with its moving samples.
    pub fn new(date: Date, samples: Vec<PacePoint>) -> Self {
        Self { date, samples }
    }
}

/// One band of the distribution.
///
/// The edges are in seconds per kilometre: the band's paces are the ones
/// from `from_s_per_km` up to `to_s_per_km`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaceBand {
    /// The lower edge of the band, in seconds per kilometre.
    pub from_s_per_km: f64,
    /// The upper edge of the band, in seconds per kilometre.
    pub to_s_per_km: f64,
    /// The time held at the band's paces, in seconds.
    pub time_s: f64,
    /// The distance covered at the band's paces, in metres.
    pub distance_m: f64,
    /// The band's share of the window's time, in per cent.
    pub share_percent: f64,
}

impl PaceBand {
    /// The band's label, in the core's own words: "4:30–5:00 /km".
    pub fn label(&self) -> Option<String> {
        units::render_pace_range(self.from_s_per_km, self.to_s_per_km)
    }
}

/// The distribution of a window's moving time by pace.
///
/// The bands are the answer; the totals are the window they were measured
/// over, so a reader can see what the distribution is a distribution *of*.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaceBands {
    /// The bands the window's time fills, fastest first.
    pub bands: Vec<PaceBand>,
    /// The window's moving time, in seconds: the sum of the bands' times.
    pub total_s: f64,
    /// The distance covered in the window, in metres.
    pub total_m: f64,
}

impl PaceBands {
    /// A window with nothing in it.
    pub fn empty() -> Self {
        Self {
            bands: Vec::new(),
            total_s: 0.0,
            total_m: 0.0,
        }
    }

    /// Whether the window holds no time at all.
    ///
    /// **A window with no moving samples has no bands**: it is "no value",
    /// never a distribution of zeros.
    pub fn is_empty(&self) -> bool {
        self.total_s <= 0.0
    }
}

/// The distribution of a window's moving time by the pace it was held at.
///
/// The runs outside the period are not counted: the bands are a property of
/// the window, and the caller says which window.
///
/// Every span of every run in the period is measured — the time between two
/// moving samples, at the pace the distance covered over it was held — and
/// the span's time and distance go into the band that pace falls in.
pub fn pace_bands(runs: &[PaceRun], period: Period) -> PaceBands {
    // The grid: the band a pace falls in, and the time and distance in it.
    let mut grid: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
    let mut total_s = 0.0;
    let mut total_m = 0.0;

    for run in runs {
        if !period.contains(run.date) {
            continue;
        }
        for span in run.samples.windows(2) {
            // **A stopped stretch is not time at a pace.** The point owns
            // the span until the next one, so a stretch the athlete stood
            // through would otherwise be charged to the pace of the point
            // before it — and the run would claim more time at a pace
            // than it spent moving.
            if !span[0].moving {
                continue;
            }
            let Some((time_s, distance_m)) = span_of(span[0], span[1]) else {
                continue;
            };
            let Some(index) = band_index(time_s, distance_m) else {
                continue;
            };
            let entry = grid.entry(index).or_insert((0.0, 0.0));
            entry.0 += time_s;
            entry.1 += distance_m;
            total_s += time_s;
            total_m += distance_m;
        }
    }

    if grid.is_empty() {
        return PaceBands::empty();
    }

    // The bands the window's time fills, fastest first.
    let mut bands: Vec<PaceBand> = Vec::with_capacity(grid.len());
    for (index, (time_s, distance_m)) in grid {
        bands.push(band(index, time_s, distance_m, total_s));
    }

    PaceBands {
        bands,
        total_s,
        total_m,
    }
}

/// The span between two samples: the time it took and the distance covered.
///
/// `None` when the span is not a measurement: no time, no distance, or a
/// distance that went backwards. A run's own samples never do that, and a
/// file that does is not read as a pace.
fn span_of(from: PacePoint, to: PacePoint) -> Option<(f64, f64)> {
    let time_s = to.elapsed_s - from.elapsed_s;
    let distance_m = to.distance_m - from.distance_m;
    if !time_s.is_finite() || time_s <= 0.0 {
        return None;
    }
    if !distance_m.is_finite() || distance_m <= 0.0 {
        return None;
    }
    Some((time_s, distance_m))
}

/// The band a span's pace falls in, as the band's index on the grid.
///
/// The pace is the distance covered over the span divided by the time it
/// took. A pace exactly on an edge belongs to the band **below** it, so the
/// interval of a band is `[edge, edge + width)`.
fn band_index(time_s: f64, distance_m: f64) -> Option<i64> {
    let pace_s_per_km = time_s / (distance_m / 1000.0);
    if !pace_s_per_km.is_finite() || pace_s_per_km <= 0.0 {
        return None;
    }
    Some((pace_s_per_km / BAND_WIDTH_S_PER_KM).floor() as i64)
}

/// The edge of a band, in seconds per kilometre.
fn edge(index: i64) -> f64 {
    index as f64 * BAND_WIDTH_S_PER_KM
}

/// One band, with its share of the window's time.
fn band(index: i64, time_s: f64, distance_m: f64, total_s: f64) -> PaceBand {
    let share_percent = if total_s > 0.0 {
        time_s / total_s * 100.0
    } else {
        0.0
    };
    PaceBand {
        from_s_per_km: edge(index),
        to_s_per_km: edge(index + 1),
        time_s,
        distance_m,
        share_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A day, when the text is a day.
    fn day(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    /// A period, when both of its ends are days.
    fn period(from: &str, to: &str) -> Option<Period> {
        Some(Period::new(day(from)?, day(to)?))
    }

    /// A run held at one pace: `kilometres` in `seconds`, in samples half a
    /// kilometre apart.
    fn run_at(text: &str, seconds_per_km: f64, kilometres: f64) -> Option<PaceRun> {
        let date = day(text)?;
        let mut samples = Vec::new();
        let steps = (kilometres * 2.0).round() as i64;
        for step in 0..=steps {
            let kilometres = step as f64 * 0.5;
            samples.push(PacePoint::new(
                kilometres * seconds_per_km,
                kilometres * 1000.0,
            ));
        }
        Some(PaceRun::new(date, samples))
    }

    #[test]
    fn a_flat_series_holds_one_band() {
        let (Some(run), Some(period)) = (
            run_at("2026-05-04", 300.0, 5.0),
            period("2026-05-01", "2026-05-10"),
        ) else {
            return;
        };

        let bands = pace_bands(&[run], period);

        assert_eq!(bands.bands.len(), 1);
        assert_eq!(bands.bands[0].from_s_per_km, 300.0);
        assert_eq!(bands.bands[0].to_s_per_km, 330.0);
        assert_eq!(bands.bands[0].time_s, 1_500.0);
        assert_eq!(bands.total_s, 1_500.0);
        assert_eq!(bands.total_m, 5_000.0);
    }

    #[test]
    fn a_rising_series_fills_the_bands_it_crosses() {
        let (Some(date), Some(period)) = (day("2026-05-04"), period("2026-05-01", "2026-05-10"))
        else {
            return;
        };
        // The first kilometre at 4:00, the second at 6:00: two bands.
        let samples = vec![
            PacePoint::new(0.0, 0.0),
            PacePoint::new(240.0, 1_000.0),
            PacePoint::new(600.0, 2_000.0),
        ];

        let bands = pace_bands(&[PaceRun::new(date, samples)], period);

        assert_eq!(bands.bands.len(), 2);
        assert_eq!(bands.bands[0].from_s_per_km, 240.0);
        assert_eq!(bands.bands[0].time_s, 240.0);
        assert_eq!(bands.bands[1].from_s_per_km, 360.0);
        assert_eq!(bands.bands[1].time_s, 360.0);
        assert_eq!(bands.total_s, 600.0);
    }

    #[test]
    fn a_pace_exactly_on_an_edge_belongs_to_the_band_below_it() {
        // 270 s/km is the edge of the 4:30 band: it is in [270, 300).
        let (Some(run), Some(period)) = (
            run_at("2026-05-04", 270.0, 2.0),
            period("2026-05-01", "2026-05-10"),
        ) else {
            return;
        };

        let bands = pace_bands(&[run], period);

        assert_eq!(bands.bands.len(), 1);
        assert_eq!(bands.bands[0].from_s_per_km, 270.0);
        assert_eq!(bands.bands[0].to_s_per_km, 300.0);
    }

    #[test]
    fn a_gap_in_the_samples_leaves_the_rest_of_the_run_counted() {
        // Two runs, one of them with no samples at all: the run with the
        // samples is still counted, and the one without it is not a band of
        // zero.
        let (Some(date), Some(run), Some(period)) = (
            day("2026-05-05"),
            run_at("2026-05-04", 300.0, 5.0),
            period("2026-05-01", "2026-05-10"),
        ) else {
            return;
        };
        let empty = PaceRun::new(date, Vec::new());

        let bands = pace_bands(&[empty, run], period);

        assert_eq!(bands.bands.len(), 1);
        assert_eq!(bands.total_s, 1_500.0);
    }

    #[test]
    fn a_run_outside_the_window_is_not_counted() {
        let (Some(inside), Some(outside), Some(period)) = (
            run_at("2026-05-04", 300.0, 5.0),
            run_at("2026-04-01", 300.0, 5.0),
            period("2026-05-01", "2026-05-10"),
        ) else {
            return;
        };

        let bands = pace_bands(&[outside, inside], period);

        assert_eq!(bands.total_s, 1_500.0);
        assert_eq!(bands.total_m, 5_000.0);
    }

    #[test]
    fn a_single_sample_carries_no_time() {
        let (Some(date), Some(period)) = (day("2026-05-04"), period("2026-05-01", "2026-05-10"))
        else {
            return;
        };
        let run = PaceRun::new(date, vec![PacePoint::new(0.0, 0.0)]);

        let bands = pace_bands(&[run], period);

        assert!(bands.is_empty());
        assert!(bands.bands.is_empty());
    }

    #[test]
    fn an_empty_window_has_no_bands() {
        let Some(period) = period("2026-05-01", "2026-05-10") else {
            return;
        };

        let bands = pace_bands(&[], period);

        assert!(bands.is_empty());
        assert_eq!(bands.total_s, 0.0);
    }

    #[test]
    fn the_bands_are_named_in_the_core_s_words() {
        let (Some(run), Some(period)) = (
            run_at("2026-05-04", 300.0, 5.0),
            period("2026-05-01", "2026-05-10"),
        ) else {
            return;
        };

        let bands = pace_bands(&[run], period);

        assert_eq!(bands.bands[0].label(), Some("5:00–5:30 /km".to_string()));
    }
}
