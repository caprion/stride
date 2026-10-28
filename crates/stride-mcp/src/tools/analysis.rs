//! `activity_analysis` — what actually happened in one run.
//!
//! The tool reports observations, not causes. A run that slowed down on the
//! hills is described as "the splits with a grade of 3% or more were slower
//! than the flatter ones", and never as "the hills made you slow down",
//! because the data cannot establish that: without a route and a grade for
//! every split, the hill and the slowdown are two facts that happened at the
//! same time.
//!
//! The evidence is in the answer: every finding carries the numbers it came
//! from and how much of the run those numbers cover.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::{pick, Envelope, MAX_SEGMENTS};
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "activity_analysis";

/// What the tool does.
pub const DESCRIPTION: &str = "Explain one run: the splits as segments, what the numbers show \
about pacing, hills, effort and form, and — when asked — a comparison with a similar run, the \
run's best comparable effort, or a run the caller names. The findings are observations: a slower \
split on a climb is reported as a coincidence with the climb, not as a cause. Missing evidence is \
listed rather than guessed at.";

/// The things the caller can ask about.
const FOCUSES: &[&str] = &["pacing", "hills", "effort", "form"];

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "activity_id": schema::integer("The run's id.", None, None),
            "compare_with": schema::string(
                "What to compare the run with: \"similar_recent\" for the most recent runs of a \
similar length, \"previous_best\" for the fastest comparable run, or \"activity_id:<id>\" for one \
named run.",
            ),
            "focus": schema::enumeration_array(
                FOCUSES,
                "Which parts of the analysis to keep: \"pacing\", \"hills\", \"effort\", \"form\". \
Defaults to all of them.",
            ),
        }),
        &["activity_id"],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, _ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["activity_id", "compare_with", "focus"])?;
    let activity_id = args.req_i64("activity_id")?;
    if activity_id < 1 {
        return Err(McpError::invalid(NAME, "activity_id", "must be at least 1"));
    }
    let compare_with = args.opt_str("compare_with")?.map(str::to_string);
    let focus = args.enum_list("focus", FOCUSES)?;

    let path = format!("/api/v1/activities/{activity_id}");
    let activity = common::get(api, NAME, &path)?;

    // The fingerprint is what carries the form and the decoupling. A run
    // without one is not an error — it is a run with less evidence.
    let fingerprint = match common::get(api, NAME, &format!("/api/v1/fitness/activity/{activity_id}"))
    {
        Ok(value) => Some(value),
        Err(McpError::Api { status: 404, .. }) => None,
        Err(error) => return Err(error),
    };

    let laps = activity
        .get("laps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut missing: Vec<String> = Vec::new();
    if laps.is_empty() {
        missing.push("the run has no splits, so it cannot be analysed segment by segment".into());
    }
    if fingerprint.is_none() {
        missing.push("the run has no fingerprint, so form and decoupling are unknown".into());
    }
    let has_grade = laps.iter().any(|lap| {
        common::f64_of(lap, "start_elevation_m").is_some()
            && common::f64_of(lap, "end_elevation_m").is_some()
    });
    if !has_grade && !laps.is_empty() {
        missing.push(
            "the splits carry no elevation, so the grade of the terrain is unknown".into(),
        );
    }
    let has_hr = laps
        .iter()
        .any(|lap| common::f64_of(lap, "avg_hr").is_some());
    if !has_hr && !laps.is_empty() {
        missing.push("the splits carry no heart rate, so effort cannot be read".into());
    }

    let segments = build_segments(&laps);
    let signals = build_signals(&laps, fingerprint.as_ref(), &focus);

    let comparison = match compare_with.as_deref() {
        None => None,
        Some(basis) => Some(compare(api, &activity, activity_id, basis)?),
    };

    let shown_segments: Vec<Value> = segments
        .iter()
        .take(MAX_SEGMENTS)
        .map(segment_value)
        .collect();

    let mut data = json!({
        "run": pick(&activity, common::ACTIVITY_DETAIL_FIELDS),
        "segments": shown_segments,
        "signals": signals,
        "missing_data": missing,
    });
    if let Some(comparison) = comparison {
        data["comparison"] = comparison;
    }

    let mut envelope = Envelope::new(data)
        .coverage("activity_id", activity_id)
        .coverage("activity_date", common::str_of(&activity, "local_date"))
        .coverage("splits", laps.len())
        .coverage("segments", shown_segments.len())
        .coverage("focus", focus.join(","))
        .coverage("attributed_to", format!("activity {activity_id}"));
    if segments.len() > MAX_SEGMENTS {
        envelope = envelope.limitation(format!(
            "the run has {} splits and only the first {MAX_SEGMENTS} are shown as segments.",
            segments.len()
        ));
    }
    Ok(envelope.finish(NAME))
}

