//! The one reader of the athlete's stored zone model — and of the
//! athlete's own declared heart rates.
//!
//! **The zone model is a dated fact about the athlete's configuration** —
//! the watch's own bands, read from Garmin and stored with the day they
//! were read. **The declared heart rates are a dated fact about the
//! athlete** — the maximum and/or the resting rate they entered, with the
//! day they were entered.
//!
//! Two facts, and every answer says which one it used.
//!
//! ## The rule
//!
//! Where a declaration exists, **the athlete's declared bands are the
//! bands** — applied to **every** run, so the series is comparable across
//! the whole window. The **watch's own model** is used where one was in
//! force and **no declaration exists** (D27).
//!
//! The two are **never mixed**: the reader returns one set of floors and
//! the basis they came from, and the answer names the basis.
//!
//! ## What the reader does
//!
//! * it reads the **stored models** through the store's own repository —
//!   the dated history in `hr_zone_models`, never a model rebuilt from a
//!   run's own heart rate;
//! * it reads the **declared heart rates** through the store's own
//!   repository, and reads the declaration in force;
//! * it turns the instant a model was read into **the athlete's own day**,
//!   because the model is read at an instant and the athlete's day is not
//!   the UTC one;
//! * it hands the core the model's **floors** and its **maximum heart
//!   rate**, and the api the model's own naming — its sport, its method
//!   and the day it was read.
//!
//! ## Which model is in force
//!
//! **The declared bands are in force for every day**, where a declaration
//! exists: the athlete's own numbers are a fact about the athlete and not
//! about a day's configuration, and one series over the whole window is
//! the property that makes the ledger readable.
//!
//! **The watch's own model in force on a day is the last one read on or
//! before the end of that day.** A run that predates the first stored
//! model has **no model in force**: it is not calibrated, and the answer
//! says so rather than borrowing a later model.
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

use chrono::{DateTime, TimeZone, Utc};
use stride_core::date::Date;
use stride_core::heart_rates::{Bands, Declaration, DeclaredRate, HeartRateKind, HeartRateSource};
use stride_core::zones::ZoneFloors;

use crate::error::ApiResult;
use crate::state::Api;

/// The athlete's declared bands, as the api reads them.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredBands {
    /// The declaration in force.
    pub declaration: Declaration,
    /// The bands it yields.
    pub bands: Bands,
}

impl DeclaredBands {
    /// The declared bands, as the core takes them.
    pub fn watch_bands(&self) -> stride_core::heart_rates::WatchBands {
        stride_core::heart_rates::WatchBands {
            floors: self.bands.floors,
            max_heart_rate: self.bands.max_heart_rate,
            read_on: self.declaration.entered_on,
        }
    }
}

/// The athlete's stored zone model, as the api reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneModel {
    /// The sport Garmin reported the model for.
    pub sport: String,
    /// The training method the model was set with — or `declared`, when the
    /// bands are the ones the athlete entered.
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
    /// The athlete's own declaration, when the bands are the declared
    /// ones. `None` for the watch's own model.
    ///
    /// It is **boxed**, so that the model stays small: the model is
    /// carried by value in a few places, and a large variant is a large
    /// copy.
    pub declared: Option<Box<DeclaredBands>>,
}

impl ZoneModel {
    /// Whether this model is the athlete's own declared bands.
    pub fn is_declared(&self) -> bool {
        self.declared.is_some()
    }
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

/// The model in force on a day: the **athlete's declared bands** where a
/// declaration exists, and **the last model read on or before the end of
/// that day** otherwise.
///
/// The answer names the reason there is no model, because the two reasons
/// are different facts: no model has ever been read, or models have been
/// read and none of them was in force on that day — a run that predates
/// the first stored model.
pub fn model_for(api: &Api, date: Date) -> ApiResult<ModelFor> {
    let models = ZoneModels::read(api)?;
    // The athlete's own declaration wins where there is one: the bands are
    // the declared ones, for every run.
    if let Some(declared) = &models.declared_model {
        return Ok(ModelFor::Model(Box::new(declared.clone())));
    }
    if models.models().is_empty() {
        return Ok(ModelFor::NoneRead);
    }
    Ok(match models.watch_in_force(date) {
        Some(model) => ModelFor::Model(Box::new(model.clone())),
        None => ModelFor::NoneInForce,
    })
}

/// The bands an answer is measured on, and the basis they came from.
///
/// **One basis per answer.** The declared bands win where a declaration
/// exists — applied to every run — and the watch's own model is used where
/// one was in force and no declaration exists.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectiveBands {
    /// The athlete's declared bands.
    Declared(DeclaredBands),
    /// The watch's own model, in force on the day.
    Watch(ZoneModel),
}

