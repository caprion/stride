//! The heart-rate zone model and the time-in-zone computation.
//!
//! The zone model is a **dated fact about the athlete's configuration**, not
//! a series derived from a run. Garmin publishes the watch's own zone floors
//! at `/biometric-service/heartRateZones/v2`; Stride reads them once and
//! stores them with the day they were read. Recomputing bands per run from
//! *that run's observed maximum* would give two runs two different scales,
//! which is how an easy run and a threshold session come out looking alike.
//!
//! ## The boundary rule
//!
//! A reading **exactly on a floor belongs to the lower zone**: the floor is
//! the first value of the zone above it, and a reading that lands on it is
//! still in the zone below. A reading at or below the first floor is below
//! every zone, and a reading above the top floor is in the top zone. Garmin
//! reports the floors as the lower boundary of each zone; Stride assigns a
//! reading exactly on a boundary to the lower of the two adjacent zones, so
//! the interval of zone *n* is `(floor_n, floor_{n+1}]`.
//!
//! A run with **no stored model is not calibrated**: the result is "no
//! value", never band 1 and never zero.

use serde::{Deserialize, Serialize};

use crate::strings;
use crate::units;

/// The five inclusive-in-name floor values of a zone model, in beats per
/// minute.
///
/// The floors must be strictly increasing for the model to be usable.
///
/// The derives are the domain's, not the storage's: the load model carries a
/// zone model inside it, and a value that crosses the wire carries its
/// floors with it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoneFloors {
    /// The lower boundary Garmin reports for zone 1.
    pub zone1: f64,
    /// The lower boundary Garmin reports for zone 2.
    pub zone2: f64,
    /// The lower boundary Garmin reports for zone 3.
    pub zone3: f64,
    /// The lower boundary Garmin reports for zone 4.
    pub zone4: f64,
    /// The lower boundary Garmin reports for zone 5.
    pub zone5: f64,
}

impl ZoneFloors {
    /// Whether the floors are usable: finite and strictly increasing.
    pub fn is_usable(&self) -> bool {
        let floors = [self.zone1, self.zone2, self.zone3, self.zone4, self.zone5];
        floors.iter().all(|floor| floor.is_finite())
            && floors.windows(2).all(|pair| pair[0] < pair[1])
    }
}

/// One heart-rate reading for the zone computation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoneSample {
    /// Seconds from the start of the run.
    pub elapsed_s: f64,
    /// The heart rate, in beats per minute. `None` = no reading.
    pub heart_rate_bpm: Option<f64>,
}

/// The time a run spent in each of the five zones, in seconds.
///
/// Every field is a duration: a value can be `0.0` (a real measurement of
/// no time in that zone), but the whole result is `None` when the run has no
/// model — "no value" is a different thing from "zero".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeInZone {
    /// Time in zone 1, in seconds.
    pub zone1_s: f64,
    /// Time in zone 2, in seconds.
    pub zone2_s: f64,
    /// Time in zone 3, in seconds.
    pub zone3_s: f64,
    /// Time in zone 4, in seconds.
    pub zone4_s: f64,
    /// Time in zone 5, in seconds.
    pub zone5_s: f64,
    /// Time with no heart-rate reading, or at or below the first floor.
    pub outside_s: f64,
}

impl TimeInZone {
    /// The time in each zone, index 0 = zone 1 through index 4 = zone 5.
    pub fn per_zone(&self) -> [f64; 5] {
        [
            self.zone1_s,
            self.zone2_s,
            self.zone3_s,
            self.zone4_s,
            self.zone5_s,
        ]
    }
}

/// The time a run spent in each of the athlete's zones, from the **stored**
/// model.
///
/// The span attributed to a sample is the time until the next sample; the
/// last sample carries no span, so a lone sample yields no time anywhere.
/// A reading exactly on a floor belongs to the lower zone (see the module
/// docs); a reading with no heart rate, or at or below the first floor, is
/// counted outside the zones.
///
/// `None` when the model is unusable — the floors are not strictly
/// increasing, or a floor is not finite — which is how "no stored model" is
/// represented. The result is then "no value", never band 1 and never zero.
pub fn time_in_zone(floors: ZoneFloors, samples: &[ZoneSample]) -> Option<TimeInZone> {
    if !floors.is_usable() {
        return None;
    }

    let mut time = TimeInZone::default();
    for window in samples.windows(2) {
        let span = window[1].elapsed_s - window[0].elapsed_s;
        if !span.is_finite() || span <= 0.0 {
            continue;
        }
        match window[0].heart_rate_bpm {
            None => time.outside_s += span,
            Some(rate) => match zone_index(rate, floors) {
                None => time.outside_s += span,
                Some(0) => time.zone1_s += span,
                Some(1) => time.zone2_s += span,
                Some(2) => time.zone3_s += span,
                Some(3) => time.zone4_s += span,
                _ => time.zone5_s += span,
            },
        }
    }
    Some(time)
}

