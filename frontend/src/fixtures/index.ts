/**
 * The synthetic fixture set.
 *
 * Every screen renders against this without a running backend, and the
 * tests assert against it. The values are invented: no real routes, no
 * names, no notes, no coordinates that mean anything.
 *
 * The sentences here are the core's, written out. Since core-contract §8
 * the domain's sentences arrive already rendered — the direction word, the
 * arithmetic, the travel, the Rest line and the pattern sentences — and
 * since the values joined them, the numbers arrive rendered too: "8.4 km",
 * "42 m", "148 bpm", "5:24 /km".
 *
 * **The fixture is a captured answer, not a computation.** Every rendered
 * value below is written out, in the core's words, because a fixture that
 * computed its own values would be a second implementation of the core's
 * rules — the very thing the core owns.
 *
 * The wording is the core's wording, from `crates/stride-core/strings.json`:
 *
 *   "6 h 10 m" · "could be better" · "1 h 10 m less than your usual"
 *   "6th night down" · "2 of 3 signals below your usual"
 *   "your HRV has been below your usual for 3 nights"
 */

import type {
  ActivityDetail,
  CompareSeriesPoint,
  ActivityListResponse,
  ActivitySummary,
  CompareCandidate,
  CompareResponse,
  ConnectionStatus,
  FitImportResult,
  Health,
  HealthDaily,
  KeyConnectResult,
  KeyTestResult,
  NoteResult,
  PlanImportResult,
  PlanSession,
  PlanWeek,
  ProgressInsight,
  RenderedValue,
  SettingsDeleteResult,
  SettingsExportResult,
  SettingsState,
  SyncStatus,
  TodayInsight,
  VitalsSnapshot,
} from '../lib/types'
import type { ApiErrorKind } from '../lib/transport'

/**
 * A failure the fixture set answers with.
 *
 * The router answers a business failure with a typed one — `no_plan`,
 * `stale_data`, `not_enough_history` — and the screen reads the kind
 * rather than a generic error. The fixtures answer the same way, so the
 * states can be rendered and tested without a running backend.
 */
export class FixtureFailure extends Error {
  readonly kind: ApiErrorKind
  readonly status: number
  readonly detail: string

  constructor(kind: ApiErrorKind, detail: string, status: number) {
    super(detail)
    this.name = 'FixtureFailure'
    this.kind = kind
    this.status = status
    this.detail = detail
  }
}

/** The day the fixture set is "now". */
export const TODAY = '2026-05-13'

/** A rendered value: the core's words, and the number a chart draws. */
function value(text: string, raw: number | null): RenderedValue {
  return { text, raw }
}

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
  generated_at: '2026-05-13T06:30:00Z',
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
    target: value('8 km', 8_000),
    actual: value('8.1 km', 8_100),
    status: 'done',
    source: 'prescribed',
  },
  {
    // The Monday before the 12th: planned, and the day went by without
    // it. **Not done** is the plain fact, and it is one of the three
    // counts the week shows.
    date: '2026-05-12',
    session_type: 'easy',
    title: 'Easy 10',
    sentence: '12 May 2026 — 10 km · easy',
    target: value('10 km', 10_000),
    actual: null,
    status: 'not_done',
    source: 'prescribed',
  },
  {
    date: '2026-05-13',
    session_type: 'easy',
    title: 'Easy 10',
    sentence: 'Today — 10 km · easy',
    target: value('10 km', 10_000),
    actual: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-14',
    session_type: 'easy',
    title: 'Easy 5',
    sentence: '14 May 2026 — 5 km · easy',
    target: value('5 km', 5_000),
    actual: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-15',
    session_type: 'long',
    title: 'Long 16',
    sentence: '15 May 2026 — 16 km · long',
    target: value('16 km', 16_000),
    actual: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-16',
    session_type: 'easy',
    title: 'Easy 30 minutes',
    sentence: '16 May 2026 — easy',
    target: null,
    actual: null,
    status: 'planned',
    source: 'prescribed',
  },
  {
    date: '2026-05-17',
    session_type: 'easy',
    title: 'Easy 6',
    sentence: '17 May 2026 — 6 km · easy',
    target: value('6 km', 6_000),
    actual: null,
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
  /**
   * The athlete's day, as the api counts it.
   *
   * The app highlights it inside the week, and the app does not keep its
   * own clock for it: the api's day is the athlete's day, with their
   * timezone in it.
   */
  today: TODAY,
  target: value('25 km', 25_000),
  sessions: planSessions,
  adherence: { done: 1, planned: 7 },
  counts: { done: 1, planned: 5, not_done: 1 },
}

/**
 * The plan's week, inferred from the athlete's own history.
 *
 * A baseline is what the athlete usually does — it is not a prescription,
 * and it never produces a "not done". The days that were not run are
 * "planned", because nothing was prescribed.
 */
