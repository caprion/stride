//! The year: the months, the calendar, and what changed.
//!
//! A running log can answer three questions about its own year, and all
//! three are facts about what happened:
//!
//! * **what each month held** — the distance, the runs, the longest run,
//!   the form at the month's end, the shape of the weeks inside it and the
//!   mix of the zones the month was trained in;
//! * **what each day held** — the zone mix of the day's runs, which is
//!   what the calendar is coloured with;
//! * **what changed** — the diff against the athlete's own past, in
//!   sentences.
//!
//! Everything here is **recomputed on read** and **never stored**, like
//! every other derived series in this crate. The calendar's colours are
//! **days**, never places: a day says *when* and *how hard*, and nothing
//! here says *where* — that is the heatmap's surface, and it has its own
//! screen and its own disclosure.
//!
//! ## The window
//!
//! The year is **the last twelve calendar months**, ending on the day the
//! answer is about ([`year_window`]), and the period it is compared
//! against is **the same span, seven days earlier** — the one comparison
//! rule the core has ([`Window::previous`]). The current month is the
//! month so far, and the card says so by showing the days it holds.
//!
//! ## No value is the absence of the value
//!
//! A day with no run has **no distance** — not a distance of zero. A day
//! whose runs carry no zone times — no file, no model, no heart-rate
//! samples — has **no zone mix**: the whole result is absent, which is
//! not the same as a mix of zero. A month with no runs has no numbers at
//! all, and a window with no run has no pace.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::plan::week_start;
use crate::rolling::{PolarisedZone, Window, ZoneTimes};
use crate::strings;
use crate::units;

/// How many calendar months the year holds.
pub const YEAR_MONTHS: usize = 12;

/// One run, as the year reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YearRun {
    /// The athlete's local day the run belongs to.
    pub date: Date,
    /// The run's distance, in metres.
    pub distance_m: f64,
    /// How long the run moved, in seconds.
    pub moving_time_s: f64,
    /// The run's elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The time the run spent in the athlete's own zones.
    ///
    /// `None` when the run carries no zone times: no stored model, no file
    /// or no heart-rate samples. The day the run falls on then has **no
    /// mix** — never a mix of zero.
    pub times: Option<ZoneTimes>,
}

/// One day of the year, as the year reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayFacts {
    /// The day.
    pub date: Date,
    /// How many runs fell on the day.
    pub sessions: usize,
    /// The day's distance, in metres. `None` on a day with no run: there is
    /// no distance to report, which is not a distance of zero.
    pub distance_m: Option<f64>,
    /// The day's moving time, in seconds. `None` on a day with no run.
    pub moving_time_s: Option<f64>,
    /// The day's elevation gain, in metres. `None` on a day with no run.
    pub elevation_gain_m: Option<f64>,
    /// The longest run of the day, in metres. `None` on a day with no run.
    pub longest_m: Option<f64>,
    /// The time the day's runs spent in each of the five zones.
    ///
    /// `None` when no run of the day carries zone times. The mix is the
    /// time of the runs that could be measured, and a day whose runs carry
    /// none has **no mix at all**.
    pub times: Option<ZoneTimes>,
    /// The training load's form on the day. `None` when the load has no
    /// value for the day.
    pub form: Option<f64>,
}

impl DayFacts {
    /// A day with nothing on it.
    fn empty(date: Date) -> Self {
        Self {
            date,
            sessions: 0,
            distance_m: None,
            moving_time_s: None,
            elevation_gain_m: None,
            longest_m: None,
            times: None,
            form: None,
        }
    }

    /// Each zone's share of the time in the zones, in per cent.
    ///
    /// `None` when there is no time in the zones at all: that is no
    /// value, never a row of zeros.
    pub fn shares(&self) -> Option<[f64; 5]> {
        self.times.as_ref().and_then(|times| times.shares())
    }

