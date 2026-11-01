/**
 * The synthetic fixture set.
 *
 * Every screen renders against this without a running backend, and the
 * tests assert against it. The values are invented: no real routes, no
 * names, no notes, no coordinates that mean anything.
 *
 * The sentences here are the core's, written out. Since core-contract §8
 * the domain's sentences arrive already rendered — the direction word, the
 * arithmetic, the travel, the Rest line and the pattern sentences — so the
 * fixture is a captured answer rather than a computation. The numbers under
 * them are the history the charts draw.
 *
 * The wording is the core's wording, from `crates/stride-core/strings.json`:
 *
 *   "6 h 10 m" · "could be better" · "1 h 10 m less than your usual"
 *   "6th night down" · "2 of 3 signals below your usual"
 *   "your HRV has been below your usual for 3 nights"
 */

import { round_pace_s_per_km } from '../lib/units'
import type {
  ActivityDetail,
  ActivityListResponse,
  ActivitySummary,
  CompareCandidate,
  CompareResponse,
  HealthDaily,
  PlanSession,
  PlanWeek,
  ProgressInsight,
  SyncStatus,
  TodayInsight,
  VitalsSnapshot,
} from '../lib/types'

/** The day the fixture set is "now". */
export const TODAY = '2026-05-12'

/**
 * The synthetic nights.
 *
 * The values are written out rather than generated, because the readout has
 * to land on the numbers the plan describes: a usual of 6 h 40 m of sleep,
 * a usual HRV of 62 ms, a usual resting heart rate of 58 bpm — and because
 * the band (median ± 1.5 × MAD) has to put today's value outside the band
 * for sleep and HRV, and inside it for the resting heart rate.
 */
const SLEEP_NIGHTS = [
  26400, 27600, 25200, 28800, 26400, 24000, 27600, 25200, 30000, 26400, 24000, 27600, 22800,
  28800, 26400, 25200, 24000, 30000, 26400, 25200, 28800, 21600, 24000, 31200, 26400, 25200,
  24600, 23800, 23000,
]
const HRV_NIGHTS = [
  62, 66, 58, 70, 62, 54, 66, 58, 74, 62, 54, 66, 50, 70, 62, 58, 54, 74, 62, 58, 66, 46, 54,
  78, 62, 58, 58, 54, 50,
]
const RHR_NIGHTS = [
  58, 60, 56, 62, 58, 54, 60, 56, 52, 58, 54, 60, 52, 62, 58, 56, 54, 52, 58, 60, 54, 52, 50,
  66, 58, 62, 56, 52, 63,
]

/** The nights, oldest first. The last one is the day itself. */
function buildNights(): {
  date: string
  sleep_s: number
  hrv_ms: number
  resting_hr: number
}[] {
  const nights: { date: string; sleep_s: number; hrv_ms: number; resting_hr: number }[] = []
  const start = new Date(Date.UTC(2026, 3, 13))
  for (let index = 0; index < 29; index += 1) {
    const date = new Date(start.getTime() + index * 86_400_000).toISOString().slice(0, 10)
    nights.push({
      date,
      sleep_s: SLEEP_NIGHTS[index] as number,
      hrv_ms: HRV_NIGHTS[index] as number,
      resting_hr: RHR_NIGHTS[index] as number,
    })
  }
  // The day itself: the hours came down, the HRV came down, the resting
  // heart rate held where it was.
  const date = new Date(start.getTime() + 29 * 86_400_000).toISOString().slice(0, 10)
  nights.push({ date, sleep_s: 22_200, hrv_ms: 48, resting_hr: 63 })
  return nights
}

const nights = buildNights()

/** The previous nights, the day itself excluded — the band's window. */
const history = nights.slice(0, 29)

/**
 * The vitals — the snapshot Today and Body both read.
 *
 * The bands are the median ± 1.5 × MAD of the nights before the day
 * itself: sleep 26 400 ± 3 600 s, HRV 62 ± 6 ms, resting heart rate
 * 58 ± 6 bpm. The sentences are the core's answers for those numbers.
 */
