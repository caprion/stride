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
  ActivityCalendar,
  CompareSeriesPoint,
  ActivityListResponse,
  ActivitySummary,
  CompareCandidate,
  CompareResponse,
  ConnectionStatus,
  FilmGround,
  FilmSample,
  FilmSplit,
  FitImportResult,
  FitnessZones,
  Fingerprint,
  Health,
  HealthDaily,
  HeatmapCell,
  HeatmapInsight,
  KeyConnectResult,
  KeyTestResult,
  LoadInsight,
  NoteResult,
  PaceInsight,
  PlanImportResult,
  PlanWeek,
  ProgressInsight,
  RenderedValue,
  RollingInsight,
  RunFilm,

  RunUpdateResponse,
  SettingsDeleteResult,
  SettingsExportResult,
  SettingsState,
  SleepDays,
  StreakInsight,
  SyncStatus,
  TodayInsight,
  UpdatePayloadResponse,
  UpdateSettings,
  VitalsSnapshot,
  YearInsight,
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
 * The router's answers for the baseline variant.
 *
 * The same seed, with no plan block: the router infers the baseline from
 * the athlete's own history and answers the plan, the Today screen, the
 * progress and the health accordingly.
 */
const baselineAnswers = (answersData as { baseline?: Record<string, unknown> }).baseline ?? {}

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
 * The baseline is the router's own answer — the same seed with no plan
 * block, so the core reads the athlete's history and infers what it can.
 * The app does not infer a plan of its own; that would be a second
 * implementation of the core's plan engine.
 */
export const planWeekBaselineFixture = baselineAnswers['/plan/week'] as PlanWeek

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
  // A run in the window **before** the window: the twelve-week window ends
  // on the 13th of May, and the window before it runs from the 26th of
  // November to the 18th of February. Without a run in there the
  // comparison has nothing to compare, and every facet says "no value" for
  // its previous — which is honest but exercises nothing.
  activity(107, '2026-02-11', value('18.2 km', 18_200), value('1 h 34 m', 5_640)),
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

/** Today, with no plan: the router's answer for the baseline seed. */
export const baselineTodayFixture = baselineAnswers['/insights/today'] as TodayInsight

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

/** Progress, with no plan: the router's answer for the baseline seed. */
export const baselineProgressFixture = baselineAnswers['/insights/progress'] as ProgressInsight

// ------------------------------------------------------------------ pace

/**
 * The pace page: the router's answer.
 *
 * The training week by week, the pace at a heart rate, and where the time
 * went by pace.
 *
 * **The seeded store holds no FIT file**, so the router answers no bands:
 * the window holds no moving samples, and the screen says so. The bands
 * themselves are the router's, and the app draws what it is given.
 */
export const paceFixture = routerAnswers['/insights/pace'] as PaceInsight

// --------------------------------------------------------------- load

/**
 * The training load: the router's answer.
 *
 * The values, the scenario and the assumption they rest on. The answer
 * comes from the router's own store — the same seed — so the numbers on
 * the card are the numbers the api computes.
 */
export const loadFixture = routerAnswers['/insights/load'] as LoadInsight

// -------------------------------------------------------------- rolling

/**
 * The rolling view: the router's answer.
 *
 * The window against the window before it, the split of the training
 * across the athlete's own zones, the weekday and daytime splits, and the
 * monthly and yearly tables. The answer comes from the router's own
 * store — the same seed — so the numbers on the cards are the numbers the
 * api computes.
 */
export const rollingFixture = routerAnswers['/insights/rolling'] as RollingInsight

/**
 * The rolling view, with nothing in the store.
 *
 * The empty state: no runs, so the window holds no distance and the
 * tables hold no rows. The zone split has no value either — there is no
 * model and no run to measure.
 */
export const rollingEmptyFixture: RollingInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  date: TODAY,
  window: {
    from: '2026-04-15',
    from_label: '15 Apr 2026',
    to: TODAY,
    to_label: '13 May 2026',
    days: 28,
  },
  comparison: {
    from: '2026-03-18',
    from_label: '18 Mar 2026',
    to: '2026-04-14',
    to_label: '14 Apr 2026',
    days: 28,
  },
  distance: {
    current: value('0 km', 0),
    previous: value('0 km', 0),
    difference: value('0 km', 0),
  },
  duration: {
    current: value('0 m', 0),
    previous: value('0 m', 0),
    difference: value('0 m', 0),
  },
  sessions: { current: 0, previous: 0, difference: value('0', 0) },
  zones: {
    model: null,
    shares: null,
    distribution: null,
    unavailable: 'zones unavailable — no zone model has been read',
  },
  weekday: ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'].map(
    (weekday) => ({
      weekday,
      sessions: 0,
      distance: value('0 km', 0),
      duration: value('0 m', 0),
      share: null,
    }),
  ),
  daytime: [
    { key: 'morning', from: 6, to: 12 },
    { key: 'afternoon', from: 12, to: 17 },
    { key: 'evening', from: 17, to: 23 },
    { key: 'night', from: 23, to: 6 },
  ].map((bucket) => ({
    key: bucket.key,
    label: bucket.key.charAt(0).toUpperCase() + bucket.key.slice(1),
    from_hour: bucket.from,
    to_hour: bucket.to,
    sessions: 0,
    distance: value('0 km', 0),
    duration: value('0 m', 0),
  })),
  distances: [],
  profile: [],
  months: [],
  years: [],
}

// -------------------------------------------------------------- streaks

/**
 * The streaks: the router's answer.
 *
 * The current run of days and weeks, and the longest one of each. A
 * streak is a count and the days it covers — nothing more.
 */
export const streaksFixture = routerAnswers['/insights/streaks'] as StreakInsight

/** The streaks, with nothing in the store: there is no streak at all. */
export const streaksEmptyFixture: StreakInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  date: TODAY,
  days: { current: null, longest: null },
  weeks: { current: null, longest: null },
  months: { current: null, longest: null },
}

