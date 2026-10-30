/**
 * Synthetic fixtures for the phone smoke test.
 *
 * **Every number here is invented.** There is no real run, no real Garmin
 * key and no real health file in this file — it is a set of answers that
 * have the *shape* the Rust core produces, so the phone can be rendered
 * against them without a database and without anybody's data.
 *
 * The shapes come from `crates/stride-core/src/insights.rs` (Today,
 * Progress, RunStory) and from `frontend/src/types.ts` /
 * `frontend/src/types_phase2.ts` (the physiology and activity rows).
 *
 * The fixtures are keyed by the route path the phone's bridge is asked
 * for, so the harness can hand them straight to the Capacitor stub.
 */

/** The day the fixtures pretend it is. */
export const TODAY = '2026-10-30'

/** A date `days` before today, `YYYY-MM-DD`. */
function daysAgo(days) {
  const date = new Date(`${TODAY}T00:00:00Z`)
  date.setUTCDate(date.getUTCDate() - days)
  return date.toISOString().slice(0, 10)
}

/** One synthetic activity, in the shape the feed draws. */
function activity(id, name, days, distanceM, movingS, pace, hr, sport = 'running') {
  const date = daysAgo(days)
  return {
    id,
    source: 'garmin_personal',
    external_id: `synthetic-${id}`,
    name,
    sport_type: sport,
    description: null,
    start_time: `${date}T06:30:00Z`,
    local_date: date,
    day_of_week: 'Friday',
    distance_m: distanceM,
    moving_time_s: movingS,
    moving_time_label: null,
    elapsed_time_s: movingS + 120,
    elevation_gain_m: 60 + id * 3,
    avg_hr: hr,
    max_hr: hr + 22,
    calories: Math.round((movingS / 60) * 11),
    avg_cadence: 178,
    max_cadence: 186,
    avg_speed_ms: 1000 / pace,
    max_speed_ms: 1000 / (pace - 20),
    pace_s_per_km: pace,
    pace_label: null,
    perceived_exertion: null,
    device: 'Synthetic watch',
    gear: null,
    is_trail: false,
    is_commute: false,
    garmin_url: null,
    has_geo: false,
    route_preview: null,
    lap_count: 10,
    running_time_s: movingS,
    walking_time_s: 0,
    stopped_time_s: 0,
    running_distance_m: distanceM,
  }
}

/** The synthetic runs, newest first. */
const ACTIVITIES = [
  activity(901, 'Morning easy', 1, 8000, 2400, 300, 148),
  activity(902, 'Lunch run', 3, 5000, 1620, 324, 152),
  activity(903, 'Long run', 5, 18000, 5700, 317, 155),
  activity(904, 'Tempo', 8, 10000, 3000, 300, 163),
  activity(905, 'Recovery jog', 10, 6000, 2100, 350, 141),
  activity(906, 'Hill repeats', 13, 9000, 2700, 300, 160),
  activity(907, 'Easy', 16, 7000, 2280, 326, 149),
  activity(908, 'Parkrun', 19, 5000, 1500, 300, 158),
]

/** The week, Monday to Sunday, with two runs in it and the rest ahead. */
const WEEK = {
  days: [
    { date: '2026-10-26', distance_m: 5000, ran: true, future: false },
    { date: '2026-10-27', distance_m: 0, ran: false, future: false },
    { date: '2026-10-28', distance_m: 18000, ran: true, future: false },
    { date: '2026-10-29', distance_m: 0, ran: false, future: false },
    { date: '2026-10-30', distance_m: 8000, ran: true, future: false },
    { date: '2026-10-31', distance_m: 0, ran: false, future: true },
    { date: '2026-11-01', distance_m: 0, ran: false, future: true },
  ],
  distance_m: 31000,
  runs: 3,
  usual_distance_m: 32000,
}

/** The story of the last run. */
const LAST_RUN_STORY = {
  id: 901,
  chip: 'Usual run',
  tone: 'neutral',
  story: ['A steady 8 km at an easy pace.', 'Heart rate stayed where it usually sits.'],
  spike: false,
  longest_prior_30d_m: 18000,
  similar_runs: 11,
  efficiency_delta_pct: 1.4,
  pace_delta_s_km: -4,
  hr_delta_bpm: 2,
  bests: [],
  recovery: { flags: ['sleep_short'], normal: 2, read: 3 },
}

/**
 * `/insights/today`.
 *
 * The advice is the graded kind: one signal that is just outside its
 * normal range eases the plan and leaves the day to the runner. The
 * screenshot has to show the advice the app actually gives, so the copy
 * here is the copy the core writes.
 */
const TODAY_INSIGHT = {
  date: TODAY,
  state: 'easy',
  headline: 'Keep it easy today.',
  reason:
    'Last night: resting heart rate 57 vs your usual 51. Just outside your normal range — if you feel off too, take the day.',
  guardrail: null,
  longest_30d_m: 18000,
  spike_limit_m: 19800,
  days_since_last_run: 1,
  usual_gap_days: 2.0,
  last_run: {
    run: {
      id: 901,
      name: 'Morning easy',
      date: daysAgo(1),
      distance_m: 8000,
      moving_time_s: 2400,
      avg_hr: 148,
    },
    story: LAST_RUN_STORY,
  },
  week: WEEK,
  recovery_normal: [2, 3],
  recovery_signals: [
    { key: 'sleep', normal: true },
    { key: 'hrv', normal: true },
    { key: 'resting_hr', normal: false },
  ],
  plan: {
    session: { label: 'Easy', session_type: 'easy', target_distance_m: 8000 },
    line: 'Your plan says Easy, 8 km. Nothing says otherwise: run it as written.',
  },
}