/// The model's own source line: the method it was set with, the maximum
/// heart rate it was built on, and the day it was read.
///
/// A number that cannot say where its input came from and when it was read
/// is not inspectable, and the bands are the product's most interpretable
/// intensity statement: this is the line that names the model they were
/// measured against.
///
/// The day is **rendered by the caller**. The api renders it in the api's
/// own form ("10 May 2026") and passes the string in, because the core does
/// not format a date for a screen — its own canonical form stays inside it,
/// and a third form would be a format nobody owns.
///
/// A model with no maximum heart rate — Garmin reported none — is described
/// without one: the method and the day are still the model's own.
pub fn model_source(method: &str, max_heart_rate: Option<f64>, read_on: &str) -> Option<String> {
    match units::render_heart_rate(max_heart_rate) {
        Some(max_hr) => strings::render(
            "zones.model.source",
            &[("method", method), ("max_hr", &max_hr), ("date", read_on)],
        ),
        None => strings::render(
            "zones.model.source.no_max",
            &[("method", method), ("date", read_on)],
        ),
    }
}

/// The zone a **sample** sits in, as an index from 0 (zone 1) to 4 (zone 5).
///
/// The same classifier the time-in-zone computation uses, so the zone of a
/// single reading and the time a run spent in that zone can never disagree:
/// one rule, in one place.
///
/// `None` when the reading is **outside the zones** — at or below the first
/// floor — and `None` when there is no reading at all. The core counts both
/// as outside, and so does this: a reading that is not there is not in a
/// zone, and it is never band 1.
///
/// The model is the athlete's **stored** one. A model whose floors are not
/// strictly increasing cannot be used, and every reading is outside it.
pub fn zone_of(heart_rate_bpm: Option<f64>, floors: ZoneFloors) -> Option<usize> {
    if !floors.is_usable() {
        return None;
    }
    zone_index(heart_rate_bpm?, floors)
}

/// The edges of the five zones: the floor each zone starts at, and the
/// floor it ends at.
///
/// The top zone ends at the maximum heart rate the model was built on,
/// and has **no upper edge** when the watch reported none.
pub fn zone_edges(floors: &ZoneFloors, max_heart_rate: Option<f64>) -> [(f64, Option<f64>); 5] {
    let all = [
        floors.zone1,
        floors.zone2,
        floors.zone3,
        floors.zone4,
        floors.zone5,
    ];
    let mut edges = [(0.0, None); 5];
    for (index, from) in all.iter().enumerate() {
        let to = all.get(index + 1).copied().or(max_heart_rate);
        edges[index] = (*from, to);
    }
    edges
}

