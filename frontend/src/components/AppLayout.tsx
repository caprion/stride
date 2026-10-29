/**
 * App shell — top navigation, left rail and the routed content area.
 *
 * A slim white top bar with the primary destinations, a secondary left rail
 * with the athlete's own sections, and a dense content column in the middle.
 */

import { useQuery } from '@tanstack/react-query'
import {
  Activity,
  BarChart3,
  CalendarDays,
  ClipboardCheck,
  FlaskConical,
  Gauge,
  HeartPulse,
  LayoutDashboard,
  Settings as SettingsIcon,
  TrendingUp,
  type LucideIcon,
} from 'lucide-react'
import { Link, NavLink, useLocation } from 'react-router-dom'
import type { ReactNode } from 'react'
import { api } from '../lib/api'
import { formatDistanceShort } from '../lib/format'
import { DESKTOP_DESTINATIONS, PHONE_DESTINATIONS, isPhone } from '../lib/surface'
import { ConnectGarmin, GarminPill } from './GarminStatus'
import { cn } from './ui'

const PRIMARY_NAV = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard, end: true },
  { to: '/plan', label: 'Plan', icon: ClipboardCheck, end: false },
  { to: '/training', label: 'Training', icon: CalendarDays, end: false },
  { to: '/stats', label: 'Stats', icon: BarChart3, end: false },
]

const SECONDARY_NAV = [
  { to: '/plan', label: 'Plan vs actual', icon: ClipboardCheck },
  { to: '/feed', label: 'Activity feed', icon: Activity },
  { to: '/training', label: 'Training log', icon: CalendarDays },
  { to: '/stats', label: 'Statistics', icon: TrendingUp },
]

const ATHLETE_NAV = [
  { to: '/physiology', label: 'Physiology', icon: HeartPulse },
  { to: '/fitness', label: 'Running form', icon: Gauge },
]

function NavItem({
  to,
  label,
  icon: Icon,
  end,
}: {
  to: string
  label: string
  icon: LucideIcon
  end?: boolean
}) {
  return (
    <NavLink
      to={to}
      end={end}
      className={({ isActive }) =>
        cn(
          'flex items-center gap-2.5 rounded-md px-3 py-2 text-[14px] font-medium transition-colors',
          isActive
            ? 'bg-brand-50 text-brand-700'
            : 'text-ink-soft hover:bg-surface-soft hover:text-ink',
        )
      }
    >
      <Icon size={17} strokeWidth={2} />
      {label}
    </NavLink>
  )
}

