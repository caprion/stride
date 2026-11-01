/**
 * The shapes the app consumes.
 *
 * The route table is frozen (`docs/contract/api-paths.md`); the shapes below
 * are the app's own reading of it. Every one of them is typed here and
 * nowhere else, and the fixtures in `src/fixtures/` are the same types, so
 * a screen can be rendered and tested without a running backend.
 *
 * Since core-contract §8 the *domain* sentences arrive already rendered:
 * the direction word, the arithmetic, the travel, the Rest line and the
 * pattern sentences are written by `stride-core` and travel as text. The
 * app renders the chrome around them. A sentence with holes in it is the
 * core's business, not the app's.
 */

import type { CatalogueKey } from '../i18n'

/** The three vitals the engine reports. */
export type VitalKey = 'sleep' | 'hrv' | 'resting_hr'

/**
 * The band a direction was measured against.
 *
 * Numbers, not a sentence: the chart draws it, and the core computed it.
 */
export interface VitalBand {
  /** The lower edge: median − 1.5 × MAD. */
  lower: number
  /** The upper edge: median + 1.5 × MAD. */
  upper: number
  /** How many nights the band was measured from. */
  nights: number
}

/**
 * One vital, as the core renders it.
 *
 * Every sentence here is the core's: `value_text` is "6 h 10 m",
 * `direction_text` is a word from the closed set, `deviation_text` is
 * "1 h 10 m less than your usual" and `travel_text` is "6th night down".
 */
export interface VitalReading {
  /** Which vital this is. */
  key: VitalKey
  /** The catalogue's label: "Sleep", "HRV", "Resting HR". */
  label: string
  /** The value, rendered: "6 h 10 m", "48 ms", "63 bpm". */
  value_text: string | null
  /** The direction word: "could be better". */
  direction_text: string | null
  /** The arithmetic: "1 h 10 m less than your usual". */
  deviation_text: string | null
  /** The travel: "6th night down". */
  travel_text: string | null
  /** What the row says when there is no comparison to make. */
  status_text: string | null
  /** The band, for the chart. */
  band: VitalBand | null
}

/**
 * The Rest row, as the core renders it.
 *
 * The line is the count and the direction — "could be better · 2 of 3
 * signals below your usual" — and the label is the name of the row.
 */
export interface RestReading {
  /** The catalogue's label: "Rest". */
  label: string
  /** The line, already written. */
  line: string | null
}

/** One night of history, for the charts on Body. */
export interface VitalHistoryPoint {
  date: string
  value: number | null
}

/**
 * A pattern: a fact about the past, with its sample size.
 *
 * The sentence is the core's — "your HRV has been below your usual for 3
 * nights" — and it always carries its sample size, because the core only
 * states a pattern from three comparable days or more.
 */
export interface PatternReading {
  id: string
  /** The sentence, already written. */
  text: string
  /** How many comparable days it was read from. */
  sample_size: number
}

/** The vitals and their directions — the same snapshot Today and Body read. */
export interface VitalsSnapshot {
  date: string
  generated_at: string
  /** The Rest row. */
  rest: RestReading
  vitals: VitalReading[]
  /** The history, for the charts. Numbers. */
  history: Record<VitalKey, VitalHistoryPoint[]>
  patterns: PatternReading[]
}

/** A session from the athlete's own plan. */
export interface PlanSession {
  date: string
  /** The plan's own vocabulary: easy, long, tempo, rest, strength. */
  session_type: string
  title: string | null
  /** The session, already rendered: "Today — 10 km · easy". */
  sentence: string
  target_distance_m: number | null
  actual_distance_m: number | null
  status: 'done' | 'planned' | 'not_done'
  source: 'baseline' | 'prescribed' | 'typed'
}

/** The plan's active block for the current week. */
export interface PlanWeek {
  block_name: string
  source: 'baseline' | 'prescribed' | 'typed'
  week_start: string
  week_end: string
  target_km: number | null
  sessions: PlanSession[]
  adherence: { done: number; planned: number } | null
}

/** An activity, as the feed shows it. */
export interface ActivitySummary {
  id: number
  name: string
  sport_type: string
  start_time: string
  local_date: string
  distance_m: number
  moving_time_s: number
  elapsed_time_s: number
  avg_hr: number | null
  max_hr: number | null
  avg_cadence: number | null
  elevation_gain_m: number | null
  pace_s_per_km: number | null
  /** The one sentence about what the run meant. */
  story: string | null
  geo_points: [number, number][]
}

/** The feed, windowed by date. */
export interface ActivityListResponse {
  activities: ActivitySummary[]
  next_cursor: string | null
  generated_at: string
}

/** One run, in full. */
export interface ActivityDetail extends ActivitySummary {
  splits: { index: number; distance_m: number; duration_s: number; avg_hr: number | null }[]
  elevation: { distance_m: number; elevation_m: number }[]
  samples: { offset_s: number; pace_s_per_km: number | null; hr: number | null }[]
  plan_line: { actual_distance_m: number; target_distance_m: number } | null
  note: string | null
}

/** What the Today screen reads. */
export interface TodayInsight {
  generated_at: string
  date: string
  /** The plan's own session for today, when there is one. */
  planned: PlanSession | null
  /** The plan's other sessions — unranked, unlinked. */
  options: PlanSession[]
  /** The last run, when there is one. */
  last_run: ActivitySummary | null
  /**
   * The distance comparison, when there is one.
   *
   * The core states it, and states it only when the gap from the target is
   * beyond 10%: "2.4 km more than the 10 km target". Within 10% it is
   * absent — a 2% difference is not news.
   */
  distance_comparison: string | null
  /** This week's running, in metres. */
  week: { distance_m: number; sessions: number }
}

/** One row of the metric diff. */
export interface CompareMetric {
  key: string
  /** The catalogue key that names the row. */
  label_key: CatalogueKey
  /** How the value is rendered: distance, duration, pace, heart rate. */
  format: 'distance' | 'duration' | 'pace' | 'hr'
  a: number | null
  b: number | null
  /** B − A, in the metric's own unit. */
  delta: number | null
}

/** The identity of a run, in a comparison. */
export interface CompareRun {
  id: number
  label: string
  date: string
  distance_m: number
  story: string | null
}

/** The comparison itself. */
export interface CompareResponse {
  /** The basis, already written: "Same distance, 3 weeks apart". */
  basis_text: string
  metrics: CompareMetric[]
  a: CompareRun
  b: CompareRun
  series: {
    a: { offset_s: number; pace_s_per_km: number | null; hr: number | null }[]
    b: { offset_s: number; pace_s_per_km: number | null; hr: number | null }[]
  }
}

/** A run that can be compared against. */
export interface CompareCandidate {
  id: number
  label: string
  date: string
  distance_m: number
}

/** The Progress screen's data. */
export interface ProgressInsight {
  generated_at: string
  /** The headline, already written: "fitter than 8 weeks ago". */
  headline: string
  /** The evidence under it: "same heart rate, 24 s/km faster". */
  headline_note: string | null
  weekly_volume: { week_start: string; distance_m: number }[]
  typical_week_m: number | null
  pace_at_hr: { week_start: string; pace_s_per_km: number | null; hr: number | null }[]
  bests: { key: CatalogueKey; value: string; date: string }[]
  consistency: { week_start: string; sessions: number }[]
  adherence: { done: number; planned: number } | null
}

/** The connection and sync state. */
export interface SyncStatus {
  last_sync_at: string | null
  last_error: string | null
  activity_count: number
}

/** The health of the sync window. */
export interface HealthDaily {
  date: string
  generated_at: string
  stale: boolean
}
