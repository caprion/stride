//! The running fingerprint: the mechanics of one run, derived from its laps.
//!
//! The reference is `backend/app/services/fingerprint.py`. Two things about it
//! are easy to get wrong and are therefore worth stating plainly:
//!
//! * **The efficiency factor is speed-based, not pace-based.**
//!   `EF = speed / (heart rate as a fraction of the maximum)`. Using the pace
//!   ratio instead would make a *slower* run look more efficient, which
//!   inverts every downstream reading.
//! * **The zones come from the athlete's profile, not from a formula.** The
//!   formula (`reference × {0, .6, .7, .8, .9}`) is only the fallback used when
//!   the athlete has no stored zones.

use serde::{Deserialize, Serialize};

use crate::model::{Activity, Fingerprint, Lap, Zone};
use crate::numeric::{change, clamp01, cv, is_number, is_positive, mean};
use crate::ordered::OrderedMap;

/// A cadence below this, together with a slow pace, means the lap was walked.
const WALKING_CADENCE: f64 = 125.0;
/// The speed below which a slow lap counts as walked.
const WALKING_SPEED_MPS: f64 = 2.2;
/// The default maximum heart rate when the athlete has no recorded maximum.
pub const DEFAULT_MAX_HR_REFERENCE: f64 = 190.0;
/// The fraction of the maximum heart rate that opens each zone.
const ZONE_BOUNDS: [f64; 5] = [0.0, 0.6, 0.7, 0.8, 0.9];
/// The names of the five heart-rate zones.
pub const ZONE_NAMES: [&str; 5] = ["Recovery", "Aerobic", "Tempo", "Threshold", "Anaerobic"];

/// A zone as the athlete's Garmin profile stores it.
///
/// The reference reads `low_bpm` / `high_bpm` and falls back to Garmin's own
/// `zoneLowBoundary` / `zoneHighBoundary` field names.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StoredZone {
    /// The zone's number.
    #[serde(default, alias = "zoneNumber")]
    pub zone_no: Option<i64>,
    /// The zone's name.
    #[serde(default)]
    pub name: Option<String>,
    /// The zone's lower bound, in beats per minute.
    #[serde(default, alias = "zoneLowBoundary")]
    pub low_bpm: Option<f64>,
    /// The zone's upper bound, in beats per minute.
    #[serde(default, alias = "zoneHighBoundary")]
    pub high_bpm: Option<f64>,
}

/// The inputs of [`fingerprint`].
#[derive(Clone, Debug)]
pub struct FingerprintInput<'a> {
    /// The athlete's observed maximum heart rate.
    ///
    /// This is the maximum over the athlete's activities — 190 when there is
    /// none — and it moves as the athlete's history grows.
    pub max_hr_reference: f64,
    /// The athlete's heart-rate zone model.
    pub zone_model: &'a [Zone],
    /// The activity to fingerprint.
    pub activity: &'a Activity,
}

/// The athlete's observed maximum heart rate.
///
/// The reference reads it from the athlete's activities, and hides the demo
/// ones only when the demo is not visible. It falls back to 190 when there is
/// nothing to read.
///
/// The rows are filtered by `athlete_id` exactly as the reference's query
/// filters them — `athlete_id IS NULL` matches the athlete-less rows.
pub fn max_hr_reference(
    activities: &[Activity],
    athlete_id: Option<i64>,
    demo_visible: bool,
) -> f64 {
    let mut maximum: Option<f64> = None;
    for activity in activities {
        if activity.athlete_id != athlete_id {
            continue;
        }
        if !demo_visible && activity.source == "demo" {
            continue;
        }
        if let Some(value) = activity.max_hr {
            if is_positive(value) && maximum.map_or(true, |current| value > current) {
                maximum = Some(value);
            }
        }
    }
    maximum.unwrap_or(DEFAULT_MAX_HR_REFERENCE)
}

