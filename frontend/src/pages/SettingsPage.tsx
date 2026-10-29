/**
 * Settings — connect Garmin, choose the network route, tune syncing.
 *
 * Written for a non-engineer: every section explains what it does, why it
 * matters and what the user should click next. Errors coming back from the
 * backend are shown verbatim — they are already written for humans.
 */

import { useEffect, useRef, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  CheckCircle2,
  CircleAlert,
  Download,
  ExternalLink,
  FileUp,
  KeyRound,
  Loader2,
  LogOut,
  Mail,
  Plug,
  RefreshCw,
  Router,
  Route as RouteIcon,
  ShieldCheck,
  Upload,
} from 'lucide-react'
import { Link } from 'react-router-dom'
import { api, isNativeApp } from '../lib/api'
import { useGarminState } from '../lib/garmin'
import { isPhone, localDataLine } from '../lib/surface'
import { ConnectGarmin } from '../components/GarminStatus'
import type { AuthStatus, NetworkStatus, SyncRun } from '../types'
import { formatDateTime, formatNumber, formatRelative } from '../lib/format'
import {
  Badge,
  Button,
  Card,
  CardHeader,
  SegmentedControl,
  Divider,
  Field,
  Skeleton,
  Toggle,
  cn,
  inputClass,
} from '../components/ui'

/* ------------------------------------------------------------------ utils */

type Notice = { tone: 'positive' | 'negative' | 'info'; text: string }

function messageOf(
  error: unknown,
  fallback = 'Something went wrong. Please try again.',
): string {
  if (error instanceof Error && error.message.trim()) return error.message
  return fallback
}

function athleteName(athlete: Record<string, unknown> | undefined | null): string | null {
  if (!athlete) return null
  const keys = ['name', 'full_name', 'display_name', 'username', 'email']
  for (const key of keys) {
    const value = athlete[key]
    if (typeof value === 'string' && value.trim()) return value.trim()
  }
  return null
}

function methodLabel(
  method: 'developer_key' | 'account' | 'personal' | null | undefined,
): string {
  if (method === 'personal') return 'Personal API key'
  if (method === 'developer_key') return 'Developer API key'
  if (method === 'account') return 'Garmin account'
  return 'Not connected'
}

function routeLabel(route: string | null | undefined): string {
  if (route === 'direct') return 'Direct connection'
  if (route === 'exit_node') return 'Home exit node'
  if (route === 'auto') return 'Automatic'
  return 'Unknown'
}

/* --------------------------------------------------------- small helpers */

function Alert({
  tone,
  title,
  children,
}: {
  tone: 'negative' | 'positive' | 'info' | 'warn'
  title?: string
  children: React.ReactNode
}) {
  const tones: Record<string, string> = {
    negative: 'border-red-200 bg-red-50 text-red-800',
    positive: 'border-emerald-200 bg-emerald-50 text-emerald-800',
    info: 'border-sky-200 bg-sky-50 text-sky-800',
    warn: 'border-amber-200 bg-amber-50 text-amber-800',
  }
  const Icon = tone === 'positive' ? CheckCircle2 : tone === 'info' ? ShieldCheck : CircleAlert
  return (
    <div className={cn('flex items-start gap-2 rounded-md border px-3 py-2.5', tones[tone])}>
      <Icon size={16} strokeWidth={2} className="mt-0.5 shrink-0" />
      <div className="min-w-0 text-[13px] leading-snug">
        {title ? <p className="font-semibold">{title}</p> : null}
        <div className={cn(title ? 'mt-0.5' : '')}>{children}</div>
      </div>
    </div>
  )
}

function Toast({ notice }: { notice: Notice | null }) {
  if (!notice) return null
  const tones: Record<string, string> = {
    positive: 'border-emerald-200 bg-emerald-50 text-emerald-800',
    negative: 'border-red-200 bg-red-50 text-red-800',
    info: 'border-sky-200 bg-sky-50 text-sky-800',
  }
  return (
    <div
      role="status"
      className={cn(
        'flex items-start gap-2 rounded-md border px-3 py-2 text-[13px] leading-snug',
        tones[notice.tone],
      )}
    >
      {notice.tone === 'positive' ? (
        <CheckCircle2 size={15} strokeWidth={2} className="mt-0.5 shrink-0" />
      ) : (
        <CircleAlert size={15} strokeWidth={2} className="mt-0.5 shrink-0" />
      )}
      <span>{notice.text}</span>
    </div>
  )
}

function SectionIntro({
  title,
  badge,
  badgeTone = 'neutral',
  children,
}: {
  title: string
  badge?: string
  badgeTone?: 'neutral' | 'brand' | 'positive' | 'warn' | 'negative' | 'info'
  children?: React.ReactNode
}) {
  return (
    <div className="mb-3">
      <div className="flex flex-wrap items-center gap-2">
        <p className="text-[14px] font-semibold text-ink">{title}</p>
        {badge ? <Badge tone={badgeTone}>{badge}</Badge> : null}
      </div>
      {children ? (
        <div className="mt-1 text-[13px] leading-snug text-ink-muted">{children}</div>
      ) : null}
    </div>
  )
}

function HowItWorks({ children }: { children: React.ReactNode }) {
  return (
    <p className="mt-2 flex items-start gap-1.5 text-[12px] leading-snug text-ink-muted">
      <ShieldCheck size={13} strokeWidth={2} className="mt-0.5 shrink-0 text-ink-faint" />
      <span>
        <span className="font-semibold text-ink-soft">How it works: </span>
        {children}
      </span>
    </p>
  )
}

/* ------------------------------------------------------------- the page */

