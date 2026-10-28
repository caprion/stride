//! `fingerprint_trends` — the mechanics, over time.
//!
//! The fingerprint is the derived view of a run: cadence, its variability,
//! drift, decoupling, efficiency, stride length, the form score. The tool
//! groups those numbers the way the caller asks — per run, per week, or for
//! the long runs only — and gives the median of each group, because one run
//! is not a trend.
//!
//! A group with one run in it is one observation. The answer says how many
//! runs each group holds, and the trend of a metric is only reported when
//! there are enough of them.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::Envelope;
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "fingerprint_trends";

/// What the tool does.
pub const DESCRIPTION: &str = "The derived mechanics of the athlete's runs over a date range: \
cadence, cadence stability, cadence drift, aerobic decoupling, efficiency, pace variability, \
stride length and the form score. The runs can be grouped per run, per week, or to the long runs \
only. Each group carries the median of the metric and how many runs it covers; the trend is \
reported only when there are enough runs to have one.";

/// The groups.
const GROUPS: &[&str] = &["run", "week", "long_run"];

/// The names the caller may ask for, and the fingerprint column they read.
const METRIC_ALIASES: &[(&str, &str)] = &[
    ("cadence_stability", "cadence_cv"),
    ("cadence_drift", "cadence_drift_pct"),
    ("decoupling", "decoupling_pct"),
    ("efficiency", "efficiency_first"),
    ("pace_variability", "pace_cv"),
    ("negative_split", "negative_split_pct"),
    ("stride_length", "stride_length_cm"),
    ("cadence", "cadence_avg"),
    ("hr_drift", "hr_drift_pct"),
    ("form", "form_score"),
];