/// The heart-rate zone model.
///
/// The athlete's stored zones win when there are exactly five usable ones;
/// otherwise the model is derived from the maximum heart rate.
pub fn zone_model(reference: f64, stored: Option<&[StoredZone]>) -> Vec<Zone> {
    if let Some(stored) = stored {
        let mut zones: Vec<Zone> = Vec::new();
        for (index, row) in stored.iter().enumerate() {
            let low = row.low_bpm;
            let high = row.high_bpm;
            let usable = low.is_some_and(is_number) && high.map_or(true, is_positive);
            if !usable {
                continue;
            }
            zones.push(Zone {
                zone_no: row.zone_no.unwrap_or(index as i64 + 1),
                name: row
                    .name
                    .clone()
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| format!("Zone {}", index + 1)),
                low_bpm: low.unwrap_or(0.0),
                high_bpm: high,
            });
        }
        if zones.len() == 5 {
            zones.sort_by(|left, right| {
                left.low_bpm
                    .partial_cmp(&right.low_bpm)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return zones;
        }
    }
    default_zone_model(reference)
}

/// The zone model derived from the maximum heart rate.
pub fn default_zone_model(reference: f64) -> Vec<Zone> {
    (0..5)
        .map(|index| Zone {
            zone_no: index as i64 + 1,
            name: ZONE_NAMES[index].to_string(),
            low_bpm: reference * ZONE_BOUNDS[index],
            high_bpm: if index < 4 {
                Some(reference * ZONE_BOUNDS[index + 1])
            } else {
                None
            },
        })
        .collect()
}

/// The athlete's zones.
///
/// The reference reads them from the athlete's cached Garmin profile and
/// never makes a network call on a read.
pub fn athlete_zones(reference: f64, stored: Option<&[StoredZone]>) -> Vec<Zone> {
    zone_model(reference, stored)
}

/// Derive the fingerprint of one run.
pub fn fingerprint(input: &FingerprintInput<'_>) -> Fingerprint {
    derive(input)
}

/// Drop the laps that were walked rather than run.
///
/// A lap is walked when its cadence is well below running *and* its pace is
/// slow; both conditions must hold, so a short-strided runner is not mistaken
/// for a walker. If every lap looks walked, the classification is not
/// trustworthy and all the laps are kept.
fn running_only(laps: &[Lap]) -> Vec<Lap> {
    if laps.is_empty() {
        return Vec::new();
    }
    let running: Vec<Lap> = laps
        .iter()
        .filter(|lap| {
            let walked = match lap.avg_cadence {
                Some(cadence) if is_positive(cadence) => {
                    cadence < WALKING_CADENCE
                        && match lap.avg_speed_ms {
                            Some(speed) if is_positive(speed) => speed < WALKING_SPEED_MPS,
                            _ => true,
                        }
                }
                _ => false,
            };
            !walked
        })
        .cloned()
        .collect();
    if running.is_empty() {
        laps.to_vec()
    } else {
        running
    }
}

/// The lap's pace, in seconds per kilometre.
///
/// The reference uses the lap's *running* duration over the lap's *total*
/// distance, so a lap that was partly walked gets a pace that is too slow
/// (contract §10.5).
fn lap_pace(lap: &Lap) -> Option<f64> {
    if let (Some(duration), Some(distance)) = (lap.duration_s, lap.distance_m) {
        if is_positive(duration) && is_positive(distance) {
            return Some(duration * 1000.0 / distance);
        }
    }
    match lap.avg_speed_ms {
        Some(speed) if is_positive(speed) => Some(1000.0 / speed),
        _ => None,
    }
}

/// The laps' values of one field, keeping only the ones above zero.
fn positive_values(laps: &[Lap], pick: fn(&Lap) -> Option<f64>) -> Vec<f64> {
    laps.iter()
        .filter_map(pick)
        .filter(|value| is_positive(*value))
        .collect()
}

/// The laps' paces.
fn paces(laps: &[Lap]) -> Vec<f64> {
    laps.iter().filter_map(lap_pace).collect()
}

/// The Minetti efficiency factor of a group of laps.
///
/// `EF = speed / (heart rate as a fraction of the maximum)`. Speed, not pace.
fn efficiency(group: &[Lap], reference: f64) -> Option<f64> {
    if reference == 0.0 {
        return None;
    }
    let paired: Vec<&Lap> = group
        .iter()
        .filter(|lap| lap_pace(lap).is_some() && lap.avg_hr.is_some_and(is_positive))
        .collect();
    let pace = mean(
        &paired
            .iter()
            .filter_map(|lap| lap_pace(lap))
            .collect::<Vec<_>>(),
    )?;
    let heart_rate = mean(
        &paired
            .iter()
            .filter_map(|lap| lap.avg_hr)
            .filter(|value| is_positive(*value))
            .collect::<Vec<_>>(),
    )?;
    if pace == 0.0 || heart_rate == 0.0 {
        return None;
    }
    let speed = 1000.0 / pace;
    Some(speed / (heart_rate / reference))
}

/// The whole derivation, in the reference's order.
fn derive(input: &FingerprintInput<'_>) -> Fingerprint {
    let activity = input.activity;
    let reference = input.max_hr_reference;

    let mut ordered: Vec<Lap> = activity.laps.clone();
    ordered.sort_by_key(|lap| lap.lap_index);
    let laps = running_only(&ordered);

    let third = laps.len() / 3;
    let (first, last): (&[Lap], &[Lap]) = if third > 0 {
        (&laps[..third], &laps[laps.len() - third..])
    } else {
        (&[], &[])
    };

    let cadence = positive_values(&laps, |lap| lap.avg_cadence);
    let efficiency_first = efficiency(first, reference);
    let efficiency_last = efficiency(last, reference);
    let decoupling = change(efficiency_first, efficiency_last).map(|value| -value);

    let pace_laps: &[Lap] = if !laps.is_empty()
        && is_positive(activity.distance_m)
        && laps[laps.len() - 1].distance_m.is_some_and(is_number)
        && laps[laps.len() - 1].distance_m.unwrap_or(0.0) < activity.distance_m * 0.15
    {
        &laps[..laps.len() - 1]
    } else {
        &laps
    };

    let cadence_cv = cv(&cadence);
    let pace_cv = cv(&paces(pace_laps));
    let pace_first = mean(&paces(first));
    let pace_last = mean(&paces(last));

    let components: [(f64, Option<f64>, f64); 3] = [
        (30.0, cadence_cv, 15.0),
        (45.0, decoupling.map(|value| value.max(0.0)), 25.0),
        (25.0, pace_cv, 25.0),
    ];
    let present: Vec<(f64, f64)> = components
        .iter()
        .filter_map(|(weight, value, limit)| {
            value.map(|value| (*weight, clamp01(1.0 - value / limit)))
        })
        .collect();
    let form_score = if present.is_empty() {
        None
    } else {
        let total_weight: f64 = present.iter().map(|(weight, _)| weight).sum();
        let weighted: f64 = present
            .iter()
            .map(|(weight, quality)| weight * quality)
            .sum();
        Some(weighted / total_weight * 100.0)
    };

    let mut zones = OrderedMap::new();
    if !activity.zones.is_empty() {
        for zone in &activity.zones {
            if is_number(zone.time_s) {
                zones.insert(zone.zone_no.to_string(), zone.time_s);
            }
        }
    } else {
        for lap in &laps {
            let heart_rate = match lap.avg_hr {
                Some(value) if is_positive(value) => value,
                _ => continue,
            };
            let duration = match lap.duration_s {
                Some(value) if is_positive(value) => value,
                _ => continue,
            };
            for zone in input.zone_model {
                let inside = heart_rate >= zone.low_bpm
                    && zone.high_bpm.map_or(true, |high| heart_rate < high);
                if inside {
                    let key = zone.zone_no.to_string();
                    match zones.get_mut(&key) {
                        Some(total) => *total += duration,
                        None => zones.push(key, duration),
                    }
                    break;
                }
            }
        }
    }

    let half = laps.len() / 2;
    let (first_half, second_half): (&[Lap], &[Lap]) = if half > 0 {
        (&laps[..half], &laps[half..])
    } else {
        (&[], &[])
    };
    let hr_first = mean(&positive_values(first_half, |lap| lap.avg_hr));
    let hr_second = mean(&positive_values(second_half, |lap| lap.avg_hr));
    let pace_first_half = mean(&paces(first_half));
    let pace_second_half = mean(&paces(second_half));

    let hr_drift_bpm = match (hr_first, hr_second) {
        (Some(first), Some(second)) => Some(second - first),
        _ => None,
    };
    let pace_drift_pct = match (pace_first_half, pace_second_half) {
        (Some(first), Some(second)) if first != 0.0 && second != 0.0 => {
            Some((second - first) / first * 100.0)
        }
        _ => None,
    };

    let mut hr_avg = match activity.avg_hr {
        Some(value) if is_positive(value) => Some(value),
        _ => mean(&positive_values(&laps, |lap| lap.avg_hr)),
    };
    if hr_avg.is_none() {
        let samples: Vec<f64> = activity
            .streams
            .get("hr")
            .map(|stream| {
                stream
                    .values
                    .iter()
                    .filter_map(|value| *value)
                    .filter(|value| is_positive(*value))
                    .collect()
            })
            .unwrap_or_default();
        hr_avg = mean(&samples);
    }

    let mut maxima = positive_values(&laps, |lap| lap.max_hr);
    if let Some(value) = activity.max_hr {
        if is_positive(value) {
            maxima.push(value);
        }
    }
    let hr_max = maxima
        .iter()
        .copied()
        .fold(None, |current: Option<f64>, value| {
            Some(match current {
                Some(existing) if existing >= value => existing,
                _ => value,
            })
        });

    let strides: Vec<f64> = positive_values(&laps, |lap| lap.stride_length_m)
        .into_iter()
        .filter(|value| *value <= 3.0)
        .collect();
    let stride_length_cm = mean(&strides).map(|value| value * 100.0);

    Fingerprint {
        cadence_avg: mean(&cadence),
        cadence_cv,
        cadence_drift_pct: change(
            mean(&positive_values(first, |lap| lap.avg_cadence)),
            mean(&positive_values(last, |lap| lap.avg_cadence)),
        ),
        hr_avg,
        hr_max,
        hr_drift_pct: change(
            mean(&positive_values(first, |lap| lap.avg_hr)),
            mean(&positive_values(last, |lap| lap.avg_hr)),
        ),
        efficiency_first,
        efficiency_last,
        decoupling_pct: decoupling,
        pace_cv,
        negative_split_pct: match (pace_first, pace_last) {
            (Some(first), Some(last)) => Some(if last < first { 100.0 } else { 0.0 }),
            _ => None,
        },
        stride_length_cm,
        form_score,
        zone_distribution: if zones.is_empty() { None } else { Some(zones) },
        running_time_s: activity.running_time_s,
        walking_time_s: activity.walking_time_s,
        stopped_time_s: activity.stopped_time_s,
        hr_drift_bpm,
        pace_drift_pct,
        first_half_hr: hr_first,
        second_half_hr: hr_second,
        first_half_pace_s_km: pace_first_half,
        second_half_pace_s_km: pace_second_half,
        notes: if activity.source == "demo" {
            Some("Synthetic demo activity.".to_string())
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Lap;

    fn lap(
        index: i64,
        distance: f64,
        duration: f64,
        cadence: Option<f64>,
        speed: Option<f64>,
    ) -> Lap {
        Lap {
            lap_index: index,
            distance_m: Some(distance),
            duration_s: Some(duration),
            avg_cadence: cadence,
            avg_speed_ms: speed,
            avg_hr: Some(140.0),
            max_hr: Some(150.0),
            ..Lap::default()
        }
    }

    #[test]
    fn a_slow_short_strided_lap_is_dropped_as_walked() {
        let laps = vec![
            lap(1, 1000.0, 300.0, Some(120.0), Some(2.0)),
            lap(2, 1000.0, 300.0, Some(160.0), Some(3.3)),
        ];
        let running = running_only(&laps);
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].lap_index, 2);
    }

    #[test]
    fn a_short_strided_but_fast_lap_is_kept() {
        // Both conditions must hold: a runner with a short stride is not a
        // walker.
        let laps = vec![lap(1, 1000.0, 300.0, Some(120.0), Some(3.3))];
        assert_eq!(running_only(&laps).len(), 1);
    }

    #[test]
    fn when_every_lap_looks_walked_they_are_all_kept() {
        let laps = vec![lap(1, 1000.0, 300.0, Some(100.0), Some(1.5))];
        assert_eq!(running_only(&laps).len(), 1);
    }

    #[test]
    fn the_efficiency_factor_is_speed_based() {
        // A slower run with the same heart rate has a *lower* efficiency
        // factor, which is the whole point of the Minetti definition.
        let fast = vec![lap(1, 1000.0, 250.0, Some(160.0), Some(4.0))];
        let slow = vec![lap(1, 1000.0, 400.0, Some(160.0), Some(2.5))];
        let fast_ef = efficiency(&fast, 190.0).expect("an efficiency");
        let slow_ef = efficiency(&slow, 190.0).expect("an efficiency");
        assert!(fast_ef > slow_ef);
    }

    #[test]
    fn the_maximum_heart_rate_hides_the_demo_only_when_the_demo_is_hidden() {
        let demo = Activity {
            max_hr: Some(199.0),
            source: "demo".to_string(),
            ..Activity::default()
        };
        let real = Activity {
            max_hr: Some(171.0),
            source: "garmin_fit".to_string(),
            ..Activity::default()
        };
        assert_eq!(
            max_hr_reference(&[demo.clone(), real.clone()], None, true),
            199.0
        );
        assert_eq!(max_hr_reference(&[demo, real], None, false), 171.0);
        assert_eq!(max_hr_reference(&[], None, true), DEFAULT_MAX_HR_REFERENCE);
    }

    #[test]
    fn a_lap_pace_uses_the_running_time_over_the_whole_distance() {
        // The reference's quirk: a lap that was partly walked gets a pace
        // that is too slow.
        let pace = lap_pace(&lap(1, 238.0, 75.0, Some(156.0), Some(3.0))).expect("a pace");
        assert!((pace - 75.0 * 1000.0 / 238.0).abs() < 1e-9);
    }
}

/// One item of the `/fitness/fingerprint` response.
///
/// The reference stores the derived payload next to a few facts about the
/// activity, and serves them in that order. The flattened fingerprint keeps
/// the order, so the router can serialise this straight into the response.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct FingerprintItem {
    /// The activity's row id.
    pub activity_id: i64,
    /// The derived mechanics.
    #[serde(flatten)]
    pub fingerprint: Fingerprint,
    /// The athlete's name for the run.
    pub activity_name: String,
    /// The athlete-local date of the run.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The average pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
    /// Where the run came from.
    pub source: String,
    /// Whether the run is a synthetic demo.
    pub is_demo: bool,
}

