//! The seam to the core's adoption: the plan the athlete adopts, stored.
//!
//! **The core owns the adoption.** This module reads the store, hands the
//! core the types it works on, and reads the core's answers back. It
//! decides nothing: the plan's shape, its paces, its refusals and every
//! sentence it carries come from `stride_core::plan_bases`.
//!
//! ## The dated copy
//!
//! An adopted plan is a **dated copy** of a programme (B1), and the copy is
//! what the store holds: the sessions, their kinds, their distances, the
//! paces that were worked out on the day of the adoption, the rule each
//! pace came from, and the threshold it was paced with. A later correction
//! to the catalogue cannot re-pace a running plan, because the paces are
//! not read again from it.
//!
//! The **week's own label** and the **mileage the book prints beside it**
//! are the catalogue's facts rather than the plan's, and they are read
//! from the catalogue. When the programme is no longer in it, the week
//! says what it can: the sessions, their paces and the plan's own source.
//!
//! ## One plan at a time
//!
//! **One plan is in force at a time** (D13/D14). A second plan cannot be
//! adopted while another one is in force, and the refusal is the core's
//! own reason.

use std::collections::BTreeMap;

use stride_core::date::Date;
use stride_core::plan::{PlanSource, SessionKind};
use stride_core::plan_bases::{
    self, AdoptedPlan, AdoptedSession, AdoptedWeek, DateWords, Pacing, PacingSource,
};
use stride_core::training_paces::PaceValue;
use stride_store::{NewAdoptedPlan, NewAdoptedSession};

use crate::error::ApiResult;
use crate::state::Api;

/// How many weeks of the athlete's own mileage the plan answers beside.
pub const MILEAGE_WEEKS: usize = 4;

/// The plan in force, as the core reads it.
///
/// `None` when no plan has been adopted.
pub fn in_force(api: &Api) -> ApiResult<Option<AdoptedPlan>> {
    let plans = api
        .store()
        .active_adopted_plans()
        .map_err(|error| api.store_error("the adopted plan could not be read", &error))?;
    let row = match plans.into_iter().next() {
        Some(row) => row,
        None => return Ok(None),
    };
    let sessions = api
        .store()
        .adopted_sessions(row.id)
        .map_err(|error| api.store_error("the adopted sessions could not be read", &error))?;
    Ok(Some(plan_of(api, &row, &sessions)?))
}

/// The plan in force, read for the week that holds a day.
///
/// `None` when no plan is in force, or when the day is not in it.
pub fn week_at(api: &Api, day: Date) -> ApiResult<Option<stride_core::plan_bases::WeekReading>> {
    let plan = match in_force(api)? {
        Some(plan) => plan,
        None => return Ok(None),
    };
    if day < plan.start_date || day > plan.end_date {
        return Ok(None);
    }
    let words = words_of(&plan);
    Ok(plan_bases::read_week_at(&plan, day, &words))
}

/// The plan's own words for the days its lines carry.
pub fn words_of(plan: &AdoptedPlan) -> DateWords {
    DateWords {
        threshold: plan
            .pacing
            .threshold
            .as_ref()
            .map(|source| crate::routes::day_label(source.date)),
        race: plan
            .pacing
            .race
            .as_ref()
            .map(|source| crate::routes::day_label(source.date)),
    }
}