    /// The share of the time in the zones that was in the high zones
    /// (Z4–Z5), in per cent.
    ///
    /// This is what the calendar colours a day by: how much of that day's
    /// running sat in the high zones. `None` when there is no time in the
    /// zones.
    pub fn high_share(&self) -> Option<f64> {
        let times = self.times.as_ref()?;
        let total = times.total_s();
        if total <= 0.0 {
            return None;
        }
        Some(units::round_half_even(
            times.band_s(PolarisedZone::High) / total * 100.0,
            2,
        ))
    }
}

/// One calendar week inside a month.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekFacts {
    /// The first day of the week — a Monday, in the athlete's own calendar.
    pub start: Date,
    /// The distance of the week's days that fall in the month, in metres.
    /// `None` when the week holds no run in the month.
    pub distance_m: Option<f64>,
    /// How many runs fell in the week's days that are in the month.
    pub sessions: usize,
}

/// One month of the year.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonthFacts {
    /// The month's key: "2026-05".
    pub key: String,
    /// How many runs fell in the month.
    pub sessions: usize,
    /// The month's distance, in metres. `None` when the month holds no run.
    pub distance_m: Option<f64>,
    /// The month's moving time, in seconds. `None` when it holds no run.
    pub moving_time_s: Option<f64>,
    /// The month's elevation gain, in metres. `None` when it holds no run.
    pub elevation_gain_m: Option<f64>,
    /// The longest run of the month, in metres. `None` when it holds no run.
    pub longest_m: Option<f64>,
    /// The weeks the month touches, oldest first.
    ///
    /// A week with nothing in it is a week with no distance: the shape of
    /// the month keeps the gap.
    pub weeks: Vec<WeekFacts>,
    /// The time the month's runs spent in each of the five zones.
    pub times: Option<ZoneTimes>,
    /// Each zone's share of the time in the zones, in per cent.
    pub shares: Option<[f64; 5]>,
    /// The share of the time in the zones that was in the high zones.
    pub high_share: Option<f64>,
    /// The training load's form at the month's end.
    ///
    /// The form of the **last day of the month that carries one**: the
    /// month so far ends with the last day the record covers.
    pub form: Option<f64>,
}

/// What a window of days holds, in the terms the year speaks in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct YearTotals {
    /// The window's distance, in metres.
    pub distance_m: f64,
    /// The window's moving time, in seconds.
    pub moving_time_s: f64,
    /// How many runs the window holds.
    pub sessions: usize,
    /// The longest run of the window, in metres.
    pub longest_m: f64,
    /// The window's average pace, in seconds per kilometre. `None` when the
    /// window holds no distance: there is no pace without a distance.
    pub pace_s_per_km: Option<f64>,
    /// The training load's form at the end of the window.
    pub form: Option<f64>,
}

/// The window the year is: the last twelve calendar months.
///
/// The window ends on the day the answer is about, and it starts on the
/// first day of the month eleven months before it — twelve calendar months,
/// the current one included and the month so far only as far as today.
pub fn year_window(today: Date) -> Window {
    let mut from = first_of_month(today);
    for _ in 1..YEAR_MONTHS {
        from = first_of_month(from.add_days(-1));
    }
    Window::span(from, today)
}

/// The first day of the month a day is in.
fn first_of_month(date: Date) -> Date {
    Date::from_ymd(date.year(), date.month(), 1).unwrap_or(date)
}

/// The key of the month a day is in: "2026-05".
fn month_key(date: Date) -> String {
    format!("{:04}-{:02}", date.year(), date.month())
}

