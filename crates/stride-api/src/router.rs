//! The route table.
//!
//! One entry per endpoint of the Python app. The table is the contract: the
//! phone asks for a path, the router finds the handler, and the handler
//! shapes the answer.
//!
//! Matching is by segment, not by string: `/api/v1/activities/12` and
//! `/api/v1/activities/12/geo` are different routes, and a path that matches
//! no route is a 404 — the same one FastAPI gives.

use serde_json::Value;

use crate::backend::Backend;
use crate::error::ApiError;
use crate::query::Query;
use crate::routes::{self, Ctx};
use crate::Response;

/// One segment of a path pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Segment {
    Literal(&'static str),
    /// `{name}` — and, for the map tiles, `{name}.png`.
    Param {
        name: &'static str,
        suffix: &'static str,
    },
}

/// The endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteId {
    Health,
    HealthDaily,
    AuthStatus,
    AuthPersonalKey,
    AuthPersonalKeyTest,
    AuthDeveloperKey,
    AuthCallback,
    AuthExchange,
    AuthLogin,
    AuthMfa,
    AuthLogout,
    AuthFitImport,
    Activities,
    Activity,
    ActivityPatch,
    ActivityGeo,
    ActivityStream,
    StatsSummary,
    StatsWeekly,
    StatsMonthly,
    StatsPaceTrends,
    StatsHrZones,
    StatsPersonalRecords,
    StatsHeatmap,
    StatsElevation,
    Calendar,
    SyncRun,
    SyncStatus,
    SyncRuns,
    SyncHistory,
    SyncHistoryPost,
    SyncSettings,
    NetworkStatus,
    NetworkRoute,
    PlanBlocks,
    PlanBlocksPost,
    PlanBlock,
    PlanBlockPhases,
    PlanPhaseWeeks,
    PlanWeekSessions,
    PlanImport,
    PlanCurrent,
    PlanWeek,
    PlanResults,
    PlanAdherence,
    PlanSources,
    PlanToday,
    PhysiologyDaily,
    PhysiologyLatest,
    PhysiologySleep,
    PhysiologyHrv,
    PhysiologyReadiness,
    PhysiologySources,
    FitnessFingerprint,
    FitnessActivity,
    FitnessTrends,
    FitnessZones,
    Gear,
    CompareCandidates,
    Compare,
    TilePolicy,
    Tile,
    InsightsToday,
    InsightsActivity,
    InsightsChips,
    InsightsProgress,
}

/// One entry of the route table.
pub struct Route {
    pub method: &'static str,
    pub pattern: &'static str,
    pub id: RouteId,
}

impl Route {
    /// The segments of the pattern.
    fn segments(&self) -> Vec<Segment> {
        self.pattern
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|part| {
                if let Some(rest) = part.strip_prefix('{') {
                    // `{y}.png` — a parameter with a suffix on it.
                    if let Some(name) = rest.strip_suffix("}.png") {
                        return Segment::Param {
                            name,
                            suffix: ".png",
                        };
                    }
                    let name = rest.strip_suffix('}').unwrap_or(rest);
                    Segment::Param { name, suffix: "" }
                } else {
                    Segment::Literal(part)
                }
            })
            .collect()
    }
}

