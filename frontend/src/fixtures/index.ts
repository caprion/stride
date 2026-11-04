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
  FitnessZones,
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
import answersData from './answers.json'
import compareAnswerData from './compare-answers.json'

/**
 * The router's answers, by the route that answers them.
 *
 * **The fixture is the router's answer, not a mock.** The file is written
 * from the router's own dump (`fixtures:export`), and the api's parity
 * test holds it true: it seeds a store from the seed that travels with
 * it, calls the real router and compares the answers value for value.
 */
const routerAnswers = answersData.answers as Record<string, unknown>

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
  const start = new Date(Date.UTC(2026, 3, 14))
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

/**
 * The nights, as the seed carries them.
 *
 * The api's test seeds a store from the fixture set, and the nights are
 * what it seeds the wellness with.
 */
export const nightsFixture = nights

/** The previous nights, the day itself excluded — the band's window. */
const history = nights.slice(0, 29)

/**
 * The vitals — the snapshot Today and Body both read.
 *
 * The band is the median ± 1.5 × MAD of the nights before the day
 * itself, and **the router computes it**: the snapshot below is the
 * router's answer, with the band it worked out (sleep 26 400 ± 2 700 s,
 * HRV 62 ± 6 ms, resting heart rate 58 ± 6 bpm) and the sentences it
 * wrote for those numbers.
 */
/** The vitals: the router's answer. */
export const vitalsFixture = routerAnswers['/physiology/vitals'] as VitalsSnapshot

// ------------------------------------------------------------------ plan

/**
 * A session, as the seed carries it.
 *
 * The seed is what a store has to hold for the router to answer the same
 * way: the day, the kind of session, its title and its target.
 *
 * **The status, the actual distance and the sentence are the router's.**
 * The router reads them from the store and writes them, and the app shows
 * them as they come — so the seed does not carry them. A seed that said
 * a session was run would be a second answer, and it would disagree with
 * the router's.
 */
export interface SeedPlanSession {
  date: string
  session_type: string
  title: string
  target: RenderedValue | null
}

const planSessions: SeedPlanSession[] = [
  { date: '2026-05-11', session_type: 'easy', title: 'Easy 8', target: value('8 km', 8_000) },
  {
    date: '2026-05-12',
    session_type: 'easy',
    title: 'Easy 10',
    target: value('10 km', 10_000),
  },
  {
    date: '2026-05-13',
    session_type: 'easy',
    title: 'Easy 10',
    target: value('10 km', 10_000),
  },
  { date: '2026-05-14', session_type: 'easy', title: 'Easy 5', target: value('5 km', 5_000) },
  {
    date: '2026-05-15',
    session_type: 'long',
    title: 'Long 16',
    target: value('16 km', 16_000),
  },
  // A session with no distance target: the time is the target, and the
  // router says so in its own words.
  { date: '2026-05-16', session_type: 'easy', title: 'Easy 30 minutes', target: null },
  { date: '2026-05-17', session_type: 'easy', title: 'Easy 6', target: value('6 km', 6_000) },
]

/**
 * The plan's sessions, as the seed carries them.
 *
 * The api's test seeds a store from the fixture set, and these are the
 * plan it seeds.
 */
export const planSessionsFixture: SeedPlanSession[] = planSessions

/** The plan's week: the router's answer. */
export const planWeekFixture = routerAnswers['/plan/week'] as PlanWeek

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
  sessions: (planWeekFixture.sessions as PlanSession[]).map((session) => ({
    ...session,
    status: session.status === 'done' ? ('done' as const) : ('planned' as const),
    source: 'baseline' as const,
  })),
  counts: { done: 0, planned: 7, not_done: 0 },
}

// -------------------------------------------------------------- activities

/**
 * A run, as the seed carries it.
 *
 * The seed is what a store has to hold: the day it happened, how far it
 * was, how long it took, and the heart rate it averaged. **The pace, the
 * group, the story and the route are the router's** — it computes them
 * from the run and writes them, so the seed does not carry them: a seed
 * that carried a pace would be a second implementation of the core's
 * arithmetic, and a seed that carried a group would be a second
 * implementation of the api's.
 */
export interface SeedActivity {
  id: number
  name: string
  sport_type: string
  start_time: string
  local_date: string
  distance: RenderedValue
  duration: RenderedValue
  elevation_gain: RenderedValue
  heart_rate: RenderedValue
}

