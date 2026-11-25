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
//! ## The read covers the window
//!
//! Every run's file is decoded on read, through the router's one decode
//! cache. **The cache is a cache, and it is not a bound on the answer.**
//! The ledger reads **every run the window holds**: the runs the cache
//! holds are served from it, and the runs it cannot hold are decoded on the
//! read and decoded again on the next one. What the window holds is the
//! store's own history, and **the coverage line describes the window** —
//! not the cache, and not a slice of it.
//!
//! The answer's own bound is the product's: `MAX_RUNS_IN_ANSWER`, the most
//! runs one answer may read. A window that holds more than that says so in
//! `more_runs`, which means exactly one thing: **the store holds runs in
//! this window that the answer does not cover**.
//!
//! ## Why the bands are missing, in four words
//!
//! The app's own bands can be missing for four different reasons, and they
//! are four different facts. The route says which one it is, in the core's
//! own words:
//!
//! | the situation | the reason |
//! |---|---|
//! | the runs carry no heart rate | no heart rate was recorded |
//! | no zone model has been read at all | no zone model has been read |
//! | a model was read, but none was in force for these runs | no zone model was in force for these runs |
//! | a model was in force and the run carries no heart rate | the run carries no heart rate |
//!
//! The third is the case the two bases exist for: **the watch's own zone
//! minutes are the only zone time before the watch's own configuration was
//! read**, and the card says so in the runner's language.

use std::collections::BTreeMap;

use serde_json::Value;
use stride_core::date::Date;
use stride_core::rolling::{PolarisedZone, Window};
use stride_core::stimulus::{band_time, ledger, Coverage, Ledger, LedgerRun, SourceZones};
use stride_core::zones::time_in_zone;

use crate::dto::{
    CoverageRow, KindRow, LastRow, LedgerWeekRow, LongestRow, OverlapRow, RepeatedRow, SourceLine,
    SourceWeekRow, SourceZoneRow, StimulusInsight, WeekBandRow,
};
use crate::error::ApiResult;
use crate::render;
use crate::router::Request;
use crate::state::Api;
use stride_store::ActivitySummary as StoredActivity;

use super::ok;

