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
  text?: string | null
}

export interface SleepDay {
  date: string
  sleep: RenderedValue | null
  deep_sleep: RenderedValue | null
  rem_sleep: RenderedValue | null
  light_sleep: RenderedValue | null
  awake: RenderedValue | null
}

export interface SleepDays {
  days: SleepDay[]
  generated_at: string
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
  /**
   * The athlete's day, as the api counts it: "2026-05-13".
   *
   * The app highlights it inside the week, and the app does not keep a
   * clock of its own for it: the api's day is the athlete's day, with
   * their timezone in it.
   */
  today: string
  /** The week's target, as the core renders it: "40 km". */
  target: RenderedValue | null
  sessions: PlanSession[]
  adherence: { done: number; planned: number } | null
  /**
   * The week's counts, by status.
   *
   * Three numbers and never a percentage: a percentage hides the count,
   * and the count is the fact. **The api counts them** — the app does not
   * count the plan's sessions itself, because a count the app makes is a
   * second answer to the same question.
   */
  counts: { done: number; planned: number; not_done: number }
  /**
   * The adopted plan's own week, when a plan is in force.
   *
   * The adopted plan is the athlete's own programme: the sessions it
   * holds, their kinds, their distances, their paces where they have one,
   * and the plan's own source. `null` when no plan is in force, or when
   * the week asked for is not in it.
   */
  adopted: AdoptedWeek | null
}

/**
 * The adopted plan's week.
 *
 * The sessions are the athlete's own programme: a dated copy of one of
 * the catalogue's programmes, worked out once at the adoption.
 */
export interface AdoptedWeek {
  /** The week's number: eleven down to one, zero for the race week. */
  week: number
  /** The weeks to go. */
  weeks_to_go: number | null
  /** The week's own label: "3 weeks to go". */
  label: string
  start_date: string
  end_date: string
  sessions: AdoptedSession[]
  /** The plan's own source: baseline, prescribed or typed. */
  source: 'baseline' | 'prescribed' | 'typed'
  /** The line: "the Pfitzinger half marathon tier 2 plan". */
  source_line: string
  /** What the plan was paced with. */
  pacing: PlanPacing
  /** The tier's entry requirement beside the athlete's own mileage. */
  entry: PlanEntry
  /** The race week, moved to the day the athlete races. */
  shift: PlanShift | null
}

/**
 * The race week, moved to the day the athlete races.
 *
 * The move is a **fact**: the week says how far it moved and which
 * sessions it left out.
 */
export interface PlanShift {
  /** The days the week moved: negative is earlier. */
  days: number
  /** The line: "your race is on Saturday · this week's sessions are earlier a day". */
  line: string | null
  /** The sessions that are not in this week. */
  dropped: string[]
}

/** One session of the adopted plan. */
export interface AdoptedSession {
  date: string
  /** The day's name: "Wednesday". */
  weekday: string
  /** The runner's wording for the kind: "tempo". */
  kind: string
  /** The method's own name, when the kind is one of the method's. */
  method_kind: string | null
  /** The distance, as the core renders it: "17.7 km". */
  distance: string | null
  /** The session's pace: "4:16–4:21 /km". */
  pace: string | null
  /** The second end of a session with two: the long run's finish pace. */
  finish_pace: string | null
  /** The cell's own detail: "3 × 8 min, jog 3". */
  detail: string | null
  /** The line the surface leads with: "17.7 km · tempo". */
  label: string
  /** The line with the pace beside it, when the session has one. */
  line: string | null
  /** The reason the session has no pace, when it has none. */
  no_pace: string | null
  /** On a progression long run: the last miles at the threshold pace. */
  lt_last: string | null
}

/** What the plan was paced with, and where the numbers came from. */
export interface PlanPacing {
  /** Do the plan's sessions carry paces? */
  paced: boolean
  /** The threshold the plan was paced with. */
  threshold: PlanSourceValue | null
  /** The race the plan's own rule was read from. */
  rule: PlanSourceValue | null
  /** The reason the sessions that need a race have no pace. */
  missing: string | null
  /** The line: "paced with the threshold of 3 Nov 2026 · 165 bpm". */
  threshold_line: string | null
  /** The line: "paced with the plan's rule on your 10K of 12 Oct 2026". */
  rule_line: string | null
}

/** One thing the plan's numbers came from: a value and the day it was read. */
export interface PlanSourceValue {
  /** The value, as the core rendered it: "165 bpm". */
  value: string
  date: string
  date_label: string
  /** Where the value came from. */
  source: string | null
  /** The base the choice was made for. */
  base: string | null
}

/**
 * The tier's entry requirement, beside the athlete's own mileage.
 *
 * Three facts, side by side, unranked. The app states them and never
 * compares them.
 */
export interface PlanEntry {
  /** "this level asks for 64.4-80.5 km a week". */
  requirement: string | null
  /** "this plan runs 74-101.4 km a week". */
  plan_band: string | null
  /** "your last 4 weeks: 60.8, 61.1, 50.8, 6 km". */
  athlete: string | null
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
  /**
   * The one sentence about what the run meant.
   *
   * Never empty: the core always has a sentence, and the fallback shape is
   * catalogued like every other one. A row without a sentence is the
   * "spreadsheet" the plan forbids.
   */
  story: string
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
    /** The lap's number, from one — a lap, not a kilometre. */
    lap_index: number
    distance: RenderedValue
    duration: RenderedValue
    heart_rate: RenderedValue | null
    pace: RenderedValue | null
  }[]
  samples: {
    offset_s: number
    distance_m: RenderedValue | null
    pace: RenderedValue | null
    heart_rate: RenderedValue | null
    altitude_m: RenderedValue | null
    kind: string | null
  }[]
  /** The line against the plan, already written: "8.4 km of a 10 km target". */
  plan_line: { text: string } | null
  /**
   * The one extra fact the detail knows and the feed does not: the run's own
   * shape — "the last 1 km was the slowest of the run".
   *
   * It is said only when it is worth a sentence: when the slowest or fastest
   * split stands out from the run's average pace by more than ten per cent.
   */
  why: string | null
  note: string | null
}

export interface RunningDay {
  date: string
  sessions: number
  distance?: RenderedValue
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
  week: {
    distance: RenderedValue
    sessions: number
    days: RunningDay[]
    /**
     * The same elapsed days of the previous calendar week.
     *
     * The comparison is **like for like**: the week to date against the
     * same days of the week before, so the two windows are the same
     * length. A week to date against a whole week would weigh a part
     * against the whole.
     */
    previous: PreviousWeek
    /**
     * This week's distance against the previous week's — the core's own
     * rendering of the difference, sign and all.
     */
    difference: RenderedValue | null
  }
}

/**
 * The totals of a stretch of days: the previous week, as the cockpit's
 * "this week" row is measured against it.
 */
export interface PreviousWeek {
  /** The first day of the stretch, in the athlete's own calendar. */
  from: string
  /** The last day of the stretch, in the athlete's own calendar. */
  to: string
  /** The distance, as the core renders it. */
  distance: RenderedValue
  /** How many runs there were. */
  sessions: number
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
  date: string
  period: {
    from: string
    to: string
    distance: RenderedValue
    duration: RenderedValue
    sessions: number
    recorded_weeks: number
  }
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
  consistency: { week_start: string; sessions: number }[] /**
   * The sessions a week, as the core averages and renders them: "3.4".
   *
   * The average and its rounding are the core's; the label beside it —
   * "sessions a week" — is the app's chrome.
   */
  sessions_per_week: RenderedValue | null
  adherence: { done: number; planned: number } | null
  /**
   * How many more weeks the charts need before they can draw a trend.
   *
   * Null when they have what they need. **The number is the core's
   * rule** — how long a trend is — and the app does not decide it.
   */
  weeks_needed: number | null
  /**
   * The Eddington number over the window the screen is showing.
   *
   * **Null when the window holds no distance** — that is no value, not
   * zero. The number is counted in kilometres, the contract's unit, and
   * it travels with the days still needed for the next number and the
   * longest distance in a day.
   */
  eddington: {
    number: number
    days_needed_for_next: number
    longest_day: RenderedValue | null
  } | null
}

/**
 * The training load over the selected window.
 *
 * **Values and a scenario, never a verdict.** There is no band name, no
 * risk and nothing that tells the athlete what to do: the numbers, the
 * assumption they were made under, and what a run of rest days is expected
 * to do with them.
 */
export interface LoadInsight {
  generated_at: string
  date: string
  /** The window the numbers describe. */
  window: {
    /** The first day of the window, in the athlete's own calendar. */
    from: string
    /** The same day, as the api writes a date: "4 May 2026". */
    from_label: string
    /** The last day of the window, in the athlete's own calendar. */
    to: string
    /** The same day, as the api writes a date: "12 May 2026". */
    to_label: string
    /** How many calendar weeks the window covers. */
    weeks: number
    /**
     * How many days of history the series was warmed with, before the
     * window: the chronic series is weighted over forty-two days, and
     * without the warm-up the numbers at the start of the window would be
     * the numbers of a series that began there.
     */
    warmup_days: number
  }
  /**
   * The load itself.
   *
   * **Null when the load has no value** — and then `unavailable` says
   * why. A missing zone model is never a zero and never a guessed
   * maximum heart rate.
   */
  load: LoadSeries | null
  /** Why there is no load, or null when there is one. */
  unavailable: string | null
  /** The recovery scenario, or null when there is no load. */
  scenario: LoadScenario | null
}

