//! The rolling view: a window of history, and what it holds.
//!
//! Two routes answer here, and they are the two halves of the item:
//!
//! * `/insights/rolling?weeks=N` — the window and the period before it,
//!   the split of the training across the athlete's own zones, the weekday
//!   and daytime splits, and the monthly and yearly tables;
//! * `/insights/streaks` — the streaks: the current run of days and the
//!   longest one, and the same for weeks and months.
//!
//! ## What is computed where
//!
//! The numbers are the **core's**: the streaks, the window's totals, the
//! split of the zones, the weekday and daytime splits and the monthly and
//! yearly tables all come from `stride-core`'s rolling view. This module
//! reads them through the seam in [`crate::rolling`], which is the one
//! place the store's rows become the engine's inputs. It opens no
//! connection and re-derives nothing.
//!
//! ## The window
//!
//! The window is **the last *N* calendar weeks, from the start of the
//! earliest one through the day the answer is about**, and the comparison
//! period is **the same span, seven days earlier**. The card names both,
//! so a number is never read apart from the period it describes.
//!
//! The athlete profile's own ladder asks for the window in **days** —
//! thirty, sixty and ninety — and it is the same window type with the
//! same comparison rule.
//!
//! ## No value is the absence of the value
//!
//! A split with no model, an unusable model or no heart-rate samples has
//! **no shares at all**, and the reason in the core's own words. A streak
//! of zero days is **no streak**. Nothing here is ever a zero that stands
//! in for a value that could not be computed.

use serde_json::Value;
use stride_core::date::Date;
use stride_core::rolling::{DaytimeCuts, Window};
use stride_core::streaks::Streak;
use stride_core::units;

use crate::dto::{
    DaytimeRow, DistanceBucketRow, MonthStreakDetail, MonthStreakValue, PeriodRow, ProfileFacet,
    RollingInsight, RollingWindow as RollingWindowDto, SessionComparison, StreakDetail,
    StreakInsight, StreakValue, WeekdayRow, WindowComparison, ZoneDistributionRow, ZoneModelSource,
    ZoneShareRow, ZoneSplit,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// How many weeks the window may cover — the Progress route's own bound.
const MAX_WEEKS: i64 = 104;

/// How many days the window may cover: the weeks' bound, in days.
const MAX_DAYS: i64 = MAX_WEEKS * DAYS_PER_WEEK;

/// The days a week holds: the window is measured in weeks, and it is
/// measured in days.
const DAYS_PER_WEEK: i64 = 7;

/// The weeks the window covers when nothing is asked for.
const DEFAULT_WEEKS: i64 = 12;

/// The rolling view over a window.
pub fn rolling(api: &Api, request: &Request) -> ApiResult<Value> {
    let current = super::today(api)?;
    let window = window_of_request(request, current)?;

    // The whole history: the window's runs and the splits' runs are in it,
    // and it is one read of the store.
    let runs = crate::rolling::window_runs(api, "0000-01-01", &current.to_iso())?;
    let activities = &runs.activities;

    let comparison = crate::rolling::window_comparison(activities, window);

    // The runs the split is measured over: the window and the period
    // before it.
    let window_runs: Vec<&stride_store::ActivitySummary> = runs
        .runs
        .iter()
        .filter(|run| {
            Date::parse(&run.local_date)
                .map(|date| window.contains(date) || window.previous().contains(date))
                .unwrap_or(false)
        })
        .collect();
    let split = crate::rolling::zone_split(api, &window_runs, window)?;

    let weekday = crate::rolling::weekday_split_of(activities);
    let daytime = crate::rolling::daytime_split_of(activities, DaytimeCuts::default());
    let (months, years) = crate::rolling::period_tables(activities);

    // The runs of the window and of the period before it: the breakdown
    // and the facets are about the window.
    let in_window: Vec<stride_core::rolling::RollingActivity> = activities
        .iter()
        .filter(|activity| window.contains(activity.date))
        .cloned()
        .collect();
    let in_previous: Vec<stride_core::rolling::RollingActivity> = activities
        .iter()
        .filter(|activity| window.previous().contains(activity.date))
        .cloned()
        .collect();
    let buckets = crate::rolling::distance_buckets(&in_window);
    let profile = crate::rolling::profile_facets(&in_window, &in_previous, window);

    let response = RollingInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        window: window_of(window),
        comparison: window_of(window.previous()),
        distance: distance_comparison(&comparison),
        duration: duration_comparison(&comparison),
        sessions: SessionComparison {
            current: comparison.current.sessions,
            previous: comparison.previous.sessions,
            difference: render::difference(
                Some(comparison.current.sessions as f64 - comparison.previous.sessions as f64),
                stride_core::ValueUnit::Count,
            ),
        },
        zones: zones_of(&split),
        distances: buckets.iter().map(bucket_of).collect(),
        profile: profile.iter().map(facet_of).collect(),
        weekday: weekday.iter().map(weekday_of).collect(),
        daytime: daytime.iter().map(daytime_of).collect(),
        months: months.iter().map(|month| period_of(month, true)).collect(),
        years: years.iter().map(|year| period_of(year, false)).collect(),
    };
    ok(&response)
}

