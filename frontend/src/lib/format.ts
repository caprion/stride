/** Formatting helpers shared across every surface. */

/** 18330 -> "18.3 km" (Strava shows one decimal below 100 km). */
export function formatDistance(
  metres: number | null | undefined,
  unit: 'km' | 'mi' = 'km',
): string {
  if (metres === null || metres === undefined) return '—'
  if (unit === 'mi') {
    const miles = metres / 1609.344
    return `${miles.toFixed(miles < 100 ? 2 : 1)} mi`
  }
  const km = metres / 1000
  return `${km.toFixed(km < 100 ? 2 : 1)} km`
}

export function formatDistanceShort(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return '—'
  const km = metres / 1000
  return km.toFixed(km < 100 ? 1 : 0)
}

/** Seconds -> "1:23:45" or "23:45". */
export function formatDuration(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const total = Math.round(seconds)
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const secs = total % 60
  if (hours > 0)
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  return `${minutes}:${String(secs).padStart(2, '0')}`
}

/** Seconds -> "2h 13m" — for dense stat tiles. */
export function formatDurationLong(seconds: number | null | undefined): string {
  if (!seconds) return '—'
  const total = Math.round(seconds)
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  if (hours && minutes) return `${hours}h ${minutes}m`
  if (hours) return `${hours}h`
  return `${minutes}m`
}

/** Pace seconds-per-km -> "6:19 /km". */
export function formatPace(
  secondsPerKm: number | null | undefined,
  unit: 'km' | 'mi' = 'km',
): string {
  if (secondsPerKm === null || secondsPerKm === undefined || secondsPerKm <= 0) return '—'
  const value = unit === 'mi' ? secondsPerKm * 1.609344 : secondsPerKm
  const total = Math.round(value)
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}

export function formatPaceWithUnit(secondsPerKm: number | null | undefined): string {
  const pace = formatPace(secondsPerKm)
  return pace === '—' ? '—' : `${pace} /km`
}

/** Metres -> "255 m" or "1.2 km" */
export function formatElevation(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return '—'
  if (metres >= 1000) return `${(metres / 1000).toFixed(1)} km`
  return `${Math.round(metres)} m`
}

export function formatNumber(value: number | null | undefined, digits = 0): string {
  if (value === null || value === undefined) return '—'
  return value.toLocaleString(undefined, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  })
}

export function formatHr(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—'
  return `${Math.round(value)}`
}

/** ISO date -> "4 Jan 2026" */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—'
  const date = new Date(iso.length <= 10 ? `${iso}T00:00:00` : iso)
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })
}

/** ISO date -> "Sunday, 4 January" */
export function formatDateLong(iso: string | null | undefined): string {
  if (!iso) return '—'
  const date = new Date(iso.length <= 10 ? `${iso}T00:00:00` : iso)
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleDateString(undefined, {
    weekday: 'long',
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}

/** ISO date -> "4 Jan, 07:22" */
export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return '—'
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return '—'
  return `${date.toLocaleDateString(undefined, { day: 'numeric', month: 'short' })}, ${date.toLocaleTimeString(
    undefined,
    { hour: '2-digit', minute: '2-digit' },
  )}`
}

/** Relative time -> "2 hours ago" */
export function formatRelative(iso: string | null | undefined): string {
  if (!iso) return 'never'
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'never'
  const seconds = Math.round((Date.now() - date.getTime()) / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes} min ago`
  const hours = Math.round(minutes / 60)
  if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`
  const days = Math.round(hours / 24)
  if (days < 30) return `${days} day${days === 1 ? '' : 's'} ago`
  const months = Math.round(days / 30)
  if (months < 12) return `${months} month${months === 1 ? '' : 's'} ago`
  return `${Math.round(months / 12)} year${Math.round(months / 12) === 1 ? '' : 's'} ago`
}

/** "2026-W43" -> "W43 '26" */
export function formatWeekKey(key: string | null | undefined): string {
  if (!key) return '—'
  const match = key.match(/^(\d{4})-W(\d{1,2})$/)
  if (!match) return key
  return `W${Number(match[2])} '${match[1].slice(2)}`
}

/** "2026-01" -> "Jan 2026" */
export function formatMonthKey(key: string | null | undefined): string {
  if (!key) return '—'
  const date = new Date(`${key}-01T00:00:00`)
  if (Number.isNaN(date.getTime())) return key
  return date.toLocaleDateString(undefined, { month: 'short', year: 'numeric' })
}

/** Seconds-per-km -> "6:19" but keeping sub-minute precision readable. */
export function paceToMinutes(secondsPerKm: number | null | undefined): number | null {
  if (!secondsPerKm) return null
  return secondsPerKm / 60
}

export function sportLabel(sport: string): string {
  const map: Record<string, string> = {
    running: 'Run',
    trail_running: 'Trail Run',
    treadmill_running: 'Treadmill',
    virtualrun: 'Virtual Run',
    walking: 'Walk',
    hiking: 'Hike',
    cycling: 'Ride',
    swimming: 'Swim',
    strength_training: 'Strength',
  }
  return map[sport] ?? sport.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

export const SPORT_COLOR: Record<string, string> = {
  running: '#fc5200',
  trail_running: '#e05700',
  treadmill_running: '#8a4bff',
  virtualrun: '#2d9cdb',
  walking: '#1a9c5a',
  hiking: '#8b6f47',
  cycling: '#0f9bd7',
  swimming: '#00a3c4',
  strength_training: '#6d7176',
}
