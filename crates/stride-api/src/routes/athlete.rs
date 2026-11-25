//! The athlete's own facts: the races, the threshold and the heart rates.
//!
//! **These are the athlete's own facts, and they are never synced.** They
//! are typed into the app and stored locally, in the encrypted database,
//! and they are the only things the athlete enters about themselves.
//!
//! ## The discipline every write carries
//!
//! The same discipline the wellness observations have: **the value, the day
//! it was entered, the day it last changed, and the source.** A race is
//! typed; a threshold choice is made; a pair of heart rates is declared.
//! None of them is ever inferred from the training, and none of them is
//! ever written twice to change it: a change is another fact, dated.
//!
//! ## What is refused
//!
//! * a **race with a label the app does not know** — the label names the
//!   distance, and the app never invents a distance for a race it cannot
//!   name;
//! * a **choice with no value** — an empty choice is not a choice, and a
//!   row that holds nothing would be read as a threshold of no value,
//!   which is not the same thing;
//! * a **second choice that would leave two in force** — one threshold is
//!   in force at a time, and the earlier choices stay in the dated
//!   history;
//! * a **declaration with no value** — an empty declaration is not a
//!   declaration, and a **source the app does not know** — the source is a
//!   closed list of five, so the app has nothing to interpret.

use serde_json::Value;
use stride_core::heart_rates::{Declaration, HeartRateKind, HeartRateSource};
use stride_core::threshold::{candidates, ThresholdSource, TypedSource};
use stride_store::{
    NewHeartRate, NewHeartRateDeclaration, NewRaceObservation, NewThresholdObservation,
    RaceObservation, ThresholdObservation,
};

use crate::dto::{
    BandsBasisRow, BoundaryRow, ChoiceRow, ChoiceWritten, DeclarationRow, DeclarationWritten,
    DeclaredBandsRow, DeclaredRateRow, HeartRateKindRow, HeartRateSourceRow, HeartRatesInsight,
    RaceWritten, RacesInsight, WatchBandsRow,
};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The athlete's races, newest first.
pub fn races(api: &Api, _request: &Request) -> ApiResult<Value> {
    let stored = api
        .store()
        .races()
        .map_err(|error| api.store_error("the races could not be read", &error))?;
    let answer = RacesInsight {
        generated_at: api.generated_at(),
        races: stored.iter().map(super::race_row).collect(),
    };
    ok(&answer)
}

/// Write a race the athlete ran.
///
/// The body carries the **label** — the closed list of the distances the
/// app knows — the **time** in seconds, the **day** the race was run, and
/// the athlete's own **name** for it. The distance is the label's own: the
/// label names the distance the race was at.
pub fn write_race(api: &Api, request: &Request) -> ApiResult<Value> {
    let label = body_text(request, "label")
        .ok_or_else(|| ApiError::bad_request("the race needs a label"))?;
    if !stride_store::model::race_label::is_known(&label) {
        return Err(ApiError::bad_request(
            "the race's label is not one the app knows",
        ));
    }
    let distance_m = stride_store::model::race_label::distance_m(&label).unwrap_or_default();
    let time_s = request
        .body_f64("time_s")?
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
        .ok_or_else(|| ApiError::bad_request("the race needs a time"))?;
    let date = body_text(request, "date")
        .ok_or_else(|| ApiError::bad_request("the race needs the day it was run"))?;
    let today = super::today(api)?;
    let race_date = super::core_date(&date)
        .map(|date| date.to_iso())
        .map_err(|_| ApiError::bad_request("the race's day could not be read"))?;
    let name = body_text(request, "name");

    // The write carries the same discipline the wellness observations
    // have: the value, the day it was entered, the day it last changed,
    // and the source.
    let entered_on = today.to_iso();
    let race = NewRaceObservation {
        distance_m,
        time_s,
        race_date,
        label,
        name,
        source: stride_store::model::race_source::TYPED.to_string(),
        entered_on: entered_on.clone(),
        updated_on: entered_on,
    };
    let id = api
        .store()
        .write_race(&race)
        .map_err(|error| api.store_error("the race could not be written", &error))?;
    let stored = api
        .store()
        .race(id)
        .map_err(|error| api.store_error("the race could not be read", &error))?;
    let stored: RaceObservation = match stored {
        Some(stored) => stored,
        None => {
            return Err(ApiError::internal(
                "the race that was written could not be read",
            ))
        }
    };
    let answer = RaceWritten {
        race: super::race_row(&stored),
        generated_at: api.generated_at(),
    };
    ok(&answer)
}