/// Every day of the window, with the runs that fell on it.
///
/// The days are the window's days, oldest first, and a day with no run is
/// a day with **no values**: no distance, no time, no mix. The day's zone
/// mix is the time of the runs that could be measured, and a day whose
/// runs carry none has no mix at all.
pub fn days_of(runs: &[YearRun], window: Window) -> Vec<DayFacts> {
    let mut by_day: BTreeMap<String, Vec<&YearRun>> = BTreeMap::new();
    for run in runs {
        if !window.contains(run.date) {
            continue;
        }
        by_day.entry(run.date.to_iso()).or_default().push(run);
    }

    let mut days: Vec<DayFacts> = Vec::new();
    let mut day = window.from();
    while day <= window.to() {
        let runs = by_day.get(&day.to_iso()).map(|runs| runs.as_slice());
        days.push(day_of(day, runs));
        day = day.add_days(1);
    }
    days
}

/// One day, from the runs that fell on it.
fn day_of(date: Date, runs: Option<&[&YearRun]>) -> DayFacts {
    let runs = match runs {
        Some(runs) => runs,
        None => return DayFacts::empty(date),
    };
    if runs.is_empty() {
        return DayFacts::empty(date);
    }

    let mut distance_m = 0.0;
    let mut moving_time_s = 0.0;
    let mut elevation_gain_m = 0.0;
    let mut longest_m: Option<f64> = None;
    let mut times: Option<ZoneTimes> = None;
    for run in runs {
        distance_m += run.distance_m;
        moving_time_s += run.moving_time_s;
        elevation_gain_m += run.elevation_gain_m;
        longest_m = Some(match longest_m {
            Some(longest) => longest.max(run.distance_m),
            None => run.distance_m,
        });
        if let Some(run_times) = run.times {
            let total = times.get_or_insert_with(ZoneTimes::default);
            let per_zone = run_times.per_zone();
            total.zone1_s += per_zone[0];
            total.zone2_s += per_zone[1];
            total.zone3_s += per_zone[2];
            total.zone4_s += per_zone[3];
            total.zone5_s += per_zone[4];
        }
    }

    DayFacts {
        date,
        sessions: runs.len(),
        distance_m: Some(distance_m),
        moving_time_s: Some(moving_time_s),
        elevation_gain_m: Some(elevation_gain_m),
        longest_m,
        times,
        form: None,
    }
}

/// The months the days fall into, oldest first.
///
/// A month with no run is **a month with no numbers**: it is in the year
/// because the year is twelve months, and the numbers it carries are none
/// of them. The weeks are the weeks the month touches, and a week with
/// nothing in it keeps its place in the shape.
pub fn months_of(days: &[DayFacts]) -> Vec<MonthFacts> {
    // The days of each month, in order.
    let mut grouped: Vec<(String, Vec<DayFacts>)> = Vec::new();
    for day in days {
        let key = month_key(day.date);
        match grouped.iter_mut().find(|(month, _)| *month == key) {
            Some((_, month_days)) => month_days.push(*day),
            None => grouped.push((key, vec![*day])),
        }
    }

    let mut months: Vec<MonthFacts> = Vec::new();
    for (key, month_days) in grouped {
        let mut month = MonthFacts {
            key,
            sessions: 0,
            distance_m: None,
            moving_time_s: None,
            elevation_gain_m: None,
            longest_m: None,
            weeks: weeks_of(&month_days),
            times: None,
            shares: None,
            high_share: None,
            form: None,
        };
        for day in month_days {
            month.sessions += day.sessions;
            if let Some(distance) = day.distance_m {
                month.distance_m = Some(month.distance_m.unwrap_or(0.0) + distance);
            }
            if let Some(moving) = day.moving_time_s {
                month.moving_time_s = Some(month.moving_time_s.unwrap_or(0.0) + moving);
            }
            if let Some(elevation) = day.elevation_gain_m {
                month.elevation_gain_m = Some(month.elevation_gain_m.unwrap_or(0.0) + elevation);
            }
            if let Some(longest) = day.longest_m {
                month.longest_m = Some(match month.longest_m {
                    Some(current) => current.max(longest),
                    None => longest,
                });
            }
            if let Some(times) = day.times {
                let total = month.times.get_or_insert_with(ZoneTimes::default);
                let per_zone = times.per_zone();
                total.zone1_s += per_zone[0];
                total.zone2_s += per_zone[1];
                total.zone3_s += per_zone[2];
                total.zone4_s += per_zone[3];
                total.zone5_s += per_zone[4];
            }
            // The form at the month's end: the last day of the month that
            // carries one.
            if day.form.is_some() {
                month.form = day.form;
            }
        }
        month.shares = month.times.as_ref().and_then(|times| times.shares());
        month.high_share = month.times.as_ref().and_then(|times| {
            let total = times.total_s();
            if total <= 0.0 {
                return None;
            }
            Some(units::round_half_even(
                times.band_s(PolarisedZone::High) / total * 100.0,
                2,
            ))
        });
        months.push(month);
    }
    months
}

