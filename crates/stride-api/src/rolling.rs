//! The seam to the core's rolling view.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's inputs: it reads the store
//! through its existing repositories, hands the core the facts, and reads
//! the core's answers back.
//!
//! It decides nothing. There is no rule about what a streak is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — the streaks, the window's totals, the split of the
//! training across the zones, the weekday and daytime splits and the
//! monthly and yearly tables all come from the core.
//!
//! ## What it reads
//!
//! * the **runs** — their local day, their distance, their moving time and
//!   the hour they started in the athlete's own local time;
//! * the **stored zone model** — the athlete's own configuration, read
//!   once from the watch and stored with the day it was read. A model that
//!   is not there is "no value", never a default;
//! * the **FIT files** of the runs in the window, decoded on read, so that
//!   the time in the zones can be measured against the stored model.

use std::collections::BTreeSet;

use chrono::Timelike;
use stride_core::date::Date;
use stride_core::profile::{facets, Facet};
use stride_core::rolling::{
    compare_windows, daytime_split, distance_breakdown, monthly_totals, polarised_shares,
    weekday_split, yearly_totals, DaytimeCuts, DaytimeTotals, DistanceBucket, PeriodTotals,
    RollingActivity, RollingWindow, WeekdayTotals, WindowComparison, ZoneShare, ZoneTimes,
};
use stride_core::streaks::{streaks, Streak, Streaks};
use stride_core::zones::{time_in_zone, ZoneFloors, ZoneSample};
use stride_store::ActivitySummary as StoredActivity;

use crate::error::ApiResult;
use crate::state::Api;

/// The stored zone model, as the rolling view reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredModel {
    /// The sport Garmin reported the model for.
    pub sport: String,
    /// The training method the model was set with.
    pub training_method: String,
    /// The maximum heart rate the model was built on, when Garmin reported
    /// one.
    pub max_heart_rate: Option<f64>,
    /// The model's five floors.
    pub floors: ZoneFloors,
    /// The day the model was read, in the athlete's own calendar.
    pub read_on: Date,
}

/// The split of the training across the athlete's zones.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneSplitAnswer {
    /// The shares of the time in the zones — the three bands, and how each
    /// of them moved against the window before it.
    ///
    /// `None` when there is no value: no model, an unusable model, or no
    /// heart-rate samples in the window.
    pub shares: Option<Vec<ZoneShare>>,
    /// The **distribution**: the time in each of the five zones and the
    /// share of the time in the zones it holds.
    ///
    /// `None` when there is no value, for the same reasons the bands have
    /// none.
    pub distribution: Option<Vec<ZoneDistributionRow>>,
    /// Why there is no split, in the core's own words.
    pub unavailable: Option<String>,
    /// The model the split was measured against, when there is one.
    pub model: Option<StoredModel>,
}

/// One zone of the distribution: the time in it and its share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneDistributionRow {
    /// The zone, from one.
    pub zone: usize,
    /// The time in the zone, in seconds.
    pub time_s: f64,
    /// The zone's share of the time in the zones, in per cent.
    pub share: f64,
}

/// The streaks of the athlete's history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreakAnswer {
    /// The run of days that is still going, if there is one.
    pub current_days: Option<Streak>,
    /// The longest run of days.
    pub longest_days: Option<Streak>,
    /// The run of weeks that is still going, if there is one.
    pub current_weeks: Option<Streak>,
    /// The longest run of weeks.
    pub longest_weeks: Option<Streak>,
    /// The run of months that is still going, if there is one.
    pub current_months: Option<Streak>,
    /// The longest run of months.
    pub longest_months: Option<Streak>,
}

impl From<Streaks> for StreakAnswer {
    fn from(streaks: Streaks) -> Self {
        Self {
            current_days: streaks.current_days,
            longest_days: streaks.longest_days,
            current_weeks: streaks.current_weeks,
            longest_weeks: streaks.longest_weeks,
            current_months: streaks.current_months,
            longest_months: streaks.longest_months,
        }
    }
}