/// The streaks of the athlete's history.
pub fn streaks(api: &Api, _request: &Request) -> ApiResult<Value> {
    let current = super::today(api)?;
    let answer = crate::rolling::streak_answer(api, current)?;
    let response = StreakInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        days: StreakDetail {
            current: answer.current_days.map(streak_of),
            longest: answer.longest_days.map(streak_of),
        },
        weeks: StreakDetail {
            current: answer.current_weeks.map(streak_of),
            longest: answer.longest_weeks.map(streak_of),
        },
        months: MonthStreakDetail {
            current: answer.current_months.map(month_streak_of),
            longest: answer.longest_months.map(month_streak_of),
        },
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The window the answer is about.
///
/// The window may be asked for in **weeks** — the Progress route's own
/// parameter, and the range the screen shows — or in **days**, which is
/// the athlete profile's ladder: thirty, sixty and ninety days.
///
/// A window asked for in weeks is **the last *N* calendar weeks**, from
/// the Monday of the earliest one through the day the answer is about: the
/// same window the Progress totals and the pace page draw. Its length is
/// therefore the window's own length and not a round number of days.
///
/// **`days` is the one that counts** when both are given: it is the finer
/// of the two, and thirty days is not four weeks and a bit.
fn window_of_request(request: &Request, today: Date) -> ApiResult<Window> {
    if let Some(days) = request.query.integer("days")? {
        if !(1..=MAX_DAYS).contains(&days) {
            return Err(ApiError::bad_request(format!(
                "the rolling window is not between 1 and {MAX_DAYS} days"
            )));
        }
        return Ok(Window::of_days(days, today));
    }
    let weeks = request.query.integer("weeks")?.unwrap_or(DEFAULT_WEEKS);
    if !(1..=MAX_WEEKS).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the rolling window is not between 1 and 104 weeks",
        ));
    }
    Ok(Window::calendar_weeks(weeks, today))
}

/// A window of days, as the answer names it.
fn window_of(window: Window) -> RollingWindowDto {
    let from = window.from();
    let to = window.to();
    RollingWindowDto {
        from: from.to_iso(),
        from_label: super::day_label(from),
        to: to.to_iso(),
        to_label: super::day_label(to),
        days: window.days(),
    }
}

/// The distance of the window and of the period before it.
///
/// The difference is the core's rendering of it, sign and all: the sign is
/// the number's own, and nothing here says which way is the better way.
fn distance_comparison(comparison: &stride_core::rolling::WindowComparison) -> WindowComparison {
    WindowComparison {
        current: render::distance(Some(comparison.current.distance_m)),
        previous: render::distance(Some(comparison.previous.distance_m)),
        difference: render::difference(
            Some(comparison.current.distance_m - comparison.previous.distance_m),
            stride_core::ValueUnit::Distance,
        ),
    }
}

