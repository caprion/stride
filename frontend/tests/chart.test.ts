/**
 * The chart's axis.
 *
 * The core renders the domain's numbers; the chart draws them. So an
 * axis's labels are the core's own words — "148 bpm", "5:24 /km" — and
 * never a bare number the app printed on its own.
 */

import { describe, expect, it } from 'vitest'
import {
  axisLabels,
  axisTicks,
  calendarWeekStarts,
  polygonPath,
  radialPoint,
  scaleToLargest,
} from '../src/lib/chart'

describe('calendar chart windows', () => {
  it('uses consecutive calendar weeks even across month and year boundaries', () => {
    expect(calendarWeekStarts('2026-01-06T06:30:00Z', 4)).toEqual([
      '2025-12-15',
      '2025-12-22',
      '2025-12-29',
      '2026-01-05',
    ])
    expect(calendarWeekStarts('2026-05-17', 1)).toEqual(['2026-05-11'])
  })
})

describe("a chart's axis", () => {
  it("labels every tick with the core's own words", () => {
    const ticks = axisTicks([
      { value: 148, text: '148 bpm' },
      { value: 152, text: '152 bpm' },
      { value: 163, text: '163 bpm' },
    ])

    expect(ticks.map((tick) => tick.text)).toEqual(['148 bpm', '152 bpm', '163 bpm'])
    // Every label carries its unit.
    for (const tick of ticks) expect(tick.text).toMatch(/bpm$/)
  })

  it('draws at most a few ticks, and the ends are always there', () => {
    const points = Array.from({ length: 20 }, (_, index) => ({
      value: index * 10,
      text: `${index * 10} bpm`,
    }))

    const ticks = axisTicks(points, 4)

    expect(ticks.length).toBe(4)
    expect(ticks[0]?.value).toBe(0)
    expect(ticks[3]?.value).toBe(190)
  })

  it('leaves out the values that are not there', () => {
    const ticks = axisTicks([
      { value: null, text: null },
      { value: 148, text: '148 bpm' },
      { value: null, text: '—' },
    ])

    expect(ticks).toHaveLength(1)
    expect(ticks[0]?.text).toBe('148 bpm')
  })

  it("finds a label by the chart's own number", () => {
    const labels = axisLabels([
      { value: 324, text: '5:24 /km' },
      { value: 306, text: '5:06 /km' },
    ])

    expect(labels.get(324)).toBe('5:24 /km')
    expect(labels.get(999)).toBeUndefined()
  })
})

/**
 * The radar's scale and its path.
 *
 * The scale puts the **larger of two values** on the outside — the
 * athlete's own scale, not an anchor somebody else chose — and the path
 * is the browser's own path syntax.
 */
describe("the radar's scale", () => {
  it('puts the larger of the two values on the outside', () => {
    const scale = scaleToLargest([30, 60])

    expect(scale.max).toBe(60)
    expect(scale.at(60)).toBe(1)
    expect(scale.at(30)).toBeCloseTo(0.5)
  })

  it('leaves a value that is not there at the centre', () => {
    const scale = scaleToLargest([null, 40, undefined])

    expect(scale.max).toBe(40)
    expect(scale.at(0)).toBe(0)
  })

  it('has no scale when there is no value at all', () => {
    const scale = scaleToLargest([null, null])

    expect(scale.max).toBe(0)
    expect(scale.at(10)).toBe(0)
  })
})

describe("the radar's path", () => {
  it('is a closed path of the points', () => {
    const path = polygonPath([
      { x: 0, y: 0 },
      { x: 10, y: 0 },
      { x: 10, y: 10 },
    ])

    expect(path).toBe('M0,0 L10,0 L10,10 Z')
  })

  it('is nothing when there are no points', () => {
    expect(polygonPath([])).toBe('')
  })
})

describe("the radar's axes", () => {
  it('start at the top and go round clockwise', () => {
    const top = radialPoint(100, 100, 50, 0, 6)

    expect(top.x).toBeCloseTo(100)
    expect(top.y).toBeCloseTo(50)

    const second = radialPoint(100, 100, 50, 1, 6)
    expect(second.x).toBeGreaterThan(100)
    expect(second.y).toBeLessThan(100)
  })
})
