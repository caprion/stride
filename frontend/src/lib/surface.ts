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
  Award,
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
 * The phone's four destinations, one per job.
 *
 * * **Today** decides: should I run, and how much.
 * * **Runs** understands: what each run meant.
 * * **Progress** motivates: proof of getting fitter, consistency, bests.
 * * **Body** is recovery against the athlete's own normal.
 *
 * Form and Stats are still on the phone, one tap from Progress: they are
 * the deep end, not destinations.
 *
 * **Plan is not one of them.** The plan was meant to be the reason the app
 * exists, and the inferred plan has just made it non-empty for everybody —
 * but the athlete has used it and it does not work. The work stays: the
 * inferred plan, the verdict logic, the `not_yet` state. The tab comes
 * back when it reads well. **The desktop keeps Plan.**
 */
export const PHONE_DESTINATIONS: Destination[] = [
  { to: '/', label: 'Today', icon: LayoutDashboard, end: true },
  { to: '/feed', label: 'Runs', icon: Activity },
  { to: '/progress', label: 'Progress', icon: TrendingUp },
  { to: '/physiology', label: 'Body', icon: HeartPulse },
]

/**
 * The desktop's full set.
 *
 * The cut is the phone's, not the product's: the desktop has the room, and
 * these screens are good there.
 */
export const DESKTOP_DESTINATIONS: Destination[] = [
  { to: '/', label: 'Today', icon: LayoutDashboard, end: true },
  { to: '/progress', label: 'Progress', icon: Award },
  { to: '/plan', label: 'Plan', icon: ClipboardCheck },
  { to: '/feed', label: 'Activity feed', icon: Activity },
  { to: '/training', label: 'Training', icon: CalendarDays },
  { to: '/physiology', label: 'Body', icon: HeartPulse },
  { to: '/fitness', label: 'Running form', icon: Gauge },
  { to: '/compare', label: 'Compare', icon: BarChart3 },
  { to: '/stats', label: 'Statistics', icon: TrendingUp },
]

/**
 * The routes the phone does not have.
 *
 * * **Compare** — two runs side by side is a laptop activity.
 * * **Training calendar** — the Plan screen carries the schedule.
 * * **Plan** — the tab comes off the phone for v1; the work stays.
 * * **Connect** — the phone has a *native* screen for the Garmin key, so
 *   the desktop's web one is not for the phone.
 *
 * The Garmin forms, the network route and the history windows are not
 * routes: they are parts of the settings screen, and the phone's settings
 * screen does not carry them.
 */
export const PHONE_CUT_ROUTES = ['/compare', '/training', '/plan', '/connect']

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