/// The moving time of the window and of the period before it.
fn duration_comparison(comparison: &stride_core::rolling::WindowComparison) -> WindowComparison {
    WindowComparison {
        current: render::duration(Some(comparison.current.moving_time_s)),
        previous: render::duration(Some(comparison.previous.moving_time_s)),
        difference: render::difference(
            Some(comparison.current.moving_time_s - comparison.previous.moving_time_s),
            stride_core::ValueUnit::Duration,
        ),
    }
}

/// One bucket of the distance breakdown.
fn bucket_of(bucket: &stride_core::rolling::DistanceBucket) -> DistanceBucketRow {
    let from_m = bucket.from_km * 1000.0;
    let to_m = bucket.to_km * 1000.0;
    DistanceBucketRow {
        key: format!("{}-{}", bucket.from_km as i64, bucket.to_km as i64),
        label: units::render_distance_range(Some(from_m), Some(to_m)).unwrap_or_default(),
        sessions: bucket.sessions,
        distance: render::distance_of(bucket.distance_m),
        elevation: render::elevation_of(bucket.elevation_gain_m),
        duration: render::duration_of(bucket.moving_time_s),
    }
}

/// One facet of the athlete's shape.
fn facet_of(facet: &stride_core::profile::Facet) -> ProfileFacet {
    ProfileFacet {
        key: facet.key.key().to_string(),
        label: facet.label.to_string(),
        value: facet_value(facet.value, facet.unit),
        total: facet_value(facet.total, facet.total_unit),
        previous: facet_value(facet.previous, facet.unit),
        movement: facet.movement.map(|movement| movement.key().to_string()),
    }
}

/// A facet's value, in the unit the facet is in.
fn facet_value(
    value: Option<f64>,
    unit: stride_core::profile::FacetUnit,
) -> Option<crate::dto::RenderedValue> {
    use stride_core::profile::FacetUnit;
    match unit {
        FacetUnit::Distance => render::distance(value),
        FacetUnit::DistancePerWeek => render::distance_per_week(value),
        FacetUnit::Count => render::whole(value),
        FacetUnit::Share => render::share_of_days(value),
        FacetUnit::Duration => render::duration(value),
        FacetUnit::HeartRate => render::heart_rate(value),
    }
}

/// The split of the training across the athlete's zones.
///
/// The models travel with the shares: a number that cannot say where its
/// input came from is not inspectable. **The source line is hoisted** —
/// one line for the screen, naming the model the window was measured
/// against — and the models themselves are the detail behind it.
fn zones_of(split: &crate::rolling::ZoneSplitAnswer) -> ZoneSplit {
    let models: Vec<ZoneModelSource> = split
        .models
        .iter()
        .map(|model| {
            let read_on_label = super::day_label(model.read_on);
            ZoneModelSource {
                read_on: model.read_on.to_iso(),
                read_on_label: read_on_label.clone(),
                source: stride_core::model_source(
                    &model.training_method,
                    model.max_heart_rate,
                    &read_on_label,
                ),
            }
        })
        .collect();
    // One model keeps its own line; several models are named by the days
    // they were read.
    let source = match split.models.as_slice() {
        [] => None,
        [one] => stride_core::model_source(
            &one.training_method,
            one.max_heart_rate,
            &super::day_label(one.read_on),
        ),
        many => {
            let dates: Vec<String> = many
                .iter()
                .map(|model| super::day_label(model.read_on))
                .collect();
            let dates: Vec<&str> = dates.iter().map(String::as_str).collect();
            stride_core::models_source(&dates)
        }
    };
    let shares = split.shares.as_ref().map(|shares| {
        shares
            .iter()
            .map(|share| ZoneShareRow {
                key: share.band.key().to_string(),
                label: share.band.label().to_string(),
                share: render::share(share.share),
                previous_share: render::share(share.previous_share),
                trend: share.trend.map(|trend| trend.key().to_string()),
            })
            .collect()
    });
    let distribution = split.distribution.as_ref().map(|rows| {
        let edges = split
            .ranges
            .as_ref()
            .map(|model| stride_core::zones::zone_edges(&model.floors, model.max_heart_rate));
        rows.iter()
            .map(|row| {
                let (from, to) = edges
                    .map(|edges| edges[row.zone - 1])
                    .unwrap_or((0.0, None));
                ZoneDistributionRow {
                    zone: units::render_zone_label(row.zone).unwrap_or_default(),
                    range: units::render_heart_rate_range(Some(from), to),
                    time: render::duration_of(row.time_s),
                    share: render::share(Some(row.share)),
                }
            })
            .collect()
    });
    ZoneSplit {
        models,
        source,
        shares,
        distribution,
        unavailable: split.unavailable.clone(),
    }
}