function activity(
  id: number,
  date: string,
  distance: RenderedValue,
  duration: RenderedValue,
  heartRate: RenderedValue = value('148 bpm', 148),
): SeedActivity {
  return {
    id,
    name: 'Run',
    sport_type: 'running',
    start_time: `${date}T07:10:00Z`,
    local_date: date,
    distance,
    duration,
    heart_rate: heartRate,
    elevation_gain: value('42 m', 42),
  }
}

export const activityFixtures: SeedActivity[] = [
  activity(101, '2026-05-10', value('12.4 km', 12_400), value('1 h 7 m', 4_020)),
  activity(
    102,
    '2026-05-09',
    value('10.2 km', 10_200),
    value('55 m', 3_320),
    value('146 bpm', 146),
  ),
  activity(103, '2026-05-07', value('8.4 km', 8_400), value('42 m', 2_520)),
  activity(104, '2026-05-06', value('5 km', 5_000), value('27 m', 1_620)),
  activity(105, '2026-05-04', value('16.1 km', 16_100), value('1 h 25 m', 5_072)),
  activity(106, '2026-05-03', value('6.3 km', 6_300), value('35 m', 2_080)),
]

/** The feed: the router's answer. */
export const activitiesFixture = routerAnswers['/activities'] as ActivityListResponse

/**
 * A run out of the feed, as the router answered it.
 *
 * The feed is the router's, so a run taken out of it carries everything
 * the router said about it — the group, the pace, the story.
 */
function runById(id: number): ActivitySummary | null {
  return (activitiesFixture.activities.find((run) => run.id === id) ??
    null) as ActivitySummary | null
}

/** One run, in full: the router's answer. */
export const activityDetailFixture = routerAnswers['/activities/101'] as ActivityDetail

// ----------------------------------------------------------------- today

/** Today: the router's answer. */
export const todayFixture = routerAnswers['/insights/today'] as TodayInsight

/**
 * A route, for the screens that draw one.
 *
 * The seeded store holds no FIT file, so **the router answers no points**
 * and the app draws nothing — which is the honest answer. These
 * coordinates are the `route` variant's own: a loop around the
 * repository's synthetic anchor — the South Pacific, far from any coast
 * and from any real route — so that the map, the thumbnail and the mini
 * thumbnail can be seen and reviewed without a real run. They mean
 * nothing.
 */
const SYNTHETIC_ROUTE: [number, number][] = [
  [-35.0, -139.99],
  [-34.99562, -139.98895],
  [-34.99126, -139.98992],
  [-34.98869, -139.99274],
  [-34.98841, -139.99608],
  [-34.98907, -139.99884],
  [-34.98907, -140.00116],
  [-34.98841, -140.00392],
  [-34.98869, -140.00726],
  [-34.99126, -140.01008],
  [-34.99562, -140.01105],
  [-35.0, -140.01],
  [-35.00323, -140.00814],
  [-35.00585, -140.00675],
  [-35.0091, -140.00584],
  [-35.01297, -140.00439],
  [-35.01579, -140.00168],
  [-35.01579, -139.99832],
  [-35.01297, -139.99561],
  [-35.0091, -139.99416],
  [-35.00585, -139.99325],
  [-35.00323, -139.99186],
]

/** A run, with the route on it. */
function withRoute(activity: ActivitySummary): ActivitySummary {
  return { ...activity, geo_points: SYNTHETIC_ROUTE }
}

// --------------------------------------------------------------- progress

/** Progress: the router's answer. */
export const progressFixture = routerAnswers['/insights/progress'] as ProgressInsight

// --------------------------------------------------------------- compare

/**
 * The two effort series, written out.
 *
 * A point carries the core's words beside the number: the text for the
 * tooltip, the raw for the line. They are written out rather than
 * computed, because a fixture that computed its own values would be a
 * second implementation of the core's rules.
 */
