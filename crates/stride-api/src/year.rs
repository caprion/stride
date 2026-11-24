//! The seam to the core's year.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's inputs: it reads the store
//! through its existing repositories, hands the core the facts, and reads
//! the core's answers back.
//!
//! It decides nothing. There is no rule about what a month is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — the days, the months, the window's totals and the
//! sentences of the diff all come from the core.
//!
//! ## What it reads
//!
//! * the **runs** — their local day, their distance, their moving time and
//!   their elevation gain;
//! * the **stored zone model** — the athlete's own configuration, read
//!   once from the watch and stored with the day it was read. It is read
//!   through the api's own seam ([`crate::zonemodel`]), so there is one
//!   way to get the zones. A model that is not there is "no value", never
//!   a default;
//! * the **FIT files** of the runs, decoded on read, so that the time in
//!   the zones can be measured against the stored model;
//! * the **training load** — for the form at the end of each month.

use std::collections::BTreeMap;

use stride_core::date::Date;
use stride_core::rolling::ZoneTimes;
use stride_core::year::YearRun;
use stride_core::zones::ZoneFloors;

use crate::error::ApiResult;
use crate::state::Api;

/// The runs of a range of days, as the core's year reads them.
///
/// The zone times are measured against the athlete's **stored model**: the
/// watch's own configuration, with the day it was read. **Each run is
/// measured against the model that was in force on its own day.** A run
/// whose file carries no heart rate — or a run that predates the first
/// stored model — has **no zone times**, and the day it falls on has no
/// mix: that is no value, never a mix of zero.
pub fn year_runs(api: &Api, from: &str, to: &str) -> ApiResult<Vec<YearRun>> {
    let runs = api
        .store()
        .activities_between(from, to)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let models = crate::zonemodel::ZoneModels::read(api)?;

    Ok(runs
        .iter()
        .filter_map(|run| {
            let date = Date::parse(&run.local_date)?;
            // The model in force on the day of the run: a run that
            // predates the first stored model is not calibrated.
            let times = match models.in_force(date) {
                Some(model) if model.floors.is_usable() => run_times(api, model.floors, run.id),
                _ => None,
            };
            Some(YearRun {
                date,
                distance_m: run.distance_m,
                moving_time_s: run.moving_time_s,
                elevation_gain_m: run.elevation_gain_m,
                times,
            })
        })
        .collect())
}

/// The time one run spent in the athlete's zones.
fn run_times(api: &Api, floors: ZoneFloors, run_id: i64) -> Option<ZoneTimes> {
    let decoded = crate::routes::series::decoded(api, run_id)?;
    let samples = crate::routes::series::zone_samples(&decoded);
    let time = stride_core::zones::time_in_zone(floors, &samples)?;
    let per_zone = time.per_zone();
    Some(ZoneTimes {
        zone1_s: per_zone[0],
        zone2_s: per_zone[1],
        zone3_s: per_zone[2],
        zone4_s: per_zone[3],
        zone5_s: per_zone[4],
    })
}

/// The training load's form on each day of a range.
///
/// The form is the load's own value, and the load is computed by the
/// core's engine over the days the record covers. A store with no zone
/// model, or no resting heart rate to weigh the days against, has **no
/// form at all** — and the months carry none.
pub fn forms(api: &Api, from: Date, to: Date) -> ApiResult<BTreeMap<Date, f64>> {
    let mut forms: BTreeMap<Date, f64> = BTreeMap::new();
    let answer = crate::load::load_report(api, from, to)?;
    if let stride_core::load::LoadAnswer::Known(report) = answer {
        for state in report.states {
            forms.insert(state.date, state.form);
        }
    }
    Ok(forms)
}