/** `/insights/progress`. */
const PROGRESS = {
  fitness: {
    headline: '3.1% faster at the same heart rate',
    delta_pct: 3.1,
    recent_runs: 12,
    earlier_runs: 9,
    pace_now_s_km: 300,
    pace_then_s_km: 309,
    at_hr: 150,
  },
  consistency: {
    runs_per_week: 4,
    current_weeks: 6,
    best_weeks: 11,
    grace_available: true,
    weeks: Array.from({ length: 52 }, (_, index) => ({
      start: daysAgo((52 - index) * 7),
      runs: index % 7 === 3 ? 1 : 3 + (index % 3),
      distance_m: 18000 + (index % 5) * 6000,
      status:
        index % 7 === 3 ? 'grace' : index % 13 === 5 ? 'gap' : index === 51 ? 'current' : 'active',
    })),
    active_weeks_this_year: 30,
  },
  bests: [
    {
      label: '5K',
      distance_m: 5000,
      scope: 'all_time',
      activity_id: 908,
      date: daysAgo(19),
      pace_s_km: 300,
      run_distance_m: 5000,
    },
    {
      label: '10K',
      distance_m: 10000,
      scope: 'all_time',
      activity_id: 904,
      date: daysAgo(8),
      pace_s_km: 300,
      run_distance_m: 10000,
    },
    {
      label: 'Half',
      distance_m: 21097,
      scope: 'all_time',
      activity_id: 903,
      date: daysAgo(5),
      pace_s_km: 317,
      run_distance_m: 18000,
    },
  ],
  milestones: {
    total_m: 1_234_000,
    total_runs: 186,
    runner_since: '2024-03-02',
    longest_m: 32000,
    tiers_reached_km: [100, 500, 1000],
    next_tier_km: 1500,
    this_year_m: 380_000,
    this_year_runs: 52,
  },
  race_estimates: [
    { label: '5K', low_s: 1140, high_s: 1260 },
    { label: '10K', low_s: 2400, high_s: 2700 },
    { label: 'Half', low_s: 5400, high_s: 6000 },
    { label: 'Marathon', low_s: 11400, high_s: 12600 },
  ],
  race_basis: 'From your best recent 5 km pace.',
}

/** `/physiology/sources`. */
const SOURCES = {
  available: true,
  reason: null,
  auth_method: 'developer_key',
  endpoints: [
    { name: 'sleep', available: true, note: null },
    { name: 'hrv', available: true, note: null },
    { name: 'resting_heart_rate', available: true, note: null },
  ],
  last_synced_at: '2026-10-30T06:12:00Z',
  days_available: 90,
}

/** `/physiology/latest`. */
const LATEST = {
  day: {
    date: TODAY,
    resting_hr: 49,
    max_hr: 171,
    sleep_s: 25200,
    deep_sleep_s: 5400,
    rem_s: 6300,
    light_sleep_s: 12600,
    awake_s: 900,
    hrv_ms: 52,
    stress: 28,
    steps: 9100,
    body_battery: 68,
    readiness: 74,
    distance_m: 8000,
  },
  readiness: {
    date: TODAY,
    score: 74,
    grade: 'good',
    drivers: [
      {
        key: 'sleep',
        label: 'Sleep',
        value: 7.0,
        unit: 'h',
        baseline: 7.4,
        delta: -0.4,
        contribution: 0.28,
        detail: '7 h 00 m, against a 7 h 24 m baseline.',
      },
      {
        key: 'sleep_score',
        label: 'Sleep score',
        value: 78,
        unit: '',
        baseline: 80,
        delta: -2,
        contribution: 0.22,
        detail: 'In your usual band.',
      },
      {
        key: 'hrv',
        label: 'HRV',
        value: 52,
        unit: 'ms',
        baseline: 58,
        delta: -6,
        contribution: 0.27,
        detail: 'Below your 30-day baseline.',
      },
      {
        key: 'resting_hr',
        label: 'Resting heart rate',
        value: 49,
        unit: 'bpm',
        baseline: 48,
        delta: 1,
        contribution: 0.23,
        detail: 'A touch above your baseline.',
      },
    ],
    recommendation: 'Run easy today.',
    rationale: 'Sleep and HRV are inside your normal range; resting heart rate is a shade high.',
  },
}

/** `/physiology/sleep`. */
const SLEEP = {
  items: Array.from({ length: 30 }, (_, index) => {
    const date = daysAgo(index)
    const total = 25200 + (index % 5) * 900
    return {
      date,
      sleep_s: total,
      sleep_score: 72 + (index % 6),
      deep_sleep_s: 5400 + (index % 4) * 300,
      rem_s: 6300 + (index % 3) * 300,
      light_sleep_s: total - 5400 - 6300 - 900,
      awake_s: 900 + (index % 3) * 120,
    }
  }),
}