/// The ledger, over the Progress window.
pub fn stimulus(api: &Api, request: &Request) -> ApiResult<Value> {
    let (window, from, today) = super::insight_window(api, request)?;

    // The athlete's own history, newest first: the runs the window holds,
    // and the runs before it.
    let history = api
        .store()
        .activities_between("0001-01-01", &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;

    // The window's runs: **every run the window holds**. The answer covers
    // the window, and the decode cache may not decide what it covers. The
    // answer's own bound is the product's, and it is stated when it bites.
    let from_iso = from.to_iso();
    let in_window: Vec<&StoredActivity> = history
        .iter()
        .filter(|run| run.local_date >= from_iso)
        .collect();
    let more_runs = in_window.len() > super::MAX_RUNS_IN_ANSWER;
    let window_runs = &in_window[..in_window.len().min(super::MAX_RUNS_IN_ANSWER)];

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
    //
    // **The cache's bound is the cache's.** The newest runs the cache holds
    // are served from it; the runs it cannot hold are decoded on the read.
    // The answer covers the window either way.
    let cached = api.decoded_cache().capacity();
    let mut ledger_runs: Vec<LedgerRun> = Vec::with_capacity(window_runs.len());
    let mut used_models: Vec<crate::zonemodel::ZoneModel> = Vec::new();
    let mut bands_found = [false; 3];
    for (index, run) in window_runs.iter().enumerate() {
        let (ledger_run, model) = match read_run(api, run, &models, &source_zones, index < cached) {
            Some(read) => read,
            None => continue,
        };
        if let Some(model) = model {
            if !used_models.iter().any(|seen| seen.read_on == model.read_on) {
                used_models.push(model.clone());
            }
        }
        note_bands(&mut bands_found, ledger_run.zone_time.as_ref());
        ledger_runs.push(ledger_run);
    }

    // The runs before the window: the last occurrence of each kind is
    // searched over the athlete's **whole retained history**, not over the
    // window alone.
    //
    // The search reads them newest first and **stops as soon as every kind
    // has been found**: an older run cannot change the answer. A run with
    // no model in force holds no band — there is nothing to measure it
    // with — so it is not read at all.
    for run in history.iter().filter(|run| run.local_date < from_iso) {
        // Every kind has been found: an older run cannot change the answer.
        if bands_found.iter().all(|found| *found) {
            break;
        }
        let date = match Date::parse(&run.local_date) {
            Some(date) => date,
            None => continue,
        };
        if models.in_force(date).is_none() {
            continue;
        }
        let (ledger_run, _) = match read_run(api, run, &models, &source_zones, false) {
            Some(read) => read,
            None => continue,
        };
        note_bands(&mut bands_found, ledger_run.zone_time.as_ref());
        ledger_runs.push(ledger_run);
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
        // **An empty overlap is a fact to state, not a hole to hide.**
        overlap_missing: overlap_missing_of(&ledger),
        unavailable: unavailable_of(&ledger, &models, !used_models.is_empty()),
        more_runs,
    };
    ok(&answer)
}

/// One run, as the ledger reads it.
///
/// The run's file is decoded through the router's one choke point: the
/// cached one for the runs the cache holds, and the uncached one for the
/// runs it cannot.
///
/// **A run with no heart rate carries no measurement.** The app's own bands
/// are measured over the time the athlete was running, and a run with no
/// heart rate has nothing to measure them from: the time in a band would be
/// a zero where there is no measurement at all.
fn read_run(
    api: &Api,
    run: &StoredActivity,
    models: &crate::zonemodel::ZoneModels,
    source_zones: &BTreeMap<i64, [Option<f64>; 5]>,
    cached: bool,
) -> Option<(LedgerRun, Option<crate::zonemodel::ZoneModel>)> {
    let date = Date::parse(&run.local_date)?;
    let samples = if cached {
        crate::routes::series::zone_samples_of(api, run.id)
    } else {
        crate::routes::series::zone_samples_uncached(api, run.id)
    };
    let has_heart_rate = samples.iter().any(|sample| sample.heart_rate_bpm.is_some());
    let model = models.in_force(date).cloned();
    let zone_time = model
        .as_ref()
        .filter(|model| model.floors.is_usable())
        .filter(|_| has_heart_rate)
        .and_then(|model| time_in_zone(model.floors, &samples));
    let ledger_run = LedgerRun {
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
    };
    Some((ledger_run, model))
}

/// Which bands a run held, so the search for the last occurrence knows when
/// it can stop.
fn note_bands(found: &mut [bool; 3], zone_time: Option<&stride_core::zones::TimeInZone>) {
    let zone_time = match zone_time {
        Some(zone_time) => zone_time,
        None => return,
    };
    for (index, band) in PolarisedZone::all().iter().enumerate() {
        if band_time(zone_time, *band) > 0.0 {
            found[index] = true;
        }
    }
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

/// Why there is no overlap: the period where the app can check its own zone
/// arithmetic against the watch's own record.
///
/// **An empty overlap is a fact to state.** The answer says so, in the
/// core's own words, rather than leaving a hole where the check would be.
fn overlap_missing_of(ledger: &Ledger) -> Option<String> {
    if ledger.bases.overlap.is_some() {
        return None;
    }
    // A period with no runs has nothing to say about the check: the reason
    // it has no value at all is already on the answer.
    if ledger.coverage.runs == 0 {
        return None;
    }
    super::core_line("stimulus.overlap.none")
}

/// Why the ledger has no value at all.
///
/// **Four situations, four reasons.** The reasons name what is missing, and
/// they never say what to do about it:
///
/// * the runs carry no heart rate;
/// * no zone model has been read at all;
/// * a model was read, but none of it was in force for these runs — the
///   case the two bases exist for, where the watch's own zone minutes are
///   the only zone time there is;
/// * a model was in force, and the run carries no heart rate.
///
/// The core owns the wording: the api says which situation it is, and the
/// catalogue says what that is called.
fn unavailable_of(
    ledger: &Ledger,
    models: &crate::zonemodel::ZoneModels,
    model_in_force: bool,
) -> Option<String> {
    if ledger.bases.app.is_some() {
        return None;
    }
    if ledger.coverage.runs == 0 {
        return stride_core::stimulus::MissingInput::NoRuns.reason();
    }
    let missing = if ledger.coverage.with_heart_rate == 0 {
        stride_core::stimulus::MissingInput::NoHeartRate
    } else if models.models().is_empty() {
        stride_core::stimulus::MissingInput::NoZoneModel
    } else if !model_in_force {
        stride_core::stimulus::MissingInput::NoZoneModelInForce
    } else {
        stride_core::stimulus::MissingInput::NoHeartRateInRun
    };
    missing.reason()
}