export const vitalsFixture: VitalsSnapshot = {
  date: TODAY,
  generated_at: '2026-05-12T06:30:00Z',
  rest: {
    label: 'Rest',
    line: 'could be better · 2 of 3 signals below your usual',
  },
  vitals: [
    {
      key: 'sleep',
      label: 'Sleep',
      value_text: '6 h 10 m',
      direction_text: 'could be better',
      deviation_text: '1 h 10 m less than your usual',
      travel_text: '6th night down',
      status_text: null,
      band: { lower: 22_800, upper: 30_000, nights: 29 },
    },
    {
      key: 'hrv',
      label: 'HRV',
      value_text: '48 ms',
      direction_text: 'could be better',
      deviation_text: '14 below your usual',
      travel_text: '3rd night down',
      status_text: null,
      band: { lower: 56, upper: 68, nights: 29 },
    },
    {
      key: 'resting_hr',
      label: 'Resting HR',
      value_text: '63 bpm',
      direction_text: 'about usual',
      deviation_text: '5 above your usual',
      travel_text: null,
      status_text: null,
      band: { lower: 52, upper: 64, nights: 29 },
    },
  ],
  history: {
    sleep: history.map((night) => ({ date: night.date, value: night.sleep_s })),
    hrv: history.map((night) => ({ date: night.date, value: night.hrv_ms })),
    resting_hr: history.map((night) => ({ date: night.date, value: night.resting_hr })),
  },
  patterns: [
    {
      id: 'hrv-below',
      text: 'your HRV has been below your usual for 3 nights',
      sample_size: 3,
    },
    {
      id: 'sleep-low-distance',
      text: 'the 5 other times your sleep was below your usual, you ran 8 km or less',
      sample_size: 5,
    },
  ],
}

// ------------------------------------------------------------------ plan

const planSessions: PlanSession[] = [
  {
    date: '2026-05-11',
    session_type: 'easy',
    title: 'Easy 8',
    sentence: '11 May 2026 — 8 km · easy',
    target_distance_m: 8_000,
    actual_distance_m: 8_100,
    status: 'done',
    source: 'prescribed',
  },
  {
    date: '2026-05-12',
    session_type: 'easy',
    title: 'Easy 10',
    sentence: 'Today — 10 km · easy',
    target_distance_m: 10_000,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-13',
    session_type: 'rest',
    title: null,
    sentence: '13 May 2026 — rest',
    target_distance_m: null,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-14',
    session_type: 'easy',
    title: 'Easy 5',
    sentence: '14 May 2026 — 5 km · easy',
    target_distance_m: 5_000,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-15',
    session_type: 'long',
    title: 'Long 16',
    sentence: '15 May 2026 — 16 km · long',
    target_distance_m: 16_000,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-16',
    session_type: 'easy',
    title: 'Easy 30 minutes',
    sentence: '16 May 2026 — easy',
    target_distance_m: null,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-17',
    session_type: 'easy',
    title: 'Easy 6',
    sentence: '17 May 2026 — 6 km · easy',
    target_distance_m: 6_000,
    actual_distance_m: null,
    status: 'planned',
    source: 'prescribed',
  },
]

/** The plan's week. */
export const planWeekFixture: PlanWeek = {
  block_name: 'Base block',
  source: 'prescribed',
  week_start: '2026-05-11',
  week_end: '2026-05-17',
  target_km: 25,
  sessions: planSessions,
  adherence: { done: 1, planned: 7 },
}

// -------------------------------------------------------------- activities

function activity(
  id: number,
  date: string,
  distance_m: number,
  moving_time_s: number,
  story: string,
  extra: Partial<ActivitySummary> = {},
): ActivitySummary {
  const elapsed = moving_time_s + 60
  return {
    id,
    name: 'Run',
    sport_type: 'running',
    start_time: `${date}T07:10:00Z`,
    local_date: date,
    distance_m,
    moving_time_s,
    elapsed_time_s: elapsed,
    avg_hr: 148,
    max_hr: 171,
    avg_cadence: 178,
    elevation_gain_m: 42,
    pace_s_per_km: round_pace_s_per_km(moving_time_s / (distance_m / 1000)),
    story,
    geo_points: [],
    ...extra,
  }
}

export const activityFixtures: ActivitySummary[] = [
  activity(101, '2026-05-10', 12_400, 4_020, '12.4 km, 2.4 km more than your usual Wednesday'),
  activity(102, '2026-05-09', 10_200, 3_320, '10.2 km, about your usual Saturday'),
  activity(103, '2026-05-07', 8_400, 2_520, '8.4 km, a bit shorter than your usual Wednesday'),
  activity(104, '2026-05-06', 5_000, 1_620, '5 km, your shortest run in three weeks'),
  activity(105, '2026-05-04', 16_100, 5_080, '16.1 km, your longest run this month'),
  activity(106, '2026-05-03', 6_300, 2_080, '6.3 km, about your usual Monday'),
]