/// Make a threshold choice.
///
/// The body names **which candidate** the athlete picked — the watch's own
/// zone model, a race, a field test, or a number they brought — and the
/// **base** the choice was made for. The value comes from that candidate,
/// so the choice and the candidate cannot disagree; a typed choice carries
/// the value itself.
pub fn write_threshold(api: &Api, request: &Request) -> ApiResult<Value> {
    let today = super::today(api)?;
    let facts = crate::athlete::Facts::read(api, today)?;

    // The base the choice was made for: the two the plan has.
    let base = body_text(request, "base")
        .ok_or_else(|| ApiError::bad_request("the choice needs the base it was made for"))?;
    let base = match base.as_str() {
        "pfitzinger" => stride_core::threshold::ThresholdBase::Pfitzinger,
        "eighty_twenty" => stride_core::threshold::ThresholdBase::EightyTwenty,
        _ => return Err(ApiError::bad_request("the base is not one the plan has")),
    };

    // Which candidate the athlete picked.
    let source = body_text(request, "source")
        .ok_or_else(|| ApiError::bad_request("the choice needs the candidate it was made from"))?;

    // The value: from the candidate, or from the athlete's own number.
    let (value, rule_id, origin_date, origin_ref, label) = match source.as_str() {
        "watch_zone_model" | "race" | "field_test" => {
            // The run the field test reads, when the choice is one.
            let run_id = body_text(request, "origin_ref")
                .and_then(|reference| reference.trim().parse::<i64>().ok());
            let inputs = super::threshold::inputs_of(api, &facts, run_id)?;
            let computed = candidates(&inputs);
            let candidate = computed
                .iter()
                .find(|candidate| candidate.source.key() == source)
                .ok_or_else(|| {
                    ApiError::bad_request("the choice names a candidate that is not there")
                })?;
            let value = candidate_value(candidate);
            // Where the value came from: the model that was read, the
            // race that was run, or the run the test read.
            let origin_ref = match source.as_str() {
                "watch_zone_model" => facts.model.as_ref().map(|model| model.read_on.to_iso()),
                "race" => facts.stored_race.as_ref().map(|race| race.id.to_string()),
                _ => run_id.map(|id| id.to_string()),
            };
            (
                value,
                candidate.rule.map(|rule| rule.id.to_string()),
                candidate.date.map(|date| date.to_iso()),
                origin_ref,
                None,
            )
        }
        "typed" => {
            // The athlete's own number: a heart rate, a pace band, or
            // both. A choice with no value is refused.
            let heart_rate_bpm = request
                .body_f64("heart_rate_bpm")?
                .filter(|bpm| bpm.is_finite() && *bpm > 0.0);
            let fast = request
                .body_f64("pace_fast_s_per_km")?
                .filter(|pace| pace.is_finite() && *pace > 0.0);
            let slow = request
                .body_f64("pace_slow_s_per_km")?
                .filter(|pace| pace.is_finite() && *pace > 0.0);
            if heart_rate_bpm.is_none() && fast.is_none() {
                return Err(ApiError::bad_request("the choice carries no value"));
            }
            let label = body_text(request, "label")
                .ok_or_else(|| ApiError::bad_request("a typed choice needs its label"))?;
            let label = typed_source_of(&label)
                .ok_or_else(|| ApiError::bad_request("the label is not one the app knows"))?;
            ((heart_rate_bpm, fast, slow), None, None, None, Some(label))
        }
        _ => {
            return Err(ApiError::bad_request(
                "the candidate is not one of the four",
            ))
        }
    };

    // One threshold is in force at a time. The store takes the flag from
    // the choice that held it, in one transaction, and the earlier
    // choices stay in the dated history.
    let before = api
        .store()
        .count_threshold_in_force()
        .map_err(|error| api.store_error("the thresholds could not be counted", &error))?;
    if before > 1 {
        return Err(ApiError::internal(
            "two thresholds are in force at once, and the choice was refused",
        ));
    }

    let (heart_rate_bpm, fast, slow) = value;
    let (pace_low, pace_high) = match (fast, slow) {
        (Some(fast), Some(slow)) => (Some(fast.min(slow)), Some(fast.max(slow))),
        (Some(fast), None) => (Some(fast), Some(fast)),
        (None, Some(slow)) => (Some(slow), Some(slow)),
        (None, None) => (None, None),
    };
    if heart_rate_bpm.is_none() && pace_low.is_none() {
        return Err(ApiError::bad_request("the choice carries no value"));
    }
    let entered_on = today.to_iso();
    let choice = NewThresholdObservation {
        hr_bpm: heart_rate_bpm,
        pace_low_s_per_km: pace_low,
        pace_high_s_per_km: pace_high,
        source: source.clone(),
        rule_id,
        base: Some(base.key().to_string()),
        origin_date,
        origin_ref,
        label: label.map(|label| store_label(label).to_string()),
        entered_on: entered_on.clone(),
        updated_on: entered_on,
    };
    let id = api
        .store()
        .write_threshold_choice(&choice, true)
        .map_err(|error| api.store_error("the choice could not be written", &error))?;

    // The write is only right if exactly one choice is in force now.
    let after = api
        .store()
        .count_threshold_in_force()
        .map_err(|error| api.store_error("the thresholds could not be counted", &error))?;
    if after != 1 {
        return Err(ApiError::internal(
            "the threshold in force could not be held, and the choice was refused",
        ));
    }

    let stored = api
        .store()
        .threshold_choice(id)
        .map_err(|error| api.store_error("the choice could not be read", &error))?;
    let stored: ThresholdObservation = match stored {
        Some(stored) => stored,
        None => {
            return Err(ApiError::internal(
                "the choice that was written could not be read",
            ))
        }
    };
    let answer = ChoiceWritten {
        choice: choice_row_of(&stored),
        generated_at: api.generated_at(),
    };
    ok(&answer)
}