/// One split, read as a segment.
struct Segment {
    index: i64,
    distance_m: f64,
    duration_s: f64,
    pace_s_per_km: Option<f64>,
    avg_hr: Option<f64>,
    elevation_gain_m: Option<f64>,
    grade_pct: Option<f64>,
}

impl Segment {
    /// The segment as the answer carries it.
    fn value(&self) -> Value {
        json!({
            "lap_index": self.index,
            "distance_m": common::round_to(self.distance_m, 1),
            "duration_s": common::round_to(self.duration_s, 1),
            "pace_s_per_km": self.pace_s_per_km,
            "hr": self.avg_hr,
            "elevation_gain_m": self.elevation_gain_m,
            "grade_pct": self.grade_pct,
        })
    }
}

/// Read the splits as segments.
fn build_segments(laps: &[Value]) -> Vec<Segment> {
    laps.iter()
        .map(|lap| {
            let distance = common::f64_of(lap, "distance_m").unwrap_or(0.0);
            let start = common::f64_of(lap, "start_elevation_m");
            let end = common::f64_of(lap, "end_elevation_m");
            let grade = match (start, end) {
                (Some(start), Some(end)) if distance > 0.0 => {
                    Some(common::round_to((end - start) / distance * 100.0, 1))
                }
                _ => None,
            };
            Segment {
                index: common::i64_of(lap, "lap_index").unwrap_or(0),
                distance_m: distance,
                duration_s: common::f64_of(lap, "duration_s").unwrap_or(0.0),
                pace_s_per_km: common::f64_of(lap, "pace_s_per_km"),
                avg_hr: common::f64_of(lap, "avg_hr"),
                elevation_gain_m: common::f64_of(lap, "elevation_gain_m"),
                grade_pct: grade,
            }
        })
        .collect()
}

/// The segment as the answer carries it.
fn segment_value(segment: &Segment) -> Value {
    segment.value()
}