/// The zone a heart rate falls into, as an index from 0 (zone 1) to 4
/// (zone 5), or `None` when it is at or below the first floor.
///
/// A reading exactly on a floor belongs to the lower zone, so the check for
/// each zone is `rate <= floor` — the reading at `floor_n` is in zone *n-1*,
/// and only a reading strictly above `floor_n` reaches zone *n*.
fn zone_index(rate: f64, floors: ZoneFloors) -> Option<usize> {
    if !rate.is_finite() || rate <= floors.zone1 {
        return None;
    }
    if rate <= floors.zone2 {
        return Some(0);
    }
    if rate <= floors.zone3 {
        return Some(1);
    }
    if rate <= floors.zone4 {
        return Some(2);
    }
    if rate <= floors.zone5 {
        return Some(3);
    }
    Some(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floors() -> ZoneFloors {
        ZoneFloors {
            zone1: 100.0,
            zone2: 120.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        }
    }

    /// Two samples one second apart: the first carries the reading, and its
    /// span is the one second until the second sample.
    fn span(heart_rate_bpm: Option<f64>) -> [ZoneSample; 2] {
        [
            ZoneSample {
                elapsed_s: 0.0,
                heart_rate_bpm,
            },
            ZoneSample {
                elapsed_s: 1.0,
                heart_rate_bpm: None,
            },
        ]
    }

    /// The time-in-zone of a usable model. An unusable model here yields the
    /// default (all zero), which the value assertions below will reject.
    fn compute(floors: ZoneFloors, samples: &[ZoneSample]) -> TimeInZone {
        time_in_zone(floors, samples).unwrap_or_default()
    }

    #[test]
    fn a_reading_exactly_on_a_floor_belongs_to_the_lower_zone() {
        // zone2Floor (120) is in zone 1, not zone 2.
        let time = compute(floors(), &span(Some(120.0)));
        assert_eq!(time.per_zone(), [1.0, 0.0, 0.0, 0.0, 0.0]);

        // zone3Floor (140) is in zone 2.
        let time = compute(floors(), &span(Some(140.0)));
        assert_eq!(time.per_zone(), [0.0, 1.0, 0.0, 0.0, 0.0]);

        // zone5Floor (175) is in zone 4.
        let time = compute(floors(), &span(Some(175.0)));
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn a_reading_at_the_first_floor_is_below_every_zone() {
        let time = compute(floors(), &span(Some(100.0)));
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(time.outside_s, 1.0);
    }

    #[test]
    fn a_reading_below_the_first_floor_is_outside() {
        let time = compute(floors(), &span(Some(90.0)));
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(time.outside_s, 1.0);
    }

    #[test]
    fn a_reading_above_the_top_floor_is_in_zone_five() {
        let time = compute(floors(), &span(Some(190.0)));
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn interior_readings_land_in_their_zone() {
        let time = compute(floors(), &span(Some(110.0)));
        assert_eq!(time.per_zone(), [1.0, 0.0, 0.0, 0.0, 0.0]);

        let time = compute(floors(), &span(Some(150.0)));
        assert_eq!(time.per_zone(), [0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn a_reading_with_no_heart_rate_is_outside() {
        let time = compute(floors(), &span(None));
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(time.outside_s, 1.0);
    }

    #[test]
    fn a_run_with_no_model_is_not_calibrated() {
        let unusable = ZoneFloors {
            zone1: 100.0,
            zone2: 90.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        };
        assert!(time_in_zone(unusable, &span(Some(110.0))).is_none());

        let non_finite = ZoneFloors {
            zone1: f64::NAN,
            zone2: 120.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        };
        assert!(time_in_zone(non_finite, &span(Some(110.0))).is_none());
    }

    #[test]
    fn the_source_line_names_the_model_and_the_day_it_was_read() {
        // The day is the caller's: the api renders it in its own form and
        // passes the string in, so the core never formats a date for a
        // screen.
        let line = model_source("HR_MAX", Some(178.0), "10 May 2026");
        assert_eq!(
            line,
            Some(
                "your zone model · HR_MAX · max heart rate 178 bpm · read on 10 May 2026"
                    .to_string()
            )
        );
    }

    #[test]
    fn a_model_with_no_maximum_heart_rate_is_described_without_one() {
        assert_eq!(
            model_source("HR_MAX", None, "10 May 2026"),
            Some("your zone model · HR_MAX · read on 10 May 2026".to_string())
        );
    }

    #[test]
    fn the_zone_of_a_sample_is_the_zone_the_time_is_counted_in() {
        // The same rule the time-in-zone computation uses.
        assert_eq!(zone_of(Some(110.0), floors()), Some(0));
        assert_eq!(zone_of(Some(150.0), floors()), Some(2));
        assert_eq!(zone_of(Some(190.0), floors()), Some(4));
        // A reading exactly on a floor belongs to the lower zone.
        assert_eq!(zone_of(Some(120.0), floors()), Some(0));
        // At or below the first floor is outside the zones.
        assert_eq!(zone_of(Some(100.0), floors()), None);
        assert_eq!(zone_of(Some(90.0), floors()), None);
        // A reading that is not there is not in a zone either.
        assert_eq!(zone_of(None, floors()), None);
    }

    #[test]
    fn a_sample_of_an_unusable_model_is_outside_every_zone() {
        let unusable = ZoneFloors {
            zone1: 100.0,
            zone2: 90.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        };
        assert_eq!(zone_of(Some(150.0), unusable), None);
    }

    #[test]
    fn a_lone_sample_carries_no_time() {
        let samples = [ZoneSample {
            elapsed_s: 0.0,
            heart_rate_bpm: Some(150.0),
        }];
        let time = compute(floors(), &samples);
        assert_eq!(time.per_zone(), [0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(time.outside_s, 0.0);
    }
}
