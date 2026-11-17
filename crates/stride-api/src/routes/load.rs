//! The training load: the series, the assumption and the scenario.
//!
//! The load is the product's most misused number, so this route answers
//! with **values and a scenario** and with nothing else: no band name, no
//! "high risk", no "optimal", no traffic light and nothing that tells the
//! athlete what to do.
//!
//! ## What it reads
//!
//! The numbers are the **core's** — the daily loads, the acute and chronic
//! series, the form, the ratio, the week's impulse, monotony and strain,
//! and the recovery scenario all come from `stride-core`'s load engine.
//! This route reads them through the seam in [`crate::load`], which is the
//! one place the store's rows become the engine's inputs: the athlete's
//! stored zone model, the measured resting heart rate of the daily summary
//! and the stored activities. It opens no connection and re-derives
//! nothing.
//!
//! ## The window and the warm-up
//!
//! The window is the one the Progress screen is showing: the same calendar
//! weeks, the same `weeks` parameter and the same bound. The load's own
//! constant is longer than most windows — the chronic series is weighted
//! over forty-two days — so the series is **warmed** with
//! [`crate::load::LOAD_WARMUP_DAYS`] days of history read before the
//! window. The days of the warm-up are weighed and they are not part of
//! the window's series: without them the numbers at the start of a window
//! would be the numbers of a series that began there.
//!
//! ## No value is the absence of the value
//!
//! A store with no zone model answers with the reason and **no numbers**:
//! never a zero, never a guessed maximum heart rate. A window with no
//! recorded days answers with the same kind of reason. A day with nothing
//! that could be weighed carries its own reason, and a week that is not
//! complete carries no impulse, no monotony and no strain.

use serde_json::Value;
use stride_core::date::Date;
use stride_core::load::{
    recovery_scenario, DailyLoad, LoadAnswer, LoadReport, NoLoadReason, WeekLoad,
};

use crate::dto::{
    LoadAssumption, LoadDay, LoadInsight, LoadProjection, LoadScenario, LoadSeries, LoadState,
    LoadWeek, LoadWindow, RenderedValue,
};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// How many weeks the window may cover — the Progress route's own bound.
const MAX_WEEKS: i64 = 104;

/// The training load over the selected window.
pub fn load(api: &Api, request: &Request) -> ApiResult<Value> {
    let weeks = request.query.integer("weeks")?.unwrap_or(12);
    if !(1..=MAX_WEEKS).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the load window is not between 1 and 104 weeks",
        ));
    }

    let current = super::today(api)?;
    // The window: the same calendar weeks the Progress screen shows, the
    // current partial week included.
    let window_start = super::week_start(current).add_days(-7 * (weeks - 1));
    let window_end = current;
    // The warm-up: the days before the window that the two weighted series
    // are seeded with.
    let read_from = window_start.add_days(-crate::load::LOAD_WARMUP_DAYS);

    let answer = crate::load::load_report(api, read_from, window_end)?;

    let window = LoadWindow {
        from: window_start.to_iso(),
        from_label: super::day_label(window_start),
        to: window_end.to_iso(),
        to_label: super::day_label(window_end),
        weeks,
        warmup_days: crate::load::LOAD_WARMUP_DAYS,
    };

    let (load, unavailable, scenario) = match answer {
        LoadAnswer::Unavailable(reason) => (None, Some(reason_key(reason)), None),
        LoadAnswer::Known(report) => {
            // The window's days: the days of the series that fall inside
            // it. The warm-up is not part of the window.
            let days: Vec<&DailyLoad> = report
                .days
                .iter()
                .filter(|day| day.date >= window_start)
                .collect();
            if days.is_empty() {
                // The record covers no day of this window: there is no
                // load to show, and the reason is the core's own.
                (None, Some(reason_key(NoLoadReason::NoHistory)), None)
            } else {
                let read_on = super::day_label(report.assumptions.zone_model_read_on);
                let series = series_of(&report, window_start, &read_on);
                // The scenario projects from the last state of the series,
                // which is a day of the window: the warm-up is over by
                // then.
                let scenario =
                    recovery_scenario(&report).map(|scenario| scenario_of(scenario, &read_on));
                (Some(series), None, scenario)
            }
        }
    };

    let response = LoadInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        window,
        load,
        unavailable,
        scenario,
    };
    ok(&response)
}

