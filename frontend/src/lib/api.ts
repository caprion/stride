/**
 * The typed API client.
 *
 * One client, one frozen route table (`docs/contract/api-paths.md`). No
 * route is added here and no response shape is changed here — this file
 * only says how each frozen route is called and what it answers.
 */

import { fixtureResponse, FixtureFailure } from '../fixtures'
import type {
  ActivityCalendar,
  ActivityDetail,
  ActivityListResponse,
  AdoptionInsight,
  AdoptionWritten,
  BasesInsight,
  ChoiceWritten,
  CompareCandidate,
  CompareResponse,
  ConnectionStatus,
  CurveInsight,
  DeclarationWritten,
  FitImportResult,
  FitnessZones,
  Fingerprint,
  Health,
  HealthDaily,
  HeartRatesInsight,
  HeatmapInsight,
  KeyConnectResult,
  KeyTestResult,
  LoadInsight,
  NoteResult,
  PacesInsight,
  PlanImportResult,
  PlanWeek,
  ProgressInsight,
  RacesInsight,
  RollingInsight,
  RunFilm,
  SettingsDeleteResult,
  SettingsExportResult,
  RunUpdateResponse,
  SettingsState,
  SleepDays,
  StimulusInsight,
  SyncStatus,
  ChallengeInsight,
  ChallengeKind,
  StreakInsight,
  SyncRunResult,
  ThresholdInsight,
  TodayInsight,
  UpdatePayloadResponse,
  UpdateSettings,
  VitalsSnapshot,
  YearInsight,
} from './types'
import {
  ApiError,
  API_ERROR_KINDS,
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
  activityFit: '/activities/{id}/fit',
  /**
   * The run's film: the run's own series, playable.
   *
   * The samples with their pace, their heart rate and the zone each one
   * sits in, the ground, the splits, and the zone model the zones were
   * measured against.
   */
  activityFilm: '/activities/{id}/film',

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
  insightsLoad: '/insights/load',
  insightsRolling: '/insights/rolling',
  insightsStreaks: '/insights/streaks',
  insightsChallenges: '/insights/challenges',
  insightsYear: '/insights/year',

  /**
   * The four insight routes of the second build.
   *
   * The ladder of the training paces, the physiological-cost curve, the
   * threshold's four candidates and the stimulus ledger — four questions,
   * four routes, and no second field on `/insights/progress`.
   */
  insightsPaces: '/insights/paces',
  insightsPacingCurve: '/insights/pacing-curve',
  insightsThreshold: '/insights/threshold',
  insightsStimulus: '/insights/stimulus',

  /**
   * The athlete's own facts, and the plan's programmes.
   *
   * The races and the threshold are the athlete's own: they are typed
   * into the app and stored on this machine, and they are never synced.
   */
  athleteRaces: '/athlete/races',
  athleteThreshold: '/athlete/threshold',
  /**
   * The athlete's own heart rates.
   *
   * They are the athlete's own: typed into the app and stored on this
   * machine, and never synced.
   */
  athleteHeartRates: '/athlete/heart-rates',
  planBases: '/plan/bases',
  planAdopt: '/plan/adopt',

  insightsHeatmap: '/insights/heatmap',
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

  /*
    The settings, the note and the two storage actions.

    These four are the routes the Settings screen needs and the frozen
    table does not yet carry. The app's declarations are the contract
    (`src/lib/types.ts`), and the api is held to them by
    `crates/stride-api/tests/frontend_shapes.rs` — so they are declared
    here, and every one of them has to be written into
    `docs/contract/api-paths.md` in the same pull request.
  */
  settings: '/settings',
  settingsExport: '/settings/export',
  settingsDelete: '/settings/delete',
  activityNote: '/activities/{id}/note',

  /*
    The run update: the stored update of a run, the payload a model may
    read, and the update's own settings.
  */
  activityUpdate: '/activities/{id}/update',
  activityUpdatePayload: '/activities/{id}/update/payload',
  updateSettings: '/update/settings',

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
  bytes = false,
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
      body,
    )
    if (answer === undefined) {
      throw new ApiError('not_found', `no fixture for ${path}`, 404)
    }
    // A fixture may answer with a failure, the way the router answers
    // with one: the plan is not there, the data is stale, the history is
    // too short. The screen reads the kind, not a generic error.
    if (answer instanceof FixtureFailure) {
      throw new ApiError(answer.kind, answer.detail, answer.status)
    }
    if (bytes && Array.isArray(answer)) {
      // A file is bytes, not JSON: the fixture's answer is the byte
      // list, and the app gets the bytes.
      return new Uint8Array(answer as number[]).buffer as T
    }
    return answer as T
  }

  const apiRequest: ApiRequest = { method, path: url, body, bytes }
  return (await transport(apiRequest)) as T
}