/// The weeks a month's days fall into, oldest first.
pub fn weeks_of(days: &[DayFacts]) -> Vec<WeekFacts> {
    let mut weeks: Vec<WeekFacts> = Vec::new();
    for day in days {
        let start = week_start(day.date);
        let index = match weeks.iter().position(|week| week.start == start) {
            Some(index) => index,
            None => {
                weeks.push(WeekFacts {
                    start,
                    distance_m: None,
                    sessions: 0,
                });
                weeks.len() - 1
            }
        };
        let week = &mut weeks[index];
        week.sessions += day.sessions;
        if let Some(distance) = day.distance_m {
            week.distance_m = Some(week.distance_m.unwrap_or(0.0) + distance);
        }
    }
    weeks
}

/// What a window of days holds.
///
/// The totals are values: a window with no run holds 0 km, and that is a
/// fact about the window rather than a missing measurement. The **pace**
/// and the **form** are the values that are absent when they cannot be
/// computed.
pub fn totals_of(days: &[DayFacts], window: Window) -> YearTotals {
    let mut totals = YearTotals {
        distance_m: 0.0,
        moving_time_s: 0.0,
        sessions: 0,
        longest_m: 0.0,
        pace_s_per_km: None,
        form: None,
    };
    for day in days {
        if !window.contains(day.date) {
            continue;
        }
        totals.sessions += day.sessions;
        if let Some(distance) = day.distance_m {
            totals.distance_m += distance;
        }
        if let Some(moving) = day.moving_time_s {
            totals.moving_time_s += moving;
        }
        if let Some(longest) = day.longest_m {
            totals.longest_m = totals.longest_m.max(longest);
        }
        // The form at the end of the window: the last day of it that
        // carries one.
        if day.form.is_some() {
            totals.form = day.form;
        }
    }
    totals.pace_s_per_km = if totals.distance_m > 0.0 {
        Some(units::round_pace_s_per_km(
            totals.moving_time_s / (totals.distance_m / 1000.0),
        ))
    } else {
        None
    };
    totals
}

/// What changed: the diff against the athlete's own past, in sentences.
///
/// **The sentences are the core's.** They are written by the catalogue and
/// travel to the app already rendered: the app writes no sentence of its
/// own. There is no verdict in any of them and nothing that tells the
/// athlete what to do — the diff says what the records say, and the reader
/// draws the conclusion.
///
/// A value that cannot be computed has **no line at all**: the diff never
/// shows a zero where there is nothing to measure.
pub fn what_changed(current: &YearTotals, previous: &YearTotals) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if let Some(line) = distance_line(current, previous) {
        lines.push(line);
    }
    if let Some(line) = sessions_line(current, previous) {
        lines.push(line);
    }
    if let Some(line) = longest_line(current, previous) {
        lines.push(line);
    }
    if let Some(line) = pace_line(current, previous) {
        lines.push(line);
    }
    if let Some(line) = form_line(current, previous) {
        lines.push(line);
    }
    lines
}

