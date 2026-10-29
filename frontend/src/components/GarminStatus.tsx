/**
 * The Garmin connection — the one place the app says whether Garmin is
 * connected, and the one button that connects it.
 *
 * **On the phone the key is typed in the app, not in the web app.** The
 * key is wrapped by a Keystore key and kept out of the WebView, so the
 * web app must not offer a key form there: it offers **Connect Garmin**,
 * which opens the native screen, and that is the only way in.
 *
 * **On the desktop the key form stays.** The desktop's key goes to the
 * desktop's server, and the desktop has the room for it.
 */

import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { CircleAlert, Plug, RefreshCw, Wifi } from 'lucide-react'
import { Link } from 'react-router-dom'
import { api, openNativeGarminSetup } from '../lib/api'
import { useGarminState } from '../lib/garmin'
import { isPhone } from '../lib/surface'
import { formatNumber, formatRelative } from '../lib/format'
import { Badge, Button, cn } from './ui'
/**
 * The indicator in the top bar.
 *
 * It says one thing: whether Garmin is connected. Every other screen reads
 * the same hook, so the app has one answer and not three that can disagree.
 */
export function GarminPill() {
  const { connected, syncing, lastSyncAt, runs } = useGarminState()
  const phone = isPhone()

  // The exit node is the desktop's business: the phone has no server and no
  // route to choose.
  const { data: network } = useQuery({
    queryKey: ['network-status'],
    queryFn: api.networkStatus,
    enabled: !phone,
  })

  const syncNow = useMutation({ mutationFn: () => api.runSync() })

  if (syncing) {
    return (
      <Badge tone="info">
        <RefreshCw size={11} className="animate-spin" />
        Syncing
      </Badge>
    )
  }

  if (!connected) {
    return <ConnectGarmin small />
  }

  const label = phone
    ? // The same words the native screen uses, from the same number.
      `Connected · ${formatNumber(runs)} ${runs === 1 ? 'run' : 'runs'}`
    : lastSyncAt
      ? `Connected · ${formatRelative(lastSyncAt)}`
      : 'Connected'

  return (
    <button
      onClick={() => (phone ? undefined : syncNow.mutate())}
      disabled={phone || syncNow.isPending}
      className="inline-flex items-center gap-2 rounded-full border border-edge bg-surface px-3 py-1.5 text-[12px] text-ink-soft transition-colors enabled:hover:bg-surface-soft"
      title={phone ? 'Garmin' : 'Sync now'}
    >
      <span className="h-2 w-2 rounded-full bg-positive" />
      <span className="font-medium">{label}</span>
      {phone ? null : (
        <RefreshCw size={12} className={cn(syncNow.isPending && 'animate-spin')} />
      )}
      {network?.exit_node_active ? (
        <Wifi size={12} className="text-brand-500" aria-label="Routed via home exit node" />
      ) : null}
    </button>
  )
}

/**
 * **Connect Garmin.**
 *
 * On the phone this opens the native screen, where the key is typed once
 * and the sync follows. On the desktop it opens the settings screen, where
 * the key forms are.
 */
export function ConnectGarmin({
  small,
  className,
  label = 'Connect Garmin',
}: {
  small?: boolean
  className?: string
  label?: string
}) {
  const [error, setError] = useState<string | null>(null)

  const open = useMutation({
    mutationFn: () => openNativeGarminSetup(),
    onError: (cause: Error) => setError(cause.message),
    onSuccess: () => setError(null),
  })

  if (!isPhone()) {
    return (
      <Link to="/settings" className={className}>
        <Button size={small ? 'sm' : 'md'}>
          <Plug size={14} strokeWidth={2} />
          {label}
        </Button>
      </Link>
    )
  }

  return (
    <span className={cn('inline-flex flex-col items-start gap-1', className)}>
      <Button
        size={small ? 'sm' : 'md'}
        disabled={open.isPending}
        onClick={() => open.mutate()}
      >
        <Plug size={14} strokeWidth={2} />
        {label}
      </Button>
      {error ? (
        <span className="inline-flex items-center gap-1 text-[12px] text-negative">
          <CircleAlert size={12} strokeWidth={2} />
          {error}
        </span>
      ) : null}
    </span>
  )
}
