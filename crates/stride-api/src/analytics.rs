//! Analytics — the numbers behind the statistics pages.
//!
//! Everything here is computed on read from the run rows, so a re-import can
//! never leave a stale number behind. The reference is
//! `backend/app/services/analytics.py`, and the port is exact: the same
//! filters, the same grouping, the same rounding.
//!
//! **Where this should live.** The frozen contract
//! (`docs/rust-core-spec.md` §3) gives `stride-core` five functions —
//! fingerprint, readiness, baselines, verdict, compare — and says nothing
//! about the statistics. The statistics are pure functions over run rows, so
//! they belong in `stride-core` next to the rest of the domain. They are
//! here because the API needs them and `stride-core` does not offer them
//! yet; moving them is a mechanical move and this workstream says so in its
//! report rather than editing another crate.

use std::collections::BTreeMap;

use chrono::Datelike;

use serde_json::{json, Map, Value};

use crate::backend::{is_run_sport, ActivityRow, LapRow};
use crate::num::{
    distance_display, duration_label, mean, median, pace_label, pace_s_per_km, round_to, week_key,
};

/// The day the statistics are computed for. Passing it in keeps the numbers
/// reproducible: the reference reads the clock, and a golden file cannot.
pub type Today = chrono::NaiveDate;

/// The date of an activity, as the reference sees it.
///
/// The reference reads `activity.start_time.date()` — the UTC date of the
/// start, not the local date, which is a separate column.
fn start_date(row: &ActivityRow) -> Option<chrono::NaiveDate> {
    let raw = row.start_time.as_deref()?;
    chrono::NaiveDate::parse_from_str(raw.get(0..10)?, "%Y-%m-%d").ok()
}

/// The runs, newest first, with the demo filter applied.
pub fn runs(rows: &[ActivityRow], demo_visible: bool) -> Vec<&ActivityRow> {
    let mut out: Vec<&ActivityRow> = rows
        .iter()
        .filter(|row| is_run_sport(&row.sport_type))
        .filter(|row| demo_visible || row.source != "demo")
        .collect();
    out.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    out
}

/// The aggregate of a group of runs.
///
/// The reference: `analytics._bucket`.
fn bucket(activities: &[&ActivityRow], label: &str) -> Value {
    let distance: f64 = activities.iter().map(|row| row.distance_m).sum();
    let elevation: f64 = activities.iter().map(|row| row.elevation_gain_m).sum();
    let moving: f64 = activities.iter().map(|row| row.moving_time_s).sum();
    let paces: Vec<f64> = activities
        .iter()
        .filter_map(|row| pace_s_per_km(row.distance_m, row.moving_time_s))
        .filter(|pace| *pace != 0.0)
        .collect();
    let heart_rates: Vec<f64> = activities
        .iter()
        .filter_map(|row| row.avg_hr)
        .filter(|hr| *hr != 0.0)
        .collect();
    let calories: Vec<f64> = activities
        .iter()
        .filter_map(|row| row.calories)
        .filter(|value| *value != 0.0)
        .collect();
    let average_pace = mean(&paces);
    let average_hr = mean(&heart_rates);
    json!({
        "label": label,
        "activity_count": activities.len(),
        "distance_m": round_to(distance, 1),
        "elevation_gain_m": round_to(elevation, 1),
        "moving_time_s": round_to(moving, 1),
        "moving_time_label": duration_label(Some(moving)),
        "avg_pace_s_per_km": average_pace.map(|value| round_to(value, 2)),
        "avg_pace_label": pace_label(average_pace),
        "avg_hr": average_hr.map(|value| round_to(value, 1)),
        "calories": if calories.is_empty() { Value::Null } else { json!(round_to(calories.iter().sum::<f64>(), 0)) },
    })
}

/// Totals and gains — all time, this year, this month, this week.
///
/// The reference: `analytics.summary`.
pub fn summary(rows: &[ActivityRow], today: Today, demo_visible: bool) -> Value {
    let activities = runs(rows, demo_visible);
    let year_start = today
        .with_day(1)
        .and_then(|day| day.with_month(1))
        .unwrap_or(today);
    let month_start = today.with_day(1).unwrap_or(today);
    let week_start = today
        - chrono::Duration::days(chrono::Datelike::weekday(&today).num_days_from_monday() as i64);

    let within = |start: chrono::NaiveDate| -> Vec<&ActivityRow> {
        activities
            .iter()
            .copied()
            .filter(|row| start_date(row).is_some_and(|day| day >= start))
            .collect()
    };

    json!({
        "totals": bucket(&activities, "All time"),
        "this_year": bucket(&within(year_start), &year_start.year().to_string()),
        "this_month": bucket(&within(month_start), &today.format("%B %Y").to_string()),
        "this_week": bucket(&within(week_start), "This week"),
        "personal_records": personal_records(&activities),
        "streaks": streaks(&activities, today),
        "sport_breakdown": sport_breakdown(rows, demo_visible),
    })
}