/// The distance, and the difference between the two windows.
fn distance_line(current: &YearTotals, previous: &YearTotals) -> Option<String> {
    let difference = units::render_difference(
        Some(current.distance_m - previous.distance_m),
        units::ValueUnit::Distance,
    )?;
    let now = units::render_distance(Some(current.distance_m))?;
    let before = units::render_distance(Some(previous.distance_m))?;
    strings::render(
        "year.changed.distance",
        &[
            ("difference", &difference),
            ("current", &now),
            ("previous", &before),
        ],
    )
}

/// The runs, and the difference between the two windows.
fn sessions_line(current: &YearTotals, previous: &YearTotals) -> Option<String> {
    let difference = units::render_difference(
        Some(current.sessions as f64 - previous.sessions as f64),
        units::ValueUnit::Count,
    )?;
    let now = units::format_whole(current.sessions as f64);
    let before = units::format_whole(previous.sessions as f64);
    strings::render(
        "year.changed.sessions",
        &[
            ("difference", &difference),
            ("current", &now),
            ("previous", &before),
        ],
    )
}

/// The longest run, and which way it moved.
fn longest_line(current: &YearTotals, previous: &YearTotals) -> Option<String> {
    let now = units::render_distance(Some(current.longest_m))?;
    let before = units::render_distance(Some(previous.longest_m))?;
    let gap = current.longest_m - previous.longest_m;
    if gap > 0.0 {
        let gap = units::render_distance(Some(gap))?;
        return strings::render(
            "year.changed.longest.grew",
            &[("gap", &gap), ("current", &now), ("previous", &before)],
        );
    }
    if gap < 0.0 {
        let gap = units::render_distance(Some(-gap))?;
        return strings::render(
            "year.changed.longest.shortened",
            &[("gap", &gap), ("current", &now), ("previous", &before)],
        );
    }
    strings::render(
        "year.changed.longest.same",
        &[("current", &now), ("previous", &before)],
    )
}

/// The average pace of the two windows.
fn pace_line(current: &YearTotals, previous: &YearTotals) -> Option<String> {
    let now = units::render_pace(current.pace_s_per_km)?;
    match units::render_pace(previous.pace_s_per_km) {
        Some(before) => strings::render(
            "year.changed.pace",
            &[("current", &now), ("previous", &before)],
        ),
        None => strings::render("year.changed.pace.one", &[("current", &now)]),
    }
}