export const SERIES_A: CompareSeriesPoint[] = [
  {
    offset_s: 0,
    pace: { text: '5:17 /km', raw: 317 },
    heart_rate: { text: '149 bpm', raw: 149 },
  },
  {
    offset_s: 60,
    pace: { text: '5:19 /km', raw: 319 },
    heart_rate: { text: '151 bpm', raw: 151 },
  },
  {
    offset_s: 120,
    pace: { text: '5:22 /km', raw: 322 },
    heart_rate: { text: '153 bpm', raw: 153 },
  },
  {
    offset_s: 180,
    pace: { text: '5:24 /km', raw: 324 },
    heart_rate: { text: '155 bpm', raw: 155 },
  },
  {
    offset_s: 240,
    pace: { text: '5:26 /km', raw: 326 },
    heart_rate: { text: '157 bpm', raw: 157 },
  },
  {
    offset_s: 300,
    pace: { text: '5:27 /km', raw: 327 },
    heart_rate: { text: '158 bpm', raw: 158 },
  },
  {
    offset_s: 360,
    pace: { text: '5:28 /km', raw: 328 },
    heart_rate: { text: '160 bpm', raw: 160 },
  },
  {
    offset_s: 420,
    pace: { text: '5:29 /km', raw: 329 },
    heart_rate: { text: '161 bpm', raw: 161 },
  },
  {
    offset_s: 480,
    pace: { text: '5:29 /km', raw: 329 },
    heart_rate: { text: '162 bpm', raw: 162 },
  },
  {
    offset_s: 540,
    pace: { text: '5:29 /km', raw: 329 },
    heart_rate: { text: '162 bpm', raw: 162 },
  },
  {
    offset_s: 600,
    pace: { text: '5:28 /km', raw: 328 },
    heart_rate: { text: '163 bpm', raw: 163 },
  },
  {
    offset_s: 660,
    pace: { text: '5:27 /km', raw: 327 },
    heart_rate: { text: '163 bpm', raw: 163 },
  },
  {
    offset_s: 720,
    pace: { text: '5:25 /km', raw: 325 },
    heart_rate: { text: '163 bpm', raw: 163 },
  },
  {
    offset_s: 780,
    pace: { text: '5:23 /km', raw: 323 },
    heart_rate: { text: '162 bpm', raw: 162 },
  },
  {
    offset_s: 840,
    pace: { text: '5:21 /km', raw: 321 },
    heart_rate: { text: '162 bpm', raw: 162 },
  },
  {
    offset_s: 900,
    pace: { text: '5:19 /km', raw: 319 },
    heart_rate: { text: '161 bpm', raw: 161 },
  },
  {
    offset_s: 960,
    pace: { text: '5:16 /km', raw: 316 },
    heart_rate: { text: '160 bpm', raw: 160 },
  },
  {
    offset_s: 1020,
    pace: { text: '5:14 /km', raw: 314 },
    heart_rate: { text: '158 bpm', raw: 158 },
  },
  {
    offset_s: 1080,
    pace: { text: '5:12 /km', raw: 312 },
    heart_rate: { text: '157 bpm', raw: 157 },
  },
  {
    offset_s: 1140,
    pace: { text: '5:10 /km', raw: 310 },
    heart_rate: { text: '155 bpm', raw: 155 },
  },
  {
    offset_s: 1200,
    pace: { text: '5:08 /km', raw: 308 },
    heart_rate: { text: '153 bpm', raw: 153 },
  },
  {
    offset_s: 1260,
    pace: { text: '5:07 /km', raw: 307 },
    heart_rate: { text: '151 bpm', raw: 151 },
  },
  {
    offset_s: 1320,
    pace: { text: '5:06 /km', raw: 306 },
    heart_rate: { text: '149 bpm', raw: 149 },
  },
  {
    offset_s: 1380,
    pace: { text: '5:05 /km', raw: 305 },
    heart_rate: { text: '147 bpm', raw: 147 },
  },
  {
    offset_s: 1440,
    pace: { text: '5:05 /km', raw: 305 },
    heart_rate: { text: '145 bpm', raw: 145 },
  },
  {
    offset_s: 1500,
    pace: { text: '5:05 /km', raw: 305 },
    heart_rate: { text: '143 bpm', raw: 143 },
  },
  {
    offset_s: 1560,
    pace: { text: '5:06 /km', raw: 306 },
    heart_rate: { text: '141 bpm', raw: 141 },
  },
  {
    offset_s: 1620,
    pace: { text: '5:08 /km', raw: 308 },
    heart_rate: { text: '140 bpm', raw: 140 },
  },
  {
    offset_s: 1680,
    pace: { text: '5:09 /km', raw: 309 },
    heart_rate: { text: '138 bpm', raw: 138 },
  },
  {
    offset_s: 1740,
    pace: { text: '5:11 /km', raw: 311 },
    heart_rate: { text: '137 bpm', raw: 137 },
  },
  {
    offset_s: 1800,
    pace: { text: '5:14 /km', raw: 314 },
    heart_rate: { text: '136 bpm', raw: 136 },
  },
  {
    offset_s: 1860,
    pace: { text: '5:16 /km', raw: 316 },
    heart_rate: { text: '136 bpm', raw: 136 },
  },
  {
    offset_s: 1920,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '135 bpm', raw: 135 },
  },
  {
    offset_s: 1980,
    pace: { text: '5:21 /km', raw: 321 },
    heart_rate: { text: '135 bpm', raw: 135 },
  },
  {
    offset_s: 2040,
    pace: { text: '5:23 /km', raw: 323 },
    heart_rate: { text: '135 bpm', raw: 135 },
  },
  {
    offset_s: 2100,
    pace: { text: '5:25 /km', raw: 325 },
    heart_rate: { text: '136 bpm', raw: 136 },
  },
  {
    offset_s: 2160,
    pace: { text: '5:27 /km', raw: 327 },
    heart_rate: { text: '136 bpm', raw: 136 },
  },
  {
    offset_s: 2220,
    pace: { text: '5:28 /km', raw: 328 },
    heart_rate: { text: '137 bpm', raw: 137 },
  },
  {
    offset_s: 2280,
    pace: { text: '5:29 /km', raw: 329 },
    heart_rate: { text: '138 bpm', raw: 138 },
  },
  {
    offset_s: 2340,
    pace: { text: '5:29 /km', raw: 329 },
    heart_rate: { text: '140 bpm', raw: 140 },
  },
]