impl Serialize for FingerprintItem {
    /// The reference's key order: the id, the derived mechanics, then the
    /// facts about the activity.
    ///
    /// Serde's `flatten` would put the derived fields *last*, so the item
    /// serialises by hand instead.
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("activity_id", &self.activity_id)?;
        map.serialize_entry("cadence_avg", &self.fingerprint.cadence_avg)?;
        map.serialize_entry("cadence_cv", &self.fingerprint.cadence_cv)?;
        map.serialize_entry("cadence_drift_pct", &self.fingerprint.cadence_drift_pct)?;
        map.serialize_entry("hr_avg", &self.fingerprint.hr_avg)?;
        map.serialize_entry("hr_max", &self.fingerprint.hr_max)?;
        map.serialize_entry("hr_drift_pct", &self.fingerprint.hr_drift_pct)?;
        map.serialize_entry("efficiency_first", &self.fingerprint.efficiency_first)?;
        map.serialize_entry("efficiency_last", &self.fingerprint.efficiency_last)?;
        map.serialize_entry("decoupling_pct", &self.fingerprint.decoupling_pct)?;
        map.serialize_entry("pace_cv", &self.fingerprint.pace_cv)?;
        map.serialize_entry("negative_split_pct", &self.fingerprint.negative_split_pct)?;
        map.serialize_entry("stride_length_cm", &self.fingerprint.stride_length_cm)?;
        map.serialize_entry("form_score", &self.fingerprint.form_score)?;
        map.serialize_entry("zone_distribution", &self.fingerprint.zone_distribution)?;
        map.serialize_entry("running_time_s", &self.fingerprint.running_time_s)?;
        map.serialize_entry("walking_time_s", &self.fingerprint.walking_time_s)?;
        map.serialize_entry("stopped_time_s", &self.fingerprint.stopped_time_s)?;
        map.serialize_entry("hr_drift_bpm", &self.fingerprint.hr_drift_bpm)?;
        map.serialize_entry("pace_drift_pct", &self.fingerprint.pace_drift_pct)?;
        map.serialize_entry("first_half_hr", &self.fingerprint.first_half_hr)?;
        map.serialize_entry("second_half_hr", &self.fingerprint.second_half_hr)?;
        map.serialize_entry(
            "first_half_pace_s_km",
            &self.fingerprint.first_half_pace_s_km,
        )?;
        map.serialize_entry(
            "second_half_pace_s_km",
            &self.fingerprint.second_half_pace_s_km,
        )?;
        map.serialize_entry("notes", &self.fingerprint.notes)?;
        map.serialize_entry("activity_name", &self.activity_name)?;
        map.serialize_entry("local_date", &self.local_date)?;
        map.serialize_entry("distance_m", &self.distance_m)?;
        map.serialize_entry("pace_s_per_km", &self.pace_s_per_km)?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("is_demo", &self.is_demo)?;
        map.end()
    }
}

