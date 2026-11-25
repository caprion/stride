/**
 * The Garmin controls.
 *
 * Two controls, and the same two on both hosts:
 *
 * * **Refresh Garmin** — the recent pull, the one that keeps the last
 *   fortnight current.
 * * **Check older history** — the dated, checkpointed walk back through
 *   the months, and the door the athlete uses when they want more of it
 *   than the phone has already checked.
 *
 * **The two hosts run the older history in their own way.** The desktop's
 * router does the walk in the request itself; the phone's shell does it
 * in its own background job, because the Garmin key never crosses the
 * bridge. The answer is the same either way, and the state the athlete
 * reads — the history that has been checked — is the same too.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef } from 'react'
import { api } from '../lib/api'
import { currentHost } from '../lib/transport'
import { formatDateShort } from '../lib/format'
import { t } from '../i18n'

export function SyncControls({ details = false }: { details?: boolean }) {
  const client = useQueryClient()
  const connection = useQuery({
    queryKey: ['auth-status'],
    queryFn: api.authStatus,
    staleTime: 15_000,
  })
  const status = useQuery({
    queryKey: ['sync-status'],
    queryFn: api.syncStatus,
    staleTime: 15_000,
    refetchInterval: currentHost() === 'native' ? 15_000 : false,
  })
  const previousSync = useRef<string | null | undefined>(undefined)
  useEffect(() => {
    const stamp = status.data?.last_sync_at
    if (stamp === undefined) return
    const previous = previousSync.current
    previousSync.current = stamp
    if (previous !== undefined && previous !== stamp) {
      void client.invalidateQueries({
        predicate: (query) => query.queryKey[0] !== 'sync-status',
      })
    }
  }, [status.data?.last_sync_at, client])
  const refresh = useMutation({
    mutationKey: ['garmin-refresh'],
    mutationFn: (older: boolean) => api.syncRun(older),
    onSuccess: () => {
      void client.invalidateQueries()
    },
  })
  if (connection.isError)
    return (
      <div role="alert" className="text-[13px] text-ink-muted">
        <p>{t('settings.connection.unknown')}</p>
        <button
          className="tap underline"
          type="button"
          onClick={() => {
            void connection.refetch()
          }}
        >
          {t('today.state.retry')}
        </button>
      </div>
    )
  if (!connection.data?.connected) return null
  const history = status.data?.history
  return (
    <div>
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          className="tap rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700 disabled:opacity-50"
          disabled={refresh.isPending}
          onClick={() => refresh.mutate(false)}
        >
          {refresh.isPending ? t('sync.refreshing') : t('sync.refresh')}
        </button>
        {details && status.data?.more_pending ? (
          <button
            type="button"
            className="tap rounded-md border border-edge px-3 text-[13px] disabled:opacity-50"
            disabled={refresh.isPending}
            data-testid="sync-older"
            onClick={() => refresh.mutate(true)}
          >
            {t('sync.older')}
          </button>
        ) : null}
      </div>
      {refresh.isError ? (
        <p role="alert" className="mt-2 text-[13px] text-ink-muted">
          {t('sync.failed')}
        </p>
      ) : null}
      {refresh.isSuccess && currentHost() === 'native' ? (
        <p role="status" className="mt-2 text-[13px] text-ink-muted">
          {refresh.variables ? t('sync.olderRequested') : t('sync.requested')}
        </p>
      ) : null}
      {details && status.isError ? (
        <p role="alert" className="mt-2 text-[13px] text-ink-muted">
          {t('common.error')}
        </p>
      ) : null}
      {details && history ? (
        <details className="mt-3 text-[12px] text-ink-muted">
          <summary className="tap-inline cursor-pointer">{t('sync.coverage')}</summary>
          <dl>
            {(['activities', 'wellness'] as const).map((key) => {
              const coverage = history[key]
              return (
                <div key={key} className="mt-1 flex flex-wrap justify-between gap-2">
                  <dt>{t(key === 'activities' ? 'sync.activities' : 'sync.wellness')}</dt>
                  <dd>
                    {coverage?.checked_from && coverage.checked_to
                      ? t('sync.coverage.range', {
                          from: formatDateShort(coverage.checked_from),
                          to: formatDateShort(coverage.checked_to),
                        })
                      : t('sync.coverage.unknown')}
                  </dd>
                </div>
              )
            })}
          </dl>
        </details>
      ) : null}
    </div>
  )
}
