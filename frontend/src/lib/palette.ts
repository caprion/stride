/**
 * The palette, in one place.
 *
 * Two families:
 *
 * - **Brand (deep azure)** — structure, gravity, primary actions, chrome.
 * - **Accent (warm amber)** — excitement, energy, the athlete's own data.
 *
 * A third, natural family (teal, leaf) carries the semantic series — heart
 * rate zones, good/neutral/bad — so a chart reads the way a landscape does:
 * water, growth, sun, heat.
 */

export const BRAND = {
  50: '#EEF5FB',
  100: '#D6E6F3',
  200: '#A9C9E2',
  300: '#7BA8CD',
  400: '#4A82B4',
  500: '#1E5E96',
  600: '#124A7C',
  700: '#0D3A63',
  800: '#082B49',
  900: '#051D33',
} as const

export const ACCENT = {
  50: '#FEF8EA',
  100: '#FDEDC7',
  200: '#FBDB93',
  300: '#F8C55C',
  400: '#F2B01E',
  500: '#D99A0B',
  600: '#B57C06',
  700: '#8A5E05',
  800: '#5F4103',
  900: '#3D2A02',
} as const

/** Chart ramp: sun through earth to deep water. */
/** Chart furniture: grid lines and the two muted text tones. */
export const GRID = '#E4EBF0'
export const INK_MUTED = '#61788D'
export const INK_FAINT = '#93A6B7'
export const SURFACE_SUNKEN = '#E9EFF4'

/** The data colours, named rather than indexed. */
export const ACCENT_DATA = ACCENT[400]
export const ACCENT_WARM = ACCENT[500]
export const ACCENT_DEEP = ACCENT[600]
export const ACCENT_SOFT = ACCENT[50]

export const RAMP = ['#F2B01E', '#E08A3C', '#B4557F', '#6E6BA8', '#3F6E9E', '#1E5E96'] as const

/** Semantic series: heart rate zones, recovery through anaerobic. */
export const ZONE_COLORS = ['#A8BCCB', '#2E8B7A', '#6E9E3C', '#D99A0B', '#C0392B'] as const

/**
 * The metric colours.
 *
 * A running app measures half a dozen things at once, and every one of them
 * should look the same everywhere it appears. Heart rate is the same red on
 * the activity page, in the comparison and on the physiology trend — and it
 * is never the same colour as pace.
 *
 * The hues are one family: similar chroma, similar weight, so a screen with
 * five of them reads as one palette rather than five. Amber is reserved for
 * **effort** — the volume you spent, the bar that is yours — so the accent
 * colour keeps one meaning.
 *
 * Pace is the azure of the brand, because pace is the spine of running and
 * everything else is measured against it.
 */
export const METRIC = {
  /** Pace — the spine. Every other metric is measured against it. */
  pace: '#1E5E96',
  /** Heart rate — heat. */
  heartRate: '#D64A4A',
  /** Elevation — terrain. */
  elevation: '#6E6BA8',
  /** Cadence — rhythm. */
  cadence: '#2E8B7A',
  /** Drift and decoupling — the gap between expected and actual. */
  drift: '#B4557F',
  /** Effort — what it cost. The one amber in the app. */
  effort: '#F2B01E',
} as const

/** Data series that need to hold their own on a white card. */
export const SERIES = {
  /** The paler tint behind the most recent bar. */
  actualSoft: ACCENT[200],
  /** A cool grey for planned volume and de-emphasised furniture. */
  plannedSoft: '#C7D6E3',
  heartRate: METRIC.heartRate,
  pace: METRIC.pace,
  cadence: METRIC.cadence,
  elevation: METRIC.elevation,
  drift: METRIC.drift,
  planned: '#C7D6E3',
  actual: METRIC.effort,
  accent: METRIC.effort,
} as const

/** Verdict colours, shared by the plan surface and the badge kit. */
export const VERDICT_COLORS = {
  hit: '#2E8B7A',
  missed_pace: '#D99A0B',
  short: '#D99A0B',
  missed: '#C0392B',
  substituted: '#3F8FB8',
  bonus: '#F2B01E',
} as const

/** The sport palette, keyed the same way as `SPORT_COLOR`. */
export const SPORT_COLORS: Record<string, string> = {
  running: '#1E5E96',
  trail_running: '#2E8B7A',
  treadmill_running: '#6E6BA8',
  virtualrun: '#3F8FB8',
  walking: '#6E9E3C',
  hiking: '#8A7048',
  cycling: '#2E8B7A',
  swimming: '#3F8FB8',
  strength_training: '#7E8FA0',
}
