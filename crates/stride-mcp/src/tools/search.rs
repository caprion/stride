//! `search_activities` — find the athlete's runs.
//!
//! A filtered, paged list. The list is what the harness shows first, so it is
//! deliberately narrow: a run's name, date, distance, pace, elevation and
//! heart rate, and nothing that would put the athlete's location or their
//! Garmin account in a model's context.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::{pick, Envelope, MAX_ITEMS};
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "search_activities";

/// What the tool does.
pub const DESCRIPTION: &str = "Find runs by date, sport, distance or name. Returns a page of \
run summaries with a cursor for the next page. Runs only, unless the sport is \"all\". The \
distance filters are applied to the page the API returned, so the total can be larger than the \
number of items that match.";

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "from": schema::date("The first day of the range, inclusive."),
            "to": schema::date("The last day of the range, inclusive."),
            "sport": schema::string(
                "The sport to match exactly, or \"all\" for every sport the athlete's history \
        holds. Defaults to the run sports (running, trail running, treadmill, virtual, street and \
        track).",
            ),
            "min_distance_m": schema::number(
                "Only runs at least this long, in metres.",
                Some(0.0),
                None,
            ),
            "max_distance_m": schema::number(
                "Only runs at most this long, in metres.",
                Some(0.0),
                None,
            ),
            "q": schema::string("Match the run's name or its date."),
            "sort": schema::enumeration(
                &["date", "distance", "pace", "elevation"],
                "How to order the runs. Defaults to date, newest first.",
            ),
            "limit": schema::integer(
                "How many runs to return, 1 to 100. Defaults to 20.",
                Some(1),
                Some(100),
            ),
            "cursor": schema::string(
                "The cursor of the page to return, as returned by a previous call. Defaults to the \
        first page.",
            ),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, _ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&[
        "from",
        "to",
        "sport",
        "min_distance_m",
        "max_distance_m",
        "q",
        "sort",
        "limit",
        "cursor",
    ])?;

    let from = args.date("from")?;
    let to = args.date("to")?;
    if let (Some(first), Some(last)) = (from, to) {
        if first > last {
            return Err(McpError::invalid(NAME, "to", "must not be before from"));
        }
    }
    let sport = args.opt_str("sport")?.map(str::to_string);
    let min_distance = args.opt_f64("min_distance_m")?;
    let max_distance = args.opt_f64("max_distance_m")?;
    if let (Some(minimum), Some(maximum)) = (min_distance, max_distance) {
        if minimum > maximum {
            return Err(McpError::invalid(
                NAME,
                "min_distance_m",
                "must not be greater than max_distance_m",
            ));
        }
    }
    let query_text = args.opt_str("q")?.map(str::to_string);
    let sort = args
        .enum_str("sort", &["date", "distance", "pace", "elevation"])?
        .unwrap_or_else(|| "date".to_string());
    let limit = args.bounded_i64("limit", 20, 1, 100)?;
    let page = cursor_page(args.opt_str("cursor")?)?;

    let mut limitations: Vec<String> = Vec::new();
    let mut collected: Vec<Value> = Vec::new();
    let mut total: i64 = 0;

    match sport.as_deref() {
        Some("all") => {
            // The API filters on one sport at a time, so "all" is a fan-out:
            // the run sports (the API's default) and every other sport in
            // the athlete's own history, which the summary knows.
            let (found, count) = fetch(
                api,
                &Page {
                    sport: None,
                    from,
                    to,
                    query: query_text.as_deref(),
                    sort: &sort,
                    page,
                    per_page: limit,
                },
            )?;
            collected.extend(found);
            total += count;
            let others = other_sports(api);
            let plural = if others.len() == 1 { "sport" } else { "sports" };
            if others.is_empty() {
                limitations.push(
                    "sport \"all\" covers the run sports; the other sports of the athlete's \
history could not be read."
                        .to_string(),
                );
            }
            for other in &others {
                let (found, count) = fetch(
                    api,
                    &Page {
                        sport: Some(other),
                        from,
                        to,
                        query: query_text.as_deref(),
                        sort: &sort,
                        page,
                        per_page: limit,
                    },
                )?;
                collected.extend(found);
                total += count;
            }
            limitations.push(format!(
                "sport \"all\" covers the run sports and the {} other {plural} of the athlete's \
history. The page is taken from each sport separately, so the cursor moves within a sport.",
                others.len()
            ));
        }
        Some(sport) => {
            let (found, count) = fetch(
                api,
                &Page {
                    sport: Some(sport),
                    from,
                    to,
                    query: query_text.as_deref(),
                    sort: &sort,
                    page,
                    per_page: limit,
                },
            )?;
            collected = found;
            total = count;
        }
        None => {
            let (found, count) = fetch(
                api,
                &Page {
                    sport: None,
                    from,
                    to,
                    query: query_text.as_deref(),
                    sort: &sort,
                    page,
                    per_page: limit,
                },
            )?;
            collected = found;
            total = count;
        }
    }

    // The distance filters are not something the API can express, so they
    // are applied here — to the page, not to the whole history.
    let mut ordered: Vec<Value> = collected
        .into_iter()
        .filter(|item| {
            let distance = common::f64_of(item, "distance_m").unwrap_or(0.0);
            if let Some(minimum) = min_distance {
                if distance < minimum {
                    return false;
                }
            }
            if let Some(maximum) = max_distance {
                if distance > maximum {
                    return false;
                }
            }
            true
        })
        .collect();

    if min_distance.is_some() || max_distance.is_some() {
        limitations.push(
            "the distance filters are applied to the page the API returned, so total_estimate \
counts the runs that match the other filters only."
                .to_string(),
        );
    }

    sort_items(&mut ordered, &sort);

    let shown = crate::output::take(ordered, MAX_ITEMS.min(limit as usize));
    let returned = shown.len();
    let has_more = page * limit < total;

    let data = json!({
        "items": shown
            .iter()
            .map(|item| pick(item, common::ACTIVITY_ITEM_FIELDS))
            .collect::<Vec<Value>>(),
        "next_cursor": if has_more {
            Value::String(format!("p{}", page + 1))
        } else {
            Value::Null
        },
        "total_estimate": total,
        "filters": {
            "from": from.map(common::date_text),
            "to": to.map(common::date_text),
            "sport": sport.clone().unwrap_or_else(|| "run_sports".to_string()),
            "min_distance_m": min_distance,
            "max_distance_m": max_distance,
            "q": query_text.clone(),
            "sort": sort.clone(),
            "limit": limit,
        },
    });

    let mut envelope = Envelope::new(data)
        .coverage("from", from.map(common::date_text))
        .coverage("to", to.map(common::date_text))
        .coverage("total_estimate", total)
        .coverage("returned", returned)
        .coverage("page", page)
        .coverage(
            "attributed_to",
            "the activity list of the athlete's own Stride database",
        );
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// Sort the runs the way the caller asked for.
fn sort_items(items: &mut [Value], sort: &str) {
    match sort {
        "distance" => items.sort_by(|left, right| {
            let left = common::f64_of(left, "distance_m").unwrap_or(0.0);
            let right = common::f64_of(right, "distance_m").unwrap_or(0.0);
            right
                .partial_cmp(&left)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "elevation" => items.sort_by(|left, right| {
            let left = common::f64_of(left, "elevation_gain_m").unwrap_or(0.0);
            let right = common::f64_of(right, "elevation_gain_m").unwrap_or(0.0);
            right
                .partial_cmp(&left)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "pace" => items.sort_by(|left, right| {
            let left = common::pace_of(left).unwrap_or(f64::MAX);
            let right = common::pace_of(right).unwrap_or(f64::MAX);
            left.partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => items.sort_by(|left, right| {
            let left = common::str_of(left, "start_time")
                .map(str::to_string)
                .unwrap_or_default();
            let right = common::str_of(right, "start_time")
                .map(str::to_string)
                .unwrap_or_default();
            right.cmp(&left)
        }),
    }
}

/// The page a cursor names.
fn cursor_page(cursor: Option<&str>) -> Result<i64, McpError> {
    let Some(cursor) = cursor else {
        return Ok(1);
    };
    let raw = cursor.strip_prefix('p').unwrap_or(cursor);
    let page = raw
        .parse::<i64>()
        .map_err(|_| McpError::invalid(NAME, "cursor", "is not a cursor this tool issued"))?;
    if page < 1 {
        return Err(McpError::invalid(
            NAME,
            "cursor",
            "must be a page of at least 1",
        ));
    }
    Ok(page)
}

/// One page of runs to ask for.
struct Page<'a> {
    sport: Option<&'a str>,
    from: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
    query: Option<&'a str>,
    sort: &'a str,
    page: i64,
    per_page: i64,
}

/// The sports of the athlete's history that are not run sports.
///
/// The router's own summary lists every sport the database holds, so "all"
/// is the athlete's sports and not a guess about what sports exist.
fn other_sports(api: &dyn StrideApi) -> Vec<String> {
    let Ok(summary) = common::get(api, NAME, "/api/v1/stats/summary") else {
        return Vec::new();
    };
    let mut sports: Vec<String> = summary
        .get("sport_breakdown")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| common::str_of(item, "sport_type"))
                .filter(|sport| !stride_api::backend::is_run_sport(sport))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    sports.sort();
    sports.dedup();
    sports
}

/// Ask the API for one page of runs.
fn fetch(api: &dyn StrideApi, page: &Page<'_>) -> Result<(Vec<Value>, i64), McpError> {
    let path = format!(
        "/api/v1/activities?{}",
        common::query(&[
            ("sport", page.sport.map(str::to_string)),
            ("from", page.from.map(common::date_text)),
            ("to", page.to.map(common::date_text)),
            ("q", page.query.map(str::to_string)),
            ("sort", Some(page.sort.to_string())),
            ("page", Some(page.page.to_string())),
            ("per_page", Some(page.per_page.to_string())),
        ])
    );
    let value = common::get(api, NAME, &path)?;
    let total = common::i64_of(&value, "total").unwrap_or(0);
    Ok((items(&value), total))
}
