/**
 * The direction band and the human comparison.
 *
 * The core owns the numbers; these are the rules the contract freezes
 * (`docs/contract/core-contract.md` §4 and §5), written down once so the
 * app, the fixtures and the tests cannot disagree about them.
 *
 *   usual  = the median of the last 30 nights, the day itself excluded
 *   band   = median ± 1.5 × MAD
 *   inside the band → "about usual"
 *   outside        → "better than usual" / "could be better"
 *
 * Which side is the better side is a fact about the measurement: more
 * sleep is better, higher HRV is better, lower resting HR is better.
 */

import type { Direction, Travel, VitalBaseline, VitalKey } from './types'

/** How many nights the band is measured over. */
export const BAND_NIGHTS = 30

/** The band multiplier: median ± 1.5 × MAD. */
export const BAND_MAD_FACTOR = 1.5

/** The distance comparison only appears beyond a 10% gap. */
export const DISTANCE_GAP_THRESHOLD = 0.1

/** The median of a list of numbers. */
export function median(values: number[]): number {
  if (values.length === 0) return Number.NaN
  const sorted = [...values].sort((a, b) => a - b)
  const middle = Math.floor(sorted.length / 2)
  if (sorted.length % 2 === 1) return sorted[middle] as number
  return ((sorted[middle - 1] as number) + (sorted[middle] as number)) / 2
}

/** The median absolute deviation. */
export function mad(values: number[], center: number): number {
  if (values.length === 0) return Number.NaN
  return median(values.map((value) => Math.abs(value - center)))
}

/**
 * The band for one metric.
 *
 * `values` are the nights, oldest first, **excluding** the day itself.
 */
export function computeBaseline(values: number[]): VitalBaseline | null {
  const window = values.filter((value) => Number.isFinite(value)).slice(-BAND_NIGHTS)
  if (window.length === 0) return null
  const center = median(window)
  const spread = mad(window, center)
  return {
    median: center,
    mad: spread,
    lower: center - BAND_MAD_FACTOR * spread,
    upper: center + BAND_MAD_FACTOR * spread,
    nights: window.length,
  }
}

/** Which side of the band is the better side, per metric. */
function betterIsHigher(metric: VitalKey): boolean {
  return metric !== 'resting_hr'
}

/** The direction word for a value against its band. */
export function directionFor(
  metric: VitalKey,
  value: number | null,
  baseline: VitalBaseline | null,
): Direction {
  if (value === null || baseline === null) return 'about usual'
  if (value > baseline.upper)
    return betterIsHigher(metric) ? 'better than usual' : 'could be better'
  if (value < baseline.lower)
    return betterIsHigher(metric) ? 'could be better' : 'better than usual'
  return 'about usual'
}

/**
 * How a vital has moved over the last few days.
 *
 * The travel is descriptive: "3rd night down", "up 3 nights", "steady".
 * It never concludes anything.
 */
export function travelFrom(values: number[]): { travel: Travel; days: number | null } {
  const recent = values.filter((value) => Number.isFinite(value)).slice(-4)
  if (recent.length < 2) return { travel: 'steady', days: null }
  const last = recent[recent.length - 1] as number
  const previous = recent[recent.length - 2] as number
  if (last === previous) return { travel: 'steady', days: null }

  const direction = last > previous ? 1 : -1
  let days = 1
  for (let index = recent.length - 2; index > 0; index -= 1) {
    const current = recent[index] as number
    const earlier = recent[index - 1] as number
    if (current === earlier) break
    if (current > earlier !== direction > 0) break
    days += 1
  }
  return { travel: direction > 0 ? 'up' : 'down', days }
}

/**
 * The distance comparison.
 *
 * It appears **only** when the gap from the target exceeds 10%, and it is
 * stated neutrally. Within 10% it is absent: a 2% difference is not news,
 * and neither more nor less is good or bad.
 */
export interface DistanceComparison {
  /** The distance that was run. */
  actual_m: number
  /** The plan's target. */
  target_m: number
  /** The size of the gap, always positive. */
  gap_m: number
  /** Whether the run was longer than the target. */
  over: boolean
}

/** The comparison, or `null` when the gap is within 10%. */
export function distanceComparison(
  actual_m: number | null | undefined,
  target_m: number | null | undefined,
): DistanceComparison | null {
  if (actual_m === null || actual_m === undefined) return null
  if (target_m === null || target_m === undefined || target_m <= 0) return null
  const gap = Math.abs(actual_m - target_m)
  if (gap <= DISTANCE_GAP_THRESHOLD * target_m) return null
  return { actual_m, target_m, gap_m: gap, over: actual_m > target_m }
}