/** The load of the days in the window. */
export interface LoadSeries {
  /** The days of the window the record covers, oldest first. */
  days: {
    /** The day, in the athlete's own calendar. */
    date: string
    /** The same day, as the api writes a date. */
    label: string
    /** The day's load, as the core renders it — null when the day has none. */
    load: RenderedValue | null
    /** Why the day has no load, or null when it has one. */
    reason: string | null
  }[]
  /** The state of the two series at each of those days, oldest first. */
  states: {
    date: string
    label: string
    /** The acute load: the seven-day weighted average. */
    acute: RenderedValue
    /** The chronic load: the forty-two-day weighted average. */
    chronic: RenderedValue
    /** The form: the chronic load minus the acute load. */
    form: RenderedValue
    /** The acute-to-chronic ratio. */
    ratio: RenderedValue
  }[]
  /**
   * The seven-day values.
   *
   * **Null when the week is not complete** — a week with a missing day
   * has no impulse, no monotony and no strain, because absence is not
   * zero.
   */
  week: LoadWeek | null
  /** The assumption the numbers rest on. */
  assumptions: LoadAssumption
}

/** The seven-day values of the load. */
export interface LoadWeek {
  /** The sum of the seven days' loads. */
  impulse: RenderedValue
  /** The monotony: the mean of the seven days over their standard deviation. */
  monotony: RenderedValue
  /** The strain: the seven-day sum times the monotony. */
  strain: RenderedValue
}

/** The assumption the load's numbers rest on. */
export interface LoadAssumption {
  /** The Banister coefficient the model used. */
  banister_k: number
  /** The maximum heart rate the zone model was built on. */
  max_heart_rate_bpm: number
  /** The threshold heart rate: the zone-5 floor of the stored model. */
  threshold_hr_bpm: number
  /** The day the zone model was read, in the athlete's own calendar. */
  zone_model_read_on: string
  /** The same day, as the api writes a date. */
  zone_model_read_on_label: string
  /** The acute window, in days. */
  acute_window_days: number
  /** The chronic window, in days. */
  chronic_window_days: number
  /** How many days the scenario projects forward. */
  forecast_days: number
  /** The assumptions as a sentence, in the core's own words. */
  statement: string
}

/** The recovery scenario. */
export interface LoadScenario {
  /**
   * How many complete rest days the ratio is expected to need to be back
   * inside its band — null when seven rest days do not bring it back.
   */
  days_to_ratio_band: number | null
  /**
   * How many days the form is expected to need to turn positive — null
   * when seven rest days do not turn it positive.
   */
  days_to_positive_form: number | null
  /** The band of the ratio, as a range: "0.8 to 1.3". */
  band: string
  /** The scenario as a sentence, in the core's own words. */
  statement: string
  /** The assumption the scenario rests on, in the core's own words. */
  assumption: string
  /** The projection itself, one entry per projected day. */
  projection: {
    day: number
    form: RenderedValue
    ratio: RenderedValue
  }[]
}

/**
 * The rolling view: a window of history and what it holds.
 *
 * The window is **the last N days ending on the day the answer is about**,
 * and the comparison period is **the same N days ending a day earlier**.
 * The two are a comparison, not a verdict: the numbers side by side and
 * the difference between them.
 */
export interface RollingInsight {
  generated_at: string
  /** The athlete's local calendar date the answer is about. */
  date: string
  /** The window the answer is about. */
  window: RollingWindow
  /** The period the window is compared against. */
  comparison: RollingWindow
  /** The distance of the window and of the period before it. */
  distance: WindowComparison
  /** The moving time of the window and of the period before it. */
  duration: WindowComparison
  /** The runs of the window and of the period before it. */
  sessions: SessionComparison
  /** The split of the training across the athlete's own zones. */
  zones: ZoneSplit
  /** The buckets of the distance breakdown: the runs, by how far they went. */
  distances: DistanceBucketRow[]
  /** The athlete's shape over the window: seven facets. */
  profile: ProfileFacet[]
  /** What usually happened on each day of the week. */
  weekday: WeekdayRow[]
  /** What usually happened in each time of day. */
  daytime: DaytimeRow[]
  /** What each month of the history holds, newest first. */
  months: PeriodRow[]
  /** What each year of the history holds, newest first. */
  years: PeriodRow[]
}

/** A window of days. */
export interface RollingWindow {
  /** The first day of the window, in the athlete's own calendar. */
  from: string
  /** The same day, as the api writes a date: "18 Feb 2026". */
  from_label: string
  /** The last day of the window. */
  to: string
  /** The same day, as the api writes a date: "12 May 2026". */
  to_label: string
  /** How many days the window covers. */
  days: number
}

/** A value in a window, in the window before it, and the difference. */
export interface WindowComparison {
  /** The value in the window. */
  current: RenderedValue | null
  /** The value in the window before it. */
  previous: RenderedValue | null
  /** The difference between the two, sign and all. */
  difference: RenderedValue | null
}

/** A count in a window, in the window before it, and the difference. */
export interface SessionComparison {
  /** The runs in the window. */
  current: number
  /** The runs in the window before it. */
  previous: number
  /** The difference between the two, as the core renders it. */
  difference: RenderedValue | null
}

/**
 * The split of the training across the athlete's own zones.
 *
 * The shares are facts — what share of the time in the zones was spent
 * in the low band, the moderate band and the high band. There is no
 * target range beside them.
 */
export interface ZoneSplit {
  /**
   * The models the window's runs were measured against, oldest first.
   *
   * Each run is measured against the model that was in force on its own
   * day, so a window that spans a change of the athlete's configuration
   * holds more than one model — and this names them all.
   */
  models: ZoneModelSource[]
  /**
   * The one source line the screen names: the model's own line when the
   * window was measured against one model, and the line that names them
   * all when it was measured against several.
   */
  source: string | null
  /** The shares, when there is a value. */
  shares: ZoneShareRow[] | null
  /** The distribution: the time in each of the five zones. */
  distribution: ZoneDistributionRow[] | null
  /** Why there is no split, in the core's own words. */
  unavailable: string | null
  /**
   * The window's weeks, and the band each one held.
   *
   * One row per calendar week that holds a run, oldest first, and the
   * time in each of the three bands that week. A band with no time in a
   * week is **no value** — never a zero — so the absence can be drawn as
   * the white space it is.
   */
  weeks: ZoneWeekRow[]
}

/** One week of the window, and the bands it held. */
export interface ZoneWeekRow {
  /** The Monday the week starts on. */
  week_start: string
  /** The same day, as the api writes a date: "4 May 2026". */
  week_start_label: string
  /** The runs the week holds. */
  sessions: number
  /** The time in each of the three bands, that week. */
  bands: ZoneWeekBand[]
}

/** One band of one week. */
export interface ZoneWeekBand {
  /** The band's key: "low", "moderate" or "high". */
  key: string
  /** The band's name, in the core's words: "Z1–Z2 low". */
  label: string
  /** The time in the band that week, or null when the week held none. */
  time: RenderedValue | null
}

/** One zone of the distribution: the time in it and its share. */
export interface ZoneDistributionRow {
  /** The zone's label: "Z1". */
  zone: string
  /** The range the zone covers, as the core renders it: "106-121 bpm". */
  range: string | null
  /** The time in the zone, as the core renders it. */
  time: RenderedValue
  /** The zone's share of the time in the zones. */
  share: RenderedValue | null
}

/** One bucket of the distance breakdown. */
export interface DistanceBucketRow {
  /** The bucket's key: "0-5". */
  key: string
  /** The bucket's range, as the core renders it: "0-5 km". */
  label: string
  /** The runs that fell into the bucket. */
  sessions: number
  /** The distance of those runs. */
  distance: RenderedValue
  /** The elevation gain of those runs. */
  elevation: RenderedValue
  /** The moving time of those runs. */
  duration: RenderedValue
}

/**
 * One facet of the athlete's shape.
 *
 * A value, and the way it moved against the window before it. There is
 * no polygon and no score: the reader draws the shape.
 *
 * **Two numbers, and the core says which is which.** The `value` is the
 * comparable one: the volume is a distance a week and the consistency is
 * the share of the window's days, so that a thirty-day window and a
 * ninety-day window can be read against each other. The `total` is what
 * the window holds: the distance of the window, the days it carries a
 * run — the number a reader asks for when they ask "how far did I run in
 * three months?".
 */
export interface ProfileFacet {
  /** The facet's key: "volume", "consistency" and so on. */
  key: string
  /** The facet's name, in the core's words: "Sport variety". */
  label: string
  /** The comparable value: the rate, or the average of the runs. Null when it cannot be computed. */
  value: RenderedValue | null
  /** The window's own total: what the window holds. */
  total: RenderedValue | null
  /** The comparable value in the window before it, of the same length. */
  previous: RenderedValue | null
  /** Which way it moved: "up", "down" or "steady". */
  movement: string | null
}

/**
 * One window of the athlete profile.
 *
 * The profile is drawn over **three windows** — thirty, sixty and ninety
 * days — and this is one of them: the answer, and the length it covers.
 */
export interface ProfileWindow {
  /** How many days the window covers: 30, 60 or 90. */
  days: number
  /** The router's answer for the window. */
  rolling: RollingInsight | undefined
}

/** The zone model a split was measured against. */
export interface ZoneModelSource {
  /** The day the model was read, in the athlete's own calendar. */
  read_on: string
  /** The same day, as the api writes a date: "10 May 2026". */
  read_on_label: string
  /** The model's own source line, in the core's words. */
  source: string | null
}