export const SERIES_B: CompareSeriesPoint[] = [
  {
    offset_s: 0,
    pace: { text: '5:06 /km', raw: 306 },
    heart_rate: { text: '146 bpm', raw: 146 },
  },
  {
    offset_s: 60,
    pace: { text: '5:08 /km', raw: 308 },
    heart_rate: { text: '148 bpm', raw: 148 },
  },
  {
    offset_s: 120,
    pace: { text: '5:11 /km', raw: 311 },
    heart_rate: { text: '150 bpm', raw: 150 },
  },
  {
    offset_s: 180,
    pace: { text: '5:13 /km', raw: 313 },
    heart_rate: { text: '152 bpm', raw: 152 },
  },
  {
    offset_s: 240,
    pace: { text: '5:15 /km', raw: 315 },
    heart_rate: { text: '154 bpm', raw: 154 },
  },
  {
    offset_s: 300,
    pace: { text: '5:16 /km', raw: 316 },
    heart_rate: { text: '155 bpm', raw: 155 },
  },
  {
    offset_s: 360,
    pace: { text: '5:17 /km', raw: 317 },
    heart_rate: { text: '157 bpm', raw: 157 },
  },
  {
    offset_s: 420,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '158 bpm', raw: 158 },
  },
  {
    offset_s: 480,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '159 bpm', raw: 159 },
  },
  {
    offset_s: 540,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '159 bpm', raw: 159 },
  },
  {
    offset_s: 600,
    pace: { text: '5:17 /km', raw: 317 },
    heart_rate: { text: '160 bpm', raw: 160 },
  },
  {
    offset_s: 660,
    pace: { text: '5:16 /km', raw: 316 },
    heart_rate: { text: '160 bpm', raw: 160 },
  },
  {
    offset_s: 720,
    pace: { text: '5:14 /km', raw: 314 },
    heart_rate: { text: '160 bpm', raw: 160 },
  },
  {
    offset_s: 780,
    pace: { text: '5:12 /km', raw: 312 },
    heart_rate: { text: '159 bpm', raw: 159 },
  },
  {
    offset_s: 840,
    pace: { text: '5:10 /km', raw: 310 },
    heart_rate: { text: '159 bpm', raw: 159 },
  },
  {
    offset_s: 900,
    pace: { text: '5:08 /km', raw: 308 },
    heart_rate: { text: '158 bpm', raw: 158 },
  },
  {
    offset_s: 960,
    pace: { text: '5:05 /km', raw: 305 },
    heart_rate: { text: '157 bpm', raw: 157 },
  },
  {
    offset_s: 1020,
    pace: { text: '5:03 /km', raw: 303 },
    heart_rate: { text: '155 bpm', raw: 155 },
  },
  {
    offset_s: 1080,
    pace: { text: '5:01 /km', raw: 301 },
    heart_rate: { text: '154 bpm', raw: 154 },
  },
  {
    offset_s: 1140,
    pace: { text: '4:59 /km', raw: 299 },
    heart_rate: { text: '152 bpm', raw: 152 },
  },
  {
    offset_s: 1200,
    pace: { text: '4:57 /km', raw: 297 },
    heart_rate: { text: '150 bpm', raw: 150 },
  },
  {
    offset_s: 1260,
    pace: { text: '4:56 /km', raw: 296 },
    heart_rate: { text: '148 bpm', raw: 148 },
  },
  {
    offset_s: 1320,
    pace: { text: '4:55 /km', raw: 295 },
    heart_rate: { text: '146 bpm', raw: 146 },
  },
  {
    offset_s: 1380,
    pace: { text: '4:54 /km', raw: 294 },
    heart_rate: { text: '144 bpm', raw: 144 },
  },
  {
    offset_s: 1440,
    pace: { text: '4:54 /km', raw: 294 },
    heart_rate: { text: '142 bpm', raw: 142 },
  },
  {
    offset_s: 1500,
    pace: { text: '4:54 /km', raw: 294 },
    heart_rate: { text: '140 bpm', raw: 140 },
  },
  {
    offset_s: 1560,
    pace: { text: '4:55 /km', raw: 295 },
    heart_rate: { text: '138 bpm', raw: 138 },
  },
  {
    offset_s: 1620,
    pace: { text: '4:57 /km', raw: 297 },
    heart_rate: { text: '137 bpm', raw: 137 },
  },
  {
    offset_s: 1680,
    pace: { text: '4:58 /km', raw: 298 },
    heart_rate: { text: '135 bpm', raw: 135 },
  },
  {
    offset_s: 1740,
    pace: { text: '5:00 /km', raw: 300 },
    heart_rate: { text: '134 bpm', raw: 134 },
  },
  {
    offset_s: 1800,
    pace: { text: '5:03 /km', raw: 303 },
    heart_rate: { text: '133 bpm', raw: 133 },
  },
  {
    offset_s: 1860,
    pace: { text: '5:05 /km', raw: 305 },
    heart_rate: { text: '133 bpm', raw: 133 },
  },
  {
    offset_s: 1920,
    pace: { text: '5:07 /km', raw: 307 },
    heart_rate: { text: '132 bpm', raw: 132 },
  },
  {
    offset_s: 1980,
    pace: { text: '5:10 /km', raw: 310 },
    heart_rate: { text: '132 bpm', raw: 132 },
  },
  {
    offset_s: 2040,
    pace: { text: '5:12 /km', raw: 312 },
    heart_rate: { text: '132 bpm', raw: 132 },
  },
  {
    offset_s: 2100,
    pace: { text: '5:14 /km', raw: 314 },
    heart_rate: { text: '133 bpm', raw: 133 },
  },
  {
    offset_s: 2160,
    pace: { text: '5:16 /km', raw: 316 },
    heart_rate: { text: '133 bpm', raw: 133 },
  },
  {
    offset_s: 2220,
    pace: { text: '5:17 /km', raw: 317 },
    heart_rate: { text: '134 bpm', raw: 134 },
  },
  {
    offset_s: 2280,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '135 bpm', raw: 135 },
  },
  {
    offset_s: 2340,
    pace: { text: '5:18 /km', raw: 318 },
    heart_rate: { text: '137 bpm', raw: 137 },
  },
]

