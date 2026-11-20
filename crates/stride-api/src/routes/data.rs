//! The data routes: the runs, the wellness, the fitness and the statistics.

use std::collections::BTreeMap;

use serde_json::Value;

use stride_core::band;
use stride_core::direction::day_directions;
use stride_core::pattern::{patterns, HistoryDay, PatternKind};
use stride_core::types::{Distance, VitalKind};
use stride_core::units;
use stride_core::zones::{time_in_zone, TimeInZone, ZoneFloors};
use stride_store::{ActivitySummary as StoredActivity, HeartRateZoneModel, WellnessDaily};

use crate::dto::{
    ActivityDetail, ActivityListResponse, ActivitySummary, ElevationStats, ElevationWeek,
    FitnessZones, HrvDay, HrvDays, LatestPhysiology, PaceTrends, PaceWeek, PersonalRecords,
    PhysiologyDay, PhysiologyDays, RenderedValue, SleepDay, SleepDays, SourceInfo, Sources,
    StatsSummary, Totals, Trend, TrendPoint, Trends, VitalBand, VitalReading, VitalsHistory,
    VitalsSnapshot, WeekTotals, WeeklyStats, ZoneBand, ZoneModel, ZoneModelFloors,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::series;
use super::{core_date, day_label, day_of, ok, pace_of, today, week_start};

// ------------------------------------------------------------------- the feed

/// The feed: the runs, newest first, in pages.
pub fn activities(api: &Api, request: &Request) -> ApiResult<Value> {
    let limit = match request.query.integer("limit")? {
        Some(value) if value > 0 => value.min(api.config().max_feed_page as i64) as u32,
        Some(_) => return Err(ApiError::bad_request("the limit is not a number")),
        None => api.config().feed_page_size,
    };
    let cursor = match request.query.get("cursor") {
        Some(text) if !text.trim().is_empty() => Some(
            stride_store::FeedCursor::decode(text.trim())
                .map_err(|_| ApiError::bad_request("the cursor is not one this build wrote"))?,
        ),
        _ => None,
    };
    let since = match request.query.get("since") {
        Some(text) if !text.trim().is_empty() => Some(core_date(text.trim())?.to_iso()),
        _ => None,
    };
    let until = match request.query.get("until") {
        Some(text) if !text.trim().is_empty() => Some(core_date(text.trim())?.to_iso()),
        _ => None,
    };
    if matches!((&since, &until), (Some(from), Some(to)) if from > to) {
        return Err(ApiError::bad_request(
            "the date window ends before it starts",
        ));
    }

    let (mut runs, next_cursor) = feed(api, cursor, limit, since, until)?;
    for run in &mut runs {
        if let Some(decoded) = series::decoded(api, run.id) {
            run.geo_points = series::geo_points(&decoded, 64);
        }
    }
    let response = ActivityListResponse {
        activities: runs,
        next_cursor,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// One page of the feed.
///
/// The store pages the feed itself; a window (`since`) is cut here, over
/// the same order the store uses — the local date, then the id, both
/// descending.
fn feed(
    api: &Api,
    cursor: Option<stride_store::FeedCursor>,
    limit: u32,
    since: Option<String>,
    until: Option<String>,
) -> ApiResult<(Vec<ActivitySummary>, Option<String>)> {
    let store = api.store();
    let limit = limit.clamp(1, api.config().max_feed_page);
    match (since, until) {
        (None, None) => {
            let page = store
                .activity_feed(cursor.as_ref(), limit)
                .map_err(|error| api.store_error("the feed could not be read", &error))?;
            let next = page.next_cursor.map(|cursor| cursor.encode());
            let runs = super::summaries(api, &page.activities)?;
            Ok((runs, next))
        }
        (from, to) => {
            let from = from.unwrap_or_else(|| "0001-01-01".to_string());
            let to = to.unwrap_or(today(api)?.to_iso());
            let rows = store
                .activities_between(&from, &to)
                .map_err(|error| api.store_error("the feed could not be read", &error))?;
            let kept: Vec<&StoredActivity> = rows
                .iter()
                .filter(|row| match &cursor {
                    None => true,
                    Some(cursor) => {
                        row.local_date.as_str() < cursor.local_date()
                            || (row.local_date == cursor.local_date() && row.id < cursor.id())
                    }
                })
                .collect();
            let more = kept.len() > limit as usize;
            let page: Vec<&StoredActivity> = kept.into_iter().take(limit as usize).collect();
            let next = match (more, page.last()) {
                (true, Some(last)) => {
                    Some(stride_store::FeedCursor::new(last.local_date.clone(), last.id).encode())
                }
                _ => None,
            };
            let rows: Vec<StoredActivity> = page.iter().map(|row| (*row).clone()).collect();
            let runs = super::summaries(api, &rows)?;
            Ok((runs, next))
        }
    }
}

/// One run, in full.
pub fn activity(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(ApiError::not_found(format!(
                "there is no run with the id {id}"
            )))
        }
    };

    let mut summary = match super::summaries(api, std::slice::from_ref(&run.summary()))?.first() {
        Some(summary) => summary.clone(),
        None => return Err(ApiError::internal("the run could not be read")),
    };
    let mut splits = Vec::new();
    let mut elevation = Vec::new();
    let mut samples = Vec::new();
    let mut why = None;
    if let Some(decoded) = series::decoded(api, id) {
        summary.geo_points = series::geo_points(&decoded, api.config().max_samples_in_response);
        splits = series::splits(&decoded);
        elevation = series::elevation(&decoded, api.config().max_samples_in_response);
        samples = series::samples(&decoded, api.config().max_samples_in_response);
        // The detail knows the run's own shape: the why is the core's,
        // beside the same sentence the feed shows.
        let history = crate::story::history_facts(api)?;
        let facts =
            crate::story::run_facts_of(&run.summary(), crate::story::story_splits(&decoded));
        if let Some(facts) = facts {
            let (_, the_why) = crate::story::story_with_why_of(&facts, &history);
            why = the_why;
        }
    }

    let plan_line = plan_line(api, &run.local_date, run.distance_m)?;
    let note = api
        .store()
        .notes_for_activity(id)
        .ok()
        .and_then(|notes| notes.into_iter().find_map(|note| note.text));

    let detail = ActivityDetail {
        summary,
        splits,
        elevation,
        samples,
        plan_line,
        note,
        why,
    };
    ok(&detail)
}

/// The run's original FIT file, byte for byte.
///
/// The FIT is the source of truth, and this is it — the file Garmin served,
/// downloaded to the athlete's own machine and uploaded nowhere.
pub fn activity_fit(api: &Api, request: &Request) -> ApiResult<crate::router::RawBody> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(ApiError::not_found(format!(
                "there is no run with the id {id}"
            )))
        }
    };
    Ok(crate::router::RawBody {
        content_type: "application/octet-stream",
        bytes: run.fit_blob,
    })
}

