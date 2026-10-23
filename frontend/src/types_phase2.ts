/**
 * Phase-2 types: plans, plan-vs-actual, physiology, fitness/fingerprint.
 * These mirror the frozen API contract in `frontend/CONTRACT.md` §Phase 2.
 */

import type { ActivitySummary } from './types'

export type SessionType =
  'easy' | 'long' | 'tempo' | 'interval' | 'race' | 'recovery' | 'strength' | 'rest'

export type Verdict = 'hit' | 'missed_pace' | 'short' | 'missed' | 'substituted' | 'bonus'

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
