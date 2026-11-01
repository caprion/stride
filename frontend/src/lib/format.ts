/**
 * Human rendering — the app's chrome.
 *
 * The domain's rendering belongs to the core: the rounding and the human
 * form of a duration live in `crates/stride-core/src/units.rs`, and this
 * app's copy of those rules is `src/lib/units.ts`. Contract §8 gives them
 * to the core, and says so twice.
 *
 * What is left here is the chrome: dates, the dense forms a table needs,
 * the sport's own name, and the em dash that stands for a number that is
 * not there. Every rounding in the app goes through `units.ts`, so the
 * phone and the desktop cannot disagree about a number.
 */

import { format, formatDistanceToNow, parseISO } from 'date-fns'
import {
  format_distance_km_short,
  format_duration,
  format_whole,
  round_half_even,
  truncate_heart_rate,
} from './units'
import type { VitalKey } from './types'

/** The mark for a number that is not there. */
const ABSENT = '—'

/** Metres -> "12.4 km", "5 km". */
export function formatDistance(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return ABSENT
  return `${format_distance_km_short(metres)} km`
}

/**
 * Seconds -> "6 h 10 m", "42 m".
 *
 * The core's `format_duration`: minutes are rounded to the nearest minute
 * and an exact zero minutes is left out — a person says "1 h", not
 * "1 h 00 m".
 */
export function formatDurationHuman(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return ABSENT
  return format_duration(seconds)
}

/**
 * Seconds per kilometre -> "5:24".
 *
 * The dense form, for a table or a chart. The number is rounded by the
 * core's rule (half to even) before the minute and the second are taken
 * from it.
 */
export function formatPace(secondsPerKm: number | null | undefined): string {
  if (secondsPerKm === null || secondsPerKm === undefined || secondsPerKm <= 0) return ABSENT
  const total = Math.trunc(round_half_even(secondsPerKm, 0))
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}

/** A number in its own scale, with no decimals: "63". */
export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined) return ABSENT
  return format_whole(value)
}

/**
 * A heart rate -> "148 bpm".
 *
 * A heart rate is truncated, never rounded (contract §3): 148.7 is 148.
 */
export function formatHeartRate(value: number | null | undefined): string {
  if (value === null || value === undefined) return ABSENT
  return `${truncate_heart_rate(value)} bpm`
}

/** Metres of climb -> "128 m". */
export function formatElevation(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return ABSENT
  return `${format_whole(metres)} m`
}

/**
 * The deviation from the usual value, in the metric's own scale.
 *
 * A duration carries its unit ("1 h 10 m"); a heart rate and an HRV do
 * not, because the unit is already on the value.
 */
export function formatDeviation(metric: VitalKey, deviation: number | null): string | null {
  if (deviation === null || deviation === undefined) return null
  const magnitude = Math.abs(deviation)
  if (metric === 'sleep') {
    return format_duration(magnitude)
  }
  return format_whole(magnitude)
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
