/**
 * The band, the direction and the distance comparison.
 *
 * These are the rules the contract freezes, and the numbers on the screen
 * depend on them.
 */

import { describe, expect, it } from 'vitest'
import {
  computeBaseline,
  directionFor,
  distanceComparison,
  mad,
  median,
  travelFrom,
} from '../src/lib/direction'
import { formatDeviation, formatDurationHuman } from '../src/lib/format'

describe('the band', () => {
  it('is the median of the nights, and the median absolute deviation around it', () => {
    expect(median([3, 1, 2])).toBe(2)
    expect(mad([1, 2, 3], 2)).toBe(1)
  })

  it('is median ± 1.5 × MAD over the last 30 nights', () => {
    const baseline = computeBaseline([10, 10, 10, 20, 30])
    expect(baseline?.median).toBe(10)
    expect(baseline?.mad).toBe(0)
    const spread = computeBaseline([10, 10, 10, 20])
    expect(spread?.lower).toBe(10 - 1.5 * 0)
  })

  it('has no band when there is no history', () => {
    expect(computeBaseline([])).toBeNull()
  })
})

describe('the direction word', () => {
  const band = { median: 60, mad: 4, lower: 54, upper: 66, nights: 30 }

  it('reads "about usual" inside the band', () => {
    expect(directionFor('hrv', 60, band)).toBe('about usual')
    expect(directionFor('resting_hr', 63, band)).toBe('about usual')
  })

  it('reads "could be better" when a high-is-better measure falls below', () => {
    expect(directionFor('hrv', 48, band)).toBe('could be better')
    const sleepBand = { median: 26_400, mad: 1_800, lower: 23_700, upper: 29_100, nights: 30 }
    expect(directionFor('sleep', 22_200, sleepBand)).toBe('could be better')
  })

  it('reads "could be better" when a low-is-better measure rises above', () => {
    expect(directionFor('resting_hr', 72, band)).toBe('could be better')
  })

  it('reads "better than usual" on the good side', () => {
    expect(directionFor('hrv', 70, band)).toBe('better than usual')
    expect(directionFor('resting_hr', 50, band)).toBe('better than usual')
  })

  it('never leaves the closed set', () => {
    const words = new Set(['better than usual', 'about usual', 'could be better'])
    for (const value of [10, 48, 60, 72, 90]) {
      expect(words.has(directionFor('hrv', value, band))).toBe(true)
    }
  })
})

describe('the direction of travel', () => {
  it('reads "steady" when the last two nights are the same', () => {
    expect(travelFrom([10, 12, 12]).travel).toBe('steady')
  })

  it('counts the nights in a run', () => {
    expect(travelFrom([10, 9, 8, 7])).toEqual({ travel: 'down', days: 3 })
    expect(travelFrom([7, 8, 9, 10])).toEqual({ travel: 'up', days: 3 })
  })
})

describe('the distance comparison', () => {
  it('appears when the gap is beyond 10%', () => {
    const comparison = distanceComparison(12_400, 10_000)
    expect(comparison?.gap_m).toBe(2_400)
    expect(comparison?.over).toBe(true)
  })

  it('is absent within 10%', () => {
    expect(distanceComparison(10_200, 10_000)).toBeNull()
    expect(distanceComparison(9_800, 10_000)).toBeNull()
  })

  it('is symmetric for under', () => {
    const comparison = distanceComparison(8_400, 10_000)
    expect(comparison?.over).toBe(false)
    expect(comparison?.gap_m).toBe(1_600)
  })

  it('needs a target', () => {
    expect(distanceComparison(12_400, null)).toBeNull()
  })
})

describe('the human rendering', () => {
  it('rounds a duration the way a human says it', () => {
    expect(formatDurationHuman(22_200)).toBe('6 h 10 m')
    expect(formatDurationHuman(2_520)).toBe('42 m')
    expect(formatDurationHuman(3_600)).toBe('1 h')
  })

  it('rounds a deviation to the metric’s own scale', () => {
    expect(formatDeviation('sleep', -4_200)).toBe('1 h 10 m')
    expect(formatDeviation('hrv', -14)).toBe('14')
    expect(formatDeviation('resting_hr', 5)).toBe('5')
  })
})