function PairedDevicesCard() {
  const queryClient = useQueryClient()
  const { data, isPending } = useQuery({
    queryKey: ['paired-devices'],
    queryFn: api.devices,
  })

  const revoke = useMutation({
    mutationFn: (id: string) => api.revokeDevice(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['paired-devices'] }),
  })

  const devices = data?.devices ?? []

  return (
    <Card>
      <CardHeader
        title="Connected devices"
        subtitle="Phones and browsers that have been paired to this instance."
      />
      <div className="px-5 pb-5">
        {isPending ? (
          <p className="text-[13px] text-ink-muted">Loading…</p>
        ) : devices.length === 0 ? (
          <p className="text-[13px] leading-relaxed text-ink-muted">
            Nothing is paired. A browser on this machine connects by itself; any other device
            needs a pairing code from{' '}
            <span className="font-mono text-[12px]">scripts/prod.sh pair</span>.
          </p>
        ) : (
          <ul className="divide-y divide-edge">
            {devices.map((device) => (
              <li key={device.id} className="flex items-center justify-between gap-3 py-2.5">
                <div className="min-w-0">
                  <p className="text-[13px] font-semibold text-ink">
                    {device.name ?? 'Device'}
                  </p>
                  <p className="text-[12px] text-ink-muted">
                    Paired{' '}
                    {device.created_at ? new Date(device.created_at).toLocaleDateString() : '—'}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => revoke.mutate(device.id)}
                  disabled={revoke.isPending}
                >
                  Revoke
                </Button>
              </li>
            ))}
          </ul>
        )}
        <p className="mt-3 text-[12px] leading-relaxed text-ink-muted">
          Revoking a device signs it out immediately. It does not affect this machine or any
          other device.
        </p>
      </div>
    </Card>
  )
}

export function SettingsPage() {
  // The phone has four things to configure: whether Garmin is connected, a
  // sync, the key itself, and what this app is. The desktop keeps the
  // whole set — it has the room, and the network route and the history
  // windows are its business.
  if (isPhone()) return <PhoneSettings />
  return <DesktopSettings />
}

function DesktopSettings() {
  const queryClient = useQueryClient()

  const authQuery = useQuery({ queryKey: ['auth-status'], queryFn: api.authStatus })
  const syncStatusQuery = useQuery({
    queryKey: ['sync-status'],
    queryFn: api.syncStatus,
    refetchInterval: 15_000,
  })
  const syncRunsQuery = useQuery({ queryKey: ['sync-runs'], queryFn: api.syncRuns })
  const networkQuery = useQuery({ queryKey: ['network-status'], queryFn: api.networkStatus })

  const auth = authQuery.data

  const refreshAuth = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ['auth-status'] }),
      queryClient.invalidateQueries({ queryKey: ['sync-status'] }),
    ])
  }

  return (
    <div className="mx-auto max-w-[860px]">
      <div className="mb-5">
        <h1 className="text-[26px] font-bold leading-tight tracking-tight text-ink">
          Settings
        </h1>
        <p className="mt-1 text-[14px] text-ink-muted">
          Connect your Garmin watch, choose how Stride reaches the internet, and decide how
          often your runs are imported.
        </p>
      </div>
      <div className="space-y-6">
        <GarminConnectionCard
          auth={auth}
          loading={authQuery.isLoading}
          error={authQuery.isError ? messageOf(authQuery.error) : null}
          onRefresh={refreshAuth}
        />

        <NetworkRouteCard
          loading={networkQuery.isLoading}
          error={networkQuery.isError ? messageOf(networkQuery.error) : null}
          network={networkQuery.data}
          authRoute={auth?.network_route ?? null}
          exitNode={auth?.exit_node ?? null}
        />

        <SyncSettingsCard
          loading={authQuery.isLoading}
          auth={auth}
          syncing={syncStatusQuery.data?.running ?? false}
          syncMessage={syncStatusQuery.data?.message ?? null}
          syncError={syncStatusQuery.data?.error ?? null}
        />

        <HistoryCard syncing={syncStatusQuery.data?.running ?? false} />

        <SyncHistoryCard
          loading={syncRunsQuery.isLoading}
          error={syncRunsQuery.isError ? messageOf(syncRunsQuery.error) : null}
          runs={syncRunsQuery.data?.items ?? []}
        />

        <PairedDevicesCard />

        <AboutCard />
      </div>
    </div>
  )
}

/* ---------------------------------------------------- the phone's screen */

/**
 * The phone's settings screen.
 *
 * Two cards: the state of the Garmin connection, and what this app is.
 *
 * **The connection is a state, not a form.** The card says whether Garmin
 * is connected and when it last synced, and offers one button that opens
 * the native screen. There is no field to fill in, so there is nothing that
 * can be half-submitted, and the key never comes near the WebView.
 *
 * v1 views the data. The phone does not author a plan, edit a run, write
 * notes, or configure a server it does not have.
 */
function PhoneSettings() {
  const { connected, syncing, lastSyncAt } = useGarminState()

  return (
    <div className="mx-auto max-w-[560px]">
      <h1 className="mb-5 text-[26px] font-bold leading-tight tracking-tight text-ink">
        Settings
      </h1>

      <div className="space-y-6">
        <Card>
          <CardHeader title="Garmin" />
          <div className="px-5 pb-5">
            <div className="flex items-center gap-2">
              <span
                className={cn(
                  'h-2.5 w-2.5 rounded-full',
                  connected ? 'bg-positive' : 'bg-ink-faint',
                )}
              />
              <p className="text-[15px] font-semibold text-ink">
                {connected ? 'Connected' : 'Not connected'}
              </p>
            </div>
            <p className="mt-1 text-[13px] text-ink-muted">
              {syncing
                ? 'Syncing…'
                : lastSyncAt
                  ? `Synced ${formatRelative(lastSyncAt)}`
                  : 'Not synced yet'}
            </p>

            <div className="mt-4">
              <ConnectGarmin />
            </div>
          </div>
        </Card>

        <Card>
          <CardHeader title="About" />
          <div className="px-5 pb-5">
            <p className="text-[13px] leading-relaxed text-ink-soft">{localDataLine()}</p>
          </div>
        </Card>
      </div>
    </div>
  )
}

/* ------------------------------------------------ 1. Garmin connection */