/// The findings, filtered by what the caller asked about.
fn build_signals(
    laps: &[Value],
    fingerprint: Option<&Value>,
    focus: &[String],
) -> Vec<Value> {
    let wanted = |name: &str| focus.is_empty() || focus.iter().any(|item| item == name);
    let mut signals: Vec<Value> = Vec::new();

    // ------------------------------------------------------------- pacing
    if wanted("pacing") {
        if let Some((first, second, count)) = half_paces(laps) {
            let change = ((second - first) / first) * 100.0;
            let finding = if change > 2.0 {
                format!("The second half was {change:.1}% slower than the first half.")
            } else if change < -2.0 {
                format!("The second half was {}% faster than the first half.", -change)
            } else {
                "The pace was even across the two halves.".to_string()
            };
            signals.push(json!({
                "focus": "pacing",
                "finding": finding,
                "evidence": {
                    "first_half_pace_s_per_km": common::round_to(first, 2),
                    "second_half_pace_s_per_km": common::round_to(second, 2),
                    "change_pct": common::round_to(change, 1),
                    "splits_covered": count,
                },
                "confidence": if count >= 6 { "high" } else { "medium" },
            }));
        }
    }

    // -------------------------------------------------------------- hills
    if wanted("hills") {
        let steep: Vec<f64> = laps
            .iter()
            .filter(|lap| {
                common::f64_of(lap, "start_elevation_m")
                    .zip(common::f64_of(lap, "end_elevation_m"))
                    .zip(common::f64_of(lap, "distance_m"))
                    .is_some_and(|((start, end), distance)| {
                        distance > 0.0 && ((end - start) / distance * 100.0).abs() >= 3.0
                    })
            })
            .filter_map(|lap| common::f64_of(lap, "pace_s_per_km"))
            .collect();
        let flat: Vec<f64> = laps
            .iter()
            .filter(|lap| {
                common::f64_of(lap, "start_elevation_m")
                    .zip(common::f64_of(lap, "end_elevation_m"))
                    .zip(common::f64_of(lap, "distance_m"))
                    .is_some_and(|((start, end), distance)| {
                        distance > 0.0 && ((end - start) / distance * 100.0).abs() < 3.0
                    })
            })
            .filter_map(|lap| common::f64_of(lap, "pace_s_per_km"))
            .collect();
        if !steep.is_empty() {
            let steep_mean = common::mean(&steep).unwrap_or_default();
            let finding = match common::mean(&flat) {
                Some(flat_mean) => format!(
                    "{} of the splits have a grade of 3% or more. Their average pace was {:.1} \
s/km against {:.1} s/km on the flatter splits; the slower splits and the steeper ground \
coincided, and the data cannot say which caused which.",
                    steep.len(),
                    steep_mean,
                    flat_mean
                ),
                None => format!(
                    "{} of the splits have a grade of 3% or more, averaging {:.1} s/km. There are \
no flatter splits to compare them with.",
                    steep.len(),
                    steep_mean
                ),
            };
            signals.push(json!({
                "focus": "hills",
                "finding": finding,
                "evidence": {
                    "steep_splits": steep.len(),
                    "steep_mean_pace_s_per_km": common::round_to(steep_mean, 2),
                    "flat_mean_pace_s_per_km": common::mean(&flat).map(|value| common::round_to(value, 2)),
                    "grade_threshold_pct": 3.0,
                },
                "confidence": if steep.len() >= 3 && !flat.is_empty() { "medium" } else { "low" },
            }));
        }
    }

    // ------------------------------------------------------------- effort
    if wanted("effort") {
        if let Some((first, second, count)) = half_heart_rates(laps) {
            let finding = if (second - first).abs() <= 1.0 {
                format!("The average heart rate was {first:.0} bpm in both halves.")
            } else {
                format!(
                    "The average heart rate of the second half was {second:.0} bpm against \
{first:.0} bpm in the first half."
                )
            };
            signals.push(json!({
                "focus": "effort",
                "finding": finding,
                "evidence": {
                    "first_half_hr_bpm": common::round_to(first, 1),
                    "second_half_hr_bpm": common::round_to(second, 1),
                    "delta_bpm": common::round_to(second - first, 1),
                    "splits_covered": count,
                },
                "confidence": if count >= 6 { "high" } else { "medium" },
            }));
        }
        if let Some(fingerprint) = fingerprint {
            if let Some(decoupling) = common::f64_of(fingerprint, "decoupling_pct") {
                signals.push(json!({
                    "focus": "effort",
                    "finding": format!(
                        "The aerobic decoupling of the run was {decoupling:.1}% — the heart rate \
drifted relative to the pace."
                    ),
                    "evidence": {
                        "decoupling_pct": common::round_to(decoupling, 1),
                        "hr_drift_bpm": common::f64_of(fingerprint, "hr_drift_bpm"),
                        "pace_drift_pct": common::f64_of(fingerprint, "pace_drift_pct"),
                    },
                    "confidence": "medium",
                }));
            }
        }
    }

    // --------------------------------------------------------------- form
    if wanted("form") {
        if let Some(fingerprint) = fingerprint {
            let cadence = common::f64_of(fingerprint, "cadence_avg");
            let variability = common::f64_of(fingerprint, "cadence_cv");
            let stride = common::f64_of(fingerprint, "stride_length_cm");
            let form = common::f64_of(fingerprint, "form_score");
            if cadence.is_some() || form.is_some() {
                let mut finding = String::new();
                if let Some(cadence) = cadence {
                    finding.push_str(&format!(
                        "The mean cadence was {cadence:.0} steps per minute"
                    ));
                    if let Some(variability) = variability {
                        finding.push_str(&format!(
                            ", with a variability of {variability:.1}%"
                        ));
                    }
                    finding.push('.');
                }
                if let Some(stride) = stride {
                    finding.push_str(&format!(" The mean stride length was {stride:.0} cm."));
                }
                if let Some(form) = form {
                    finding.push_str(&format!(" The form score was {form:.0} of 100."));
                }
                signals.push(json!({
                    "focus": "form",
                    "finding": finding,
                    "evidence": {
                        "cadence_avg_spm": cadence,
                        "cadence_cv_pct": variability,
                        "stride_length_cm": stride,
                        "form_score": form,
                    },
                    "confidence": "medium",
                }));
            }
        }
    }

    signals
}

/// The mean pace of the first half of the splits and of the second.
fn half_paces(laps: &[Value]) -> Option<(f64, f64, usize)> {
    let paces: Vec<f64> = laps
        .iter()
        .filter_map(|lap| common::f64_of(lap, "pace_s_per_km"))
        .filter(|pace| *pace > 0.0)
        .collect();
    if paces.len() < 2 {
        return None;
    }
    let middle = paces.len() / 2;
    let first = common::mean(&paces[..middle])?;
    let second = common::mean(&paces[middle..])?;
    Some((first, second, paces.len()))
}

