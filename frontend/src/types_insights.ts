/**
 * The insight routes: `/insights/*`. Rust-native, no Python reference; the
 * shapes are `stride_core::insights`.
 */

export type Tone = 'good' | 'warn' | 'neutral'

export interface InsightRun {
  id: number
  name: string
  date: string
  distance_m: number
  moving_time_s: number
  avg_hr: number | null
}

export interface RunStory {
  id: number
  chip: string
  tone: Tone
  story: string[]
  spike: boolean
  longest_prior_30d_m: number | null
  similar_runs: number
  efficiency_delta_pct: number | null
  pace_delta_s_km: number | null
  hr_delta_bpm: number | null
  bests: string[]
  recovery?: { flags: string[]; normal: number; read: number } | null
}

export interface WeekDay {
  date: string
  distance_m: number
  ran: boolean
  future: boolean
}

export interface WeekSoFar {
  days: WeekDay[]
  distance_m: number
  runs: number
  usual_distance_m: number | null
}

export type TodayState = 'empty' | 'done' | 'go' | 'easy' | 'nudge' | 'comeback'

export interface TodayInsight {
  date: string
  state: TodayState
  headline: string
  reason: string | null
  guardrail: string | null
  longest_30d_m: number | null
  spike_limit_m: number | null
  days_since_last_run: number | null
  usual_gap_days: number | null
  last_run: { run: InsightRun; story: RunStory } | null
  week: WeekSoFar
  recovery_normal: [number, number] | null
}

export interface Chip {
  chip: string
  tone: Tone
}

export interface Fitness {
  headline: string
  delta_pct: number
  recent_runs: number
  earlier_runs: number
  pace_now_s_km: number | null
  pace_then_s_km: number | null
  at_hr: number | null
}

export interface ConsistencyWeek {
  start: string
  runs: number
  distance_m: number
  status: 'active' | 'grace' | 'gap' | 'current'
}

export interface Consistency {
  runs_per_week: number
  current_weeks: number
  best_weeks: number
  grace_available: boolean
  weeks: ConsistencyWeek[]
  active_weeks_this_year: number
}

export interface BestRun {
  label: string
  distance_m: number
  scope: 'all_time' | 'this_year'
  activity_id: number
  date: string
  pace_s_km: number
  run_distance_m: number
}

export interface Milestones {
  total_m: number
  total_runs: number
  runner_since: string | null
  longest_m: number
  tiers_reached_km: number[]
  next_tier_km: number | null
  this_year_m: number
  this_year_runs: number
}

export interface RaceEstimate {
  label: string
  low_s: number
  high_s: number
}

export interface ProgressInsight {
  fitness: Fitness | null
  consistency: Consistency
  bests: BestRun[]
  milestones: Milestones
  race_estimates: RaceEstimate[]
  race_basis: string | null
}