/// The window's series, and the assumption its numbers rest on.
fn series_of(report: &LoadReport, window_start: Date, read_on: &str) -> LoadSeries {
    let days: Vec<LoadDay> = report
        .days
        .iter()
        .filter(|day| day.date >= window_start)
        .map(|day| LoadDay {
            date: day.date.to_iso(),
            label: super::day_label(day.date),
            load: day.load_text().map(|text| RenderedValue {
                text,
                raw: day.load,
            }),
            reason: day.reason.map(reason_key),
        })
        .collect();
    let states: Vec<LoadState> = report
        .states
        .iter()
        .filter(|state| state.date >= window_start)
        .map(state_of)
        .collect();
    LoadSeries {
        days,
        states,
        week: report.week.as_ref().map(week_of),
        assumptions: assumption_of(report, read_on),
    }
}

/// One day's state, as the core renders it.
fn state_of(state: &stride_core::load::LoadState) -> LoadState {
    LoadState {
        date: state.date.to_iso(),
        label: super::day_label(state.date),
        acute: RenderedValue {
            text: state.acute_text(),
            raw: Some(state.acute),
        },
        chronic: RenderedValue {
            text: state.chronic_text(),
            raw: Some(state.chronic),
        },
        form: RenderedValue {
            text: state.form_text(),
            raw: Some(state.form),
        },
        ratio: RenderedValue {
            text: state.ratio_text(),
            raw: Some(state.ratio),
        },
    }
}

/// The seven-day values, as the core renders them.
fn week_of(week: &WeekLoad) -> LoadWeek {
    LoadWeek {
        impulse: RenderedValue {
            text: week.impulse_text(),
            raw: Some(week.impulse),
        },
        monotony: RenderedValue {
            text: week.monotony_text(),
            raw: Some(week.monotony),
        },
        strain: RenderedValue {
            text: week.strain_text(),
            raw: Some(week.strain),
        },
    }
}

/// The assumptions the numbers were made under, in the core's own words.
fn assumption_of(report: &LoadReport, read_on: &str) -> LoadAssumption {
    let assumptions = &report.assumptions;
    LoadAssumption {
        banister_k: assumptions.banister_k,
        max_heart_rate_bpm: assumptions.max_heart_rate_bpm,
        threshold_hr_bpm: assumptions.threshold_hr_bpm,
        zone_model_read_on: assumptions.zone_model_read_on.to_iso(),
        zone_model_read_on_label: read_on.to_string(),
        acute_window_days: assumptions.acute_window_days,
        chronic_window_days: assumptions.chronic_window_days,
        forecast_days: assumptions.forecast_days,
        statement: assumptions.statement(read_on),
    }
}

/// The scenario, in the core's own words, with its assumption attached.
fn scenario_of(scenario: stride_core::load::RecoveryScenario, read_on: &str) -> LoadScenario {
    LoadScenario {
        days_to_ratio_band: scenario.days_to_ratio_band,
        days_to_positive_form: scenario.days_to_positive_form,
        band: stride_core::load::ratio_band_text(),
        statement: scenario.scenario_text(),
        assumption: scenario.assumptions.statement(read_on),
        projection: scenario
            .projection
            .iter()
            .map(|day| LoadProjection {
                day: day.day,
                form: RenderedValue {
                    text: day.form_text(),
                    raw: Some(day.form),
                },
                ratio: RenderedValue {
                    text: day.ratio_text(),
                    raw: Some(day.ratio),
                },
            })
            .collect(),
    }
}

/// Why there is no load, as the key the app reads.
///
/// The reasons are the **core's** — this crate does not write a reason of
/// its own and does not add one: the core's own vocabulary is the
/// product's, and a second list would drift from it.
fn reason_key(reason: NoLoadReason) -> String {
    match reason {
        NoLoadReason::NoZoneModel => "no_zone_model".to_string(),
        NoLoadReason::NoMaxHeartRate => "no_max_heart_rate".to_string(),
        NoLoadReason::UnusableZoneModel => "unusable_zone_model".to_string(),
        NoLoadReason::NoHistory => "no_history".to_string(),
        NoLoadReason::NoRestingHr => "no_resting_hr".to_string(),
        NoLoadReason::NothingWeighable => "nothing_weighable".to_string(),
        NoLoadReason::UnusableThreshold => "unusable_threshold".to_string(),
    }
}
