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
use crate::ordered::OrderedMap;

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

/// The session types that say what a run was *for*.
///
/// `rest` and `strength` are not in the list: a run is never one of them,
/// so a session of that type gives a run no intent.
pub const RUN_INTENT_TYPES: [&str; 6] = ["easy", "long", "tempo", "interval", "race", "recovery"];

/// The basis of a comparison: what makes the runs comparable at all.
///
/// "Strongest" only means something between runs that had the same job. A
/// time trial beats an easy run on pace by design; marking it teaches
/// faster-is-better on the page a runner opens after easy weeks. So the
/// basis is decided before anything is marked, and the page says which
/// basis it used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ComparisonBasis {
    /// Every run was matched to a plan session of the same kind, so the
    /// comparison is like for like.
    PlanSession {
        /// The canonical session type they share.
        session_type: String,
    },
    /// The runs were matched to plan sessions of different kinds, so they
    /// had different jobs and nothing is marked as strongest.
    DifferentJobs,
    /// At least one run has no plan intent, so distance is the proxy.
    Distance,
}

/// The intent of one run: the canonical session type it was matched to.
///
/// `None` when the run was not matched to a plan session, or when the
/// session it was matched to is not a run — a rest day or a strength
/// session says nothing about what the run was for.
pub fn plan_intent(session_type: Option<&str>) -> Option<String> {
    let canonical = crate::plan::canonical_session_type(session_type?);
    if RUN_INTENT_TYPES.contains(&canonical.as_str()) {
        Some(canonical)
    } else {
        None
    }
}

/// The basis of a comparison, from the plan intent of each run.
///
/// * every run has an intent and they are all the same — like for like;
/// * every run has an intent and they differ — different jobs;
/// * any run has no intent — distance is the proxy, and the page says so.
pub fn comparison_basis(intents: &[Option<String>]) -> ComparisonBasis {
    if intents.is_empty() {
        return ComparisonBasis::Distance;
    }
    let mut kinds: Vec<&str> = Vec::with_capacity(intents.len());
    for intent in intents {
        match intent.as_deref() {
            Some(kind) => kinds.push(kind),
            None => return ComparisonBasis::Distance,
        }
    }
    let first = kinds[0];
    if kinds.iter().all(|kind| *kind == first) {
        ComparisonBasis::PlanSession {
            session_type: first.to_string(),
        }
    } else {
        ComparisonBasis::DifferentJobs
    }
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

/// The fifteen parameter values of one run, keyed by the parameter's key.
///
/// This is the reference's `values` map — the object the comparison view
/// reads — so the router can serve it without inventing a shape.
pub fn values_for(activity: &Activity) -> OrderedMap<Option<f64>> {
    let mut values = OrderedMap::new();
    for parameter in PARAMETERS.iter() {
        values.push(parameter.key, value_for(parameter, activity));
    }
    values
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
        Stream {
            seconds,
            values,
            unit: None,
        }
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
            stream(
                vec![0.0, 1.0, 2.0],
                vec![Some(100.0), Some(200.0), Some(300.0)],
            ),
        );
        let activity = Activity {
            streams,
            ..Activity::default()
        };
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
        let activity = Activity {
            streams,
            ..Activity::default()
        };
        let profile = profile_for(&activity, 3);
        // The absent sample is dropped, so 200 pairs with 0 m and 300 with
        // 10 m; beyond 10 m the profile holds the last value.
        assert_eq!(profile[0].hr, Some(200.0));
        assert_eq!(profile[1].hr, Some(300.0));
        assert_eq!(profile[2].hr, Some(300.0));
    }

    #[test]
    fn the_values_map_is_the_reference_values_object() {
        // The router serves the reference's `values` object: the fifteen
        // parameter keys, in the matrix's order.
        let activity = Activity {
            distance_m: 1980.0,
            moving_time_s: Some(600.0),
            elevation_gain_m: Some(0.0),
            avg_hr: Some(145.0),
            ..Activity::default()
        };
        let values = values_for(&activity);
        let keys: Vec<&str> = values.iter().map(|(key, _)| key).collect();
        let expected: Vec<&str> = PARAMETERS.iter().map(|p| p.key).collect();
        assert_eq!(keys, expected);
        assert_eq!(values.get("distance_m"), Some(&Some(1980.0)));
        let pace = values
            .get("pace_s_per_km")
            .copied()
            .flatten()
            .expect("a pace");
        assert!((pace - 303.0303030303).abs() < 1e-9);
        assert_eq!(values.get("avg_hr"), Some(&Some(145.0)));
        assert_eq!(values.get("form_score"), Some(&None));
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

/// The sports the comparison offers as candidates.
///
/// The reference lists four: the run sports the comparison can chart.
pub const CANDIDATE_SPORTS: [&str; 4] = [
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
];

/// A run as the comparison view reads it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunPayload {
    /// The run's row id.
    pub id: i64,
    /// The athlete's name for the run.
    pub name: String,
    /// The athlete-local date of the run.
    pub local_date: String,
    /// The weekday name of the run.
    pub day_of_week: Option<String>,
    /// The normalised sport type.
    pub sport_type: String,
    /// Where the run came from.
    pub source: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: Option<f64>,
    /// The moving time as `h:mm:ss` or `mm:ss`.
    pub moving_time_label: Option<String>,
    /// The elevation gain, in metres.
    pub elevation_gain_m: Option<f64>,
    /// The average heart rate, in beats per minute.
    pub avg_hr: Option<f64>,
    /// The maximum heart rate, in beats per minute.
    pub max_hr: Option<f64>,
    /// The calories burned.
    pub calories: Option<f64>,
    /// The average pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
    /// Whether the run has a route.
    pub has_geo: bool,
    /// The fifteen parameter values.
    pub values: OrderedMap<Option<f64>>,
    /// The run's profile: 120 points on a shared distance axis.
    pub profile: Vec<ProfilePoint>,
}