export const activitiesFixture: ActivityListResponse = {
  activities: activityFixtures,
  next_cursor: null,
  generated_at: '2026-05-12T06:30:00Z',
}

export const activityDetailFixture: ActivityDetail = {
  ...(activityFixtures[0] as ActivitySummary),
  splits: [
    { index: 1, distance_m: 1_000, duration_s: 322, avg_hr: 149 },
    { index: 2, distance_m: 1_000, duration_s: 318, avg_hr: 151 },
    { index: 3, distance_m: 1_000, duration_s: 315, avg_hr: 152 },
    { index: 4, distance_m: 1_000, duration_s: 320, avg_hr: 153 },
    { index: 5, distance_m: 1_000, duration_s: 326, avg_hr: 154 },
    { index: 6, distance_m: 1_000, duration_s: 330, avg_hr: 155 },
    { index: 7, distance_m: 1_000, duration_s: 334, avg_hr: 156 },
    { index: 8, distance_m: 1_000, duration_s: 336, avg_hr: 158 },
    { index: 9, distance_m: 1_000, duration_s: 335, avg_hr: 157 },
    { index: 10, distance_m: 1_000, duration_s: 337, avg_hr: 158 },
    { index: 11, distance_m: 1_000, duration_s: 341, avg_hr: 159 },
    { index: 12, distance_m: 1_000, duration_s: 338, avg_hr: 158 },
    { index: 13, distance_m: 400, duration_s: 138, avg_hr: 156 },
  ],
  elevation: Array.from({ length: 26 }, (_, index) => ({
    distance_m: index * 500,
    elevation_m: 40 + Math.round(20 * Math.sin(index / 3)),
  })),
  samples: Array.from({ length: 60 }, (_, index) => ({
    offset_s: index * 60,
    pace_s_per_km: 300 + Math.round(20 * Math.sin(index / 6)),
    hr: 140 + Math.round(18 * Math.sin(index / 9)),
  })),
  plan_line: { actual_distance_m: 12_400, target_distance_m: 10_000 },
  note: null,
}

// ----------------------------------------------------------------- today

function todayInsight(
  lastRun: ActivitySummary,
  distanceComparison: string | null,
): TodayInsight {
  return {
    generated_at: '2026-05-12T06:30:00Z',
    date: TODAY,
    planned: planSessions[1] as PlanSession,
    options: [planSessions[3] as PlanSession, planSessions[5] as PlanSession],
    last_run: lastRun,
    distance_comparison: distanceComparison,
    week: { distance_m: 20_600, sessions: 2 },
  }
}

/**
 * Today, with a last run inside 10% of the target.
 *
 * The distance comparison is absent here, and that is the rule: 10.2 km
 * against a 10 km target is not news.
 */
export const todayFixture: TodayInsight = todayInsight(
  activityFixtures[1] as ActivitySummary,
  null,
)

/**
 * Today, with a last run 24% beyond the target.
 *
 * The distance comparison appears here, stated neutrally.
 */
export const todayOverFixture: TodayInsight = todayInsight(
  activityFixtures[0] as ActivitySummary,
  '2.4 km more than the 10 km target',
)

// --------------------------------------------------------------- progress

export const progressFixture: ProgressInsight = {
  generated_at: '2026-05-12T06:30:00Z',
  headline: 'fitter than 8 weeks ago',
  headline_note: 'same heart rate, 24 s/km faster',
  weekly_volume: [
    { week_start: '2026-03-16', distance_m: 24_100 },
    { week_start: '2026-03-23', distance_m: 27_400 },
    { week_start: '2026-03-30', distance_m: 21_800 },
    { week_start: '2026-04-06', distance_m: 31_200 },
    { week_start: '2026-04-13', distance_m: 28_900 },
    { week_start: '2026-04-20', distance_m: 35_400 },
    { week_start: '2026-04-27', distance_m: 30_100 },
    { week_start: '2026-05-04', distance_m: 38_600 },
    { week_start: '2026-05-11', distance_m: 20_600 },
  ],
  typical_week_m: 30_000,
  pace_at_hr: [
    { week_start: '2026-03-16', pace_s_per_km: 336, hr: 150 },
    { week_start: '2026-03-30', pace_s_per_km: 330, hr: 150 },
    { week_start: '2026-04-13', pace_s_per_km: 324, hr: 150 },
    { week_start: '2026-04-27', pace_s_per_km: 318, hr: 150 },
    { week_start: '2026-05-11', pace_s_per_km: 312, hr: 150 },
  ],
  bests: [
    { key: 'progress.best.5k', value: '19:48', date: '2026-04-27' },
    { key: 'progress.best.10k', value: '41:12', date: '2026-05-04' },
    { key: 'progress.best.longest', value: '16.1 km', date: '2026-05-04' },
  ],
  consistency: [
    { week_start: '2026-03-16', sessions: 3 },
    { week_start: '2026-03-23', sessions: 4 },
    { week_start: '2026-03-30', sessions: 2 },
    { week_start: '2026-04-06', sessions: 4 },
    { week_start: '2026-04-13', sessions: 3 },
    { week_start: '2026-04-20', sessions: 5 },
    { week_start: '2026-04-27', sessions: 4 },
    { week_start: '2026-05-11', sessions: 2 },
  ],
  adherence: { done: 7, planned: 8 },
}