/// The stored plan, as the core reads it.
///
/// The **sessions** are the store's own rows — the dated copy — and the
/// week's own facts are the catalogue's.
fn plan_of(
    api: &Api,
    row: &stride_store::AdoptedPlan,
    sessions: &[stride_store::AdoptedSession],
) -> ApiResult<AdoptedPlan> {
    let programme = plan_bases::programme(&row.template_id);
    let entry_mileage = programme.and_then(|programme| programme.entry_mileage.clone());
    let plan_mileage = programme.and_then(|programme| programme.plan_mileage.clone());

    // The plan's own days. A row whose day cannot be read is not a plan
    // the app can show: the plan is dated, and a plan with no days has no
    // place in the athlete's calendar.
    let race_date = crate::routes::core_date(&row.race_date)
        .map_err(|_| crate::error::ApiError::internal("the plan's race day could not be read"))?;
    let start_date = crate::routes::core_date(&row.start_date)
        .map_err(|_| crate::error::ApiError::internal("the plan's first day could not be read"))?;
    let end_date = crate::routes::core_date(&row.end_date)
        .map_err(|_| crate::error::ApiError::internal("the plan's last day could not be read"))?;
    let adopted_on = crate::routes::core_date(&row.adopted_on)
        .map_err(|_| crate::error::ApiError::internal("the plan's day could not be read"))?;

    // The weeks, from the sessions: each session knows the week it is in.
    let mut by_week: BTreeMap<i32, Vec<&stride_store::AdoptedSession>> = BTreeMap::new();
    for session in sessions.iter() {
        by_week
            .entry(session.week as i32)
            .or_default()
            .push(session);
    }

    let plan_start = start_date;
    let mut weeks: Vec<AdoptedWeek> = Vec::with_capacity(by_week.len());
    for (number, rows) in by_week.iter() {
        let template = programme.and_then(|programme| programme.week(*number));
        let sessions: Vec<AdoptedSession> = rows.iter().filter_map(|row| session_of(row)).collect();
        let cells: f64 = sessions
            .iter()
            .map(|session| session.distance_m.unwrap_or(0.0))
            .sum();
        let printed = template.and_then(|week| week.printed_volume_m);
        let (start_date, end_date) = bounds_of(&sessions, plan_start);
        weeks.push(AdoptedWeek {
            week: *number,
            weeks_to_go: template.and_then(|week| week.weeks_to_go),
            label: template
                .map(|week| week.label.clone())
                .unwrap_or_else(|| week_label(*number)),
            in_plan: template.map(|week| week.in_plan).unwrap_or(*number >= 0),
            recovery_week: template.map(|week| week.recovery_week).unwrap_or(false),
            start_date,
            end_date,
            printed_volume_m: printed,
            volume_check: plan_bases::volume_check_of(cells, printed),
            sessions,
            shift: None,
        });
    }

    let pacing = Pacing {
        threshold: row
            .threshold_value
            .as_ref()
            .zip(row.threshold_date.as_ref())
            .map(|(value, date)| PacingSource {
                value: value.clone(),
                date: crate::routes::core_date(date).unwrap_or(race_date),
                source: row.threshold_source.clone(),
                base: row.threshold_base.clone(),
            }),
        race: row
            .rule_value
            .as_ref()
            .zip(row.rule_date.as_ref())
            .map(|(value, date)| PacingSource {
                value: value.clone(),
                date: crate::routes::core_date(date).unwrap_or(race_date),
                source: None,
                base: None,
            }),
        missing: row.pacing_missing.clone(),
    };

    let goal_distance_m = row.goal_distance_m;
    let distance_m = programme.map(|p| p.distance_m).unwrap_or(goal_distance_m);
    let distance_label = programme
        .map(|p| p.distance_label.clone())
        .unwrap_or_else(|| row.goal_label.clone());

    Ok(AdoptedPlan {
        template_id: row.template_id.clone(),
        template_version: row.template_version.clone(),
        label: row.label.clone(),
        method: row.method.clone(),
        tier: row.tier.clone(),
        distance_m,
        distance_label,
        entry_mileage,
        plan_mileage,
        goal_distance_m,
        goal_distance_label: row.goal_label.clone(),
        race_date,
        adopted_on,
        start_date,
        end_date,
        weeks,
        pacing,
        athlete_mileage_m: mileage(api)?,
        // An adopted programme is a prescription: the athlete chose it.
        source: PlanSource::Prescribed,
    })
}

/// One stored session, as the core reads it.
///
/// A row whose day cannot be read is not a session: the plan is dated, and
/// a session with no day has no place in it.
fn session_of(row: &stride_store::AdoptedSession) -> Option<AdoptedSession> {
    let date = Date::parse(&row.date)?;
    let kind = SessionKind::from_key(&row.kind)?;
    let pace = plan_bases::stored_pace(&kind, row.pace_low_s_per_km, row.pace_high_s_per_km);
    let finish_pace =
        plan_bases::stored_finish_pace(&kind, row.finish_low_s_per_km, row.finish_high_s_per_km);
    Some(AdoptedSession {
        date,
        kind,
        distance_m: row.distance_m,
        detail: row.detail.clone(),
        pace,
        finish_pace,
        pace_rule: row.pace_rule.clone(),
        lt_last_m: row.lt_last_m,
    })
}

