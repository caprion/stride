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
 *
 * **The numbers are rendered too.** The core's `units.rs` is the one
 * implementation of "how do we say 30 seconds" and "how do we say 8.4 km",
 * so the API sends the words beside the numbers: the text for the screen,
 * the raw number for a chart. The app has no rounding of its own — see
 * `src/lib/format.ts`, which is the chrome and nothing else.
 *
 * The API's DTOs do not exist yet. **These are the frontend's proposal**
 * for them, and they change when the API's land.
 */

import type { CatalogueKey } from '../i18n'

/**
 * A domain value, as the core renders it.
 *
 * The core renders the numbers — the distance, the duration, the heart
 * rate, the pace — from `units.rs`, and the API sends the result. The
 * app shows `text` and draws `raw`; it never renders the value itself.
 *
 * A value that is not there crosses as a null value — the whole
 * `RenderedValue` is absent, never a `text` of "0" and never a `raw` of
 * zero. The core's renderers return nothing for a missing value
 * (`render_distance(None)` is `None`), so the API sends nothing.
 */
export interface RenderedValue {
  /** The core's words: "8.4 km", "42 m", "148 bpm", "5:02 /km". */
  text: string
  /** The number, for a chart. Null when there is no number to draw. */
  raw: number | null
}

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
  /** The target, as the core renders it: "10 km". */
  target: RenderedValue | null
  /** What was actually run, as the core renders it: "8.4 km". */
  actual: RenderedValue | null
  status: 'done' | 'planned' | 'not_done'
  source: 'baseline' | 'prescribed' | 'typed'
}

/** The plan's active block for the current week. */
export interface PlanWeek {
  block_name: string
  source: 'baseline' | 'prescribed' | 'typed'
  week_start: string
  week_end: string
  /** The week's target, as the core renders it: "40 km". */
  target: RenderedValue | null
  sessions: PlanSession[]
  adherence: { done: number; planned: number } | null
}

/**
 * The totals of a group of runs.
 *
 * The feed is grouped by the API — it is the API that knows the totals —
 * and the app labels the groups.
 */
export interface ActivityGroupTotals {
  /** The group's key: the date the group starts. */
  key: string
  /** The group's distance, as the core renders it: "42.3 km". */
  distance: RenderedValue
  /** The group's time, as the core renders it: "3 h 42 m". */
  duration: RenderedValue
}

/** An activity, as the feed shows it. */
export interface ActivitySummary {
  id: number
  name: string
  sport_type: string
  start_time: string
  local_date: string
  /** The distance, as the core renders it: "8.4 km". */
  distance: RenderedValue
  /** The moving time, as the core renders it: "42 m". */
  duration: RenderedValue
  /** The average heart rate, when the run had one: "148 bpm". */
  heart_rate: RenderedValue | null
  /** The pace, when the run had one: "5:02 /km". */
  pace: RenderedValue | null
  /** The climb, when the run had one: "128 m". */
  elevation_gain: RenderedValue | null
  /** The group the run is in, and that group's totals. */
  group: ActivityGroupTotals
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
  splits: {
    index: number
    distance: RenderedValue
    duration: RenderedValue
    heart_rate: RenderedValue | null
  }[]
  elevation: { distance: RenderedValue; elevation: RenderedValue }[]
  samples: { offset_s: number; pace: RenderedValue | null; heart_rate: RenderedValue | null }[]
  /** The line against the plan, already written: "8.4 km of a 10 km target". */
  plan_line: { text: string } | null
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
  /** This week's running, as the core renders it. */
  week: { distance: RenderedValue; sessions: number }
}

/** One row of the metric diff. */
export interface CompareMetric {
  key: string
  /** The catalogue key that names the row. */
  label_key: CatalogueKey
  /** The value of A, as the core renders it. */
  a: RenderedValue | null
  /** The value of B, as the core renders it. */
  b: RenderedValue | null
  /** B − A, as the core renders it — sign and all. */
  delta: RenderedValue | null
}

/** One point of a comparison's series: where it was, and what it was. */
export interface CompareSeriesPoint {
  /** How far into the run the point is, in seconds. */
  offset_s: number
  /** The pace at that point, as the core renders it. */
  pace: RenderedValue | null
  /** The heart rate at that point, as the core renders it. */
  heart_rate: RenderedValue | null
}

/** The identity of a run, in a comparison. */
export interface CompareRun {
  id: number
  label: string
  date: string
  distance: RenderedValue
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
    /**
     * The chart's own points.
     *
     * A chart draws a line; it does not read a sentence. But the words
     * for these points are the core's too, so each point carries the
     * rendered value beside the number: `raw` for the line, `text` for
     * the tooltip.
     */
    a: CompareSeriesPoint[]
    b: CompareSeriesPoint[]
  }
}

/** A run that can be compared against. */
export interface CompareCandidate {
  id: number
  label: string
  date: string
  distance: RenderedValue
}

/** The Progress screen's data. */
export interface ProgressInsight {
  generated_at: string
  /** The headline, already written: "fitter than 8 weeks ago". */
  headline: string
  /** The evidence under it: "same heart rate, 24 s/km faster". */
  headline_note: string | null
  weekly_volume: { week_start: string; distance: RenderedValue }[]
  /** The typical week, as the core renders it: "32 km". */
  typical_week: RenderedValue | null
  /**
   * The pace at a given heart rate, for the chart.
   *
   * The numbers are the line; the words are the tooltip.
   */
  pace_at_hr: {
    week_start: string
    pace: RenderedValue | null
    heart_rate: RenderedValue | null
  }[]
  bests: { key: CatalogueKey; value: string; date: string }[]
  consistency: { week_start: string; sessions: number }[]
  /**
   * The sessions a week, as the core averages and renders them: "3.4".
   *
   * The average and its rounding are the core's; the label beside it —
   * "sessions a week" — is the app's chrome.
   */
  sessions_per_week: RenderedValue | null
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
