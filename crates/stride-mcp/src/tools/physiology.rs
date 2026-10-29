//! `physiology_context` — readiness, sleep, HRV and resting heart rate.
//!
//! The tool says what was measured and what was not. A day with no HRV has
//! `null` there, not a zero, and the answer lists the sources that have no
//! data at all — a model that does not know the watch was not worn will read
//! a low readiness as a fact about the athlete's fitness.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::Envelope;
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "physiology_context";

/// What the tool does.
pub const DESCRIPTION: &str = "Readiness, sleep, HRV and resting heart rate for a date range, \
day by day, with the drivers behind each readiness score and the athlete's own baselines. A \
measurement that was never taken is null and the missing sources are listed; a missing day is \
never shown as a zero.";

/// What the caller can ask about.
const FOCUSES: &[&str] = &["readiness", "sleep", "hrv", "resting_hr", "all"];

/// The fields the daily endpoint is asked for.
const DAILY_FIELDS: &str = "date,sleep_s,sleep_score,deep_sleep_s,rem_s,light_sleep_s,awake_s,\
resting_hr,max_hr,stress_avg,body_battery_avg,source";

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "from": schema::date("The first day of the range, inclusive. Defaults to a fortnight ago."),
            "to": schema::date("The last day of the range, inclusive. Defaults to today."),
            "focus": schema::enumeration(
                FOCUSES,
                "Which measurement to lead with: \"readiness\", \"sleep\", \"hrv\", \
        \"resting_hr\" or \"all\". Defaults to \"all\".",
            ),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["from", "to", "focus"])?;
    let to = args.date("to")?.unwrap_or(ctx.today);
    let from = args
        .date("from")?
        .unwrap_or_else(|| ctx.today - chrono::Duration::days(13));
    if from > to {
        return Err(McpError::invalid(NAME, "from", "must not be after to"));
    }
    let focus = args
        .enum_str("focus", FOCUSES)?
        .unwrap_or_else(|| "all".to_string());

    let range = common::query(&[
        ("from", Some(common::date_text(from))),
        ("to", Some(common::date_text(to))),
    ]);

    // Each of the three endpoints knows a part of a day. One of them being
    // empty is not a failure of the tool: the rest is still an answer, and
    // the answer says which part is missing.
    let mut limitations: Vec<String> = Vec::new();
    let readiness = match common::get(api, NAME, &format!("/api/v1/physiology/readiness?{range}")) {
        Ok(value) => items(&value),
        Err(_) => {
            limitations.push("the readiness of the range could not be read".to_string());
            Vec::new()
        }
    };
    let daily = match common::get(
        api,
        NAME,
        &format!("/api/v1/physiology/daily?{range}&fields={DAILY_FIELDS}"),
    ) {
        Ok(value) => items(&value),
        Err(_) => {
            limitations.push(
                "the sleep and the resting heart rate of the range could not be read".to_string(),
            );
            Vec::new()
        }
    };
    let hrv = match common::get(api, NAME, &format!("/api/v1/physiology/hrv?{range}")) {
        Ok(value) => items(&value),
        Err(_) => {
            limitations.push("the HRV of the range could not be read".to_string());
            Vec::new()
        }
    };

    // One row per day, keyed by the date. The three endpoints each know
    // part of a day, so the parts are merged rather than overwritten.
    let mut days: Vec<(String, Value)> = Vec::new();

    for row in &daily {
        let Some(date) = common::str_of(row, "date") else {
            continue;
        };
        merge_day(
            &mut days,
            date.to_string(),
            json!({
                "sleep": {
                    "sleep_s": common::f64_of(row, "sleep_s"),
                    "sleep_score": common::f64_of(row, "sleep_score"),
                    "deep_sleep_s": common::f64_of(row, "deep_sleep_s"),
                    "rem_s": common::f64_of(row, "rem_s"),
                    "light_sleep_s": common::f64_of(row, "light_sleep_s"),
                    "awake_s": common::f64_of(row, "awake_s"),
                },
                "resting_hr": {
                    "value": common::f64_of(row, "resting_hr"),
                    "baseline": Value::Null,
                },
                "source": common::str_of(row, "source"),
            }),
        );
    }

    for row in &hrv {
        let Some(date) = common::str_of(row, "date") else {
            continue;
        };
        merge_day(
            &mut days,
            date.to_string(),
            json!({
                "hrv": {
                    "value": common::f64_of(row, "hrv_ms"),
                    "status": common::str_of(row, "status"),
                    "baseline": common::f64_of(row, "baseline"),
                },
                "resting_hr": {
                    "value": common::f64_of(row, "resting_hr"),
                    "baseline": common::f64_of(row, "rhr_baseline"),
                },
            }),
        );
    }

    for row in &readiness {
        let Some(date) = common::str_of(row, "date") else {
            continue;
        };
        merge_day(
            &mut days,
            date.to_string(),
            json!({
                "readiness": {
                    "score": common::f64_of(row, "score"),
                    "grade": common::str_of(row, "grade"),
                    "drivers": drivers(row),
                },
            }),
        );
    }

    days.sort_by(|left, right| left.0.cmp(&right.0));

    // A year of daily data is 365 rows. The most recent ones are the ones
    // that answer a question, so the oldest are dropped and the answer says
    // how many.
    let mut dropped_days = 0usize;
    if days.len() > crate::output::MAX_SERIES {
        dropped_days = days.len() - crate::output::MAX_SERIES;
        let start = days.len() - crate::output::MAX_SERIES;
        days = days.split_off(start);
    }

    // What is not there at all.
    let mut missing_sources: Vec<String> = Vec::new();
    if let Ok(sources) = common::get(api, NAME, "/api/v1/physiology/sources") {
        if let Some(endpoints) = sources.get("endpoints").and_then(Value::as_array) {
            for endpoint in endpoints {
                let available = common::bool_of(endpoint, "available").unwrap_or(false);
                if !available {
                    if let Some(name) = common::str_of(endpoint, "name") {
                        missing_sources.push(name.to_string());
                    }
                }
            }
        }
    }

    let measured = days
        .iter()
        .filter(|(_, value)| {
            value
                .get("readiness")
                .and_then(|item| item.get("score"))
                .is_some_and(|score| !score.is_null())
        })
        .count();

    let rendered: Vec<Value> = days
        .iter()
        .map(|(date, value)| {
            let mut day = value.clone();
            day["date"] = json!(date);
            day
        })
        .collect();

    if focus != "all" {
        limitations.push(format!(
            "the focus is \"{focus}\"; the other measurements are still returned so that the \
readiness drivers can be read against them."
        ));
    }
    if dropped_days > 0 {
        limitations.push(format!(
            "the range holds {dropped_days} more days than the answer carries; the oldest days \
are not shown."
        ));
    }
    if measured < rendered.len() {
        limitations.push(format!(
            "{} of the {} days in the range have no readiness score.",
            rendered.len() - measured,
            rendered.len()
        ));
    }

    let data = json!({
        "focus": focus,
        "days": rendered,
    });

    let mut envelope = Envelope::new(data)
        .coverage("from", common::date_text(from))
        .coverage("to", common::date_text(to))
        .coverage("days_in_range", (to - from).num_days() + 1)
        .coverage("days", days.len())
        .coverage("days_with_readiness", measured)
        .coverage("missing_sources", missing_sources.join(","))
        .coverage(
            "attributed_to",
            "the daily physiology table of the athlete's own database",
        );
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// Merge one part of a day into the day's row.
fn merge_day(days: &mut Vec<(String, Value)>, date: String, value: Value) {
    match days.iter_mut().find(|(key, _)| key == &date) {
        Some((_, slot)) => *slot = merge_value(slot, &value),
        None => days.push((date, value)),
    }
}

/// Merge two JSON values, the new one winning where the two disagree.
fn merge_value(old: &Value, new: &Value) -> Value {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            let mut out = old_map.clone();
            for (key, value) in new_map {
                let merged = match out.get(key) {
                    Some(existing) => merge_value(existing, value),
                    None => value.clone(),
                };
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => new.clone(),
    }
}

/// The drivers behind a readiness score, the three that matter most.
fn drivers(readiness: &Value) -> Vec<Value> {
    let mut list: Vec<Value> = readiness
        .get("drivers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    list.sort_by(|left, right| {
        let left = common::f64_of(left, "contribution").unwrap_or(0.0);
        let right = common::f64_of(right, "contribution").unwrap_or(0.0);
        right
            .partial_cmp(&left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    list.truncate(3);
    list.iter()
        .map(|driver| {
            json!({
                "key": common::str_of(driver, "key"),
                "label": common::str_of(driver, "label"),
                "value": common::f64_of(driver, "value"),
                "unit": common::str_of(driver, "unit"),
                "baseline": common::f64_of(driver, "baseline"),
                "delta": common::f64_of(driver, "delta"),
                "contribution": common::f64_of(driver, "contribution"),
            })
        })
        .collect()
}
