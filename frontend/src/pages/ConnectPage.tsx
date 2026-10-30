/**
 * Connect Garmin — the desktop's front door.
 *
 * **The phone has a native screen for this. The desktop has this one.**
 *
 * The phone's key is typed into a native field and goes straight into the
 * Keystore. The desktop's browser has no native field, so it gets a page
 * with the same field, the same instructions and the same promise — and
 * nothing else in front of it.
 *
 * The key is Garmin's own **Personal API key**: one string, no developer
 * account, no consumer secret, and it covers activities *and* wellness.
 * The reference application also accepts a developer key and an account
 * login; those live in Settings, and they are the reference's ways in, not
 * this one's.
 */
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { ExternalLink, Loader2, Plug } from 'lucide-react'
import { api } from '../lib/api'
import { localDataLine } from '../lib/surface'
import { Button, Card, CardHeader, inputClass } from '../components/ui'

type Notice = { tone: 'positive' | 'negative' | 'info'; text: string }

function messageOf(
  error: unknown,
  fallback = 'Something went wrong. Please try again.',
): string {
  if (error instanceof Error && error.message.trim()) return error.message
  return fallback
}

export function ConnectPage() {
  const navigate = useNavigate()
  const [apiKey, setApiKey] = useState('')
  const [notice, setNotice] = useState<Notice | null>(null)

  const connect = useMutation({
    mutationFn: () => api.connectPersonalKey(apiKey.trim()),
    onSuccess: async (data) => {
      setApiKey('')
      setNotice({
        tone: 'positive',
        text: `Garmin is connected.${
          data.athlete?.name ? ` Welcome, ${data.athlete.name}.` : ''
        } Your first sync starts now.`,
      })
      // The app is the point; the screen is the door.
      setTimeout(() => navigate('/'), 1200)
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const test = useMutation({
    mutationFn: () => api.testPersonalKey(apiKey.trim()),
    onSuccess: (data) =>
      setNotice({
        tone: 'positive',
        text: `That key works${
          data.athlete?.name ? ` — it belongs to ${data.athlete.name}` : ''
        }.`,
      }),
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const busy = connect.isPending || test.isPending

  return (
    <div className="flex min-h-screen items-center justify-center bg-surface-soft px-4 py-10">
      <Card className="w-full max-w-lg p-6 shadow-card">
        <CardHeader
          title="Connect Garmin"
          subtitle="One key. It covers your activities and your sleep, resting heart rate and body battery."
        />

        <div className="mt-4 space-y-4">
          <p className="text-[13px] leading-relaxed text-ink-soft">
            Garmin shows the key once — copy it and paste it below. Nothing is sent anywhere but
            Garmin, and {localDataLine().replace(/^Your /, 'your ')}
          </p>

          <div>
            <label
              className="mb-1.5 block text-[12px] font-semibold text-ink"
              htmlFor="api-key"
            >
              Personal API key
            </label>
            <input
              id="api-key"
              type="password"
              autoComplete="off"
              spellCheck={false}
              placeholder="GCDC_..."
              className={inputClass}
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
            />
            <a
              className="mt-2 inline-flex items-center gap-1.5 text-[13px] font-semibold text-brand-600 hover:text-brand-700"
              href="https://connect.garmin.com/app/settings/personalApiToken"
              target="_blank"
              rel="noreferrer noopener"
            >
              <ExternalLink size={14} strokeWidth={2} />
              Get a key from Garmin — pick the one-year expiry
            </a>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="md"
              disabled={busy || !apiKey.trim()}
              onClick={() => connect.mutate()}
            >
              {connect.isPending ? (
                <Loader2 size={14} strokeWidth={2} className="animate-spin" />
              ) : (
                <Plug size={14} strokeWidth={2} />
              )}
              Connect with key
            </Button>
            <Button
              variant="secondary"
              size="md"
              disabled={busy || !apiKey.trim()}
              onClick={() => test.mutate()}
            >
              {test.isPending ? (
                <Loader2 size={14} strokeWidth={2} className="animate-spin" />
              ) : null}
              Test key
            </Button>
            <Button variant="secondary" size="md" onClick={() => navigate('/')}>
              Not now
            </Button>
          </div>

          {notice ? (
            <p
              className={`text-[13px] ${
                notice.tone === 'negative' ? 'text-negative' : 'text-ink-soft'
              }`}
            >
              {notice.text}
            </p>
          ) : null}

          <p className="text-[12px] leading-relaxed text-ink-muted">
            No key? You can still{' '}
            <button
              className="font-semibold text-brand-600 hover:text-brand-700"
              onClick={() => navigate('/settings')}
            >
              import .FIT files
            </button>{' '}
            from any watch — Strava, COROS, Wahoo, Polar or Garmin.
          </p>
        </div>
      </Card>
    </div>
  )
}

export default ConnectPage
