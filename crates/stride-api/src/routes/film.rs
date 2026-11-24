//! The run film: one run, playable.
//!
//! **The run's own series, and the athlete's own zones behind it.** The
//! samples are decoded from the run's FIT file on read — the file is the
//! source of truth and the derived series are never stored — and each
//! sample carries the zone it sits in, measured against the athlete's
//! **stored** zone model.
//!
//! The film is a surface, not a fitness metric: it is one run, and it
//! answers with that run's own timeline. So it sits with the run's other
//! sub-routes (`/activities/{id}/fit`, `/activities/{id}/note`) and is
//! named by the run it belongs to.
//!
//! ## No value is the absence of the whole result
//!
//! A run with no file has no series at all, and says why in the core's
//! words. It is never a zero, never a band and never an empty chart that
//! looks like a run of nothing.

use serde_json::Value;
use stride_core::date::Date;
use stride_core::units;
use stride_core::zones::ZoneFloors;

use crate::dto::{FilmZone, RunFilm, ZoneModel};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::data::{floors_of, zone_model_of};
use super::ok;
use super::series;

/// The run's film.
pub fn film(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let Some(run) = run else {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        )));
    };

    // The athlete's model: **the one in force on the day of the run**,
    // named and dated.
    //
    // A model whose floors are not in order cannot be used, and the reason
    // is the core's own — the film itself is still there, because the pace,
    // the heart rate and the ground do not need a zone model. A run that
    // predates the first stored model has no model in force, and says so.
    let model_for = match Date::parse(&run.local_date) {
        Some(date) => crate::zonemodel::model_for(api, date)?,
        None => crate::zonemodel::ModelFor::NoneRead,
    };
    let (model, floors, max_heart_rate, zones_unavailable) = match model_for.model() {
        None => (None, None, None, core_line(model_for.reason_id())),
        Some(stored) => {
            let model = zone_model_of(stored);
            let max_heart_rate = stored.max_heart_rate;
            let floors = floors_of(stored);
            if floors.is_usable() {
                (Some(model), Some(floors), max_heart_rate, None)
            } else {
                (
                    Some(model),
                    None,
                    max_heart_rate,
                    core_line("zones.unusable_model"),
                )
            }
        }
    };

    // The run's file: the source of truth for the series.
    let Some(decoded) = series::decoded(api, id) else {
        return ok(&no_film(
            id,
            run.local_date.clone(),
            model,
            zones_unavailable,
            core_line("fingerprint.unavailable"),
            api.generated_at(),
        ));
    };

    let cap = api.config().max_samples_in_response;
    let samples = series::film_samples(&decoded, floors, cap);
    if samples.is_empty() {
        return ok(&no_film(
            id,
            run.local_date.clone(),
            model,
            zones_unavailable,
            core_line("fingerprint.no_samples"),
            api.generated_at(),
        ));
    }

    let zones = match floors {
        Some(floors) => bands_of(&floors, max_heart_rate),
        None => Vec::new(),
    };
    let outside_label = floors.and_then(|_| core_line("film.zone.outside"));

    let answer = RunFilm {
        activity_id: id,
        local_date: run.local_date.clone(),
        basis: Some("samples".to_string()),
        model,
        zones,
        outside_label,
        samples,
        ground: series::ground(&decoded, cap),
        splits: series::film_splits(&decoded),
        unavailable: None,
        zones_unavailable,
        generated_at: api.generated_at(),
    };
    ok(&answer)
}

/// The answer when there is no film: no samples, and the reason.
///
/// The reason is the **core's own sentence** — the run has no file to read,
/// or no samples to read — asked for out of the core's catalogue and passed
/// on. The api never writes a sentence of its own.
#[allow(clippy::too_many_arguments)]
fn no_film(
    activity_id: i64,
    local_date: String,
    model: Option<ZoneModel>,
    zones_unavailable: Option<String>,
    reason: Option<String>,
    generated_at: String,
) -> RunFilm {
    RunFilm {
        activity_id,
        local_date,
        basis: Some("samples".to_string()),
        model,
        zones: Vec::new(),
        outside_label: None,
        samples: Vec::new(),
        ground: Vec::new(),
        splits: Vec::new(),
        unavailable: reason,
        zones_unavailable,
        generated_at,
    }
}

/// The five bands of the athlete's zones, as the film draws them.
///
/// The band is the model's own: its name and its range are the core's, and
/// its edges are the numbers the core rendered the range from — so the
/// ribbon and the legend cannot disagree.
fn bands_of(floors: &ZoneFloors, max_heart_rate: Option<f64>) -> Vec<FilmZone> {
    stride_core::zones::zone_edges(floors, max_heart_rate)
        .iter()
        .enumerate()
        .map(|(index, (from, to))| {
            let (from_bpm, to_bpm) = match units::render_heart_rate_range_edges(Some(*from), *to) {
                Some((lower, upper)) => (Some(lower), Some(upper)),
                // The top band of a model with no maximum heart rate: it
                // has a floor, and no upper edge to draw.
                None => (Some(units::truncate_heart_rate(*from)), None),
            };
            FilmZone {
                label: units::render_zone_label(index + 1)
                    .unwrap_or_else(|| format!("Z{}", index + 1)),
                range: units::render_heart_rate_range(Some(*from), *to),
                from_bpm,
                to_bpm,
            }
        })
        .collect()
}

/// A sentence from the core's catalogue.
fn core_line(id: &str) -> Option<String> {
    stride_core::strings::render(id, &[])
}
