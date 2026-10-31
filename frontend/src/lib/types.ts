/**
 * The shapes the app consumes.
 *
 * The route table is frozen (`docs/contract/api-paths.md`); the shapes below
 * are the app's own reading of it. Every one of them is typed here and
 * nowhere else, and the fixtures in `src/fixtures/` are the same types, so
 * a screen can be rendered and tested without a running backend.
 */

import type { CatalogueKey } from '../i18n'

/** The closed direction set, as the core hands it over. */
export type Direction = 'better than usual' | 'about usual' | 'could be better'

/** Which way a vital has moved over the last few days. */
export type Travel = 'up' | 'down' | 'steady'

/** The three vitals the engine reports. */
export type VitalKey = 'sleep' | 'hrv' | 'resting_hr'

/** The personal band a direction is measured against. */
export interface VitalBaseline {
  /** The median of the last 30 nights, the day itself excluded. */
  median: number
  /** median − 1.5 × MAD */
  lower: number
  /** median + 1.5 × MAD */
  upper: number
  /** The median absolute deviation. */
  mad: number
  /** How many nights the band was computed from. */
  nights: number
}

/** One vital: the value, its direction, and the arithmetic behind it. */
export interface VitalValue {
  key: VitalKey
  /** The measurement in its own unit: seconds, milliseconds, bpm. */
  value: number | null
  unit: 's' | 'ms' | 'bpm'
  direction: Direction
  /** The value minus the usual value, in the metric's own scale. */
  deviation: number | null
  travel: Travel
  /** How many days the travel covers. */
  travel_days: number | null
  /** The band, when there is enough history for one. */
  baseline: VitalBaseline | null
}

/** One night of history, for the charts on Body. */
export interface VitalHistoryPoint {
  date: string
  value: number | null
}

/** A pattern: a fact about the past, with its sample size. */
export interface Pattern {
  id: string
  /** The catalogue key that phrases it. */
  key: CatalogueKey
  /** The sample size. Never below three. */
  n: number
  /** The numbers the phrasing carries. */
  params?: Record<string, string | number>
}

/** The vitals and their directions — the same snapshot Today and Body read. */
export interface VitalsSnapshot {
  date: string
  generated_at: string
  vitals: VitalValue[]
  history: Record<VitalKey, VitalHistoryPoint[]>
  patterns: Pattern[]
}

/** A session from the athlete's own plan. */
export interface PlanSession {
  date: string
  /** The plan's own vocabulary: easy, long, tempo, rest, strength. */
  session_type: string
  title: string | null
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
  /** The plan's target for today, in metres. */
  target_distance_m: number | null
  /** This week's running, in metres. */
  week: { distance_m: number; sessions: number }
}

/** The basis a comparison is drawn on. */
export type CompareBasis =
  | { kind: 'same_distance'; weeks_apart: number }
  | { kind: 'same_route' }
  | { kind: 'same_hr' }
  | { kind: 'different' }

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
  basis: CompareBasis
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
  headline:
    | { kind: 'fitter'; weeks: number; pace_delta_s_per_km: number }
    | { kind: 'not_enough'; weeks_needed: number }
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