/// A run as the candidate list reads it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Candidate {
    /// The run's row id.
    pub id: i64,
    /// The athlete's name for the run.
    pub name: String,
    /// The athlete-local date of the run.
    pub local_date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The average pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
    /// The average heart rate, in beats per minute.
    pub avg_hr: Option<f64>,
    /// The elevation gain, in metres.
    pub elevation_gain_m: Option<f64>,
    /// Whether the run has a fingerprint to compare.
    pub has_fingerprint: bool,
    /// Whether the run has streams, and so a profile.
    pub has_profile: bool,
    /// Where the run came from.
    pub source: String,
}

/// A duration as `h:mm:ss` or `mm:ss`.
///
/// The reference rounds to the nearest second and leaves the hours out when
/// there are none.
pub fn duration_label(seconds: Option<f64>) -> Option<String> {
    let seconds = seconds?;
    if seconds == 0.0 {
        return None;
    }
    let total = crate::numeric::round_int(seconds);
    let (hours, remainder) = (total.div_euclid(3600), total.rem_euclid(3600));
    let (minutes, secs) = (remainder.div_euclid(60), remainder.rem_euclid(60));
    if hours > 0 {
        Some(format!("{hours}:{minutes:02}:{secs:02}"))
    } else {
        Some(format!("{minutes}:{secs:02}"))
    }
}

/// Everything the comparison view needs about one run.
pub fn run_payload(activity: &Activity) -> RunPayload {
    RunPayload {
        id: activity.id,
        name: activity.name.clone(),
        local_date: activity.local_date.clone(),
        day_of_week: activity.day_of_week.clone(),
        sport_type: activity.sport_type.clone(),
        source: activity.source.clone(),
        distance_m: activity.distance_m,
        moving_time_s: activity.moving_time_s,
        moving_time_label: duration_label(activity.moving_time_s),
        elevation_gain_m: activity.elevation_gain_m,
        avg_hr: activity.avg_hr,
        max_hr: activity.max_hr,
        calories: activity.calories,
        pace_s_per_km: pace_s_per_km(activity.distance_m, activity.moving_time_s),
        has_geo: activity
            .geo_points
            .as_ref()
            .is_some_and(|points| !points.is_empty()),
        values: values_for(activity),
        profile: profile_for(activity, PROFILE_POINTS),
    }
}

/// The runs that can be compared.
///
/// The reference reads them in start-time order and keeps the first `limit`;
/// the caller passes the runs in that order, because the order is the store's
/// query, not the domain's.
///
/// The demo runs are hidden when the demo is hidden, which is the same rule
/// the reference's `demo_filter` applies.
pub fn candidates(activities: &[Activity], limit: usize, demo_visible: bool) -> Vec<Candidate> {
    activities
        .iter()
        .filter(|activity| {
            (demo_visible || activity.source != "demo")
                && CANDIDATE_SPORTS.contains(&activity.sport_type.as_str())
        })
        .take(limit)
        .map(|activity| Candidate {
            id: activity.id,
            name: activity.name.clone(),
            local_date: activity.local_date.clone(),
            distance_m: activity.distance_m,
            pace_s_per_km: pace_s_per_km(activity.distance_m, activity.moving_time_s),
            avg_hr: activity.avg_hr,
            elevation_gain_m: activity.elevation_gain_m,
            has_fingerprint: activity.fingerprint.is_some(),
            has_profile: !activity.streams.is_empty(),
            source: activity.source.clone(),
        })
        .collect()
}

#[cfg(test)]
mod payload_tests {
    use super::*;

