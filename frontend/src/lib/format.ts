/**
 * Human rendering — the app's chrome.
 *
 * **The domain's rendering belongs to the core.** The rounding and the
 * human form of a number live in `crates/stride-core/src/units.rs`, and
 * contract §8 gives them to the core: "the frontend renders app chrome
 * only. It does not re-implement the domain's wording, and it does not
 * re-implement the domain's rounding."
 *
 * So this file does not render a domain value at all. The distance, the
 * duration, the heart rate and the pace arrive **already rendered** — the
 * API sends the core's words beside the numbers (see `src/lib/types.ts`),
 * and the app shows them as they stand.
 *
 * What is left here is the chrome: the dates, the dense forms a table
 * needs, the sport's own name, and the em dash that stands for a value
 * that is not there.
 */

import { format, formatDistanceToNow, parseISO } from 'date-fns'
import type { RenderedValue } from './types'

/** The mark for a number that is not there. */
export const ABSENT = '—'

/**
 * A rendered value, as the athlete reads it.
 *
 * The core rendered it; the app shows it. A value that is not there shows
 * the em dash — never "null", never "0" (contract §8: a missing value
 * crosses as null, never as "0").
 */
export function renderedText(value: RenderedValue | null | undefined): string {
  return value?.text ?? ABSENT
}

/** An ISO date -> "Sat 10 May". */
export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return ABSENT
  const date = parseISO(iso)
  if (Number.isNaN(date.getTime())) return ABSENT
  return format(date, 'EEE d MMM')
}

/** An ISO date -> "10 May 2026". */
export function formatDateMedium(iso: string | null | undefined): string {
  if (!iso) return ABSENT
  const date = parseISO(iso)
  if (Number.isNaN(date.getTime())) return ABSENT
  return format(date, 'd MMM yyyy')
}

/** An ISO timestamp -> "2 days ago". */
export function formatRelative(iso: string | null | undefined): string {
  if (!iso) return 'never'
  const date = parseISO(iso)
  if (Number.isNaN(date.getTime())) return 'never'
  return formatDistanceToNow(date, { addSuffix: true })
}

/**
 * A count -> "8".
 *
 * A count is a count: it is already a whole number, so there is nothing
 * to round and nothing to render.
 */
export function formatCount(count: number | null | undefined): string {
  if (count === null || count === undefined) return ABSENT
  return String(count)
}

/** The plan's own vocabulary for a session type, read as the plan reads it. */
export function sessionTypeLabel(sessionType: string | null | undefined): string {
  if (!sessionType) return 'Unknown'
  if (sessionType === 'unknown') return 'Unknown'
  return sessionType.replace(/_/g, ' ')
}

/** The sport's own name, read as the athlete reads it. */
export function sportLabel(sport: string | null | undefined): string {
  if (!sport) return 'Run'
  switch (sport) {
    case 'running':
      return 'Run'
    case 'trail_running':
      return 'Trail run'
    case 'treadmill_running':
      return 'Treadmill run'
    case 'virtualrun':
      return 'Virtual run'
    case 'walking':
      return 'Walk'
    case 'hiking':
      return 'Hike'
    case 'cycling':
      return 'Ride'
    case 'swimming':
      return 'Swim'
    case 'strength_training':
      return 'Strength'
    default:
      return sport.replace(/_/g, ' ')
  }
}
