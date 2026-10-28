//! The run comparison: two runs, fifteen parameters, one shared distance axis.
//!
//! The reference is `backend/app/services/comparison.py`. The idea is that two
//! runs of the same athlete at different points in their career answer
//! different questions, and the interesting ones are comparative: *was that run
//! really better, or was it just shorter and flatter?* So both runs are
//! resampled onto the same distance axis — 120 normalised points — and every
//! metric is read at the same point of that axis.
//!
//! # The fifteen parameters
//!
//! Each parameter says which direction is good (`neutral`, `lower` or
//! `higher`), so the winner is a fact about the parameter rather than a rule
//! the caller has to know. Eight of them are read from the run's fingerprint
//! when it has one, and from the activity otherwise.

use serde::{Deserialize, Serialize};

use crate::model::{Activity, Fingerprint};

/// How many points each run's profile is resampled to.
pub const PROFILE_POINTS: usize = 120;

/// Which direction of a parameter is good.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Better {
    /// Neither direction is better.
    Neutral,
    /// A smaller value is better.
    Lower,
    /// A larger value is better.
    Higher,
}

/// One row of the comparison matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Parameter {
    /// The parameter's key.
    pub key: &'static str,
    /// The parameter's label.
    pub label: &'static str,
    /// The parameter's unit.
    pub unit: &'static str,
    /// Which direction is good.
    pub better: Better,
    /// How the UI should format the value.
    pub format: &'static str,
}

/// The comparison matrix.
pub const PARAMETERS: [Parameter; 15] = [
    Parameter {
        key: "distance_m",
        label: "Distance",
        unit: "km",
        better: Better::Neutral,
        format: "distance",
    },
    Parameter {
        key: "moving_time_s",
        label: "Moving time",
        unit: "",
        better: Better::Neutral,
        format: "duration",
    },
    Parameter {
        key: "pace_s_per_km",
        label: "Average pace",
        unit: "/km",
        better: Better::Lower,
        format: "pace",
    },
    Parameter {
        key: "elevation_gain_m",
        label: "Elevation gain",
        unit: "m",
        better: Better::Neutral,
        format: "elevation",
    },
    Parameter {
        key: "avg_hr",
        label: "Average heart rate",
        unit: "bpm",
        better: Better::Neutral,
        format: "hr",
    },
    Parameter {
        key: "hr_drift_bpm",
        label: "HR drift",
        unit: "bpm",
        better: Better::Lower,
        format: "number",
    },
    Parameter {
        key: "pace_drift_pct",
        label: "Pace drift",
        unit: "%",
        better: Better::Lower,
        format: "number",
    },
    Parameter {
        key: "decoupling_pct",
        label: "Aerobic decoupling",
        unit: "%",
        better: Better::Lower,
        format: "number",
    },
    Parameter {
        key: "cadence_avg",
        label: "Average cadence",
        unit: "spm",
        better: Better::Neutral,
        format: "number",
    },
    Parameter {
        key: "cadence_cv",
        label: "Cadence stability",
        unit: "%",
        better: Better::Lower,
        format: "number",
    },
    Parameter {
        key: "efficiency_first",
        label: "Efficiency factor",
        unit: "",
        better: Better::Higher,
        format: "number",
    },
    Parameter {
        key: "form_score",
        label: "Form score",
        unit: "",
        better: Better::Higher,
        format: "number",
    },
    Parameter {
        key: "running_time_s",
        label: "Running time",
        unit: "",
        better: Better::Neutral,
        format: "duration",
    },
    Parameter {
        key: "walking_time_s",
        label: "Walking excluded",
        unit: "",
        better: Better::Lower,
        format: "duration",
    },
    Parameter {
        key: "stopped_time_s",
        label: "Standing excluded",
        unit: "",
        better: Better::Lower,
        format: "duration",
    },
];

/// Which fingerprint field each parameter reads, when it is not on the
/// activity.
const FINGERPRINT_KEYS: [(&str, &str); 11] = [
    ("avg_hr", "hr_avg"),
    ("hr_drift_bpm", "hr_drift_bpm"),
    ("pace_drift_pct", "pace_drift_pct"),
    ("decoupling_pct", "decoupling_pct"),
    ("cadence_avg", "cadence_avg"),
    ("cadence_cv", "cadence_cv"),
    ("efficiency_first", "efficiency_first"),
    ("form_score", "form_score"),
    ("running_time_s", "running_time_s"),
    ("walking_time_s", "walking_time_s"),
    ("stopped_time_s", "stopped_time_s"),
];

/// Which run won a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Winner {
    /// The first run.
    A,
    /// The second run.
    B,
    /// Neither: the two runs are the same.
    Tie,
}