impl FingerprintItem {
    /// Build the item from an activity and its fingerprint.
    pub fn new(activity_id: i64, activity: &Activity, fingerprint: Fingerprint) -> Self {
        let pace = match (activity.moving_time_s, activity.distance_m) {
            (Some(moving), distance) if moving > 0.0 && distance > 0.0 => {
                Some(moving * 1000.0 / distance)
            }
            _ => None,
        };
        Self {
            activity_id,
            fingerprint,
            activity_name: activity.name.clone(),
            local_date: activity.local_date.clone(),
            distance_m: activity.distance_m,
            pace_s_per_km: pace,
            source: activity.source.clone(),
            is_demo: activity.source == "demo",
        }
    }
}

/// One zone's share of an activity's time in the zones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneShare {
    /// The zone's number.
    pub zone_no: i64,
    /// The zone's name.
    pub name: String,
    /// The zone's lower bound, in beats per minute.
    pub low_bpm: Option<f64>,
    /// The zone's upper bound, in beats per minute.
    pub high_bpm: Option<f64>,
    /// The time spent in the zone, in seconds.
    pub time_s: f64,
    /// The share of the activity's zoned time, in percent.
    pub percent: f64,
}

/// The time in each zone, as a share of the activity's zoned time.
///
/// The reference merges the zone model with the activity's own zone rows — a
/// stored row wins, because it carries the zone's real boundaries — and adds
/// the zones the distribution names even when the model does not.
pub fn zone_shares(
    distribution: Option<&OrderedMap<f64>>,
    model: &[Zone],
    stored: &[crate::model::ZoneTime],
) -> Vec<ZoneShare> {
    let empty = OrderedMap::new();
    let distribution = distribution.unwrap_or(&empty);
    let total: f64 = distribution.iter().map(|(_, seconds)| *seconds).sum();

    let mut zone_numbers: Vec<i64> = distribution
        .iter()
        .filter_map(|(key, _)| key.parse::<i64>().ok())
        .collect();
    for zone in model {
        zone_numbers.push(zone.zone_no);
    }
    zone_numbers.sort_unstable();
    zone_numbers.dedup();

    zone_numbers
        .into_iter()
        .map(|zone_no| {
            let metadata = model.iter().find(|zone| zone.zone_no == zone_no);
            let stored_row = stored.iter().find(|zone| zone.zone_no == zone_no);
            let (name, low_bpm, high_bpm) = match stored_row {
                Some(row) => (
                    row.name
                        .clone()
                        .unwrap_or_else(|| format!("Zone {zone_no}")),
                    row.low_bpm,
                    row.high_bpm,
                ),
                None => match metadata {
                    Some(zone) => (zone.name.clone(), Some(zone.low_bpm), zone.high_bpm),
                    None => (format!("Zone {zone_no}"), None, None),
                },
            };
            let seconds = distribution
                .get(&zone_no.to_string())
                .copied()
                .unwrap_or(0.0);
            ZoneShare {
                zone_no,
                name,
                low_bpm,
                high_bpm,
                time_s: seconds,
                percent: if total != 0.0 {
                    seconds / total * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod item_tests {
    use super::*;

    #[test]
    fn the_fitness_item_serialises_in_the_reference_order() {
        let activity = Activity {
            id: 7,
            name: "Morning easy".to_string(),
            local_date: "2026-03-02".to_string(),
            source: "garmin_fit".to_string(),
            distance_m: 443.0,
            moving_time_s: Some(150.0),
            ..Activity::default()
        };
        let item = FingerprintItem::new(7, &activity, Fingerprint::default());
        let value = serde_json::to_value(&item).expect("an item");
        assert_eq!(value["pace_s_per_km"], serde_json::json!(338.6004514672686));
        assert_eq!(value["is_demo"], serde_json::json!(false));

        // The key order is part of the contract, and `serde_json::Value`
        // sorts its keys — so the order is checked in the *text*, which
        // keeps the insertion order.
        let text = serde_json::to_string(&item).expect("an item");
        let expected = [
            "activity_id",
            "cadence_avg",
            "cadence_cv",
            "cadence_drift_pct",
            "hr_avg",
            "hr_max",
            "hr_drift_pct",
            "efficiency_first",
            "efficiency_last",
            "decoupling_pct",
            "pace_cv",
            "negative_split_pct",
            "stride_length_cm",
            "form_score",
            "zone_distribution",
            "running_time_s",
            "walking_time_s",
            "stopped_time_s",
            "hr_drift_bpm",
            "pace_drift_pct",
            "first_half_hr",
            "second_half_hr",
            "first_half_pace_s_km",
            "second_half_pace_s_km",
            "notes",
            "activity_name",
            "local_date",
            "distance_m",
            "pace_s_per_km",
            "source",
            "is_demo",
        ];
        let mut position = 0;
        for key in expected {
            let needle = format!("\"{key}\":");
            let found = text[position..]
                .find(&needle)
                .unwrap_or_else(|| panic!("{key} is missing or out of order"));
            position += found + needle.len();
        }
    }

    #[test]
    fn the_zone_shares_merge_the_model_and_the_stored_rows() {
        let model = default_zone_model(155.0);
        let mut distribution = OrderedMap::new();
        distribution.push("4", 75.0);
        distribution.push("5", 63.0);
        let shares = zone_shares(Some(&distribution), &model, &[]);
        assert_eq!(shares.len(), 5);
        assert_eq!(shares[3].zone_no, 4);
        assert_eq!(shares[3].name, "Threshold");
        assert_eq!(shares[3].low_bpm, Some(124.0));
        assert_eq!(shares[3].high_bpm, Some(139.5));
        assert_eq!(shares[3].time_s, 75.0);
        assert_eq!(shares[3].percent, 75.0 / 138.0 * 100.0);
        assert_eq!(shares[0].time_s, 0.0);
        assert_eq!(shares[0].percent, 0.0);
    }
}

/// One metric the trends view can chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Metric {
    /// The metric's key: the fingerprint field it reads.
    pub key: &'static str,
    /// The metric's label, as the UI shows it.
    pub label: &'static str,
    /// The metric's unit.
    pub unit: &'static str,
    /// Which direction is good: `higher`, `lower` or `neutral`.
    pub better: &'static str,
}

/// The metrics the trends view can chart.
///
/// The reference validates the metric against this table and serves the
/// label, the unit and the direction with every trend.
pub const METRICS: [Metric; 11] = [
    Metric {
        key: "cadence_avg",
        label: "Average cadence",
        unit: "spm",
        better: "higher",
    },
    Metric {
        key: "cadence_cv",
        label: "Cadence variability",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "cadence_drift_pct",
        label: "Cadence drift",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "hr_drift_pct",
        label: "Heart rate drift",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "hr_drift_bpm",
        label: "Heart rate drift",
        unit: "bpm",
        better: "lower",
    },
    Metric {
        key: "pace_drift_pct",
        label: "Pace drift",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "decoupling_pct",
        label: "Aerobic decoupling",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "efficiency_first",
        label: "Efficiency factor",
        unit: "m/s per HR fraction",
        better: "higher",
    },
    Metric {
        key: "pace_cv",
        label: "Pace variability",
        unit: "%",
        better: "lower",
    },
    Metric {
        key: "stride_length_cm",
        label: "Stride length",
        unit: "cm",
        better: "neutral",
    },
    Metric {
        key: "form_score",
        label: "Form score",
        unit: "points",
        better: "higher",
    },
];

/// The metadata of one chartable metric.
pub fn metric(key: &str) -> Option<&'static Metric> {
    METRICS.iter().find(|metric| metric.key == key)
}

/// One run's value of a chartable metric.
pub fn fingerprint_metric(fingerprint: &Fingerprint, key: &str) -> Option<f64> {
    match key {
        "cadence_avg" => fingerprint.cadence_avg,
        "cadence_cv" => fingerprint.cadence_cv,
        "cadence_drift_pct" => fingerprint.cadence_drift_pct,
        "hr_drift_pct" => fingerprint.hr_drift_pct,
        "hr_drift_bpm" => fingerprint.hr_drift_bpm,
        "pace_drift_pct" => fingerprint.pace_drift_pct,
        "decoupling_pct" => fingerprint.decoupling_pct,
        "efficiency_first" => fingerprint.efficiency_first,
        "pace_cv" => fingerprint.pace_cv,
        "stride_length_cm" => fingerprint.stride_length_cm,
        "form_score" => fingerprint.form_score,
        _ => None,
    }
}

/// One run in a trend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendInput {
    /// The athlete-local date of the run.
    pub local_date: String,
    /// The run's row id.
    pub activity_id: i64,
    /// The run's value of the metric, if it has one.
    pub value: Option<f64>,
}

/// One point of a trend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendPoint {
    /// The athlete-local date of the run.
    pub date: String,
    /// The run's value of the metric.
    pub value: f64,
    /// The run's row id.
    pub activity_id: i64,
}

/// The trend of one metric over the athlete's runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trend {
    /// The metric's key.
    pub metric: String,
    /// The points, in date order.
    pub points: Vec<TrendPoint>,
    /// The mean of the first half.
    pub baseline: Option<f64>,
    /// The change from the first half to the second, in percent.
    pub trend_pct: Option<f64>,
    /// The metric's label.
    pub label: String,
    /// The metric's unit.
    pub unit: String,
    /// Which direction is good.
    pub better: String,
}