/// The distance per sport.
///
/// The reference: `analytics.sport_breakdown`.
pub fn sport_breakdown(rows: &[ActivityRow], demo_visible: bool) -> Value {
    let mut totals: BTreeMap<&str, (i64, f64, f64, f64)> = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|row| demo_visible || row.source != "demo")
    {
        let entry = totals
            .entry(row.sport_type.as_str())
            .or_insert((0, 0.0, 0.0, 0.0));
        entry.0 += 1;
        entry.1 += row.distance_m;
        entry.2 += row.elevation_gain_m;
        entry.3 += row.moving_time_s;
    }
    let mut out: Vec<(f64, Value)> = totals
        .into_iter()
        .map(|(sport, (count, distance, elevation, moving))| {
            (
                distance,
                json!({
                    "sport_type": sport,
                    "activity_count": count,
                    "distance_m": round_to(distance, 1),
                    "elevation_gain_m": round_to(elevation, 1),
                    "moving_time_s": round_to(moving, 1),
                }),
            )
        })
        .collect();
    // The reference orders by the summed distance, descending.
    out.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    Value::Array(out.into_iter().map(|(_, value)| value).collect())
}

/// The volume of each ISO week.
///
/// The reference: `analytics.weekly_volume`.
pub fn weekly_volume(rows: &[ActivityRow], demo_visible: bool, weeks: usize) -> Value {
    let activities = runs(rows, demo_visible);
    let mut buckets: BTreeMap<String, Vec<&ActivityRow>> = BTreeMap::new();
    for activity in activities {
        if activity.local_date.is_empty() {
            continue;
        }
        buckets
            .entry(week_key(&activity.local_date))
            .or_default()
            .push(activity);
    }
    let keys: Vec<String> = buckets.keys().rev().take(weeks).rev().cloned().collect();
    let mut out = Vec::new();
    for key in keys {
        let mut item = bucket(&buckets[&key], &key);
        if let Some(object) = item.as_object_mut() {
            object.insert("week_key".into(), json!(key));
            object.insert("start_date".into(), json!(crate::num::week_start(&key)));
        }
        out.push(item);
    }
    Value::Array(out)
}

/// The volume of each month.
///
/// The reference: `analytics.monthly_volume`.
pub fn monthly_volume(rows: &[ActivityRow], demo_visible: bool, months: usize) -> Value {
    let activities = runs(rows, demo_visible);
    let mut buckets: BTreeMap<String, Vec<&ActivityRow>> = BTreeMap::new();
    for activity in activities {
        if !activity.local_date.is_empty() {
            buckets
                .entry(crate::num::month_key(&activity.local_date))
                .or_default()
                .push(activity);
        }
    }
    let keys: Vec<String> = buckets.keys().rev().take(months).rev().cloned().collect();
    let mut out = Vec::new();
    for key in keys {
        let mut item = bucket(&buckets[&key], &key);
        if let Some(object) = item.as_object_mut() {
            object.insert("month_key".into(), json!(key));
        }
        out.push(item);
    }
    Value::Array(out)
}