/**
 * The compare answers, one for every pair the screen can ask about.
 *
 * They are **data**: the router's answers, written down, and the api's
 * test holds that they are the router's, byte for byte, against the
 * store the fixture set describes. The effort series are the two the
 * fixture set has; the other runs have no samples in the store, so their
 * series is empty — the way the router would answer it.
 */
const compareAnswers = compareAnswerData as unknown as Record<string, CompareResponse>

/**
 * The answer for a pair.
 *
 * The pair is the app's question and the api's business: the app asks
 * for two runs and shows the answer as it comes.
 */
export function compareAnswer(a: string, b: string): CompareResponse | undefined {
  return compareAnswers[`${a}:${b}`]
}

/** The runs the Compare screen can offer: the router's answer. */
export const compareCandidatesFixture = routerAnswers[
  '/compare/candidates'
] as CompareCandidate[]
// ------------------------------------------------------------------ sync

/** The state of the last sync: the router's answer. */
export const syncStatusFixture = routerAnswers['/sync/status'] as SyncStatus

/** The health of the sync window: the router's answer. */
export const healthDailyFixture = routerAnswers['/health/daily'] as HealthDaily

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
/** The connection: the router's answer. */
export const connectionFixture = routerAnswers['/auth/status'] as ConnectionStatus

