/**
 * The typed API client.
 *
 * One client, one frozen route table (`docs/contract/api-paths.md`). No
 * route is added here and no response shape is changed here — this file
 * only says how each frozen route is called and what it answers.
 */

import { fixtureResponse } from '../fixtures'
import type {
  ActivityDetail,
  ActivityListResponse,
  CompareCandidate,
  CompareResponse,
  HealthDaily,
  PlanWeek,
  ProgressInsight,
  SyncStatus,
  TodayInsight,
  VitalsSnapshot,
} from './types'
import {
  ApiError,
  currentHost,
  selectTransport,
  type ApiRequest,
  type HttpMethod,
} from './transport'

export { ApiError }

/** The base every route is served under. */
export const API_BASE = import.meta.env.VITE_API_BASE ?? '/api/v1'

/**
 * The frozen route table.
 *
 * Every route the app is allowed to call, and nothing else. The cut
 * routes — `/network/*`, `/tiles/*`, `/gear`, `/auth/garmin/{login,mfa,
 * developer-key,callback}`, `/auth/{pair-code,pair,devices}`,
 * `/stats/monthly`, `/stats/hr-zones` — are not here and must never be
 * added back.
 */
export const ROUTES = {
  activities: '/activities',
  activity: '/activities/{id}',

  physiologyDaily: '/physiology/daily',
  physiologySleep: '/physiology/sleep',
  physiologyHrv: '/physiology/hrv',
  physiologyVitals: '/physiology/vitals',
  physiologyLatest: '/physiology/latest',
  physiologySources: '/physiology/sources',

  fitnessFingerprint: '/fitness/fingerprint',
  fitnessTrends: '/fitness/trends',
  fitnessZones: '/fitness/zones',

  statsSummary: '/stats/summary',
  statsWeekly: '/stats/weekly',
  statsPaceTrends: '/stats/pace-trends',
  statsElevation: '/stats/elevation',
  statsPersonalRecords: '/stats/personal-records',

  planCurrent: '/plan/current',
  planToday: '/plan/today',
  planWeek: '/plan/week',
  planBlocks: '/plan/blocks',
  planSources: '/plan/sources',
  planAdherence: '/plan/adherence',
  planResults: '/plan/results',
  planImport: '/plan/import',

  insightsToday: '/insights/today',
  insightsProgress: '/insights/progress',
  insightsChips: '/insights/chips',

  compare: '/compare',
  compareCandidates: '/compare/candidates',

  health: '/health',
  healthDaily: '/health/daily',

  syncStatus: '/sync/status',
  syncRun: '/sync/run',
  syncRuns: '/sync/runs',
  syncHistory: '/sync/history',
  syncSettings: '/sync/settings',

  authStatus: '/auth/status',
  authGarminPersonalKey: '/auth/garmin/personal-key',
  authGarminPersonalKeyTest: '/auth/garmin/personal-key/test',
  authGarminFitImport: '/auth/garmin/fit-import',
  authGarminLogout: '/auth/garmin/logout',

  calendar: '/calendar',
} as const

export type RouteName = keyof typeof ROUTES

/** Fill a `{placeholder}` in a frozen route. */
export function route(name: RouteName, params: Record<string, string | number> = {}): string {
  return ROUTES[name].replace(/\{(\w+)\}/g, (match, key: string) =>
    key in params ? String(params[key]) : match,
  )
}

/**
 * The fixture mode.
 *
 * With `?fixtures=1` the app answers from the synthetic fixture set
 * instead of the network, so every screen can be rendered, reviewed and
 * tested without a running backend. The shapes are the same either way.
 *
 * `?fixtures=over` selects the variant whose last run is beyond 10% of the
 * plan's target — the one fixture where the distance comparison appears.
 */
export function fixtureVariant(): string | null {
  if (typeof window === 'undefined') return null
  const params = new URLSearchParams(window.location.search)
  return params.get('fixtures')
}

export function fixtureMode(): boolean {
  return fixtureVariant() !== null
}

const transport = selectTransport()

async function request<T>(
  method: HttpMethod,
  path: string,
  body?: unknown,
  params?: Record<string, string | number | undefined>,
): Promise<T> {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value !== undefined && value !== null) query.set(key, String(value))
  }
  const search = query.toString()
  const url = `${API_BASE}${path}${search ? `?${search}` : ''}`

  if (fixtureMode()) {
    const answer = fixtureResponse(
      path,
      Object.fromEntries(query.entries()),
      fixtureVariant() ?? '',
    )
    if (answer === undefined) {
      throw new ApiError('not_found', `no fixture for ${path}`, 404)
    }
    return answer as T
  }

  const apiRequest: ApiRequest = { method, path: url, body }
  return (await transport(apiRequest)) as T
}

const get = <T>(path: string, params?: Record<string, string | number | undefined>) =>
  request<T>('GET', path, undefined, params)

const post = <T>(path: string, body?: unknown) => request<T>('POST', path, body)

