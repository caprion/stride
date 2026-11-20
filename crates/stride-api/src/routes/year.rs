//! The year: twelve months, a calendar of the days, and what changed.
//!
//! One route answers here, and it answers the three questions a running
//! log can answer about its own year:
//!
//! * `/insights/year` — the months with their own character, the calendar
//!   of the days coloured by the zones they were run in, and the diff
//!   against the athlete's own past.
//!
//! ## What is computed where
//!
//! The numbers are the **core's**: the days, the months, the window's
//! totals and the sentences of the diff all come from `stride-core`'s
//! year. This module reads them through the seam in [`crate::year`], which
//! is the one place the store's rows become the engine's inputs. It opens
//! no connection and re-derives nothing.
//!
//! ## The window
//!
//! The window is **the last twelve calendar months**, ending on the day the
//! answer is about, and the period it is compared against is **the same
//! number of days, ending a day earlier** — the rolling view's own
//! comparison. The answer names both, so a number is never read apart from
//! the period it describes.
//!
//! ## The privacy line
//!
//! **The calendar colours days, never places.** The days carry the zone
//! mix of that day's runs and nothing else: no position, no place name and
//! nothing that says where the athlete ran. Where they ran is the
//! heatmap's surface, and it has its own screen, its own disclosure and
//! its own rules.
//!
//! ## No value is the absence of the value
//!
//! A day with no run has **no distance**. A day whose runs carry no zone
//! times has **no mix**: the whole result is absent, which is not the same
//! as a mix of zero. A month with no runs has no numbers at all, and a
//! diff with nothing to compare has no line for that value.

use serde_json::Value;
use stride_core::rolling::RollingWindow;
use stride_core::units;
use stride_core::year::{DayFacts, MonthFacts};

use crate::dto::{
    RenderedValue, YearDay, YearInsight, YearMonth, YearWeek, YearWindow, YearZoneShare,
};
use crate::error::ApiResult;
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The year.
pub fn year(api: &Api, _request: &Request) -> ApiResult<Value> {
    let today = super::today(api)?;
    let window = stride_core::year::year_window(today);
    let comparison = window.previous();

    // The runs: the window's and the period before it. One read of the
    // store covers both.
    let from = comparison.from();
    let runs = crate::year::year_runs(api, &from.to_iso(), &today.to_iso())?;

    // The days of the two periods.
    let mut days = stride_core::year::days_of(&runs, window);
    let mut before = stride_core::year::days_of(&runs, comparison);

    // The form at the end of each month, and at the end of each period:
    // the load's own value, read over the same days and warmed with the
    // history the load's own window needs.
    let read_from = from.add_days(-crate::load::LOAD_WARMUP_DAYS);
    let forms = crate::year::forms(api, read_from, today)?;
    for day in days.iter_mut().chain(before.iter_mut()) {
        day.form = forms.get(&day.date).copied();
    }

    let months = stride_core::year::months_of(&days);
    let current = stride_core::year::totals_of(&days, window);
    let previous = stride_core::year::totals_of(&before, comparison);
    let changed = stride_core::year::what_changed(&current, &previous);

    let response = YearInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window: window_of(window),
        comparison: window_of(comparison),
        months: months.iter().map(month_of).collect(),
        days: days.iter().map(day_of).collect(),
        changed,
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// A window of days, as the answer names it.
fn window_of(window: RollingWindow) -> YearWindow {
    let from = window.from();
    let to = window.to;
    YearWindow {
        from: from.to_iso(),
        from_label: super::day_label(from),
        to: to.to_iso(),
        to_label: super::day_label(to),
        days: window.days,
    }
}

/// One month of the year.
fn month_of(month: &MonthFacts) -> YearMonth {
    YearMonth {
        key: month.key.clone(),
        label: month_label(&month.key),
        sessions: month.sessions,
        distance: render::distance(month.distance_m),
        duration: render::duration(month.moving_time_s),
        elevation: render::elevation(month.elevation_gain_m),
        longest: render::distance(month.longest_m),
        form: form_of(month.form),
        weeks: month.weeks.iter().map(week_of).collect(),
        mix: mix_of(month),
        high: render::share(month.high_share),
    }
}

/// The form at the end of a month, as the core renders it.
fn form_of(form: Option<f64>) -> Option<RenderedValue> {
    let form = form?;
    Some(RenderedValue {
        text: units::format_decimal(form, 1),
        raw: Some(form),
    })
}

/// One calendar week inside a month.
fn week_of(week: &stride_core::year::WeekFacts) -> YearWeek {
    YearWeek {
        start: week.start.to_iso(),
        start_label: super::day_label(week.start),
        distance: render::distance(week.distance_m),
        sessions: week.sessions,
    }
}

/// The mix of the zones a month was trained in.
fn mix_of(month: &MonthFacts) -> Vec<YearZoneShare> {
    let shares = match month.shares {
        Some(shares) => shares,
        None => return Vec::new(),
    };
    let mut mix: Vec<YearZoneShare> = Vec::with_capacity(5);
    for (index, share) in shares.iter().enumerate() {
        mix.push(YearZoneShare {
            zone: units::render_zone_label(index + 1).unwrap_or_default(),
            share: render::share(Some(*share)),
        });
    }
    mix
}

/// One day of the calendar.
fn day_of(day: &DayFacts) -> YearDay {
    YearDay {
        date: day.date.to_iso(),
        sessions: day.sessions,
        distance: render::distance(day.distance_m),
        high: render::share(day.high_share()),
    }
}

/// A month, as the api writes it: "May 2026".
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
