/**
 * Phase-2 types: plans, plan-vs-actual, physiology, fitness/fingerprint.
 * These mirror the frozen API contract in `frontend/CONTRACT.md` §Phase 2.
 */

import type { ActivitySummary } from './types'

export type SessionType =
  'easy' | 'long' | 'tempo' | 'interval' | 'race' | 'recovery' | 'strength' | 'rest'

/**
 * The verdicts of a prescribed plan.
 *
 * `not_yet` is a verdict of its own: a session that has not happened yet is
 * not a failure, and it is never red.
 */
export type Verdict =
  'hit' | 'missed_pace' | 'short' | 'missed' | 'substituted' | 'long' | 'bonus' | 'not_yet'

export interface PlanBlock {
  id: number
  name: string
  start_date: string
  end_date: string | null
  goal: string | null
  status: 'active' | 'completed' | 'archived'
  source: 'manual' | 'imported' | 'garmin'
  notes: string | null
  phase_count: number
  week_count: number
  session_count: number
}

export interface PlanPhase {
  id: number
  block_id: number
  name: string
  start_date: string | null
  end_date: string | null
  target_weekly_km_min: number | null
  target_weekly_km_max: number | null
  target_sessions: number | null
  pace_easy_low: number | null
  pace_easy_high: number | null
  pace_tempo_low: number | null
  pace_tempo_high: number | null
  pace_interval_low: number | null
  pace_interval_high: number | null
  pace_long_low: number | null
  pace_long_high: number | null
  hr_zone_target: number | null
  notes: string | null
}

export interface PlanWeek {
  id: number
  phase_id: number
  week_number: number
  start_date: string | null
  end_date: string | null
  target_km: number | null
  key_workout: string | null
  notes: string | null
}

export interface PlanSession {
  id: number
  week_id: number | null
  block_id: number | null
  date: string
  session_type: SessionType
  title: string | null
  details: string | null
  target_distance_m: number | null
  pace_low_s_km: number | null
  pace_high_s_km: number | null
  hr_zone_target: number | null
  status: string
  source: string
  notes: string | null
}

export interface PlanResult {
  id: number
  plan_session_id: number
  activity_id: number | null
  verdict: Verdict
  distance_delta_m: number | null
  pace_delta_s_km: number | null
  pace_band_hit: boolean | null
  distance_hit: boolean | null
  adherence_score: number | null
  cause: string | null
  cause_confidence: number | null
  note: string | null
}

export interface PlanSessionWithResult extends PlanSession {
  result: PlanResult | null
  activity: ActivitySummary | null
}

export interface PlanWeekDetail {
  week: PlanWeek
  phase: PlanPhase | null
  sessions: PlanSessionWithResult[]
  planned_km: number
  actual_km: number
  planned_count: number
  completed_count: number
  adherence_pct: number | null
}

export interface PlanBlockDetail {
  block: PlanBlock
  phases: (PlanPhase & { weeks: PlanWeekDetail[] })[]
}

export interface PlanAdherenceRow {
  week_key: string
  start_date: string | null
  phase: string | null
  planned_km: number
  actual_km: number
  delta_km: number
  planned_sessions: number
  completed_sessions: number
  missed_sessions: number
  adherence_pct: number | null
}

export interface AdherenceSummary {
  overall_pct: number | null
  planned_km: number
  actual_km: number
  planned_sessions: number
  completed_sessions: number
  hit_sessions: number
  missed_sessions: number
  extra_sessions: number
  by_verdict: Record<Verdict, number>
  rows: PlanAdherenceRow[]
}

export interface PlanToday {
  date: string
  session: PlanSessionWithResult | null
  week: PlanWeekDetail | null
  readiness: Readiness | null
  recommendation: string
  rationale: string | null
  /** True when a training block exists — not the same as "a session today". */
  has_plan: boolean
  /** True when the plan schedules something for today. */
  has_session: boolean
}

export interface ImportPlanResult {
  ok: boolean
  block_id: number | null
  name: string | null
  phases: number
  weeks: number
  sessions: number
  warnings: string[]
  error: string | null
}

export interface DailyPhysiology {
  date: string
  sleep_s: number | null
  sleep_score: number | null
  deep_sleep_s: number | null
  rem_s: number | null
  light_sleep_s: number | null
  awake_s: number | null
  hrv_ms: number | null
  hrv_status: string | null
  resting_hr: number | null
  stress_avg: number | null
  stress_max: number | null
  body_battery_min: number | null
  body_battery_avg: number | null
  body_battery_max: number | null
  steps: number | null
  training_readiness: number | null
  acute_load: number | null
  chronic_load: number | null
  vo2max: number | null
  endurance_score: number | null
  running_tolerance: number | null
  source: string
}

