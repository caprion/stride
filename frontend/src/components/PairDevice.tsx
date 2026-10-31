/**
 * The screen that signs a browser in with a code.
 *
 * **It is never what you land on.** The gate explains the situation first
 * and this is behind a deliberate click, because a code is the operator's
 * decision and not the athlete's: the operator either trusts the network
 * this browser is on, or mints a code for it on the machine.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { KeyRound, ShieldCheck } from 'lucide-react'
import { ApiError, api } from '../lib/api'
import { localDataLine } from '../lib/surface'
import { Button, inputClass } from '../components/ui'

export function PairDevice({ initialError = null }: { initialError?: string | null }) {
  const queryClient = useQueryClient()
  const [code, setCode] = useState('')
  const [error, setError] = useState<string | null>(initialError)

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
            This browser is not on the machine running Stride. Pair it to read your training
            data.
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

            {error && <p className="mt-2 text-[12px] font-medium text-negative">{error}</p>}

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
              It prints a QR code and the same code in text. It works once and expires in 10
              minutes.
            </p>
            <p className="mt-2 text-[12px] leading-relaxed text-ink-muted">
              The phone app never needs this: it has its own database and its own Garmin key,
              and it is signed in from the first pixel. A code is for a browser on another
              machine, and only when the operator has not trusted that network.
            </p>
          </div>
        </div>

        <p className="mt-4 text-center text-[12px] text-ink-faint">{localDataLine()}</p>
      </div>
    </div>
  )
}