/// The route table, in the order the paths are tried.
pub const ROUTES: &[Route] = &[
    Route {
        method: "GET",
        pattern: "/api/v1/health",
        id: RouteId::Health,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/health/daily",
        id: RouteId::HealthDaily,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/auth/status",
        id: RouteId::AuthStatus,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/personal-key",
        id: RouteId::AuthPersonalKey,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/personal-key/test",
        id: RouteId::AuthPersonalKeyTest,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/developer-key",
        id: RouteId::AuthDeveloperKey,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/auth/garmin/callback",
        id: RouteId::AuthCallback,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/exchange",
        id: RouteId::AuthExchange,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/login",
        id: RouteId::AuthLogin,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/mfa",
        id: RouteId::AuthMfa,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/logout",
        id: RouteId::AuthLogout,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/auth/garmin/fit-import",
        id: RouteId::AuthFitImport,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/activities",
        id: RouteId::Activities,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/activities/{activity_id}",
        id: RouteId::Activity,
    },
    Route {
        method: "PATCH",
        pattern: "/api/v1/activities/{activity_id}",
        id: RouteId::ActivityPatch,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/activities/{activity_id}/geo",
        id: RouteId::ActivityGeo,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/activities/{activity_id}/streams/{kind}",
        id: RouteId::ActivityStream,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/summary",
        id: RouteId::StatsSummary,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/weekly",
        id: RouteId::StatsWeekly,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/monthly",
        id: RouteId::StatsMonthly,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/pace-trends",
        id: RouteId::StatsPaceTrends,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/hr-zones",
        id: RouteId::StatsHrZones,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/personal-records",
        id: RouteId::StatsPersonalRecords,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/heatmap",
        id: RouteId::StatsHeatmap,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/stats/elevation",
        id: RouteId::StatsElevation,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/calendar",
        id: RouteId::Calendar,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/sync/run",
        id: RouteId::SyncRun,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/sync/status",
        id: RouteId::SyncStatus,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/sync/runs",
        id: RouteId::SyncRuns,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/sync/history",
        id: RouteId::SyncHistory,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/sync/history",
        id: RouteId::SyncHistoryPost,
    },
    Route {
        method: "PATCH",
        pattern: "/api/v1/sync/settings",
        id: RouteId::SyncSettings,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/network/status",
        id: RouteId::NetworkStatus,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/network/route",
        id: RouteId::NetworkRoute,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/blocks",
        id: RouteId::PlanBlocks,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/plan/blocks",
        id: RouteId::PlanBlocksPost,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/blocks/{block_id}",
        id: RouteId::PlanBlock,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/plan/blocks/{block_id}/phases",
        id: RouteId::PlanBlockPhases,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/plan/phases/{phase_id}/weeks",
        id: RouteId::PlanPhaseWeeks,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/plan/weeks/{week_id}/sessions",
        id: RouteId::PlanWeekSessions,
    },
    Route {
        method: "POST",
        pattern: "/api/v1/plan/import",
        id: RouteId::PlanImport,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/current",
        id: RouteId::PlanCurrent,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/week",
        id: RouteId::PlanWeek,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/results",
        id: RouteId::PlanResults,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/adherence",
        id: RouteId::PlanAdherence,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/sources",
        id: RouteId::PlanSources,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/plan/today",
        id: RouteId::PlanToday,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/daily",
        id: RouteId::PhysiologyDaily,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/latest",
        id: RouteId::PhysiologyLatest,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/sleep",
        id: RouteId::PhysiologySleep,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/hrv",
        id: RouteId::PhysiologyHrv,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/readiness",
        id: RouteId::PhysiologyReadiness,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/physiology/sources",
        id: RouteId::PhysiologySources,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/fitness/fingerprint",
        id: RouteId::FitnessFingerprint,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/fitness/activity/{id}",
        id: RouteId::FitnessActivity,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/fitness/trends",
        id: RouteId::FitnessTrends,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/fitness/zones",
        id: RouteId::FitnessZones,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/gear",
        id: RouteId::Gear,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/compare/candidates",
        id: RouteId::CompareCandidates,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/compare",
        id: RouteId::Compare,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/tiles/policy",
        id: RouteId::TilePolicy,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/tiles/{z}/{x}/{y}.png",
        id: RouteId::Tile,
    },
    // The insight routes have no Python reference: see `insights.rs`.
    Route {
        method: "GET",
        pattern: "/api/v1/insights/today",
        id: RouteId::InsightsToday,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/insights/activity/{id}",
        id: RouteId::InsightsActivity,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/insights/chips",
        id: RouteId::InsightsChips,
    },
    Route {
        method: "GET",
        pattern: "/api/v1/insights/progress",
        id: RouteId::InsightsProgress,
    },
];

