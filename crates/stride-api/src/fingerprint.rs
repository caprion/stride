//! The seam to the core's fingerprint engine.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's inputs: it reads the run, the
//! athlete's stored zone model and the run's own file, hands the core the
//! facts, and reads the core's answers back.
//!
//! It decides nothing. There is no rule about what a drift is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — every observation, every value and every reason comes
//! from the core.
//!
//! ## What it reads
//!
//! * the **run** — its date and its distance. The distance tells a fragment
//!   of a last lap from a lap;
//! * the **zone model** — the athlete's own stored configuration, read once
//!   from the watch and stored with the day it was read. It is read through
//!   the api's own seam ([`crate::zonemodel`]), so there is **one** way to
//!   get the zones: the same one the time-in-zone computation reads. It is
//!   **the model in force on the day of the run** — the configuration the
//!   athlete was wearing when they ran it;
//! * the **file** — the run's own FIT file, decoded on read. The derived
//!   series are recomputable and are never stored, so a run with no file
//!   has no fingerprint: there is nothing to measure, and the answer says
//!   so in the core's words.

use stride_core::date::Date;
use stride_core::fingerprint::{
    fingerprint as core_fingerprint, FingerprintInput, FingerprintZoneModel, LapReading, NoValue,
    ZoneDistribution,
};
use stride_core::units;
use stride_core::zones::ZoneSample;

use crate::dto::{
    Fingerprint, FingerprintObservation, FingerprintZones, RenderedValue, ZoneModel, ZoneShare,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::routes::series;
use crate::state::Api;

/// The basis the fingerprint's numbers were computed on.
///
/// The observations come from the run's **laps** and the zone shares from
/// its **moving samples** — the same filter the zones route uses, so the
/// two answers cannot disagree.
const BASIS: &str = "laps and moving samples";

/// The athlete's stored zone model: the core's reading of it and the
/// payload's naming of it.
struct StoredModel {
    /// The model, as the core reads it.
    core: FingerprintZoneModel,
    /// The model, as the payload names it: its sport, its method, its
    /// maximum heart rate, its floors and the day it was read.
    payload: ZoneModel,
}

/// The fingerprint of one run.
pub fn fingerprint(api: &Api, id: i64) -> ApiResult<Fingerprint> {
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let Some(run) = run else {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        )));
    };
    let model = stored_model(api, &run.local_date)?;

    // The run's own file, decoded on read. A run with no file has no laps
    // and no samples, and there is nothing to measure.
    let Some(decoded) = series::decoded(api, id) else {
        return Ok(Fingerprint {
            activity_id: id,
            local_date: run.local_date.clone(),
            basis: None,
            laps: 0,
            observations: Vec::new(),
            zones: zones_without_value(model.as_ref()),
            unavailable: NoValue::NoFile.reason(),
            generated_at: api.generated_at(),
        });
    };

    let laps: Vec<LapReading> = decoded.laps.iter().map(lap_reading).collect();
    // The same sample list the zones route reads, from the same one
    // place: the per-run bands and the window's split cannot disagree
    // about which samples count.
    let samples: Vec<ZoneSample> = crate::routes::series::zone_samples(&decoded);

    let answer = core_fingerprint(&FingerprintInput {
        laps,
        run_distance_m: run.distance_m,
        zone_model: model.as_ref().map(|model| model.core),
        samples,
    });

    let observations = answer
        .observations
        .iter()
        .map(|observation| FingerprintObservation {
            key: observation.key.clone(),
            label: observation.label.clone(),
            value: observation.text.clone().map(|text| RenderedValue {
                text,
                raw: observation.value,
            }),
            reason: observation.reason.clone(),
        })
        .collect();

    Ok(Fingerprint {
        activity_id: id,
        local_date: run.local_date.clone(),
        basis: Some(BASIS.to_string()),
        laps: answer.laps,
        observations,
        zones: zone_shares(answer.zones, model.as_ref()),
        unavailable: None,
        generated_at: api.generated_at(),
    })
}