/** One band's share of the time in the zones. */
export interface ZoneShareRow {
  /** The band's key: "low", "moderate" or "high". */
  key: string
  /** The band's name, in the core's words: "Z1–Z2 low". */
  label: string
  /** The share of the time in the zones. */
  share: RenderedValue | null
  /** The share in the window before it. */
  previous_share: RenderedValue | null
  /** Which way it moved: "up", "down" or "steady". */
  trend: string | null
}

/** What usually happened on one day of the week. */
export interface WeekdayRow {
  /** The day's name, in the core's words: "Monday". */
  weekday: string
  /** How many runs fell on it. */
  sessions: number
  /** The distance of those runs. */
  distance: RenderedValue
  /** The moving time of those runs. */
  duration: RenderedValue
  /** The day's share of the moving time. */
  share: RenderedValue | null
}

/** What usually happened in one time of day. */
export interface DaytimeRow {
  /** The bucket's key: "morning", "afternoon", "evening" or "night". */
  key: string
  /** The bucket's name, in the core's words. */
  label: string
  /** The hour the bucket starts at, in the athlete's own local time. */
  from_hour: number
  /** The hour the bucket ends at. */
  to_hour: number
  /** How many runs fell into it. */
  sessions: number
  /** The distance of those runs. */
  distance: RenderedValue
  /** The moving time of those runs. */
  duration: RenderedValue
}

/** What a month or a year of the history holds. */
export interface PeriodRow {
  /** The period's key: "2026-04" for a month, "2026" for a year. */
  key: string
  /** The period's name, as the api writes it: "April 2026" or "2026". */
  label: string
  /** How many runs fell into it. */
  sessions: number
  /** The distance of those runs. */
  distance: RenderedValue
  /** The moving time of those runs. */
  duration: RenderedValue
  /** The elevation gain of those runs. */
  elevation: RenderedValue
}

/**
 * The streaks of the athlete's history.
 *
 * A streak is a count and the days it covers: nothing more. There is no
 * "you are consistent" here — the number is the number.
 */
export interface StreakInsight {
  generated_at: string
  /** The athlete's local calendar date the answer is about. */
  date: string
  /** The streaks of days. */
  days: StreakDetail
  /** The streaks of weeks. */
  weeks: StreakDetail
  /** The streaks of months. */
  months: MonthStreakDetail
}

/** The streaks of months: the current one and the longest one. */
export interface MonthStreakDetail {
  /** The streak that is still going. Null when there is none. */
  current: MonthStreakValue | null
  /** The longest one in the history. */
  longest: MonthStreakValue | null
}

/**
 * One streak of months: how long it is, and the months it covers.
 *
 * A month streak is a run of calendar months, each with a run in it. The
 * months are named as months — "November 2025" — and the days inside
 * them are not the unit here.
 */
export interface MonthStreakValue {
  /** How many months the streak runs. */
  count: number
  /** The month it started, as `YYYY-MM`. */
  start: string
  /** The same month, as the api writes a month: "November 2025". */
  start_label: string
  /** The month it ended, when it ended in the past. */
  end: string | null
  /** The same month, as the api writes a month. */
  end_label: string | null
}

/** The streaks of one kind: the current one and the longest one. */
export interface StreakDetail {
  /** The streak that is still going. Null when there is none. */
  current: StreakValue | null
  /** The longest one in the history. */
  longest: StreakValue | null
}

/** One streak: how long it is, and the days it covers. */
export interface StreakValue {
  /** How many days (or weeks) the streak runs. */
  count: number
  /** The day it started, in the athlete's own calendar. */
  start: string
  /** The same day, as the api writes a date: "2 May 2026". */
  start_label: string
  /** The day it ended, when it ended in the past. */
  end: string | null
  /** The same day, as the api writes a date. */
  end_label: string | null
}

export interface ActivityCalendar {
  year: number
  month: number
  generated_at: string
  days: {
    date: string
    day: number
    distance: RenderedValue | null
    duration: RenderedValue | null
    activities: ActivitySummary[]
  }[]
}
/** The connection and sync state. */
export interface SyncStatus {
  last_sync_at: string | null
  last_error: string | null
  activity_count: number
  /**
   * The newest Garmin activity the app has already pulled.
   *
   * The pull is a delta: the next one asks Garmin for what is after this, so
   * a known run is never downloaded twice.
   */
  cursor: string | null
  /** The runs the last pull downloaded. */
  pulled: number | null
  /** The runs the last pull recognised and did not download again. */
  skipped: number | null
  /** The runs Garmin no longer lists, marked and not resurrected. */
  deleted: number | null
  more_pending: boolean
  history: {
    activities: SyncResourceCoverage | null
    wellness: SyncResourceCoverage | null
  } | null
}

export interface SyncResourceCoverage {
  target_from: string
  checked_from: string | null
  checked_to: string | null
  next_date: string | null
}

export interface SyncRunResult {
  activities: number
  wellness_days: number
  more_pending: boolean
  complete: boolean
  failure: {
    code: string
    detail: string
    status: number
    retry_after_seconds: number | null
  } | null
  finished_at: string
  generated_at: string
}

/**
 * One heart-rate band of a run.
 *
 * The bands are the athlete's own zones: their edges are the floors of the
 * **stored zone model**, not tenths of this run's maximum heart rate.
 */
export interface ZoneBand {
  /** The band's name, as the core names it: "Z2". */
  label: string
  /** The band's range, in the core's words: "106-121 bpm". */
  range: string | null
  /**
   * The band's edges, as numbers — for **drawing** the band on a chart.
   *
   * The words are what the athlete reads; the numbers are what the chart
   * draws. Both come from the core: the words are rendered there, and the
   * edges are the same numbers it rendered them from.
   */
  from_bpm: number | null
  to_bpm: number | null
  /**
   * The time spent in the band, as the core renders it: "12 m".
   *
   * The number beside the words is the time in **seconds**.
   */
  time: RenderedValue | null
}

/**
 * The athlete's zone model: a dated fact about their configuration.
 *
 * The model is read from Garmin and stored with the day it was read. It is
 * named in the payload, so a number can say where its input came from and
 * when it was read.
 */
export interface ZoneModel {
  /** The sport Garmin reported the model for: "DEFAULT". */
  sport: string
  /** The training method the model was set with: "HR_MAX". */
  training_method: string
  /** The maximum heart rate the model was built on: "178 bpm". */
  max_heart_rate: RenderedValue | null
  /** The five floors, in beats per minute. */
  floors: ZoneModelFloors
  /** When the model was read from Garmin: "2026-05-10T06:30:00Z". */
  read_at: string
  /**
   * The **day** the model was read, in the athlete's own calendar.
   *
   * The api renders the day, in the api's own form — "10 May 2026", the
   * form the run detail already uses for the run's own date. The core's
   * canonical form stays inside the core, and the app never formats a
   * date of its own.
   */
  read_on: string
  /**
   * The model's own source line, in the core's words: where the bands
   * came from and when the model was read.
   *
   * It is a sentence the core rendered, so the app shows it as it
   * arrives.
   */
  source: string | null
}

/**
 * The five floors of a zone model.
 *
 * The floors are the **exclusive lower bounds** of the zones: the interval
 * of zone *n* is `(floor_n, floor_{n+1}]`, and a reading at or below the
 * first floor is outside the zones.
 */
export interface ZoneModelFloors {
  /** The floor of zone 1: "106 bpm". */
  zone1: RenderedValue
  /** The floor of zone 2: "121 bpm". */
  zone2: RenderedValue
  /** The floor of zone 3: "135 bpm". */
  zone3: RenderedValue
  /** The floor of zone 4: "149 bpm". */
  zone4: RenderedValue
  /** The floor of zone 5: "163 bpm". */
  zone5: RenderedValue
}

/**
 * The heart-rate zones of a run, computed from the athlete's **stored zone
 * model**.
 *
 * When there is not enough to compute them — no model has been read, or the
 * stored model cannot be used — the answer says so in words rather than
 * inventing bands. "No value" is never band 1 and never a zero.
 */
export interface FitnessZones {
  /** The basis the times were computed on: "moving". */
  basis: string | null
  /**
   * The watch's own model, in force on the run's day.
   *
   * `null` when no model has been read; present even when the model cannot
   * be used, so the reader can see which model was rejected.
   *
   * This is the **other fact**: the bands the times were computed on are
   * named by `bands_basis`, and the two are never mixed inside one answer.
   */
  model: ZoneModel | null
  /**
   * The bands from the heart rates the athlete entered, where a
   * declaration is in force.
   */
  bands: DeclaredBandsRow | null
  /** The basis the bands came from — **one** of the two, never both. */
  bands_basis: BandsBasisRow | null
  /** The five bands, each with the time spent in it. */
  zones: ZoneBand[]
  /**
   * The time outside the zones: no heart-rate reading, or a reading at or
   * below the first floor.
   */
  outside: RenderedValue | null
  /**
   * The time the athlete was not moving: the stretches they stood still.
   *
   * It is kept apart from the zones so that the zones and the outside add
   * up to the time the athlete actually moved, and so that a run can never
   * claim more time in a zone than it spent moving.
   */
  stopped: RenderedValue | null
  /** Why there is no value, in the core's words. */
  unavailable: string | null
}

/**
 * One observation of a run's mechanics.
 *
 * **An observation is a value or a reason** — never a zero where there is
 * nothing to measure, and never a score: a weighted mean of a run's
 * numbers would be a grade, and a grade is a verdict.
 */