impl EffectiveBands {
    /// The five floors the bands are measured on.
    pub fn floors(&self) -> ZoneFloors {
        match self {
            EffectiveBands::Declared(declared) => declared.bands.floors,
            EffectiveBands::Watch(model) => model.floors,
        }
    }

    /// The maximum heart rate the bands were built on.
    pub fn max_heart_rate(&self) -> Option<f64> {
        match self {
            EffectiveBands::Declared(declared) => declared.bands.max_heart_rate,
            EffectiveBands::Watch(model) => model.max_heart_rate,
        }
    }

    /// The model the bands came from, as the api's model reads it.
    pub fn model(&self) -> ZoneModel {
        match self {
            EffectiveBands::Declared(declared) => declared_model(declared),
            EffectiveBands::Watch(model) => model.clone(),
        }
    }
}

/// The bands an answer is measured on: the athlete's declared bands where a
/// declaration exists, and the watch's own model otherwise.
///
/// `None` when there is nothing to measure against: no model has been
/// read, and no declaration has been made.
pub fn bands_for(api: &Api, date: Date) -> ApiResult<Option<EffectiveBands>> {
    let models = ZoneModels::read(api)?;
    if let Some(declared) = &models.declared {
        return Ok(Some(EffectiveBands::Declared(declared.clone())));
    }
    Ok(models
        .watch_in_force(date)
        .cloned()
        .map(EffectiveBands::Watch))
}

/// The athlete's declared heart rates, as the core reads them.
///
/// The rows are the store's; the declarations are assembled here, because
/// **one declaration is one or two rows** and only the declaration knows
/// that.
pub fn declarations(api: &Api) -> ApiResult<Vec<Declaration>> {
    let rows = api.store().heart_rate_declarations().map_err(|error| {
        api.store_error("the heart rate declarations could not be read", &error)
    })?;
    Ok(declarations_of(&rows))
}

/// The declarations the rows hold, as the core reads them.
///
/// A row whose day cannot be read is not a declaration the app can show:
/// the history is dated, and a declaration with no day has no place in it.
fn declarations_of(rows: &[stride_store::HeartRateDeclarationRow]) -> Vec<Declaration> {
    let mut declarations: Vec<Declaration> = Vec::new();
    for row in rows {
        let entered_on = match Date::parse(&row.entered_on) {
            Some(date) => date,
            None => continue,
        };
        let updated_on = Date::parse(&row.updated_on).unwrap_or(entered_on);
        let source = match HeartRateSource::of(&row.source) {
            Some(source) => source,
            None => continue,
        };
        let measured_on = row.measured_on.as_deref().and_then(Date::parse);
        let rate = DeclaredRate {
            value_bpm: row.value_bpm,
            source,
            measured_on,
        };
        match declarations
            .iter_mut()
            .find(|declaration| declaration.id == row.declaration_id)
        {
            Some(declaration) => match row.kind.as_str() {
                "max" => declaration.max = Some(rate),
                "resting" => declaration.resting = Some(rate),
                _ => {}
            },
            None => {
                let mut declaration = Declaration {
                    id: row.declaration_id,
                    max: None,
                    resting: None,
                    entered_on,
                    updated_on,
                };
                match row.kind.as_str() {
                    "max" => declaration.max = Some(rate),
                    "resting" => declaration.resting = Some(rate),
                    _ => continue,
                }
                declarations.push(declaration);
            }
        }
    }
    declarations
}

/// The declaration in force: the most recent one that carries a value.
pub fn declaration_in_force(api: &Api) -> ApiResult<Option<Declaration>> {
    Ok(stride_core::heart_rates::in_force(&declarations(api)?).cloned())
}

