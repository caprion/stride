//! The seam to the core's run update.
//!
//! **The payload is built here, and it is metrics only.** The model is
//! handed the run's own facts — the distance, the duration, the pace, the
//! heart rate, the splits, the time in the athlete's own zones and the
//! run's own mechanics — and nothing else. Never the positions, never the
//! athlete's note, never their name, never the plan.
//!
//! Every fact is already computed: the splits come from the run's own file,
//! the zones come from the athlete's stored model, and the observations
//! come from the core's fingerprint. This module only arranges them into
//! the core's payload and hashes it, so that the update can be checked
//! against exactly what the model was given.

use stride_core::units;
use stride_core::update::{
    payload as core_payload, ObservationInput, Quantity, SplitInput, UpdateInput, ZoneInput,
};

use crate::dto::Fingerprint;
use crate::error::ApiResult;
use crate::fingerprint::fingerprint;
use crate::render;
use crate::routes::series;
use crate::state::Api;

/// The payload of a run, and the hash of it.
///
/// The hash is taken over the payload's **rendered text**: the words the
/// model is actually given. A draft whose numbers are not in that text was
/// not written from that payload, and the hash is what lets anybody check
/// that later.
pub struct RunPayload {
    /// The payload, as the core builds it.
    pub payload: stride_core::update::Payload,
    /// The payload's rendered text.
    pub text: String,
    /// The SHA-256 of the rendered text.
    pub hash: String,
}

impl RunPayload {
    /// The payload as the model reads it.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The payload for a run.
///
/// `None` when the run has no file to read: there is nothing to write
/// about, and the answer is "no value" — never an empty update and never
/// an update about nothing.
pub fn payload(api: &Api, activity_id: i64) -> ApiResult<Option<RunPayload>> {
    let run = api
        .store()
        .activity(activity_id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let Some(run) = run else {
        return Ok(None);
    };

    let decoded = series::decoded(api, activity_id);
    let has_file = decoded.is_some();

    // The run's own facts.
    let pace_s_km = if run.distance_m > 0.0 && run.moving_time_s > 0.0 {
        Some(run.moving_time_s / (run.distance_m / 1000.0))
    } else {
        None
    };

    // The splits, from the run's own file.
    let splits = match &decoded {
        Some(decoded) => series::splits(decoded)
            .into_iter()
            .map(|split| SplitInput {
                lap_index: split.lap_index.max(1) as usize,
                distance_m: split.distance.raw.unwrap_or_default(),
                pace_s_km: split.pace.as_ref().and_then(|pace| pace.raw),
                heart_rate: split.heart_rate.as_ref().and_then(|rate| rate.raw),
            })
            .collect(),
        None => Vec::new(),
    };

    // The zones and the observations, from the core's fingerprint: the same
    // computation the run detail shows, so the two cannot disagree.
    let print = fingerprint(api, activity_id)?;
    let zones = zone_inputs(&print);
    let observations = observation_inputs(&print);

    let model = api
        .store()
        .update_settings(api.now())
        .map_err(|error| api.store_error("the update's settings could not be read", &error))?
        .writer_model;

    let input = UpdateInput {
        has_file,
        model,
        read_on: read_on(api),
        distance_m: Some(run.distance_m),
        duration_s: Some(run.moving_time_s),
        pace_s_km,
        average_heart_rate: run.avg_hr,
        highest_heart_rate: run.max_hr,
        elevation_gain_m: Some(run.elevation_gain_m),
        splits,
        zones,
        observations,
    };

    let Some(payload) = core_payload(&input) else {
        return Ok(None);
    };
    let text = payload.text();
    let hash = stride_store::hash::sha256_hex(text.as_bytes());
    Ok(Some(RunPayload {
        payload,
        text,
        hash,
    }))
}

/// The day the run was read, in the api's own rendering.
fn read_on(api: &Api) -> String {
    let offset = chrono::Duration::minutes(api.utc_offset_minutes() as i64);
    let naive = (api.now() + offset).naive_utc();
    let date = stride_core::date::Date::parse(&naive.format("%Y-%m-%d").to_string());
    date.map(crate::routes::day_label).unwrap_or_default()
}

/// The time in the athlete's own zones, as the payload carries it.
fn zone_inputs(print: &Fingerprint) -> Vec<ZoneInput> {
    print
        .zones
        .shares
        .iter()
        .enumerate()
        .map(|(index, share)| ZoneInput {
            index: index + 1,
            seconds: share
                .time
                .as_ref()
                .and_then(|time| time.raw)
                .unwrap_or_default(),
            share_pct: share.share.raw,
        })
        .collect()
}

/// The run's own mechanics, as the payload carries them.
///
/// The signed observations are the ones the **direction** check reads: the
/// drifts and the decoupling. The others are values without a sign, and the
/// payload carries them as they are.
fn observation_inputs(print: &Fingerprint) -> Vec<ObservationInput> {
    print
        .observations
        .iter()
        .filter_map(|observation| {
            // An observation with no value is not a fact, and the payload
            // carries facts: the reason it has none is the app's business,
            // not the model's.
            let value = observation.value.as_ref()?;
            let signed = value.raw;
            let quantity = quantity_of(&observation.key);
            Some(ObservationInput {
                label: observation.label.clone(),
                value: value.text.clone(),
                signed: match quantity {
                    Some(_) => signed,
                    None => None,
                },
                quantity,
            })
        })
        .collect()
}

/// What a signed observation counts.
///
/// The core's own keys: the pace drift is a pace, the heart-rate drift is
/// a heart rate, and the cadence drift and the decoupling are percentages.
fn quantity_of(key: &str) -> Option<Quantity> {
    match key {
        "pace_drift" => Some(Quantity::Pace),
        "heart_rate_drift" => Some(Quantity::HeartRate),
        "cadence_drift" | "decoupling" => Some(Quantity::Percent),
        _ => None,
    }
}

/// The run's average pace, as the core renders it.
pub fn pace_of(distance_m: f64, duration_s: f64) -> Option<String> {
    if distance_m <= 0.0 || duration_s <= 0.0 {
        return None;
    }
    units::render_pace(Some(duration_s / (distance_m / 1000.0)))
}

/// The distance, as the core renders it.
pub fn distance_of(distance_m: f64) -> Option<String> {
    render::distance(Some(distance_m)).map(|value| value.text)
}