export const planWeekBaselineFixture: PlanWeek = {
  ...planWeekFixture,
  block_name: 'Base block',
  source: 'baseline',
  sessions: planSessions.map((session) => ({
    ...session,
    status: session.status === 'done' ? ('done' as const) : ('planned' as const),
    source: 'baseline' as const,
  })),
  counts: { done: 1, planned: 6, not_done: 0 },
}

// -------------------------------------------------------------- activities

/**
 * The feed's groups.
 *
 * The API groups the feed — it is the API that knows the totals — and the
 * app labels the groups. The key is the date the group starts.
 */
const THIS_WEEK = {
  key: '2026-05-06',
  distance: value('36 km', 36_000),
  duration: value('3 h 11 m', 11_480),
}
const LONG_RUN_DAY = {
  key: '2026-05-04',
  distance: value('16.1 km', 16_100),
  duration: value('1 h 25 m', 5_072),
}
const MONDAY = {
  key: '2026-05-03',
  distance: value('6.3 km', 6_300),
  duration: value('35 m', 2_080),
}

function activity(
  id: number,
  date: string,
  distance: RenderedValue,
  duration: RenderedValue,
  pace: RenderedValue,
  story: string,
  group: typeof THIS_WEEK,
): ActivitySummary {
  return {
    id,
    name: 'Run',
    sport_type: 'running',
    start_time: `${date}T07:10:00Z`,
    local_date: date,
    distance,
    duration,
    heart_rate: value('148 bpm', 148),
    pace,
    elevation_gain: value('42 m', 42),
    group,
    story,
    geo_points: [],
  }
}

export const activityFixtures: ActivitySummary[] = [
  activity(
    101,
    '2026-05-10',
    value('12.4 km', 12_400),
    value('1 h 7 m', 4_020),
    value('5:24 /km', 324.19),
    '12.4 km, 2.4 km more than your usual Wednesday',
    THIS_WEEK,
  ),
  activity(
    102,
    '2026-05-09',
    value('10.2 km', 10_200),
    value('55 m', 3_320),
    value('5:25 /km', 325.49),
    '10.2 km, about your usual Saturday',
    THIS_WEEK,
  ),
  activity(
    103,
    '2026-05-07',
    value('8.4 km', 8_400),
    value('42 m', 2_520),
    value('5:00 /km', 300),
    '8.4 km, a bit shorter than your usual Wednesday',
    THIS_WEEK,
  ),
  activity(
    104,
    '2026-05-06',
    value('5 km', 5_000),
    value('27 m', 1_620),
    value('5:24 /km', 324),
    '5 km, your shortest run in three weeks',
    THIS_WEEK,
  ),
  activity(
    105,
    '2026-05-04',
    value('16.1 km', 16_100),
    value('1 h 25 m', 5_072),
    value('5:15 /km', 315.03),
    '16.1 km, your longest run this month',
    LONG_RUN_DAY,
  ),
  activity(
    106,
    '2026-05-03',
    value('6.3 km', 6_300),
    value('35 m', 2_080),
    value('5:30 /km', 330.16),
    '6.3 km, about your usual Monday',
    MONDAY,
  ),
]

export const activitiesFixture: ActivityListResponse = {
  activities: activityFixtures,
  next_cursor: null,
  generated_at: '2026-05-13T06:30:00Z',
}

export const activityDetailFixture: ActivityDetail = {
  ...(activityFixtures[0] as ActivitySummary),
  splits: [
    { index: 1, distance: value('1 km', 1_000), duration: value('5 m', 322), heart_rate: value('149 bpm', 149) },
    { index: 2, distance: value('1 km', 1_000), duration: value('5 m', 318), heart_rate: value('151 bpm', 151) },
    { index: 3, distance: value('1 km', 1_000), duration: value('5 m', 315), heart_rate: value('152 bpm', 152) },
    { index: 4, distance: value('1 km', 1_000), duration: value('5 m', 320), heart_rate: value('153 bpm', 153) },
    { index: 5, distance: value('1 km', 1_000), duration: value('5 m', 326), heart_rate: value('154 bpm', 154) },
    { index: 6, distance: value('1 km', 1_000), duration: value('5 m', 330), heart_rate: value('155 bpm', 155) },
    { index: 7, distance: value('1 km', 1_000), duration: value('5 m', 334), heart_rate: value('156 bpm', 156) },
    { index: 8, distance: value('1 km', 1_000), duration: value('5 m', 336), heart_rate: value('158 bpm', 158) },
    { index: 9, distance: value('1 km', 1_000), duration: value('5 m', 335), heart_rate: value('157 bpm', 157) },
    { index: 10, distance: value('1 km', 1_000), duration: value('5 m', 337), heart_rate: value('158 bpm', 158) },
    { index: 11, distance: value('1 km', 1_000), duration: value('5 m', 341), heart_rate: value('159 bpm', 159) },
    { index: 12, distance: value('1 km', 1_000), duration: value('5 m', 338), heart_rate: value('158 bpm', 158) },
    { index: 13, distance: value('400 m', 400), duration: value('2 m', 138), heart_rate: value('156 bpm', 156) },
  ],
  elevation: [
    { distance: value('0 km', 0), elevation: value('40 m', 40) },
    { distance: value('1.5 km', 1_500), elevation: value('51 m', 51) },
    { distance: value('3 km', 3_000), elevation: value('40 m', 40) },
    { distance: value('4.5 km', 4_500), elevation: value('29 m', 29) },
    { distance: value('6 km', 6_000), elevation: value('40 m', 40) },
    { distance: value('7.5 km', 7_500), elevation: value('51 m', 51) },
    { distance: value('9 km', 9_000), elevation: value('40 m', 40) },
    { distance: value('10.5 km', 10_500), elevation: value('29 m', 29) },
    { distance: value('12 km', 12_000), elevation: value('40 m', 40) },
  ],
  samples: [
    { offset_s: 0, pace: value('5:24 /km', 324), heart_rate: value('140 bpm', 140) },
    { offset_s: 60, pace: value('5:13 /km', 313), heart_rate: value('154 bpm', 154) },
    { offset_s: 120, pace: value('5:04 /km', 304), heart_rate: value('156 bpm', 156) },
    { offset_s: 180, pace: value('5:00 /km', 300), heart_rate: value('152 bpm', 152) },
    { offset_s: 240, pace: value('5:05 /km', 305), heart_rate: value('147 bpm', 147) },
    { offset_s: 300, pace: value('5:15 /km', 315), heart_rate: value('144 bpm', 144) },
    { offset_s: 360, pace: value('5:26 /km', 326), heart_rate: value('141 bpm', 141) },
    { offset_s: 420, pace: value('5:36 /km', 336), heart_rate: value('139 bpm', 139) },
  ],
  plan_line: { text: '12.4 km of a 10 km target' },
  note: null,
}