function GarminConnectionCard({
  auth,
  loading,
  error,
  onRefresh,
}: {
  auth: AuthStatus | undefined
  loading: boolean
  error: string | null
  onRefresh: () => Promise<void>
}) {
  const [notice, setNotice] = useState<Notice | null>(null)
  const [confirmDisconnect, setConfirmDisconnect] = useState(false)

  const logout = useMutation({
    mutationFn: () => api.logout(),
    onSuccess: async () => {
      setNotice({ tone: 'positive', text: 'Disconnected. Your imported activities are safe.' })
      setConfirmDisconnect(false)
      await onRefresh()
    },
    onError: (err) =>
      setNotice({ tone: 'negative', text: messageOf(err, 'Could not disconnect.') }),
  })

  if (loading) {
    return (
      <Card>
        <CardHeader title="Garmin connection" subtitle="Checking your connection…" />
        <div className="space-y-2 px-5 pb-5">
          <Skeleton className="h-16 w-full" />
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader title="Garmin connection" />
        <div className="px-5 pb-5">
          <Alert tone="negative" title="We couldn't check your Garmin connection">
            {error}
          </Alert>
        </div>
      </Card>
    )
  }

  if (!auth) {
    return (
      <Card>
        <CardHeader title="Garmin connection" />
        <div className="px-5 pb-5">
          <Alert tone="negative" title="We couldn't check your Garmin connection">
            The backend did not answer. Try again in a moment.
          </Alert>
        </div>
      </Card>
    )
  }

  if (auth.garmin_connected) {
    const name = athleteName(auth.athlete)
    return (
      <Card>
        <CardHeader
          title="Garmin connection"
          subtitle="Your watch is linked — Stride imports your runs on its own."
          action={
            <Button
              variant={confirmDisconnect ? 'danger' : 'secondary'}
              size="sm"
              disabled={logout.isPending}
              onClick={() => {
                if (confirmDisconnect) logout.mutate()
                else setConfirmDisconnect(true)
              }}
            >
              {logout.isPending ? (
                <Loader2 size={14} strokeWidth={2} className="animate-spin" />
              ) : (
                <LogOut size={14} strokeWidth={2} />
              )}
              {confirmDisconnect ? 'Yes, disconnect' : 'Disconnect'}
            </Button>
          }
        />
        <div className="px-5 pb-5">
          {confirmDisconnect ? (
            <div className="mb-3">
              <Alert tone="warn" title="Disconnect your Garmin account?">
                Stride stops importing new activities. Everything already imported stays in your
                history.
              </Alert>
              <div className="mt-2 flex gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setConfirmDisconnect(false)}
                  disabled={logout.isPending}
                >
                  Keep connected
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => logout.mutate()}
                  disabled={logout.isPending}
                >
                  Disconnect
                </Button>
              </div>
            </div>
          ) : null}

          <div className="rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3">
            <div className="flex flex-wrap items-center gap-2">
              <span className="h-2.5 w-2.5 rounded-full bg-positive" />
              <p className="text-[14px] font-semibold text-emerald-900">Connected</p>
              <Badge tone="positive">{methodLabel(auth.garmin_method)}</Badge>
            </div>
            <dl className="mt-2 grid grid-cols-1 gap-x-6 gap-y-1 sm:grid-cols-2">
              <div className="flex items-baseline gap-2">
                <dt className="text-[12px] uppercase tracking-wide text-emerald-800/70">
                  Athlete
                </dt>
                <dd className="text-[13px] font-medium text-emerald-900">
                  {name ?? 'Anonymous — Garmin shares no name'}
                </dd>
              </div>
              <div className="flex items-baseline gap-2">
                <dt className="text-[12px] uppercase tracking-wide text-emerald-800/70">
                  Last sync
                </dt>
                <dd className="text-[13px] font-medium text-emerald-900">
                  {auth.last_sync_at
                    ? `${formatRelative(auth.last_sync_at)} (${formatDateTime(auth.last_sync_at)})`
                    : 'Not synced yet'}
                </dd>
              </div>
            </dl>
          </div>

          <div className="mt-3">
            <Toast notice={notice} />
          </div>
        </div>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader
        title="Garmin connection"
        subtitle="Pick whichever suits you — you can change later, and nothing is lost."
      />
      <div className="px-5 pb-5">
        <div className="mb-4">
          <Alert tone="info" title="Not connected yet">
            Connect your watch. Your runs start appearing on their own.
          </Alert>
        </div>
        <Toast notice={notice} />
        <div className="mt-4 space-y-6">
          <PersonalKeySection onRefresh={onRefresh} />
          {/*
            The phone connects with a Personal API key and nothing else. It
            never asks for a Garmin password and it never runs an OAuth
            dance, so offering those forms there would be a dead end - and
            asking for a password on a screen is a thing not to do.
          */}
          {!isNativeApp() ? (
            <>
              <Divider />
              <AccountSection onRefresh={onRefresh} />
              <Divider />
              <DeveloperKeySection onRefresh={onRefresh} />
            </>
          ) : null}
          <Divider />
          <FitImportSection />
        </div>
      </div>
    </Card>
  )
}

/* ------------------------------------------------- 1a. developer key */

interface AuthorizeStep {
  authorize_url: string
  instructions: string
  request_token: string
}

