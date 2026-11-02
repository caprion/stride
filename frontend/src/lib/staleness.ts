/**
 * How old the data is.
 *
 * The plan says it in one line: the header says "last synced 2 days ago",
 * and the screen says which day the numbers are from. **Staleness is
 * shown, never hidden** — old numbers are still numbers, and the athlete
 * is the one who decides what to do with them.
 *
 * The window is the plan's: older than a day and a half, the data is
 * stale.
 */

import { useQuery } from '@tanstack/react-query'
import { api } from './api'
import { t } from '../i18n'

/** How long ago the last sync was, in words — only when it matters. */
export function stalenessText(lastSyncAt: string | null | undefined): string | null {
  if (!lastSyncAt) return null
  const hours = (Date.now() - new Date(lastSyncAt).getTime()) / 3_600_000
  if (!Number.isFinite(hours) || hours < 36) return null
  const days = Math.round(hours / 24)
  if (days >= 1) return t('today.state.stale.days', { n: days })
  return t('today.state.stale.hours', { n: Math.round(hours) })
}

/**
 * The line, for any screen that wants it.
 *
 * One query, shared by every screen that asks: the sync status is the
 * same answer wherever it is read from.
 */
export function useStaleness(): string | null {
  const sync = useQuery({ queryKey: ['sync-status'], queryFn: api.syncStatus })
  return stalenessText(sync.data?.last_sync_at)
}
