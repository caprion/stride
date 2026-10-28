/**
 * The gate in front of the app.
 *
 * Stride's API is protected by a capability cookie that the server mints
 * when the app is loaded from a loopback address. A browser on the tailnet
 * cannot mint one — deliberately — so it gets a 401 and is asked to pair.
 *
 * Everything else (a transient network error, a server that is still
 * starting) must not be mistaken for "not paired", so only a 401 opens the
 * pairing screen.
 */

import type { ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Activity } from 'lucide-react'
import { ApiError, api } from '../lib/api'
import { PairDevice } from './PairDevice'

export function AuthGate({ children }: { children: ReactNode }) {
  const { error, isPending } = useQuery({
    queryKey: ['auth-status'],
    queryFn: api.authStatus,
    retry: false,
    staleTime: 30_000,
  })

  if (isPending) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-surface-soft">
        <div className="flex items-center gap-2 text-ink-muted">
          <Activity size={18} className="animate-pulse text-brand-500" />
          <span className="text-[13px]">Opening Stride…</span>
        </div>
      </div>
    )
  }

  if (error instanceof ApiError && error.status === 401) {
    return <PairDevice />
  }

  if (error instanceof ApiError) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-surface-soft px-4">
        <div className="max-w-md rounded-xl border border-edge bg-surface p-6 text-center shadow-card">
          <h1 className="text-[17px] font-bold text-ink">Stride is not answering</h1>
          <p className="mt-2 text-[13px] leading-relaxed text-ink-muted">
            The server returned {error.status}. It may still be starting — try again in a
            moment.
          </p>
          <button
            type="button"
            onClick={() => window.location.reload()}
            className="mt-4 rounded-md bg-brand-600 px-4 py-2 text-[13px] font-semibold text-white"
          >
            Try again
          </button>
        </div>
      </div>
    )
  }

  return <>{children}</>
}
