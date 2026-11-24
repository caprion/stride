//! The ledger: the athlete's own bands, the shape rows, and the two bases.
//!
//! The ledger answers against the athlete's **own recorded history**: the
//! time in their own bands, the longest run, and the repeated work their
//! watch recorded. It never says what was not done, and it never scores the
//! training.
//!
//! ## The two bases
//!
//! | basis | what it is | measured over |
//! |---|---|---|
//! | the app's own zone time | the time in the athlete's own bands, from the run's samples with the model in force that day | the time the athlete was **running** |
//! | the source's own zone minutes | the watch's own record of the run | the **whole run**, stops included |
//!
//! The two are **never added together**, each carries its own coverage, and
//! the **overlap** — the runs where both exist — states the distance between
//! the two measurements without saying which is right.
//!
//! ## Absence is not zero
//!
//! A band a week held **none** has **no cell** in the weeks matrix, so the
//! absence is drawable as white space. A band with no time in a measured
//! period is a **measured zero**. A period that could not be measured at
//! all is a **no value with a reason**.
//!
//! ## What the read costs
//!
//! Every run's file is decoded on read, through the router's one decode
//! cache. **The read is bounded**: the ledger reads the newest
//! [`MAX_LEDGER_RUNS`] runs, which is what the cache holds, so a second
//! read of the same window costs no decodes at all. A store that holds
//! more runs than that says so, and the rows cover the runs that were
//! read.

use std::collections::BTreeMap;

use serde_json::Value;
use stride_core::date::Date;
use stride_core::rolling::Window;
use stride_core::stimulus::{ledger, Coverage, Ledger, LedgerRun, SourceZones};
use stride_core::zones::time_in_zone;

use crate::dto::{
    CoverageRow, KindRow, LastRow, LedgerWeekRow, LongestRow, OverlapRow, RepeatedRow, SourceLine,
    SourceWeekRow, SourceZoneRow, StimulusInsight, WeekBandRow,
};
use crate::error::ApiResult;
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// How many runs the ledger reads.
///
/// **The bound is the decode cache's**: the ledger reads the newest runs
/// the cache holds, so the second read of the same window is free. A store
/// that holds more runs than this says so, and the rows cover the runs
/// that were read.
pub const MAX_LEDGER_RUNS: usize = 64;