/// Compare the chronological halves of one metric.
///
/// A zero baseline has no percentage change, so the trend is `None` rather
/// than a division by zero.
pub fn fingerprint_trend(rows: &[TrendInput], key: &str) -> Option<Trend> {
    let metric = metric(key)?;
    let mut points: Vec<TrendPoint> = rows
        .iter()
        .filter(|row| row.value.is_some_and(|value| value.is_finite()))
        .map(|row| TrendPoint {
            date: row.local_date.clone(),
            value: row.value.unwrap_or_default(),
            activity_id: row.activity_id,
        })
        .collect();
    points.sort_by(|left, right| {
        left.date
            .cmp(&right.date)
            .then_with(|| left.activity_id.cmp(&right.activity_id))
    });

    let half = points.len() / 2;
    let baseline = if half == 0 {
        None
    } else {
        crate::numeric::mean(
            &points[..half]
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
        )
    };
    let recent = if half == 0 {
        None
    } else {
        crate::numeric::mean(
            &points[half..]
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
        )
    };
    let trend_pct = match (baseline, recent) {
        (Some(baseline), Some(recent)) if baseline != 0.0 => {
            Some((recent - baseline) / baseline.abs() * 100.0)
        }
        _ => None,
    };

    Some(Trend {
        metric: key.to_string(),
        points,
        baseline,
        trend_pct,
        label: metric.label.to_string(),
        unit: metric.unit.to_string(),
        better: metric.better.to_string(),
    })
}