// ---------------------------------------------------------------- year

/**
 * The year: the twelve calendar months, the calendar of the days and
 * what changed.
 *
 * The answer comes from the router's own store - the same seed - so the
 * numbers on the page are the numbers the api computes.
 *
 * **The seeded store holds no FIT file**, so the days carry no mix: the
 * calendar is drawn in its neutral colour, which is the honest answer
 * and not a drawn one.
 */
export const yearFixture = routerAnswers['/insights/year'] as YearInsight

// ------------------------------------------------------------- heatmap

/**
 * The heatmap: a grid over the stored positions.
 *
 * The seeded store holds no FIT file, so **the router answers no cells**
 * and the app draws nothing — which is the honest answer. This grid is
 * the fixture set's own: a few cells around the repository's synthetic
 * anchor — the South Pacific, far from any coast and from any real route
 * — so that the surface can be seen and reviewed without a real run. The
 * cells mean nothing, and they name nothing.
 */
const SYNTHETIC_CELLS: HeatmapCell[] = [
  {
    x: 1818,
    y: 8218,
    south: -35.022,
    west: -140.044,
    north: -35.0,
    east: -140.022,
    runs: 4,
  },
  {
    x: 1819,
    y: 8218,
    south: -35.022,
    west: -140.022,
    north: -35.0,
    east: -140.0,
    runs: 2,
  },
  {
    x: 1820,
    y: 8218,
    south: -35.022,
    west: -140.0,
    north: -35.0,
    east: -139.978,
    runs: 1,
  },
  {
    x: 1818,
    y: 8219,
    south: -35.044,
    west: -140.044,
    north: -35.022,
    east: -140.022,
    runs: 3,
  },
  {
    x: 1819,
    y: 8219,
    south: -35.044,
    west: -140.022,
    north: -35.022,
    east: -140.0,
    runs: 1,
  },
]

/** The heatmap, with the synthetic grid on it. */
export const heatmapFixture: HeatmapInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  date: '2026-05-13',
  runs: 6,
  more_runs: false,
  cell_size: { text: '2 km', raw: 2003.4 },
  cells: SYNTHETIC_CELLS,
  extent: {
    south: -35.044,
    west: -140.044,
    north: -35.0,
    east: -139.978,
  },
  unavailable: null,
}

/** The heatmap, with nothing to draw: no run carries a position. */
export const heatmapEmptyFixture: HeatmapInsight = {
  generated_at: '2026-05-13T06:30:00Z',
  date: '2026-05-13',
  runs: 0,
  more_runs: false,
  cell_size: null,
  cells: [],
  extent: null,
  unavailable: 'no_positions',
}


// ------------------------------------------------------------- the film

/**
 * The run film: the run's own series, playable.
 *
 * **The seeded store holds no FIT file**, so the router answers the film
 * with the honest answer — "the run has no file to read" — and that is the
 * answer the fixture set carries (see `answers.json`).
 *
 * The one below is the state the surface is **reviewed** in: the run's
 * file read, with its samples, its ground, its splits and the athlete's
 * own zone model. The values are invented, like every other value in this
 * fixture set, and the words are the core's — "5:39 /km", "144 bpm",
 * "18 m".
 */


/**
 * The run's samples, as the core renders them.
 *
 * A minute apart, over the run the seed carries: twelve and a half
 * kilometres in an hour and seven minutes, easing from 5:39 to 5:13 a
 * kilometre while the heart rate climbs from 132 to 166.
 */
