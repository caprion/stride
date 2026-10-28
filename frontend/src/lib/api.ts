/**
 * Typed API client.
 *
 * Every call goes through `request`, so error handling and the base path live
 * in exactly one place. The backend is served from the same origin in
 * production and proxied by Vite in development.
 */

import type {
  ActivityDetail,
  ActivityListResponse,
  ActivitySummary,
  AuthStatus,
  CalendarResponse,
  HeatmapResponse,
  HrZonesResponse,
  NetworkStatus,
  PaceTrend,
  PersonalRecord,
  SummaryResponse,
  SyncRun,
  SyncStatus,
  Bucket,
} from '../types'
import type {
  AdherenceSummary,
  CompareCandidate,
  CompareResponse,
  HistoryLoadResult,
  HistorySettings,
  DailyPhysiology,
  DailyPhysiologySummary,
  FitnessTrend,
  Fingerprint,
  GearItem,
  HrvSummary,
  ImportPlanResult,
  PlanBlock,
  PlanBlockDetail,
  PlanPhase,
  PlanResult,
  PlanSession,
  PlanToday,
  PlanWeek,
  PlanWeekDetail,
  PhysiologySources,
  Readiness,
  ReadinessSummary,
  SleepPoint,
  ZoneShare,
} from '../types_phase2'

const BASE = '/api/v1'