/// The first and the last day of a week's sessions.
///
/// A week with no sessions has no days of its own, and the plan's own
/// first day stands in for it.
fn bounds_of(sessions: &[AdoptedSession], fallback: Date) -> (Date, Date) {
    let first = sessions.iter().map(|session| session.date).min();
    let last = sessions.iter().map(|session| session.date).max();
    match (first, last) {
        (Some(first), Some(last)) => (first, last),
        _ => (fallback, fallback),
    }
}

/// The label a week carries, from its number.
///
/// The catalogue's own labels, for a week the catalogue no longer holds.
fn week_label(week: i32) -> String {
    match week {
        0 => "race week".to_string(),
        number if number < 0 => "post-race week".to_string(),
        1 => "1 week to go".to_string(),
        number => format!("{number} weeks to go"),
    }
}

/// The athlete's own weekly mileage, oldest first.
///
/// The app computes it from the athlete's own runs; the core only states it
/// beside the tier's entry requirement.
pub fn mileage(api: &Api) -> ApiResult<Vec<f64>> {
    let today = crate::routes::today(api)?;
    let from = today.add_days(-((MILEAGE_WEEKS as i64 - 1) * 7) - 6);
    let runs = api
        .store()
        .activities_between(&from.to_iso(), &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let summaries = crate::routes::week_summaries(&runs);
    // The weeks the athlete ran in, oldest first. A week with no runs is
    // not a week of zero: it is a week the answer does not name.
    let values: Vec<f64> = summaries
        .iter()
        .rev()
        .take(MILEAGE_WEEKS)
        .rev()
        .map(|week| week.distance_m)
        .collect();
    Ok(values)
}

/// The stored rows of an adoption: the plan and its own sessions.
pub fn rows_of(
    plan: &AdoptedPlan,
    created_at: chrono::DateTime<chrono::Utc>,
) -> (NewAdoptedPlan, Vec<NewAdoptedSession>) {
    let threshold = plan.pacing.threshold.as_ref();
    let rule = plan.pacing.race.as_ref();
    let row = NewAdoptedPlan {
        template_id: plan.template_id.clone(),
        template_version: plan.template_version.clone(),
        label: plan.label.clone(),
        method: plan.method.clone(),
        tier: plan.tier.clone(),
        goal_distance_m: plan.goal_distance_m,
        goal_label: plan.goal_distance_label.clone(),
        race_date: plan.race_date.to_iso(),
        start_date: plan.start_date.to_iso(),
        end_date: plan.end_date.to_iso(),
        adopted_on: plan.adopted_on.to_iso(),
        threshold_value: threshold.map(|source| source.value.clone()),
        threshold_date: threshold.map(|source| source.date.to_iso()),
        threshold_source: threshold.and_then(|source| source.source.clone()),
        threshold_base: threshold.and_then(|source| source.base.clone()),
        rule_value: rule.map(|source| source.value.clone()),
        rule_date: rule.map(|source| source.date.to_iso()),
        paced: plan.pacing.missing.is_none(),
        pacing_missing: plan.pacing.missing.clone(),
        created_at,
    };
    let mut sessions = Vec::new();
    for week in plan.weeks.iter() {
        for session in week.sessions.iter() {
            sessions.push(NewAdoptedSession {
                plan_id: 0,
                date: session.date.to_iso(),
                week: week.week as i64,
                kind: session.kind.key().to_string(),
                distance_m: session.distance_m,
                pace_low_s_per_km: ends_of(&session.pace).0,
                pace_high_s_per_km: ends_of(&session.pace).1,
                finish_low_s_per_km: ends_of(&session.finish_pace).0,
                finish_high_s_per_km: ends_of(&session.finish_pace).1,
                pace_rule: session.pace_rule.clone(),
                threshold_value: threshold.map(|source| source.value.clone()),
                detail: session.detail.clone(),
                lt_last_m: session.lt_last_m,
            });
        }
    }
    (row, sessions)
}

/// The two ends of a pace, when it has one.
fn ends_of(value: &PaceValue) -> (Option<f64>, Option<f64>) {
    match value {
        PaceValue::Known(band) => (Some(band.fast_s_per_km), Some(band.slow_s_per_km)),
        _ => (None, None),
    }
}

// ------------------------------------------------------------------ the rows

/// The plan, as the answer carries it.
///
/// Every sentence in it is the core's: the entry requirement beside the
/// athlete's own mileage, the line that names the threshold the plan was
/// paced with, and the line that names the rule it was read from.
pub fn plan_row(plan: &AdoptedPlan, words: &DateWords) -> crate::dto::AdoptedPlanRow {
    use crate::dto::AdoptedPlanRow;
    AdoptedPlanRow {
        programme_id: plan.template_id.clone(),
        programme_version: plan.template_version.clone(),
        label: plan.label.clone(),
        method: plan.method.clone(),
        tier: plan.tier.clone(),
        goal_distance: crate::render::distance_of(plan.goal_distance_m),
        goal_label: plan.goal_distance_label.clone(),
        race_date: plan.race_date.to_iso(),
        race_date_label: crate::routes::day_label(plan.race_date),
        start_date: plan.start_date.to_iso(),
        end_date: plan.end_date.to_iso(),
        adopted_on: plan.adopted_on.to_iso(),
        source: plan.source.as_str().to_string(),
        pacing: pacing_row(&plan_bases::read_pacing(plan, words)),
        entry: entry_row(&plan_bases::read_entry(plan)),
    }
}

/// What the plan was paced with, as the answer carries it.
pub fn pacing_row(pacing: &stride_core::plan_bases::PacingReading) -> crate::dto::PacingRow {
    use crate::dto::PacingRow;
    PacingRow {
        paced: pacing.missing.is_none(),
        threshold: pacing.threshold.as_ref().map(source_row),
        rule: pacing.race.as_ref().map(source_row),
        missing: pacing.missing.clone(),
        threshold_line: pacing.threshold_line.clone(),
        rule_line: pacing.rule_line.clone(),
    }
}

/// One thing the plan's numbers came from.
fn source_row(source: &PacingSource) -> crate::dto::SourceRow {
    crate::dto::SourceRow {
        value: source.value.clone(),
        date: source.date.to_iso(),
        date_label: crate::routes::day_label(source.date),
        source: source.source.clone(),
        base: source.base.clone(),
    }
}

/// The tier's entry requirement beside the athlete's own mileage.
pub fn entry_row(entry: &stride_core::plan_bases::EntryReading) -> crate::dto::EntryRow {
    crate::dto::EntryRow {
        requirement: entry.requirement.clone(),
        plan_band: entry.plan_band.clone(),
        athlete: entry.athlete.clone(),
    }
}

/// The adopted plan's week, as the answer carries it.
pub fn week_row(week: &stride_core::plan_bases::WeekReading) -> crate::dto::AdoptedWeekRow {
    use crate::dto::AdoptedWeekRow;
    AdoptedWeekRow {
        week: week.week,
        weeks_to_go: week.weeks_to_go,
        label: week.label.clone(),
        start_date: week.start_date.to_iso(),
        end_date: week.end_date.to_iso(),
        sessions: week.sessions.iter().map(session_row).collect(),
        source: week.source.source.as_str().to_string(),
        source_line: week.source.line.clone(),
        pacing: pacing_row(&week.pacing),
        entry: entry_row(&week.entry),
    }
}

/// One session of the adopted plan, as the answer carries it.
fn session_row(session: &stride_core::plan_bases::SessionReading) -> crate::dto::AdoptedSessionRow {
    use crate::dto::AdoptedSessionRow;
    AdoptedSessionRow {
        date: session.date.to_iso(),
        weekday: session.weekday.clone(),
        kind: session.kind.clone(),
        method_kind: session.method_kind.clone(),
        distance: session.distance.clone(),
        pace: session.pace.clone(),
        finish_pace: session.finish_pace.clone(),
        detail: session.detail.clone(),
        label: session.label.clone(),
        line: session.line.clone(),
        no_pace: session.no_pace.clone(),
        lt_last: session.lt_last.clone(),
    }
}
