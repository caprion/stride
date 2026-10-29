/**
 * The one place the app knows whether Garmin is connected.
 *
 * Every screen that mentions the connection reads this hook, so the app has
 * a single answer rather than three that can disagree.
 *
 * **What the answer is made of.**
 *
 * * `GET /api/v1/auth/status` — `garmin_connected` and `garmin_method`;
 * * `GET /api/v1/sync/status` — `running`, `last_sync_at` and `connected`;
 * * `GET /api/v1/stats/summary` — how many runs the device holds.
 *
 * **On the phone the key lives in the Keystore** and the core holds it in
 * memory only for the length of a pull, so `garmin_connected` is true for
 * the seconds a pull runs. The runs on the phone are the same evidence the
 * native screen uses when it says "Connected · 160 runs", so the web app
 * says the same thing from the same number.
 *
 * The native screen is the authority on the key itself: it is where the key
 * is typed, and where the athlete sees whether it is there. This hook is
 * the app's answer about the data.
 */

import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from './api'
import { isPhone } from './surface'

/**
 * Whether the athlete forgot the key on this device.
 *
 * Forgetting the key leaves the runs on the phone, so the runs alone would
 * say "Connected" while the native screen says it is not. The web app
 * remembers the forgetting — it is the app's own action — until the API
 * says Garmin is connected again.
 */
const FORGOTTEN_KEY = 'stride.garmin.forgotten'

function keyForgotten(): boolean {
  try {
    return window.localStorage.getItem(FORGOTTEN_KEY) === '1'
  } catch {
    return false
  }
}

function forgetIt(): void {
  try {
    window.localStorage.setItem(FORGOTTEN_KEY, '1')
  } catch {
    /* the phone may not have storage; the answer is then the API's alone */
  }
}

function rememberIt(): void {
  try {
    window.localStorage.removeItem(FORGOTTEN_KEY)
  } catch {
    /* nothing to forget */
  }
}

/** What the app knows about the Garmin connection. */
export interface GarminState {
  /** Whether Garmin is connected. */
  connected: boolean
  /** Whether a pull is running right now. */
  syncing: boolean
  /** When the last successful pull finished. */
  lastSyncAt: string | null
  /** How the phone or the desktop is connected. */
  method: string | null
  /** How many runs the device holds. */
  runs: number
  /** Whether the answer is still on its way. */
  loading: boolean
  /** Why the answer could not be read, if it could not. */
  error: string | null
}

export function useGarminState(): GarminState {
  const auth = useQuery({
    queryKey: ['auth-status'],
    queryFn: api.authStatus,
    refetchInterval: 30_000,
  })

  const sync = useQuery({
    queryKey: ['sync-status'],
    queryFn: api.syncStatus,
    refetchInterval: 15_000,
  })

  const summary = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const lastSyncAt = sync.data?.last_sync_at ?? auth.data?.last_sync_at ?? null
  const runs = summary.data?.totals?.activity_count ?? 0

  // What the API says.
  const apiConnected = Boolean(auth.data?.garmin_connected) || Boolean(sync.data?.connected)

  // The API says Garmin is connected again, so the web app stops saying it
  // was forgotten.
  useEffect(() => {
    if (apiConnected) rememberIt()
  }, [apiConnected])

  // On the phone the runs are the evidence, and the native screen reads the
  // same number. On the desktop the API's own answer is the answer: it
  // keeps the connection in its state file.
  const connected = isPhone() ? apiConnected || (!keyForgotten() && runs > 0) : apiConnected

  return {
    connected,
    syncing: Boolean(sync.data?.running),
    lastSyncAt,
    method: sync.data?.method ?? auth.data?.garmin_method ?? null,
    runs,
    loading: auth.isPending || sync.isPending,
    error:
      auth.error instanceof Error
        ? auth.error.message
        : sync.error instanceof Error
          ? sync.error.message
          : null,
  }
}

/**
 * The two actions the phone's settings screen offers: pull now, and forget
 * the key.
 *
 * Both live here so that every screen acts on the connection the same way,
 * and so that the cache is refreshed the same way afterwards.
 */
export function useGarminActions() {
  const client = useQueryClient()

  const refresh = async () => {
    await Promise.all([
      client.invalidateQueries({ queryKey: ['auth-status'] }),
      client.invalidateQueries({ queryKey: ['sync-status'] }),
      client.invalidateQueries({ queryKey: ['sync-runs'] }),
    ])
  }

  const syncNow = useMutation({
    mutationFn: () => api.runSync(),
    onSuccess: refresh,
  })

  const forgetKey = useMutation({
    mutationFn: () => api.logout(),
    onSuccess: async () => {
      forgetIt()
      await refresh()
    },
  })

  return { syncNow, forgetKey, refresh }
}