/// The runs of a window of days, and the same runs as the core reads them.
pub struct WindowRuns {
    /// The stored runs.
    pub runs: Vec<StoredActivity>,
    /// The same runs, as the core's rolling view reads them.
    pub activities: Vec<RollingActivity>,
}

/// The runs between two days, as the core's rolling view reads them.
///
/// The hour is the hour the run **started**, in the athlete's own local
/// time: a run at 07:10 is a morning run wherever the athlete is.
pub fn window_runs(api: &Api, from: &str, to: &str) -> ApiResult<WindowRuns> {
    let runs = api
        .store()
        .activities_between(from, to)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let activities = runs
        .iter()
        .filter_map(|run| {
            let date = Date::parse(&run.local_date)?;
            Some(RollingActivity {
                date,
                distance_m: run.distance_m,
                moving_time_s: run.moving_time_s,
                elevation_gain_m: run.elevation_gain_m,
                start_hour: local_hour(api, run.start_time),
                average_hr: run.avg_hr,
                sport: run.sport_type.clone(),
            })
        })
        .collect();
    Ok(WindowRuns { runs, activities })
}

/// The hour of an instant, in the athlete's own local time.
fn local_hour(api: &Api, instant: chrono::DateTime<chrono::Utc>) -> u32 {
    let offset = chrono::Duration::minutes(api.utc_offset_minutes() as i64);
    (instant + offset).hour()
}

/// The window's totals and the window before it.
///
/// The window before it is the same number of days, ending a day earlier:
/// that is the period the answer compares itself against.
pub fn window_comparison(
    activities: &[RollingActivity],
    window: RollingWindow,
) -> WindowComparison {
    compare_windows(activities, window)
}

/// The split of the training across the athlete's zones, over a window.
///
/// The shares are measured against the athlete's **stored model**: the
/// watch's own configuration, with the day it was read. The time in the
/// zones comes from the runs' own FIT files, decoded on read, and only the
/// moving samples — the same filter the zones route has always used.
///
/// There is no value when there is no model, when the model's floors do
/// not increase, or when the runs in the window carry no heart-rate
/// samples. None of those is a zero.
pub fn zone_split(
    api: &Api,
    runs: &[&StoredActivity],
    window: RollingWindow,
) -> ApiResult<ZoneSplitAnswer> {
    let model = match stored_model(api)? {
        Some(model) => model,
        None => {
            return Ok(ZoneSplitAnswer {
                shares: None,
                distribution: None,
                unavailable: core_line("zones.no_model"),
                model: None,
            })
        }
    };
    if !model.floors.is_usable() {
        return Ok(ZoneSplitAnswer {
            shares: None,
            distribution: None,
            unavailable: core_line("zones.unusable_model"),
            model: Some(model),
        });
    }

    let mut current = ZoneTimes::default();
    let mut previous = ZoneTimes::default();
    for run in runs {
        let date = match Date::parse(&run.local_date) {
            Some(date) => date,
            None => continue,
        };
        let Some(time) = time_in_zone(model.floors, &zone_samples(api, run.id)) else {
            continue;
        };
        if window.contains(date) {
            add(&mut current, &time);
        }
        if window.previous().contains(date) {
            add(&mut previous, &time);
        }
    }

    let shares = polarised_shares(&current, &previous);
    // The distribution: the time in each of the five zones and the share
    // of the time in the zones it holds.
    let distribution = current.shares().map(|shares| {
        let times = current.per_zone();
        let mut rows: Vec<ZoneDistributionRow> = Vec::with_capacity(5);
        for index in 0..5 {
            rows.push(ZoneDistributionRow {
                zone: index + 1,
                time_s: times[index],
                share: shares[index],
            });
        }
        rows
    });
    let unavailable = if shares.is_some() {
        None
    } else {
        core_line("rolling.zones.no_samples")
    };
    Ok(ZoneSplitAnswer {
        shares,
        distribution,
        unavailable,
        model: Some(model),
    })
}