const get = <T>(path: string, params?: Record<string, string | number | undefined>) =>
  request<T>('GET', path, undefined, params)

/** A GET whose answer is bytes rather than JSON. */
const getBytes = (path: string) => request<ArrayBuffer>('GET', path, undefined, undefined, true)

const post = <T>(path: string, body?: unknown) => request<T>('POST', path, body)

/** The typed client. */
export const api = {
  /** The feed, windowed by date. */
  activities: (params?: { cursor?: string; limit?: number; since?: string; until?: string }) =>
    get<ActivityListResponse>(
      ROUTES.activities,
      params as Record<string, string | number | undefined>,
    ),
  /** One run. */
  activity: (id: number | string) => get<ActivityDetail>(route('activity', { id })),

  /**
   * The run's film: the run's own series, playable.
   *
   * The route names the run, the way `/activities/{id}` does: the film is
   * one run's timeline, and the zone model travels with it, named and
   * dated.
   */
  runFilm: (id: number | string) => get<RunFilm>(route('activityFilm', { id })),

  /**
   * The run's original FIT file, byte for byte.
   *
   * The source of truth, downloaded to the athlete's own machine and
   * uploaded nowhere. The phone's bridge carries JSON, so it cannot
   * fetch a file yet; the desktop can.
   */
  activityFit: (id: number | string) => getBytes(route('activityFit', { id })),

  /** The vitals and their directions — the snapshot Today and Body read. */
  vitals: (params?: { date?: string; days?: number }) =>
    get<VitalsSnapshot>(ROUTES.physiologyVitals, params),
  physiologyDaily: () => get<unknown>(ROUTES.physiologyDaily),
  physiologySleep: (params?: { from?: string; to?: string }) =>
    get<SleepDays>(ROUTES.physiologySleep, params),
  physiologyHrv: () => get<unknown>(ROUTES.physiologyHrv),
  physiologyLatest: () => get<unknown>(ROUTES.physiologyLatest),
  physiologySources: () => get<unknown>(ROUTES.physiologySources),

  fitnessTrends: () => get<unknown>(ROUTES.fitnessTrends),
  /**
   * The fingerprint of one run: the run's own mechanics, as observations.
   *
   * The route takes the run's id: the observations belong to a run, and
   * the run detail shows them.
   */
  fitnessFingerprint: (id: number | string) =>
    get<Fingerprint>(ROUTES.fitnessFingerprint, { id: String(id) }),
  /**
   * The heart-rate zones of one run.
   *
   * The route takes the run's id: the bands belong to a run, and the
   * chart draws them.
   */
  fitnessZones: (id: number | string) =>
    get<FitnessZones>(ROUTES.fitnessZones, { id: String(id) }),

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
  planImport: (payload: unknown) => post<PlanImportResult>(ROUTES.planImport, payload),

  /** Today's readout. */
  insightsToday: () => get<TodayInsight>(ROUTES.insightsToday),
  insightsProgress: (weeks = 12) => get<ProgressInsight>(ROUTES.insightsProgress, { weeks }),
  /**
   * The training load over the same window.
   *
   * A route of its own: the load needs its own window and its own "no
   * value" reasons, and it carries the assumption its numbers rest on.
   */
  insightsLoad: (weeks = 12) => get<LoadInsight>(ROUTES.insightsLoad, { weeks }),
  /**
   * The rolling view: the window against the window before it, the split
   * of the training across the athlete's own zones, the weekday and
   * daytime splits, and the monthly and yearly tables.
   *
   * It follows the same range as the Progress screen, so the card is
   * never read apart from the control that changes it.
   */
  insightsRolling: (weeks = 12) => get<RollingInsight>(ROUTES.insightsRolling, { weeks }),
  /**
   * The rolling view over a window of **days**.
   *
   * The athlete profile's ladder: thirty, sixty and ninety days. The
   * weeks are the Progress screen's range; the days are the profile's
   * own windows, and a thirty-day window is not four weeks.
   */
  insightsRollingDays: (days: number) => get<RollingInsight>(ROUTES.insightsRolling, { days }),
  /**
   * The streaks: the current run of days and weeks, and the longest one
   * of each.
   */
  insightsStreaks: () => get<StreakInsight>(ROUTES.insightsStreaks),
  /**
   * The challenges: the athlete's own, measured week by week.
   *
   * The weekly goal is one of them, so the grid has something in it from
   * the first day, and the athlete's own challenges sit beside it.
   */
  insightsChallenges: (weeks = 12) =>
    get<ChallengeInsight>(ROUTES.insightsChallenges, { weeks }),
  /**
   * The year: the twelve calendar months, the calendar of the days
   * coloured by the zones they were run in, and what changed against the
   * athlete's own past.
   *
   * The calendar says **when** and **how hard**. It carries no position
   * and no place name: where the athlete ran is the heatmap's surface.
   */
  insightsYear: () => get<YearInsight>(ROUTES.insightsYear),

  /**
   * The heatmap: the grid over the stored positions.
   *
   * **Computed on the device.** The answer is a count of runs per cell,
   * made from the runs the store already holds; nothing is fetched for it
   * and nothing is sent with it.
   */
  insightsHeatmap: () => get<HeatmapInsight>(ROUTES.insightsHeatmap),
  insightsChips: (id: number | string) =>
    get<unknown>(ROUTES.insightsChips, { id: String(id) }),

  /**
   * The ladder: the training paces, each with the rule it comes from.
   *
   * The rungs, the anchors they hang on, the race the paces came from and
   * the equivalent race times. **The ladder's own numbers do not depend on
   * the window** — the paces come from the athlete's own race — but the
   * answer names the window it is shown over.
   */
  insightsPaces: (weeks = 12) => get<PacesInsight>(ROUTES.insightsPaces, { weeks }),
  /**
   * The physiological-cost curve: the share of the maximum a duration
   * holds, against the time it is held for.
   *
   * A second reading of the same athlete, and not a second ladder.
   */
  insightsPacingCurve: (weeks = 12) => get<CurveInsight>(ROUTES.insightsPacingCurve, { weeks }),
  /**
   * The threshold: the four candidates, the choice in force and the dated
   * history of the choices.
   */
  insightsThreshold: (weeks = 12) => get<ThresholdInsight>(ROUTES.insightsThreshold, { weeks }),
  /**
   * The ledger: the athlete's own bands, the shape rows, and the two
   * bases the time was measured on.
   */
  insightsStimulus: (weeks = 12) => get<StimulusInsight>(ROUTES.insightsStimulus, { weeks }),

  /**
   * The programmes the app can adopt.
   *
   * The four of the catalogue, each with its tier and its entry
   * requirement — and the athlete's own mileage beside it.
   */
  planBases: () => get<BasesInsight>(ROUTES.planBases),
  /**
   * The plan in force, and the programme it was adopted from.
   */
  planAdopted: () => get<AdoptionInsight>(ROUTES.planAdopt),
  /**
   * Adopt a programme.
   *
   * The paces are worked out once, here, from the threshold in force on
   * the day of the adoption.
   */
  adoptPlan: (adoption: {
    programme_id: string
    tier?: string
    goal_distance_m?: number
    race_date: string
    start_date?: string
    threshold?: {
      source: string
      base?: string
      origin_ref?: string
      heart_rate_bpm?: number
      pace_fast_s_per_km?: number
      pace_slow_s_per_km?: number
      label?: string
    }
  }) => post<AdoptionWritten>(ROUTES.planAdopt, adoption),
  /**
   * The athlete's own races.
   */
  athleteRaces: () => get<RacesInsight>(ROUTES.athleteRaces),
  /**
   * The athlete's own heart rates.
   */
  athleteHeartRates: () => get<HeartRatesInsight>(ROUTES.athleteHeartRates),
  /**
   * Declare the athlete's heart rates.
   *
   * The body carries the maximum, the resting rate, or both — each with the
   * source it came from and, optionally, the day it was measured. The kind
   * that is not given carries its value forward.
   */
  declareHeartRates: (declaration: {
    max_heart_rate_bpm?: number
    max_source?: string
    max_measured_on?: string | null
    resting_heart_rate_bpm?: number
    resting_source?: string
    resting_measured_on?: string | null
  }) => post<DeclarationWritten>(ROUTES.athleteHeartRates, declaration),
  /**
   * Make a threshold choice.
   *
   * The body names the candidate the athlete picked, or the number they
   * brought, and the base the choice was made for.
   */
  chooseThreshold: (choice: {
    source: string
    base: string
    origin_ref?: string
    heart_rate_bpm?: number
    pace_fast_s_per_km?: number
    pace_slow_s_per_km?: number
    label?: string
  }) => post<ChoiceWritten>(ROUTES.athleteThreshold, choice),

  /** The metric diff. */
  compare: (a: number | string, b: number | string) =>
    get<CompareResponse>(ROUTES.compare, { a: String(a), b: String(b) }),
  compareCandidates: (id: number | string) =>
    get<CompareCandidate[]>(ROUTES.compareCandidates, { id: String(id) }),

  health: () => get<Health>(ROUTES.health),
  healthDaily: () => get<HealthDaily>(ROUTES.healthDaily),

  syncStatus: () => get<SyncStatus>(ROUTES.syncStatus),
  syncRun: async (backfill = false): Promise<unknown> => {
    const result = await post<SyncRunResult | { queued: true }>(
      ROUTES.syncRun,
      backfill ? { backfill: true } : undefined,
    )
    if (!('queued' in result) && result.failure) {
      const failure = result.failure
      const kind = API_ERROR_KINDS.find((kind) => kind === failure.code) ?? 'unknown'
      throw new ApiError(kind, failure.detail, failure.status)
    }
    return result
  },
  syncRuns: () => get<unknown>(ROUTES.syncRuns),
  syncHistory: () => get<unknown>(ROUTES.syncHistory),
  syncSettings: () => get<unknown>(ROUTES.syncSettings),

  authStatus: () => get<ConnectionStatus>(ROUTES.authStatus),

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
    return post<KeyConnectResult>(ROUTES.authGarminPersonalKey, { key })
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
    return post<KeyTestResult>(ROUTES.authGarminPersonalKeyTest, { key })
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

  importFit: (payload: unknown) => post<FitImportResult>(ROUTES.authGarminFitImport, payload),
  garminLogout: () => post<KeyConnectResult>(ROUTES.authGarminLogout),
  calendar: (params?: { year?: number; month?: number }) =>
    get<ActivityCalendar>(ROUTES.calendar, params),

  /**
   * The settings: the weekly goal, the pause and the storage.
   *
   * The goal and the pause are written through the same route, and the
   * answer is the whole state — so the screen never has to guess what
   * the api did with the number it sent.
   */
  settings: () => get<SettingsState>(ROUTES.settings),
  saveSettings: (change: {
    /** The weekly goal in kilometres. Null clears the goal. */
    goal_km?: number | null
    /** The pause. Null clears the pause. */
    pause?: { start_date: string; end_date: string; reason: string | null } | null
    /**
     * The challenges, in the order the athlete wants them. The list
     * replaces the set: what is sent is what is there.
     */
    challenges?: { kind: ChallengeKind; km: number; label?: string | null }[]
  }) => post<SettingsState>(ROUTES.settings, change),

  /** A backup, written by the host. */
  exportBackup: () => post<SettingsExportResult>(ROUTES.settingsExport),

  /**
   * Everything, deleted.
   *
   * The word has to travel with the request: the api refuses the call
   * unless it carries the confirmation, so a delete cannot happen by
   * accident.
   */
  deleteEverything: (confirmation: string) =>
    post<SettingsDeleteResult>(ROUTES.settingsDelete, { confirm: confirmation }),

  /** A note on a run — the only free text the product stores. */
  saveNote: (id: number | string, text: string | null) =>
    post<NoteResult>(route('activityNote', { id }), { text }),

  /**
   * The stored update of a run: the model's own reading of it, with the
   * checks it passed and its verdict.
   *
   * It is `null` when the run has none: a draft that did not pass the
   * checks, and one the reviewer discarded, keep nothing.
   */
  activityUpdate: (id: number | string) =>
    get<RunUpdateResponse>(route('activityUpdate', { id })),
  /**
   * The payload a model may read for a run: the facts, and nothing else.
   *
   * This is what would be sent, and the app shows it before the first
   * send.
   */
  activityUpdatePayload: (id: number | string) =>
    get<UpdatePayloadResponse>(route('activityUpdatePayload', { id })),
  /**
   * The update's own settings: which model writes, which model reviews,
   * and whether anything may leave the machine at all.
   */
  updateSettings: () => get<UpdateSettings>(ROUTES.updateSettings),
  /**
   * Write the update's own settings.
   *
   * The key is not among them: the settings name the environment
   * variable it is read from.
   */
  saveUpdateSettings: (change: {
    enabled?: boolean
    allow_remote?: boolean
    writer_model?: string
    writer_family?: string
    writer_base_url?: string
    reviewer_model?: string
    reviewer_family?: string
    reviewer_base_url?: string
    key_env?: string
  }) => post<UpdateSettings>(ROUTES.updateSettings, change),
}