/** The typed client. */
export const api = {
  /** The feed, windowed by date. */
  activities: (params?: { cursor?: string; limit?: number; since?: string }) =>
    get<ActivityListResponse>(
      ROUTES.activities,
      params as Record<string, string | number | undefined>,
    ),
  /** One run. */
  activity: (id: number | string) => get<ActivityDetail>(route('activity', { id })),

  /** The vitals and their directions — the snapshot Today and Body read. */
  vitals: () => get<VitalsSnapshot>(ROUTES.physiologyVitals),
  physiologyDaily: () => get<unknown>(ROUTES.physiologyDaily),
  physiologySleep: () => get<unknown>(ROUTES.physiologySleep),
  physiologyHrv: () => get<unknown>(ROUTES.physiologyHrv),
  physiologyLatest: () => get<unknown>(ROUTES.physiologyLatest),
  physiologySources: () => get<unknown>(ROUTES.physiologySources),

  fitnessFingerprint: () => get<unknown>(ROUTES.fitnessFingerprint),
  fitnessTrends: () => get<unknown>(ROUTES.fitnessTrends),
  fitnessZones: () => get<unknown>(ROUTES.fitnessZones),

  statsSummary: () => get<unknown>(ROUTES.statsSummary),
  statsWeekly: () => get<unknown>(ROUTES.statsWeekly),
  statsPaceTrends: () => get<unknown>(ROUTES.statsPaceTrends),
  statsElevation: () => get<unknown>(ROUTES.statsElevation),
  statsPersonalRecords: () => get<unknown>(ROUTES.statsPersonalRecords),

  planCurrent: () => get<unknown>(ROUTES.planCurrent),
  planToday: () => get<unknown>(ROUTES.planToday),
  planWeek: () => get<PlanWeek>(ROUTES.planWeek),
  planBlocks: () => get<unknown>(ROUTES.planBlocks),
  planSources: () => get<unknown>(ROUTES.planSources),
  planAdherence: () => get<unknown>(ROUTES.planAdherence),
  planResults: () => get<unknown>(ROUTES.planResults),
  planImport: () => post<unknown>(ROUTES.planImport),

  /** Today's readout. */
  insightsToday: () => get<TodayInsight>(ROUTES.insightsToday),
  insightsProgress: () => get<ProgressInsight>(ROUTES.insightsProgress),
  insightsChips: (id: number | string) =>
    get<unknown>(ROUTES.insightsChips, { id: String(id) }),

  /** The metric diff. */
  compare: (a: number | string, b: number | string) =>
    get<CompareResponse>(ROUTES.compare, { a: String(a), b: String(b) }),
  compareCandidates: (id: number | string) =>
    get<CompareCandidate[]>(ROUTES.compareCandidates, { id: String(id) }),

  health: () => get<unknown>(ROUTES.health),
  healthDaily: () => get<HealthDaily>(ROUTES.healthDaily),

  syncStatus: () => get<SyncStatus>(ROUTES.syncStatus),
  syncRun: () => post<unknown>(ROUTES.syncRun),
  syncRuns: () => get<unknown>(ROUTES.syncRuns),
  syncHistory: () => get<unknown>(ROUTES.syncHistory),
  syncSettings: () => get<unknown>(ROUTES.syncSettings),

  authStatus: () => get<unknown>(ROUTES.authStatus),

  /**
   * The Garmin key, typed into the desktop.
   *
   * The desktop's server keeps the key, so the desktop has a field and
   * this call carries the key to the server. The phone has no server and
   * no field: the bridge refuses the key routes, so this call refuses
   * here, before anything is sent. **The key never crosses the bridge.**
   */
  setGarminPersonalKey: async (key: string) => {
    if (currentHost() === 'native') {
      throw new ApiError('garmin_rejected', 'the key is typed on the phone', 0)
    }
    return post<unknown>(ROUTES.authGarminPersonalKey, { key })
  },

  /**
   * The key, checked against Garmin.
   *
   * The desktop's server checks it; the phone's own screen does. So this
   * one is the desktop's as well.
   */
  testGarminPersonalKey: async (key: string) => {
    if (currentHost() === 'native') {
      throw new ApiError('garmin_rejected', 'the key is typed on the phone', 0)
    }
    return post<unknown>(ROUTES.authGarminPersonalKeyTest, { key })
  },

  /**
   * The phone's key setup.
   *
   * The native setup screen owns the key on the phone, and the bridge
   * opens it: asking for the key route with **no key in it** makes the
   * plugin open its own screen and refuse the call. The refusal is the
   * answer — the screen is open — so it is not an error to show.
   *
   * The desktop has no native screen, and no need of one: its field
   * belongs to the Settings screen.
   */
  openGarminKeySetup: async () => {
    if (currentHost() !== 'native') {
      throw new ApiError('unknown', 'the desktop has no native setup screen', 0)
    }
    try {
      await post<unknown>(ROUTES.authGarminPersonalKey)
    } catch (reason) {
      const failure =
        reason instanceof ApiError
          ? reason
          : new ApiError('transport', 'the setup screen did not open', 0)
      // The plugin opens the screen and *then* refuses the call, with a
      // 400: that refusal is how the phone says "the screen is open".
      if (failure.status === 400) return
      throw failure
    }
  },

  importFit: (payload: unknown) => post<unknown>(ROUTES.authGarminFitImport, payload),
  garminLogout: () => post<unknown>(ROUTES.authGarminLogout),
  calendar: () => get<unknown>(ROUTES.calendar),
}