/// The weekly pace statistics.
///
/// The reference: `analytics.pace_trends`.
pub fn pace_trends(rows: &[ActivityRow], demo_visible: bool, weeks: usize) -> Value {
    let activities = runs(rows, demo_visible);
    let mut buckets: BTreeMap<String, Vec<&ActivityRow>> = BTreeMap::new();
    for activity in activities {
        if !activity.local_date.is_empty() {
            buckets
                .entry(week_key(&activity.local_date))
                .or_default()
                .push(activity);
        }
    }
    let keys: Vec<String> = buckets.keys().rev().take(weeks).rev().cloned().collect();
    let mut out = Vec::new();
    for key in keys {
        let entries = &buckets[&key];
        let paces: Vec<f64> = entries
            .iter()
            .filter_map(|row| pace_s_per_km(row.distance_m, row.moving_time_s))
            .filter(|pace| *pace != 0.0)
            .collect();
        let long_runs: Vec<f64> = entries
            .iter()
            .filter(|row| row.distance_m >= 10000.0)
            .filter_map(|row| pace_s_per_km(row.distance_m, row.moving_time_s))
            .filter(|pace| *pace != 0.0)
            .collect();
        let median_pace = median(&paces);
        let best = paces.iter().cloned().fold(None::<f64>, |acc, pace| {
            Some(match acc {
                None => pace,
                Some(current) => current.min(pace),
            })
        });
        let worst = paces.iter().cloned().fold(None::<f64>, |acc, pace| {
            Some(match acc {
                None => pace,
                Some(current) => current.max(pace),
            })
        });
        let long_median = median(&long_runs);
        out.push(json!({
            "week_key": key,
            "start_date": crate::num::week_start(&key),
            "run_count": entries.len(),
            "median_pace_s_per_km": median_pace.map(|value| round_to(value, 2)),
            "median_pace_label": pace_label(median_pace),
            "best_pace_s_per_km": best.map(|value| round_to(value, 2)),
            "best_pace_label": pace_label(best),
            "worst_pace_s_per_km": worst.map(|value| round_to(value, 2)),
            "long_run_pace_s_per_km": long_median.map(|value| round_to(value, 2)),
            "long_run_pace_label": pace_label(long_median),
        }));
    }
    Value::Array(out)
}

/// The heart-rate zone distribution.
///
/// The reference: `analytics.hr_zones`. The zones are Garmin's default
/// five-zone model, anchored on the athlete's observed maximum heart rate.
pub fn hr_zones(
    rows: &[ActivityRow],
    laps: &[LapRow],
    demo_visible: bool,
    weeks: usize,
    today: Today,
) -> Value {
    let activities = runs(rows, demo_visible);
    let cutoff = today - chrono::Duration::weeks(weeks as i64);
    let recent: Vec<&&ActivityRow> = activities
        .iter()
        .filter(|row| start_date(row).is_some_and(|day| day >= cutoff))
        .collect();

    let peak = recent
        .iter()
        .filter_map(|row| row.max_hr)
        .filter(|hr| *hr != 0.0)
        .fold(None::<f64>, |acc, hr| {
            Some(match acc {
                None => hr,
                Some(current) => current.max(hr),
            })
        })
        .unwrap_or(190.0);
    let bounds = [0.0, 0.6, 0.7, 0.8, 0.9, 1.01];
    let names = ["Recovery", "Aerobic", "Tempo", "Threshold", "Anaerobic"];
    let mut zones: Vec<(i64, String, f64, f64, f64)> = (0..5)
        .map(|index| {
            (
                index as i64 + 1,
                names[index].to_string(),
                round_to(bounds[index] * peak, 0),
                round_to(bounds[index + 1] * peak, 0),
                0.0,
            )
        })
        .collect();

    let by_activity: BTreeMap<i64, &ActivityRow> =
        activities.iter().map(|row| (row.id, *row)).collect();
    let mut total_time = 0.0;
    for lap in laps {
        let Some(activity) = by_activity.get(&lap.activity_id) else {
            continue;
        };
        let Some(day) = start_date(activity) else {
            continue;
        };
        if day < cutoff {
            continue;
        }
        let Some(hr) = lap.avg_hr.filter(|value| *value != 0.0) else {
            continue;
        };
        total_time += lap.duration_s;
        for zone in zones.iter_mut() {
            if zone.2 <= hr && hr < zone.3 {
                zone.4 += lap.duration_s;
                break;
            }
        }
    }

    let rendered: Vec<Value> = zones
        .iter_mut()
        .map(|(zone_no, name, low, high, time)| {
            *time = round_to(*time, 1);
            let percent = if total_time > 0.0 {
                round_to(*time / total_time * 100.0, 1)
            } else {
                0.0
            };
            json!({
                "zone_no": zone_no,
                "name": name,
                "low_bpm": low,
                "high_bpm": high,
                "time_s": time,
                "percent": percent,
            })
        })
        .collect();

    json!({
        "zones": rendered,
        "total_time_s": round_to(total_time, 1),
        "peak_hr": peak,
        "weeks": weeks,
    })
}