/// The time in the zones of one run, added to a total.
fn add(total: &mut ZoneTimes, time: &stride_core::zones::TimeInZone) {
    let times = time.per_zone();
    total.zone1_s += times[0];
    total.zone2_s += times[1];
    total.zone3_s += times[2];
    total.zone4_s += times[3];
    total.zone5_s += times[4];
}

/// The stored zone model, when there is one.
///
/// `None` when no model has been read yet: the split then answers "no
/// value", with the reason, and never a guessed maximum heart rate.
pub fn stored_model(api: &Api) -> ApiResult<Option<StoredModel>> {
    let model = api
        .store()
        .current_hr_zone_model()
        .map_err(|error| api.store_error("the zone model could not be read", &error))?;
    let model = match model {
        Some(model) => model,
        None => return Ok(None),
    };
    // The day the model was read, in the athlete's own calendar.
    let offset = chrono::Duration::minutes(api.utc_offset_minutes() as i64);
    let naive = (model.read_at + offset).naive_utc();
    let read_on = match Date::parse(&naive.format("%Y-%m-%d").to_string()) {
        Some(date) => date,
        None => return Ok(None),
    };
    Ok(Some(StoredModel {
        sport: model.sport.clone(),
        training_method: model.training_method.clone(),
        max_heart_rate: model.max_heart_rate,
        floors: ZoneFloors {
            zone1: model.zone1_floor,
            zone2: model.zone2_floor,
            zone3: model.zone3_floor,
            zone4: model.zone4_floor,
            zone5: model.zone5_floor,
        },
        read_on,
    }))
}

/// The heart-rate samples of a run, decoded on read.
fn zone_samples(api: &Api, run_id: i64) -> Vec<ZoneSample> {
    match crate::routes::series::decoded(api, run_id) {
        Some(decoded) => crate::routes::series::zone_samples(&decoded),
        None => Vec::new(),
    }
}

/// The streaks of the athlete's history.
///
/// The days are the distinct local days that carry a run, and they are
/// read from the whole history: a streak is a fact about everything that
/// came before, not about a window.
pub fn streak_answer(api: &Api, today: Date) -> ApiResult<StreakAnswer> {
    let runs = api
        .store()
        .activities_between("0000-01-01", &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut days: BTreeSet<String> = BTreeSet::new();
    for run in runs.iter() {
        if Date::parse(&run.local_date).is_some() {
            days.insert(run.local_date.clone());
        }
    }
    let days: Vec<Date> = days.iter().filter_map(|day| Date::parse(day)).collect();
    Ok(streaks(&days, today).into())
}

/// The weekday split of the history.
pub fn weekday_split_of(activities: &[RollingActivity]) -> Vec<WeekdayTotals> {
    weekday_split(activities)
}

/// The daytime split of the history.
pub fn daytime_split_of(activities: &[RollingActivity], cuts: DaytimeCuts) -> Vec<DaytimeTotals> {
    daytime_split(activities, cuts)
}

/// The monthly and the yearly tables of the history.
pub fn period_tables(activities: &[RollingActivity]) -> (Vec<PeriodTotals>, Vec<PeriodTotals>) {
    (monthly_totals(activities), yearly_totals(activities))
}

/// The distance breakdown of a window of runs.
pub fn distance_buckets(activities: &[RollingActivity]) -> Vec<DistanceBucket> {
    distance_breakdown(activities)
}

/// The seven facets of the athlete's shape, and how they moved.
pub fn profile_facets(current: &[RollingActivity], previous: &[RollingActivity]) -> Vec<Facet> {
    facets(current, previous)
}

/// A sentence from the core's catalogue.
///
/// The api never writes a sentence of its own: it asks the core's
/// catalogue for it and passes it on.
pub fn core_line(id: &str) -> Option<String> {
    stride_core::strings::render(id, &[])
}