// ----------------------------------------------------------------- today

function todayInsight(
  lastRun: ActivitySummary,
  distanceComparison: string | null,
): TodayInsight {
  return {
    generated_at: '2026-05-13T06:30:00Z',
    date: TODAY,
    planned: planSessions[2] as PlanSession,
    options: [planSessions[3] as PlanSession, planSessions[5] as PlanSession],
    last_run: lastRun,
    distance_comparison: distanceComparison,
    week: { distance: value('20.6 km', 20_600), sessions: 2 },
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
  generated_at: '2026-05-13T06:30:00Z',
  headline: 'fitter than 8 weeks ago',
  headline_note: 'same heart rate, 24 s/km faster',
  weekly_volume: [
    { week_start: '2026-03-16', distance: value('24.1 km', 24_100) },
    { week_start: '2026-03-23', distance: value('27.4 km', 27_400) },
    { week_start: '2026-03-30', distance: value('21.8 km', 21_800) },
    { week_start: '2026-04-06', distance: value('31.2 km', 31_200) },
    { week_start: '2026-04-13', distance: value('28.9 km', 28_900) },
    { week_start: '2026-04-20', distance: value('35.4 km', 35_400) },
    { week_start: '2026-04-27', distance: value('30.1 km', 30_100) },
    { week_start: '2026-05-04', distance: value('38.6 km', 38_600) },
    { week_start: '2026-05-11', distance: value('20.6 km', 20_600) },
  ],
  typical_week: value('30 km', 30_000),
  pace_at_hr: [
    {
      week_start: '2026-03-16',
      pace: value('5:36 /km', 336),
      heart_rate: value('150 bpm', 150),
    },
    {
      week_start: '2026-03-30',
      pace: value('5:30 /km', 330),
      heart_rate: value('150 bpm', 150),
    },
    {
      week_start: '2026-04-13',
      pace: value('5:24 /km', 324),
      heart_rate: value('150 bpm', 150),
    },
    {
      week_start: '2026-04-27',
      pace: value('5:18 /km', 318),
      heart_rate: value('150 bpm', 150),
    },
    {
      week_start: '2026-05-11',
      pace: value('5:12 /km', 312),
      heart_rate: value('150 bpm', 150),
    },
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
  sessions_per_week: value('3.4', 3.4),
  adherence: { done: 7, planned: 8 },
}

// --------------------------------------------------------------- compare

/**
 * The two effort series, written out.
 *
 * A point carries the core's words beside the number: the text for the
 * tooltip, the raw for the line. They are written out rather than
 * computed, because a fixture that computed its own values would be a
 * second implementation of the core's rules.
 */
const SERIES_A: CompareSeriesPoint[] = [
    { offset_s: 0, pace: { text: '5:17 /km', raw: 317 }, heart_rate: { text: '149 bpm', raw: 149 } },
    { offset_s: 60, pace: { text: '5:19 /km', raw: 319 }, heart_rate: { text: '151 bpm', raw: 151 } },
    { offset_s: 120, pace: { text: '5:22 /km', raw: 322 }, heart_rate: { text: '153 bpm', raw: 153 } },
    { offset_s: 180, pace: { text: '5:24 /km', raw: 324 }, heart_rate: { text: '155 bpm', raw: 155 } },
    { offset_s: 240, pace: { text: '5:26 /km', raw: 326 }, heart_rate: { text: '157 bpm', raw: 157 } },
    { offset_s: 300, pace: { text: '5:27 /km', raw: 327 }, heart_rate: { text: '158 bpm', raw: 158 } },
    { offset_s: 360, pace: { text: '5:28 /km', raw: 328 }, heart_rate: { text: '160 bpm', raw: 160 } },
    { offset_s: 420, pace: { text: '5:29 /km', raw: 329 }, heart_rate: { text: '161 bpm', raw: 161 } },
    { offset_s: 480, pace: { text: '5:29 /km', raw: 329 }, heart_rate: { text: '162 bpm', raw: 162 } },
    { offset_s: 540, pace: { text: '5:29 /km', raw: 329 }, heart_rate: { text: '162 bpm', raw: 162 } },
    { offset_s: 600, pace: { text: '5:28 /km', raw: 328 }, heart_rate: { text: '163 bpm', raw: 163 } },
    { offset_s: 660, pace: { text: '5:27 /km', raw: 327 }, heart_rate: { text: '163 bpm', raw: 163 } },
    { offset_s: 720, pace: { text: '5:25 /km', raw: 325 }, heart_rate: { text: '163 bpm', raw: 163 } },
    { offset_s: 780, pace: { text: '5:23 /km', raw: 323 }, heart_rate: { text: '162 bpm', raw: 162 } },
    { offset_s: 840, pace: { text: '5:21 /km', raw: 321 }, heart_rate: { text: '162 bpm', raw: 162 } },
    { offset_s: 900, pace: { text: '5:19 /km', raw: 319 }, heart_rate: { text: '161 bpm', raw: 161 } },
    { offset_s: 960, pace: { text: '5:16 /km', raw: 316 }, heart_rate: { text: '160 bpm', raw: 160 } },
    { offset_s: 1020, pace: { text: '5:14 /km', raw: 314 }, heart_rate: { text: '158 bpm', raw: 158 } },
    { offset_s: 1080, pace: { text: '5:12 /km', raw: 312 }, heart_rate: { text: '157 bpm', raw: 157 } },
    { offset_s: 1140, pace: { text: '5:10 /km', raw: 310 }, heart_rate: { text: '155 bpm', raw: 155 } },
    { offset_s: 1200, pace: { text: '5:08 /km', raw: 308 }, heart_rate: { text: '153 bpm', raw: 153 } },
    { offset_s: 1260, pace: { text: '5:07 /km', raw: 307 }, heart_rate: { text: '151 bpm', raw: 151 } },
    { offset_s: 1320, pace: { text: '5:06 /km', raw: 306 }, heart_rate: { text: '149 bpm', raw: 149 } },
    { offset_s: 1380, pace: { text: '5:05 /km', raw: 305 }, heart_rate: { text: '147 bpm', raw: 147 } },
    { offset_s: 1440, pace: { text: '5:05 /km', raw: 305 }, heart_rate: { text: '145 bpm', raw: 145 } },
    { offset_s: 1500, pace: { text: '5:05 /km', raw: 305 }, heart_rate: { text: '143 bpm', raw: 143 } },
    { offset_s: 1560, pace: { text: '5:06 /km', raw: 306 }, heart_rate: { text: '141 bpm', raw: 141 } },
    { offset_s: 1620, pace: { text: '5:08 /km', raw: 308 }, heart_rate: { text: '140 bpm', raw: 140 } },
    { offset_s: 1680, pace: { text: '5:09 /km', raw: 309 }, heart_rate: { text: '138 bpm', raw: 138 } },
    { offset_s: 1740, pace: { text: '5:11 /km', raw: 311 }, heart_rate: { text: '137 bpm', raw: 137 } },
    { offset_s: 1800, pace: { text: '5:14 /km', raw: 314 }, heart_rate: { text: '136 bpm', raw: 136 } },
    { offset_s: 1860, pace: { text: '5:16 /km', raw: 316 }, heart_rate: { text: '136 bpm', raw: 136 } },
    { offset_s: 1920, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '135 bpm', raw: 135 } },
    { offset_s: 1980, pace: { text: '5:21 /km', raw: 321 }, heart_rate: { text: '135 bpm', raw: 135 } },
    { offset_s: 2040, pace: { text: '5:23 /km', raw: 323 }, heart_rate: { text: '135 bpm', raw: 135 } },
    { offset_s: 2100, pace: { text: '5:25 /km', raw: 325 }, heart_rate: { text: '136 bpm', raw: 136 } },
    { offset_s: 2160, pace: { text: '5:27 /km', raw: 327 }, heart_rate: { text: '136 bpm', raw: 136 } },
    { offset_s: 2220, pace: { text: '5:28 /km', raw: 328 }, heart_rate: { text: '137 bpm', raw: 137 } },
    { offset_s: 2280, pace: { text: '5:29 /km', raw: 329 }, heart_rate: { text: '138 bpm', raw: 138 } },
    { offset_s: 2340, pace: { text: '5:29 /km', raw: 329 }, heart_rate: { text: '140 bpm', raw: 140 } },
]

const SERIES_B: CompareSeriesPoint[] = [
    { offset_s: 0, pace: { text: '5:06 /km', raw: 306 }, heart_rate: { text: '146 bpm', raw: 146 } },
    { offset_s: 60, pace: { text: '5:08 /km', raw: 308 }, heart_rate: { text: '148 bpm', raw: 148 } },
    { offset_s: 120, pace: { text: '5:11 /km', raw: 311 }, heart_rate: { text: '150 bpm', raw: 150 } },
    { offset_s: 180, pace: { text: '5:13 /km', raw: 313 }, heart_rate: { text: '152 bpm', raw: 152 } },
    { offset_s: 240, pace: { text: '5:15 /km', raw: 315 }, heart_rate: { text: '154 bpm', raw: 154 } },
    { offset_s: 300, pace: { text: '5:16 /km', raw: 316 }, heart_rate: { text: '155 bpm', raw: 155 } },
    { offset_s: 360, pace: { text: '5:17 /km', raw: 317 }, heart_rate: { text: '157 bpm', raw: 157 } },
    { offset_s: 420, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '158 bpm', raw: 158 } },
    { offset_s: 480, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '159 bpm', raw: 159 } },
    { offset_s: 540, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '159 bpm', raw: 159 } },
    { offset_s: 600, pace: { text: '5:17 /km', raw: 317 }, heart_rate: { text: '160 bpm', raw: 160 } },
    { offset_s: 660, pace: { text: '5:16 /km', raw: 316 }, heart_rate: { text: '160 bpm', raw: 160 } },
    { offset_s: 720, pace: { text: '5:14 /km', raw: 314 }, heart_rate: { text: '160 bpm', raw: 160 } },
    { offset_s: 780, pace: { text: '5:12 /km', raw: 312 }, heart_rate: { text: '159 bpm', raw: 159 } },
    { offset_s: 840, pace: { text: '5:10 /km', raw: 310 }, heart_rate: { text: '159 bpm', raw: 159 } },
    { offset_s: 900, pace: { text: '5:08 /km', raw: 308 }, heart_rate: { text: '158 bpm', raw: 158 } },
    { offset_s: 960, pace: { text: '5:05 /km', raw: 305 }, heart_rate: { text: '157 bpm', raw: 157 } },
    { offset_s: 1020, pace: { text: '5:03 /km', raw: 303 }, heart_rate: { text: '155 bpm', raw: 155 } },
    { offset_s: 1080, pace: { text: '5:01 /km', raw: 301 }, heart_rate: { text: '154 bpm', raw: 154 } },
    { offset_s: 1140, pace: { text: '4:59 /km', raw: 299 }, heart_rate: { text: '152 bpm', raw: 152 } },
    { offset_s: 1200, pace: { text: '4:57 /km', raw: 297 }, heart_rate: { text: '150 bpm', raw: 150 } },
    { offset_s: 1260, pace: { text: '4:56 /km', raw: 296 }, heart_rate: { text: '148 bpm', raw: 148 } },
    { offset_s: 1320, pace: { text: '4:55 /km', raw: 295 }, heart_rate: { text: '146 bpm', raw: 146 } },
    { offset_s: 1380, pace: { text: '4:54 /km', raw: 294 }, heart_rate: { text: '144 bpm', raw: 144 } },
    { offset_s: 1440, pace: { text: '4:54 /km', raw: 294 }, heart_rate: { text: '142 bpm', raw: 142 } },
    { offset_s: 1500, pace: { text: '4:54 /km', raw: 294 }, heart_rate: { text: '140 bpm', raw: 140 } },
    { offset_s: 1560, pace: { text: '4:55 /km', raw: 295 }, heart_rate: { text: '138 bpm', raw: 138 } },
    { offset_s: 1620, pace: { text: '4:57 /km', raw: 297 }, heart_rate: { text: '137 bpm', raw: 137 } },
    { offset_s: 1680, pace: { text: '4:58 /km', raw: 298 }, heart_rate: { text: '135 bpm', raw: 135 } },
    { offset_s: 1740, pace: { text: '5:00 /km', raw: 300 }, heart_rate: { text: '134 bpm', raw: 134 } },
    { offset_s: 1800, pace: { text: '5:03 /km', raw: 303 }, heart_rate: { text: '133 bpm', raw: 133 } },
    { offset_s: 1860, pace: { text: '5:05 /km', raw: 305 }, heart_rate: { text: '133 bpm', raw: 133 } },
    { offset_s: 1920, pace: { text: '5:07 /km', raw: 307 }, heart_rate: { text: '132 bpm', raw: 132 } },
    { offset_s: 1980, pace: { text: '5:10 /km', raw: 310 }, heart_rate: { text: '132 bpm', raw: 132 } },
    { offset_s: 2040, pace: { text: '5:12 /km', raw: 312 }, heart_rate: { text: '132 bpm', raw: 132 } },
    { offset_s: 2100, pace: { text: '5:14 /km', raw: 314 }, heart_rate: { text: '133 bpm', raw: 133 } },
    { offset_s: 2160, pace: { text: '5:16 /km', raw: 316 }, heart_rate: { text: '133 bpm', raw: 133 } },
    { offset_s: 2220, pace: { text: '5:17 /km', raw: 317 }, heart_rate: { text: '134 bpm', raw: 134 } },
    { offset_s: 2280, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '135 bpm', raw: 135 } },
    { offset_s: 2340, pace: { text: '5:18 /km', raw: 318 }, heart_rate: { text: '137 bpm', raw: 137 } },
]

