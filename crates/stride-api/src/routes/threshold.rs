//! The threshold: four candidates, one choice, a dated history.
//!
//! The app **computes candidates**; the athlete **chooses one**. The
//! choice is a dated fact with its source, and the app never says which
//! candidate is right.
//!
//! | candidate | what it is | what it yields |
//! |---|---|---|
//! | the watch's own zone model | the floor of the fifth zone | a heart rate |
//! | from a race | the hour-pace band | a pace band |
//! | from a field test | the estimate from a run | a heart rate and a pace |
//! | typed | the athlete's own number | a heart rate, a pace band, or both |
//!
//! ## What it reads
//!
//! * the **zone model** — the athlete's own configuration, the one in
//!   force today;
//! * the **race** — the athlete's own, the one the paces come from;
//! * the **run** — the one the field test reads, named by the `run`
//!   query. The athlete points at it; the app never picks one;
//! * the **typed number** — the athlete's own most recent typed choice.
//!
//! ## What each candidate cannot know
//!
//! That line **lives in the notes**, not in the number: a candidate
//! carries its value, its date and its rule, and the limit on it is an
//! annotation the surface may show under the card. The core holds the
//! note's wording, so that the app never writes one of its own.

use serde_json::Value;
use stride_core::threshold::{
    candidates, history, in_force, Candidate, FieldTest, FieldTestSample, ThresholdChoice,
    ThresholdInputs, ThresholdSource, TypedSource, WatchZoneModel,
};

use crate::dto::{CandidateRow, ChoiceRow, HistoryRow, RuleRow, ThresholdInsight};
use crate::error::ApiResult;
use crate::router::Request;
use crate::state::Api;
use stride_store::ThresholdObservation;

use super::ok;

/// The threshold: the four candidates, the choice in force, and the history.
pub fn threshold(api: &Api, request: &Request) -> ApiResult<Value> {
    let (window, _, today) = super::insight_window(api, request)?;

    // The athlete's own facts: the race, the model and the resting heart
    // rate.
    let facts = crate::athlete::Facts::read(api, today)?;

    // The choices the athlete has made: the choice in force and the dated
    // history.
    let stored = api
        .store()
        .threshold_choices()
        .map_err(|error| api.store_error("the threshold choices could not be read", &error))?;
    let choices: Vec<ThresholdChoice> = stored.iter().filter_map(choice_of).collect();

    // The run the field test reads: the one the athlete pointed at, named
    // by the query, or the one the athlete's own last field-test choice
    // points at.
    let run_id = match request.query.integer("run")? {
        Some(id) => Some(id),
        None => stored
            .iter()
            .filter(|choice| choice.source == "field_test")
            .filter_map(|choice| choice.origin_ref.as_ref())
            .filter_map(|reference| reference.parse::<i64>().ok())
            .next(),
    };
    let field_test = match run_id {
        Some(id) => field_test_of(api, id)?,
        None => None,
    };

    // The typed number: the athlete's own most recent typed choice.
    let typed = stored
        .iter()
        .find(|choice| choice.source == "typed")
        .and_then(|choice| {
            let label = choice.label.as_deref().and_then(typed_source_of);
            Some(stride_core::threshold::TypedThreshold {
                heart_rate_bpm: choice.hr_bpm,
                pace: pace_band_of(choice),
                label: label?,
                entered_on: super::core_date(&choice.entered_on).ok()?,
            })
        });

    let inputs = ThresholdInputs {
        watch: facts.model.as_ref().map(|model| WatchZoneModel {
            floors: model.floors,
            read_on: model.read_on,
        }),
        race: facts.race.clone(),
        field_test,
        typed,
    };
    let computed = candidates(&inputs);

    // The source lines: the zone model's own line, and the race's.
    let (source, sources) = super::source_lines(&facts, &facts.ladder());

    let answer = ThresholdInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window,
        source,
        sources,
        candidates: computed.iter().map(candidate_row).collect(),
        in_force: in_force(&choices).map(choice_row),
        history: history(&choices)
            .iter()
            .map(|choice| history_row(choice))
            .collect(),
        unavailable: None,
    };
    ok(&answer)
}

/// One candidate, as the answer carries it.
fn candidate_row(candidate: &Candidate) -> CandidateRow {
    CandidateRow {
        source: candidate.source.key().to_string(),
        label: candidate.source.label().to_string(),
        value: candidate.render(),
        heart_rate_bpm: candidate.value.heart_rate_bpm(),
        pace: candidate
            .value
            .pace()
            .map(|band| super::pace_band_row(&band)),
        date: candidate.date.map(|date| date.to_iso()),
        date_label: candidate.date.map(super::day_label),
        rule: candidate.rule.map(|rule| RuleRow {
            id: rule.id.to_string(),
            citation: rule.citation.to_string(),
            line: rule.line(),
        }),
        note: candidate.note(),
        missing: candidate
            .value
            .missing()
            .and_then(|missing| missing.reason()),
    }
}