export interface FingerprintObservation {
  /** The observation's key: "cadence", "cadence_variability", and so on. */
  key: string
  /** The label, in the core's own words: "Cadence". */
  label: string
  /** The value, as the core renders it: "163.0 spm". */
  value: RenderedValue | null
  /** Why there is no value, in the core's words. */
  reason: string | null
}

/**
 * One zone's share of the moving time.
 *
 * The share is of the time **in the zones**; the time outside them is
 * reported on its own.
 */
export interface ZoneShare {
  /** The zone's name, as the core names it: "Z1". */
  label: string
  /** The zone's share of the time in the zones: "8.0 %". */
  share: RenderedValue
  /** The time spent in the zone, as the core renders it: "12 m". */
  time: RenderedValue | null
}

/**
 * The zone distribution of a run, against the athlete's **stored** model.
 *
 * When there is not enough to compute it — no model has been read, or the
 * stored model cannot be used — the answer says so in words rather than
 * inventing shares.
 */
export interface FingerprintZones {
  /** The model the shares were computed against, named and dated. */
  model: ZoneModel | null
  /** The shares, one per zone. Empty when there is no value. */
  shares: ZoneShare[]
  /** The time outside the zones. */
  outside: RenderedValue | null
  /** Why there is no value, in the core's words. */
  unavailable: string | null
  /** Where the model came from, in the core's words. */
  source: string | null
}

/**
 * The fingerprint of one run: the run's own mechanics, as **observations**.
 *
 * The observations come from the run's own laps and its moving samples,
 * decoded on read; the zone shares are the athlete's own model's, with the
 * day that model was read. There is no score in the answer, and no
 * composite: the reader decides what the numbers mean.
 */
export interface Fingerprint {
  /** The run's id. */
  activity_id: number
  /** The athlete's local date. */
  local_date: string
  /** The basis the numbers were computed on: "laps and moving samples". */
  basis: string | null
  /** How many laps the analysis used: the running ones. */
  laps: number
  /** The observations, in the order the app shows them. */
  observations: FingerprintObservation[]
  /** The zone distribution. */
  zones: FingerprintZones
  /** Why there is no value at all, in the core's words. */
  unavailable: string | null
  /** When the answer was produced. */
  generated_at: string
}

/**
 * The heatmap: a grid over the stored positions.
 *
 * **Computed on the device.** The api fetches nothing for this answer and
 * sends nothing with it: the cells are made of the positions the athlete's
 * own runs carry, decoded from their stored FIT files on read.
 *
 * A cell's value is a **count of runs**, never a route: no cell carries a
 * time, an order or a connection to another cell, so no path can be read
 * back out of the answer.
 */
export interface HeatmapInsight {
  /** When the answer was produced. */
  generated_at: string
  /** The athlete's local date. */
  date: string
  /** How many runs carried a position and are in the picture. */
  runs: number
  /**
   * How many runs carried a position but are **outside** the picture:
   * the places the athlete ran in, and the picture does not cover.
   */
  runs_outside: number
  /**
   * Whether the store holds runs this answer did not read.
   *
   * The answer is bounded — it reads the newest runs — and says so rather
   * than letting a partial map read as the whole history.
   */
  more_runs: boolean
  /**
   * The size of one cell on the ground, as the core renders it: "2 km".
   *
   * `null` when there is no cell to size.
   */
  cell_size: RenderedValue | null
  /** The cells, north to south and west to east. */
  cells: HeatmapCell[]
  /** The box the cells cover, or null when there is none. */
  extent: HeatmapExtent | null
  /** Why there is no heatmap, or null when there is one. */
  unavailable: string | null
}

/** One cell of the heatmap. */
export interface HeatmapCell {
  /** The cell's column in the grid. */
  x: number
  /** The cell's row in the grid. */
  y: number
  /** The cell's southern edge. */
  south: number
  /** The cell's western edge. */
  west: number
  /** The cell's northern edge. */
  north: number
  /** The cell's eastern edge. */
  east: number
  /** How many runs went through this cell. */
  runs: number
}

/** The box the heatmap's cells cover. */
export interface HeatmapExtent {
  /** The southern edge. */
  south: number
  /** The western edge. */
  west: number
  /** The northern edge. */
  north: number
  /** The eastern edge. */
  east: number
}

/** The health of the sync window. */
export interface HealthDaily {
  date: string
  generated_at: string
  stale: boolean
}

// ---------------------------------------------------------------- the app
//
// The settings, the connection and the app's own facts. These are the
// shapes the Settings screen reads and the Connect screen writes.
//
// The route table is frozen (`docs/contract/api-paths.md`), and the routes
// below are the ones the app needs and the table does not yet carry:
// `/settings`, `/settings/export`, `/settings/delete` and
// `/activities/{id}/note`. They are declared here because **the app's
// declarations are the contract** — the api is held to this file by
// `crates/stride-api/tests/frontend_shapes.rs` — and every one of them
// has to be written into `docs/contract/api-paths.md` in the same pull
// request.

/**
 * The connection: Garmin's state, the key's identifier, the last sync.
 *
 * **The key never crosses this boundary.** The answer carries the last
 * four characters of the key and nothing else — an identifier the athlete
 * can check against their own key, not a credential. The plan's rule is
 * absolute: no endpoint returns the key, and no screen renders it.
 */
export interface ConnectionStatus {
  /** Whether a key is held. */
  connected: boolean
  /** The method the key is used with: "personal_key". */
  method: string | null
  /** Where the key comes from: "personal_key", or "none". */
  key_source: string
  /**
   * The last four characters of the key — and never the key.
   *
   * Null when no key is held.
   */
  key_last4: string | null
  /** When the last sync succeeded. */
  last_sync_at: string | null
  /** When this answer was produced. */
  generated_at: string
}

