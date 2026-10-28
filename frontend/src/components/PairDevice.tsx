/**
 * Connect this device.
 *
 * Stride is local-first: the browser holds an HttpOnly capability cookie
 * that the server mints when the app is loaded from a loopback address. A
 * browser on the tailnet is not a loopback client, so it cannot mint one —
 * which is the point. "Any device on my tailnet can read my health data" is
 * not a property anyone wants.
 *
 * But the athlete still wants their phone to work. So the operator mints a
 * single-use code on the machine (`scripts/prod.sh pair`) and the athlete
 * types it in here. The code is exchanged for the same cookie a loopback
 * browser gets; afterwards the device needs no further help.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { KeyRound, ShieldCheck } from 'lucide-react'
import { ApiError, api } from '../lib/api'
import { Button, inputClass } from '../components/ui'

export function PairDevice() {
  const queryClient = useQueryClient()
  const [code, setCode] = useState('')
  const [error, setError] = useState<string | null>(null)

  const pair = useMutation({
    mutationFn: (value: string) => api.pair(value),
    onSuccess: () => {
      setError(null)
      queryClient.invalidateQueries()
      window.location.reload()
    },
    onError: (cause) => {
      setError(
        cause instanceof ApiError
          ? cause.message
          : 'Could not reach Stride. Is the server still running?',
      )
    },
  })

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    const trimmed = code.trim()
    if (!trimmed) {
      setError('Enter the code shown by the Stride machine.')
      return
    }
    setError(null)
    pair.mutate(trimmed)
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-surface-soft px-4 py-10">
      <div className="w-full max-w-md">
        <div className="mb-6 flex items-center gap-2">
          <span className="flex h-8 w-8 items-center justify-center rounded-md bg-brand-700 text-accent-400">
            <ShieldCheck size={17} strokeWidth={2.6} />
          </span>
          <span className="text-[20px] font-extrabold uppercase tracking-tight text-brand-500">
            Stride
          </span>
        </div>

        <div className="rounded-xl border border-edge bg-surface p-6 shadow-card">
          <h1 className="text-[19px] font-bold text-ink">Connect this device</h1>
          <p className="mt-2 text-[13px] leading-relaxed text-ink-muted">
            Stride keeps your training data on your own machine. A browser that is
            not on the same device needs to be paired before it can read anything.
          </p>

          <form onSubmit={submit} className="mt-5">
            <label
              htmlFor="pair-code"
              className="mb-1.5 block text-[12px] font-semibold uppercase tracking-wide text-ink-muted"
            >
              Pairing code
            </label>
            <div className="relative">
              <KeyRound
                size={16}
                className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-ink-faint"
              />
              <input
                id="pair-code"
                value={code}
                onChange={(event) => setCode(event.target.value.toUpperCase())}
                placeholder="XXXX-XXXX"
                autoComplete="off"
                autoCapitalize="characters"
                spellCheck={false}
                className={`${inputClass} pl-9 font-mono tracking-[0.12em] uppercase`}
              />
            </div>

            {error && (
              <p className="mt-2 text-[12px] font-medium text-negative">{error}</p>
            )}

            <Button
              type="submit"
              className="mt-4 w-full justify-center"
              disabled={pair.isPending}
            >
              {pair.isPending ? 'Connecting…' : 'Connect'}
            </Button>
          </form>

          <div className="mt-6 rounded-lg border border-edge bg-surface-sunken p-3.5">
            <p className="text-[12px] font-semibold text-ink">How to get the code</p>
            <p className="mt-1 text-[12px] leading-relaxed text-ink-muted">
              On the machine running Stride, run:
            </p>
            <code className="mt-2 block rounded bg-surface px-2.5 py-1.5 font-mono text-[12px] text-ink">
              ./scripts/prod.sh pair
            </code>
            <p className="mt-2 text-[12px] leading-relaxed text-ink-muted">
              It prints a code that works once and expires in 10 minutes.
            </p>
          </div>
        </div>

        <p className="mt-4 text-center text-[12px] text-ink-faint">
          Nothing is sent to a third party. The code is exchanged for a session
          cookie and never leaves your machine.
        </p>
      </div>
    </div>
  )
}
