/**
 * The empty state every screen shares.
 *
 * **The app opens empty now.** The demo dataset is gone, so a new
 * installation has no runs, no records, no charts — and the first thing it
 * says has to be the next step, not a blank panel.
 *
 * Two states, and they are not the same thing:
 *
 * * **Garmin is not connected** — the app has nothing because nothing has
 *   been asked for yet. The state says what to do and offers the one
 *   button that does it. On the phone that button opens the native
 *   screen, where the key is typed once.
 * * **Garmin is connected** — the app has nothing because the first sync
 *   has not landed yet. Nothing is being asked for, so nothing is
 *   offered.
 */

import { Plug } from 'lucide-react'
import { useGarminState } from '../lib/garmin'
import { localDataLine } from '../lib/surface'
import { EmptyState } from './ui'
import { ConnectGarmin } from './GarminStatus'

export function ConnectPrompt({
  what = 'your runs',
  emptyTitle,
  emptyDescription,
}: {
  what?: string
  emptyTitle?: string
  emptyDescription?: string
}) {
  const { connected } = useGarminState()

  if (connected) {
    return (
      <EmptyState
        icon={<Plug size={22} strokeWidth={1.8} />}
        title={emptyTitle ?? `No ${what.replace('your ', '')} yet`}
        description={emptyDescription ?? 'This fills in once the first sync lands.'}
      />
    )
  }

  return (
    <EmptyState
      icon={<Plug size={22} strokeWidth={1.8} />}
      title={`Connect Garmin to see ${what}`}
      description={`Add your Garmin key. ${localDataLine()}`}
      action={<ConnectGarmin />}
    />
  )
}