export const compareFixture: CompareResponse = {
  basis_text: 'Same distance, 3 weeks apart',
  metrics: [
    {
      key: 'distance',
      label_key: 'vital.distance',
      a: value('10.4 km', 10_400),
      b: value('10.6 km', 10_600),
      delta: value('+0.2 km', 200),
    },
    {
      key: 'time',
      label_key: 'vital.time',
      a: value('55 m', 3_300),
      b: value('54 m', 3_240),
      delta: value('-1 m', -60),
    },
    {
      key: 'pace',
      label_key: 'vital.pace',
      a: value('5:17 /km', 317),
      b: value('5:06 /km', 306),
      delta: value('-11 s/km', -11),
    },
    {
      key: 'hr',
      label_key: 'vital.heartRate',
      a: value('149 bpm', 149),
      b: value('146 bpm', 146),
      delta: value('-3 bpm', -3),
    },
  ],
  a: {
    id: 102,
    label: 'Run',
    date: '2026-04-19',
    distance: value('10.4 km', 10_400),
    story: '10.4 km, about your usual Saturday',
  },
  b: {
    id: 101,
    label: 'Run',
    date: '2026-05-10',
    distance: value('10.6 km', 10_600),
    story: '10.6 km, a little more than your usual Saturday',
  },
  series: {
    a: SERIES_A,
    b: SERIES_B,
  },
}