/// The value a candidate carries, as the write needs it.
fn candidate_value(
    candidate: &stride_core::threshold::Candidate,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let heart_rate_bpm = candidate.value.heart_rate_bpm();
    let pace = candidate.value.pace();
    match pace {
        Some(band) => (
            heart_rate_bpm,
            Some(band.fast_s_per_km),
            Some(band.slow_s_per_km),
        ),
        None => (heart_rate_bpm, None, None),
    }
}

/// The stored choice, as the answer carries it.
fn choice_row_of(stored: &ThresholdObservation) -> ChoiceRow {
    let source = match stored.source.as_str() {
        "race" => ThresholdSource::Race,
        "field_test" => ThresholdSource::FieldTest,
        "typed" => ThresholdSource::Typed,
        _ => ThresholdSource::WatchZoneModel,
    };
    let pace = match (stored.pace_low_s_per_km, stored.pace_high_s_per_km) {
        (Some(low), Some(high)) => Some(stride_core::training_paces::PaceBand::between(low, high)),
        (Some(low), None) => Some(stride_core::training_paces::PaceBand::single(low)),
        _ => None,
    };
    ChoiceRow {
        source: source.key().to_string(),
        label: source.label().to_string(),
        value: super::threshold_value(stored.hr_bpm, pace.as_ref()),
        heart_rate_bpm: stored.hr_bpm,
        pace: pace.as_ref().map(super::pace_band_row),
        base: stored.base.clone().unwrap_or_default(),
        typed_label: stored
            .label
            .as_deref()
            .and_then(typed_source_of)
            .map(|l| l.label().to_string()),
        origin_date: stored.origin_date.clone(),
        entered_on: stored.entered_on.clone(),
        updated_on: stored.updated_on.clone(),
        in_force: stored.in_force,
    }
}

/// The typed label the app sends, as the core reads it.
///
/// The closed list is the core's: the app sends the label's **key**, and
/// the store keeps the label the athlete gave it.
pub(crate) fn typed_source_of(key: &str) -> Option<TypedSource> {
    match key {
        "lab_test" => Some(TypedSource::LabTest),
        "coach" => Some(TypedSource::Coach),
        "another_app" => Some(TypedSource::AnotherApp),
        "book_or_table" => Some(TypedSource::BookOrTable),
        "my_own_number" => Some(TypedSource::MyOwnNumber),
        _ => None,
    }
}

