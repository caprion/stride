/** Shared API types. These mirror the FastAPI schemas exactly. */

export type SportType =
  | 'running'
  | 'trail_running'
  | 'treadmill_running'
  | 'virtualrun'
  | 'walking'
  | 'hiking'
  | 'cycling'
  | 'swimming'
  | 'strength_training'
  | string

export interface ActivitySummary {
  id: number
  source: string
  external_id: string
  name: string
  sport_type: SportType
  description: string | null
  start_time: string | null
  local_date: string
  day_of_week: string | null
  distance_m: number
  moving_time_s: number
  moving_time_label: string | null
  elapsed_time_s: number
  elevation_gain_m: number
  avg_hr: number | null
  max_hr: number | null
  calories: number | null
  avg_cadence: number | null
  max_cadence: number | null
  avg_speed_ms: number | null
  max_speed_ms: number | null
  pace_s_per_km: number | null
  pace_label: string | null
  perceived_exertion: number | null
  device: string | null
  gear: string | null
  is_trail: boolean
  is_commute: boolean
  /** The original activity in Garmin Connect — sharing is Garmin's business. */
  garmin_url: string | null
  has_geo: boolean
  route_preview: [number, number][] | null
  lap_count: number
  /** Time spent actually running (walking and standing are excluded). */
  running_time_s: number | null
  walking_time_s: number | null
  stopped_time_s: number | null
  running_distance_m: number | null
}

export interface Lap {
  lap_index: number
  distance_m: number
  distance_label: string
  duration_s: number
  duration_label: string | null
  avg_speed_ms: number | null
  pace_s_per_km: number | null
  pace_label: string | null
  avg_hr: number | null
  max_hr: number | null
  avg_cadence: number | null
  max_cadence: number | null
  stride_length_m: number | null
  elevation_gain_m: number | null
  elevation_loss_m: number | null
  start_elevation_m: number | null
  end_elevation_m: number | null
}

export interface Stream {
  seconds: number[]
  values: (number | null)[]
  unit: string | null
}

export interface HrZone {
  zone_no: number
  name: string
  low_bpm: number | null
  high_bpm: number | null
  time_s: number
  time_label: string | null
}

export interface ActivityDetail extends ActivitySummary {
  laps: Lap[]
  streams: Record<string, Stream>
  hr_zones: HrZone[]
  geo_points: [number, number][]
}

export interface ActivityListResponse {
  items: ActivitySummary[]
  total: number
  page: number
  per_page: number
  pages: number
}

export interface Bucket {
  label: string
  activity_count: number
  distance_m: number
  elevation_gain_m: number
  moving_time_s: number
  moving_time_label: string | null
  avg_pace_s_per_km: number | null
  avg_pace_label: string | null
  avg_hr: number | null
  calories: number | null
  week_key?: string
  start_date?: string | null
  month_key?: string
}

export interface PersonalRecord {
  label: string
  value: number
  display: string
  unit: string
  better: 'lower' | 'higher'
  activity_id: number
  activity_name: string
  activity_date: string
}

export interface Streaks {
  current_weeks: number
  longest_weeks: number
  total_active_weeks: number
  current_week_key: string
}

export interface SportBreakdown {
  sport_type: string
  activity_count: number
  distance_m: number
  elevation_gain_m: number
  moving_time_s: number
}

export interface SummaryResponse {
  totals: Bucket
  this_year: Bucket
  this_month: Bucket
  this_week: Bucket
  personal_records: PersonalRecord[]
  streaks: Streaks
  sport_breakdown: SportBreakdown[]
  recent: ActivitySummary[]
}

export interface PaceTrend {
  week_key: string
  start_date: string | null
  run_count: number
  median_pace_s_per_km: number | null
  median_pace_label: string | null
  best_pace_s_per_km: number | null
  best_pace_label: string | null
  worst_pace_s_per_km: number | null
  long_run_pace_s_per_km: number | null
  long_run_pace_label: string | null
}

export interface HrZoneSummary {
  zone_no: number
  name: string
  low_bpm: number | null
  high_bpm: number | null
  time_s: number
  percent: number
}

export interface HrZonesResponse {
  zones: HrZoneSummary[]
  total_time_s: number
  peak_hr: number
  weeks: number
}

export interface HeatmapDay {
  date: string
  distance_m: number
  activity_count: number
}

export interface HeatmapResponse {
  year: number
  days: HeatmapDay[]
}

export interface CalendarDay {
  date: string
  day: number
  activities: ActivitySummary[]
  distance_m: number
  moving_time_s: number
}

export interface CalendarResponse {
  year: number
  month: number
  days: CalendarDay[]
  total_distance_m: number
  activity_count: number
}

export interface AuthStatus {
  garmin_connected: boolean
  garmin_method: 'developer_key' | 'account' | null
  athlete: Record<string, unknown>
  has_developer_key: boolean
  has_account_credentials: boolean
  pending_authorization: boolean
  network_route: 'auto' | 'direct' | 'exit_node'
  exit_node: string | null
  sync_days: number
  auto_sync: boolean
  sync_interval_minutes: number
  last_sync_at: string | null
  network?: NetworkStatus
}

export interface NetworkStatus {
  tailscale_available: boolean
  exit_node: string | null
  exit_node_active: boolean
  public_ip: string | null
  public_org: string | null
  route?: string
  effective_route?: string
  resolved_exit_node?: string | null
  nodes?: { id: string; ip: string; hostname: string; online: boolean }[]
  public_identity?: { ip: string | null; org: string | null }
}

export interface SyncStatus {
  running: boolean
  started_at: string | null
  message: string | null
  error?: string
  last_sync_at: string | null
  connected: boolean
  method: string | null
}

export interface SyncRun {
  id: number
  source: string
  started_at: string | null
  finished_at: string | null
  status: 'running' | 'ok' | 'error'
  days: number
  activities_found: number
  activities_new: number
  activities_updated: number
  message: string | null
  error: string | null
}

export interface DailyMetric {
  date: string
  resting_hr: number | null
  max_hr: number | null
  sleep_s: number | null
  deep_sleep_s: number | null
  hrv_ms: number | null
  stress: number | null
  steps: number | null
  body_battery: number | null
  readiness: number | null
  distance_m: number
}