/**
 * The comparison the other way round.
 *
 * Asking for the newer run as A is a different question, and the API
 * answers it — the app does not swap the columns itself: the delta's sign
 * is the core's arithmetic, and the app cannot un-render a word.
 */
export const compareSwappedFixture: CompareResponse = {
  basis_text: 'Same distance, 3 weeks apart',
  metrics: [
    {
      key: 'distance',
      label_key: 'vital.distance',
      a: value('10.6 km', 10_600),
      b: value('10.4 km', 10_400),
      delta: value('-0.2 km', -200),
    },
    {
      key: 'time',
      label_key: 'vital.time',
      a: value('54 m', 3_240),
      b: value('55 m', 3_300),
      delta: value('+1 m', 60),
    },
    {
      key: 'pace',
      label_key: 'vital.pace',
      a: value('5:06 /km', 306),
      b: value('5:17 /km', 317),
      delta: value('+11 s/km', 11),
    },
    {
      key: 'hr',
      label_key: 'vital.heartRate',
      a: value('146 bpm', 146),
      b: value('149 bpm', 149),
      delta: value('+3 bpm', 3),
    },
  ],
  a: {
    id: 101,
    label: 'Run',
    date: '2026-05-10',
    distance: value('10.6 km', 10_600),
    story: '10.6 km, a little more than your usual Saturday',
  },
  b: {
    id: 102,
    label: 'Run',
    date: '2026-04-19',
    distance: value('10.4 km', 10_400),
    story: '10.4 km, about your usual Saturday',
  },
  series: {
    a: SERIES_B,
    b: SERIES_A,
  },
}