export function AppLayout({ children }: { children: ReactNode }) {
  const location = useLocation()

  // The phone and the desktop are the same app with a different amount of
  // room. The phone shows five destinations; the desktop shows its full
  // set.
  const phone = isPhone()
  const destinations = phone ? PHONE_DESTINATIONS : DESKTOP_DESTINATIONS

  const { data: auth } = useQuery({
    queryKey: ['auth-status'],
    queryFn: api.authStatus,
    refetchInterval: 30_000,
  })

  const { data: summary } = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const athleteName =
    (auth?.athlete?.full_name as string | undefined) ||
    (auth?.athlete?.name as string | undefined) ||
    'Athlete'

  return (
    <div className="min-h-screen bg-surface-soft">
      {/* ------------------------------------------------------- top bar */}
      <header className="sticky top-0 z-40 border-b border-edge bg-surface shadow-nav">
        <div className="mx-auto flex h-14 max-w-[1400px] items-center gap-4 px-4">
          <Link to="/" className="flex items-center gap-2">
            <span className="flex h-7 w-7 items-center justify-center rounded-md bg-brand-700 text-accent-400">
              <Activity size={16} strokeWidth={2.6} />
            </span>
            <span className="text-[19px] font-extrabold uppercase tracking-tight text-brand-500">
              Stride
            </span>
          </Link>

          <nav className="ml-4 hidden items-center gap-1 md:flex">
            {phone
              ? null
              : PRIMARY_NAV.map((item) => (
                  <NavLink
                    key={item.to}
                    to={item.to}
                    end={item.end}
                    className={({ isActive }) =>
                      cn(
                        'rounded-md px-3 py-1.5 text-[14px] font-semibold transition-colors',
                        isActive ? 'text-brand-600' : 'text-ink-soft hover:text-ink',
                      )
                    }
                  >
                    {item.label}
                  </NavLink>
                ))}
          </nav>

          <div className="ml-auto flex items-center gap-2">
            <GarminPill />
            <Link to="/settings" aria-label="Settings">
              <span className="flex h-8 w-8 items-center justify-center rounded-full bg-surface-sunken text-ink-soft transition-colors hover:bg-edge">
                <SettingsIcon size={16} />
              </span>
            </Link>
            <AthleteAvatar name={athleteName} />
          </div>
        </div>
      </header>

      {auth?.demo_visible ? <DemoNotice /> : null}

      <div className="mx-auto flex max-w-[1400px] gap-6 px-4 py-6">
        {/* -------------------------------------------------- left rail */}
        <aside className={cn('w-[212px] shrink-0', phone ? 'hidden' : 'hidden lg:block')}>
          <div className="sticky top-20 space-y-5">
            <nav className="space-y-0.5">
              {SECONDARY_NAV.map((item) => (
                <NavItem key={item.to + item.label} {...item} />
              ))}
            </nav>

            <nav className="space-y-0.5">
              <p className="px-3 pb-1 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                Your athlete
              </p>
              {ATHLETE_NAV.map((item) => (
                <NavItem key={item.to + item.label} {...item} />
              ))}
            </nav>

            <nav className="space-y-0.5">
              <p className="px-3 pb-1 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                Account
              </p>
              <NavItem to="/settings" label="Settings" icon={SettingsIcon} />
            </nav>

            {summary ? (
              <div className="rounded-card border border-edge bg-surface p-4 shadow-card">
                <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  This week
                </p>
                <p className="mt-1 text-[24px] font-bold leading-none text-ink">
                  {formatDistanceShort(summary.this_week.distance_m)}
                  <span className="ml-1 text-[13px] font-medium text-ink-muted">km</span>
                </p>
                <p className="mt-1 text-[12px] text-ink-muted">
                  {summary.this_week.activity_count} run
                  {summary.this_week.activity_count === 1 ? '' : 's'}
                  {summary.streaks.current_weeks > 0
                    ? ` · ${summary.streaks.current_weeks}w streak`
                    : ''}
                </p>
                <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface-sunken">
                  <div
                    className="h-full rounded-full bg-brand-500"
                    style={{
                      width: `${Math.min(100, (summary.this_week.distance_m / 50000) * 100)}%`,
                    }}
                  />
                </div>
                <p className="mt-1.5 text-[11px] text-ink-faint">50 km weekly goal</p>
              </div>
            ) : null}

            {/*
              The connection state is said in one place — the indicator in
              the top bar. The rail does not carry a second opinion about
              it.
            */}
          </div>
        </aside>

        {/* --------------------------------------------------- content */}
        <main className="min-w-0 flex-1 pb-16" key={location.pathname}>
          <div className="animate-fade-up">{children}</div>
        </main>
      </div>

      {/* --------------------------------------------- mobile bottom bar */}
      {/*
        The phone's navigation: five destinations, always at the bottom —
        a phone turned sideways is still a phone. The desktop shows its
        full set in the same place.
      */}
      <nav
        className={cn(
          'fixed bottom-0 left-0 right-0 z-40 border-t border-edge bg-surface',
          phone ? 'flex' : 'hidden md:flex',
        )}
      >
        {destinations.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className={({ isActive }) =>
              cn(
                'flex flex-1 flex-col items-center gap-0.5 py-2 text-[11px] font-medium',
                isActive ? 'text-brand-600' : 'text-ink-muted',
              )
            }
          >
            <item.icon size={19} strokeWidth={2} />
            {item.label}
          </NavLink>
        ))}
      </nav>
    </div>
  )
}

function AthleteAvatar({ name }: { name: string }) {
  const initials = name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('')
  return (
    <span
      title={name}
      className="flex h-8 w-8 items-center justify-center rounded-full bg-ink text-[12px] font-bold text-white"
    >
      {initials || 'A'}
    </span>
  )
}

/** A slim notice so nobody mistakes the seeded demo set for their real data. */
function DemoNotice() {
  return (
    <div className="border-b border-amber-200 bg-amber-50">
      <div className="mx-auto flex max-w-[1400px] items-center gap-3 px-4 py-2">
        <FlaskConical size={15} className="shrink-0 text-warn" />
        <p className="min-w-0 flex-1 text-[13px] text-amber-900">
          You are browsing <strong>demo data</strong>. Connect your Garmin to sync your real
          runs.
        </p>
        <ConnectGarmin small className="shrink-0" />
      </div>
    </div>
  )
}
