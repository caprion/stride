//! The one reader of the athlete's stored zone model.
//!
//! **The zone model is a dated fact about the athlete's configuration** —
//! the watch's own bands, read from Garmin and stored with the day they
//! were read. Three answers need it: the run detail's zone table, the
//! fingerprint's distribution and efficiency factor, and the training
//! load's threshold and maximum heart rate.
//!
//! They read it here, in one place, so that there is **one way to get the
//! zones**. A rule that lives in two places is a rule that will drift, and
//! three copies of the reader is three places.
//!
//! ## What the reader does
//!
//! * it reads the **stored models** through the store's own repository —
//!   the dated history in `hr_zone_models`, never a model rebuilt from a
//!   run's own heart rate;
//! * it turns the instant a model was read into **the athlete's own day**,
//!   because the model is read at an instant and the athlete's day is not
//!   the UTC one;
//! * it hands the core the model's **floors** and its **maximum heart
//!   rate**, and the api the model's own naming — its sport, its method
//!   and the day it was read.
//!
//! ## Which model is in force
//!
//! **The model in force on a day is the last one read on or before the
//! end of that day.** A run is measured against the configuration the
//! athlete was wearing when they ran it, and the configuration is dated.
//! A run that predates the first stored model has **no model in force**:
//! it is not calibrated, and the answer says so rather than borrowing a
//! later model.
//!
//! The load's own model is the exception, and it is a later change: the
//! training load weighs a whole window of days against **one** model, and
//! that moves with its own announcement.
//!
//! ## No value
//!
//! A model that has not been read is **no value**, never a default and
//! never a guessed maximum heart rate. The answers that need a model say
//! so in the core's own words.

use chrono::{DateTime, Utc};
use stride_core::date::Date;
use stride_core::zones::ZoneFloors;

use crate::error::ApiResult;
use crate::state::Api;

/// The athlete's stored zone model, as the api reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneModel {
    /// The sport Garmin reported the model for.
    pub sport: String,
    /// The training method the model was set with.
    pub training_method: String,
    /// The maximum heart rate the model was built on, when Garmin reported
    /// one.
    pub max_heart_rate: Option<f64>,
    /// The model's five floors.
    pub floors: ZoneFloors,
    /// The instant the model was read.
    pub read_at: DateTime<Utc>,
    /// The day the model was read, in the athlete's own calendar.
    pub read_on: Date,
}

/// The model in force today: **the one read most recently**.
///
/// `None` when no model has been read yet: the answers that need a model
/// then say "no value", with the reason, and never a guessed maximum
/// heart rate.
pub fn current_model(api: &Api) -> ApiResult<Option<ZoneModel>> {
    let model = api
        .store()
        .current_hr_zone_model()
        .map_err(|error| api.store_error("the zone model could not be read", &error))?;
    let model = match model {
        Some(model) => model,
        None => return Ok(None),
    };
    // The day the model was read, in the athlete's own calendar: a model
    // read at 23:30 belongs to the athlete's day, not to the UTC one. A
    // model whose day cannot be read is no model at all.
    let read_on = match crate::routes::day_of(api, model.read_at) {
        Ok(day) => day,
        Err(_) => return Ok(None),
    };
    Ok(Some(zone_model(model, read_on)))
}

/// The model in force on a day: **the last one read on or before the end
/// of that day**.
///
/// The answer names the reason there is no model, because the two reasons
/// are different facts: no model has ever been read, or models have been
/// read and none of them was in force on that day — a run that predates
/// the first stored model.
pub fn model_for(api: &Api, date: Date) -> ApiResult<ModelFor> {
    let models = ZoneModels::read(api)?;
    if models.models().is_empty() {
        return Ok(ModelFor::NoneRead);
    }
    Ok(match models.in_force(date) {
        Some(model) => ModelFor::Model(model.clone()),
        None => ModelFor::NoneInForce,
    })
}

/// The model in force on a day, and the reason there is none when there is
/// none.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelFor {
    /// The model in force on that day.
    Model(ZoneModel),
    /// No zone model has been read at all.
    NoneRead,
    /// Zone models have been read, but **none of them was in force on that
    /// day**: the run predates the first stored model.
    NoneInForce,
}

impl ModelFor {
    /// The model, when there is one.
    pub fn model(&self) -> Option<&ZoneModel> {
        match self {
            ModelFor::Model(model) => Some(model),
            _ => None,
        }
    }

    /// The reason there is no model, as the core's catalogue names it.
    pub fn reason_id(&self) -> &'static str {
        match self {
            ModelFor::NoneRead => "zones.no_model",
            ModelFor::NoneInForce => "zones.no_model_in_force",
            ModelFor::Model(_) => "zones.no_model",
        }
    }
}

/// The athlete's stored zone models: **the dated history**, oldest first.
///
/// The models are read once and asked about many times: a window of
/// history holds dozens of runs, and each run is measured against the
/// model that was in force on its own day.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZoneModels {
    /// The models, oldest first.
    models: Vec<ZoneModel>,
}

impl ZoneModels {
    /// The dated history the store holds.
    pub fn read(api: &Api) -> ApiResult<Self> {
        let rows = api
            .store()
            .hr_zone_models()
            .map_err(|error| api.store_error("the zone models could not be read", &error))?;
        let mut models = Vec::with_capacity(rows.len());
        for row in rows {
            // A model whose day cannot be read is no model at all.
            let read_on = match crate::routes::day_of(api, row.read_at) {
                Ok(day) => day,
                Err(_) => continue,
            };
            models.push(zone_model(row, read_on));
        }
        Ok(Self { models })
    }

    /// The model in force on a day: **the last one read on or before the
    /// end of that day**.
    pub fn in_force(&self, date: Date) -> Option<&ZoneModel> {
        self.models.iter().rev().find(|model| model.read_on <= date)
    }

    /// The model in force now: the one read most recently.
    pub fn current(&self) -> Option<&ZoneModel> {
        self.models.last()
    }

    /// The models, oldest first.
    pub fn models(&self) -> &[ZoneModel] {
        &self.models
    }
}

/// The store's own row, as the api's model.
fn zone_model(model: stride_store::HeartRateZoneModel, read_on: Date) -> ZoneModel {
    ZoneModel {
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
        read_at: model.read_at,
        read_on,
    }
}
