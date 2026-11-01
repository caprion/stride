/**
 * Human rendering.
 *
 * The core contract (§5) is the rulebook: values are rounded to what a
 * human would say, never to what a machine would. "6 h 10 m", not "6.166 h";
 * "14 below your usual", not "−13.7%".
 */

import { format, formatDistanceToNow, parseISO } from 'date-fns'
import type { VitalKey } from './types'

/** Metres -> "12.4 km". */
export function formatDistance(metres: number | null | undefined, decimals = 1): string {
  if (metres === null || metres === undefined) return '—'
  return `${(metres / 1000).toFixed(decimals)} km`
}

/**
 * Seconds -> "6 h 10 m", "42 m".
 *
 * Minutes are rounded to the nearest minute; hours are written with a
 * space, because "6h10m" is a code, not a sentence.
 */
export function formatDurationHuman(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const totalMinutes = Math.round(seconds / 60)
  if (totalMinutes < 1) return '0 m'
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  if (hours > 0 && minutes > 0) return `${hours} h ${minutes} m`
  if (hours > 0) return `${hours} h`
  return `${minutes} m`
}

/** Seconds -> "1:23:45" or "23:45" — the dense form, for tables. */
export function formatDurationClock(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const total = Math.round(seconds)
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const secs = total % 60
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${minutes}:${String(secs).padStart(2, '0')}`
}

/** Seconds per kilometre -> "5:24". */
export function formatPace(secondsPerKm: number | null | undefined): string {
  if (secondsPerKm === null || secondsPerKm === undefined || secondsPerKm <= 0) return '—'
  const total = Math.round(secondsPerKm)
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}

/** A number -> "63". */
export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—'
  return String(Math.round(value))
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
    return formatDurationHuman(magnitude)
  }
  return String(Math.round(magnitude))
}

/** Metres of climb -> "128 m". */
export function formatElevation(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return '—'
  return `${Math.round(metres)} m`
}

/** An ISO date -> "Sat 10 May". */
export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return '—'
  const date = parseISO(iso)
  if (Number.isNaN(date.getTime())) return '—'
  return format(date, 'EEE d MMM')
}

/** An ISO date -> "10 May 2026". */
export function formatDateMedium(iso: string | null | undefined): string {
  if (!iso) return '—'
  const date = parseISO(iso)
  if (Number.isNaN(date.getTime())) return '—'
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
