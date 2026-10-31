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
 * wants it on purpose. **The gate offers it, but never demands it:** the
 * operator's own way to sign a browser in from another machine is to trust
 * the network it is on (`STRIDE_TRUSTED_NETWORKS`), and the code is the
 * deliberate alternative for a network they have not trusted.
 *
 * Everything else (a transient network error, a server that is still
 * starting) must not be mistaken for "not signed in", so only a 401 opens
 * this screen.
 */

import { useState, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Activity, KeyRound, ShieldCheck } from 'lucide-react'
import { ApiError, api } from '../lib/api'
import { localDataLine } from '../lib/surface'
import { PairDevice } from './PairDevice'

export function AuthGate({ children }: { children: ReactNode }) {
  const [wantsCode, setWantsCode] = useState(false)
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
    // Not signed in. The app does not demand a credential: it says what
    // is true, and it offers the two ways in.
    if (wantsCode) {
      return <PairDevice />
    }
    return (
      <div className="flex min-h-screen items-center justify-center bg-surface-soft px-4">
        <div className="max-w-md rounded-xl border border-edge bg-surface p-6 text-center shadow-card">
          <ShieldCheck size={22} strokeWidth={1.8} className="mx-auto text-brand-600" />
          <h1 className="mt-3 text-[17px] font-bold text-ink">This browser is not signed in</h1>
          <p className="mt-2 text-[13px] leading-relaxed text-ink-muted">
            Stride runs on another machine — the operator's own — and this browser reached it
            from somewhere else, so it has no session.
          </p>
          <p className="mt-3 text-[13px] leading-relaxed text-ink-soft">
            Two ways in. The operator can trust the network this browser is on{' '}
            <span className="font-mono text-[12px]">STRIDE_TRUSTED_NETWORKS</span>, and then
            this browser is signed in like any other. Or they can mint a code on the machine and
            type it in here.
          </p>
          <p className="mt-3 text-[12px] leading-relaxed text-ink-muted">
            The app on your phone is the phone app: paste your Garmin key into one native screen
            and everything stays on the phone. {localDataLine()}
          </p>
          <button
            className="mt-4 inline-flex items-center gap-1.5 text-[13px] font-semibold text-brand-600 hover:text-brand-700"
            onClick={() => setWantsCode(true)}
          >
            <KeyRound size={14} strokeWidth={2} />
            Sign in with a code
          </button>
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