function DeveloperKeySection({ onRefresh }: { onRefresh: () => Promise<void> }) {
  const [consumerKey, setConsumerKey] = useState('')
  const [consumerSecret, setConsumerSecret] = useState('')
  const [code, setCode] = useState('')
  const [authorize, setAuthorize] = useState<AuthorizeStep | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)

  const start = useMutation({
    mutationFn: () => api.startDeveloperKey(consumerKey.trim(), consumerSecret.trim()),
    onSuccess: (data) => {
      setAuthorize({
        authorize_url: data.authorize_url,
        instructions: data.instructions,
        request_token: data.request_token,
      })
      setNotice({
        tone: 'info',
        text: 'Step 1 done. Open the Garmin authorisation page and approve the app.',
      })
    },
    onError: (error) => {
      setAuthorize(null)
      setNotice({ tone: 'negative', text: messageOf(error) })
    },
  })

  const exchange = useMutation({
    mutationFn: () => api.exchangeDeveloperKey(authorize?.request_token ?? '', code.trim()),
    onSuccess: async () => {
      setAuthorize(null)
      setCode('')
      setConsumerKey('')
      setConsumerSecret('')
      setNotice({ tone: 'positive', text: 'Garmin is connected. Your first sync starts now.' })
      await onRefresh()
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  return (
    <section>
      <SectionIntro title="Developer API key" badge="Advanced" badgeTone="neutral">
        Stride talks to Garmin with your own developer key. Your Garmin password is never typed
        here, and Stride only reads your activities.
      </SectionIntro>

      <p className="mb-3 rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-[13px] text-amber-900">
        A developer key covers <strong>activities only</strong>. Sleep, HRV, body battery and
        readiness live on Garmin&rsquo;s separate Health API, so the{' '}
        <Link to="/physiology" className="font-semibold underline">
          physiology page
        </Link>{' '}
        needs the account connection below.
      </p>

      <HowItWorks>
        Create an app at{' '}
        <span className="font-semibold text-ink-soft">developer.garmin.com</span> and copy the
        consumer key and the consumer secret below.
      </HowItWorks>

      {!authorize ? (
        <>
          <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Field
              label="Consumer Key"
              hint="The long key from your Garmin app. It looks like a mix of letters and numbers."
            >
              <input
                className={inputClass}
                value={consumerKey}
                onChange={(event) => setConsumerKey(event.target.value)}
                placeholder="e.g. 3f9a…b71c"
                autoComplete="off"
                spellCheck={false}
              />
            </Field>
            <Field
              label="Consumer Secret"
              hint="The second code, shown once when you create the app. Keep it to yourself."
            >
              <input
                className={inputClass}
                type="password"
                value={consumerSecret}
                onChange={(event) => setConsumerSecret(event.target.value)}
                placeholder="Your consumer secret"
                autoComplete="off"
                spellCheck={false}
              />
            </Field>
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-3">
            <Button
              onClick={() => start.mutate()}
              disabled={start.isPending || !consumerKey.trim() || !consumerSecret.trim()}
            >
              {start.isPending ? (
                <Loader2 size={15} strokeWidth={2} className="animate-spin" />
              ) : (
                <KeyRound size={15} strokeWidth={2} />
              )}
              {start.isPending ? 'Checking your key…' : 'Connect with key'}
            </Button>
            {!consumerKey.trim() || !consumerSecret.trim() ? (
              <span className="text-[12px] text-ink-muted">
                Fill in both fields to continue.
              </span>
            ) : null}
          </div>
        </>
      ) : (
        <div className="mt-3 rounded-md border border-edge bg-surface-soft px-4 py-4">
          <p className="text-[13px] font-semibold text-ink">Step 2 — approve in Garmin</p>
          <p className="mt-1 text-[13px] leading-snug text-ink-muted">
            {authorize.instructions}
          </p>

          <a
            href={authorize.authorize_url}
            target="_blank"
            rel="noreferrer noopener"
            className="mt-3 inline-flex h-9 items-center gap-1.5 rounded-md bg-brand-500 px-4 text-[14px] font-semibold text-white transition-colors hover:bg-brand-600"
          >
            <ExternalLink size={15} strokeWidth={2} />
            Open Garmin authorisation
          </a>
          <p className="mt-2 break-all text-[12px] text-ink-faint">
            If the button does not open, copy this address: {authorize.authorize_url}
          </p>

          <div className="mt-4 border-t border-edge pt-4">
            <p className="text-[13px] font-semibold text-ink">
              Step 3 — paste the authorisation code
            </p>
            <p className="mt-1 text-[13px] leading-snug text-ink-muted">
              Garmin usually sends you back to Stride on its own. If it shows you a code instead
              (for example a six-character code), paste it here to finish.
            </p>
            <div className="mt-3 flex flex-wrap items-end gap-3">
              <div className="w-full max-w-[280px]">
                <Field label="Authorisation code">
                  <input
                    className={inputClass}
                    value={code}
                    onChange={(event) => setCode(event.target.value)}
                    placeholder="Paste the code Garmin gave you"
                    autoComplete="off"
                    spellCheck={false}
                  />
                </Field>
              </div>
              <Button
                onClick={() => exchange.mutate()}
                disabled={exchange.isPending || !code.trim()}
              >
                {exchange.isPending ? (
                  <Loader2 size={15} strokeWidth={2} className="animate-spin" />
                ) : (
                  <CheckCircle2 size={15} strokeWidth={2} />
                )}
                {exchange.isPending ? 'Finishing…' : 'Finish connection'}
              </Button>
              <Button
                variant="ghost"
                onClick={() => {
                  setAuthorize(null)
                  setNotice(null)
                }}
                disabled={exchange.isPending}
              >
                Start over
              </Button>
            </div>
          </div>
        </div>
      )}

      {notice ? (
        <div className="mt-3">
          <Toast notice={notice} />
        </div>
      ) : null}
    </section>
  )
}

/* ------------------------------------------- 1a. personal API key */

function PersonalKeySection({ onRefresh }: { onRefresh: () => Promise<void> }) {
  const [apiKey, setApiKey] = useState('')
  const [notice, setNotice] = useState<Notice | null>(null)

  const connect = useMutation({
    mutationFn: () => api.connectPersonalKey(apiKey.trim()),
    onSuccess: async (data) => {
      setApiKey('')
      setNotice({
        tone: 'positive',
        text: `Garmin is connected.${data.athlete?.name ? ` Welcome, ${data.athlete.name}.` : ''} Your first sync starts now.`,
      })
      await onRefresh()
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const test = useMutation({
    mutationFn: () => api.testPersonalKey(apiKey.trim()),
    onSuccess: (data) =>
      setNotice({
        tone: 'positive',
        text: `That key works${data.athlete?.name ? ` — it belongs to ${data.athlete.name}` : ''}.`,
      }),
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const busy = connect.isPending || test.isPending

  return (
    <section>
      <SectionIntro title="Personal API key — recommended" badge="Easiest" badgeTone="brand">
        Your own key from Garmin. No developer account and no consumer secret — and it covers
        sleep, resting heart rate and body battery as well as your activities.
      </SectionIntro>

      <HowItWorks>
        Get a key from{' '}
        <span className="font-semibold text-ink-soft">
          connect.garmin.com/app/settings/personalApiToken
        </span>
        . Garmin shows it once — copy it and paste it below.
      </HowItWorks>

      <Field label="Personal API key" hint={localDataLine()}>
        <input
          type="password"
          autoComplete="off"
          spellCheck={false}
          placeholder="GCDC_..."
          className={inputClass}
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
        />
      </Field>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Button size="md" disabled={busy || !apiKey.trim()} onClick={() => connect.mutate()}>
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
      </div>

      {notice ? (
        <div className="mt-3">
          <Toast notice={notice} />
        </div>
      ) : null}
    </section>
  )
}

/* --------------------------------------------------- 1b. account + MFA */

function AccountSection({ onRefresh }: { onRefresh: () => Promise<void> }) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [code, setCode] = useState('')
  const [mfaToken, setMfaToken] = useState<string | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)

  const login = useMutation({
    mutationFn: () => api.login(email.trim(), password),
    onSuccess: async (result) => {
      if (result.status === 'mfa_required' && result.mfa_token) {
        setMfaToken(result.mfa_token)
        setNotice({
          tone: 'info',
          text: 'Garmin sent a verification code to your phone. Enter it below.',
        })
        return
      }
      setPassword('')
      setNotice({ tone: 'positive', text: 'Connected. Your first sync starts now.' })
      await onRefresh()
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const mfa = useMutation({
    mutationFn: () => api.submitMfa(mfaToken ?? '', code.trim()),
    onSuccess: async () => {
      setMfaToken(null)
      setCode('')
      setPassword('')
      setNotice({ tone: 'positive', text: 'Verified. Your Garmin account is connected.' })
      await onRefresh()
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  return (
    <section>
      <SectionIntro title="Garmin account">
        Sign in with your normal Garmin email and password.
      </SectionIntro>

      <HowItWorks>Garmin sends a one-time code to your phone.</HowItWorks>

      {!mfaToken ? (
        <>
          <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Field label="Email" hint="The email address you sign in to Garmin with.">
              <input
                className={inputClass}
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="you@example.com"
                autoComplete="username"
              />
            </Field>
            <Field label="Password" hint={localDataLine()}>
              <input
                className={inputClass}
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder="Your password"
                autoComplete="current-password"
              />
            </Field>
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-3">
            <Button
              onClick={() => login.mutate()}
              disabled={login.isPending || !email.trim() || !password}
            >
              {login.isPending ? (
                <Loader2 size={15} strokeWidth={2} className="animate-spin" />
              ) : (
                <Mail size={15} strokeWidth={2} />
              )}
              {login.isPending ? 'Signing in…' : 'Connect account'}
            </Button>
            {!email.trim() || !password ? (
              <span className="text-[12px] text-ink-muted">
                Fill in both fields to continue.
              </span>
            ) : null}
          </div>
        </>
      ) : (
        <div className="mt-3 rounded-md border border-edge bg-surface-soft px-4 py-4">
          <p className="text-[13px] font-semibold text-ink">Two-step verification</p>
          <p className="mt-1 text-[13px] leading-snug text-ink-muted">
            Enter the code Garmin sent to your phone.
          </p>
          <div className="mt-3 flex flex-wrap items-end gap-3">
            <div className="w-full max-w-[220px]">
              <Field label="Verification code">
                <input
                  className={inputClass}
                  value={code}
                  onChange={(event) => setCode(event.target.value)}
                  placeholder="6-digit code"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                />
              </Field>
            </div>
            <Button onClick={() => mfa.mutate()} disabled={mfa.isPending || !code.trim()}>
              {mfa.isPending ? (
                <Loader2 size={15} strokeWidth={2} className="animate-spin" />
              ) : (
                <CheckCircle2 size={15} strokeWidth={2} />
              )}
              {mfa.isPending ? 'Verifying…' : 'Verify'}
            </Button>
            <Button
              variant="ghost"
              onClick={() => {
                setMfaToken(null)
                setCode('')
                setNotice(null)
              }}
              disabled={mfa.isPending}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {notice ? (
        <div className="mt-3">
          <Toast notice={notice} />
        </div>
      ) : null}
    </section>
  )
}

/* ------------------------------------------------ 1c. FIT file import */

interface FitActivity {
  id?: number
  name?: string
  date?: string
  distance_m?: number
  created?: boolean
}

interface FitError {
  filename?: string
  error?: string
}

function FitImportSection() {
  const inputRef = useRef<HTMLInputElement>(null)
  const [fileNames, setFileNames] = useState<string[]>([])
  const [result, setResult] = useState<{
    imported: number
    activities: FitActivity[]
    errors: FitError[]
  } | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)

  const importFiles = useMutation({
    mutationFn: (files: File[]) => api.importFit(files),
    onSuccess: (data) => {
      const activities = (data.activities ?? []) as FitActivity[]
      const errors = (data.errors ?? []) as FitError[]
      setResult({ imported: data.imported ?? 0, activities, errors })
      setNotice({
        tone: (data.imported ?? 0) > 0 ? 'positive' : 'info',
        text:
          (data.imported ?? 0) > 0
            ? `Imported ${formatNumber(data.imported ?? 0)} activit${
                (data.imported ?? 0) === 1 ? 'y' : 'ies'
              }.`
            : 'No activities were imported. Check the messages below.',
      })
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const handleFiles = (files: FileList | null) => {
    if (!files || files.length === 0) return
    const list = Array.from(files)
    setFileNames(list.map((file) => file.name))
    importFiles.mutate(list)
  }

  return (
    <section>
      <SectionIntro title="Import FIT files" badge="No account needed" badgeTone="info">
        Already have files from Garmin Connect? Drop them in and they show up immediately.
      </SectionIntro>

      <HowItWorks>
        Export <span className="font-semibold text-ink-soft">.FIT</span> files from Garmin
        Connect (Activities → ⋯ → Download FIT) and choose them below.
      </HowItWorks>

      <div className="mt-3">
        <input
          ref={inputRef}
          type="file"
          accept=".fit,.FIT"
          multiple
          className="hidden"
          onChange={(event) => handleFiles(event.target.files)}
        />
        <div className="flex flex-wrap items-center gap-3">
          <Button
            variant="secondary"
            onClick={() => inputRef.current?.click()}
            disabled={importFiles.isPending}
          >
            {importFiles.isPending ? (
              <Loader2 size={15} strokeWidth={2} className="animate-spin" />
            ) : (
              <Upload size={15} strokeWidth={2} />
            )}
            {importFiles.isPending ? 'Importing…' : 'Choose FIT files'}
          </Button>
          {fileNames.length > 0 ? (
            <span className="text-[12px] text-ink-muted">
              {fileNames.length} file{fileNames.length === 1 ? '' : 's'} selected:{' '}
              {fileNames.slice(0, 3).join(', ')}
              {fileNames.length > 3 ? '…' : ''}
            </span>
          ) : null}
        </div>
      </div>

      {notice ? (
        <div className="mt-3">
          <Toast notice={notice} />
        </div>
      ) : null}

      {result ? (
        <div className="mt-3 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone={result.imported > 0 ? 'positive' : 'warn'}>
              <FileUp size={11} strokeWidth={2} />
              {formatNumber(result.imported)} imported
            </Badge>
            {result.errors.length > 0 ? (
              <Badge tone="negative">
                <CircleAlert size={11} strokeWidth={2} />
                {formatNumber(result.errors.length)} failed
              </Badge>
            ) : null}
          </div>

          {result.activities.length > 0 ? (
            <ul className="divide-y divide-edge rounded-md border border-edge">
              {result.activities.slice(0, 10).map((activity, index) => (
                <li
                  key={activity.id ?? index}
                  className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 text-[13px]"
                >
                  <span className="min-w-0 truncate font-medium text-ink">
                    {activity.name ?? 'Untitled activity'}
                  </span>
                  <span className="shrink-0 text-ink-muted">
                    {activity.date ? formatDateTime(activity.date) : '—'}
                    {typeof activity.distance_m === 'number'
                      ? ` · ${(activity.distance_m / 1000).toFixed(2)} km`
                      : ''}
                  </span>
                </li>
              ))}
              {result.activities.length > 10 ? (
                <li className="px-3 py-2 text-[12px] text-ink-muted">
                  …and {formatNumber(result.activities.length - 10)} more.
                </li>
              ) : null}
            </ul>
          ) : null}

          {result.errors.length > 0 ? (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2.5">
              <p className="text-[13px] font-semibold text-red-800">
                Some files could not be read
              </p>
              <ul className="mt-1 space-y-1">
                {result.errors.map((item, index) => (
                  <li key={index} className="text-[12px] leading-snug text-red-700">
                    <span className="font-semibold">{item.filename ?? 'Unnamed file'}:</span>{' '}
                    {item.error ?? 'Unknown error'}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>
      ) : null}
    </section>
  )
}

/* ----------------------------------------------------- 2. network route */

function NetworkRouteCard({
  loading,
  error,
  network,
  authRoute,
  exitNode,
}: {
  loading: boolean
  error: string | null
  network: NetworkStatus | undefined
  authRoute: 'auto' | 'direct' | 'exit_node' | null
  exitNode: string | null
}) {
  const queryClient = useQueryClient()
  const [notice, setNotice] = useState<Notice | null>(null)

  const route = network?.route ?? authRoute ?? 'auto'
  const identity = network?.public_identity
  const publicIp = identity?.ip ?? network?.public_ip ?? null
  const publicOrg = identity?.org ?? network?.public_org ?? null
  const nodes = network?.nodes ?? []
  const activeExitNode =
    exitNode ?? network?.exit_node ?? network?.resolved_exit_node ?? nodes[0]?.ip ?? null
  const canUseExitNode = Boolean(activeExitNode)
  // The route the machine is *actually* using right now.
  const effectiveRoute = network?.effective_route ?? null

  const setRoute = useMutation({
    mutationFn: (value: 'direct' | 'exit_node') => api.setRoute(value),
    onSuccess: async (result, value) => {
      setNotice({
        tone: 'positive',
        text:
          result.message ||
          (value === 'direct'
            ? 'Now using a direct connection.'
            : 'Now routing through your home connection.'),
      })
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['network-status'] }),
        queryClient.invalidateQueries({ queryKey: ['auth-status'] }),
      ])
    },
    onError: (err) => setNotice({ tone: 'negative', text: messageOf(err) }),
  })

  return (
    <Card>
      <CardHeader
        title="Network route"
        subtitle="How Stride reaches Garmin when it downloads your activities."
      />
      <div className="px-5 pb-5">
        <div className="rounded-md border border-sky-200 bg-sky-50 px-4 py-3">
          <div className="flex items-start gap-2">
            <Router size={16} strokeWidth={2} className="mt-0.5 shrink-0 text-sky-700" />
            <div className="text-[13px] leading-snug text-sky-900">
              <p className="font-semibold">Why this matters</p>
              <p className="mt-1">
                Garmin limits how much data a single internet address may pull. When Stride runs
                on a datacentre server it can get blocked. Routing through your home connection
                (a Tailscale exit node) makes Garmin see your normal home IP address instead, so
                downloads keep working.
              </p>
            </div>
          </div>
        </div>

        {loading ? (
          <div className="mt-4 space-y-2">
            <Skeleton className="h-16 w-full" />
            <Skeleton className="h-9 w-64" />
          </div>
        ) : error ? (
          <div className="mt-4">
            <Alert tone="negative" title="We couldn't read the network status">
              {error}
            </Alert>
          </div>
        ) : (
          <>
            <dl className="mt-4 grid grid-cols-1 gap-x-8 gap-y-3 sm:grid-cols-3">
              <div>
                <dt className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  Current route
                </dt>
                <dd className="mt-0.5 text-[14px] font-semibold text-ink">
                  {routeLabel(route)}
                  {route === 'exit_node' && activeExitNode ? (
                    <span className="ml-1.5 text-[12px] font-normal text-ink-muted">
                      {activeExitNode}
                    </span>
                  ) : null}
                  {effectiveRoute && effectiveRoute !== route ? (
                    <span className="ml-1.5 text-[12px] font-normal text-ink-muted">
                      (actually {routeLabel(effectiveRoute as 'direct' | 'exit_node')})
                    </span>
                  ) : null}
                </dd>
              </div>
              <div>
                <dt className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  Public IP address
                </dt>
                <dd className="mt-0.5 text-[14px] font-semibold text-ink">
                  {publicIp ?? 'Not detected yet'}
                </dd>
              </div>
              <div>
                <dt className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  Network operator
                </dt>
                <dd className="mt-0.5 text-[13px] leading-snug text-ink">
                  {publicOrg ?? 'Not detected yet'}
                </dd>
              </div>
            </dl>

            <Divider className="my-4" />

            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant={route === 'direct' ? 'primary' : 'secondary'}
                onClick={() => setRoute.mutate('direct')}
                disabled={setRoute.isPending || route === 'direct'}
              >
                <RouteIcon size={15} strokeWidth={2} />
                Use direct connection
              </Button>
              <Button
                variant={route === 'exit_node' ? 'primary' : 'secondary'}
                onClick={() => setRoute.mutate('exit_node')}
                disabled={setRoute.isPending || !canUseExitNode || route === 'exit_node'}
              >
                <Plug size={15} strokeWidth={2} />
                Route via home connection
              </Button>
              {setRoute.isPending ? (
                <Loader2 size={15} strokeWidth={2} className="animate-spin text-ink-muted" />
              ) : null}
            </div>

            {!canUseExitNode ? (
              <p className="mt-2 text-[12px] text-ink-muted">
                No home exit node is configured yet, so “route via home connection” is
                unavailable. Start Tailscale on your home machine and make it an exit node.
              </p>
            ) : null}

            {nodes.length > 0 ? (
              <div className="mt-4">
                <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  Available exit nodes
                </p>
                <ul className="mt-1.5 divide-y divide-edge rounded-md border border-edge">
                  {nodes.map((node) => (
                    <li
                      key={node.id}
                      className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 text-[13px]"
                    >
                      <span className="min-w-0 truncate font-medium text-ink">
                        {node.hostname || node.id}
                      </span>
                      <span className="flex items-center gap-2 text-ink-muted">
                        <span className="font-mono text-[12px]">{node.ip}</span>
                        <Badge tone={node.online ? 'positive' : 'neutral'}>
                          {node.online ? 'Online' : 'Offline'}
                        </Badge>
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </>
        )}

        {notice ? (
          <div className="mt-3">
            <Toast notice={notice} />
          </div>
        ) : null}
      </div>
    </Card>
  )
}

/* ----------------------------------------------------- 3. sync settings */

function SyncSettingsCard({
  loading,
  auth,
  syncing,
  syncMessage,
  syncError,
}: {
  loading: boolean
  auth: AuthStatus | undefined
  syncing: boolean
  syncMessage: string | null
  syncError: string | null
}) {
  const queryClient = useQueryClient()
  const [days, setDays] = useState('90')
  const [interval, setIntervalValue] = useState('180')
  const [notice, setNotice] = useState<Notice | null>(null)

  useEffect(() => {
    if (!auth) return
    setDays(String(auth.sync_days))
    setIntervalValue(String(auth.sync_interval_minutes))
  }, [auth?.sync_days, auth?.sync_interval_minutes])

  const saveSettings = useMutation({
    mutationFn: (patch: Record<string, unknown>) => api.syncSettings(patch),
    onSuccess: async () => {
      setNotice({ tone: 'positive', text: 'Sync settings saved.' })
      await queryClient.invalidateQueries({ queryKey: ['auth-status'] })
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const runSync = useMutation({
    mutationFn: () => api.runSync(),
    onSuccess: async (result) => {
      setNotice({
        tone: 'positive',
        text: result.reason
          ? result.reason
          : 'Sync started. New activities will appear in the feed shortly.',
      })
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['sync-status'] }),
        queryClient.invalidateQueries({ queryKey: ['sync-runs'] }),
      ])
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const daysNumber = Number.parseInt(days, 10)
  const intervalNumber = Number.parseInt(interval, 10)

  return (
    <Card>
      <CardHeader
        title="Sync settings"
        subtitle="How much Stride downloads and how often it checks for new activities."
      />
      <div className="px-5 pb-5">
        {loading ? (
          <div className="space-y-3">
            <Skeleton className="h-9 w-full max-w-xs" />
            <Skeleton className="h-9 w-full max-w-xs" />
          </div>
        ) : (
          <>
            <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
              <Field
                label="Days to sync on each run"
                hint="How far back Stride looks each time. 90 days is plenty for most people."
              >
                <input
                  className={inputClass}
                  type="number"
                  min={1}
                  max={3650}
                  value={days}
                  onChange={(event) => setDays(event.target.value)}
                  onBlur={() => {
                    if (Number.isFinite(daysNumber) && daysNumber >= 1) {
                      saveSettings.mutate({ sync_days: daysNumber })
                    }
                  }}
                />
              </Field>

              <Field
                label="Sync every (minutes)"
                hint="How often Stride checks Garmin in the background. Minimum 15 minutes."
              >
                <input
                  className={inputClass}
                  type="number"
                  min={15}
                  max={1440}
                  step={15}
                  value={interval}
                  onChange={(event) => setIntervalValue(event.target.value)}
                  onBlur={() => {
                    if (Number.isFinite(intervalNumber) && intervalNumber >= 15) {
                      saveSettings.mutate({ sync_interval_minutes: intervalNumber })
                    }
                  }}
                />
              </Field>
            </div>

            <div className="mt-4">
              <Toggle
                checked={auth?.auto_sync ?? false}
                onChange={(value) => saveSettings.mutate({ auto_sync: value })}
                label="Sync automatically"
                description="Stride keeps itself up to date in the background, so runs show up without you doing anything."
              />
            </div>

            <Divider className="my-4" />

            <div className="flex flex-wrap items-center gap-3">
              <Button onClick={() => runSync.mutate()} disabled={runSync.isPending || syncing}>
                {runSync.isPending || syncing ? (
                  <Loader2 size={15} strokeWidth={2} className="animate-spin" />
                ) : (
                  <RefreshCw size={15} strokeWidth={2} />
                )}
                {runSync.isPending ? 'Starting…' : syncing ? 'Sync running…' : 'Sync now'}
              </Button>
              <p className="text-[12px] text-ink-muted">
                A full sync of {formatNumber(daysNumber || 90)} days can take about a minute.
                You can keep using Stride while it works.
              </p>
            </div>

            {syncError ? (
              <div className="mt-3">
                <Alert tone="negative" title="The last sync did not finish">
                  {syncError}
                </Alert>
              </div>
            ) : syncMessage ? (
              <div className="mt-3">
                <Alert tone="info">{syncMessage}</Alert>
              </div>
            ) : null}
          </>
        )}

        {notice ? (
          <div className="mt-3">
            <Toast notice={notice} />
          </div>
        ) : null}
      </div>
    </Card>
  )
}

/* ------------------------------------------------------ 4. sync history */

function SyncHistoryCard({
  loading,
  error,
  runs,
}: {
  loading: boolean
  error: string | null
  runs: SyncRun[]
}) {
  return (
    <Card>
      <CardHeader title="Sync history" subtitle="The last few times Stride talked to Garmin." />
      <div className="px-5 pb-5">
        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-9 w-full" />
            <Skeleton className="h-9 w-full" />
            <Skeleton className="h-9 w-full" />
          </div>
        ) : error ? (
          <Alert tone="negative" title="We couldn't load the sync history">
            {error}
          </Alert>
        ) : runs.length === 0 ? (
          <div className="rounded-md border border-dashed border-edge px-4 py-8 text-center">
            <p className="text-[14px] font-semibold text-ink">No syncs yet</p>
            <p className="mt-1 text-[13px] text-ink-muted">
              Once Garmin is connected, every sync will be listed here.
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[560px] border-collapse text-left">
              <thead>
                <tr className="border-b border-edge">
                  <th className="py-2 pr-3 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    When
                  </th>
                  <th className="py-2 pr-3 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    Source
                  </th>
                  <th className="py-2 pr-3 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    Status
                  </th>
                  <th className="py-2 pr-3 text-right text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    Found
                  </th>
                  <th className="py-2 pr-3 text-right text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    New
                  </th>
                  <th className="py-2 text-right text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                    Updated
                  </th>
                </tr>
              </thead>
              <tbody>
                {runs.map((run) => (
                  <tr key={run.id} className="border-b border-edge last:border-b-0">
                    <td className="py-2.5 pr-3 text-[13px] text-ink">
                      {formatDateTime(run.started_at)}
                    </td>
                    <td className="py-2.5 pr-3 text-[13px] text-ink-muted">
                      {run.source || 'garmin'}
                    </td>
                    <td className="py-2.5 pr-3">
                      <Badge
                        tone={
                          run.status === 'ok'
                            ? 'positive'
                            : run.status === 'error'
                              ? 'negative'
                              : 'info'
                        }
                      >
                        {run.status}
                      </Badge>
                    </td>
                    <td className="py-2.5 pr-3 text-right text-[13px] text-ink">
                      {formatNumber(run.activities_found)}
                    </td>
                    <td className="py-2.5 pr-3 text-right text-[13px] font-semibold text-ink">
                      {formatNumber(run.activities_new)}
                    </td>
                    <td className="py-2.5 text-right text-[13px] text-ink-muted">
                      {formatNumber(run.activities_updated)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {runs.some((run) => run.error) ? (
              <ul className="mt-3 space-y-1">
                {runs
                  .filter((run) => run.error)
                  .map((run) => (
                    <li
                      key={`error-${run.id}`}
                      className="text-[12px] leading-snug text-negative"
                    >
                      {formatDateTime(run.started_at)} — {run.error}
                    </li>
                  ))}
              </ul>
            ) : null}
          </div>
        )}
      </div>
    </Card>
  )
}

/* ------------------------------------------------------------ 5. about */

/** Choose how much history to pull from Garmin. */
function HistoryCard({ syncing }: { syncing: boolean }) {
  // 0 means "everything the account holds".
  const [months, setMonths] = useState(12)
  const [healthDays, setHealthDays] = useState<number>(90)
  const [notice, setNotice] = useState<Notice | null>(null)

  const history = useQuery({ queryKey: ['history-settings'], queryFn: api.historySettings })
  const status = useQuery({
    queryKey: ['sync-status'],
    queryFn: api.syncStatus,
    refetchInterval: 5000,
  })

  const load = useMutation({
    mutationFn: () => api.loadHistory(months === 0 ? null : months, healthDays),
    onSuccess: async () => {
      setNotice({
        tone: 'positive',
        text: 'Loading history in the background. This can take a few minutes.',
      })
      await Promise.all([history.refetch(), status.refetch()])
    },
    onError: (error) => setNotice({ tone: 'negative', text: messageOf(error) }),
  })

  const activityOptions: { label: string; value: number }[] = [
    { label: '3 months', value: 3 },
    { label: '6 months', value: 6 },
    { label: '12 months', value: 12 },
    { label: 'Everything', value: 0 },
  ]
  const healthOptions: { label: string; value: number }[] = [
    { label: '30 days', value: 30 },
    { label: '90 days', value: 90 },
    { label: '12 months', value: 365 },
  ]

  return (
    <Card>
      <CardHeader
        title="History"
        subtitle="How much of your Garmin past to pull in."
        action={
          status.data?.running ? (
            <Badge tone="info">
              <Loader2 size={11} className="animate-spin" />
              Loading
            </Badge>
          ) : null
        }
      />
      <div className="px-5 pb-5">
        <p className="mb-4 text-[13px] leading-relaxed text-ink-muted">
          Garmin's activity service serves everything your account holds — there is no short
          retention window on runs. Health data (sleep, resting heart rate, stress, body
          battery) goes back about a year. Loading more takes longer: each run is fetched with
          its original file so the route and per-second streams stay intact.
        </p>

        <div className="grid gap-4 sm:grid-cols-2">
          <div>
            <p className="mb-1.5 text-[13px] font-semibold text-ink-soft">Run history</p>
            <SegmentedControl
              options={activityOptions}
              value={months}
              onChange={(value) => setMonths(value as number)}
            />
          </div>
          <div>
            <p className="mb-1.5 text-[13px] font-semibold text-ink-soft">Health history</p>
            <SegmentedControl
              options={healthOptions}
              value={healthDays}
              onChange={(value) => setHealthDays(value as number)}
            />
          </div>
        </div>

        <div className="mt-4 flex flex-wrap items-center gap-3">
          <Button size="md" disabled={load.isPending || syncing} onClick={() => load.mutate()}>
            {load.isPending || syncing ? (
              <Loader2 size={14} strokeWidth={2} className="animate-spin" />
            ) : (
              <Download size={14} strokeWidth={2} />
            )}
            Load history
          </Button>
          {status.data?.message ? (
            <span className="text-[12px] text-ink-muted">{status.data.message}</span>
          ) : null}
        </div>

        {history.data ? (
          <p className="mt-3 text-[12px] text-ink-muted">
            Currently loading {history.data.history_months ?? 'all'} of runs and{' '}
            {history.data.health_history_days ?? 0} days of health data.
          </p>
        ) : null}

        {notice ? (
          <div className="mt-3">
            <Toast notice={notice} />
          </div>
        ) : null}
      </div>
    </Card>
  )
}

function AboutCard() {
  return (
    <Card>
      <CardHeader title="About" />
      <div className="px-5 pb-5">
        <p className="text-[13px] leading-relaxed text-ink-soft">{localDataLine()}</p>
        <p className="mt-3 text-[13px] leading-relaxed text-ink-soft">
          Stride is a private running log for your Garmin watch.
        </p>
      </div>
    </Card>
  )
}