/// One day of the week, as the answer names it.
fn weekday_of(day: &stride_core::rolling::WeekdayTotals) -> WeekdayRow {
    WeekdayRow {
        weekday: day.weekday.name().to_string(),
        sessions: day.sessions,
        distance: render::distance_of(day.distance_m),
        duration: render::duration_of(day.moving_time_s),
        share: render::share(day.share),
    }
}

/// One time of day, as the answer names it.
fn daytime_of(daytime: &stride_core::rolling::DaytimeTotals) -> DaytimeRow {
    DaytimeRow {
        key: daytime.daytime.key().to_string(),
        label: daytime.daytime.label().to_string(),
        from_hour: daytime.from_hour,
        to_hour: daytime.to_hour,
        sessions: daytime.sessions,
        distance: render::distance_of(daytime.distance_m),
        duration: render::duration_of(daytime.moving_time_s),
    }
}

/// One month or year of the history, as the answer names it.
fn period_of(period: &stride_core::rolling::PeriodTotals, month: bool) -> PeriodRow {
    PeriodRow {
        key: period.key.clone(),
        label: if month {
            month_label(&period.key)
        } else {
            period.key.clone()
        },
        sessions: period.sessions,
        distance: render::distance_of(period.distance_m),
        duration: render::duration_of(period.moving_time_s),
        elevation: render::elevation_of(period.elevation_gain_m),
    }
}

/// A month, as the api writes it: "April 2026".
fn month_label(key: &str) -> String {
    let mut parts = key.split('-');
    let year = parts.next().and_then(|year| year.parse::<i32>().ok());
    let month = parts.next().and_then(|month| month.parse::<u32>().ok());
    match (year, month) {
        (Some(year), Some(month)) => {
            let naive = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
            naive.format("%B %Y").to_string()
        }
        _ => key.to_string(),
    }
}

/// One streak, as the answer names it.
fn streak_of(streak: Streak) -> StreakValue {
    StreakValue {
        count: streak.count,
        start: streak.start.to_iso(),
        start_label: super::day_label(streak.start),
        end: streak.end.map(|end| end.to_iso()),
        end_label: streak.end.map(super::day_label),
    }
}

/// One streak of months, as the months it covers.
///
/// The core's month streak is a run of calendar months, each with a run in
/// it; the days it names are the newest run day of its oldest month, so
/// the answer names the **month** and not the day inside it.
fn month_streak_of(streak: Streak) -> MonthStreakValue {
    MonthStreakValue {
        count: streak.count,
        start: month_key(streak.start),
        start_label: super::month_label(streak.start),
        end: streak.end.map(month_key),
        end_label: streak.end.map(super::month_label),
    }
}

/// The month a day is in, as `YYYY-MM`.
fn month_key(day: Date) -> String {
    format!("{:04}-{:02}", day.year(), day.month())
}