export class ApiError extends Error {  status: number
  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

/** The ISO date `days` away from `iso`, in UTC. */
function shiftDate(iso: string, days: number): string {
  const date = new Date(`${iso}T00:00:00Z`)
  date.setUTCDate(date.getUTCDate() + days)
  return date.toISOString().slice(0, 10)
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE}${path}`, {
    headers: init?.body instanceof FormData ? {} : { 'Content-Type': 'application/json' },
    ...init,
  })
  if (!response.ok) {
    let detail = `Request failed (${response.status})`
    try {
      const body = await response.json()
      if (typeof body?.detail === 'string') detail = body.detail
      else if (Array.isArray(body?.detail))
        detail = body.detail.map((d: any) => d?.msg).join(', ')
    } catch {
      /* keep the generic message */
    }
    throw new ApiError(detail, response.status)
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

function qs(params: Record<string, string | number | boolean | undefined | null>): string {  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null || value === '') continue
    search.set(key, String(value))
  }
  const text = search.toString()
  return text ? `?${text}` : ''
}

export interface PairedDevice {
  id: string
  name: string | null
  created_at: string | null
  last_seen_at: string | null
}

export const api = {
  // ---------------------------------------------------------------- auth
  authStatus: () => request<AuthStatus>('/auth/status'),

  /**
   * Exchange a single-use pairing code for the session cookie. Used when the
   * browser is not on the same machine as the server — the phone on the
   * tailnet, for instance.
   */
  pair: (code: string) => request<void>('/auth/pair', { method: 'POST', body: JSON.stringify({ code }) }),

  /** The devices that have been paired to this instance. */
  devices: () => request<{ devices: PairedDevice[] }>('/auth/devices'),

  /** Revoke a paired device — a lost phone, say. */
  revokeDevice: (id: string) =>
    request<{ ok: boolean }>(`/auth/devices/${id}`, { method: 'DELETE' }),

  startDeveloperKey: (consumer_key: string, consumer_secret: string) =>
    request<{ authorize_url: string; request_token: string; instructions: string }>(
      '/auth/garmin/developer-key',
      { method: 'POST', body: JSON.stringify({ consumer_key, consumer_secret }) },
    ),

  exchangeDeveloperKey: (oauth_token: string, oauth_verifier: string) =>
    request<{ ok: boolean }>('/auth/garmin/exchange', {
      method: 'POST',
      body: JSON.stringify({ oauth_token, oauth_verifier }),
    }),

  login: (email: string, password: string) =>
    request<{ status: string; mfa_token?: string }>('/auth/garmin/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),

  submitMfa: (mfa_token: string, code: string) =>
    request<{ status: string }>('/auth/garmin/mfa', {
      method: 'POST',
      body: JSON.stringify({ mfa_token, code }),
    }),

  connectPersonalKey: (api_key: string) =>
    request<{
      ok: boolean
      garmin_method: string
      athlete: Record<string, unknown>
      sources: Record<string, boolean>
    }>('/auth/garmin/personal-key', { method: 'POST', body: JSON.stringify({ api_key }) }),

  testPersonalKey: (api_key: string) =>
    request<{ ok: boolean; athlete: Record<string, unknown> }>(
      '/auth/garmin/personal-key/test',
      {
        method: 'POST',
        body: JSON.stringify({ api_key }),
      },
    ),

  logout: () => request<{ ok: boolean }>('/auth/garmin/logout', { method: 'POST' }),

  importFit: (files: File[]) => {
    const form = new FormData()
    files.forEach((file) => form.append('files', file))
    return request<{ imported: number; activities: any[]; errors: any[] }>(
      '/auth/garmin/fit-import',
      { method: 'POST', body: form },
    )
  },

  // ------------------------------------------------------------ activities
  activities: (
    params: {
      page?: number
      per_page?: number
      sport?: string
      from?: string
      to?: string
      q?: string
      sort?: string
    } = {},
  ) => request<ActivityListResponse>(`/activities${qs(params)}`),

  activity: (id: number) => request<ActivityDetail>(`/activities/${id}`),

  /**
   * The runs either side of one activity, and nothing else.
   *
   * The neighbour links need four numbers per run, but the list endpoint
   * answers with the whole summary — the route preview included — which is
   * two kilobytes a run and half a megabyte for a year of history. So a
   * window around the activity is fetched first, and the whole list is only
   * asked for when the window cannot answer the question: when the run is
   * not in it, when a neighbour may be just outside it, or when there are
   * more runs than one page. The two links are the same either way.
   */
  activityNeighbours: async (activityId: number, localDate?: string | null) => {
    const slim = (items: ActivitySummary[]) =>
      items.map((item) => ({
        id: item.id,
        name: item.name,
        local_date: item.local_date,
        distance_m: item.distance_m,
      }))
    const wholeList = async () => slim((await api.activities({ per_page: 200 })).items)
    if (!localDate) return wholeList()

    const response = await api.activities({
      per_page: 200,
      from: shiftDate(localDate, -30),
      to: shiftDate(localDate, 30),
    })
    // More runs than a single page: the list the app has always used is
    // the most recent 200, so the window cannot stand in for it.
    if (response.total > 200) return wholeList()

    const sorted = [...response.items].sort((a, b) =>
      a.local_date === b.local_date ? a.id - b.id : a.local_date < b.local_date ? -1 : 1,
    )
    const index = sorted.findIndex((item) => item.id === activityId)
    // The run is not in the window, or it sits at its edge: a neighbour may
    // be just outside, so ask for the whole list as the app always has.
    if (index === -1 || index === 0 || index === sorted.length - 1) return wholeList()
    return slim(sorted)
  },

  patchActivity: (id: number, patch: Record<string, unknown>) =>
    request<ActivitySummary>(`/activities/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(patch),
    }),

  // ---------------------------------------------------------------- stats
  summary: () => request<SummaryResponse>('/stats/summary'),
  weekly: (weeks = 26) => request<{ items: Bucket[] }>(`/stats/weekly${qs({ weeks })}`),
  monthly: (months = 12) => request<{ items: Bucket[] }>(`/stats/monthly${qs({ months })}`),
  paceTrends: (weeks = 26) =>
    request<{ items: PaceTrend[] }>(`/stats/pace-trends${qs({ weeks })}`),
  hrZones: (weeks = 12) => request<HrZonesResponse>(`/stats/hr-zones${qs({ weeks })}`),
  personalRecords: () => request<{ items: PersonalRecord[] }>('/stats/personal-records'),
  heatmap: (year?: number) => request<HeatmapResponse>(`/stats/heatmap${qs({ year })}`),

  // -------------------------------------------------------------- calendar
  calendar: (year: number, month: number) =>
    request<CalendarResponse>(`/calendar${qs({ year, month })}`),

  // ------------------------------------------------------------------ sync
  syncStatus: () => request<SyncStatus>('/sync/status'),
  runSync: (days?: number) =>
    request<{ started: boolean; reason?: string }>('/sync/run', {
      method: 'POST',
      body: JSON.stringify({ days, background: true }),
    }),
  syncRuns: () => request<{ items: SyncRun[] }>('/sync/runs'),
  syncSettings: (patch: Record<string, unknown>) =>
    request<AuthStatus>('/sync/settings', { method: 'PATCH', body: JSON.stringify(patch) }),

  // --------------------------------------------------------------- network
  networkStatus: () => request<NetworkStatus>('/network/status'),
  setRoute: (route: 'direct' | 'exit_node') =>
    request<{ ok: boolean; message: string; status: NetworkStatus }>('/network/route', {
      method: 'POST',
      body: JSON.stringify({ route }),
    }),

  // ------------------------------------------------------------------ plan
  planBlocks: () => request<{ items: PlanBlock[] }>('/plan/blocks'),
  planBlock: (id: number) => request<PlanBlockDetail>(`/plan/blocks/${id}`),
  createPlanBlock: (payload: Partial<PlanBlock>) =>
    request<PlanBlock>('/plan/blocks', { method: 'POST', body: JSON.stringify(payload) }),
  createPlanPhase: (blockId: number, payload: Partial<PlanPhase>) =>
    request<PlanPhase>(`/plan/blocks/${blockId}/phases`, {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  createPlanWeek: (phaseId: number, payload: Partial<PlanWeek>) =>
    request<PlanWeek>(`/plan/phases/${phaseId}/weeks`, {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  createPlanSession: (weekId: number, payload: Partial<PlanSession>) =>
    request<PlanSession>(`/plan/weeks/${weekId}/sessions`, {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  importPlan: (payload: { format: string; content: string; name?: string }) =>
    request<ImportPlanResult>('/plan/import', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  planCurrent: () =>
    request<{ block: PlanBlockDetail | null; week: PlanWeekDetail | null }>('/plan/current'),

  /**
   * The current block and week, without the phases.
   *
   * The block arrives with every phase of the plan attached — a hundred
   * kilobytes of weeks and sessions, none of which the plan screen draws.
   * Dropping them here keeps them out of the query cache, which is what the
   * renderer holds while the athlete browses.
   */
  planCurrentSummary: async () => {
    const data = await request<{
      block: PlanBlockDetail | null
      week: PlanWeekDetail | null
    }>('/plan/current')
    return {
      block: data.block ? { ...data.block, phases: [] } : null,
      week: data.week,
    }
  },
  planWeek: (date: string) => request<PlanWeekDetail>(`/plan/week?date=${date}`),
  planResults: (from?: string, to?: string) =>
    request<{ items: PlanResult[] }>(`/plan/results${qs({ from, to })}`),
  planAdherence: (from?: string, to?: string) =>
    request<AdherenceSummary>(`/plan/adherence${qs({ from, to })}`),
  planToday: () => request<PlanToday>('/plan/today'),

  // ------------------------------------------------------------ physiology
  // The physiology endpoints answer for a year of days. The full rows carry
  // every Garmin field, and the readiness row carries seven drivers with
  // their baselines and prose — hundreds of kilobytes a phone has to parse
  // and hold. Each of these asks for the fields the screen actually draws;
  // the field list is a required argument so the trim is visible at the
  // call site rather than hidden in the client.
  physiologyDaily: (fields: string, from?: string, to?: string) =>
    request<{ items: DailyPhysiologySummary[] }>(`/physiology/daily${qs({ from, to, fields })}`),
  physiologyLatest: () =>
    request<{ day: DailyPhysiology | null; readiness: Readiness | null }>('/physiology/latest'),
  physiologySleep: (from?: string, to?: string, fields?: string) =>
    request<{ items: SleepPoint[] }>(`/physiology/sleep${qs({ from, to, fields })}`),
  physiologyHrv: (fields: string, from?: string, to?: string) =>
    request<{ items: HrvSummary[]; baseline: number | null }>(
      `/physiology/hrv${qs({ from, to, fields })}`,
    ),
  physiologyReadiness: (fields: string, from?: string, to?: string) =>
    request<{ items: ReadinessSummary[] }>(`/physiology/readiness${qs({ from, to, fields })}`),
  physiologySources: () => request<PhysiologySources>('/physiology/sources'),

  // --------------------------------------------------------------- fitness
  fitnessFingerprint: (from?: string, to?: string) =>
    request<{ items: Fingerprint[] }>(`/fitness/fingerprint${qs({ from, to })}`),

  /**
   * The fingerprint list, projected to the fields the screen draws.
   *
   * The full row carries twenty-nine fields per run; the running-form
   * screen draws thirteen of them. The projection happens here so the
   * unused half never reaches the query cache.
   */
  fitnessFingerprintSummary: async (from?: string, to?: string) => {
    const data = await request<{ items: Fingerprint[] }>(
      `/fitness/fingerprint${qs({ from, to })}`,
    )
    return {
      items: data.items.map((item) => ({
        activity_id: item.activity_id,
        activity_name: item.activity_name,
        local_date: item.local_date,
        distance_m: item.distance_m,
        pace_s_per_km: item.pace_s_per_km,
        cadence_avg: item.cadence_avg,
        cadence_cv: item.cadence_cv,
        hr_drift_pct: item.hr_drift_pct,
        decoupling_pct: item.decoupling_pct,
        pace_drift_pct: item.pace_drift_pct,
        form_score: item.form_score,
        walking_time_s: item.walking_time_s,
        stopped_time_s: item.stopped_time_s,
      })),
    }
  },
  fitnessActivity: (activityId: number) =>
    request<Fingerprint & { zones: ZoneShare[] }>(`/fitness/activity/${activityId}`),
  fitnessTrends: (metric?: string, from?: string, to?: string) =>
    request<FitnessTrend>(`/fitness/trends${qs({ metric, from, to })}`),
  fitnessZones: (from?: string, to?: string) =>
    request<{ zones: ZoneShare[]; total_time_s: number }>(`/fitness/zones${qs({ from, to })}`),
  gearList: () => request<{ items: GearItem[] }>('/gear'),

  // ------------------------------------------------------------- compare
  historySettings: () => request<HistorySettings>('/sync/history'),
  loadHistory: (activities: number | null, health_days: number | null) =>
    request<HistoryLoadResult>('/sync/history', {
      method: 'POST',
      body: JSON.stringify({ activities, health_days, background: true }),
    }),

  compareCandidates: (limit = 300) =>
    request<{ items: CompareCandidate[] }>(`/compare/candidates${qs({ limit })}`),
  compareRuns: (ids: number[]) =>
    request<CompareResponse>(`/compare${qs({ ids: ids.join(',') })}`),
}
