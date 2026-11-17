//! The seam to the core's training-load engine.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's inputs: it reads the store
//! through its existing repositories, hands the core the facts, and reads
//! the core's answers back.
//!
//! It decides nothing. There is no rule about what a load is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — the daily loads, the acute and chronic loads, the
//! form, the ratio, the week and the recovery scenario all come from the
//! core.
//!
//! ## The three facts it reads
//!
//! * the **zone model** — the athlete's own stored configuration, read once
//!   from the watch and stored with the day it was read. It is read through
//!   the store's own repository, so there is **one** way to get the zones:
//!   the same one the time-in-zone computation reads. A model that is not
//!   there is "no value", never a default;
//! * the **resting heart rate** — the measured one from the daily summary.
//!   The core falls back to the most recent reading on or before the day,
//!   so the readings before the window are read as well;
//! * the **activities** — the stored moving time and average heart rate.

use std::collections::BTreeSet;

use stride_core::date::Date;
use stride_core::load::{
    ActivityLoad, DayInput, LoadAnswer, LoadInputs, LoadZoneModel, RestingHrReading,
};
use stride_core::types::{HeartRate, RestingHr};
use stride_core::zones::ZoneFloors;

use crate::error::ApiResult;
use crate::state::Api;

/// How many days of history the load is read over.
///
/// The chronic series is weighted over forty-two days and the week the
/// impulse is measured over needs seven more, so the series is read over
/// forty-nine days: the chronic load is warm when it is read, and the week
/// is there.
pub const LOAD_HISTORY_DAYS: i64 = 49;

/// How many days of history the series is **warmed** with, before the
/// window the screen is showing.
///
/// The two series are exponentially weighted, and the chronic one is
/// weighted over forty-two days: a series that begins at the window's
/// first day has a chronic load of zero there, and the numbers at the
/// start of the window would be the numbers of a series that began there.
/// So the series is read over the window **plus** this many days, and the
/// window's own numbers are the ones the warmed series gives.
///
/// The number is the reference implementation's own: it warms its
/// exponentially weighted series with 210 days before the window it draws
/// (`TrainingLoadChart::EMA_WARMUP_DAYS`), and after 210 days the weight
/// still carried by the first day's seed is `exp(−210/42)`, about two
/// thirds of one per cent.
pub const LOAD_WARMUP_DAYS: i64 = 210;

/// The load of the days from `from` to `to`, read from the store.
pub fn load_report(api: &Api, from: Date, to: Date) -> ApiResult<LoadAnswer> {
    let inputs = load_inputs(api, from, to)?;
    Ok(stride_core::load::load_report(&inputs))
}

/// The inputs of the load engine, read from the store.
///
/// The days of the series are the days the record covers: a day with an
/// activity, and a day the watch recorded and the athlete did not train on
/// — a **rest day**, whose load is zero. A day the record does not cover is
/// left out of the series: it is missing, not a rest day.
pub fn load_inputs(api: &Api, from: Date, to: Date) -> ApiResult<LoadInputs> {
    let from_text = from.to_iso();
    let to_text = to.to_iso();

    let runs = api
        .store()
        .activities_between(&from_text, &to_text)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    // The resting heart rate of a day falls back to the most recent reading
    // on or before it, so every reading up to the last day of the window is
    // read — not only the ones inside the window.
    let wellness = api
        .store()
        .wellness_between("0000-01-01", &to_text)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;

    let mut activities: Vec<ActivityLoad> = Vec::with_capacity(runs.len());
    let mut activity_days: BTreeSet<String> = BTreeSet::new();
    for run in runs.iter() {
        let date = match Date::parse(&run.local_date) {
            Some(date) => date,
            None => continue,
        };
        activity_days.insert(date.to_iso());
        activities.push(ActivityLoad {
            date,
            moving_time_s: run.moving_time_s,
            average_hr: run.avg_hr.and_then(HeartRate::from_bpm),
        });
    }

    let mut resting_hr: Vec<RestingHrReading> = Vec::new();
    let mut wellness_days: BTreeSet<String> = BTreeSet::new();
    for row in wellness.iter() {
        let date = match Date::parse(&row.date) {
            Some(date) => date,
            None => continue,
        };
        wellness_days.insert(date.to_iso());
        if let Some(resting) = row.resting_hr.and_then(RestingHr::from_bpm) {
            resting_hr.push(RestingHrReading {
                date,
                resting_hr: resting,
            });
        }
    }

    let mut days: Vec<DayInput> = Vec::new();
    let mut covered: BTreeSet<String> = activity_days.clone();
    covered.extend(wellness_days.iter().cloned());
    for text in covered {
        if text < from_text || text > to_text {
            continue;
        }
        let date = match Date::parse(&text) {
            Some(date) => date,
            None => continue,
        };
        // A day with an activity carries its own load; a day the watch
        // recorded and the athlete did not train on is a rest day, and its
        // load is zero.
        let rest_day = !activity_days.contains(&text);
        days.push(DayInput { date, rest_day });
    }
    days.sort_by_key(|day| day.date);

    Ok(LoadInputs {
        days,
        activities,
        resting_hr,
        zone_model: zone_model(api)?,
    })
}

/// The athlete's stored zone model, as the load engine reads it.
///
/// `None` when no model has been read yet: the load then answers "no
/// value", with the reason, and never a guessed max heart rate.
fn zone_model(api: &Api) -> ApiResult<Option<LoadZoneModel>> {
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
    Ok(Some(LoadZoneModel {
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
