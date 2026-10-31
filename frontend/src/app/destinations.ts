/**
 * The four destinations.
 *
 * This list is the information architecture, and it is the *whole* of it:
 * the same four at every width, in the same order. Compare and the Plan
 * week are reached contextually — from a run, and from Today — so they are
 * routes, not destinations.
 *
 * The labels come from the string catalogue, like every other
 * user-facing string.
 */

import {
  Activity,
  HeartPulse,
  LayoutDashboard,
  TrendingUp,
  type LucideIcon,
} from 'lucide-react'
import type { CatalogueKey } from '../i18n'

export interface Destination {
  /** The route. */
  to: string
  /** The catalogue key that names it. */
  labelKey: CatalogueKey
  /** The icon beside it. */
  icon: LucideIcon
  /** Whether the route is a prefix match. */
  end: boolean
}

export const DESTINATIONS: readonly Destination[] = [
  { to: '/', labelKey: 'nav.today', icon: LayoutDashboard, end: true },
  { to: '/runs', labelKey: 'nav.runs', icon: Activity, end: false },
  { to: '/progress', labelKey: 'nav.progress', icon: TrendingUp, end: false },
  { to: '/body', labelKey: 'nav.body', icon: HeartPulse, end: false },
] as const

/** The destination routes, in order — the set the viewport gate compares. */
export const DESTINATION_ROUTES = DESTINATIONS.map((destination) => destination.to)