/// The parameters a route captured.
#[derive(Debug, Default)]
pub struct Params {
    values: Vec<(&'static str, String)>,
}

impl Params {
    /// The value of a parameter.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The value of a parameter, parsed as an integer.
    ///
    /// A parameter that is not an integer is FastAPI's 422: the path
    /// `/api/v1/activities/abc` names an activity that does not exist, and
    /// the reference says so with a validation error, not with a 404.
    pub fn int(&self, name: &str) -> Result<i64, ApiError> {
        let raw = self.get(name).unwrap_or_default();
        raw.parse::<i64>().map_err(|_| {
            crate::error::validation_error(
                vec!["path", name],
                "Input should be a valid integer, unable to parse string as an integer",
                "int_parsing",
            )
        })
    }
}

/// Match a path against the route table.
///
/// A path that matches a route with another method is a 405, and a path that
/// matches no route at all is a 404 — the same two answers FastAPI gives.
pub fn match_route(method: &str, path: &str) -> Result<(RouteId, Params), ApiError> {
    let request_segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    let mut path_matched = false;
    for route in ROUTES {
        let segments = route.segments();
        if segments.len() != request_segments.len() {
            continue;
        }
        let mut params = Params::default();
        let mut matched = true;
        for (pattern, actual) in segments.iter().zip(request_segments.iter()) {
            match pattern {
                Segment::Literal(literal) => {
                    if literal != actual {
                        matched = false;
                        break;
                    }
                }
                Segment::Param { name, suffix } => {
                    let Some(stripped) = actual.strip_suffix(*suffix) else {
                        matched = false;
                        break;
                    };
                    if stripped.is_empty() {
                        matched = false;
                        break;
                    }
                    params.values.push((name, stripped.to_string()));
                }
            }
        }
        if !matched {
            continue;
        }
        path_matched = true;
        if route.method == method {
            return Ok((route.id, params));
        }
    }
    if path_matched {
        return Err(ApiError::new(405, "Method Not Allowed"));
    }
    Err(ApiError::not_found("Not found"))
}

/// Run the handler of a route.
pub fn dispatch(
    ctx: &Ctx<'_>,
    id: RouteId,
    params: &Params,
    query: &Query,
    body: Option<&str>,
) -> Result<Response, ApiError> {
    let json = |value: Result<Value, ApiError>| -> Result<Response, ApiError> {
        value.map(Response::Json)
    };
    match id {
        RouteId::Health => json(routes::health(ctx)),
        RouteId::HealthDaily => json(routes::health_daily(ctx, query)),
        RouteId::AuthStatus => json(routes::auth_status(ctx)),
        RouteId::AuthPersonalKey => json(routes::connect_personal_key(ctx, body)),
        RouteId::AuthPersonalKeyTest => json(routes::test_personal_key(ctx, body)),
        RouteId::AuthDeveloperKey => json(routes::garmin_developer_key(ctx, body)),
        RouteId::AuthCallback => json(routes::garmin_callback(ctx, query)),
        RouteId::AuthExchange => json(routes::garmin_exchange(ctx, body)),
        RouteId::AuthLogin => json(routes::garmin_login(ctx, body)),
        RouteId::AuthMfa => json(routes::garmin_mfa(ctx, body)),
        RouteId::AuthLogout => json(routes::garmin_logout(ctx)),
        RouteId::AuthFitImport => json(routes::fit_import(ctx, body)),
        RouteId::Activities => json(routes::list_activities(ctx, query)),
        RouteId::Activity => json(routes::get_activity(ctx, params.int("activity_id")?, query)),
        RouteId::ActivityPatch => json(routes::patch_activity(
            ctx,
            params.int("activity_id")?,
            body,
        )),
        RouteId::ActivityGeo => json(routes::activity_geo(ctx, params.int("activity_id")?)),
        RouteId::ActivityStream => json(routes::activity_stream(
            ctx,
            params.int("activity_id")?,
            params.get("kind").unwrap_or_default(),
        )),
        RouteId::StatsSummary => json(routes::stats_summary(ctx)),
        RouteId::StatsWeekly => json(routes::stats_weekly(ctx, query)),
        RouteId::StatsMonthly => json(routes::stats_monthly(ctx, query)),
        RouteId::StatsPaceTrends => json(routes::stats_pace_trends(ctx, query)),
        RouteId::StatsHrZones => json(routes::stats_hr_zones(ctx, query)),
        RouteId::StatsPersonalRecords => json(routes::stats_personal_records(ctx, query)),
        RouteId::StatsHeatmap => json(routes::stats_heatmap(ctx, query)),
        RouteId::StatsElevation => json(routes::stats_elevation(ctx, query)),
        RouteId::Calendar => json(routes::calendar(ctx, query)),
        RouteId::SyncRun => json(routes::sync_run(ctx, body)),
        RouteId::SyncStatus => json(routes::sync_status(ctx)),
        RouteId::SyncRuns => json(routes::sync_runs(ctx, query)),
        RouteId::SyncHistory => json(routes::history_settings(ctx)),
        RouteId::SyncHistoryPost => json(routes::load_history(ctx, body)),
        RouteId::SyncSettings => json(routes::patch_settings(ctx, body)),
        RouteId::NetworkStatus => json(routes::network_status(ctx)),
        RouteId::NetworkRoute => json(routes::set_route(ctx, body)),
        RouteId::PlanBlocks => json(routes::plan_blocks(ctx, query)),
        RouteId::PlanBlocksPost => json(routes::create_plan_block(ctx, body)),
        RouteId::PlanBlock => json(routes::get_plan_block(ctx, params.int("block_id")?)),
        RouteId::PlanBlockPhases => {
            json(routes::add_plan_phase(ctx, params.int("block_id")?, body))
        }
        RouteId::PlanPhaseWeeks => json(routes::add_plan_week(ctx, params.int("phase_id")?, body)),
        RouteId::PlanWeekSessions => {
            json(routes::add_plan_session(ctx, params.int("week_id")?, body))
        }
        RouteId::PlanImport => json(routes::import_plan(ctx, body)),
        RouteId::PlanCurrent => json(routes::plan_current(ctx, query)),
        RouteId::PlanWeek => json(routes::plan_week(ctx, query)),
        RouteId::PlanResults => json(routes::plan_results(ctx, query)),
        RouteId::PlanAdherence => json(routes::plan_adherence(ctx, query)),
        RouteId::PlanSources => json(routes::plan_sources(ctx, query)),
        RouteId::PlanToday => json(routes::plan_today(ctx, query)),
        RouteId::PhysiologyDaily => json(routes::physiology_daily(ctx, query)),
        RouteId::PhysiologyLatest => json(routes::physiology_latest(ctx, query)),
        RouteId::PhysiologySleep => json(routes::physiology_sleep(ctx, query)),
        RouteId::PhysiologyHrv => json(routes::physiology_hrv(ctx, query)),
        RouteId::PhysiologyReadiness => json(routes::physiology_readiness(ctx, query)),
        RouteId::PhysiologySources => json(routes::physiology_sources(ctx)),
        RouteId::FitnessFingerprint => json(routes::fitness_fingerprint(ctx, query)),
        RouteId::FitnessActivity => json(routes::fitness_activity(ctx, params.int("id")?, query)),
        RouteId::FitnessTrends => json(routes::fitness_trends(ctx, query)),
        RouteId::FitnessZones => json(routes::fitness_zones(ctx, query)),
        RouteId::Gear => json(routes::gear(ctx)),
        RouteId::CompareCandidates => json(routes::compare_candidates(ctx, query)),
        RouteId::Compare => json(routes::compare(ctx, query)),
        RouteId::TilePolicy => json(routes::tile_policy(ctx)),
        RouteId::InsightsToday => json(crate::insights::today(ctx)),
        RouteId::InsightsActivity => json(crate::insights::activity(ctx, params.int("id")?)),
        RouteId::InsightsChips => json(crate::insights::chips(ctx, query)),
        RouteId::InsightsProgress => json(crate::insights::progress(ctx)),
        RouteId::Tile => routes::tile(ctx, params),
    }
}

/// The backend the router reads from, for the callers that need it.
pub fn backend_of<'a>(ctx: &'a Ctx<'_>) -> &'a dyn Backend {
    ctx.backend
}