// --------------------------------------------------------------- compare

function series(distance_m: number, moving_time_s: number, base_hr: number) {
  return Array.from({ length: 40 }, (_, index) => ({
    offset_s: index * 60,
    pace_s_per_km: Math.round(moving_time_s / (distance_m / 1000) + 12 * Math.sin(index / 5)),
    hr: base_hr + Math.round(14 * Math.sin(index / 7)),
  }))
}

export const compareFixture: CompareResponse = {
  basis_text: 'Same distance, 3 weeks apart',
  metrics: [
    {
      key: 'distance',
      label_key: 'vital.distance',
      format: 'distance',
      a: 10_400,
      b: 10_600,
      delta: 200,
    },
    {
      key: 'time',
      label_key: 'vital.time',
      format: 'duration',
      a: 3_300,
      b: 3_240,
      delta: -60,
    },
    { key: 'pace', label_key: 'vital.pace', format: 'pace', a: 317, b: 306, delta: -11 },
    { key: 'hr', label_key: 'vital.heartRate', format: 'hr', a: 149, b: 146, delta: -3 },
  ],
  a: {
    id: 102,
    label: 'Run',
    date: '2026-04-19',
    distance_m: 10_400,
    story: '10.4 km, about your usual Saturday',
  },
  b: {
    id: 101,
    label: 'Run',
    date: '2026-05-10',
    distance_m: 10_600,
    story: '10.6 km, a little more than your usual Saturday',
  },
  series: {
    a: series(10_400, 3_300, 149),
    b: series(10_600, 3_240, 146),
  },
}

export const compareCandidatesFixture: CompareCandidate[] = activityFixtures.map((entry) => ({
  id: entry.id,
  label: entry.name,
  date: entry.local_date,
  distance_m: entry.distance_m,
}))

// ------------------------------------------------------------------ sync

export const syncStatusFixture: SyncStatus = {
  // Two hours ago, so the rig shows a fresh sync rather than a stale one.
  last_sync_at: new Date(Date.now() - 2 * 3_600_000).toISOString(),
  last_error: null,
  activity_count: 42,
}

export const healthDailyFixture: HealthDaily = {
  date: TODAY,
  generated_at: '2026-05-12T06:30:00Z',
  stale: false,
}

// ------------------------------------------------------------- dispatcher

/**
 * Answer a route from the fixture set.
 *
 * `undefined` means "this route has no fixture", which the client turns
 * into a typed not-found — the same shape a real router would answer.
 */
export function fixtureResponse(
  path: string,
  params: Record<string, string> = {},
  variant = '',
): unknown {
  switch (path) {
    case '/insights/today':
      return variant === 'over' ? todayOverFixture : todayFixture
    case '/physiology/vitals':
      return vitalsFixture
    case '/activities':
      return activitiesFixture
    case '/activities/101':
      return activityDetailFixture
    case '/plan/week':
      return planWeekFixture
    case '/insights/progress':
      return progressFixture
    case '/compare': {
      // Swapping A and B flips every delta's sign. The fixture's A is
      // the older run; asking for the newer one as A reverses the rows.
      const reversed = params['a'] === '101'
      if (!reversed) return compareFixture
      return {
        ...compareFixture,
        a: compareFixture.b,
        b: compareFixture.a,
        series: { a: compareFixture.series.b, b: compareFixture.series.a },
        metrics: compareFixture.metrics.map((metric) => ({
          ...metric,
          a: metric.b,
          b: metric.a,
          delta: metric.delta === null ? null : -metric.delta,
        })),
      }
    }
    case '/compare/candidates':
      return compareCandidatesFixture
    case '/sync/status':
      return syncStatusFixture
    case '/health/daily':
      return healthDailyFixture
    default:
      return undefined
  }
}
