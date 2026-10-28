//! `get_activity` — one run, in full.
//!
//! The route and the per-second streams are the two things a harness can
//! blow its context on, and the route is the athlete's location. So both are
//! opt-in, and when they are asked for they are downsampled to a bounded
//! number of points.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::{pick, sample_indices, Envelope, MAX_ITEMS, MAX_SAMPLES};
use crate::schema;
use crate::tools::common::{self};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "get_activity";

/// What the tool does.
pub const DESCRIPTION: &str = "One run in detail: the summary, and by request its splits, its \
per-second streams, its heart-rate zones and its route. The splits and the zones are included by \
default; the streams and the route are only returned when they are asked for, and both are \
downsampled. The route is the athlete's location.";

/// What may be included.
const INCLUDES: &[&str] = &["laps", "streams", "zones", "route"];

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "activity_id": schema::integer("The run's id.", None, None),
            "include": schema::enumeration_array(
                INCLUDES,
                "What to include: \"laps\", \"streams\", \"zones\", \"route\". Defaults to the \
        laps and the zones.",
            ),
        }),
        &["activity_id"],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, _ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["activity_id", "include"])?;
    let activity_id = args.req_i64("activity_id")?;
    if activity_id < 1 {
        return Err(McpError::invalid(NAME, "activity_id", "must be at least 1"));
    }
    let include = {
        let wanted = args.enum_list("include", INCLUDES)?;
        if wanted.is_empty() {
            vec!["laps".to_string(), "zones".to_string()]
        } else {
            wanted
        }
    };
    let wants = |what: &str| include.iter().any(|name| name == what);

    let path = format!("/api/v1/activities/{activity_id}");
    let value = common::get(api, NAME, &path)?;

    let mut data = json!({
        "activity": pick(&value, common::ACTIVITY_DETAIL_FIELDS),
    });

    let mut limitations: Vec<String> = Vec::new();

    // ------------------------------------------------------------- the laps
    let laps = value
        .get("laps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let lap_count = laps.len();
    if wants("laps") {
        let shown: Vec<Value> = laps
            .iter()
            .take(MAX_ITEMS)
            .map(|lap| {
                json!({
                    "lap_index": common::i64_of(lap, "lap_index"),
                    "distance_m": common::f64_of(lap, "distance_m"),
                    "duration_s": common::f64_of(lap, "duration_s"),
                    "pace_s_per_km": common::f64_of(lap, "pace_s_per_km"),
                    "avg_hr": common::f64_of(lap, "avg_hr"),
                    "max_hr": common::f64_of(lap, "max_hr"),
                    "avg_cadence": common::f64_of(lap, "avg_cadence"),
                    "elevation_gain_m": common::f64_of(lap, "elevation_gain_m"),
                    "elevation_loss_m": common::f64_of(lap, "elevation_loss_m"),
                    "start_elevation_m": common::f64_of(lap, "start_elevation_m"),
                    "end_elevation_m": common::f64_of(lap, "end_elevation_m"),
                    "running_distance_m": common::f64_of(lap, "running_distance_m"),
                })
            })
            .collect();
        data["laps"] = Value::Array(shown);
        if lap_count > MAX_ITEMS {
            limitations.push(format!(
                "the run has {lap_count} splits and only the first {MAX_ITEMS} are returned."
            ));
        }
    }

    // ---------------------------------------------------------- the streams
    let streams = value.get("streams").and_then(Value::as_object).cloned();
    let stream_kinds: Vec<String> = streams
        .as_ref()
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    if wants("streams") {
        let mut rendered = serde_json::Map::new();
        for kind in ["hr", "cadence", "pace", "distance", "elevation"] {
            let Some(stream) = streams.as_ref().and_then(|map| map.get(kind)) else {
                continue;
            };
            let seconds = stream
                .get("seconds")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let values = stream
                .get("values")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let length = seconds.len().min(values.len());
            let indices = sample_indices(length, MAX_SAMPLES);
            let mut samples: Vec<Value> = Vec::with_capacity(indices.len());
            let mut skipped = 0usize;
            for index in indices {
                let Some(value) = values.get(index) else {
                    continue;
                };
                if value.is_null() {
                    skipped += 1;
                    continue;
                }
                samples.push(json!({
                    "t": seconds.get(index).cloned().unwrap_or(Value::Null),
                    "value": value,
                }));
            }
            if skipped > 0 {
                limitations.push(format!(
                    "the {kind} series has {skipped} samples with no measurement; they are not \
shown and are not counted as zero."
                ));
            }
            rendered.insert(
                kind.to_string(),
                json!({
                    "unit": stream.get("unit").cloned().unwrap_or(Value::Null),
                    "samples": samples,
                }),
            );
        }
        data["streams"] = Value::Object(rendered);
    }

    // ------------------------------------------------------------ the zones
    let zones = value
        .get("hr_zones")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if wants("zones") {
        data["zones"] = Value::Array(
            zones
                .iter()
                .map(|zone| {
                    json!({
                        "zone_no": common::i64_of(zone, "zone_no"),
                        "name": common::str_of(zone, "name"),
                        "low_bpm": common::f64_of(zone, "low_bpm"),
                        "high_bpm": common::f64_of(zone, "high_bpm"),
                        "time_s": common::f64_of(zone, "time_s"),
                    })
                })
                .collect(),
        );
    }

    // ------------------------------------------------------------ the route
    let points = value
        .get("geo_points")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if wants("route") {
        let indices = sample_indices(points.len(), MAX_SAMPLES);
        let sampled: Vec<Value> = indices
            .iter()
            .filter_map(|index| points.get(*index))
            .map(|point| {
                let lat = point.get(0).and_then(Value::as_f64).unwrap_or(0.0);
                let lon = point.get(1).and_then(Value::as_f64).unwrap_or(0.0);
                json!([common::round_to(lat, 5), common::round_to(lon, 5)])
            })
            .collect();
        data["route"] = json!({
            "points": sampled,
            "point_count": points.len(),
        });
        if points.len() > MAX_SAMPLES {
            limitations.push(format!(
                "the route has {} points and was downsampled to {MAX_SAMPLES}.",
                points.len()
            ));
        }
    }

    let mut envelope = Envelope::new(data)
        .coverage("activity_id", activity_id)
        .coverage("activity_date", common::str_of(&value, "local_date"))
        .coverage("has_laps", lap_count > 0)
        .coverage("laps", lap_count)
        .coverage("stream_kinds", stream_kinds.join(","))
        .coverage("zones", zones.len())
        .coverage("has_route", !points.is_empty())
        .coverage("attributed_to", format!("activity {activity_id}"));
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}