export interface ReadinessDriver {
  key: string
  label: string
  value: number | null
  /** The unit the value is measured in — 'h', 'ms', 'bpm', or empty. */
  unit: string
  baseline: number | null
  delta: number | null
  contribution: number
  detail: string | null
}

export interface Readiness {
  date: string
  score: number
  grade: 'peak' | 'good' | 'fair' | 'low' | 'unknown'
  drivers: ReadinessDriver[]
  recommendation: string | null
  rationale: string | null
}

/**
 * The readiness fields a chart draws: the score and the grade.
 *
 * The full readiness row also carries every driver with its baseline, its
 * delta and a sentence of prose — about 600 bytes a day, which is 200 KB
 * for a year of trend data that the trend never looks at. The drivers are
 * still available for the latest day, through the latest endpoint.
 */
export interface ReadinessSummary {
  date: string
  score: number | null
  grade: 'peak' | 'good' | 'fair' | 'low' | 'unknown'
}

/** The daily fields the physiology table draws, out of thirty-odd. */
export interface DailyPhysiologySummary {
  date: string
  body_battery_avg: number | null
  stress_avg: number | null
  acute_load: number | null
  chronic_load: number | null
  steps: number | null
}

/** The HRV fields the chart draws, without the unused status label. */
export interface HrvSummary {
  date: string
  hrv_ms: number | null
  baseline: number | null
  resting_hr: number | null
  rhr_baseline: number | null
}

export interface PhysiologySources {
  available: boolean
  reason: string | null
  auth_method: string | null
  endpoints: { name: string; available: boolean; note: string | null }[]
  last_synced_at: string | null
  days_available: number
}

export interface SleepPoint {
  date: string
  sleep_s: number | null
  sleep_score: number | null
  deep_sleep_s: number | null
  rem_s: number | null
  light_sleep_s: number | null
  awake_s: number | null
}

export interface HrvPoint {
  date: string
  hrv_ms: number | null
  status: string | null
  baseline: number | null
  resting_hr: number | null
  rhr_baseline: number | null
}

export interface Fingerprint {
  activity_id: number
  /** Time actually spent running — walking and standing are excluded. */
  running_time_s: number | null
  walking_time_s: number | null
  stopped_time_s: number | null
  /** Heart-rate rise between the first and second half of the running. */
  hr_drift_bpm: number | null
  /** Pace fade between the first and second half of the running (positive = slower). */
  pace_drift_pct: number | null
  first_half_hr: number | null
  second_half_hr: number | null
  first_half_pace_s_km: number | null
  second_half_pace_s_km: number | null
  activity_name: string | null
  local_date: string | null
  distance_m: number
  pace_s_per_km: number | null
  cadence_avg: number | null
  cadence_cv: number | null
  cadence_drift_pct: number | null
  hr_avg: number | null
  hr_max: number | null
  hr_drift_pct: number | null
  efficiency_first: number | null
  efficiency_last: number | null
  decoupling_pct: number | null
  pace_cv: number | null
  negative_split_pct: number | null
  stride_length_cm: number | null
  form_score: number | null
  zone_distribution: Record<string, number> | null
}

/**
 * The fingerprint fields the running-form screen draws.
 *
 * A fingerprint row carries twenty-nine fields — the first-half and
 * second-half splits, the efficiency pair, the zone distribution — and the
 * screen shows thirteen of them. Over a year of runs the unused half is
 * most of the JSON.
 */
export interface FingerprintSummary {
  activity_id: number
  activity_name: string | null
  local_date: string | null
  distance_m: number
  pace_s_per_km: number | null
  cadence_avg: number | null
  cadence_cv: number | null
  hr_drift_pct: number | null
  decoupling_pct: number | null
  pace_drift_pct: number | null
  form_score: number | null
  walking_time_s: number | null
  stopped_time_s: number | null
}

export interface FitnessTrend {
  metric:
    | 'cadence_avg'
    | 'cadence_cv'
    | 'cadence_drift_pct'
    | 'hr_drift_pct'
    | 'decoupling_pct'
    | 'efficiency_first'
    | 'pace_cv'
    | 'stride_length_cm'
    | 'form_score'
  label: string
  unit: string
  better: 'lower' | 'higher'
  points: { date: string; value: number | null; activity_id: number }[]
  baseline: number | null
  trend_pct: number | null
}

export interface ZoneShare {
  zone_no: number
  name: string
  low_bpm: number | null
  high_bpm: number | null
  time_s: number
  percent: number
}

export interface GearItem {
  id: number
  uuid: string
  name: string
  brand: string | null
  model: string | null
  status: 'active' | 'retired'
  distance_m: number
  activity_count: number
  first_used: string | null
  last_used: string | null
  retire_at_m: number
  retired: boolean
}

/* ------------------------------------------------------------ comparison */

export type CompareFormat = 'distance' | 'duration' | 'pace' | 'elevation' | 'hr' | 'number'

