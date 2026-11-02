/**
 * The shell.
 *
 * Three arrangements of one information architecture:
 *
 *   compact  (< 600 px)  a bottom tab bar
 *   medium   (600–1023)  a top bar
 *   wide     (>= 1024)   a left rail
 *
 * Which one is showing is decided by **container** width, in CSS — no
 * JavaScript reads the viewport, and nothing sniffs the device. All three
 * arrangements are in the markup; the container query picks one, so the
 * destination set is identical at every width by construction.
 */

import type { ReactNode } from 'react'
import { NavLink } from 'react-router-dom'
import { Settings } from 'lucide-react'
import { DESTINATIONS, type Destination } from '../../app/destinations'
import { Brand } from './Brand'
import { t } from '../../i18n'

function DestinationLink({
  destination,
  variant,
}: {
  destination: Destination
  variant: 'tab' | 'bar' | 'rail'
}) {
  const label = t(destination.labelKey)
  const Icon = destination.icon

  if (variant === 'tab') {
    return (
      <NavLink
        to={destination.to}
        end={destination.end}
        data-destination={destination.to}
        className={({ isActive }) =>
          [
            'tap flex flex-1 flex-col items-center justify-center gap-0.5 py-1.5 text-[11px] font-medium transition-colors',
            isActive ? 'text-brand-600' : 'text-ink-muted',
          ].join(' ')
        }
      >
        <Icon size={19} strokeWidth={2} />
        <span>{label}</span>
      </NavLink>
    )
  }

  if (variant === 'bar') {
    return (
      <NavLink
        to={destination.to}
        end={destination.end}
        data-destination={destination.to}
        className={({ isActive }) =>
          [
            'tap rounded-md px-3 text-[14px] font-semibold transition-colors',
            isActive ? 'bg-brand-50 text-brand-700' : 'text-ink-soft hover:text-ink',
          ].join(' ')
        }
      >
        {label}
      </NavLink>
    )
  }

  return (
    <NavLink
      to={destination.to}
      end={destination.end}
      data-destination={destination.to}
      className={({ isActive }) =>
        [
          'tap flex items-center gap-2.5 rounded-md px-3 text-[14px] font-medium transition-colors',
          isActive
            ? 'bg-brand-50 text-brand-700'
            : 'text-ink-soft hover:bg-surface-soft hover:text-ink',
        ].join(' ')
      }
    >
      <Icon size={17} strokeWidth={2} />
      {label}
    </NavLink>
  )
}

/**
 * The settings link.
 *
 * **It is not a destination.** The destination set is four — Today, Runs,
 * Progress, Body — and it is the same four at every width. The settings
 * screen is secondary, and each band puts the link where it has room:
 * a gear at the end of the tab bar at compact, a gear at the end of the
 * top bar at medium, and a link under the destinations in the rail.
 */
function SettingsLink({ variant }: { variant: 'bar' | 'rail' }) {
  if (variant === 'bar') {
    return (
      <NavLink
        to="/settings"
        aria-label={t('nav.settings')}
        data-testid="shell-settings"
        className={({ isActive }) =>
          [
            'tap ml-auto flex items-center justify-center rounded-md px-2',
            isActive ? 'bg-brand-50 text-brand-700' : 'text-ink-muted hover:text-ink',
          ].join(' ')
        }
      >
        <Settings size={19} strokeWidth={2} />
      </NavLink>
    )
  }

  return (
    <NavLink
      to="/settings"
      data-testid="shell-settings"
      className={({ isActive }) =>
        [
          'tap flex items-center gap-2.5 rounded-md px-3 text-[14px] font-medium transition-colors',
          isActive ? 'bg-brand-50 text-brand-700' : 'text-ink-muted hover:bg-surface-soft hover:text-ink',
        ].join(' ')
      }
    >
      <Settings size={17} strokeWidth={2} />
      {t('nav.settings')}
    </NavLink>
  )
}

/** The bottom tab bar — the compact arrangement. */
function TabBar() {
  return (
    <nav
      aria-label={t('nav.label')}
      data-nav="tabbar"
      className="nav-tabbar safe-bottom shrink-0 border-t border-edge bg-surface"
    >
      {DESTINATIONS.map((destination) => (
        <DestinationLink key={destination.to} destination={destination} variant="tab" />
      ))}
      <span className="ml-1 border-l border-edge pl-1">
        <SettingsLink variant="bar" />
      </span>
    </nav>
  )
}

/** The top bar — the medium arrangement. */
function TopBar() {
  return (
    <header
      data-nav="topbar"
      className="nav-topbar safe-top sticky top-0 z-30 shrink-0 items-center gap-4 border-b border-edge bg-surface px-4 py-2"
    >
      <Brand />
      <nav aria-label={t('nav.label')} className="flex items-center gap-1">
        {DESTINATIONS.map((destination) => (
          <DestinationLink key={destination.to} destination={destination} variant="bar" />
        ))}
      </nav>
      <SettingsLink variant="bar" />
    </header>
  )
}

/** The left rail — the wide arrangement. */
function Rail() {
  return (
    <aside
      data-nav="rail"
      className="nav-rail w-[212px] shrink-0 overflow-y-auto border-r border-edge bg-surface px-3 py-5"
    >
      <div className="px-2 pb-4">
        <Brand />
      </div>
      <nav aria-label={t('nav.label')} className="space-y-0.5">
        {DESTINATIONS.map((destination) => (
          <DestinationLink key={destination.to} destination={destination} variant="rail" />
        ))}
      </nav>
      <div className="mt-4 border-t border-edge pt-3">
        <SettingsLink variant="rail" />
      </div>
    </aside>
  )
}

/** The shell every screen renders inside. */
export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="shell">
      <TopBar />
      <div className="shell__body">
        <Rail />
        <main className="shell__main">
          <div className="column py-4">{children}</div>
        </main>
      </div>
      <TabBar />
    </div>
  )
}