/// The form at the end of the two windows.
fn form_line(current: &YearTotals, previous: &YearTotals) -> Option<String> {
    let now = units::format_decimal(current.form?, 1);
    let before = units::format_decimal(previous.form?, 1);
    strings::render(
        "year.changed.form",
        &[("current", &now), ("previous", &before)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    fn floors() -> ZoneTimes {
        ZoneTimes {
            zone1_s: 60.0,
            zone2_s: 120.0,
            zone3_s: 180.0,
            zone4_s: 240.0,
            zone5_s: 300.0,
        }
    }

    fn run(day: &str, distance_m: f64, times: Option<ZoneTimes>) -> Option<YearRun> {
        Some(YearRun {
            date: date(day)?,
            distance_m,
            moving_time_s: distance_m / 3.0,
            elevation_gain_m: 10.0,
            times,
        })
    }

    fn runs(list: &[Option<YearRun>]) -> Option<Vec<YearRun>> {
        let mut out: Vec<YearRun> = Vec::new();
        for run in list {
            out.push(*run.as_ref()?);
        }
        Some(out)
    }

    #[test]
    fn a_day_of_one_run_carries_its_mix() {
        let runs = match runs(&[run("2026-05-10", 10_000.0, Some(floors()))]) {
            Some(runs) => runs,
            None => return,
        };
        let window = match date("2026-05-10") {
            Some(day) => Window::of_days(3, day),
            None => return,
        };
        let days = days_of(&runs, window);
        assert_eq!(days.len(), 3);
        let day = days[0];
        assert_eq!(day.date.to_iso(), "2026-05-08");
        // The day before the run: no run, and so no values at all.
        assert_eq!(day.sessions, 0);
        assert_eq!(day.distance_m, None);
        assert_eq!(day.shares(), None);
        assert_eq!(day.high_share(), None);

        let day = days[2];
        assert_eq!(day.sessions, 1);
        assert_eq!(day.distance_m, Some(10_000.0));
        // 240 + 300 of 900 seconds were in the high zones.
        assert_eq!(day.high_share(), Some(60.0));
        let shares = match day.shares() {
            Some(shares) => shares,
            None => return,
        };
        assert_eq!(shares[0], 6.67);
        assert_eq!(shares[4], 33.33);
    }

    #[test]
    fn a_day_with_a_run_that_has_no_file_has_no_mix() {
        // The run is there — it has a distance — but its file carries no
        // heart rate, so the day has no mix at all. That is no value, not
        // a mix of zero.
        let runs = match runs(&[run("2026-05-10", 10_000.0, None)]) {
            Some(runs) => runs,
            None => return,
        };
        let window = match date("2026-05-10") {
            Some(day) => Window::of_days(1, day),
            None => return,
        };
        let days = days_of(&runs, window);
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].sessions, 1);
        assert_eq!(days[0].distance_m, Some(10_000.0));
        assert_eq!(days[0].times, None);
        assert_eq!(days[0].shares(), None);
        assert_eq!(days[0].high_share(), None);
    }

    #[test]
    fn the_window_is_the_last_twelve_calendar_months() {
        let today = match date("2026-05-13") {
            Some(day) => day,
            None => return,
        };
        let window = year_window(today);
        assert_eq!(window.from().to_iso(), "2025-06-01");
        assert_eq!(window.to().to_iso(), "2026-05-13");
        assert_eq!(window.days(), 347);
        // The period it is compared against is the same span, seven days
        // earlier.
        assert_eq!(window.previous().to().to_iso(), "2026-05-06");
        assert_eq!(window.previous().from().to_iso(), "2025-05-25");
        assert_eq!(window.previous().days(), 347);
    }

    #[test]
    fn a_month_holds_its_runs_its_longest_and_its_shape() {
        let runs = match runs(&[
            run("2026-05-04", 10_000.0, Some(floors())),
            run("2026-05-07", 21_000.0, Some(floors())),
            run("2026-04-30", 5_000.0, Some(floors())),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let today = match date("2026-05-13") {
            Some(day) => day,
            None => return,
        };
        let window = year_window(today);
        let days = days_of(&runs, window);
        let months = months_of(&days);
        assert_eq!(months.len(), 12);
        // April: one run of 5 km.
        let april = &months[10];
        assert_eq!(april.key, "2026-04");
        assert_eq!(april.sessions, 1);
        assert_eq!(april.distance_m, Some(5_000.0));
        assert_eq!(april.longest_m, Some(5_000.0));
        // May: two runs, and the longer one is the month's longest.
        let may = &months[11];
        assert_eq!(may.key, "2026-05");
        assert_eq!(may.sessions, 2);
        assert_eq!(may.distance_m, Some(31_000.0));
        assert_eq!(may.longest_m, Some(21_000.0));
        // The weeks the month touches: the week of the 27th of April, and
        // the weeks of the 4th and the 11th of May.
        assert_eq!(may.weeks.len(), 3);
        assert_eq!(may.weeks[0].start.to_iso(), "2026-04-27");
        // The week the month starts in holds the run of the 30th of
        // April, but that run is not the month's: the week keeps its
        // place in the shape and holds nothing of May's.
        assert_eq!(may.weeks[0].distance_m, None);
        assert_eq!(may.weeks[1].start.to_iso(), "2026-05-04");
        assert_eq!(may.weeks[1].distance_m, Some(31_000.0));
        // The zone mix of the month: both runs were in the same zones, so
        // the shares are the same as one run's.
        assert_eq!(may.high_share, Some(60.0));
    }

    #[test]
    fn a_month_with_no_run_has_no_numbers() {
        let today = match date("2026-05-13") {
            Some(day) => day,
            None => return,
        };
        let window = year_window(today);
        let days = days_of(&[], window);
        let months = months_of(&days);
        assert_eq!(months.len(), 12);
        for month in months {
            assert_eq!(month.sessions, 0);
            assert_eq!(month.distance_m, None);
            assert_eq!(month.longest_m, None);
            assert_eq!(month.shares, None);
            assert_eq!(month.high_share, None);
        }
    }

    #[test]
    fn the_totals_of_a_window_are_a_fact_about_the_window() {
        let runs = match runs(&[
            run("2026-05-10", 10_000.0, Some(floors())),
            run("2026-05-11", 20_000.0, Some(floors())),
            run("2026-05-20", 8_000.0, Some(floors())),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let today = match date("2026-05-20") {
            Some(day) => day,
            None => return,
        };
        let window = Window::of_days(11, today);
        let days = days_of(&runs, window);
        let totals = totals_of(&days, window);
        assert_eq!(totals.sessions, 3);
        assert_eq!(totals.distance_m, 38_000.0);
        assert_eq!(totals.longest_m, 20_000.0);
        // The pace of the window: the moving time over the distance.
        assert_eq!(totals.pace_s_per_km, Some(333.33));
        // The period it is compared against is the same span, seven days
        // earlier: the 3rd to the 13th of May, which holds two of the
        // three runs.
        let before_days = days_of(&runs, window.previous());
        let before = totals_of(&before_days, window.previous());
        assert_eq!(before.sessions, 2);
        assert_eq!(before.distance_m, 30_000.0);
        assert_eq!(before.longest_m, 20_000.0);
    }

    #[test]
    fn what_changed_says_the_facts_and_nothing_else() {
        let current = YearTotals {
            distance_m: 217_300.0,
            moving_time_s: 217_300.0 * 431.0,
            sessions: 26,
            longest_m: 21_300.0,
            pace_s_per_km: Some(431.0),
            form: Some(12.3),
        };
        let previous = YearTotals {
            distance_m: 166_300.0,
            moving_time_s: 166_300.0 * 435.0,
            sessions: 17,
            longest_m: 18_900.0,
            pace_s_per_km: Some(435.0),
            form: Some(-3.4),
        };
        let lines = what_changed(&current, &previous);
        assert_eq!(lines.len(), 5);
        assert_eq!(
            lines[0],
            "+51 km against the same days, a week earlier (217.3 km against 166.3 km)"
        );
        assert_eq!(lines[1], "+9 recorded sessions (26 against 17)");
        assert_eq!(
            lines[2],
            "the longest run grew by 2.4 km (21.3 km against 18.9 km)"
        );
        assert_eq!(lines[3], "the average pace was 7:11 /km against 7:15 /km");
        assert_eq!(
            lines[4],
            "the difference was 12.3 at the end of this period against -3.4 at the end of the same days, a week earlier"
        );
        // Nothing in the diff is a verdict or an instruction.
        for line in lines {
            let text = line.to_lowercase();
            for word in ["should", "must", "good", "bad", "better", "worse", "poor"] {
                assert!(!text.contains(word), "\"{line}\" carries \"{word}\"");
            }
        }
    }

    #[test]
    fn a_diff_with_nothing_to_compare_has_no_line() {
        let empty = YearTotals {
            distance_m: 0.0,
            moving_time_s: 0.0,
            sessions: 0,
            longest_m: 0.0,
            pace_s_per_km: None,
            form: None,
        };
        // The distance and the sessions are facts about the window, and
        // they are said; the pace and the form have no value, and they are
        // not said at all.
        let lines = what_changed(&empty, &empty);
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            "0 km against the same days, a week earlier (0 km against 0 km)"
        );
        assert_eq!(lines[1], "0 recorded sessions (0 against 0)");
        assert_eq!(lines[2], "the longest run was 0 km in both periods");
    }
}
