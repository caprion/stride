//! `training_statistics` — the trends.
//!
//! The aggregates come from the router's own analytics, so the numbers here
//! are the numbers the app shows. The tool adds the two things the app's
//! shapes do not carry: the date range the series covers, and the method
//! behind the heart-rate zones — which are estimated from the athlete's
//! observed maximum, not read from Garmin's own zone model.

use chrono::Datelike;
use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::{Envelope, MAX_ITEMS};
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "training_statistics";

/// What the tool does.
pub const DESCRIPTION: &str = "Aggregate trends over a date range: the volume of each week or \
month, the pace, the elevation, the heart-rate zone distribution and the personal records. The \
zone times are estimated from the athlete's observed maximum heart rate, not from Garmin's own \
zone model, unless the run itself carries stored zones.";

/// What the caller can ask for.
const METRICS: &[&str] = &["volume", "pace", "zones", "records", "elevation"];

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "from": schema::date("The first day of the range, inclusive. Defaults to twelve weeks ago."),
            "to": schema::date("The last day of the range, inclusive. Defaults to today."),
            "group_by": schema::enumeration(
                &["week", "month"],
                "Whether the series is by week or by month. Defaults to week.",
            ),
            "metrics": schema::enumeration_array(
                METRICS,
                "What to include: \"volume\", \"pace\", \"zones\", \"records\", \"elevation\". \
Defaults to all of them.",
            ),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["from", "to", "group_by", "metrics"])?;
    let to = args.date("to")?.unwrap_or(ctx.today);
    let from = args
        .date("from")?
        .unwrap_or_else(|| ctx.today - chrono::Duration::days(83));
    if from > to {
        return Err(McpError::invalid(NAME, "from", "must not be after to"));
    }
    let group_by = args
        .enum_str("group_by", &["week", "month"])?
        .unwrap_or_else(|| "week".to_string());
    let metrics = {
        let wanted = args.enum_list("metrics", METRICS)?;
        if wanted.is_empty() {
            METRICS.iter().map(|name| (*name).to_string()).collect()
        } else {
            wanted
        }
    };
    let wants = |name: &str| metrics.iter().any(|metric| metric == name);

    let mut limitations: Vec<String> = Vec::new();

    // ------------------------------------------------------------- the series
    let mut series: Vec<Value> = Vec::new();
    if group_by == "week" {
        let weeks = ((to - from).num_days() / 7 + 2).clamp(1, 200);
        if weeks == 200 && (to - from).num_days() / 7 + 2 > 200 {
            limitations.push(
                "the range is longer than the API's 200 weeks; the oldest weeks are not in the \
series."
                    .to_string(),
            );
        }
        let path = format!("/api/v1/stats/weekly?weeks={weeks}");
        for item in items(&common::get(api, NAME, &path)?) {
            let start = common::str_of(&item, "start_date").unwrap_or_default();
            let end = common::str_of(&item, "week_key").unwrap_or_default();
            let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").ok();
            let in_range = start_date.is_some_and(|date| date >= from && date <= to);
            if !in_range {
                continue;
            }
            series.push(json!({
                "period": end,
                "start_date": start,
                "run_count": common::i64_of(&item, "activity_count"),
                "distance_m": common::f64_of(&item, "distance_m"),
                "elevation_gain_m": common::f64_of(&item, "elevation_gain_m"),
                "moving_time_s": common::f64_of(&item, "moving_time_s"),
                "avg_pace_s_per_km": common::f64_of(&item, "avg_pace_s_per_km"),
                "avg_hr": common::f64_of(&item, "avg_hr"),
            }));
        }
    } else {
        let months = ((to.year() - from.year()) as i64 * 12
            + (to.month() as i64 - from.month() as i64)
            + 2)
            .clamp(1, 60);
        let path = format!("/api/v1/stats/monthly?months={months}");
        for item in items(&common::get(api, NAME, &path)?) {
            let key = common::str_of(&item, "month_key").unwrap_or_default();
            let first_month = from.format("%Y-%m").to_string();
            let last_month = to.format("%Y-%m").to_string();
            let in_range = key >= first_month.as_str() && key <= last_month.as_str();
            if !in_range {
                continue;
            }
            series.push(json!({
                "period": key,
                "start_date": Value::Null,
                "run_count": common::i64_of(&item, "activity_count"),
                "distance_m": common::f64_of(&item, "distance_m"),
                "elevation_gain_m": common::f64_of(&item, "elevation_gain_m"),
                "moving_time_s": common::f64_of(&item, "moving_time_s"),
                "avg_pace_s_per_km": common::f64_of(&item, "avg_pace_s_per_km"),
                "avg_hr": common::f64_of(&item, "avg_hr"),
            }));
        }
    }

    let runs: i64 = series
        .iter()
        .filter_map(|item| common::i64_of(item, "run_count"))
        .sum();
    let distance: f64 = series
        .iter()
        .filter_map(|item| common::f64_of(item, "distance_m"))
        .sum();
    let elevation: f64 = series
        .iter()
        .filter_map(|item| common::f64_of(item, "elevation_gain_m"))
        .sum();
    let moving: f64 = series
        .iter()
        .filter_map(|item| common::f64_of(item, "moving_time_s"))
        .sum();
    let weighted_pace = if distance > 0.0 {
        Some(common::round_to(moving / (distance / 1000.0), 2))
    } else {
        None
    };

    // ------------------------------------------------------------ the zones
    let mut zones = Value::Null;
    if wants("zones") {
        let weeks = ((to - from).num_days() / 7 + 2).clamp(1, 104);
        let path = format!("/api/v1/stats/hr-zones?weeks={weeks}");
        let value = common::get(api, NAME, &path)?;
        let list = value
            .get("zones")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        zones = json!({
            "zones": list
                .iter()
                .map(|zone| json!({
                    "zone_no": common::i64_of(zone, "zone_no"),
                    "name": common::str_of(zone, "name"),
                    "low_bpm": common::f64_of(zone, "low_bpm"),
                    "high_bpm": common::f64_of(zone, "high_bpm"),
                    "time_s": common::f64_of(zone, "time_s"),
                    "percent": common::f64_of(zone, "percent"),
                    "method": "estimated",
                }))
                .collect::<Vec<Value>>(),
            "total_time_s": common::f64_of(&value, "total_time_s"),
            "peak_hr": common::f64_of(&value, "peak_hr"),
            "method": "estimated",
        });
        limitations.push(
            "the zone times are estimated from the observed maximum heart rate, not read from \
Garmin's own zone model."
                .to_string(),
        );
    }

    // ---------------------------------------------------------- the records
    let mut records = Value::Null;
    if wants("records") {
        let value = common::get(api, NAME, "/api/v1/stats/personal-records")?;
        let list = items(&value);
        records = Value::Array(
            list.iter()
                .take(MAX_ITEMS)
                .map(|record| {
                    json!({
                        "label": common::str_of(record, "label"),
                        "value": common::f64_of(record, "value"),
                        "unit": common::str_of(record, "unit"),
                        "better": common::str_of(record, "better"),
                        "activity_id": common::i64_of(record, "activity_id"),
                        "activity_name": common::str_of(record, "activity_name"),
                        "activity_date": common::str_of(record, "activity_date"),
                    })
                })
                .collect(),
        );
        limitations.push(
            "the records are the best of the whole history, not of the range that was asked for."
                .to_string(),
        );
    }

    let data = json!({
        "totals": {
            "runs": runs,
            "distance_m": common::round_to(distance, 1),
            "elevation_gain_m": common::round_to(elevation, 1),
            "moving_time_s": common::round_to(moving, 1),
            "pace_s_per_km": weighted_pace,
        },
        "series": series,
        "hr_zones": zones,
        "records": records,
    });

    let mut envelope = Envelope::new(data)
        .coverage("from", common::date_text(from))
        .coverage("to", common::date_text(to))
        .coverage("group_by", group_by.clone())
        .coverage("periods", series.len())
        .coverage("runs", runs)
        .coverage("attributed_to", "the activity and split tables of the athlete's own database");
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}
