/**
 * Which surface the app is on, and what that surface shows.
 *
 * The phone and the desktop are the same app with a different amount of
 * room. The phone has five destinations; the desktop keeps all of them.
 *
 * `isNativeApp()` tells the two apart: Capacitor puts `window.Capacitor`
 * into the phone's WebView and not into a browser. That is the only signal
 * — there is no build flag, so one bundle serves both.
 */

import {
  Activity,
  BarChart3,
  CalendarDays,
  ClipboardCheck,
  Gauge,
  HeartPulse,
  LayoutDashboard,
  TrendingUp,
  type LucideIcon,
} from 'lucide-react'
import { isNativeApp } from './api'

/** Is this the phone? */
export function isPhone(): boolean {
  return isNativeApp()
}

/** One destination in the navigation. */
export interface Destination {
  to: string
  label: string
  icon: LucideIcon
  end?: boolean
}

/**
 * The phone's six destinations.
 *
 * What the phone keeps: what to do now, the plan against what was done, the
 * activity list, the body, the running fingerprint, and the statistics — the
 * personal records and the year in one screen, which is the thing an
 * athlete looks at after a good month.
 *
 * What it cuts is a laptop activity: two runs side by side, and the
 * calendar, which the Plan screen already carries.
 */
export const PHONE_DESTINATIONS: Destination[] = [
  { to: '/', label: 'Today', icon: LayoutDashboard, end: true },
  { to: '/plan', label: 'Plan', icon: ClipboardCheck },
  { to: '/feed', label: 'Runs', icon: Activity },
  { to: '/physiology', label: 'Body', icon: HeartPulse },
  { to: '/fitness', label: 'Form', icon: Gauge },
  { to: '/stats', label: 'Stats', icon: BarChart3 },
]

/**
 * The desktop's full set.
 *
 * The cut is the phone's, not the product's: the desktop has the room, and
 * these screens are good there.
 */
export const DESKTOP_DESTINATIONS: Destination[] = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard, end: true },
  { to: '/plan', label: 'Plan', icon: ClipboardCheck },
  { to: '/feed', label: 'Activity feed', icon: Activity },
  { to: '/training', label: 'Training', icon: CalendarDays },
  { to: '/physiology', label: 'Physiology', icon: HeartPulse },
  { to: '/fitness', label: 'Running form', icon: Gauge },
  { to: '/compare', label: 'Compare', icon: BarChart3 },
  { to: '/stats', label: 'Statistics', icon: TrendingUp },
]

/**
 * The routes the phone does not have.
 *
 * * **Compare** — two runs side by side is a laptop activity.
 * * **Training calendar** — the Plan screen carries the schedule.
 *
 * The Garmin forms, the network route and the history windows are not
 * routes: they are parts of the settings screen, and the phone's settings
 * screen does not carry them.
 */
export const PHONE_CUT_ROUTES = ['/compare', '/training']

/** Is this path one the phone does not have? */
export function isCutOnPhone(pathname: string): boolean {
  return PHONE_CUT_ROUTES.some(
    (route) => pathname === route || pathname.startsWith(`${route}/`),
  )
}

/**
 * The one line the app says about where the data lives.
 *
 * One line, and then it stops. The athlete does not need to be persuaded;
 * they need to know where their data is.
 */
export function localDataLine(): string {
  return isPhone() ? 'Your data stays on this phone.' : 'Your data stays on this machine.'
}