/// One parameter, in both runs.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ParameterComparison {
    /// The parameter.
    pub parameter: Parameter,
    /// The first run's value.
    pub a: Option<f64>,
    /// The second run's value.
    pub b: Option<f64>,
    /// The winner. `None` when the parameter is neutral or a value is missing.
    pub winner: Option<Winner>,
}

/// One point of a resampled run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePoint {
    /// The distance along the run, in metres.
    pub distance_m: f64,
    /// How far along the run this point is, in percent.
    pub pct: f64,
    /// The interpolated heart rate.
    pub hr: Option<f64>,
    /// The interpolated cadence.
    pub cadence: Option<f64>,
    /// The interpolated pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
    /// The interpolated elevation, in metres.
    pub elevation: Option<f64>,
}

/// The comparison of two runs.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Comparison {
    /// The fifteen parameters, with both runs' values.
    pub parameters: Vec<ParameterComparison>,
    /// The first run's profile.
    pub profile_a: Vec<ProfilePoint>,
    /// The second run's profile.
    pub profile_b: Vec<ProfilePoint>,
}

/// The average pace of a run, in seconds per kilometre.
///
/// The comparison uses the stricter of the reference's two pace helpers: a
/// distance or a moving time that is zero or negative means there is no pace.
fn pace_s_per_km(distance_m: f64, moving_time_s: Option<f64>) -> Option<f64> {
    let moving = moving_time_s?;
    if distance_m <= 0.0 || moving <= 0.0 {
        return None;
    }
    Some(moving / (distance_m / 1000.0))
}

/// Linear interpolation over `(x, y)` pairs, sorted by x.
fn interpolate(samples: &[(f64, f64)], target: f64) -> Option<f64> {
    let first = samples.first()?;
    let last = samples.last()?;
    if target <= first.0 {
        return Some(first.1);
    }
    if target >= last.0 {
        return Some(last.1);
    }
    for index in 1..samples.len() {
        let (x0, y0) = samples[index - 1];
        let (x1, y1) = samples[index];
        if x0 <= target && target <= x1 {
            if x1 == x0 {
                return Some(y1);
            }
            return Some(y0 + (y1 - y0) * (target - x0) / (x1 - x0));
        }
    }
    None
}