/** The connection, with no key held. */
export const connectionEmptyFixture: ConnectionStatus = {
  connected: false,
  method: null,
  key_source: 'none',
  key_last4: null,
  last_sync_at: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/**
 * The connection, with a key held.
 *
 * The seeded store holds no key, so the router answers "not connected" —
 * and the app says so. This is the other state: a key has been taken, and
 * the answer carries the last four characters of it. **The key itself is
 * not here and never will be**: four characters cannot be a Garmin
 * Personal API key.
 */
export const connectionHeldFixture: ConnectionStatus = {
  connected: true,
  method: 'personal_key',
  key_source: 'personal_key',
  key_last4: '4F2A',
  last_sync_at: '2026-05-13T04:30:00Z',
  generated_at: '2026-05-13T06:30:00Z',
}

/**
 * The heart-rate zones of a run.
 *
 * The seeded store holds no samples, so the router answers the honest
 * one: the bands cannot be computed, and the core says why. The bands
 * themselves are the api's — the app draws what it is given.
 */
export const zonesFixture: FitnessZones = {
  basis: null,
  max_heart_rate: null,
  zones: [],
  without_heart_rate: null,
  unavailable: 'zones unavailable — no stored max or threshold heart rate',
}

/** The service's own health — and the version, for About. */
export const healthFixture = routerAnswers['/health'] as Health

/**
 * The settings.
 *
 * A goal of forty kilometres a week, no pause, and a database of a
 * couple of megabytes. The goal is a number the athlete wrote; the app
 * never invents one.
 */
/** The settings: the router's answer. */
export const settingsFixture = routerAnswers['/settings'] as SettingsState

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
    resting_hr: history
      .slice(-2)
      .map((night) => ({ date: night.date, value: night.resting_hr })),
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
  week: { distance: value('0 km', 0), sessions: 0, days: [] },
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
  date: '2026-05-13',
  headline: 'not enough data yet',
  headline_note: null,
  weekly_volume: [
    { week_start: '2026-05-04', distance: value('38.6 km', 38_600) },
    { week_start: '2026-05-11', distance: value('20.6 km', 20_600) },
  ],
  typical_week: null,
  pace_at_hr: [
    {
      week_start: '2026-05-04',
      pace: value('5:18 /km', 318),
      heart_rate: value('150 bpm', 150),
    },
    {
      week_start: '2026-05-11',
      pace: value('5:12 /km', 312),
      heart_rate: value('150 bpm', 150),
    },
  ],
  bests: [],
  consistency: [
    { week_start: '2026-05-04', sessions: 4 },
    { week_start: '2026-05-11', sessions: 2 },
  ],
  sessions_per_week: null,
  adherence: null,
  weeks_needed: 4,
}

/** Progress, with no plan behind it: the adherence card is omitted. */
export const progressNoPlanFixture: ProgressInsight = {
  ...progressFixture,
  adherence: null,
}

/** A sync that is three days old. */
export const syncStatusStaleFixture: SyncStatus = {
  history: null,
  more_pending: false,
  last_sync_at: new Date(Date.now() - 3 * 86_400_000).toISOString(),
  last_error: null,
  activity_count: 6,
  cursor: null,
  pulled: 0,
  skipped: 0,
  deleted: 0,
}

/**
 * A sync that never happened.
 *
 * The app has no key yet: nothing has been pulled, and the first-run
 * page is the one that takes the key.
 */
export const syncStatusFreshFixture: SyncStatus = {
  history: null,
  more_pending: false,
  last_sync_at: null,
  last_error: null,
  activity_count: 0,
  // Nothing has been pulled, so there is no cursor and nothing to count.
  cursor: null,
  pulled: 0,
  skipped: 0,
  deleted: 0,
}

/**
 * A sync that did not finish.
 *
 * The cursor stays where it was — a failed pull never advances it — and
 * the counts are not known, because the pull never finished.
 */