/** The answer to holding a key. */
export interface KeyConnectResult {
  /** Whether a key is held. */
  connected: boolean
  /** The method the key is used with. */
  method: string | null
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The answer to checking a key.
 *
 * The key is checked and not held: the check exchanges one token and
 * drops it. A key Garmin refuses is a typed failure, not a `false`.
 */
export interface KeyTestResult {
  /** Whether Garmin accepted the key. */
  ok: boolean
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The answer to importing a FIT file.
 *
 * A file that is already in the log is a duplicate, and a duplicate adds
 * no row — it is reported, once, in one line.
 */
export interface FitImportResult {
  /** Whether the file became a run. */
  imported: boolean
  /** Whether the file was a run that is already in the log. */
  duplicate: boolean
  /** The run the file is: the one it became, or the one already there. */
  activity_id: number | null
  /** The run's day, when the file was a duplicate. */
  date: string | null
  /** The run's distance, as the core renders it: "10.2 km". */
  distance: RenderedValue | null
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The answer to importing a plan file.
 *
 * The plan's source is named by the block, not by the file: a file with no
 * recognisable workout keywords is unclassified, never "easy".
 */
export interface PlanImportResult {
  /** Whether the plan was imported. */
  imported: boolean
  /** The block that was written. */
  block_id: number | null
  /** How many sessions were written. */
  sessions: number
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The answer to writing a note on a run.
 *
 * A note is the only free text the product stores, and it belongs to the
 * run it is about — not to the settings.
 */
export interface NoteResult {
  /** The note as it stands now: the athlete's own words. */
  note: string | null
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The service's own health — and the app's version, for About.
 *
 * The version is the binary's, so it is asked for rather than written in
 * the bundle: the rig and the phone are the same app, and they say the
 * same version.
 */
export interface Health {
  /** Whether the router is answering. */
  ok: boolean
  /** The service's name: "stride". */
  service: string
  /** The version, as the build's environment has it. */
  version: string
  /** Whether a Garmin key is held. */
  garmin_connected: boolean
  /** How many runs are stored. */
  activities: number
  /** How many wellness days are stored. */
  wellness_days: number
  /** How many planned sessions are stored. */
  plan_sessions: number
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The weekly goal.
 *
 * **Blank means no goal.** There is no default: the app never invents a
 * number the athlete did not write, and a goal of zero would read as a
 * failure rather than as the absence of one.
 */
export interface SettingsGoal {
  /** The goal, as the core renders it: "40 km". Null when there is none. */
  value: RenderedValue | null
  /**
   * The goal in kilometres, for the field: 40.
   *
   * The unit is the API's and the core's; the app puts this number in a
   * number field and sends it back unchanged. It never converts a
   * distance of its own.
   */
  km: number | null
}

/**
 * Why the plan is paused.
 *
 * The reason is one of three words, not free text: a run's note is the
 * only free text the product stores.
 */
export type PauseReason = 'travel' | 'sick' | 'injured'

/** A pause in the plan: a start, an end and a reason. */
export interface SettingsPause {
  /** The first day of the pause: "2026-05-12". */
  start_date: string
  /** The last day of the pause: "2026-05-19". */
  end_date: string
  /** The reason: travel, sick or injured. Null when there is none. */
  reason: PauseReason | null
}

/** The storage: what is on the device, and how big it is. */
export interface SettingsStorage {
  /** The size of the database, as the host renders it: "2.4 MB". */
  size: RenderedValue | null
  /** How many runs are in it. */
  activity_count: number
  /** How many wellness days are in it. */
  wellness_day_count: number
  /** How many planned sessions are in it. */
  plan_session_count: number
  /** How many notes are in it — the only free text the app stores. */
  note_count: number
}

/**
 * The settings the athlete has set.
 *
 * The goal, the pause and the storage. The connection is not here: it
 * lives on `/auth/status`, which owns it, and the app's version lives on
 * `/health`, which owns it.
 */
export interface SettingsState {
  /** The challenges the athlete has set, in the order they were set. */
  challenges: SettingsChallenge[]
  /** The weekly goal. */
  goal: SettingsGoal
  /** The pause, when there is one. */
  pause: SettingsPause | null
  /** The storage. */
  storage: SettingsStorage
  /** When this answer was produced. */
  generated_at: string
}

/** The answer to exporting a backup. */
export interface SettingsExportResult {
  /** Whether the backup was written. */
  exported: boolean
  /** How big it is, as the host renders it: "2.4 MB". */
  size: RenderedValue | null
  /** When this answer was produced. */
  generated_at: string
}

/**
 * The answer to deleting everything.
 *
 * The typed confirmation is the caller's: the word has to arrive with the
 * request, so the answer can only say what happened.
 */
export interface SettingsDeleteResult {
  /** Whether the data was deleted. */
  deleted: boolean
  /** When this answer was produced. */
  generated_at: string
}

/* --------------------------------- the ladder, the curve, the threshold, the ledger */

/**
 * The window an insight answer describes.
 *
 * **The window is named, and the answer says what it covers.** A number
 * is never read apart from the period it came from.
 */
export interface InsightWindow {
  /** The first day of the window, in the athlete's own calendar. */
  from: string
  /** The same day, as the api writes a date: "31 Aug 2026". */
  from_label: string
  /** The last day of the window. */
  to: string
  /** The same day, as the api writes a date: "20 Nov 2026". */
  to_label: string
  /** How many calendar weeks the window covers. */
  weeks: number
}

/**
 * One source line, behind the click.
 *
 * **The source line is hoisted**: every answer carries one line for the
 * screen to name, and the lines it was built from travel behind a click.
 */
export interface SourceLine {
  /** What the source is: `race` or `zone_model`. */
  kind: string
  /** The line, in the core's own words. */
  line: string | null
  /** The day the source was read or run. */
  date: string | null
  /** The same day, as the api writes a date: "3 Nov 2026". */
  date_label: string | null
}

/**
 * The rule a number comes from.
 *
 * The rule carries its identifier, its citation — the chapter it comes
 * from — and the wording the surface prints beside the number.
 */
export interface RuleRow {
  /** The rule's identifier: "book.lt_pace_band". */
  id: string
  /** Where the rule comes from: "ch. 1, ch. 7". */
  citation: string
  /** The wording, in the core's own words. */
  line: string | null
}

/**
 * A pace, or the reason there is none.
 *
 * **A missing pace is null with a reason**, never a band of zero.
 */
export interface PaceRow {
  /** The band, in the core's own words: "4:16–4:21 /km". */
  value: string | null
  /** The fast edge, in seconds per kilometre. */
  fast_s_per_km: number | null
  /** The slow edge, in seconds per kilometre. */
  slow_s_per_km: number | null
  /** Why there is no value, in the core's own words. */
  missing: string | null
}

/**
 * A heart-rate band, or the reason there is none.
 *
 * The band is the athlete's own: the method's share of **their** maximum
 * heart rate, or of their heart-rate reserve.
 */
export interface HeartRateRow {
  /** The band, in the core's own words: "146–167 bpm". */
  value: string | null
  /** The lower edge, in beats per minute. */
  from_bpm: number | null
  /** The upper edge, in beats per minute. */
  to_bpm: number | null
  /** The ceiling, in beats per minute, when the band is a ceiling. */
  below_bpm: number | null
  /** The maximum heart rate the band was computed on. */
  max_heart_rate: number | null
  /** The resting heart rate the reserve was computed on. */
  resting_heart_rate: number | null
  /** Why there is no value, in the core's own words. */
  missing: string | null
}

/**
 * One rung of the ladder.
 *
 * A rung is two things: a pace where the method states one, and a
 * heart-rate band where it states one. The two are never turned into one
 * another.
 */
export interface RungRow {
  /** What the rung is: "lactate_threshold", "recovery", … */
  kind: string
  /** The rung's name, in the core's own words. */
  label: string
  /** The rule the rung's numbers come from. */
  rule: RuleRow
  /** What the rung's numbers hang on: "goal_race", "threshold_band", … */
  anchor: string
  /** The pace the rung is run at. */
  pace: PaceRow
  /** The pace the long run finishes at. */
  finish_pace: PaceRow
  /** The band, as a share of the athlete's maximum heart rate. */
  heart_rate: HeartRateRow
  /** The same band as a share of the athlete's heart-rate reserve. */
  heart_rate_reserve: HeartRateRow
}

/**
 * A race the athlete typed.
 *
 * A race is an observation, not a verdict: the athlete types it, and the
 * app never invents one.
 */
export interface RaceRow {
  id: number
  /** The athlete's own label for the race: "10K". */
  label: string
  distance: RenderedValue
  time: RenderedValue
  pace: RenderedValue | null
  date: string
  date_label: string
  name: string | null
  /** Where the race came from: "typed". */
  source: string
  entered_on: string
  updated_on: string
  /** Do the paces come from this race? At most one race carries it. */
  anchor: boolean
}

/** One equivalent race time, and the two ways it was reached. */
export interface EquivalentRow {
  /** The distance's name: "10K", "half marathon". */
  distance: string
  /** The time by Riegel's form, as the core renders it. */
  time: string | null
  /** The time by the inversion of the relations, as the core renders it. */
  vdot_time: string | null
  /** The pace by Riegel's form, in the core's own words. */
  pace: string | null
  /** The gap between the two models, in seconds. */
  difference_s: number | null
}

/**
 * The ladder: the rungs, the anchors and the chain that produced them.
 *
 * **The ladder is the picture**; the caption names the race it came from.
 */
export interface PacesInsight {
  generated_at: string
  date: string
  window: InsightWindow
  /** The one source line the screen names. */
  source: string | null
  /** The lines behind the click. */
  sources: SourceLine[]
  /** The chain that produced the number, in the core's own words. */
  chain: string | null
  /** The race the paces come from. */
  race: RaceRow | null
  /** The equivalent race times. */
  equivalents: EquivalentRow[]
  /** The rungs, fastest first. */
  rungs: RungRow[]
  /** Why there is no value at all, in the core's own words. */
  unavailable: string | null
}

/** One point of the physiological-cost curve. */
export interface CurvePointRow {
  /** The duration, in minutes. */
  minutes: number
  /** The share of the maximum the duration holds. */
  percent_max: RenderedValue
  /** The pace that costs it, in the core's own words. */
  pace: RenderedValue | null
}

/** One of the athlete's own races, as a point on the curve. */
export interface CurveRaceRow {
  /** The distance's name: "10K". */
  label: string
  /** The finish time, as the core renders it. */
  time: string
  /** The duration, in minutes. */
  minutes: number
  /** The share of the maximum the race held. */
  percent_max: RenderedValue
  /** The pace the race was run at. */
  pace: RenderedValue | null
  date: string
  date_label: string
}

/** One of the method's own sessions, as a band on the curve. */
export interface CurveSessionRow {
  /** What the session is: "threshold", "recovery", "long_run". */
  kind: string
  /** The session's name, in the core's own words. */
  label: string
  /** The first minute of the band's duration. */
  from_minutes: number | null
  /** The last minute of the band's duration. */
  to_minutes: number | null
  /** The pace the session is run at. */
  pace: PaceRow | null
  /**
   * The share of the maximum the session's **fast** end costs.
   *
   * The curve's vertical axis, and the core's own relation: the app
   * never computes it.
   */
  from_percent_max: RenderedValue | null
  /** The share of the maximum the session's **slow** end costs. */
  to_percent_max: RenderedValue | null
  /** The rule the session comes from. */
  rule: RuleRow
  /** Why the band has no value, in the core's own words. */
  missing: string | null
}

/**
 * The physiological-cost curve: the share of the maximum against time.
 *
 * A second reading of the same athlete, and not a second ladder.
 */
export interface CurveInsight {
  generated_at: string
  date: string
  window: InsightWindow
  /** The one source line the screen names. */
  source: string | null
  /** The lines behind the click. */
  sources: SourceLine[]
  /** The curve's own points. */
  curve: CurvePointRow[]
  /** The athlete's own races, as points on the curve. */
  races: CurveRaceRow[]
  /** The method's own sessions, as bands on the curve. */
  sessions: CurveSessionRow[]
  /** The field test's place in time. */
  field_test: CurvePointRow | null
  /** The pace equivalents. */
  equivalents: EquivalentRow[]
  /** Why there is no value at all, in the core's own words. */
  unavailable: string | null
}

/** One candidate: a value, a date, and the rule that computed it. */
export interface CandidateRow {
  /** Where the candidate comes from: "watch_zone_model", "race", … */
  source: string
  /** The source's name, in the runner's words. */
  label: string
  /** The value, in the core's own words. */
  value: string | null
  /** The heart rate, in beats per minute. */
  heart_rate_bpm: number | null
  /** The pace band, when the candidate carries one. */
  pace: PaceRow | null
  /** The day of the fact the value came from. */
  date: string | null
  /** The same day, as the api writes a date. */
  date_label: string | null
  /** The rule that computed the value, where a rule did. */
  rule: RuleRow | null
  /**
   * Why there is no value, in the core's own words.
   *
   * **The note that says what the candidate cannot know is not here**:
   * that line is the explanation, and the explanation is not product
   * content.
   */
  missing: string | null
}

/** The choice in force, and the choices that came before it. */
export interface ChoiceRow {
  /** Where the choice came from. */
  source: string
  /** The source's name, in the runner's words. */
  label: string
  /** The value, in the core's own words. */
  value: string | null
  /** The heart rate, in beats per minute. */
  heart_rate_bpm: number | null
  /** The pace band, when the choice carries one. */
  pace: PaceRow | null
  /** The base the choice was made for: "pfitzinger", "eighty_twenty". */
  base: string
  /** The athlete's own label for a typed number. */
  typed_label: string | null
  /** The day of the fact the number came from. */
  origin_date: string | null
  /** The day the choice was made. */
  entered_on: string
  /** The day the row was last changed. */
  updated_on: string
  /** Is this the choice in force now? */
  in_force: boolean
}

/** One choice in the dated history. */
export interface HistoryRow {
  /** The day the choice was made, in the athlete's own calendar. */
  date: string
  /** The same day, as the api writes a date: "3 Nov 2026". */
  date_label: string
  /** The choice's own sentence, in the core's words. */
  line: string | null
  /** The value, in the core's own words. */
  value: string | null
  /** The base the choice was made for. */
  base: string
  /** Is this the choice in force now? */
  in_force: boolean
}

/**
 * The threshold: the four candidates, the choice in force, and the history.
 *
 * **The app computes the candidates and the athlete chooses one.** The
 * card carries the value, the source and the date — never a note.
 */
export interface ThresholdInsight {
  generated_at: string
  date: string
  window: InsightWindow
  /** The one source line the screen names. */
  source: string | null
  /** The lines behind the click. */
  sources: SourceLine[]
  /** The four candidates, in the product's own order. */
  candidates: CandidateRow[]
  /** The choice in force. */
  in_force: ChoiceRow | null
  /** The choices in their dated history, newest first. */
  history: HistoryRow[]
  /** Why there is no value at all, in the core's own words. */
  unavailable: string | null
}

/** One of the athlete's own bands, over the period. */
export interface KindRow {
  /** The band's key: "low", "moderate" or "high". */
  key: string
  /** The band's name, in the core's own words: "Z1–Z2 low". */
  label: string
  /** The time in the band. */
  time: RenderedValue | null
  /** The share of the measured moving time. */
  share: RenderedValue | null
  /** The runs that carried the band. */
  runs: number
  /** When the band last happened, over the athlete's own history. */
  last: LastRow | null
}

/** When a kind last happened. */
export interface LastRow {
  date: string
  date_label: string
  /** The days between that day and the end of the window. */
  days_before_end: number | null
}

/** One band of a week, in the weeks matrix. */
export interface WeekBandRow {
  /** The band's key: "low", "moderate" or "high". */
  key: string
  /** The band's name, in the core's own words. */
  label: string
  /**
   * The time in the band that week.
   *
   * **A band a week held none has no value**: the absence is drawn as
   * white space, never as a zero.
   */
  time: RenderedValue | null
}

/** One week of the weeks matrix. */
export interface LedgerWeekRow {
  /** The Monday the week begins on, in the athlete's own calendar. */
  week_start: string
  /** The same day, as the api writes a date: "7 Sep 2026". */
  week_start_label: string
  /** The bands that week, in the product's own order. */
  bands: WeekBandRow[]
}

/** What the period covers. */
export interface CoverageRow {
  /** The runs in the period. */
  runs: number
  /** The runs that carried heart-rate samples. */
  with_heart_rate: number
  /**
   * The runs the app could measure — a zone model was in force **and the
   * run carried a heart rate**.
   *
   * A run with no heart rate carries no measurement: the time in a band
   * would be a zero where there is nothing to measure.
   */
  measured: number
  /** The runs that carried the source's own zone minutes. */
  with_source_zones: number
}

/** The longest run in the period. */
export interface LongestRow {
  date: string
  date_label: string
  distance: RenderedValue | null
  /** How long the run moved. */
  time: RenderedValue
  /** The share of the period's moving time. */
  share: RenderedValue | null
}

/** The repeated work: the runs the watch recorded as workouts. */
export interface RepeatedRow {
  time: RenderedValue
  runs: number
  last_on: string | null
  last_on_label: string | null
}

/** One of the watch's own zones, over the period. */
export interface SourceZoneRow {
  /** The watch's zone, one through five. */
  zone: number
  time: RenderedValue | null
  share: RenderedValue | null
  runs: number
}

/** One week of the source's own matrix. */
export interface SourceWeekRow {
  week_start: string
  week_start_label: string
  /** The time in each of the watch's five zones, that week. */
  zones: (RenderedValue | null)[]
}

/** Where both bases exist. */
export interface OverlapRow {
  from: string
  from_label: string
  to: string
  to_label: string
  /** The runs in the overlap. */
  runs: number
  /** The app's own zone time over the overlap. */
  app_moving: RenderedValue
  /** The source's own zone minutes over the overlap. */
  source_elapsed: RenderedValue
  /** The distance between the two measurements, in seconds. */
  difference: RenderedValue
}

/**
 * The ledger: the athlete's own bands, the shape rows, and the two bases.
 *
 * **Two bases, side by side and never added up**: the app's own zone
 * time, measured over the time the athlete was running, and the watch's
 * own zone minutes, measured over the whole run.
 */
export interface StimulusInsight {
  generated_at: string
  date: string
  window: InsightWindow
  /** The one source line the screen names. */
  source: string | null
  /** The lines behind the click. */
  sources: SourceLine[]
  /** The kinds' rows, in the product's own order. */
  kinds: KindRow[]
  /** The weeks matrix: the band each week held. */
  weeks: LedgerWeekRow[]
  /** What the period covers. */
  coverage: CoverageRow
  /** The longest run in the period. */
  longest: LongestRow | null
  /** The repeated work in the period. */
  repeated: RepeatedRow | null
  /** Why the repeated-work row has no value, in the core's own words. */
  repeated_missing: string | null
  /** The watch's own zone minutes: the source's own basis. */
  source_zones: SourceZoneRow[]
  /** The source's own weeks matrix. */
  source_weeks: SourceWeekRow[]
  /** Where both bases exist. */
  overlap: OverlapRow | null
  /**
   * Why there is no overlap, in the core's own words.
   *
   * **An empty overlap is a fact to state**, not a hole to hide: the
   * period where the app can check its own zone arithmetic against the
   * watch's own record is named, even when there is none.
   */
  overlap_missing: string | null
  /** Whether the store holds runs this answer did not read. */
  more_runs: boolean
  /** Why there is no value at all, in the core's own words. */
  unavailable: string | null
}

/** One adoptable programme. */
export interface BaseRow {
  /** The template's identifier: "pfitzinger.10k.tier1". */
  id: string
  /** The template's name, in the runner's words: "10K tier 1". */
  label: string
  /** The base's name: "Pfitzinger". */
  method: string
  distance: RenderedValue
  /** The distance's name: "10K". */
  distance_label: string
  /** The tier's entry requirement, as the core renders it: "48–67 km". */
  entry_requirement: string | null
  /** The entry requirement, in the core's own words. */
  entry_line: string | null
  /** The lower edge of the entry requirement, in metres. */
  entry_from_m: number | null
  /** The upper edge of the entry requirement, in metres. */
  entry_to_m: number | null
  /** How many weeks the programme runs. */
  weeks: number
}

/**
 * The programmes the app can adopt.
 *
 * **A programme is an intent, not a baseline**: it is what the athlete
 * adopts, and the app never suggests one.
 */
export interface BasesInsight {
  generated_at: string
  /** The four programmes. */
  bases: BaseRow[]
  /**
   * The athlete's own recent weekly mileage, in the core's own words.
   *
   * The fact the tier's entry requirement is stated beside — and the app
   * never compares the two.
   */
  athlete_mileage: string | null
  /** Why there is no programme to adopt, in the core's own words. */
  unavailable: string | null
}

/**
 * The plan in force: the programme, its tier, its dates and what it was
 * paced with.
 *
 * The entry requirement and the athlete's own mileage travel together:
 * three facts, side by side, unranked.
 */
export interface AdoptionInsight {
  generated_at: string
  /** The plan in force. Null when no plan has been adopted. */
  plan: AdoptedPlan | null
  /** Why there is no plan, in the core's own words. */
  unavailable: string | null
}

/** The adopted plan, as the answer carries it. */
export interface AdoptedPlan {
  /** The programme's catalogue id. */
  programme_id: string
  /** The catalogue version the plan was adopted from. */
  programme_version: string
  /** The programme's own name: "half marathon tier 2". */
  label: string
  /** Whose method it is: "Pfitzinger". */
  method: string
  /** The tier the athlete chose: "tier 2". */
  tier: string
  /** The goal race's own distance, as the core renders it. */
  goal_distance: RenderedValue
  /** The distance's name: "half marathon". */
  goal_label: string
  /** The day of the goal race. */
  race_date: string
  /** The day of the goal race, as the app shows it. */
  race_date_label: string
  /** The plan's first day. */
  start_date: string
  /** The plan's last day. */
  end_date: string
  /** The day the plan was adopted. */
  adopted_on: string
  /** Where the plan came from. */
  source: string
  /** What the plan was paced with. */
  pacing: PlanPacing
  /** The tier's entry requirement beside the athlete's own mileage. */
  entry: PlanEntry
}

/** The plan that was adopted. */
export interface AdoptionWritten {
  /** Whether the plan was adopted. */
  adopted: boolean
  /** The plan, as the answer carries it. */
  plan: AdoptedPlan
  generated_at: string
}

/**
 * The athlete's own races, newest first.
 *
 * **The athlete's own facts are never synced.** They are typed into the
 * app and stored locally, and the app never infers a race from the
 * training.
 */
export interface RacesInsight {
  generated_at: string
  /** The races the athlete typed, newest first. */
  races: RaceRow[]
}

/** The choice that was made. */
export interface ChoiceWritten {
  /** The choice, as it is now stored: the one in force. */
  choice: ChoiceRow
  generated_at: string
}

/**
 * The athlete's own heart rates.
 *
 * **The two facts are never blended.** The answer carries them side by
 * side, says which one the bands come from, and states the difference as a
 * fact.
 */
export interface HeartRatesInsight {
  generated_at: string
  /** The declaration in force, when there is one. */
  declaration: DeclarationRow | null
  /** The bands the declaration yields, when it yields any. */
  bands: DeclaredBandsRow | null
  /** The watch's own floors, when a model has been read. */
  watch: WatchBandsRow | null
  /** The difference between the two, boundary by boundary. */
  difference: BoundaryRow[] | null
  /** The declarations that came before, newest first. */
  history: DeclarationRow[]
  /**
   * The five sources a number can come from, in the runner's words.
   *
   * The list is the core's, and the app's picker reads it.
   */
  sources: HeartRateSourceRow[]
  /**
   * The two kinds of heart rate, in the runner's words: "your maximum heart
   * rate" and "your resting heart rate".
   */
  kinds: HeartRateKindRow[]
  /** Why there are no bands, in the core's own words. */
  unavailable: string | null
}

/** The declaration that was written. */
export interface DeclarationWritten {
  /** The declaration, as it is now stored: the one in force. */
  declaration: DeclarationRow
  generated_at: string
}

/**
 * One declaration: the athlete's own statement about their heart rates.
 *
 * **A pair entered together is one declaration**, and the two rows it
 * carries are the two numbers it holds.
 */
export interface DeclarationRow {
  /** The declaration's own id. */
  id: number
  /** The maximum heart rate, when the declaration carries one. */
  max: DeclaredRateRow | null
  /** The resting heart rate, when the declaration carries one. */
  resting: DeclaredRateRow | null
  /** The day the declaration was made. */
  entered_on: string
  /**
   * The day the declaration was made, in the app's own date rendering:
   * "24 Nov 2026".
   */
  entered_on_label: string
  /** The entry, in the core's own words: "entered on 24 Nov 2026". */
  entered: string
  /** The day the declaration was last changed. */
  updated_on: string
  /** Whether this declaration is the one in force. */
  in_force: boolean
}

/**
 * One declared heart rate: the value, where it came from, and the day it
 * was measured.
 */
export interface DeclaredRateRow {
  /**
   * The kind of heart rate, in the runner's words: "your maximum heart
   * rate".
   */
  label: string
  /** The value, in beats per minute. */
  value_bpm: number
  /** The value as the core renders it: "186 bpm". */
  value: RenderedValue | null
  /** Where it came from: "watch", "test", "doctor_lab", "partner_watch" or "typed". */
  source: string
  /** Where it came from, in the runner's words: "from a test". */
  source_label: string
  /** The day it was measured, when the athlete said. */
  measured_on: string | null
  /** The day it was measured, in the app's own date rendering. */
  measured_on_label: string | null
}

/** The bands the athlete's declared heart rates yield. */
export interface DeclaredBandsRow {
  /**
   * The bands, in the runner's words: "the bands from the heart rates you
   * entered".
   */
  label: string
  /** The five floors, in beats per minute. */
  floors: ZoneModelFloors
  /** The maximum heart rate the bands were built on. */
  max_heart_rate: RenderedValue | null
  /** The range the ladder was applied to: "reserve" or "max_share". */
  range: string
  /** The range, in the runner's words. */
  range_label: string
  /** The day the declaration was entered. */
  entered_on: string
  /** The bands' own source line, in the core's words. */
  source: string | null
}

/**
 * The watch's own floors: the configuration in the watch, read once and
 * dated.
 */
export interface WatchBandsRow {
  /**
   * The watch's own zones, in the runner's words: "your watch's zones".
   */
  label: string
  /** The five floors, in beats per minute. */
  floors: ZoneModelFloors
  /** The maximum heart rate the watch's model was built on. */
  max_heart_rate: RenderedValue | null
  /** The day the model was read. */
  read_on: string
  /** The day's own rendering: "3 Nov 2026". */
  read_on_label: string
  /** The model's own source line, in the core's words. */
  source: string | null
}

/** One of the five sources a heart rate can come from. */
export interface HeartRateSourceRow {
  /** The source's name, as it is stored: "watch". */
  key: string
  /** The source, in the runner's words: "from my watch". */
  label: string
}

/** One of the two kinds of heart rate. */
export interface HeartRateKindRow {
  /** The kind's name, as it is stored: "max". */
  key: string
  /** The kind, in the runner's words: "your maximum heart rate". */
  label: string
}

/** The bands the window's moving time falls into. */
/**
 * How a challenge is measured.
 *
 * `week_distance` is the week's own distance; `run_distance` is the
 * week's longest run.
 */
export type ChallengeKind = 'week_distance' | 'run_distance'

/**
 * One challenge of the athlete's own, as the settings carry it.
 */
export interface SettingsChallenge {
  /** How it is measured. */
  kind: ChallengeKind
  /** The target, in kilometres. */
  km: number
  /** The athlete's own words for it, when they gave any. */
  label: string | null
}

/**
 * The challenges: the athlete's own, measured week by week.
 *
 * A week is met or it is not — a fact about the week, not a grade.
 */
export interface ChallengeInsight {
  /** When this answer was produced. */
  generated_at: string
  /** The athlete's local date. */
  date: string
  /** The window the weeks cover. */
  window: ChallengeWindow
  /** The challenges, each with the weeks against it. */
  challenges: ChallengeRow[]
  /** Why there is no value, in the core's words. */
  unavailable: string | null
}

/** The window the challenge grid covers. */
export interface ChallengeWindow {
  /** The first day of the window, in the athlete's own calendar. */
  from: string
  /** The same day, as the api writes a date: "4 May 2026". */
  from_label: string
  /** The last day of the window. */
  to: string
  /** The same day, as the api writes a date. */
  to_label: string
  /** How many weeks the window covers. */
  weeks: number
}

/** One challenge, and the weeks against it. */
export interface ChallengeRow {
  /** The challenge's own key: its kind and its target. */
  key: string
  /** How it is measured: "week_distance" or "run_distance". */
  kind: string
  /** The name the core gives it, or the athlete's own words. */
  label: string
  /** The target, as the core renders it: "40 km". */
  target: RenderedValue
  /** How many weeks in a row it was met, at the end of the window. */
  streak: RenderedValue | null
  /** The weeks, oldest first. */
  weeks: ChallengeWeekRow[]
}

/** One week against one challenge. */
export interface ChallengeWeekRow {
  /** The Monday the week starts on. */
  week_start: string
  /** The same day, as the api writes a date. */
  label: string
  /** Whether the week reached the target. */
  met: boolean
  /** What the week measured, as the core renders it. */
  measured: RenderedValue
}

/**
 * The year: the months, the calendar of the days, and what changed.
 *
 * **The calendar colours days, never places.** A day carries the zone mix
 * of that day's runs and nothing else: no position, no place name and
 * nothing that says where the athlete ran. Where they ran is the heatmap's
 * surface, and it has its own screen and its own disclosure.
 */
export interface YearInsight {
  /** When this answer was produced. */
  generated_at: string
  /** The athlete's local calendar date the answer is about. */
  date: string
  /** The window the answer is about: the last twelve calendar months. */
  window: YearWindow
  /** The period the window is compared against. */
  comparison: YearWindow
  /** The months of the window, oldest first. */
  months: YearMonth[]
  /** Every day of the window, oldest first. */
  days: YearDay[]
  /**
   * What changed, in the core's own sentences.
   *
   * The sentences are written by the core and travel to the app already
   * rendered: the app writes no sentence of its own.
   */
  changed: string[]
}

/** A window of days, as the year names it. */
export interface YearWindow {
  /** The first day of the window, in the athlete's own calendar. */
  from: string
  /** The same day, as the api writes a date: "1 Jun 2025". */
  from_label: string
  /** The last day of the window. */
  to: string
  /** The same day, as the api writes a date: "13 May 2026". */
  to_label: string
  /** How many days the window covers. */
  days: number
}

/** One month of the year. */
export interface YearMonth {
  /** The month's key: "2026-05". */
  key: string
  /** The month's label, as the api writes it: "May 2026". */
  label: string
  /** How many runs fell in the month. */
  sessions: number
  /** The month's distance. Null when the month holds no run. */
  distance: RenderedValue | null
  /** The month's moving time. Null when it holds no run. */
  duration: RenderedValue | null
  /** The month's elevation gain. Null when it holds no run. */
  elevation: RenderedValue | null
  /** The longest run of the month. Null when it holds no run. */
  longest: RenderedValue | null
  /** The training load's form at the month's end. */
  form: RenderedValue | null
  /** The weeks the month touches, oldest first. */
  weeks: YearWeek[]
  /**
   * The mix of the zones the month was trained in: each zone's share of
   * the time in the zones.
   *
   * Empty when the month holds no time in the zones: that is no value,
   * never a row of zeros.
   */
  mix: YearZoneShare[]
  /**
   * The share of the time in the zones that was in the high zones
   * (Z4–Z5). Null when the month holds no time in the zones.
   */
  high: RenderedValue | null
}

/** One calendar week inside a month. */
export interface YearWeek {
  /** The first day of the week — a Monday. */
  start: string
  /** The same day, as the api writes a date: "4 May 2026". */
  start_label: string
  /**
   * The distance of the week's days that fall in the month. Null when the
   * week holds no run in the month.
   */
  distance: RenderedValue | null
  /** How many runs fell in the week's days that are in the month. */
  sessions: number
}

/** One zone's share of the time in the zones. */
export interface YearZoneShare {
  /** The zone, as the core writes it: "Z1". */
  zone: string
  /** The zone's share of the time in the zones. */
  share: RenderedValue | null
}

/** One day of the calendar. */
export interface YearDay {
  /** The day, in the athlete's own calendar. */
  date: string
  /** How many runs fell on the day. */
  sessions: number
  /** The day's distance. Null on a day with no run. */
  distance: RenderedValue | null
  /**
   * The share of the day's time in the zones that was in the high zones
   * (Z4–Z5) — what the calendar colours the day by.
   *
   * Null when the day's runs carry no time in the zones: no value, never
   * a share of zero.
   */
  high: RenderedValue | null
}

/**
 * One sample of a run's film.
 *
 * The sample is the run's own: decoded from its FIT file on read. It
 * carries the zone it sits in, measured against the athlete's **stored**
 * zone model — the model's own, named and dated beside the series.
 */
export interface FilmSample {
  /**
   * The time from the start of the run, as the core renders it: "42 m".
   *
   * The number beside the words is the time in **seconds**, and it is the
   * number the film's timeline draws on.
   */
  elapsed: RenderedValue
  /**
   * The distance from the start, as the core renders it: "3.2 km".
   *
   * The number beside the words is the distance in **metres**.
   */
  distance: RenderedValue | null
  /** The pace over the sample, as the core renders it: "5:24 /km". */
  pace: RenderedValue | null
  /** The heart rate over the sample, as the core renders it: "148 bpm". */
  heart_rate: RenderedValue | null
  /**
   * The zone the sample sits in: the index of the band in the film's
   * `zones`, counted from zero.
   *
   * `null` when the reading is **outside** the zones — at or below the
   * first floor, or not there at all.
   */
  zone: number | null
}

/** One point of the run's ground, on the run's own time axis. */
export interface FilmGround {
  /** The seconds from the start of the run. */
  offset_s: number
  /** The altitude, as the core renders it: "926 m". */
  elevation: RenderedValue
}

/**
 * One split of a run's film.
 *
 * The split carries the time it **began**: a split is a place on the
 * timeline, and the film can be played from it.
 */
export interface FilmSplit {
  /** The lap's number, from one — a lap, not a kilometre. */
  lap_index: number
  /** The seconds from the start of the run to the start of the split. */
  offset_s: number
  /** The split's distance, as the core renders it. */
  distance: RenderedValue
  /** The split's duration, as the core renders it. */
  duration: RenderedValue
  /** The split's pace, as the core renders it. */
  pace: RenderedValue | null
  /** The average heart rate over the split, as the core renders it. */
  heart_rate: RenderedValue | null
}

/** One band of the athlete's zones, as the film draws it. */
export interface FilmZone {
  /** The band's name, as the core names it: "Z2". */
  label: string
  /** The band's range, in the core's words: "106-121 bpm". */
  range: string | null
  /** The band's lower edge, in beats per minute — for drawing. */
  from_bpm: number | null
  /** The band's upper edge, in beats per minute. */
  to_bpm: number | null
}

/**
 * A run's film: the run, playable.
 *
 * The series is the run's own, decoded from its file on read: the samples
 * with their pace, their heart rate and the zone each one sits in, the
 * ground, and the splits. The zone model the zones were measured against
 * is named and dated beside them.
 *
 * **No value is the absence of the whole result.** A run with no file has
 * no series at all, and says why in the core's words.
 */
export interface RunFilm {
  /** The run's id. */
  activity_id: number
  /** The athlete's local date of the run. */
  local_date: string
  /** The basis the series was read on: "samples". */
  basis: string | null
  /** The zone model the zones were measured against, named and dated. */
  model: ZoneModel | null
  /** The five bands of the athlete's zones. */
  zones: FilmZone[]
  /** The name of the space outside the zones, in the core's words. */
  outside_label: string | null
  /** The run's samples. */
  samples: FilmSample[]
  /** The run's ground. */
  ground: FilmGround[]
  /** The run's splits. */
  splits: FilmSplit[]
  /** Why there is no film at all, in the core's words. */
  unavailable: string | null
  /**
   * Why the zones have no value, in the core's words.
   *
   * The film itself may still be there: the pace, the heart rate and the
   * ground do not need a zone model.
   */
  zones_unavailable: string | null
  /** When the answer was produced. */
  generated_at: string
}

/**
 * One check of a run's update, and what it found.
 *
 * The three checks are the core's: groundedness, direction and the
 * register. They are mechanical, and their findings are the core's own
 * words.
 */
export interface UpdateCheck {
  /** The check's name: "groundedness", "direction" or "register". */
  check: string
  /** What the check found, in the core's own words. */
  findings: string[]
}

/**
 * A run's update: the model's own reading of the run.
 *
 * **It is not the product's voice.** It is attributed — the writer and
 * the reviewer are named — and it is kept with the checks it passed and
 * the hash of the payload it was written from, so it can be re-checked
 * rather than believed.
 */
export interface RunUpdate {
  /** The writer's model, by name. */
  writer: string
  /** The reviewer's model, by name. */
  reviewer: string
  /** The SHA-256 of the payload the model was given. */
  payload_hash: string
  /** The model's own words. */
  draft: string
  /** The checks it passed, and what they found. */
  checks: UpdateCheck[]
  /** The verdict: "ship". */
  verdict: string
  /** When the update was made. */
  created_at: string
}

/**
 * The payload of a run.
 */
export interface UpdatePayloadResponse {
  /** The run's id. */
  activity_id: number
  /** The hash of the payload's rendered text. */
  payload_hash: string | null
  /** The payload, when there is one. */
  payload: UpdatePayload | null
  /** The payload as the model reads it: the core's own wording. */
  text: string
  /** Why there is no payload, in the core's own words. */
  unavailable: string | null
  /** When the answer was produced. */
  generated_at: string
}

/**
 * The update's own settings.
 *
 * **The key is not in them.** The settings name the environment variable
 * it is read from, so the key is never in the database and never in an
 * export.
 */
export interface UpdateSettings {
  /** Is the feature on? It is off until the athlete turns it on. */
  enabled: boolean
  /** May anything leave the machine? It may not, until the athlete says so. */
  allow_remote: boolean
  /** The writer's model, by name. */
  writer_model: string
  /** The writer's family. */
  writer_family: string
  /** The base URL the writer is called at. */
  writer_base_url: string
  /** The reviewer's model, by name. */
  reviewer_model: string
  /** The reviewer's family. It has to differ from the writer's. */
  reviewer_family: string
  /** The base URL the reviewer is called at. */
  reviewer_base_url: string
  /** The environment variable the key is read from. */
  key_env: string
  /** When these settings were last written. */
  updated_at: string
  /** When the answer was produced. */
  generated_at: string
}

/**
 * The stored update of a run.
 *
 * A run without an update has none: the answer says so with a null.
 */
export interface RunUpdateResponse {
  /** The run's id. */
  activity_id: number
  /** The update, when the run has one. */
  update: RunUpdate | null
  /** When the answer was produced. */
  generated_at: string
}

/**
 * One fact of the payload: a label and a rendered value.
 */
export interface UpdateFact {
  /** The label, in the core's own words. */
  label: string
  /** The value, as the core renders it. */
  value: string
}

/**
 * One split of the payload.
 */
export interface UpdateSplit {
  /** The lap's number, from one — a lap, not a kilometre marker. */
  lap_index: number
  /** The distance, as the core renders it. */
  distance: string
  /** The pace, as the core renders it. */
  pace: string
  /** The heart rate, as the core renders it. */
  heart_rate: string
}

/**
 * One zone of the payload: the time in the athlete's own zones.
 */
export interface UpdateZone {
  /** The zone's name: "Z1". */
  label: string
  /** The time in the zone, as the core renders it. */
  time: string
  /** The share of the moving time, as the core renders it. */
  share: string
}

/**
 * One observation of the payload: the run's own mechanics.
 */
export interface UpdateObservation {
  /** The label, in the core's own words. */
  label: string
  /** The value, as the core renders it. */
  value: string
  /** The direction of a signed value: "faster", "slower", and so on. */
  direction: string | null
  /** The signed number behind the value, when it has one. */
  signed: number | null
  /** What the number counts: "pace", "heart_rate" or "percent". */
  quantity: string | null
}

/**
 * The payload: the facts a model may use, and nothing else.
 *
 * **Metrics only.** Never the positions, never the athlete's note, never
 * their name: the payload is the answer to "what would be sent".
 */
export interface UpdatePayload {
  /** The writer's model, by name. */
  model: string
  /** The day the run was read, as the api writes a date. */
  read_on: string
  /** The run's own facts. */
  facts: UpdateFact[]
  /** The run's splits. */
  splits: UpdateSplit[]
  /** The time in the athlete's own zones. */
  zones: UpdateZone[]
  /** The run's own mechanics. */
  observations: UpdateObservation[]
}