/// The mean heart rate of the first half of the splits and of the second.
fn half_heart_rates(laps: &[Value]) -> Option<(f64, f64, usize)> {
    let rates: Vec<f64> = laps
        .iter()
        .filter_map(|lap| common::f64_of(lap, "avg_hr"))
        .filter(|rate| *rate > 0.0)
        .collect();
    if rates.len() < 2 {
        return None;
    }
    let middle = rates.len() / 2;
    let first = common::mean(&rates[..middle])?;
    let second = common::mean(&rates[middle..])?;
    Some((first, second, rates.len()))
}

/// Compare the run with something else.
fn compare(
    api: &dyn StrideApi,
    activity: &Value,
    activity_id: i64,
    basis: &str,
) -> Result<Value, McpError> {
    if let Some(raw) = basis.strip_prefix("activity_id:") {
        let other_id = raw
            .parse::<i64>()
            .map_err(|_| McpError::invalid(NAME, "compare_with", "activity_id must be a number"))?;
        let other = common::get(api, NAME, &format!("/api/v1/activities/{other_id}"))?;
        let candidate = candidate_value(&other, activity);
        return Ok(json!({
            "basis": format!("activity_id:{other_id}"),
            "candidates": [candidate],
        }));
    }

    let basis = match basis {
        "similar_recent" | "previous_best" => basis,
        other => {
            return Err(McpError::invalid(
                NAME,
                "compare_with",
                format!(
                    "must be \"similar_recent\", \"previous_best\" or \"activity_id:<id>\", not \
{other:?}"
                ),
            ))
        }
    };

    let path = format!(
        "/api/v1/activities?{}",
        common::query(&[
            ("sort", Some("date".to_string())),
            ("per_page", Some("100".to_string())),
        ])
    );
    let all = items(&common::get(api, NAME, &path)?);
    let distance = common::f64_of(activity, "distance_m").unwrap_or(0.0);
    let similar: Vec<&Value> = all
        .iter()
        .filter(|item| common::i64_of(item, "id") != Some(activity_id))
        .filter(|item| {
            let other = common::f64_of(item, "distance_m").unwrap_or(0.0);
            if distance <= 0.0 {
                return false;
            }
            let ratio = (other - distance).abs() / distance;
            ratio <= 0.1
        })
        .collect();

    if similar.is_empty() {
        return Ok(json!({
            "basis": basis,
            "candidates": [],
            "note": "no other run of a comparable length is in the database",
        }));
    }

    let candidates: Vec<Value> = if basis == "previous_best" {
        let best = similar
            .iter()
            .filter(|item| common::pace_of(item).is_some())
            .min_by(|left, right| {
                let left = common::pace_of(left).unwrap_or(f64::MAX);
                let right = common::pace_of(right).unwrap_or(f64::MAX);
                left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();
        match best {
            Some(item) => vec![candidate_value(item, activity)],
            None => Vec::new(),
        }
    } else {
        similar
            .iter()
            .take(5)
            .map(|item| candidate_value(item, activity))
            .collect()
    };

    Ok(json!({
        "basis": basis,
        "candidates": candidates,
        "note": "the candidates are the runs within 10% of this run's distance; they are not \
matched for terrain, duration or effort",
    }))
}

/// One comparison candidate, against the analysed run.
fn candidate_value(other: &Value, activity: &Value) -> Value {
    let pace = common::pace_of(other);
    let base_pace = common::pace_of(activity);
    let distance = common::f64_of(other, "distance_m");
    let base_distance = common::f64_of(activity, "distance_m");
    let heart_rate = common::f64_of(other, "avg_hr");
    let base_heart_rate = common::f64_of(activity, "avg_hr");
    json!({
        "activity_id": common::i64_of(other, "id"),
        "name": common::str_of(other, "name"),
        "local_date": common::str_of(other, "local_date"),
        "distance_m": distance,
        "pace_s_per_km": pace,
        "avg_hr": heart_rate,
        "elevation_gain_m": common::f64_of(other, "elevation_gain_m"),
        "pace_delta_s_per_km": match (pace, base_pace) {
            (Some(pace), Some(base)) => Some(common::round_to(pace - base, 2)),
            _ => None,
        },
        "pace_delta_pct": match (pace, base_pace) {
            (Some(pace), Some(base)) if base > 0.0 => {
                Some(common::round_to((pace - base) / base * 100.0, 1))
            }
            _ => None,
        },
        "hr_delta_bpm": match (heart_rate, base_heart_rate) {
            (Some(heart_rate), Some(base)) => Some(common::round_to(heart_rate - base, 1)),
            _ => None,
        },
        "distance_delta_m": match (distance, base_distance) {
            (Some(distance), Some(base)) => Some(common::round_to(distance - base, 1)),
            _ => None,
        },
    })
}
