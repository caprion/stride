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
/// The reference reads it from the athlete's activities, ignoring demo rows
/// once real data exists. It falls back to 190 when there is nothing to read.
pub fn max_hr_reference(activities: &[Activity]) -> f64 {
    let mut maximum: Option<f64> = None;
    for activity in activities {
        if activity.source == "demo" {
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
    fn a_lap_pace_uses_the_running_time_over_the_whole_distance() {
        // The reference's quirk: a lap that was partly walked gets a pace
        // that is too slow.
        let pace = lap_pace(&lap(1, 238.0, 75.0, Some(156.0), Some(3.0))).expect("a pace");
        assert!((pace - 75.0 * 1000.0 / 238.0).abs() < 1e-9);
    }
}
