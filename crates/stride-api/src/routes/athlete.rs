//! The athlete's own facts: the races and the threshold.
//!
//! **These are the athlete's own facts, and they are never synced.** They
//! are typed into the app and stored locally, in the encrypted database,
//! and they are the only things the athlete enters about themselves.
//!
//! ## The discipline every write carries
//!
//! The same discipline the wellness observations have: **the value, the day
//! it was entered, the day it last changed, and the source.** A race is
//! typed; a threshold choice is made. Neither is ever inferred from the
//! training, and neither is ever written twice to change it: a change is
//! another fact, dated.
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
//!   history.

use serde_json::Value;
use stride_core::threshold::{candidates, ThresholdSource, TypedSource};
use stride_store::{
    NewRaceObservation, NewThresholdObservation, RaceObservation, ThresholdObservation,
};

use crate::dto::{ChoiceRow, ChoiceWritten, RaceWritten, RacesInsight};
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
fn typed_source_of(key: &str) -> Option<TypedSource> {
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