/// The ledger, over the Progress window.
pub fn stimulus(api: &Api, request: &Request) -> ApiResult<Value> {
    let (window, from, today) = super::insight_window(api, request)?;

    // The runs: the athlete's own history, newest first, up to the bound.
    let runs = api
        .store()
        .activities_between("0001-01-01", &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let selected: Vec<_> = runs.iter().take(MAX_LEDGER_RUNS).collect();
    let more_runs = runs.len() > selected.len();

    // The zone models: the athlete's own configuration, dated. Each run is
    // measured against the model in force on its own day.
    let models = crate::zonemodel::ZoneModels::read(api)?;

    // The source's own facts about the runs: the watch's own zone minutes,
    // over the whole run.
    let observations = api
        .store()
        .activity_observations_between(&from.to_iso(), &today.to_iso())
        .map_err(|error| {
            api.store_error(
                "the source's own facts about the runs could not be read",
                &error,
            )
        })?;
    let mut source_zones: BTreeMap<i64, [Option<f64>; 5]> = BTreeMap::new();
    for observation in &observations {
        let index = match observation.field.as_str() {
            "hrTimeInZone_1" => 0,
            "hrTimeInZone_2" => 1,
            "hrTimeInZone_3" => 2,
            "hrTimeInZone_4" => 3,
            "hrTimeInZone_5" => 4,
            _ => continue,
        };
        source_zones
            .entry(observation.activity_id)
            .or_insert([None; 5])[index] = observation.value;
    }

    // The runs, as the ledger reads them.
    let mut ledger_runs: Vec<LedgerRun> = Vec::with_capacity(selected.len());
    let mut used_models: Vec<crate::zonemodel::ZoneModel> = Vec::new();
    for run in &selected {
        let date = match Date::parse(&run.local_date) {
            Some(date) => date,
            None => continue,
        };
        let samples = crate::routes::series::zone_samples_of(api, run.id);
        let has_heart_rate = samples.iter().any(|sample| sample.heart_rate_bpm.is_some());
        let model = models.in_force(date);
        if let Some(model) = model {
            if !used_models.iter().any(|seen| seen.read_on == model.read_on) {
                used_models.push(model.clone());
            }
        }
        let zone_time = model
            .filter(|model| model.floors.is_usable())
            .and_then(|model| time_in_zone(model.floors, &samples));
        ledger_runs.push(LedgerRun {
            date,
            distance_m: Some(run.distance_m),
            moving_time_s: Some(run.moving_time_s),
            elapsed_time_s: Some(run.elapsed_time_s),
            zone_time,
            source_zones: source_zones
                .get(&run.id)
                .map(|zones| SourceZones { zone_s: *zones }),
            has_heart_rate,
            // The watch's own workout steps are not read from the file yet,
            // so the repeated-work row is a no value with its reason.
            workout_steps: None,
        });
    }

    let ledger = ledger(&ledger_runs, Window::calendar_weeks(window.weeks, today));

    // The source lines: the models the runs were measured against.
    used_models.sort_by_key(|model| model.read_on);
    let (source, sources) = model_lines(&used_models);

    let answer = StimulusInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window,
        source,
        sources,
        kinds: kinds_of(&ledger),
        weeks: weeks_of(&ledger),
        coverage: coverage_of(&ledger),
        longest: ledger.longest.as_ref().map(|longest| LongestRow {
            date: longest.date.to_iso(),
            date_label: super::day_label(longest.date),
            distance: render::distance(longest.distance_m),
            time: render::duration_of(longest.moving_time_s),
            share: render::share(longest.share_percent),
        }),
        repeated: ledger.repeated.as_ref().map(|repeated| RepeatedRow {
            time: render::duration_of(repeated.time_s),
            runs: repeated.runs,
            last_on: repeated.last_on.map(|date| date.to_iso()),
            last_on_label: repeated.last_on.map(super::day_label),
        }),
        repeated_missing: ledger.repeated_missing.and_then(|missing| missing.reason()),
        source_zones: ledger
            .bases
            .source
            .as_ref()
            .map(|source| {
                source
                    .zones
                    .iter()
                    .map(|row| SourceZoneRow {
                        zone: row.zone,
                        time: render::duration(row.time_s),
                        share: render::share(row.share_percent),
                        runs: row.runs,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        source_weeks: ledger
            .bases
            .source
            .as_ref()
            .map(|source| {
                source
                    .weeks
                    .iter()
                    .map(|week| SourceWeekRow {
                        week_start: week.week_start.to_iso(),
                        week_start_label: super::day_label(week.week_start),
                        zones: week
                            .zones
                            .iter()
                            .map(|zone| render::duration(*zone))
                            .collect(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        overlap: ledger.bases.overlap.as_ref().map(|overlap| OverlapRow {
            from: overlap.from.to_iso(),
            from_label: super::day_label(overlap.from),
            to: overlap.to.to_iso(),
            to_label: super::day_label(overlap.to),
            runs: overlap.runs,
            app_moving: render::duration_of(overlap.app_moving_s),
            source_elapsed: render::duration_of(overlap.source_elapsed_s),
            difference: render::difference(
                Some(overlap.difference_s),
                stride_core::ValueUnit::Duration,
            )
            .unwrap_or_else(|| crate::dto::RenderedValue {
                text: String::new(),
                raw: Some(overlap.difference_s),
            }),
        }),
        unavailable: unavailable_of(&ledger, &models),
        more_runs,
    };
    ok(&answer)
}

/// The kinds' rows, with the last occurrence of each.
fn kinds_of(ledger: &Ledger) -> Vec<KindRow> {
    ledger
        .kinds
        .iter()
        .map(|row| {
            let last = ledger.last_occurrence_of(row.band).and_then(|last| {
                last.date.map(|date| LastRow {
                    date: date.to_iso(),
                    date_label: super::day_label(date),
                    days_before_end: last.days_before_end,
                })
            });
            KindRow {
                key: row.band.key().to_string(),
                label: row.band.label().to_string(),
                time: render::duration(row.time_s),
                share: render::share(row.share_percent),
                runs: row.runs,
                last,
            }
        })
        .collect()
}

/// The weeks matrix: the band each week held.
fn weeks_of(ledger: &Ledger) -> Vec<LedgerWeekRow> {
    let app = match ledger.bases.app.as_ref() {
        Some(app) => app,
        None => return Vec::new(),
    };
    app.weeks
        .iter()
        .map(|week| LedgerWeekRow {
            week_start: week.week_start.to_iso(),
            week_start_label: super::day_label(week.week_start),
            bands: stride_core::rolling::PolarisedZone::all()
                .iter()
                .enumerate()
                .map(|(index, band)| WeekBandRow {
                    key: band.key().to_string(),
                    label: band.label().to_string(),
                    time: render::duration(week.bands[index]),
                })
                .collect(),
        })
        .collect()
}

/// What the period covers.
fn coverage_of(ledger: &Ledger) -> CoverageRow {
    coverage_row(&ledger.coverage)
}

/// The coverage, as the answer carries it.
fn coverage_row(coverage: &Coverage) -> CoverageRow {
    CoverageRow {
        runs: coverage.runs,
        with_heart_rate: coverage.with_heart_rate,
        measured: coverage.measured,
        with_source_zones: coverage.with_source_zones,
    }
}

/// The models' lines: one hoisted, the rest behind a click.
fn model_lines(models: &[crate::zonemodel::ZoneModel]) -> (Option<String>, Vec<SourceLine>) {
    if models.is_empty() {
        return (None, Vec::new());
    }
    let sources: Vec<SourceLine> = models
        .iter()
        .map(|model| {
            let read_on = super::day_label(model.read_on);
            SourceLine {
                kind: "zone_model".to_string(),
                line: stride_core::model_source(
                    &model.training_method,
                    model.max_heart_rate,
                    &read_on,
                ),
                date: Some(model.read_on.to_iso()),
                date_label: Some(read_on),
            }
        })
        .collect();
    let source = if models.len() == 1 {
        sources.first().and_then(|line| line.line.clone())
    } else {
        let days: Vec<String> = models
            .iter()
            .map(|model| super::day_label(model.read_on))
            .collect();
        let borrowed: Vec<&str> = days.iter().map(|day| day.as_str()).collect();
        stride_core::zones::models_source(&borrowed)
    };
    (source, sources)
}

/// Why the ledger has no value at all.
fn unavailable_of(ledger: &Ledger, models: &crate::zonemodel::ZoneModels) -> Option<String> {
    if ledger.bases.app.is_some() {
        return None;
    }
    if ledger.coverage.runs == 0 {
        return super::core_line("stimulus.no_runs");
    }
    if models.models().is_empty() {
        return super::core_line("zones.no_model");
    }
    super::core_line("rolling.zones.no_samples")
}
