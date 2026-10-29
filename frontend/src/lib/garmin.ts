/**
 * The one place the app knows whether Garmin is connected.
 *
 * Every screen that mentions the connection reads this hook, so the app has
 * a single answer rather than three that can disagree.
 *
 * **What the answer is made of.**
 *
 * * `GET /api/v1/auth/status` — `garmin_connected` and `garmin_method`;
 * * `GET /api/v1/sync/status` — `running`, `last_sync_at` and `connected`.
 *
 * **On the phone the key lives in the Keystore** and the core holds it in
 * memory only for the length of a pull, so the shell tells the core the
 * *fact* of the connection when the app starts. That is what the API
 * answers with, and it is the same truth the native screen shows.
 *
 * The native screen is the authority on the key itself: it is where the key
 * is typed, where the sync follows, and where the key is forgotten. This
 * hook is the app's answer about the data.
 */

import { useQuery } from '@tanstack/react-query'
import { api } from './api'

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

  return {
    // The API's own answer, and the same answer twice.
    connected: Boolean(auth.data?.garmin_connected) || Boolean(sync.data?.connected),
    syncing: Boolean(sync.data?.running),
    lastSyncAt: sync.data?.last_sync_at ?? auth.data?.last_sync_at ?? null,
    method: sync.data?.method ?? auth.data?.garmin_method ?? null,
    loading: auth.isPending || sync.isPending,
    error:
      auth.error instanceof Error
        ? auth.error.message
        : sync.error instanceof Error
          ? sync.error.message
          : null,
  }
}