/// The choice in force, as the answer carries it.
fn choice_row(choice: &ThresholdChoice) -> ChoiceRow {
    ChoiceRow {
        source: choice.source.key().to_string(),
        label: choice.source.label().to_string(),
        value: choice.render(),
        heart_rate_bpm: choice.heart_rate_bpm,
        pace: choice.pace.map(|band| super::pace_band_row(&band)),
        base: choice.base.key().to_string(),
        typed_label: choice.label.map(|label| label.label().to_string()),
        origin_date: choice.origin_date.map(|date| date.to_iso()),
        entered_on: choice.entered_on.to_iso(),
        updated_on: choice.updated_on.to_iso(),
        in_force: choice.in_force,
    }
}

/// One choice in the dated history.
fn history_row(choice: &ThresholdChoice) -> HistoryRow {
    let date = super::day_label(choice.entered_on);
    let value = choice.render().unwrap_or_default();
    HistoryRow {
        date: choice.entered_on.to_iso(),
        date_label: date.clone(),
        line: stride_core::strings::render(
            "threshold.history",
            &[("date", &date), ("value", &value)],
        ),
        value: choice.render(),
        base: choice.base.key().to_string(),
        in_force: choice.in_force,
    }
}

/// A stored choice, as the core reads it.
///
/// A row whose day cannot be read is not a choice the app can show: the
/// history is dated, and a choice with no date has no place in it.
fn choice_of(stored: &ThresholdObservation) -> Option<ThresholdChoice> {
    let entered_on = super::core_date(&stored.entered_on).ok()?;
    let updated_on = super::core_date(&stored.updated_on).ok()?;
    Some(ThresholdChoice {
        source: source_of(&stored.source),
        heart_rate_bpm: stored.hr_bpm,
        pace: pace_band_of(stored),
        rule_id: stored.rule_id.clone(),
        base: base_of(stored.base.as_deref().unwrap_or("pfitzinger")),
        origin_date: stored
            .origin_date
            .as_deref()
            .and_then(|date| super::core_date(date).ok()),
        origin_ref: stored.origin_ref.clone(),
        label: stored.label.as_deref().and_then(typed_source_of),
        entered_on,
        updated_on,
        in_force: stored.in_force,
    })
}

/// The pace band a stored choice carries.
fn pace_band_of(stored: &ThresholdObservation) -> Option<stride_core::training_paces::PaceBand> {
    let low = stored.pace_low_s_per_km?;
    let high = stored.pace_high_s_per_km.unwrap_or(low);
    Some(stride_core::training_paces::PaceBand::between(low, high))
}

/// The source a stored choice came from.
fn source_of(source: &str) -> ThresholdSource {
    match source {
        "race" => ThresholdSource::Race,
        "field_test" => ThresholdSource::FieldTest,
        "typed" => ThresholdSource::Typed,
        _ => ThresholdSource::WatchZoneModel,
    }
}

/// The base a stored choice was made for.
fn base_of(base: &str) -> stride_core::threshold::ThresholdBase {
    match base {
        "eighty_twenty" => stride_core::threshold::ThresholdBase::EightyTwenty,
        _ => stride_core::threshold::ThresholdBase::Pfitzinger,
    }
}

/// The typed label the store holds, as the core reads it.
fn typed_source_of(label: &str) -> Option<TypedSource> {
    match label {
        "lab test" => Some(TypedSource::LabTest),
        "coach" => Some(TypedSource::Coach),
        "another app" => Some(TypedSource::AnotherApp),
        "book or table" => Some(TypedSource::BookOrTable),
        "my own number" => Some(TypedSource::MyOwnNumber),
        _ => None,
    }
}

/// The field test's run, as the rule reads it.
///
/// The run is the athlete's own file, decoded on read: the last thirty
/// minutes of it, its pace and its heart rate. A run that cannot be
/// decoded is no value, and the reason says so.
fn field_test_of(api: &Api, activity_id: i64) -> ApiResult<Option<FieldTest>> {
    let stored = api
        .store()
        .activity(activity_id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let stored = match stored {
        Some(stored) => stored,
        None => return Ok(None),
    };
    let decoded = match crate::routes::series::decoded(api, activity_id) {
        Some(decoded) => decoded,
        None => return Ok(None),
    };
    let samples: Vec<FieldTestSample> = decoded
        .samples
        .iter()
        .map(|sample| FieldTestSample {
            elapsed_s: sample.elapsed_s,
            moving: sample.kind != stride_ingest::SegmentKind::Stand,
            heart_rate_bpm: sample.heart_rate_bpm,
            pace_s_per_km: sample
                .speed_ms
                .filter(|speed| *speed > 0.0)
                .map(|speed| 1_000.0 / speed),
        })
        .collect();
    let date = match super::core_date(&stored.local_date) {
        Ok(date) => date,
        Err(_) => return Ok(None),
    };
    Ok(Some(FieldTest {
        date,
        duration_s: Some(decoded.elapsed_time_s),
        samples,
    }))
}