#[cfg(test)]
mod trend_tests {
    use super::*;

    fn row(date: &str, id: i64, value: Option<f64>) -> TrendInput {
        TrendInput {
            local_date: date.to_string(),
            activity_id: id,
            value,
        }
    }

    #[test]
    fn the_metric_table_is_the_reference_table() {
        assert_eq!(METRICS.len(), 11);
        assert_eq!(metric("form_score").expect("a metric").label, "Form score");
        assert_eq!(metric("form_score").expect("a metric").better, "higher");
        assert_eq!(metric("pace_cv").expect("a metric").better, "lower");
        assert_eq!(
            metric("stride_length_cm").expect("a metric").better,
            "neutral"
        );
        assert!(metric("hr_avg").is_none());
    }

    #[test]
    fn the_trend_compares_the_chronological_halves() {
        let rows = vec![
            row("2026-03-03", 3, Some(30.0)),
            row("2026-03-01", 1, Some(10.0)),
            row("2026-03-02", 2, Some(20.0)),
        ];
        let trend = fingerprint_trend(&rows, "form_score").expect("a trend");
        // The points are in date order, whatever order the rows arrive in.
        let dates: Vec<&str> = trend
            .points
            .iter()
            .map(|point| point.date.as_str())
            .collect();
        assert_eq!(dates, vec!["2026-03-01", "2026-03-02", "2026-03-03"]);
        // Three points: the first half is one point, the second half is two.
        assert_eq!(trend.baseline, Some(10.0));
        assert_eq!(trend.trend_pct, Some(150.0));
        assert_eq!(trend.label, "Form score");
        assert_eq!(trend.unit, "points");
        assert_eq!(trend.better, "higher");
    }

    #[test]
    fn a_trend_without_a_baseline_has_no_percentage() {
        let rows = vec![row("2026-03-01", 1, Some(10.0))];
        let trend = fingerprint_trend(&rows, "form_score").expect("a trend");
        assert_eq!(trend.baseline, None);
        assert_eq!(trend.trend_pct, None);
    }

    #[test]
    fn an_absent_value_is_not_a_point() {
        let rows = vec![row("2026-03-01", 1, None), row("2026-03-02", 2, Some(20.0))];
        let trend = fingerprint_trend(&rows, "form_score").expect("a trend");
        assert_eq!(trend.points.len(), 1);
        assert_eq!(trend.points[0].activity_id, 2);
    }
}