/// One lap of the run, as the core reads it.
fn lap_reading(lap: &stride_ingest::FitLap) -> LapReading {
    LapReading {
        duration_s: lap.duration_s,
        distance_m: lap.distance_m,
        avg_hr_bpm: lap.avg_hr.map(|rate| rate as f64),
        avg_cadence_spm: lap.avg_cadence,
        avg_speed_ms: lap.avg_speed_ms,
        stride_length_m: lap.stride_length_m,
    }
}

/// The zone shares of a run, against the athlete's stored model.
fn zone_shares(distribution: ZoneDistribution, model: Option<&StoredModel>) -> FingerprintZones {
    match distribution {
        ZoneDistribution::Known { zones, outside_s } => FingerprintZones {
            model: model.map(|model| model.payload.clone()),
            shares: zones
                .iter()
                .enumerate()
                .map(|(index, share)| ZoneShare {
                    label: units::render_zone_label(index + 1)
                        .unwrap_or_else(|| format!("Z{}", index + 1)),
                    share: render::percent(Some(share.share_pct)).unwrap_or(RenderedValue {
                        text: String::new(),
                        raw: Some(share.share_pct),
                    }),
                    time: render::duration(Some(share.seconds)),
                })
                .collect(),
            outside: render::duration(Some(outside_s)),
            unavailable: None,
            source: model.and_then(|model| model.core.source()),
        },
        ZoneDistribution::Unavailable(reason) => FingerprintZones {
            model: model.map(|model| model.payload.clone()),
            shares: Vec::new(),
            outside: None,
            unavailable: reason.reason(),
            source: model.and_then(|model| model.core.source()),
        },
    }
}

/// The zone distribution when there is nothing to compute it from.
fn zones_without_value(model: Option<&StoredModel>) -> FingerprintZones {
    FingerprintZones {
        model: model.map(|model| model.payload.clone()),
        shares: Vec::new(),
        outside: None,
        unavailable: None,
        source: model.and_then(|model| model.core.source()),
    }
}

/// The athlete's stored zone model, as the core and the payload read it.
///
/// The model is **the one in force on the day of the run**: the
/// configuration the athlete was wearing when they ran it.
///
/// `None` when no model was in force: the zone shares are then "no
/// value", with the reason, and never a default.
fn stored_model(api: &Api, local_date: &str) -> ApiResult<Option<StoredModel>> {
    let date = match Date::parse(local_date) {
        Some(date) => date,
        None => return Ok(None),
    };
    let model_for = crate::zonemodel::model_for(api, date)?;
    let model = match model_for.model() {
        Some(model) => model,
        None => return Ok(None),
    };
    // the api's own rendering of the day the model was read
    let read_on_label = crate::routes::day_label(model.read_on);
    Ok(Some(StoredModel {
        core: FingerprintZoneModel {
            max_heart_rate: model.max_heart_rate,
            floors: model.floors,
            read_on: Some(model.read_on),
        },
        payload: ZoneModel {
            sport: model.sport.clone(),
            training_method: model.training_method.clone(),
            max_heart_rate: render::heart_rate(model.max_heart_rate),
            floors: crate::dto::ZoneModelFloors {
                zone1: floor_value(model.floors.zone1),
                zone2: floor_value(model.floors.zone2),
                zone3: floor_value(model.floors.zone3),
                zone4: floor_value(model.floors.zone4),
                zone5: floor_value(model.floors.zone5),
            },
            read_at: crate::state::format_instant(model.read_at),
            // the api's own rendering of the day, as the run detail uses it
            read_on: read_on_label.clone(),
            source: stride_core::model_source(
                &model.training_method,
                model.max_heart_rate,
                &read_on_label,
            ),
        },
    }))
}

/// A floor, as the core renders it.
fn floor_value(bpm: f64) -> RenderedValue {
    render::heart_rate(Some(bpm)).unwrap_or(RenderedValue {
        text: String::new(),
        raw: Some(bpm),
    })
}