const FILM_SAMPLES: FilmSample[] = [
  {
    elapsed: { text: '0 m', raw: 0 },
    distance: { text: '0 km', raw: 0 },
    pace: { text: '5:39 /km', raw: 339.36 },
    heart_rate: { text: '132 bpm', raw: 132 },
    zone: 1,
  },
  {
    elapsed: { text: '1 m', raw: 60 },
    distance: { text: '0.2 km', raw: 177.03 },
    pace: { text: '5:39 /km', raw: 338.92 },
    heart_rate: { text: '133 bpm', raw: 133 },
    zone: 1,
  },
  {
    elapsed: { text: '2 m', raw: 120 },
    distance: { text: '0.4 km', raw: 354.3 },
    pace: { text: '5:38 /km', raw: 338.47 },
    heart_rate: { text: '134 bpm', raw: 134 },
    zone: 1,
  },
  {
    elapsed: { text: '3 m', raw: 180 },
    distance: { text: '0.5 km', raw: 531.81 },
    pace: { text: '5:38 /km', raw: 338.02 },
    heart_rate: { text: '135 bpm', raw: 135 },
    zone: 1,
  },
  {
    elapsed: { text: '4 m', raw: 240 },
    distance: { text: '0.7 km', raw: 709.55 },
    pace: { text: '5:38 /km', raw: 337.57 },
    heart_rate: { text: '136 bpm', raw: 136 },
    zone: 2,
  },
  {
    elapsed: { text: '5 m', raw: 300 },
    distance: { text: '0.9 km', raw: 887.53 },
    pace: { text: '5:37 /km', raw: 337.12 },
    heart_rate: { text: '136 bpm', raw: 136 },
    zone: 2,
  },
  {
    elapsed: { text: '6 m', raw: 360 },
    distance: { text: '1.1 km', raw: 1065.74 },
    pace: { text: '5:37 /km', raw: 336.67 },
    heart_rate: { text: '137 bpm', raw: 137 },
    zone: 2,
  },
  {
    elapsed: { text: '7 m', raw: 420 },
    distance: { text: '1.2 km', raw: 1244.19 },
    pace: { text: '5:36 /km', raw: 336.23 },
    heart_rate: { text: '138 bpm', raw: 138 },
    zone: 2,
  },
  {
    elapsed: { text: '8 m', raw: 480 },
    distance: { text: '1.4 km', raw: 1422.88 },
    pace: { text: '5:36 /km', raw: 335.78 },
    heart_rate: { text: '138 bpm', raw: 138 },
    zone: 2,
  },
  {
    elapsed: { text: '9 m', raw: 540 },
    distance: { text: '1.6 km', raw: 1601.81 },
    pace: { text: '5:35 /km', raw: 335.33 },
    heart_rate: { text: '139 bpm', raw: 139 },
    zone: 2,
  },
  {
    elapsed: { text: '10 m', raw: 600 },
    distance: { text: '1.8 km', raw: 1780.98 },
    pace: { text: '5:35 /km', raw: 334.88 },
    heart_rate: { text: '139 bpm', raw: 139 },
    zone: 2,
  },
  {
    elapsed: { text: '11 m', raw: 660 },
    distance: { text: '2 km', raw: 1960.38 },
    pace: { text: '5:34 /km', raw: 334.43 },
    heart_rate: { text: '140 bpm', raw: 140 },
    zone: 2,
  },
  {
    elapsed: { text: '12 m', raw: 720 },
    distance: { text: '2.1 km', raw: 2140.03 },
    pace: { text: '5:34 /km', raw: 333.99 },
    heart_rate: { text: '141 bpm', raw: 141 },
    zone: 2,
  },
  {
    elapsed: { text: '13 m', raw: 780 },
    distance: { text: '2.3 km', raw: 2319.92 },
    pace: { text: '5:34 /km', raw: 333.54 },
    heart_rate: { text: '141 bpm', raw: 141 },
    zone: 2,
  },
  {
    elapsed: { text: '14 m', raw: 840 },
    distance: { text: '2.5 km', raw: 2500.05 },
    pace: { text: '5:33 /km', raw: 333.09 },
    heart_rate: { text: '142 bpm', raw: 142 },
    zone: 2,
  },
  {
    elapsed: { text: '15 m', raw: 900 },
    distance: { text: '2.7 km', raw: 2680.43 },
    pace: { text: '5:33 /km', raw: 332.64 },
    heart_rate: { text: '142 bpm', raw: 142 },
    zone: 2,
  },
  {
    elapsed: { text: '16 m', raw: 960 },
    distance: { text: '2.9 km', raw: 2861.05 },
    pace: { text: '5:32 /km', raw: 332.19 },
    heart_rate: { text: '143 bpm', raw: 143 },
    zone: 2,
  },
  {
    elapsed: { text: '17 m', raw: 1020 },
    distance: { text: '3 km', raw: 3041.91 },
    pace: { text: '5:32 /km', raw: 331.74 },
    heart_rate: { text: '143 bpm', raw: 143 },
    zone: 2,
  },
  {
    elapsed: { text: '18 m', raw: 1080 },
    distance: { text: '3.2 km', raw: 3219.26 },
    pace: { text: '5:38 /km', raw: 338.3 },
    heart_rate: { text: '144 bpm', raw: 144 },
    zone: 2,
  },
  {
    elapsed: { text: '19 m', raw: 1140 },
    distance: { text: '3.4 km', raw: 3396.86 },
    pace: { text: '5:38 /km', raw: 337.86 },
    heart_rate: { text: '144 bpm', raw: 144 },
    zone: 2,
  },
  {
    elapsed: { text: '20 m', raw: 1200 },
    distance: { text: '3.6 km', raw: 3578.45 },
    pace: { text: '5:30 /km', raw: 330.4 },
    heart_rate: { text: '145 bpm', raw: 145 },
    zone: 2,
  },
  {
    elapsed: { text: '21 m', raw: 1260 },
    distance: { text: '3.8 km', raw: 3760.3 },
    pace: { text: '5:30 /km', raw: 329.95 },
    heart_rate: { text: '145 bpm', raw: 145 },
    zone: 2,
  },
  {
    elapsed: { text: '22 m', raw: 1320 },
    distance: { text: '3.9 km', raw: 3942.39 },
    pace: { text: '5:30 /km', raw: 329.5 },
    heart_rate: { text: '146 bpm', raw: 146 },
    zone: 2,
  },
  {
    elapsed: { text: '23 m', raw: 1380 },
    distance: { text: '4.1 km', raw: 4124.73 },
    pace: { text: '5:29 /km', raw: 329.05 },
    heart_rate: { text: '146 bpm', raw: 146 },
    zone: 2,
  },
  {
    elapsed: { text: '24 m', raw: 1440 },
    distance: { text: '4.3 km', raw: 4307.32 },
    pace: { text: '5:29 /km', raw: 328.61 },
    heart_rate: { text: '147 bpm', raw: 147 },
    zone: 2,
  },
  {
    elapsed: { text: '25 m', raw: 1500 },
    distance: { text: '4.5 km', raw: 4490.16 },
    pace: { text: '5:28 /km', raw: 328.16 },
    heart_rate: { text: '147 bpm', raw: 147 },
    zone: 2,
  },
  {
    elapsed: { text: '26 m', raw: 1560 },
    distance: { text: '4.7 km', raw: 4673.25 },
    pace: { text: '5:28 /km', raw: 327.71 },
    heart_rate: { text: '148 bpm', raw: 148 },
    zone: 2,
  },
  {
    elapsed: { text: '27 m', raw: 1620 },
    distance: { text: '4.9 km', raw: 4856.59 },
    pace: { text: '5:27 /km', raw: 327.26 },
    heart_rate: { text: '148 bpm', raw: 148 },
    zone: 2,
  },
  {
    elapsed: { text: '28 m', raw: 1680 },
    distance: { text: '5 km', raw: 5040.18 },
    pace: { text: '5:27 /km', raw: 326.81 },
    heart_rate: { text: '149 bpm', raw: 149 },
    zone: 2,
  },
  {
    elapsed: { text: '29 m', raw: 1740 },
    distance: { text: '5.2 km', raw: 5224.02 },
    pace: { text: '5:26 /km', raw: 326.37 },
    heart_rate: { text: '149 bpm', raw: 149 },
    zone: 2,
  },
  {
    elapsed: { text: '30 m', raw: 1800 },
    distance: { text: '5.4 km', raw: 5408.12 },
    pace: { text: '5:26 /km', raw: 325.92 },
    heart_rate: { text: '154 bpm', raw: 154 },
    zone: 3,
  },
  {
    elapsed: { text: '31 m', raw: 1860 },
    distance: { text: '5.6 km', raw: 5592.47 },
    pace: { text: '5:25 /km', raw: 325.47 },
    heart_rate: { text: '154 bpm', raw: 154 },
    zone: 3,
  },
  {
    elapsed: { text: '32 m', raw: 1920 },
    distance: { text: '5.8 km', raw: 5777.07 },
    pace: { text: '5:25 /km', raw: 325.02 },
    heart_rate: { text: '151 bpm', raw: 151 },
    zone: 3,
  },
  {
    elapsed: { text: '33 m', raw: 1980 },
    distance: { text: '6 km', raw: 5961.93 },
    pace: { text: '5:25 /km', raw: 324.57 },
    heart_rate: { text: '151 bpm', raw: 151 },
    zone: 3,
  },
  {
    elapsed: { text: '34 m', raw: 2040 },
    distance: { text: '6.1 km', raw: 6147.04 },
    pace: { text: '5:24 /km', raw: 324.12 },
    heart_rate: { text: '152 bpm', raw: 152 },
    zone: 3,
  },
  {
    elapsed: { text: '35 m', raw: 2100 },
    distance: { text: '6.3 km', raw: 6332.41 },
    pace: { text: '5:24 /km', raw: 323.68 },
    heart_rate: { text: '152 bpm', raw: 152 },
    zone: 3,
  },
  {
    elapsed: { text: '36 m', raw: 2160 },
    distance: { text: '6.5 km', raw: 6518.04 },
    pace: { text: '5:23 /km', raw: 323.23 },
    heart_rate: { text: '153 bpm', raw: 153 },
    zone: 3,
  },
  {
    elapsed: { text: '37 m', raw: 2220 },
    distance: { text: '6.7 km', raw: 6703.93 },
    pace: { text: '5:23 /km', raw: 322.78 },
    heart_rate: { text: '153 bpm', raw: 153 },
    zone: 3,
  },
  {
    elapsed: { text: '38 m', raw: 2280 },
    distance: { text: '6.9 km', raw: 6890.07 },
    pace: { text: '5:22 /km', raw: 322.33 },
    heart_rate: { text: '154 bpm', raw: 154 },
    zone: 3,
  },
  {
    elapsed: { text: '39 m', raw: 2340 },
    distance: { text: '7.1 km', raw: 7076.47 },
    pace: { text: '5:22 /km', raw: 321.88 },
    heart_rate: { text: '154 bpm', raw: 154 },
    zone: 3,
  },
  {
    elapsed: { text: '40 m', raw: 2400 },
    distance: { text: '7.3 km', raw: 7263.14 },
    pace: { text: '5:21 /km', raw: 321.43 },
    heart_rate: { text: '155 bpm', raw: 155 },
    zone: 3,
  },
  {
    elapsed: { text: '41 m', raw: 2460 },
    distance: { text: '7.5 km', raw: 7450.06 },
    pace: { text: '5:21 /km', raw: 320.99 },
    heart_rate: { text: '155 bpm', raw: 155 },
    zone: 3,
  },
  {
    elapsed: { text: '42 m', raw: 2520 },
    distance: { text: '7.6 km', raw: 7637.25 },
    pace: { text: '5:21 /km', raw: 320.54 },
    heart_rate: { text: '155 bpm', raw: 155 },
    zone: 3,
  },
  {
    elapsed: { text: '43 m', raw: 2580 },
    distance: { text: '7.8 km', raw: 7824.69 },
    pace: { text: '5:20 /km', raw: 320.09 },
    heart_rate: { text: '156 bpm', raw: 156 },
    zone: 3,
  },
  {
    elapsed: { text: '44 m', raw: 2640 },
    distance: { text: '8 km', raw: 8008.38 },
    pace: { text: '5:27 /km', raw: 326.65 },
    heart_rate: { text: '156 bpm', raw: 156 },
    zone: 3,
  },
  {
    elapsed: { text: '45 m', raw: 2700 },
    distance: { text: '8.2 km', raw: 8196.35 },
    pace: { text: '5:19 /km', raw: 319.19 },
    heart_rate: { text: '157 bpm', raw: 157 },
    zone: 3,
  },
  {
    elapsed: { text: '46 m', raw: 2760 },
    distance: { text: '8.4 km', raw: 8384.59 },
    pace: { text: '5:19 /km', raw: 318.75 },
    heart_rate: { text: '157 bpm', raw: 157 },
    zone: 3,
  },
  {
    elapsed: { text: '47 m', raw: 2820 },
    distance: { text: '8.6 km', raw: 8573.09 },
    pace: { text: '5:18 /km', raw: 318.3 },
    heart_rate: { text: '158 bpm', raw: 158 },
    zone: 3,
  },
  {
    elapsed: { text: '48 m', raw: 2880 },
    distance: { text: '8.8 km', raw: 8761.86 },
    pace: { text: '5:18 /km', raw: 317.85 },
    heart_rate: { text: '158 bpm', raw: 158 },
    zone: 3,
  },
  {
    elapsed: { text: '49 m', raw: 2940 },
    distance: { text: '9 km', raw: 8950.9 },
    pace: { text: '5:17 /km', raw: 317.4 },
    heart_rate: { text: '158 bpm', raw: 158 },
    zone: 3,
  },
  {
    elapsed: { text: '50 m', raw: 3000 },
    distance: { text: '9.1 km', raw: 9140.2 },
    pace: { text: '5:17 /km', raw: 316.95 },
    heart_rate: { text: '159 bpm', raw: 159 },
    zone: 3,
  },
  {
    elapsed: { text: '51 m', raw: 3060 },
    distance: { text: '9.3 km', raw: 9329.77 },
    pace: { text: '5:17 /km', raw: 316.5 },
    heart_rate: { text: '159 bpm', raw: 159 },
    zone: 3,
  },
  {
    elapsed: { text: '52 m', raw: 3120 },
    distance: { text: '9.5 km', raw: 9519.61 },
    pace: { text: '5:16 /km', raw: 316.06 },
    heart_rate: { text: '160 bpm', raw: 160 },
    zone: 3,
  },
  {
    elapsed: { text: '53 m', raw: 3180 },
    distance: { text: '9.7 km', raw: 9709.72 },
    pace: { text: '5:16 /km', raw: 315.61 },
    heart_rate: { text: '160 bpm', raw: 160 },
    zone: 3,
  },
  {
    elapsed: { text: '54 m', raw: 3240 },
    distance: { text: '9.9 km', raw: 9900.1 },
    pace: { text: '5:15 /km', raw: 315.16 },
    heart_rate: { text: '161 bpm', raw: 161 },
    zone: 3,
  },
  {
    elapsed: { text: '55 m', raw: 3300 },
    distance: { text: '10.1 km', raw: 10090.75 },
    pace: { text: '5:15 /km', raw: 314.71 },
    heart_rate: { text: '161 bpm', raw: 161 },
    zone: 3,
  },
  {
    elapsed: { text: '56 m', raw: 3360 },
    distance: { text: '10.3 km', raw: 10281.67 },
    pace: { text: '5:14 /km', raw: 314.26 },
    heart_rate: { text: '161 bpm', raw: 161 },
    zone: 3,
  },
  {
    elapsed: { text: '57 m', raw: 3420 },
    distance: { text: '10.5 km', raw: 10472.87 },
    pace: { text: '5:14 /km', raw: 313.81 },
    heart_rate: { text: '162 bpm', raw: 162 },
    zone: 3,
  },
  {
    elapsed: { text: '58 m', raw: 3480 },
    distance: { text: '10.7 km', raw: 10664.34 },
    pace: { text: '5:13 /km', raw: 313.37 },
    heart_rate: { text: '162 bpm', raw: 162 },
    zone: 3,
  },
  {
    elapsed: { text: '59 m', raw: 3540 },
    distance: { text: '10.9 km', raw: 10856.08 },
    pace: { text: '5:13 /km', raw: 312.92 },
    heart_rate: { text: '163 bpm', raw: 163 },
    zone: 3,
  },
  {
    elapsed: { text: '1 h', raw: 3600 },
    distance: { text: '11 km', raw: 11048.1 },
    pace: { text: '5:12 /km', raw: 312.47 },
    heart_rate: { text: '163 bpm', raw: 163 },
    zone: 3,
  },
  {
    elapsed: { text: '1 h 1 m', raw: 3660 },
    distance: { text: '11.2 km', raw: 11240.4 },
    pace: { text: '5:12 /km', raw: 312.02 },
    heart_rate: { text: '164 bpm', raw: 164 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 2 m', raw: 3720 },
    distance: { text: '11.4 km', raw: 11432.97 },
    pace: { text: '5:12 /km', raw: 311.57 },
    heart_rate: { text: '164 bpm', raw: 164 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 3 m', raw: 3780 },
    distance: { text: '11.6 km', raw: 11625.82 },
    pace: { text: '5:11 /km', raw: 311.13 },
    heart_rate: { text: '164 bpm', raw: 164 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 4 m', raw: 3840 },
    distance: { text: '11.8 km', raw: 11818.94 },
    pace: { text: '5:11 /km', raw: 310.68 },
    heart_rate: { text: '165 bpm', raw: 165 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 5 m', raw: 3900 },
    distance: { text: '12 km', raw: 12012.35 },
    pace: { text: '5:10 /km', raw: 310.23 },
    heart_rate: { text: '165 bpm', raw: 165 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 6 m', raw: 3960 },
    distance: { text: '12.2 km', raw: 12206.03 },
    pace: { text: '5:10 /km', raw: 309.78 },
    heart_rate: { text: '166 bpm', raw: 166 },
    zone: 4,
  },
  {
    elapsed: { text: '1 h 7 m', raw: 4020 },
    distance: { text: '12.4 km', raw: 12400 },
    pace: { text: '5:09 /km', raw: 309.33 },
    heart_rate: { text: '166 bpm', raw: 166 },
    zone: 4,
  },
]

const FILM_GROUND: FilmGround[] = [
  { offset_s: 0, elevation: { text: '118 m', raw: 118 } },
  { offset_s: 300, elevation: { text: '121 m', raw: 121 } },
  { offset_s: 600, elevation: { text: '124 m', raw: 124 } },
  { offset_s: 900, elevation: { text: '127 m', raw: 127 } },
  { offset_s: 1200, elevation: { text: '131 m', raw: 131 } },
  { offset_s: 1500, elevation: { text: '134 m', raw: 134 } },
  { offset_s: 1800, elevation: { text: '137 m', raw: 137 } },
  { offset_s: 2100, elevation: { text: '140 m', raw: 140 } },
  { offset_s: 2400, elevation: { text: '143 m', raw: 143 } },
  { offset_s: 2700, elevation: { text: '149 m', raw: 149 } },
  { offset_s: 3000, elevation: { text: '152 m', raw: 152 } },
  { offset_s: 3300, elevation: { text: '155 m', raw: 155 } },
  { offset_s: 3600, elevation: { text: '159 m', raw: 159 } },
  { offset_s: 3900, elevation: { text: '162 m', raw: 162 } },
]

const FILM_SPLITS: FilmSplit[] = [
  {
    index: 1,
    offset_s: 0,
    distance: { text: '3.2 km', raw: 3219.26 },
    duration: { text: '18 m', raw: 1080 },
    pace: { text: '5:35 /km', raw: 335.48 },
    heart_rate: { text: '144 bpm', raw: 144 },
  },
  {
    index: 2,
    offset_s: 1080,
    distance: { text: '3.1 km', raw: 3113.15 },
    duration: { text: '17 m', raw: 1020 },
    pace: { text: '5:28 /km', raw: 327.64 },
    heart_rate: { text: '152 bpm', raw: 152 },
  },
  {
    index: 3,
    offset_s: 2100,
    distance: { text: '3 km', raw: 2997.36 },
    duration: { text: '16 m', raw: 960 },
    pace: { text: '5:20 /km', raw: 320.28 },
    heart_rate: { text: '159 bpm', raw: 159 },
  },
  {
    index: 4,
    offset_s: 3060,
    distance: { text: '3.1 km', raw: 3070.23 },
    duration: { text: '16 m', raw: 960 },
    pace: { text: '5:13 /km', raw: 312.68 },
    heart_rate: { text: '166 bpm', raw: 166 },
  },
]


/**
 * The run's film: the router's answer for a store whose run has a file.
 *
 * The model is the seeded store's own — the watch's configuration, read
 * on the tenth of May — and the zones are its five bands.
 */
export const filmFixture: RunFilm = {
  activity_id: 101,
  local_date: '2026-05-10',
  basis: 'samples',
  model: {
    sport: 'DEFAULT',
    training_method: 'HR_MAX',
    max_heart_rate: { text: '178 bpm', raw: 178 },
    floors: {
      zone1: { text: '106 bpm', raw: 106 },
      zone2: { text: '121 bpm', raw: 121 },
      zone3: { text: '135 bpm', raw: 135 },
      zone4: { text: '149 bpm', raw: 149 },
      zone5: { text: '163 bpm', raw: 163 },
    },
    read_at: '2026-05-10T06:30:00Z',
    read_on: '10 May 2026',
    source: 'your zone model · HR_MAX · max heart rate 178 bpm · read on 10 May 2026',
  },
  zones: [
    { label: 'Z1', range: '106-121 bpm', from_bpm: 106, to_bpm: 121 },
    { label: 'Z2', range: '121-135 bpm', from_bpm: 121, to_bpm: 135 },
    { label: 'Z3', range: '135-149 bpm', from_bpm: 135, to_bpm: 149 },
    { label: 'Z4', range: '149-163 bpm', from_bpm: 149, to_bpm: 163 },
    { label: 'Z5', range: '163-178 bpm', from_bpm: 163, to_bpm: 178 },
  ],
  outside_label: 'outside your zones',
  samples: FILM_SAMPLES,
  ground: FILM_GROUND,
  splits: FILM_SPLITS,
  unavailable: null,
  zones_unavailable: null,
  generated_at: '2026-05-13T06:30:00Z',
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
 * The zone model, as the seed carries it.
 *
 * The watch's own configuration, read from Garmin and stored with the day
 * it was read. The api's test seeds a store from the fixture set, so the
 * router answers with a model it can name — and with the day it was read.
 *
 * The values are invented: the repository is public, and no athlete's real
 * floors belong in a fixture.
 */
export interface SeedZoneModel {
  sport: string
  training_method: string
  max_heart_rate: number | null
  zone1_floor: number
  zone2_floor: number
  zone3_floor: number
  zone4_floor: number
  zone5_floor: number
  read_at: string
}

export const zoneModelFixture: SeedZoneModel = {
  sport: 'DEFAULT',
  training_method: 'HR_MAX',
  max_heart_rate: 178,
  zone1_floor: 106,
  zone2_floor: 121,
  zone3_floor: 135,
  zone4_floor: 149,
  zone5_floor: 163,
  read_at: '2026-05-10T06:30:00Z',
}

/**
 * The heart-rate zones of a run.
 *
 * The router's answer for the seeded store: the athlete's stored zone
 * model — its sport, its method, the maximum heart rate it was built on,
 * its five floors and **the day it was read** — and the honest answer for
 * the run itself, which has no heart-rate samples to read.
 *
 * The bands themselves are the api's: the app draws what it is given.
 */
/**
 * The zones of a run in a store with **no zone model**.
 *
 * The seeded store holds one — the athlete's watch configuration — and the
 * router answers from it. This is the other state: no model has been read,
 * so the run is not calibrated, there is no value at all, and the core says
 * why.
 */
/**
 * The heart-rate zones of a run: the router's answer.
 *
 * The seeded store holds the athlete's own zone model — the watch's
 * configuration — and the run carries no FIT file, so the router names
 * the model and says that the run has no heart-rate samples to read. The
 * bands themselves are the api's: the app draws what it is given.
 */
export const zonesFixture: FitnessZones = routerAnswers['/fitness/zones'] as FitnessZones

/**
 * The zones with no model in the store.
 *
 * The run is not calibrated, so there is no value, and the core says why
 * — "no zone model has been read".
 */
export const zonesUncalibratedFixture: FitnessZones = {
  basis: null,
  model: null,
  zones: [],
  outside: null,
  unavailable: 'zones unavailable — no zone model has been read',
}

/**
 * The zones of a run whose heart-rate samples have been read.
 *
 * The seeded store holds no FIT file, so the router's own answer is the
 * honest one: the run has no heart-rate samples to read. This is the
 * other state — the samples read — and it is the state the cockpit's
 * **zone ribbon** is reviewed in.
 *
 * The times are invented, like every other value in this fixture set, and
 * the model is the router's own: the athlete's stored zone model, with
 * the day it was read.
 */
export const zonesWithTimeFixture: FitnessZones = {
  ...zonesFixture,
  basis: 'moving',
  zones: [
    { label: 'Z1', range: '106-121 bpm', from_bpm: 106, to_bpm: 121, time: value('6 m', 360) },
    { label: 'Z2', range: '121-135 bpm', from_bpm: 121, to_bpm: 135, time: value('12 m', 720) },
    {
      label: 'Z3',
      range: '135-149 bpm',
      from_bpm: 135,
      to_bpm: 149,
      time: value('18 m', 1080),
    },
    { label: 'Z4', range: '149-163 bpm', from_bpm: 149, to_bpm: 163, time: value('9 m', 540) },
    { label: 'Z5', range: '163-178 bpm', from_bpm: 163, to_bpm: 178, time: value('0 m', 0) },
  ],
  outside: value('2 m', 120),
  unavailable: null,
}

/**
 * The fingerprint of a run whose file has been read.
 *
 * The seeded store holds no FIT file, so the router's answer for the
 * fixture's run is the honest one: there is nothing to measure, and the
 * core says why — "the run has no file to read". That answer is the
 * default fixture.
 *
 * This one is the state the card is **reviewed** in: the run's file read,
 * and the observations it holds. The values are invented, like every other
 * value in this fixture set, and the words are the core's — "163.0 spm",
 * "+5 bpm", "the run's file carries no stride length".
 *
 * There is no score in it, and no composite: the rows are the run's own
 * measurements, and the reader decides what they mean.
 */
export const fingerprintFixture: Fingerprint = {
  activity_id: 101,
  local_date: '2026-05-10',
  basis: 'laps and moving samples',
  laps: 6,
  observations: [
    { key: 'cadence', label: 'Cadence', value: { text: '163.0 spm', raw: 163 }, reason: null },
    {
      key: 'cadence_variability',
      label: 'Cadence variability',
      value: { text: '2.3 %', raw: 2.3 },
      reason: null,
    },
    {
      key: 'efficiency',
      label: 'Efficiency factor',
      value: { text: '4.05 m/s per unit of max heart rate', raw: 4.05 },
      reason: null,
    },
    {
      key: 'pace_variability',
      label: 'Pace variability',
      value: { text: '1.4 %', raw: 1.4 },
      reason: null,
    },
    {
      key: 'cadence_drift',
      label: 'Cadence drift',
      value: { text: '−1.4 %', raw: -1.4 },
      reason: null,
    },
    {
      key: 'heart_rate_drift',
      label: 'Heart rate drift',
      value: { text: '+5 bpm', raw: 5 },
      reason: null,
    },
    {
      key: 'pace_drift',
      label: 'Pace drift',
      value: { text: '−4 s/km', raw: -4 },
      reason: null,
    },
    {
      key: 'decoupling',
      label: 'Aerobic decoupling',
      value: { text: '3.9 %', raw: 3.9 },
      reason: null,
    },
    {
      key: 'stride_length',
      label: 'Stride length',
      value: null,
      reason: "the run's file carries no stride length",
    },
  ],
  zones: {
    model: null,
    shares: [],
    outside: null,
    unavailable: null,
    source: null,
  },
  unavailable: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/** The service's own health — and the version, for About. */
export const healthFixture = routerAnswers['/health'] as Health

/** The service's health, with no plan: the router's answer for the baseline seed. */
export const baselineHealthFixture = baselineAnswers['/health'] as Health

/**
 * The settings.
 *
 * A goal of forty kilometres a week, no pause, and a database of a
 * couple of megabytes. The goal is a number the athlete wrote; the app
 * never invents one.
 */
/** The settings: the router's answer. */
export const settingsFixture = routerAnswers['/settings'] as SettingsState

/** The settings, with no plan: the router's answer for the baseline seed. */
export const baselineSettingsFixture = baselineAnswers['/settings'] as SettingsState

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
 * The router's shape is frozen: **three vitals, one per kind**, and when
 * there is no data each one carries a null value and the status the core
 * writes for a comparison it cannot make. The rest line carries the same
 * status. **No invented numbers** — the history is empty, not zero.
 */
export const vitalsEmptyFixture: VitalsSnapshot = {
  date: TODAY,
  generated_at: '2026-05-13T06:30:00Z',
  rest: { label: 'Rest', line: 'not enough data for a comparison yet' },
  vitals: [
    {
      key: 'sleep',
      label: 'Sleep',
      value_text: null,
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
    {
      key: 'hrv',
      label: 'HRV',
      value_text: null,
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
    {
      key: 'resting_hr',
      label: 'Resting HR',
      value_text: null,
      direction_text: null,
      deviation_text: null,
      travel_text: null,
      status_text: 'not enough data for a comparison yet',
      band: null,
    },
  ],
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
  rest: { label: 'Rest', line: 'not enough data for a comparison yet' },
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
  week: {
    distance: value('0 km', 0),
    sessions: 0,
    days: [],
    previous: {
      from: '2026-05-04',
      to: '2026-05-06',
      distance: value('0 km', 0),
      sessions: 0,
    },
    difference: value('0 km', 0),
  },
}

/** The feed, with nothing in it. */
export const activitiesEmptyFixture: ActivityListResponse = {
  activities: [],
  next_cursor: null,
  generated_at: '2026-05-13T06:30:00Z',
}

/** Progress, with too little history to draw a trend. */
export const progressNotEnoughFixture: ProgressInsight = {
  period: progressFixture.period,
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
  // The number over the same window: this variant is about the headline,
  // not about the days.
  eddington: progressFixture.eddington,
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
 * | `uncalibrated` | no zone model has been read, so the bands have no value |
 * | `zones` | the run's heart-rate samples have been read, so the zone ribbon has its times |
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
      // No plan block: the router inferred the baseline from the
      // athlete's history, and Today shows what it inferred.
      if (variant === 'baseline') return baselineTodayFixture
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
    case '/physiology/sleep': {
      if (['empty', 'fresh', 'nowellness'].includes(variant))
        return { days: [], generated_at: vitalsFixture.generated_at }
      const answer = routerAnswers['/physiology/sleep'] as SleepDays
      return {
        ...answer,
        days: answer.days.filter(
          (day) =>
            (!params.from || day.date >= params.from) && (!params.to || day.date <= params.to),
        ),
      }
    }
    case '/activities': {
      const answer =
        variant === 'empty' || variant === 'fresh' ? activitiesEmptyFixture : activitiesFixture
      return {
        ...answer,
        activities: answer.activities
          .filter(
            (activity) =>
              (!params.since || activity.local_date >= params.since) &&
              (!params.until || activity.local_date <= params.until),
          )
          .map((activity) => (variant === 'route' ? withRoute(activity) : activity)),
      }
    }
    case '/calendar': {
      const saved = routerAnswers['/calendar'] as ActivityCalendar
      const answer = ['empty', 'fresh'].includes(variant)
        ? {
            ...saved,
            days: saved.days.map((day) => ({
              ...day,
              activities: [],
              distance: null,
              duration: null,
            })),
          }
        : saved
      if (
        (!params.year || Number(params.year) === answer.year) &&
        (!params.month || Number(params.month) === answer.month)
      )
        return answer
      const year = Number(params.year ?? answer.year)
      const month = Number(params.month ?? answer.month)
      const count = new Date(Date.UTC(year, month, 0)).getUTCDate()
      return {
        year,
        month,
        generated_at: answer.generated_at,
        days: Array.from({ length: count }, (_, index) => ({
          date: `${year}-${String(month).padStart(2, '0')}-${String(index + 1).padStart(2, '0')}`,
          day: index + 1,
          distance: null,
          duration: null,
          activities: [],
        })),
      }
    }
    case '/activities/101':
      if (variant === 'route' || variant === 'film') {
        return { ...activityDetailFixture, geo_points: SYNTHETIC_ROUTE }
      }
      return activityDetailFixture
    case '/activities/101/film':
      // The film's own series: the state the surface is reviewed in. The
      // router's answer for the seeded store — the honest one, with no
      // file to read — is in `answers.json`.
      if (variant === 'film') return filmFixture
      return routerAnswers['/activities/101/film'] as RunFilm

    case '/activities/101/update':
      // The stored update of a run: the router's answer. The seeded store
      // holds none, so the run has none.
      return routerAnswers['/activities/101/update'] as RunUpdateResponse
    case '/activities/101/update/payload':
      // The payload a model may read: the router's answer.
      return routerAnswers['/activities/101/update/payload'] as UpdatePayloadResponse
    case '/update/settings':
      return routerAnswers['/update/settings'] as UpdateSettings
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
      // No plan block: the router's progress has no adherence card, and
      // the counts are the ones the history produced.
      if (variant === 'baseline')
        return (
          baselineAnswers[`/insights/progress?weeks=${params.weeks}`] ?? baselineProgressFixture
        )
      return routerAnswers[`/insights/progress?weeks=${params.weeks}`] ?? progressFixture
    case '/insights/pace':
      // The seeded store holds no file, so the router answers no bands:
      // the window holds no moving samples, and the screen says so.
      return routerAnswers[`/insights/pace?weeks=${params.weeks}`] ?? paceFixture
    case '/insights/rolling': {
      // Nothing in the store: the window holds no distance and the
      // tables hold no rows.
      if (variant === 'empty' || variant === 'fresh') return rollingEmptyFixture
      // No zone model in the store: the split has no value, and the card
      // says why.
      if (variant === 'noload') {
        return {
          ...(routerAnswers[`/insights/rolling?weeks=${params.weeks}`] ?? rollingFixture),
          zones: {
            model: null,
            shares: null,
            distribution: null,
            unavailable: 'zones unavailable — no zone model has been read',
          },
        }
      }
      return (
        routerAnswers[`/insights/rolling?weeks=${params.weeks}`] ?? rollingFixture
      )
    }
    case '/insights/streaks':
      if (variant === 'empty' || variant === 'fresh') return streaksEmptyFixture
      return streaksFixture
    case '/insights/load':
      // No zone model in the store: the load has no value, and the card
      // says why.
      if (variant === 'noload') {
        return { ...loadFixture, load: null, scenario: null, unavailable: 'no_zone_model' }
      }
      return routerAnswers[`/insights/load?weeks=${params.weeks}`] ?? loadFixture
    case '/insights/year': {
      // Nothing in the store: the months hold no numbers and the days
      // hold no distance.
      if (variant === 'empty' || variant === 'fresh') {
        return {
          ...yearFixture,
          months: yearFixture.months.map((month) => ({
            ...month,
            sessions: 0,
            distance: null,
            duration: null,
            elevation: null,
            longest: null,
            form: null,
            weeks: [],
            mix: [],
            high: null,
          })),
          days: yearFixture.days.map((day) => ({
            ...day,
            sessions: 0,
            distance: null,
            high: null,
          })),
        }
      }
      return yearFixture
    }
    case '/insights/heatmap':
      // No run carries a position: there is no grid to draw, and the
      // screen says so.
      if (variant === 'empty' || variant === 'fresh') return heatmapEmptyFixture
      return heatmapFixture
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
      if (variant === 'uncalibrated') return zonesUncalibratedFixture
      // The run's heart-rate samples have been read: the zone ribbon has
      // the times to draw.
      if (variant === 'zones') return zonesWithTimeFixture
      return zonesFixture
    case '/fitness/fingerprint':
      // The run's file has been read in the `route` variant — the state the
      // card is reviewed in. The default answer is the router's: the
      // seeded store holds no file, so there is nothing to measure.
      if (variant === 'route') return fingerprintFixture
      return routerAnswers['/fitness/fingerprint'] as Fingerprint
    case '/health':
      if (variant === 'baseline') return baselineHealthFixture
      return healthFixture
    case '/auth/status':
      if (variant === 'empty' || variant === 'fresh') return connectionEmptyFixture
      if (variant === 'connected') return connectionHeldFixture
      return connectionFixture
    case '/settings':
      if (variant === 'empty' || variant === 'fresh') return settingsEmptyFixture
      if (variant === 'paused') return settingsPausedFixture
      if (variant === 'baseline') return baselineSettingsFixture
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