    fn activity() -> Activity {
        Activity {
            id: 7,
            name: "Morning easy".to_string(),
            local_date: "2026-03-02".to_string(),
            start_date: "2026-03-02".to_string(),
            day_of_week: Some("Monday".to_string()),
            sport_type: "running".to_string(),
            source: "garmin_fit".to_string(),
            distance_m: 1980.0,
            moving_time_s: Some(600.0),
            elevation_gain_m: Some(0.0),
            avg_hr: Some(145.0),
            max_hr: Some(150.0),
            calories: Some(210.0),
            ..Activity::default()
        }
    }

    #[test]
    fn a_duration_is_labelled_in_hours_and_minutes() {
        assert_eq!(duration_label(Some(60.0)), Some("1:00".to_string()));
        assert_eq!(duration_label(Some(600.0)), Some("10:00".to_string()));
        assert_eq!(duration_label(Some(3661.0)), Some("1:01:01".to_string()));
        assert_eq!(duration_label(Some(0.0)), None);
        assert_eq!(duration_label(None), None);
    }

    #[test]
    fn the_run_payload_is_the_reference_shape() {
        let payload = run_payload(&activity());
        assert_eq!(payload.id, 7);
        assert_eq!(payload.name, "Morning easy");
        assert_eq!(payload.moving_time_label, Some("10:00".to_string()));
        let pace = payload.pace_s_per_km.expect("a pace");
        assert!((pace - 303.0303030303).abs() < 1e-9);
        assert!(!payload.has_geo);
        assert_eq!(payload.values.len(), 15);
        // Without streams there is no profile to compare.
        assert!(payload.profile.is_empty());
    }

    #[test]
    fn a_run_with_a_route_says_so() {
        let mut activity = activity();
        activity.geo_points = Some(vec![[28.6, 77.2]]);
        assert!(run_payload(&activity).has_geo);
    }

    #[test]
    fn the_candidates_are_the_four_run_sports() {
        let mut walking = activity();
        walking.id = 8;
        walking.sport_type = "walking".to_string();
        let mut running = activity();
        running.id = 9;
        let list = candidates(&[walking, running], 300, true);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, 9);
        assert!(!list[0].has_fingerprint);
        assert!(!list[0].has_profile);
    }

    #[test]
    fn the_demo_runs_are_hidden_when_the_demo_is_hidden() {
        let mut demo = activity();
        demo.id = 10;
        demo.source = "demo".to_string();
        assert_eq!(candidates(&[demo.clone()], 300, true).len(), 1);
        assert_eq!(candidates(&[demo], 300, false).len(), 0);
    }

    #[test]
    fn two_runs_with_the_same_plan_intent_are_compared_like_for_like() {
        let basis = comparison_basis(&[Some("easy".to_string()), Some("easy".to_string())]);
        assert_eq!(
            basis,
            ComparisonBasis::PlanSession {
                session_type: "easy".to_string()
            }
        );
    }

    #[test]
    fn two_runs_with_different_plan_intents_had_different_jobs() {
        let basis = comparison_basis(&[Some("easy".to_string()), Some("tempo".to_string())]);
        assert_eq!(basis, ComparisonBasis::DifferentJobs);
    }

    #[test]
    fn a_run_without_a_plan_intent_falls_back_to_distance() {
        let basis = comparison_basis(&[Some("tempo".to_string()), None]);
        assert_eq!(basis, ComparisonBasis::Distance);
        let both = comparison_basis(&[None, None]);
        assert_eq!(both, ComparisonBasis::Distance);
        let none = comparison_basis(&[]);
        assert_eq!(none, ComparisonBasis::Distance);
    }

    #[test]
    fn a_rest_or_a_strength_session_gives_a_run_no_intent() {
        assert_eq!(plan_intent(Some("easy")), Some("easy".to_string()));
        assert_eq!(plan_intent(Some("Intervals")), Some("interval".to_string()));
        assert_eq!(plan_intent(Some("long_run")), Some("long".to_string()));
        assert_eq!(plan_intent(Some("rest")), None);
        assert_eq!(plan_intent(Some("strength")), None);
        assert_eq!(plan_intent(None), None);
    }

    #[test]
    fn an_unknown_session_type_gives_a_run_no_intent() {
        // A row that names no kind of session says nothing about what the
        // run was for, so the comparison has no basis and falls back to
        // distance — and says so.
        assert_eq!(plan_intent(Some("unknown")), None);
        assert_eq!(plan_intent(Some("Unknown")), None);
        assert_eq!(plan_intent(Some("unknown_session")), None);

        let basis = comparison_basis(&[plan_intent(Some("unknown")), plan_intent(Some("easy"))]);
        assert_eq!(basis, ComparisonBasis::Distance);
    }
}