/// The plan's line under a run: what was planned and what was run.
fn plan_line(api: &Api, date: &str, actual_m: f64) -> ApiResult<Option<crate::dto::PlanLine>> {
    let sessions = api
        .store()
        .plan_sessions_for_day(date)
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let target = sessions
        .iter()
        .find_map(|session| session.target_distance_m)
        .and_then(Distance::from_metres);
    let actual = Distance::from_metres(actual_m);
    let (actual, target) = match (actual, target) {
        (Some(actual), Some(target)) => (actual, target),
        _ => return Ok(None),
    };
    // The line is the core's statement, and it is absent when the gap from
    // the target is within ten per cent: 10.2 km against a 10 km target is
    // not news, and neither here nor on Today.
    Ok(
        stride_core::statement::distance_comparison_line(actual, target)
            .map(|text| crate::dto::PlanLine { text }),
    )
}

// --------------------------------------------------------------- physiology

/// The window the wellness routes read.
fn window(api: &Api, request: &Request) -> ApiResult<(String, String)> {
    let current = today(api)?;
    let to = match request.query.get("to") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.to_iso(),
    };
    let from = match request.query.get("from") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.add_days(-api.config().health_history_days).to_iso(),
    };
    Ok((from, to))
}

/// The days of wellness.
pub fn physiology_daily(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<PhysiologyDay> = rows.iter().map(physiology_day).collect();
    let response = PhysiologyDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The nights of sleep.
pub fn physiology_sleep(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<SleepDay> = rows
        .iter()
        .map(|row| SleepDay {
            date: row.date.clone(),
            sleep: render::duration(row.sleep_s),
            deep_sleep: render::duration(row.deep_sleep_s),
            rem_sleep: render::duration(row.rem_s),
            light_sleep: render::duration(row.light_sleep_s),
            awake: render::duration(row.awake_s),
        })
        .collect();
    let response = SleepDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The mornings.
pub fn physiology_hrv(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<HrvDay> = rows
        .iter()
        .map(|row| HrvDay {
            date: row.date.clone(),
            hrv: render::hrv(row.hrv_ms),
            resting_hr: render::heart_rate(row.resting_hr),
        })
        .collect();
    let response = HrvDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The vitals and their directions — the snapshot Today and Body read.
pub fn physiology_vitals(api: &Api, request: &Request) -> ApiResult<Value> {
    let day = match request.query.get("date") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?,
        _ => today(api)?,
    };
    let rows = wellness_rows(api, day)?;
    let history = super::wellness_days(&rows);
    let directions = day_directions(&history, day);

    let vitals: Vec<VitalReading> = directions
        .vitals
        .iter()
        .map(|vital| VitalReading {
            key: vital_key(vital.vital).to_string(),
            label: vital.vital.label().to_string(),
            value_text: vital.value_text(),
            direction_text: vital.direction_text().map(|text| text.to_string()),
            deviation_text: vital.deviation_text(),
            travel_text: vital.travel_text(),
            status_text: vital.status_text(),
            band: vital.band.map(|band| VitalBand {
                lower: band.lower(),
                upper: band.upper(),
                nights: band.samples(),
            }),
        })
        .collect();

    let rest_line = {
        let line = directions.rest.line();
        if line.is_empty() {
            directions.rest.status_text()
        } else {
            Some(line)
        }
    };

    let history_window_days = match request.query.integer("days")? {
        Some(days) if (1..=366).contains(&days) => days,
        Some(_) => {
            return Err(ApiError::bad_request(
                "the history window is outside 1 to 366 days",
            ))
        }
        None => band::BAND_WINDOW_NIGHTS,
    };
    let history_from = day.add_days(1 - history_window_days);
    let history_rows = api
        .store()
        .wellness_between(&history_from.to_iso(), &day.to_iso())
        .map_err(|error| api.store_error("the history could not be read", &error))?;
    let history_point = |offset: i64,
                         value: fn(&WellnessDaily) -> Option<f64>,
                         render_value: fn(Option<f64>) -> Option<String>| {
        let date = history_from.add_days(offset).to_iso();
        let value = history_rows
            .iter()
            .find(|row| row.date == date)
            .and_then(value);
        crate::dto::HistoryPoint {
            value,
            text: render_value(value),
            date,
        }
    };
    let charts = VitalsHistory {
        sleep: (0..history_window_days)
            .map(|offset| {
                history_point(
                    offset,
                    |row| row.sleep_s,
                    stride_core::units::render_duration,
                )
            })
            .collect(),
        hrv: (0..history_window_days)
            .map(|offset| {
                history_point(
                    offset,
                    |row| row.hrv_ms,
                    |value| render::hrv(value).map(|value| value.text),
                )
            })
            .collect(),
        resting_hr: (0..history_window_days)
            .map(|offset| {
                history_point(
                    offset,
                    |row| row.resting_hr,
                    stride_core::units::render_heart_rate,
                )
            })
            .collect(),
    };

    let pattern_rows = history_days(api, &rows, day)?;
    let found = patterns(&pattern_rows, day);
    let pattern_readings: Vec<crate::dto::PatternReading> = found
        .iter()
        .map(|pattern| crate::dto::PatternReading {
            id: pattern_id(&pattern.kind),
            text: pattern.text.clone(),
            sample_size: pattern.sample_size,
        })
        .collect();

    let response = VitalsSnapshot {
        date: day.to_iso(),
        generated_at: api.generated_at(),
        rest: crate::dto::RestReading {
            label: stride_core::strings::text("vital.rest.label")
                .unwrap_or("Rest")
                .to_string(),
            line: rest_line,
        },
        vitals,
        history: charts,
        patterns: pattern_readings,
    };
    ok(&response)
}

/// The most recent day of wellness.
pub fn physiology_latest(api: &Api, _request: &Request) -> ApiResult<Value> {
    let current = today(api)?;
    let row = api
        .store()
        .latest_wellness(&current.to_iso())
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let response = LatestPhysiology {
        day: row.as_ref().map(physiology_day),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Where the wellness came from.
pub fn physiology_sources(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let mut sources = Vec::new();
    for (name, count) in [
        (
            "sleep",
            rows.iter().filter(|row| row.sleep_s.is_some()).count(),
        ),
        (
            "hrv",
            rows.iter().filter(|row| row.hrv_ms.is_some()).count(),
        ),
        (
            "resting_hr",
            rows.iter().filter(|row| row.resting_hr.is_some()).count(),
        ),
    ] {
        let latest = rows
            .iter()
            .rev()
            .find(|row| match name {
                "sleep" => row.sleep_s.is_some(),
                "hrv" => row.hrv_ms.is_some(),
                _ => row.resting_hr.is_some(),
            })
            .map(|row| row.date.clone());
        sources.push(SourceInfo {
            name: name.to_string(),
            days: count as i64,
            latest,
        });
    }
    let response = Sources {
        sources,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// One day of wellness, as it travels.
fn physiology_day(row: &WellnessDaily) -> PhysiologyDay {
    PhysiologyDay {
        date: row.date.clone(),
        sleep: render::duration(row.sleep_s),
        deep_sleep: render::duration(row.deep_sleep_s),
        rem_sleep: render::duration(row.rem_s),
        light_sleep: render::duration(row.light_sleep_s),
        awake: render::duration(row.awake_s),
        hrv: render::hrv(row.hrv_ms),
        resting_hr: render::heart_rate(row.resting_hr),
        stress: render::whole(row.stress_avg),
        source: row.source.clone(),
    }
}

/// The wellness of the window that ends with the day.
fn wellness_rows(api: &Api, day: stride_core::date::Date) -> ApiResult<Vec<WellnessDaily>> {
    let from = day.add_days(-band::BAND_WINDOW_NIGHTS).to_iso();
    api.store()
        .wellness_between(&from, &day.to_iso())
        .map_err(|error| api.store_error("the wellness could not be read", &error))
}

/// The history the pattern engine reads: the wellness and the runs.
fn history_days(
    api: &Api,
    rows: &[WellnessDaily],
    day: stride_core::date::Date,
) -> ApiResult<Vec<HistoryDay>> {
    let from = day.add_days(-band::BAND_WINDOW_NIGHTS).to_iso();
    let runs = api
        .store()
        .activities_between(&from, &day.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut distances: BTreeMap<String, f64> = BTreeMap::new();
    for run in runs {
        *distances.entry(run.local_date.clone()).or_insert(0.0) += run.distance_m;
    }
    let mut days = Vec::new();
    for row in rows {
        let date = match core_date(&row.date) {
            Ok(date) => date,
            Err(_) => continue,
        };
        days.push(HistoryDay::new(
            date,
            super::vitals_of(row),
            distances.get(&row.date).copied(),
        ));
    }
    Ok(days)
}

/// The key a vital travels under.
fn vital_key(vital: VitalKind) -> &'static str {
    match vital {
        VitalKind::Sleep => "sleep",
        VitalKind::Hrv => "hrv",
        VitalKind::RestingHr => "resting_hr",
    }
}

/// The id a pattern travels under.
fn pattern_id(kind: &PatternKind) -> String {
    match kind {
        PatternKind::UsualStreak { vital, side } => {
            format!("usual_streak.{}.{}", vital_key(*vital), side.word())
        }
        PatternKind::ComparableOutcome { vital, side } => {
            format!("comparable_outcome.{}.{}", vital_key(*vital), side.word())
        }
    }
}

// ------------------------------------------------------------------ fitness

/// The fingerprints of the runs that have been synced.
///
/// **Endpoint only.** The data is already synced; the route has no screen,
/// and the plan says so in the OpenAPI document so that nobody builds one
/// by accident.
/// The fingerprint of one run: the run's own mechanics, as **observations**.
///
/// The route answers the per-run question — *what did this run's laps say
/// about this run* — from the run's own file, decoded on read, and from the
/// athlete's **stored zone model**. It is not a score and carries no
/// score: every observation is a value or the reason it has none.
///
/// The run is named by the query (`?id=…`), like the zones. The basis is
/// named too — the run's laps and its moving samples — and the zone
/// distribution names the model it was measured against, with the day that
/// model was read.
///
/// **No value is the absence of the whole result.** A run with no file has
/// no observations at all, and says why in the core's words. It is never a
/// zero and never a band.
pub fn fitness_fingerprint(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = match request.query.integer("id")? {
        Some(id) => id,
        None => return Err(ApiError::bad_request("the fingerprint needs a run")),
    };
    let answer = crate::fingerprint::fingerprint(api, id)?;
    ok(&answer)
}

/// The measured trends.
pub fn fitness_trends(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let mut pace_points: Vec<TrendPoint> = Vec::new();
    let mut heart_points: Vec<TrendPoint> = Vec::new();
    for (week, week_runs) in super::weeks(&runs) {
        let paces: Vec<f64> = week_runs
            .iter()
            .filter_map(|run| pace_of(run.distance_m, run.moving_time_s))
            .collect();
        let hearts: Vec<f64> = week_runs.iter().filter_map(|run| run.avg_hr).collect();
        if let Some(pace) = super::median(&paces) {
            pace_points.push(TrendPoint {
                date: week.to_iso(),
                value: render::pace(Some(units::round_pace_s_per_km(pace))),
            });
        }
        if !hearts.is_empty() {
            let mean = hearts.iter().sum::<f64>() / hearts.len() as f64;
            heart_points.push(TrendPoint {
                date: week.to_iso(),
                value: render::heart_rate(Some(units::round_half_even(mean, 1))),
            });
        }
    }
    let trends = vec![
        Trend {
            metric: "median_pace".to_string(),
            label: "Median pace".to_string(),
            unit: "s/km".to_string(),
            baseline: render::pace(super::median(
                &pace_points
                    .iter()
                    .filter_map(|point| point.value.as_ref().and_then(|value| value.raw))
                    .collect::<Vec<f64>>(),
            )),
            points: pace_points,
        },
        Trend {
            metric: "avg_hr".to_string(),
            label: "Average heart rate".to_string(),
            unit: "bpm".to_string(),
            baseline: render::heart_rate(super::median(
                &heart_points
                    .iter()
                    .filter_map(|point| point.value.as_ref().and_then(|value| value.raw))
                    .collect::<Vec<f64>>(),
            )),
            points: heart_points,
        },
    ];
    let response = Trends {
        trends,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The heart-rate zones of a run, from the athlete's **stored zone model**.
///
/// The zone model is a dated fact about the athlete's configuration: the
/// watch's own bands, read from Garmin and stored with the day they were
/// read. The times in the zones are computed from that model and from the
/// run's moving samples, and from nothing else — never from the run's own
/// observed maximum, which would give two runs two different scales.
///
/// The run is named by the query (`?id=…`), like the chips. The basis is
/// named — "moving" — and the model is named with it: its sport, its
/// training method, its maximum heart rate, its five floors and **the day
/// it was read**. A number that cannot say where its input came from and
/// when it was read is not inspectable.
///
/// **No value is the absence of the whole result.** A run with no stored
/// model has no bands and no time outside them, and says why in the core's
/// words. It is never band 1 and never a zero. A stored model whose floors
/// are not strictly increasing cannot be used, and says so the same way.
pub fn fitness_zones(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = match request.query.integer("id")? {
        Some(id) => id,
        None => return Err(ApiError::bad_request("the zones need a run")),
    };
    let exists = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?
        .is_some();
    if !exists {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        )));
    }

    // The athlete's model: the one read most recently.
    let stored = api
        .store()
        .current_hr_zone_model()
        .map_err(|error| api.store_error("the zone model could not be read", &error))?;
    let Some(stored) = stored else {
        return ok(&no_value_zones(None, "zones.no_model"));
    };
    // The day the model was read, in the athlete's own calendar and in the
    // api's own date rendering: "10 May 2026".
    let read_on = day_label(day_of(api, stored.read_at)?);
    let model = zone_model_of(&stored, &read_on);
    let floors = floors_of(&stored);
    if !floors.is_usable() {
        return ok(&no_value_zones(Some(model), "zones.unusable_model"));
    }

    // The samples: the run's own FIT file, decoded on read, and only the
    // moving ones — the same filter the route has always used, so the
    // numbers stay comparable with what it answered before.
    let Some(decoded) = super::series::decoded(api, id) else {
        return ok(&no_value_zones(Some(model), "zones.unavailable"));
    };
    let samples = super::series::zone_samples(&decoded);
    let Some(time) = time_in_zone(floors, &samples) else {
        return ok(&no_value_zones(Some(model), "zones.unusable_model"));
    };

    ok(&zones_of(model, &floors, stored.max_heart_rate, &time))
}

/// The answer when there is no value: no bands, no time outside them, and
/// the reason in the core's own words.
///
/// The api never writes the sentence itself — it asks the core's catalogue
/// for it and passes it on.
fn no_value_zones(model: Option<ZoneModel>, reason: &str) -> FitnessZones {
    FitnessZones {
        basis: None,
        model,
        zones: Vec::new(),
        outside: None,
        unavailable: stride_core::strings::render(reason, &[]),
    }
}

/// The zones of a run, against the athlete's stored model.
fn zones_of(
    model: ZoneModel,
    floors: &ZoneFloors,
    max_heart_rate: Option<f64>,
    time: &TimeInZone,
) -> FitnessZones {
    let times = time.per_zone();
    let edges = floors_of_edges(floors, max_heart_rate);
    let zones = edges
        .iter()
        .enumerate()
        .map(|(index, (from, to))| {
            // The edges are the same truncated pair the words were rendered
            // from — the core truncates both, once, so the chart and the
            // words cannot disagree.
            let rendered = units::render_heart_rate_range_edges(Some(*from), *to);
            let (from_bpm, to_bpm) = match rendered {
                Some((lower, upper)) => (Some(lower), Some(upper)),
                // The top band of a model with no maximum heart rate: it has
                // a floor, and no upper edge to draw.
                None => (Some(units::truncate_heart_rate(*from)), None),
            };
            ZoneBand {
                label: units::render_zone_label(index + 1)
                    .unwrap_or_else(|| format!("Z{}", index + 1)),
                range: units::render_heart_rate_range(Some(*from), *to),
                from_bpm,
                to_bpm,
                time: render::duration(Some(times[index])),
            }
        })
        .collect();

    FitnessZones {
        basis: Some("moving".to_string()),
        model: Some(model),
        zones,
        outside: render::duration(Some(time.outside_s)),
        unavailable: None,
    }
}

/// The five bands' edges, in beats per minute.
///
/// The bands are the model's own: zone *n* runs from `floor_n` to
/// `floor_{n+1}`, and the top band runs from `floor_5` to the maximum heart
/// rate the model was built on — which Garmin may not have reported, in
/// which case that band has no upper edge.
fn floors_of_edges(floors: &ZoneFloors, max_heart_rate: Option<f64>) -> [(f64, Option<f64>); 5] {
    stride_core::zones::zone_edges(floors, max_heart_rate)
}

/// The stored model's floors, as the core takes them.
fn floors_of(model: &HeartRateZoneModel) -> ZoneFloors {
    ZoneFloors {
        zone1: model.zone1_floor,
        zone2: model.zone2_floor,
        zone3: model.zone3_floor,
        zone4: model.zone4_floor,
        zone5: model.zone5_floor,
    }
}

/// The model as the payload names it.
///
/// The date the model was read is the **api's** rendering of it: the day,
/// in the athlete's own calendar, written the way the api writes a day —
/// "10 May 2026". The core's canonical form stays inside the core, so the
/// source line the core renders carries the api's form and not a third
/// one.
fn zone_model_of(model: &HeartRateZoneModel, read_on: &str) -> ZoneModel {
    ZoneModel {
        sport: model.sport.clone(),
        training_method: model.training_method.clone(),
        max_heart_rate: render::heart_rate(model.max_heart_rate),
        floors: ZoneModelFloors {
            zone1: floor_value(model.zone1_floor),
            zone2: floor_value(model.zone2_floor),
            zone3: floor_value(model.zone3_floor),
            zone4: floor_value(model.zone4_floor),
            zone5: floor_value(model.zone5_floor),
        },
        read_at: crate::state::format_instant(model.read_at),
        read_on: read_on.to_string(),
        source: stride_core::model_source(&model.training_method, model.max_heart_rate, read_on),
    }
}

/// A floor, as the core renders it.
///
/// The store's floors are numbers, and the core renders every one of them;
/// the fallback is the unreachable case, and it carries the number without
/// words rather than an invented sentence.
fn floor_value(bpm: f64) -> RenderedValue {
    render::heart_rate(Some(bpm)).unwrap_or(RenderedValue {
        text: String::new(),
        raw: Some(bpm),
    })
}

// -------------------------------------------------------------------- stats

/// The totals, over four windows.
pub fn stats_summary(api: &Api, request: &Request) -> ApiResult<Value> {
    let current = today(api)?;
    let year_start = current.add_days(-day_of_year(current) + 1).to_iso();
    let month_start = current.add_days(-(current.day() as i64) + 1).to_iso();
    let week = week_start(current).to_iso();

    let all = stored_runs(api, request)?;
    let this_year: Vec<&StoredActivity> = all
        .iter()
        .filter(|run| run.local_date >= year_start)
        .collect();
    let this_month: Vec<&StoredActivity> = all
        .iter()
        .filter(|run| run.local_date >= month_start)
        .collect();
    let this_week: Vec<&StoredActivity> = all.iter().filter(|run| run.local_date >= week).collect();

    let response = StatsSummary {
        all_time: totals(all.iter()),
        this_year: totals(this_year),
        this_month: totals(this_month),
        this_week: totals(this_week),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The totals of a set of runs.
fn totals<'a>(runs: impl IntoIterator<Item = &'a StoredActivity>) -> Totals {
    let runs: Vec<&StoredActivity> = runs.into_iter().collect();
    let distance_m: f64 = render::total_of(runs.iter().map(|run| run.distance_m));
    let moving_time_s: f64 = render::total_of(runs.iter().map(|run| run.moving_time_s));
    let hearts: Vec<f64> = runs.iter().filter_map(|run| run.avg_hr).collect();
    let avg_pace_s_per_km = if distance_m > 0.0 {
        Some(units::round_pace_s_per_km(
            moving_time_s / (distance_m / 1000.0),
        ))
    } else {
        None
    };
    Totals {
        activity_count: runs.len() as i64,
        distance: render::distance(Some(distance_m)),
        elevation_gain: render::elevation(Some(render::total_of(
            runs.iter().map(|run| run.elevation_gain_m),
        ))),
        duration: render::duration(Some(moving_time_s)),
        avg_pace: render::pace(avg_pace_s_per_km),
        heart_rate: render::heart_rate(if hearts.is_empty() {
            None
        } else {
            Some(units::round_half_even(
                hearts.iter().sum::<f64>() / hearts.len() as f64,
                1,
            ))
        }),
    }
}

/// The weeks of running.
pub fn stats_weekly(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<WeekTotals> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| {
            let distance_m: f64 = render::total_of(week_runs.iter().map(|run| run.distance_m));
            let moving_time_s: f64 =
                render::total_of(week_runs.iter().map(|run| run.moving_time_s));
            WeekTotals {
                week_start: week.to_iso(),
                activity_count: week_runs.len() as i64,
                distance: render::distance(Some(distance_m)),
                duration: render::duration(Some(moving_time_s)),
                elevation_gain: render::elevation(Some(render::total_of(
                    week_runs.iter().map(|run| run.elevation_gain_m),
                ))),
                avg_pace: render::pace(if distance_m > 0.0 {
                    Some(units::round_pace_s_per_km(
                        moving_time_s / (distance_m / 1000.0),
                    ))
                } else {
                    None
                }),
            }
        })
        .collect();
    let response = WeeklyStats {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The pace, week by week.
pub fn stats_pace_trends(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<PaceWeek> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| {
            let paces: Vec<f64> = week_runs
                .iter()
                .filter_map(|run| pace_of(run.distance_m, run.moving_time_s))
                .collect();
            PaceWeek {
                week_start: week.to_iso(),
                run_count: week_runs.len() as i64,
                median_pace: render::pace(super::median(&paces).map(units::round_pace_s_per_km)),
                best_pace: render::pace(
                    paces
                        .iter()
                        .copied()
                        .fold(None, |best: Option<f64>, pace| {
                            Some(match best {
                                Some(current) => current.min(pace),
                                None => pace,
                            })
                        })
                        .map(units::round_pace_s_per_km),
                ),
            }
        })
        .collect();
    let response = PaceTrends {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The climbing, week by week.
pub fn stats_elevation(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<ElevationWeek> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| ElevationWeek {
            week_start: week.to_iso(),
            elevation_gain: render::elevation(Some(render::total_of(
                week_runs.iter().map(|run| run.elevation_gain_m),
            ))),
        })
        .collect();
    let response = ElevationStats {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The personal records.
pub fn stats_personal_records(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let records = super::personal_records(api, &runs)?;
    let response = PersonalRecords {
        records,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The runs the statistics routes read.
fn stored_runs(api: &Api, request: &Request) -> ApiResult<Vec<StoredActivity>> {
    let current = today(api)?;
    let to = match request.query.get("to") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.to_iso(),
    };
    let from = match request.query.get("from") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.add_days(-api.config().history_months * 30).to_iso(),
    };
    let runs = api
        .store()
        .activities_between(&from, &to)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    Ok(runs)
}

/// The day of the year, as the days of a year are counted.
fn day_of_year(day: stride_core::date::Date) -> i64 {
    let start = stride_core::date::Date::from_ymd(day.year(), 1, 1);
    match start {
        Some(start) => day.days_since(start) + 1,
        None => 1,
    }
}