/// The metrics that are used when the caller names none.
const DEFAULT_METRICS: &[&str] = &[
    "cadence_stability",
    "decoupling",
    "efficiency",
    "pace_variability",
];

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "from": schema::date("The first day of the range, inclusive. Defaults to twelve weeks ago."),
            "to": schema::date("The last day of the range, inclusive. Defaults to today."),
            "metrics": schema::array(
                schema::string("A metric name."),
                "Which metrics to report: \"cadence\", \"cadence_stability\", \"cadence_drift\", \
\"decoupling\", \"efficiency\", \"pace_variability\", \"negative_split\", \"stride_length\", \
\"hr_drift\", \"form\", \"zones\". Defaults to cadence stability, decoupling, efficiency and pace \
variability.",
            ),
            "group_by": schema::enumeration(
                GROUPS,
                "Whether the series is per run, per week, or over the long runs only. Defaults to \
\"week\".",
            ),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["from", "to", "metrics", "group_by"])?;
    let to = args.date("to")?.unwrap_or(ctx.today);
    let from = args
        .date("from")?
        .unwrap_or_else(|| ctx.today - chrono::Duration::days(83));
    if from > to {
        return Err(McpError::invalid(NAME, "from", "must not be after to"));
    }
    let group_by = args
        .enum_str("group_by", GROUPS)?
        .unwrap_or_else(|| "week".to_string());
    let wanted = {
        let named = args.str_list("metrics")?;
        if named.is_empty() {
            DEFAULT_METRICS.iter().map(|name| (*name).to_string()).collect()
        } else {
            named
        }
    };
    let mut columns: Vec<String> = Vec::new();
    let mut wants_zones = false;
    for name in &wanted {
        if name == "zones" {
            wants_zones = true;
            continue;
        }
        let column = column_of(name);
        if !columns.contains(&column) {
            columns.push(column);
        }
    }

    let range = common::query(&[
        ("from", Some(common::date_text(from))),
        ("to", Some(common::date_text(to))),
    ]);
    let fingerprints = items(&common::get(
        api,
        NAME,
        &format!("/api/v1/fitness/fingerprint?{range}"),
    )?);

    // The long runs are the ones worth looking at when the question is about
    // aerobic form; the threshold is the distance of the longer of them.
    let mut long_ids: Vec<i64> = Vec::new();
    let mut threshold: Option<f64> = None;
    if group_by == "long_run" {
        let path = format!(
            "/api/v1/activities?{}",
            common::query(&[
                ("from", Some(common::date_text(from))),
                ("to", Some(common::date_text(to))),
                ("per_page", Some("200".to_string())),
            ])
        );
        let activities = items(&common::get(api, NAME, &path)?);
        let mut distances: Vec<f64> = activities
            .iter()
            .map(|item| common::f64_of(item, "distance_m").unwrap_or(0.0))
            .filter(|distance| *distance > 0.0)
            .collect();
        distances.sort_by(|left, right| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(position) = distances.len().checked_sub(4) {
            let candidate = distances[position];
            let floor = 10_000.0;
            threshold = Some(candidate.max(floor));
        }
        if let Some(threshold) = threshold {
            long_ids = activities
                .iter()
                .filter(|item| {
                    common::f64_of(item, "distance_m").unwrap_or(0.0) >= threshold
                })
                .filter_map(|item| common::i64_of(item, "id"))
                .collect();
        }
    }

    // The groups.
    let mut groups: Vec<(String, Vec<&Value>)> = Vec::new();
    for item in &fingerprints {
        let date = common::str_of(item, "local_date").unwrap_or_default();
        if date.is_empty() {
            continue;
        }
        let activity_id = common::i64_of(item, "activity_id").unwrap_or(0);
        if group_by == "long_run" && !long_ids.contains(&activity_id) {
            continue;
        }
        let period = match group_by.as_str() {
            "run" => date.to_string(),
            _ => common::week_key(date),
        };
        match groups.iter_mut().find(|(key, _)| key == &period) {
            Some((_, runs)) => runs.push(item),
            None => groups.push((period, vec![item])),
        }
    }
    groups.sort_by(|left, right| left.0.cmp(&right.0));

    let series: Vec<Value> = groups
        .iter()
        .map(|(period, runs)| {
            let mut metrics = serde_json::Map::new();
            for column in &columns {
                let values: Vec<f64> = runs
                    .iter()
                    .filter_map(|item| common::f64_of(item, column))
                    .filter(|value| value.is_finite())
                    .collect();
                let median = common::median(&values);
                metrics.insert(
                    column.clone(),
                    json!({
                        "median": median.map(|value| common::round_to(value, 2)),
                        "runs": values.len(),
                    }),
                );
            }
            let sample: Vec<Value> = runs
                .iter()
                .filter_map(|item| common::i64_of(item, "activity_id"))
                .take(5)
                .map(|id| json!(id))
                .collect();
            json!({
                "period": period,
                "runs": runs.len(),
                "metrics": Value::Object(metrics),
                "sample_activity_ids": sample,
            })
        })
        .collect();

    let mut limitations: Vec<String> = Vec::new();

    // The trend of each metric over the whole range, from the router's own
    // trend endpoint.
    let mut trends = serde_json::Map::new();
    let mut without_trend: Vec<String> = Vec::new();
    for column in &columns {
        let path = format!("/api/v1/fitness/trends?{range}&metric={column}");
        match common::get(api, NAME, &path) {
            Ok(value) => {
                trends.insert(
                    column.clone(),
                    json!({
                        "baseline": common::f64_of(&value, "baseline"),
                        "trend_pct": common::f64_of(&value, "trend_pct"),
                        "label": common::str_of(&value, "label"),
                        "unit": common::str_of(&value, "unit"),
                        "better": common::str_of(&value, "better"),
                        "points": value
                            .get("points")
                            .and_then(Value::as_array)
                            .map(|points| points.len())
                            .unwrap_or(0),
                    }),
                );
            }
            Err(_) => without_trend.push(column.clone()),
        }
    }

    // The zones of the range, when they were asked for.
    let zones = if wants_zones {
        let path = format!("/api/v1/fitness/zones?{range}");
        match common::get(api, NAME, &path) {
            Ok(value) => json!({
                "zones": value.get("zones").cloned().unwrap_or(Value::Null),
                "total_time_s": common::f64_of(&value, "total_time_s"),
                "method": "the zone model of the athlete, with the run's stored zones where the run has them",
            }),
            Err(_) => {
                limitations.push("the zones of the range could not be read".to_string());
                Value::Null
            }
        }
    } else {
        Value::Null
    };

    if !without_trend.is_empty() {
        limitations.push(format!(
            "the trend of {} is not read by the API, so only the median is given.",
            without_trend.join(", ")
        ));
    }
    if group_by == "long_run" {
        match threshold {
            Some(threshold) => limitations.push(format!(
                "the long runs are the runs of at least {threshold:.0} m in the range."
            )),
            None => limitations.push(
                "the range holds too few runs to tell a long run from a short one".to_string(),
            ),
        }
    }
    if groups.len() == 1 {
        limitations.push(
            "the range holds one group, so no trend can be read from it".to_string(),
        );
    }

    let data = json!({
        "group_by": group_by,
        "series": series,
        "trends": Value::Object(trends),
        "zones": zones,
    });

    let mut envelope = Envelope::new(data)
        .coverage("from", common::date_text(from))
        .coverage("to", common::date_text(to))
        .coverage("groups", groups.len())
        .coverage("fingerprints", fingerprints.len())
        .coverage("metrics", columns.join(","))
        .coverage("attributed_to", "the fingerprints of the athlete's own runs");
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// The fingerprint column a metric name reads.
fn column_of(name: &str) -> String {
    for (alias, column) in METRIC_ALIASES {
        if *alias == name {
            return (*column).to_string();
        }
    }
    name.to_string()
}
