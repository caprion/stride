/**
 * The gate in front of the app.
 *
 * Stride's API is protected by a capability cookie that the server mints
 * when the app is loaded from a loopback address. A browser on the tailnet
 * cannot mint one — deliberately — so it gets a 401.
 *
 * **V1 does not pair.** The phone app is the whole thing: it has its own
 * database and its own Garmin key, and it is signed in from the first
 * pixel. The desktop's own browser is signed in automatically too, because
 * it *is* the machine Stride runs on. So a browser that is somewhere else
 * is told what is true — this is the operator's machine, and the app on
 * the phone is the phone app — rather than being handed a credential
 * ceremony the product does not use.
 *
 * The route that mints a pairing code is still there, for somebody who
 * wants it on purpose. Nothing in the app calls it.
 *
 * Everything else (a transient network error, a server that is still
 * starting) must not be mistaken for "not signed in", so only a 401 opens
 * this screen.
 */

import { type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Activity, ShieldCheck } from 'lucide-react'
import { ApiError, api } from '../lib/api'
import { localDataLine } from '../lib/surface'

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
    // Not signed in, and V1 does not ask this browser to be.
    return (
      <div className="flex min-h-screen items-center justify-center bg-surface-soft px-4">
        <div className="max-w-md rounded-xl border border-edge bg-surface p-6 text-center shadow-card">
          <ShieldCheck size={22} strokeWidth={1.8} className="mx-auto text-brand-600" />
          <h1 className="mt-3 text-[17px] font-bold text-ink">
            This is the operator's own machine
          </h1>
          <p className="mt-2 text-[13px] leading-relaxed text-ink-muted">
            Stride runs here, and this browser is somewhere else, so it is not signed in.
          </p>
          <p className="mt-3 text-[13px] leading-relaxed text-ink-soft">
            The app on your phone is the phone app: paste your Garmin key into one native screen
            and everything stays on the phone. {localDataLine()}
          </p>
        </div>
      </div>
    )
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