/** `/physiology/hrv`. */
const HRV = {
  items: Array.from({ length: 30 }, (_, index) => ({
    date: daysAgo(index),
    hrv_ms: 52 + (index % 7),
    baseline: 58,
    resting_hr: 49 + (index % 4),
    rhr_baseline: 48,
  })),
  baseline: 58,
}

/** `/physiology/readiness`. */
const READINESS = {
  items: Array.from({ length: 30 }, (_, index) => ({
    date: daysAgo(index),
    score: 68 + (index % 9),
    grade: index % 4 === 0 ? 'peak' : index % 3 === 0 ? 'fair' : 'good',
  })),
}

/** `/auth/status`. */
const AUTH_STATUS = {
  garmin_connected: true,
  garmin_method: 'developer_key',
  athlete: { full_name: 'Sample Athlete', name: 'Sample Athlete' },
  has_developer_key: true,
  has_account_credentials: false,
  pending_authorization: false,
  network_route: 'auto',
  exit_node: null,
  sync_days: 30,
  auto_sync: true,
  sync_interval_minutes: 360,
  last_sync_at: '2026-10-30T06:12:00Z',
}

/** `/sync/status`. */
const SYNC_STATUS = {
  running: false,
  started_at: null,
  message: null,
  last_sync_at: '2026-10-30T06:12:00Z',
  connected: true,
  method: 'developer_key',
}

/** `/stats/summary`. */
const SUMMARY = {
  totals: {
    label: 'All time',
    activity_count: 186,
    distance_m: 1_234_000,
    elevation_gain_m: 8200,
    moving_time_s: 384_000,
    moving_time_label: null,
    avg_pace_s_per_km: 318,
    avg_pace_label: null,
    avg_hr: 149,
    calories: 70_400,
  },
  this_year: {
    label: 'This year',
    activity_count: 52,
    distance_m: 380_000,
    elevation_gain_m: 2400,
    moving_time_s: 121_000,
    moving_time_label: null,
    avg_pace_s_per_km: 312,
    avg_pace_label: null,
    avg_hr: 148,
    calories: 22_200,
  },
  this_month: {
    label: 'This month',
    activity_count: 9,
    distance_m: 62_000,
    elevation_gain_m: 380,
    moving_time_s: 19_800,
    moving_time_label: null,
    avg_pace_s_per_km: 310,
    avg_pace_label: null,
    avg_hr: 147,
    calories: 3_600,
  },
  this_week: {
    label: 'This week',
    activity_count: WEEK.runs,
    distance_m: WEEK.distance_m,
    elevation_gain_m: 140,
    moving_time_s: 9_720,
    moving_time_label: null,
    avg_pace_s_per_km: 313,
    avg_pace_label: null,
    avg_hr: 150,
    calories: 1_780,
  },
  personal_records: [],
  streaks: {
    current_weeks: 6,
    longest_weeks: 11,
    total_active_weeks: 48,
    current_week_key: '2026-W44',
  },
  sport_breakdown: [{ sport_type: 'running', activity_count: 186, distance_m: 1_234_000, moving_time_s: 384_000 }],
  recent: ACTIVITIES.slice(0, 4),
}

/** `/insights/chips`. */
const CHIPS = {
  items: Object.fromEntries(
    ACTIVITIES.map((run) => [
      String(run.id),
      // The long run is the longest of the last 30 days, so it is the one
      // run with something to say about it.
      run.id === 903
        ? { chip: 'Longest in 30 days', tone: 'good' }
        : { chip: 'Usual run', tone: 'neutral' },
    ]),
  ),
}

/** Every fixture, by route path (without the query string). */
export const FIXTURES = {
  '/api/v1/auth/status': AUTH_STATUS,
  '/api/v1/sync/status': SYNC_STATUS,
  '/api/v1/stats/summary': SUMMARY,
  '/api/v1/insights/today': TODAY_INSIGHT,
  '/api/v1/insights/progress': PROGRESS,
  '/api/v1/insights/chips': CHIPS,
  '/api/v1/physiology/sources': SOURCES,
  '/api/v1/physiology/latest': LATEST,
  '/api/v1/physiology/sleep': SLEEP,
  '/api/v1/physiology/hrv': HRV,
  '/api/v1/physiology/readiness': READINESS,
  '/api/v1/activities': {
    items: ACTIVITIES,
    total: ACTIVITIES.length,
    page: 1,
    per_page: 12,
    pages: 1,
  },
}

/**
 * The answer to one bridge call.
 *
 * @param {string} method the HTTP method
 * @param {string} path the path, with the query string
 * @returns {{ok: boolean, status: number, body?: unknown, error?: {detail: string}}}
 */
export function answerFor(method, path) {
  const clean = path.split('?')[0]
  const body = FIXTURES[clean]
  if (body === undefined) {
    return { ok: false, status: 404, error: { detail: `no fixture for ${clean}` } }
  }
  return { ok: true, status: 200, body }
}