/// The model in force on a day, and the reason there is none when there is
/// none.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelFor {
    /// The model in force on that day.
    ///
    /// **Boxed**, so the enum is not three hundred bytes wide: the model
    /// is read on one day at a time, and the other two variants carry
    /// nothing at all.
    Model(Box<ZoneModel>),
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
            ModelFor::Model(model) => Some(model.as_ref()),
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
/// bands the athlete's own declaration gives — or, where there is no
/// declaration, the model that was in force on its own day.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZoneModels {
    /// The watch's own models, oldest first.
    models: Vec<ZoneModel>,
    /// The athlete's declared bands, where a declaration exists.
    declared: Option<DeclaredBands>,
    /// The declared bands, as the api's model reads them.
    declared_model: Option<ZoneModel>,
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
        // The athlete's own declaration, where one is in force and it
        // yields bands.
        let declared = declaration_in_force(api)?.and_then(|declaration| {
            let derived = stride_core::heart_rates::derive(&declaration);
            derived
                .bands
                .map(|bands| DeclaredBands { declaration, bands })
        });
        let declared_model = declared.as_ref().map(declared_model);
        Ok(Self {
            models,
            declared,
            declared_model,
        })
    }

    /// The bands in force on a day.
    ///
    /// **The declared bands are in force for every day**, where a
    /// declaration exists: the athlete's own numbers are a fact about the
    /// athlete, and one series over the whole window is what makes the
    /// ledger readable. Otherwise it is the watch's own model in force on
    /// that day.
    pub fn in_force(&self, date: Date) -> Option<&ZoneModel> {
        if let Some(declared) = &self.declared_model {
            return Some(declared);
        }
        self.watch_in_force(date)
    }

    /// The watch's own model in force on a day: **the last one read on or
    /// before the end of that day**.
    ///
    /// This is the watch's fact alone, and it is what the threshold's own
    /// candidates and the ladder's anchor read.
    pub fn watch_in_force(&self, date: Date) -> Option<&ZoneModel> {
        self.models.iter().rev().find(|model| model.read_on <= date)
    }

    /// The model in force now: the one read most recently.
    pub fn current(&self) -> Option<&ZoneModel> {
        self.models.last()
    }

    /// The models, oldest first.
    ///
    /// The watch's own models, and the athlete's declared bands where a
    /// declaration exists: the two facts, and every answer that names a
    /// model names one of them.
    pub fn models(&self) -> Vec<&ZoneModel> {
        let mut all: Vec<&ZoneModel> = self.models.iter().collect();
        if let Some(declared) = &self.declared_model {
            all.push(declared);
        }
        all
    }

    /// The athlete's declared bands, where a declaration exists.
    pub fn declared(&self) -> Option<&DeclaredBands> {
        self.declared.as_ref()
    }

    /// The athlete's declared bands, as the api's model reads them.
    pub fn declared_model(&self) -> Option<&ZoneModel> {
        self.declared_model.as_ref()
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
        declared: None,
    }
}

/// The athlete's declared bands, as the api's model reads them.
///
/// The **basis** is the declaration, and the line that names it is the
/// declared one — never the watch's.
fn declared_model(declared: &DeclaredBands) -> ZoneModel {
    ZoneModel {
        sport: String::new(),
        training_method: stride_core::heart_rates::DECLARED_METHOD.to_string(),
        max_heart_rate: declared.bands.max_heart_rate,
        floors: declared.bands.floors,
        // The declaration is dated by a day, and the api renders the day.
        read_at: midnight(declared.declaration.entered_on),
        read_on: declared.declaration.entered_on,
        declared: Some(Box::new(declared.clone())),
    }
}

/// The first moment of a day, in UTC.
///
/// A declaration is dated by a day and not by an instant; this is the
/// instant the api's own shape needs, and it carries the day with it.
fn midnight(day: Date) -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(day.year(), day.month(), day.day(), 0, 0, 0) {
        chrono::LocalResult::Single(instant) => instant,
        _ => Utc
            .with_ymd_and_hms(1970, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| {
                Utc.timestamp_opt(0, 0)
                    .single()
                    .unwrap_or_else(|| Utc.timestamp_nanos(0))
            }),
    }
}

/// The kind of heart rate a row carries, as the core reads it.
pub fn kind_of(kind: &str) -> Option<HeartRateKind> {
    match kind {
        "max" => Some(HeartRateKind::Max),
        "resting" => Some(HeartRateKind::Resting),
        _ => None,
    }
}
