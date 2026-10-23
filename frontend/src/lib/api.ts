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

const BASE = '/api/v1'

export class ApiError extends Error {
  status: number
  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
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
      else if (Array.isArray(body?.detail)) detail = body.detail.map((d: any) => d?.msg).join(', ')
    } catch {
      /* keep the generic message */
    }
    throw new ApiError(detail, response.status)
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

function qs(params: Record<string, string | number | boolean | undefined | null>): string {
  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null || value === '') continue
    search.set(key, String(value))
  }
  const text = search.toString()
  return text ? `?${text}` : ''
}

export const api = {
  // ---------------------------------------------------------------- auth
  authStatus: () => request<AuthStatus>('/auth/status'),

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
  activities: (params: {
    page?: number
    per_page?: number
    sport?: string
    from?: string
    to?: string
    q?: string
    sort?: string
  } = {}) => request<ActivityListResponse>(`/activities${qs(params)}`),

  activity: (id: number) => request<ActivityDetail>(`/activities/${id}`),

  patchActivity: (id: number, patch: Record<string, unknown>) =>
    request<ActivitySummary>(`/activities/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),

  // ---------------------------------------------------------------- stats
  summary: () => request<SummaryResponse>('/stats/summary'),
  weekly: (weeks = 26) => request<{ items: Bucket[] }>(`/stats/weekly${qs({ weeks })}`),
  monthly: (months = 12) => request<{ items: Bucket[] }>(`/stats/monthly${qs({ months })}`),
  paceTrends: (weeks = 26) => request<{ items: PaceTrend[] }>(`/stats/pace-trends${qs({ weeks })}`),
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
}