export interface CompareParameter {
  key: string
  label: string
  unit: string
  better: 'lower' | 'higher' | 'neutral'
  format: CompareFormat
}

export interface CompareCandidate {
  id: number
  name: string
  local_date: string
  distance_m: number
  pace_s_per_km: number | null
  avg_hr: number | null
  elevation_gain_m: number
  has_fingerprint: boolean
  has_profile: boolean
  source: string
}

/** One point of a run, resampled onto the shared distance axis. */
export interface CompareProfilePoint {
  distance_m: number
  /** 0-100, the percentage of the run's distance. */
  pct: number
  hr: number | null
  cadence: number | null
  pace_s_per_km: number | null
  elevation: number | null
}

export interface CompareRun {
  id: number
  name: string
  local_date: string
  day_of_week: string | null
  sport_type: string
  source: string
  distance_m: number
  moving_time_s: number
  moving_time_label: string | null
  elevation_gain_m: number
  avg_hr: number | null
  max_hr: number | null
  calories: number | null
  pace_s_per_km: number | null
  has_geo: boolean
  values: Record<string, number | null>
  profile: CompareProfilePoint[]
}

export interface CompareResponse {
  runs: CompareRun[]
  parameters: CompareParameter[]
  missing: number[]
}

/* --------------------------------------------------------------- history */

export interface HistorySettings {
  /** Months of activity history to pull; null means everything. */
  history_months: number | null
  /** Days of wellness history to pull. */
  health_history_days: number | null
  sync_days: number
  limits: { activities: string; health: string }
}

export interface HistoryLoadResult {
  started: boolean
  activities_months: number | null
  health_days: number | null
  reason?: string
}

/* ---------------------------------------------------------- plan sources */

/**
 * Where a plan came from.
 *
 * Two kinds, and they are not the same kind of thing:
 *
 * * `prescribed: false` — the **inferred** plan, derived from the athlete's
 *   own history: their weekly volume, their rest days, their long-run day,
 *   their quality day and their usual session per weekday. It is a
 *   *baseline*, not a prescription, and it is compared with *less than
 *   usual*, *more than usual* and *on a usual day*.
 * * `prescribed: true` — the plan the athlete wrote down: Garmin's
 *   scheduled workouts or an imported document. It is compared with *hit*
 *   and *missed*.
 *
 * The two vocabularies are never mixed on one screen.
 */
export interface PlanSource {
  /** Which source: `inferred`, `garmin`, `imported`, … */
  source: string
  /** Whether the plan was written down by the athlete. */
  prescribed: boolean
  /** What the source says about itself. */
  description: string
  /** Whether there was enough history to infer a pattern. */
  sufficient?: boolean
  /** Whether the source could answer at all — the Garmin one needs an account. */
  available?: boolean
  /** Why it declined, when it declined. A sentence, not an error. */
  reason?: string | null
  /** Weeks of history the inference saw. */
  history_weeks?: number | null
  /** Runs the inference saw. */
  history_runs?: number | null
  /** The window the inference looked at. */
  window_start?: string | null
  window_end?: string | null
  /** The usual weekly volume, in metres. */
  weekly_volume_m?: number | null
  /** The lightest week in the window. */
  weekly_volume_low_m?: number | null
  /** The heaviest week in the window. */
  weekly_volume_high_m?: number | null
  /** The weekdays the athlete does not run. */
  rest_days?: string[]
  /** The weekday of the usual long run. */
  long_run_day?: string | null
  /** The weekday of the usual quality run. */
  quality_day?: string | null
  /** The usual distance per weekday, in metres. */
  usual_by_weekday?: Record<string, number | null>
  /** The sessions of the plan. */
  sessions?: PlanSourceSession[]
}

/** One session of a plan source. */
export interface PlanSourceSession {
  id?: number | null
  date: string
  session_type: string
  title: string | null
  details?: string | null
  target_distance_m: number | null
  pace_low_s_km: number | null
  pace_high_s_km: number | null
  /** The verdict of this session, judged by its source's own ruler. */
  verdict?: string | null
  /** The words the athlete reads. */
  verdict_label?: string | null
  /** The sentence the baseline produces. */
  message?: string | null
  result?: PlanSourceResult | null
  activity?: PlanSourceActivity | null
}

/**
 * The verdict of one session.
 *
 * The words are in the payload: `verdict_label` is what the athlete reads,
 * and for an inferred plan it is *less than usual* and never *missed*.
 */
export interface PlanSourceResult {
  verdict: string
  verdict_label?: string | null
  message?: string | null
  cause?: string | null
  cause_confidence?: number | null
  usual_distance_m?: number | null
  actual_distance_m?: number | null
  delta_m?: number | null
  pace_delta_s_km?: number | null
  activity_id?: number | null
}

/** The run a session was matched with. */
export interface PlanSourceActivity {
  id: number
  name: string
  distance_m: number
  pace_s_per_km: number | null
}
