//! `compare_runs` — two to five runs on one distance axis.
//!
//! The comparison the app draws is a distance-normalised profile: each run
//! is resampled onto the same percentage of its own distance, so the pace,
//! heart rate and cadence curves can be read against each other. The tool
//! hands that profile over, resampled to the number of points the caller
//! asked for.
//!
//! Two runs are only comparable if they are comparable. The answer says how
//! far apart the distances were and what is missing from each run, so a
//! model does not read a difference in the curves as a difference in fitness.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::{Envelope, MAX_OUTPUT_BYTES};
use crate::schema;
use crate::tools::common::{self};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "compare_runs";

/// What the tool does.
pub const DESCRIPTION: &str = "Put two to five known runs side by side on one distance axis: \
each run's pace, heart rate, cadence and elevation are resampled onto the same share of its \
distance, so the curves can be read against each other. The runs are compared as they are — \
different terrain, duration or effort are not adjusted for, and the answer says how far apart \
the distances were.";

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "activity_ids": schema::array(
                schema::integer("A run's id.", None, None),
                "The runs to compare: two to five ids, all different.",
            ),
            "points": schema::integer(
                "How many points each run's profile is resampled to, 20 to 240. Defaults to 120.",
                Some(20),
                Some(240),
            ),
            "metrics": schema::array(
                schema::string("A parameter name."),
                "Which comparison parameters to report, by the name the API uses: \"distance_m\", \
        \"moving_time_s\", \"pace_s_per_km\", \"elevation_gain_m\", \"avg_hr\", \"hr_drift_bpm\", \
        \"pace_drift_pct\", \"decoupling_pct\", \"cadence_avg\", \"cadence_cv\", \"efficiency_first\", \
        \"form_score\", \"running_time_s\", \"walking_time_s\", \"stopped_time_s\", \"avg_speed_ms\". \
        Defaults to all of them.",
            ),
        }),
        &["activity_ids"],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, _ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["activity_ids", "points", "metrics"])?;
    let raw = args
        .value()
        .get("activity_ids")
        .cloned()
        .unwrap_or(Value::Null);
    let ids = match raw {
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_i64().ok_or_else(|| {
                    McpError::invalid(NAME, "activity_ids", "must be a list of run ids")
                })
            })
            .collect::<Result<Vec<i64>, McpError>>()?,
        _ => {
            return Err(McpError::invalid(
                NAME,
                "activity_ids",
                "must be a list of run ids",
            ))
        }
    };
    if ids.len() < 2 || ids.len() > 5 {
        return Err(McpError::invalid(
            NAME,
            "activity_ids",
            "must hold two to five run ids",
        ));
    }
    let mut unique: Vec<i64> = Vec::new();
    for id in &ids {
        if unique.contains(id) {
            return Err(McpError::invalid(
                NAME,
                "activity_ids",
                format!("the run {id} is listed twice"),
            ));
        }
        unique.push(*id);
    }
    let points = args.bounded_i64("points", 120, 20, 240)? as usize;
    let metrics = args.str_list("metrics")?;

    let path = format!(
        "/api/v1/compare?{}",
        common::query(&[(
            "ids",
            Some(
                unique
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        )])
    );
    let value = common::get(api, NAME, &path)?;

    let parameters = value
        .get("parameters")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let allowed: Vec<String> = parameters
        .iter()
        .filter_map(|parameter| common::str_of(parameter, "key"))
        .map(str::to_string)
        .collect();

    let runs = value
        .get("runs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    // The runs are rendered at the number of points the caller asked for,
    // and the number is reduced until the answer fits the bound. The points
    // are the bulk of a comparison, and every run must carry the same
    // number of them: the point of the comparison is that the runs sit on
    // one distance axis.
    let render = |points: usize| -> Vec<Value> {
        runs.iter()
            .map(|run| {
                let mut item = json!({
                    "id": common::i64_of(run, "id"),
                    "name": common::str_of(run, "name"),
                    "local_date": common::str_of(run, "local_date"),
                    "distance_m": common::f64_of(run, "distance_m"),
                    "moving_time_s": common::f64_of(run, "moving_time_s"),
                    "pace_s_per_km": common::f64_of(run, "pace_s_per_km"),
                    "avg_hr": common::f64_of(run, "avg_hr"),
                    "elevation_gain_m": common::f64_of(run, "elevation_gain_m"),
                    "has_geo": common::bool_of(run, "has_geo"),
                    "values": values_of(run, &allowed, &metrics),
                    "profile": resample(
                        run.get("profile").and_then(Value::as_array).cloned().unwrap_or_default(),
                        points,
                    ),
                });
                if item["name"].is_null() {
                    item["name"] = json!("");
                }
                item
            })
            .collect()
    };

    let requested_points = points;
    let mut points = points;
    let mut rendered = render(points);
    while crate::output::size(&Value::Array(rendered.clone())) > MAX_OUTPUT_BYTES / 2 && points > 20
    {
        points /= 2;
        rendered = render(points);
    }

    let missing = value
        .get("missing")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let distances: Vec<f64> = rendered
        .iter()
        .filter_map(|run| common::f64_of(run, "distance_m"))
        .collect();
    let spread = match (
        distances
            .iter()
            .copied()
            .fold(None, |low: Option<f64>, value| {
                Some(match low {
                    Some(low) => low.min(value),
                    None => value,
                })
            }),
        distances
            .iter()
            .copied()
            .fold(None, |high: Option<f64>, value| {
                Some(match high {
                    Some(high) => high.max(value),
                    None => value,
                })
            }),
    ) {
        (Some(low), Some(high)) => Some(common::round_to(high - low, 1)),
        _ => None,
    };

    let mut limitations: Vec<String> = Vec::new();
    if !missing.is_empty() {
        limitations.push(format!(
            "{} of the runs asked for are not in the database.",
            missing.len()
        ));
    }
    // The router sends a profile at its own resolution. When the caller asks
    // for another, the profile is resampled here, and that is not the same
    // thing as a profile the router computed at that resolution.
    let source_points = runs
        .iter()
        .filter_map(|run| run.get("profile").and_then(Value::as_array))
        .map(|profile| profile.len())
        .max()
        .unwrap_or(0);
    if source_points > 0 && source_points != points {
        limitations.push(format!(
            "the profile was resampled from the router's {source_points}-point profile by linear \
interpolation on the share of the distance; a profile the router computed at {points} points \
would differ by the interpolation error."
        ));
    }
    if let Some(spread) = spread {
        if spread > 0.0 {
            limitations.push(format!(
                "the runs differ by {spread:.0} m in distance; the profiles are aligned on the \
share of each run's own distance, not on the same ground."
            ));
        }
    }
    let without_profile = rendered
        .iter()
        .filter(|run| {
            run.get("profile")
                .and_then(Value::as_array)
                .map(|profile| profile.is_empty())
                .unwrap_or(true)
        })
        .count();
    if without_profile > 0 {
        limitations.push(format!(
            "{without_profile} of the runs have no streams, so their profile is empty."
        ));
    }

    let data = json!({
        "runs": rendered,
        "parameters": parameters,
        "missing": missing,
        "profile_points": points,
    });
    if points < requested_points {
        limitations.push(format!(
            "the profiles carry {points} points each rather than the {requested_points} asked \
for, so that the answer stays inside the harness's budget."
        ));
    }

    let mut envelope = Envelope::new(data)
        .coverage("runs_requested", unique.len())
        .coverage("runs_returned", rendered.len())
        .coverage("profile_points", points)
        .coverage("profile_points_requested", requested_points)
        .coverage("source_profile_points", source_points)
        .coverage("distance_spread_m", spread)
        .coverage("attributed_to", "the runs named in activity_ids");
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// The comparison values of one run, limited to what the caller asked for.
fn values_of(run: &Value, allowed: &[String], wanted: &[String]) -> Value {
    let Some(values) = run.get("values").and_then(Value::as_object) else {
        return Value::Null;
    };
    let mut out = serde_json::Map::new();
    for (key, value) in values {
        if !allowed.is_empty() && !allowed.contains(key) {
            continue;
        }
        if !wanted.is_empty() && !wanted.iter().any(|name| name == key) {
            continue;
        }
        out.insert(key.clone(), value.clone());
    }
    Value::Object(out)
}

/// Resample a run's profile onto `points` points.
///
/// The profile the API returns is already evenly spaced over the run's
/// distance, so the resampling is a linear interpolation on the percentage
/// of the distance. A value that is missing at either end stays missing.
///
/// When the caller asks for the number of points the API already sent, the
/// profile is handed back untouched: the router's numbers are not rounded
/// on the way through.
fn resample(profile: Vec<Value>, points: usize) -> Vec<Value> {
    if profile.is_empty() || points < 2 {
        return Vec::new();
    }
    let length = profile.len();
    if length == points {
        return profile;
    }
    let mut out = Vec::with_capacity(points);
    for index in 0..points {
        let fraction = index as f64 / (points - 1) as f64;
        let position = fraction * (length - 1) as f64;
        let low = position.floor() as usize;
        let high = position.ceil() as usize;
        let weight = position - low as f64;
        let entry = |key: &str| -> Value {
            let low_value = profile[low].get(key).and_then(Value::as_f64);
            let high_value = profile[high].get(key).and_then(Value::as_f64);
            match (low_value, high_value) {
                (Some(low_value), Some(high_value)) => {
                    json!(common::round_to(
                        low_value + (high_value - low_value) * weight,
                        2
                    ))
                }
                _ => Value::Null,
            }
        };
        let distance = entry("distance_m");
        out.push(json!({
            "distance_m": distance,
            "pct": common::round_to(fraction * 100.0, 2),
            "pace_s_per_km": entry("pace_s_per_km"),
            "hr": entry("hr"),
            "cadence": entry("cadence"),
            "elevation": entry("elevation"),
        }));
    }
    out
}