/// Resample a run's running streams onto a shared distance axis.
///
/// The streams all share one sample axis (the running samples only), so the
/// profile is built by walking the distance stream and interpolating every
/// other metric at the same point.
pub fn profile_for(activity: &Activity, points: usize) -> Vec<ProfilePoint> {
    if points < 2 {
        return Vec::new();
    }
    let distance_stream = match activity.streams.get("distance") {
        Some(stream) => stream,
        None => return Vec::new(),
    };

    let raw_seconds = &distance_stream.seconds;
    let raw_distances = distance_stream.present_values();
    let count = raw_seconds.len().min(raw_distances.len());
    if count < 2 {
        return Vec::new();
    }
    let distances = &raw_distances[..count];

    let series = |kind: &str| -> Vec<(f64, f64)> {
        match activity.streams.get(kind) {
            Some(stream) => {
                let values = stream.present_values();
                let mut pairs: Vec<(f64, f64)> = (0..count.min(values.len()))
                    .map(|index| (distances[index], values[index]))
                    .collect();
                // The reference sorts the `(x, y)` tuples, so equal distances
                // are ordered by their value.
                pairs.sort_by(|left, right| {
                    left.0
                        .partial_cmp(&right.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            left.1
                                .partial_cmp(&right.1)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                });
                pairs
            }
            None => Vec::new(),
        }
    };

    let hr = series("hr");
    let cadence = series("cadence");
    let pace = series("pace");
    let elevation = series("elevation");

    let start = distances[0];
    let end = distances[count - 1];
    if end <= start {
        return Vec::new();
    }

    (0..points)
        .map(|index| {
            let fraction = index as f64 / (points - 1) as f64;
            let metres = start + (end - start) * fraction;
            ProfilePoint {
                distance_m: crate::numeric::round_to(metres, 1),
                pct: crate::numeric::round_to(fraction * 100.0, 2),
                hr: interpolate(&hr, metres),
                cadence: interpolate(&cadence, metres),
                pace_s_per_km: interpolate(&pace, metres),
                elevation: interpolate(&elevation, metres),
            }
        })
        .collect()
}

/// The value of one parameter for one run.
pub fn value_for(parameter: &Parameter, activity: &Activity) -> Option<f64> {
    if let Some(fingerprint) = &activity.fingerprint {
        if let Some((_, field)) = FINGERPRINT_KEYS
            .iter()
            .find(|(key, _)| *key == parameter.key)
        {
            return fingerprint_field(fingerprint, field);
        }
    }
    match parameter.key {
        "distance_m" => Some(activity.distance_m),
        "moving_time_s" => activity.moving_time_s,
        "pace_s_per_km" => pace_s_per_km(activity.distance_m, activity.moving_time_s),
        "elevation_gain_m" => activity.elevation_gain_m,
        "avg_hr" => activity.avg_hr,
        _ => None,
    }
}

/// One field of the fingerprint, by name.
fn fingerprint_field(fingerprint: &Fingerprint, field: &str) -> Option<f64> {
    match field {
        "hr_avg" => fingerprint.hr_avg,
        "hr_drift_bpm" => fingerprint.hr_drift_bpm,
        "pace_drift_pct" => fingerprint.pace_drift_pct,
        "decoupling_pct" => fingerprint.decoupling_pct,
        "cadence_avg" => fingerprint.cadence_avg,
        "cadence_cv" => fingerprint.cadence_cv,
        "efficiency_first" => fingerprint.efficiency_first,
        "form_score" => fingerprint.form_score,
        "running_time_s" => fingerprint.running_time_s,
        "walking_time_s" => fingerprint.walking_time_s,
        "stopped_time_s" => fingerprint.stopped_time_s,
        _ => None,
    }
}

/// Which run won one parameter.
fn winner(better: Better, a: Option<f64>, b: Option<f64>) -> Option<Winner> {
    match better {
        Better::Neutral => None,
        Better::Lower | Better::Higher => {
            let (a, b) = (a?, b?);
            if a == b {
                Some(Winner::Tie)
            } else {
                let a_wins = match better {
                    Better::Lower => a < b,
                    Better::Higher => a > b,
                    Better::Neutral => false,
                };
                Some(if a_wins { Winner::A } else { Winner::B })
            }
        }
    }
}

/// Compare two runs.
pub fn compare(a: &Activity, b: &Activity) -> Comparison {
    let parameters = PARAMETERS
        .iter()
        .map(|parameter| {
            let left = value_for(parameter, a);
            let right = value_for(parameter, b);
            ParameterComparison {
                parameter: *parameter,
                a: left,
                b: right,
                winner: winner(parameter.better, left, right),
            }
        })
        .collect();
    Comparison {
        parameters,
        profile_a: profile_for(a, PROFILE_POINTS),
        profile_b: profile_for(b, PROFILE_POINTS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Stream;

    fn stream(seconds: Vec<f64>, values: Vec<Option<f64>>) -> Stream {
        Stream { seconds, values, unit: None }
    }

    #[test]
    fn the_profile_sorts_the_pairs_by_distance_and_then_by_value() {
        // The reference sorts `(x, y)` tuples, so two samples at the same
        // distance are ordered by their value and the interpolation reads the
        // smaller one.
        let mut streams = std::collections::BTreeMap::new();
        streams.insert(
            "distance".to_string(),
            stream(vec![0.0, 1.0, 2.0], vec![Some(0.0), Some(10.0), Some(20.0)]),
        );
        streams.insert(
            "hr".to_string(),
            stream(vec![0.0, 1.0, 2.0], vec![Some(100.0), Some(200.0), Some(300.0)]),
        );
        let activity = Activity { streams, ..Activity::default() };
        let profile = profile_for(&activity, 3);
        assert_eq!(profile.len(), 3);
        assert_eq!(profile[0].distance_m, 0.0);
        assert_eq!(profile[1].distance_m, 10.0);
        assert_eq!(profile[2].distance_m, 20.0);
        assert_eq!(profile[0].hr, Some(100.0));
        assert_eq!(profile[2].hr, Some(300.0));
    }

    #[test]
    fn a_missing_value_shifts_the_series_like_the_reference_does() {
        // The reference filters the absent values out of a stream *before*
        // pairing it with the distance axis, so the series shifts. The port
        // reproduces that, because the golden files were made with it.
        let mut streams = std::collections::BTreeMap::new();
        streams.insert(
            "distance".to_string(),
            stream(vec![0.0, 1.0, 2.0], vec![Some(0.0), Some(10.0), Some(20.0)]),
        );
        streams.insert(
            "hr".to_string(),
            stream(vec![0.0, 1.0, 2.0], vec![None, Some(200.0), Some(300.0)]),
        );
        let activity = Activity { streams, ..Activity::default() };
        let profile = profile_for(&activity, 3);
        // The absent sample is dropped, so 200 pairs with 0 m and 300 with
        // 10 m; beyond 10 m the profile holds the last value.
        assert_eq!(profile[0].hr, Some(200.0));
        assert_eq!(profile[1].hr, Some(300.0));
        assert_eq!(profile[2].hr, Some(300.0));
    }

    #[test]
    fn the_parameter_list_is_the_reference_matrix() {
        assert_eq!(PARAMETERS.len(), 15);
        assert_eq!(PARAMETERS[0].key, "distance_m");
        assert_eq!(PARAMETERS[2].better, Better::Lower);
        assert_eq!(PARAMETERS[10].better, Better::Higher);
        assert_eq!(PARAMETERS[4].better, Better::Neutral);
    }
}