export const compareCandidatesFixture: CompareCandidate[] = activityFixtures.map((entry) => ({
  id: entry.id,
  label: entry.name,
  date: entry.local_date,
  distance: entry.distance,
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
  generated_at: '2026-05-13T06:30:00Z',
  stale: false,
}

// ------------------------------------------------------- the app's own facts

/**
 * The connection.
 *
 * **The key is not here and never will be.** The answer carries the last
 * four characters — an identifier the athlete can check against their own
 * key — and nothing else. Four characters cannot be a Garmin Personal API
 * key, so the fixture holds no credential and no fragment of one that
 * could be used.
 */
export const connectionFixture: ConnectionStatus = {
  connected: true,
  method: 'personal_key',
  key_source: 'personal_key',
  key_last4: '4F2A',
  last_sync_at: new Date(Date.now() - 2 * 3_600_000).toISOString(),
  generated_at: '2026-05-13T06:30:00Z',
}

/** The connection, with no key held. */
export const connectionEmptyFixture: ConnectionStatus = {
  connected: false,
  method: null,
  key_source: 'none',
  key_last4: null,
  last_sync_at: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/** The service's own health — and the version, for About. */
export const healthFixture: Health = {
  ok: true,
  service: 'stride',
  version: '0.9.0',
  garmin_connected: true,
  activities: 42,
  wellness_days: 29,
  plan_sessions: 7,
  generated_at: '2026-05-13T06:30:00Z',
}

/**
 * The settings.
 *
 * A goal of forty kilometres a week, no pause, and a database of a
 * couple of megabytes. The goal is a number the athlete wrote; the app
 * never invents one.
 */
export const settingsFixture: SettingsState = {
  goal: { value: value('40 km', 40_000), km: 40 },
  pause: null,
  storage: {
    size: value('2.4 MB', 2_516_582),
    activity_count: 42,
    wellness_day_count: 29,
    plan_session_count: 7,
    note_count: 1,
  },
  generated_at: '2026-05-13T06:30:00Z',
}

/** The settings with nothing set: no goal, no pause, an empty log. */
export const settingsEmptyFixture: SettingsState = {
  goal: { value: null, km: null },
  pause: null,
  storage: {
    size: value('64 KB', 65_536),
    activity_count: 0,
    wellness_day_count: 0,
    plan_session_count: 0,
    note_count: 0,
  },
  generated_at: '2026-05-13T06:30:00Z',
}

/**
 * The settings with a pause on.
 *
 * The plan is paused until the 19th, for travel. While it is paused the
 * app says so — a fact, in the app's voice.
 */
export const settingsPausedFixture: SettingsState = {
  ...settingsFixture,
  pause: {
    start_date: '2026-05-13',
    end_date: '2026-05-19',
    reason: 'travel',
  },
}

/** The answer to an export. */
export const exportFixture: SettingsExportResult = {
  exported: true,
  size: value('2.4 MB', 2_516_582),
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to deleting everything. */
export const deleteFixture: SettingsDeleteResult = {
  deleted: true,
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to a note. */
export const noteFixture: NoteResult = {
  note: 'Easy from the start, and the wind picked up on the way back.',
  generated_at: '2026-05-13T06:30:00Z',
}

/**
 * The answer to importing a FIT file.
 *
 * A file that is already in the log is a duplicate: it adds no row, and
 * it is reported once, in one line.
 */
export const fitImportFixture: FitImportResult = {
  imported: true,
  duplicate: false,
  activity_id: 107,
  date: null,
  distance: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to importing a file that is already in the log. */
export const fitImportDuplicateFixture: FitImportResult = {
  imported: false,
  duplicate: true,
  activity_id: 102,
  date: '2026-05-09',
  distance: value('10.2 km', 10_200),
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to a key that Garmin accepted. */
export const keyConnectFixture: KeyConnectResult = {
  connected: true,
  method: 'personal_key',
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to a key that Garmin accepted, when it was only tested. */
export const keyTestFixture: KeyTestResult = {
  ok: true,
  generated_at: '2026-05-13T06:30:00Z',
}

/** The answer to importing a plan file. */
export const planImportFixture: PlanImportResult = {
  imported: true,
  block_id: 1,
  sessions: 7,
  generated_at: '2026-05-13T06:30:00Z',
}

// ------------------------------------------------------------- the states

/**
 * The vitals with nothing behind them.
 *
 * No nights, no band and no direction: the reading is there, and there is
 * nothing to compare it with. **No invented numbers** — the history is
 * empty, not zero.
 */
export const vitalsEmptyFixture: VitalsSnapshot = {
  date: TODAY,
  generated_at: '2026-05-13T06:30:00Z',
  rest: { label: 'Rest', line: null },
  vitals: [],
  history: { sleep: [], hrv: [], resting_hr: [] },
  patterns: [],
}

/**
 * The vitals with too little history to compare them with.
 *
 * The value is there and the comparison is not: the row says so, in the
 * core's own words.
 */
export const vitalsNotEnoughFixture: VitalsSnapshot = {
  date: TODAY,
  generated_at: '2026-05-13T06:30:00Z',
  rest: { label: 'Rest', line: 'not enough data yet' },
  vitals: [
    {
      key: 'sleep',
      label: 'Sleep',
      value_text: '6 h 10 m',
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
    {
      key: 'hrv',
      label: 'HRV',
      value_text: '48 ms',
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
    {
      key: 'resting_hr',
      label: 'Resting HR',
      value_text: '63 bpm',
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
  ],
  history: {
    sleep: history.slice(-2).map((night) => ({ date: night.date, value: night.sleep_s })),
    hrv: history.slice(-2).map((night) => ({ date: night.date, value: night.hrv_ms })),
    resting_hr: history.slice(-2).map((night) => ({ date: night.date, value: night.resting_hr })),
  },
  patterns: [],
}

/** Today, with nothing in it. */
export const todayEmptyFixture: TodayInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  date: TODAY,
  planned: null,
  options: [],
  last_run: null,
  distance_comparison: null,
  week: { distance: value('0 km', 0), sessions: 0 },
}

/** The feed, with nothing in it. */
export const activitiesEmptyFixture: ActivityListResponse = {
  activities: [],
  next_cursor: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/** Progress, with too little history to draw a trend. */
export const progressNotEnoughFixture: ProgressInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  headline: 'not enough data yet',
  headline_note: null,
  weekly_volume: [
    { week_start: '2026-05-04', distance: value('38.6 km', 38_600) },
    { week_start: '2026-05-11', distance: value('20.6 km', 20_600) },
  ],
  typical_week: null,
  pace_at_hr: [
    { week_start: '2026-05-04', pace: value('5:18 /km', 318), heart_rate: value('150 bpm', 150) },
    { week_start: '2026-05-11', pace: value('5:12 /km', 312), heart_rate: value('150 bpm', 150) },
  ],
  bests: [],
  consistency: [
    { week_start: '2026-05-04', sessions: 4 },
    { week_start: '2026-05-11', sessions: 2 },
  ],
  sessions_per_week: null,
  adherence: null,
}

/** Progress, with no plan behind it: the adherence card is omitted. */
export const progressNoPlanFixture: ProgressInsight = {
  ...progressFixture,
  adherence: null,
}

/** A sync that is three days old. */
export const syncStatusStaleFixture: SyncStatus = {
  last_sync_at: new Date(Date.now() - 3 * 86_400_000).toISOString(),
  last_error: null,
  activity_count: 42,
}

/** A sync that did not finish. */
export const syncStatusFailedFixture: SyncStatus = {
  last_sync_at: new Date(Date.now() - 2 * 3_600_000).toISOString(),
  last_error: 'Garmin refused the token',
  activity_count: 42,
}

/** The health of a sync window that has gone stale. */
export const healthDailyStaleFixture: HealthDaily = {
  date: TODAY,
  generated_at: '2026-05-13T06:30:00Z',
  stale: true,
}

// ------------------------------------------------------------- dispatcher

/**
 * Answer a route from the fixture set.
 *
 * `undefined` means "this route has no fixture", which the client turns
 * into a typed not-found — the same shape a real router would answer.
 *
 * The **variant** is the state the screens are reviewed in:
 *
 * | variant | the state |
 * |---|---|
 * | `1` (or empty) | everything present |
 * | `over` | the last run beyond 10% of the plan's target |
 * | `empty` | no runs, no wellness, no plan, no key |
 * | `stale` | the last sync three days old |
 * | `failed` | the last sync did not finish |
 * | `baseline` | a plan inferred from the athlete's history |
 * | `noplan` | no plan at all |
 * | `notenough` | too little history to compare anything |
 * | `paused` | the plan is paused until the 19th, for travel |
 * | `duplicate` | a FIT file that is already in the log |
 */
export function fixtureResponse(
  path: string,
  params: Record<string, string> = {},
  variant = '',
): unknown {
  switch (path) {
    case '/insights/today':
      if (variant === 'empty') return todayEmptyFixture
      if (variant === 'noplan') return { ...todayFixture, planned: null, options: [] }
      if (variant === 'paused') return { ...todayFixture, options: [] }
      return variant === 'over' ? todayOverFixture : todayFixture
    case '/physiology/vitals':
      if (variant === 'empty') return vitalsEmptyFixture
      if (variant === 'notenough') return vitalsNotEnoughFixture
      return vitalsFixture
    case '/activities':
      return variant === 'empty' ? activitiesEmptyFixture : activitiesFixture
    case '/activities/101':
      return activityDetailFixture
    case '/plan/week':
      if (variant === 'empty' || variant === 'noplan') {
        return new FixtureFailure('no_plan', 'there is no plan for this week', 404)
      }
      if (variant === 'baseline') return planWeekBaselineFixture
      return planWeekFixture
    case '/insights/progress':
      if (variant === 'notenough') return progressNotEnoughFixture
      if (variant === 'empty' || variant === 'noplan') return progressNoPlanFixture
      return progressFixture
    case '/compare': {
      // Which run is A is the API's business: the app asks, and the API
      // answers the question in that order.
      return params['a'] === '101' ? compareSwappedFixture : compareFixture
    }
    case '/compare/candidates':
      return compareCandidatesFixture
    case '/sync/status':
      if (variant === 'stale') return syncStatusStaleFixture
      if (variant === 'failed') return syncStatusFailedFixture
      return syncStatusFixture
    case '/health/daily':
      return variant === 'stale' ? healthDailyStaleFixture : healthDailyFixture
    case '/health':
      return healthFixture
    case '/auth/status':
      return variant === 'empty' ? connectionEmptyFixture : connectionFixture
    case '/settings':
      if (variant === 'empty') return settingsEmptyFixture
      if (variant === 'paused') return settingsPausedFixture
      return settingsFixture
    case '/settings/export':
      return exportFixture
    case '/settings/delete':
      return deleteFixture
    case '/auth/garmin/personal-key':
      return keyConnectFixture
    case '/auth/garmin/personal-key/test':
      return keyTestFixture
    case '/auth/garmin/fit-import':
      return variant === 'duplicate' ? fitImportDuplicateFixture : fitImportFixture
    case '/auth/garmin/logout':
      return { ...keyConnectFixture, connected: false, method: null }
    case '/plan/import':
      return planImportFixture
    default:
      if (path.startsWith('/activities/') && path.endsWith('/note')) {
        return noteFixture
      }
      return undefined
  }
}