/// The best performances across the whole history.
///
/// The reference: `analytics.personal_records`.
pub fn personal_records(activities: &[&ActivityRow]) -> Value {
    let mut records: Vec<Value> = Vec::new();

    let mut add = |label: &str, value: f64, activity: &ActivityRow, unit: &str, better: &str| {
        records.push(json!({
            "label": label,
            "value": value,
            "display": distance_display(value, unit),
            "unit": unit,
            "better": better,
            "activity_id": activity.id,
            "activity_name": activity.name,
            "activity_date": activity.local_date,
        }));
    };

    let with_distance: Vec<&&ActivityRow> = activities
        .iter()
        .filter(|row| row.distance_m > 0.0)
        .collect();
    if let Some(longest) = with_distance.iter().max_by(|a, b| {
        a.distance_m
            .partial_cmp(&b.distance_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        add("Longest run", longest.distance_m, longest, "m", "lower");
    }
    if let Some(fastest) = with_distance
        .iter()
        .filter(|row| row.moving_time_s > 0.0)
        .min_by(|a, b| {
            let left = pace_s_per_km(a.distance_m, a.moving_time_s).unwrap_or(1e9);
            let right = pace_s_per_km(b.distance_m, b.moving_time_s).unwrap_or(1e9);
            left.partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        if let Some(pace) = pace_s_per_km(fastest.distance_m, fastest.moving_time_s) {
            add("Fastest pace", pace, fastest, "s/km", "lower");
        }
    }
    if let Some(climb) = activities.iter().max_by(|a, b| {
        a.elevation_gain_m
            .partial_cmp(&b.elevation_gain_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        add("Biggest climb", climb.elevation_gain_m, climb, "m", "lower");
    }
    if let Some(longest) = activities.iter().max_by(|a, b| {
        a.moving_time_s
            .partial_cmp(&b.moving_time_s)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        add("Longest time", longest.moving_time_s, longest, "s", "lower");
    }
    // No heart-rate "record": the highest average heart rate goes up when the
    // athlete overheats or overreaches, and a record is an incentive.
    let with_calories: Vec<&&ActivityRow> = activities
        .iter()
        .filter(|row| row.calories.is_some_and(|value| value != 0.0))
        .collect();
    if let Some(biggest) = with_calories.iter().max_by(|a, b| {
        a.calories
            .unwrap_or(0.0)
            .partial_cmp(&b.calories.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        add(
            "Most calories",
            biggest.calories.unwrap_or(0.0),
            biggest,
            "kcal",
            "higher",
        );
    }

    records.sort_by(|a, b| {
        let left = a.get("label").and_then(Value::as_str).unwrap_or("");
        let right = b.get("label").and_then(Value::as_str).unwrap_or("");
        left.cmp(right)
    });
    Value::Array(records)
}

/// The daily distance for the calendar heatmap.
///
/// The reference: `analytics.heatmap`.
pub fn heatmap(rows: &[ActivityRow], demo_visible: bool, year: i32) -> Value {
    let prefix = format!("{year:04}-");
    let mut totals: BTreeMap<String, (f64, i64)> = BTreeMap::new();
    for row in rows.iter().filter(|row| is_run_sport(&row.sport_type)) {
        if !demo_visible && row.source == "demo" {
            continue;
        }
        if !row.local_date.starts_with(&prefix) {
            continue;
        }
        let entry = totals.entry(row.local_date.clone()).or_insert((0.0, 0));
        entry.0 += row.distance_m;
        entry.1 += 1;
    }
    let days: Vec<Value> = totals
        .iter()
        .map(|(date, (distance, count))| {
            json!({
                "date": date,
                "distance_m": round_to(*distance, 1),
                "activity_count": count,
            })
        })
        .collect();
    json!({ "year": year, "days": days })
}

/// The current and longest run streaks, in weeks.
///
/// The reference: `analytics.streaks`.
pub fn streaks(activities: &[&ActivityRow], today: Today) -> Value {
    let mut weeks: Vec<String> = activities
        .iter()
        .filter(|row| !row.local_date.is_empty())
        .map(|row| week_key(&row.local_date))
        .collect();
    weeks.sort();
    weeks.dedup();
    if weeks.is_empty() {
        return json!({
            "current_weeks": 0,
            "longest_weeks": 0,
            "total_active_weeks": 0,
        });
    }

    let to_date = |key: &str| -> Option<chrono::NaiveDate> {
        let (year, week) = key.split_once("-W")?;
        let year: i32 = year.parse().ok()?;
        let week: u32 = week.parse().ok()?;
        let jan4 = chrono::NaiveDate::from_ymd_opt(year, 1, 4)?;
        let monday = jan4
            - chrono::Duration::days(chrono::Datelike::weekday(&jan4).num_days_from_monday() as i64);
        Some(monday + chrono::Duration::days(7 * (week as i64 - 1)))
    };

    let mut longest = 0i64;
    let mut current = 0i64;
    let mut previous: Option<chrono::NaiveDate> = None;
    for key in &weeks {
        let Some(current_date) = to_date(key) else {
            continue;
        };
        if let Some(previous) = previous {
            if (current_date - previous).num_days() == 7 {
                current += 1;
            } else {
                current = 1;
            }
        } else {
            current = 1;
        }
        longest = longest.max(current);
        previous = Some(current_date);
    }

    let run_weeks: Vec<chrono::NaiveDate> = weeks.iter().filter_map(|key| to_date(key)).collect();
    let this_week = today
        - chrono::Duration::days(chrono::Datelike::weekday(&today).num_days_from_monday() as i64);
    let mut current_streak = 0i64;
    let mut cursor = this_week;
    while run_weeks.contains(&cursor) {
        current_streak += 1;
        cursor -= chrono::Duration::days(7);
    }

    json!({
        "current_weeks": current_streak,
        "longest_weeks": longest,
        "total_active_weeks": weeks.len(),
        "current_week_key": week_key(&today.format("%Y-%m-%d").to_string()),
    })
}

/// The training calendar of one month.
///
/// The reference: `analytics.calendar`.
pub fn calendar(rows: &[ActivityRow], demo_visible: bool, year: i32, month: u32) -> Value {
    let prefix = format!("{year:04}-{month:02}");
    let mut by_day: BTreeMap<String, Vec<&ActivityRow>> = BTreeMap::new();
    let mut activities: Vec<&ActivityRow> = Vec::new();
    for row in runs(rows, demo_visible) {
        if !row.local_date.starts_with(&prefix) {
            continue;
        }
        activities.push(row);
        by_day.entry(row.local_date.clone()).or_default().push(row);
    }
    activities.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // The reference emits one day fewer than the month has: its loop stops
    // on the first day that does not exist and then counts from 1 to that
    // day, so the last day of every month is missing. The golden fixture
    // agrees with the reference, so the port reproduces it — but it looks
    // like a bug in the reference, and this workstream says so in its
    // report. Fixing the reference means regenerating the fixture.
    let day_count = days_in_month(year, month).saturating_sub(1);
    let mut days = Vec::new();
    for day in 1..=day_count {
        let key = format!("{prefix}-{day:02}");
        let entries = by_day.get(&key).map(Vec::as_slice).unwrap_or(&[]);
        days.push(json!({
            "date": key,
            "day": day,
            "activities": entries.iter().map(|row| crate::shapes::activity_card(row)).collect::<Vec<Value>>(),
            "distance_m": round_to(entries.iter().map(|row| row.distance_m).sum::<f64>(), 1),
            "moving_time_s": round_to(entries.iter().map(|row| row.moving_time_s).sum::<f64>(), 1),
        }));
    }

    json!({
        "year": year,
        "month": month,
        "days": days,
        "total_distance_m": round_to(activities.iter().map(|row| row.distance_m).sum::<f64>(), 1),
        "activity_count": activities.len(),
    })
}

/// The days in a month.
fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1);
    let next = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1);
    match (first, next) {
        (Some(first), Some(next)) => (next - first).num_days() as u32,
        _ => 30,
    }
}

/// The most recent runs, as the summary page shows them.
///
/// The reference: `analytics.recent_activities`.
pub fn recent_activities(rows: &[ActivityRow], demo_visible: bool, limit: usize) -> Value {
    let activities = runs(rows, demo_visible);
    let items: Vec<Value> = activities
        .iter()
        .take(limit)
        .map(|row| crate::shapes::activity_card(row))
        .collect();
    Value::Array(items)
}

/// The shape of a summary payload, used by the tests.
pub fn empty_summary() -> Value {
    let mut out = Map::new();
    out.insert("totals".into(), Value::Null);
    Value::Object(out)
}