export const syncStatusFailedFixture: SyncStatus = {
  history: null,
  more_pending: false,
  last_sync_at: new Date(Date.now() - 2 * 3_600_000).toISOString(),
  last_error: 'Garmin refused the token',
  activity_count: 6,
  // The cursor stays where it was — a failed pull never advances it — and
  // the counts are the ones the last pull that finished made.
  cursor: null,
  pulled: 0,
  skipped: 0,
  deleted: 0,
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
 * | `noclose` | the last run inside 10% of the target: no comparison |
 * | `empty` | no runs, no wellness, no plan, and nothing ever synced |
 * | `nowellness` | synced, but no wellness data yet |
 * | `fresh` | no key yet: the first-run page is the one that takes it |
 * | `connected` | a key has been taken: the app is connected |
 * | `route` | the runs carry a route, so the map can be seen |
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
  body?: unknown,
): unknown {
  switch (path) {
    case '/insights/today': {
      if (variant === 'empty' || variant === 'fresh') return todayEmptyFixture
      // The wellness has not come, but the runs have: the run and the
      // plan are the router's.
      if (variant === 'nowellness') return todayFixture
      if (variant === 'noplan') return { ...todayFixture, planned: null, options: [] }
      // The gap is not news: the last run is inside ten per cent of the
      // target, so there is no comparison to state.
      if (variant === 'noclose') {
        return {
          ...todayFixture,
          last_run: runById(102),
          distance_comparison: null,
        }
      }
      if (variant === 'paused') return { ...todayFixture, options: [] }
      // The run carries a route, so the mini thumbnail has something to
      // draw.
      if (variant === 'route') {
        return {
          ...todayFixture,
          last_run: todayFixture.last_run ? withRoute(todayFixture.last_run) : null,
        }
      }
      return todayFixture
    }
    case '/physiology/vitals':
      if (variant === 'empty' || variant === 'fresh') return vitalsEmptyFixture
      if (variant === 'nowellness') return vitalsEmptyFixture
      if (variant === 'notenough') return vitalsNotEnoughFixture
      return vitalsFixture
    case '/activities': {
      const answer =
        variant === 'empty' || variant === 'fresh' ? activitiesEmptyFixture : activitiesFixture
      if (variant !== 'route') return answer
      return {
        ...answer,
        activities: answer.activities.map(withRoute),
      }
    }
    case '/activities/101':
      if (variant === 'route') {
        return { ...activityDetailFixture, geo_points: SYNTHETIC_ROUTE }
      }
      return activityDetailFixture
    case '/activities/101/fit':
      // The run's original file: the header of a FIT file, byte for
      // byte. The fixture set is synthetic, and a file is no less so
      // than a run.
      return [14, 0, 0, 0, 16, 32, 0, 0, 46, 70, 73, 84, 0, 0]
    case '/plan/week':
      if (
        variant === 'empty' ||
        variant === 'fresh' ||
        variant === 'nowellness' ||
        variant === 'noplan'
      ) {
        return new FixtureFailure('no_plan', 'there is no plan for this week', 404)
      }
      if (variant === 'baseline') return planWeekBaselineFixture
      return planWeekFixture
    case '/insights/progress':
      if (variant === 'notenough') return progressNotEnoughFixture
      if (variant === 'empty' || variant === 'fresh' || variant === 'noplan') {
        return progressNoPlanFixture
      }
      return progressFixture
    case '/compare': {
      // Which run is A is the API's business: the app asks, and the API
      // answers the question in that order.
      return compareAnswer(params['a'] ?? '', params['b'] ?? '')
    }
    case '/compare/candidates':
      return compareCandidatesFixture
    case '/sync/status':
      if (variant === 'stale') return syncStatusStaleFixture
      if (variant === 'failed') return syncStatusFailedFixture
      if (variant === 'fresh' || variant === 'empty') return syncStatusFreshFixture
      return syncStatusFixture
    case '/health/daily':
      return variant === 'stale' ? healthDailyStaleFixture : healthDailyFixture
    case '/fitness/zones':
      return zonesFixture
    case '/health':
      return healthFixture
    case '/auth/status':
      if (variant === 'empty' || variant === 'fresh') return connectionEmptyFixture
      if (variant === 'connected') return connectionHeldFixture
      return connectionFixture
    case '/settings':
      if (variant === 'empty' || variant === 'fresh') return settingsEmptyFixture
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
        // The fixture answers as a captured answer would: it echoes the
        // note that was written.
        const written = (body as { text?: string | null } | undefined)?.text
        return { ...noteFixture, note: written ?? null }
      }
      return undefined
  }
}