/// The label the store keeps for a typed number.
///
/// The **closed list is the core's**: the app sends the label's key, and
/// the store keeps the label the athlete gave it.
fn store_label(source: TypedSource) -> &'static str {
    match source {
        TypedSource::LabTest => stride_store::model::threshold_label::LAB_TEST,
        TypedSource::Coach => stride_store::model::threshold_label::COACH,
        TypedSource::AnotherApp => stride_store::model::threshold_label::ANOTHER_APP,
        TypedSource::BookOrTable => stride_store::model::threshold_label::BOOK_OR_TABLE,
        TypedSource::MyOwnNumber => stride_store::model::threshold_label::MY_OWN_NUMBER,
    }
}

/// A field of the body, as a trimmed string.
fn body_text(request: &Request, name: &str) -> Option<String> {
    request
        .body_str(name)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

// ------------------------------------------------- the declared heart rates

/// The athlete's heart rates: the declaration in force, the bands it
/// yields, the watch's own floors, and the difference between the two.
///
/// **The two facts are never blended.** The answer carries them side by
/// side, says which one the bands come from, and states the difference as
/// a fact — boundary by boundary — and never as a correction.
pub fn heart_rates(api: &Api, _request: &Request) -> ApiResult<Value> {
    let declarations = crate::zonemodel::declarations(api)?;
    let in_force = stride_core::heart_rates::in_force(&declarations).cloned();
    let history: Vec<DeclarationRow> = stride_core::heart_rates::history(&declarations)
        .into_iter()
        .map(|declaration| declaration_row(declaration, in_force.as_ref()))
        .collect();

    // The bands the declaration yields, and the reason when it yields
    // none. **Absence is not zero**: no maximum means no bands at all.
    let derived = in_force.as_ref().map(stride_core::heart_rates::derive);
    let bands = derived.as_ref().and_then(|derived| derived.bands);
    let unavailable = match (&in_force, &derived) {
        (None, _) => super::core_line("heart_rates.none"),
        (Some(_), Some(derived)) => derived
            .missing
            .and_then(|missing| missing.reason()),
        (Some(_), None) => None,
    };

    // The watch's own floors: the other fact, and the one the app states
    // beside the declared one.
    let watch = crate::zonemodel::ZoneModels::read(api)?;
    let watch_model = watch.current().cloned();

    // The comparison: the two, and the difference between them.
    let comparison = stride_core::heart_rates::compare(
        in_force.as_ref(),
        watch_model.as_ref().map(|model| stride_core::heart_rates::WatchBands {
            floors: model.floors,
            max_heart_rate: model.max_heart_rate,
            read_on: model.read_on,
        }),
    );

    let answer = HeartRatesInsight {
        generated_at: api.generated_at(),
        declaration: in_force
            .as_ref()
            .map(|declaration| declaration_row(declaration, in_force.as_ref())),
        bands: bands
            .as_ref()
            .map(|bands| declared_bands_row(bands, in_force.as_ref())),
        watch: watch_model.as_ref().map(watch_bands_row),
        difference: comparison
            .difference
            .as_ref()
            .map(|differences| differences.iter().map(boundary_row).collect()),
        history,
        sources: HeartRateSource::ALL
            .iter()
            .map(|source| HeartRateSourceRow {
                key: source.key().to_string(),
                label: source.label().to_string(),
            })
            .collect(),
        kinds: vec![
            HeartRateKindRow {
                key: HeartRateKind::Max.key().to_string(),
                label: HeartRateKind::Max.label().to_string(),
            },
            HeartRateKindRow {
                key: HeartRateKind::Resting.key().to_string(),
                label: HeartRateKind::Resting.label().to_string(),
            },
        ],
        unavailable,
    };
    ok(&answer)
}

/// Declare the athlete's heart rates.
///
/// The body carries the **maximum**, the **resting rate**, or **both** —
/// each with the **source** it came from and, optionally, the **day it was
/// measured**. The kind that is not given carries the value it had
/// forward, so a declaration is always one coherent statement.
///
/// The write is a **new declaration**: the earlier ones stay in the dated
/// history, and the bands are re-derived from the declaration in force.
pub fn write_heart_rates(api: &Api, request: &Request) -> ApiResult<Value> {
    let today = super::today(api)?;
    let in_force = crate::zonemodel::declaration_in_force(api)?;

    let max = declared_rate(request, "max", in_force.as_ref())?;
    let resting = declared_rate(request, "resting", in_force.as_ref())?;
    if max.is_none() && resting.is_none() {
        return Err(ApiError::bad_request(
            "the declaration carries no heart rate at all",
        ));
    }

    let mut rates = Vec::new();
    if let Some(rate) = max {
        rates.push(rate);
    }
    if let Some(rate) = resting {
        rates.push(rate);
    }

    let entered_on = today.to_iso();
    let declaration = NewHeartRateDeclaration {
        rates,
        entered_on: entered_on.clone(),
        updated_on: entered_on,
    };
    api.store()
        .write_heart_rate_declaration(&declaration)
        .map_err(|error| api.store_error("the heart rates could not be written", &error))?;

    // The declaration in force is the one that was written.
    let declarations = crate::zonemodel::declarations(api)?;
    let current = stride_core::heart_rates::in_force(&declarations).cloned();
    let answer = DeclarationWritten {
        declaration: declaration_row(
            current.as_ref().ok_or_else(|| {
                ApiError::internal("the declaration that was written could not be read")
            })?,
            current.as_ref(),
        ),
        generated_at: api.generated_at(),
    };
    ok(&answer)
}

/// One declared heart rate, as the body carries it.
///
/// The kind that is not given carries its value forward from the
/// declaration in force: the athlete changes one number, and the
/// declaration stays one coherent statement.
fn declared_rate(
    request: &Request,
    kind: &str,
    in_force: Option<&Declaration>,
) -> ApiResult<Option<NewHeartRate>> {
    let field = format!("{kind}_heart_rate_bpm");
    let value = request.body_f64(&field)?;
    let source_field = format!("{kind}_source");
    let measured_field = format!("{kind}_measured_on");
    let carried = in_force.and_then(|declaration| {
        let rate = match kind {
            "max" => declaration.max,
            _ => declaration.resting,
        };
        rate
    });

    let value = match value {
        Some(value) if value.is_finite() && value > 0.0 => value,
        Some(_) => {
            return Err(ApiError::bad_request(
                "a heart rate is a number of beats per minute",
            ))
        }
        // Not given: the value carries forward, and the source with it.
        None => match carried {
            Some(carried) => carried.value_bpm,
            None => return Ok(None),
        },
    };

    // The source: the athlete picks one of the five, and the app has
    // nothing to interpret.
    let source = match body_text(request, &source_field) {
        Some(source) => HeartRateSource::of(&source).ok_or_else(|| {
            ApiError::bad_request("the source is not one of the five the app knows")
        })?,
        None => match carried {
            Some(carried) => carried.source,
            None => {
                return Err(ApiError::bad_request(
                    "a heart rate needs the source it came from",
                ))
            }
        },
    };

    // The day it was measured: optional, and never a zero.
    let measured_on = match body_text(request, &measured_field) {
        Some(day) => {
            let date = super::core_date(&day).map_err(|_| {
                ApiError::bad_request("the day the heart rate was measured could not be read")
            })?;
            Some(date.to_iso())
        }
        None => carried.and_then(|carried| carried.measured_on.map(|day| day.to_iso())),
    };

    Ok(Some(NewHeartRate {
        kind: kind.to_string(),
        value_bpm: value,
        source: source.key().to_string(),
        measured_on,
    }))
}

/// One declaration, as the answer carries it.
fn declaration_row(declaration: &Declaration, in_force: Option<&Declaration>) -> DeclarationRow {
    DeclarationRow {
        id: declaration.id,
        max: declaration
            .max
            .map(|rate| declared_rate_row(HeartRateKind::Max, rate)),
        resting: declaration
            .resting
            .map(|rate| declared_rate_row(HeartRateKind::Resting, rate)),
        entered_on: declaration.entered_on.to_iso(),
        entered_on_label: super::day_label(declaration.entered_on),
        entered: stride_core::strings::render(
            "heart_rates.entered",
            &[("date", &super::day_label(declaration.entered_on))],
        )
        .unwrap_or_default(),
        updated_on: declaration.updated_on.to_iso(),
        in_force: in_force.map(|in_force| in_force.id) == Some(declaration.id),
    }
}

/// One declared heart rate, as the answer carries it.
fn declared_rate_row(
    kind: HeartRateKind,
    rate: stride_core::heart_rates::DeclaredRate,
) -> DeclaredRateRow {
    DeclaredRateRow {
        // The kind's own label, in the runner's words: "your maximum heart
        // rate".
        label: kind.label().to_string(),
        value_bpm: rate.value_bpm,
        value: crate::render::heart_rate(Some(rate.value_bpm)),
        source: rate.source.key().to_string(),
        source_label: rate.source.label().to_string(),
        measured_on: rate.measured_on.map(|day| day.to_iso()),
        measured_on_label: rate.measured_on.map(super::day_label),
    }
}

/// The bands the declaration yields, as the answer carries them.
pub(crate) fn declared_bands_row(
    bands: &stride_core::heart_rates::Bands,
    declaration: Option<&Declaration>,
) -> DeclaredBandsRow {
    let entered_on = declaration.map(|declaration| declaration.entered_on);
    let resting = declaration.and_then(|declaration| {
        declaration
            .resting
            .map(|rate| rate.value_bpm)
    });
    let source = entered_on.and_then(|entered_on| {
        stride_core::heart_rates::source_line(
            bands,
            resting,
            &super::day_label(entered_on),
        )
    });
    DeclaredBandsRow {
        label: stride_core::strings::render("heart_rates.bands.label", &[]).unwrap_or_default(),
        floors: super::data::floors_row(&bands.floors),
        max_heart_rate: crate::render::heart_rate(bands.max_heart_rate),
        range: bands.range.key().to_string(),
        range_label: bands.range.label().to_string(),
        entered_on: entered_on.map(|day| day.to_iso()).unwrap_or_default(),
        source,
    }
}

/// The watch's own floors, as the answer carries them.
fn watch_bands_row(model: &crate::zonemodel::ZoneModel) -> WatchBandsRow {
    let read_on = super::day_label(model.read_on);
    WatchBandsRow {
        label: stride_core::strings::render("heart_rates.watch.label", &[]).unwrap_or_default(),
        floors: super::data::floors_row(&model.floors),
        max_heart_rate: crate::render::heart_rate(model.max_heart_rate),
        read_on: model.read_on.to_iso(),
        read_on_label: read_on.clone(),
        source: stride_core::model_source(&model.training_method, model.max_heart_rate, &read_on),
    }
}

/// One boundary's difference, as the answer carries it.
fn boundary_row(difference: &stride_core::heart_rates::BoundaryDifference) -> BoundaryRow {
    BoundaryRow {
        zone: difference.zone,
        declared_bpm: difference.declared_bpm,
        watch_bpm: difference.watch_bpm,
        difference_bpm: difference.difference_bpm,
        difference: crate::render::difference(
            Some(difference.difference_bpm),
            stride_core::units::ValueUnit::HeartRate,
        ),
    }
}

/// The basis the bands came from, as the answer carries it.
pub fn bands_basis_row(
    basis: &stride_core::heart_rates::Basis,
    declared: Option<&stride_core::heart_rates::Declaration>,
    watch: Option<&crate::zonemodel::ZoneModel>,
) -> BandsBasisRow {
    let (source, dated_on) = match basis {
        stride_core::heart_rates::Basis::Declared { .. } => {
            let source = declared.and_then(|declaration| {
                let derived = stride_core::heart_rates::derive(declaration);
                let bands = derived.bands?;
                let resting = declaration.resting.map(|rate| rate.value_bpm);
                stride_core::heart_rates::source_line(
                    &bands,
                    resting,
                    &super::day_label(declaration.entered_on),
                )
            });
            let dated_on = declared.map(|declaration| declaration.entered_on);
            (source, dated_on)
        }
        stride_core::heart_rates::Basis::Watch { read_on } => {
            let source = watch.and_then(|model| {
                stride_core::model_source(
                    &model.training_method,
                    model.max_heart_rate,
                    &super::day_label(model.read_on),
                )
            });
            (source, Some(*read_on))
        }
        stride_core::heart_rates::Basis::None => (None, None),
    };
    BandsBasisRow {
        kind: basis.key().to_string(),
        label: basis.label().to_string(),
        source,
        dated_on: dated_on.map(|day| day.to_iso()).unwrap_or_default(),
        dated_on_label: dated_on.map(super::day_label).unwrap_or_default(),
    }
}
