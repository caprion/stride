/**
 * The load's series, as the charts draw them.
 *
 * The three rules the charts rest on are here: **the days keep their
 * places**, **a day with no load is not a day with a load of zero**, and
 * **the scale always holds the zero**. They are the parts a reader can
 * be misled by, so they are the parts with tests.
 */

import { describe, expect, it } from 'vitest'
import {
  domainWithZero,
  labelIndices,
  loadDayBars,
  loadProjectionPoints,
  loadSeriesPoints,
} from '../src/lib/loadSeries'

/** A day of the series, as the router sends it. */
function day(date: string, load: number | null, reason: string | null = null) {
  return {
    date,
    label: `${date} 2026`,
    load: load === null ? null : { raw: load, text: String(load) },
    reason,
  }
}

/** A state of the series, as the router sends it. */
function state(date: string, acute: number, chronic: number, form: number) {
  const one = (value: number) => ({ raw: value, text: value.toFixed(1) })
  return {
    date,
    label: `${date} 2026`,
    acute: one(acute),
    chronic: one(chronic),
    form: one(form),
    ratio: { raw: 1, text: '1.00' },
  }
}

describe("the horizontal axis's labels", () => {
  it('thins them out, and keeps the ends', () => {
    const indices = labelIndices(78)

    expect(indices.length).toBeLessThanOrEqual(5)
    expect(indices[0]).toBe(0)
    expect(indices[indices.length - 1]).toBe(77)
    // They are in order and none of them is the same day twice.
    expect([...indices].sort((left, right) => left - right)).toEqual(indices)
  })

  it('keeps every label when there are only a few days', () => {
    expect(labelIndices(3)).toEqual([0, 1, 2])
    expect(labelIndices(1)).toEqual([0])
    expect(labelIndices(0)).toEqual([])
  })
})

describe("the chart's scale", () => {
  it('always holds the zero, so the form can be read against it', () => {
    expect(domainWithZero([19.7, 7, -12.7])).toEqual([-12.7, 19.7])
    // A form that has not gone below the zero still shows the zero.
    expect(domainWithZero([10, 20])).toEqual([0, 20])
  })

  it('has a scale when every value is the same', () => {
    const [low, high] = domainWithZero([0, 0, 0])

    expect(high).toBeGreaterThan(low)
  })

  it('leaves out a value that is not there', () => {
    expect(domainWithZero([null, 5, null])).toEqual([0, 5])
  })
})

describe("the load's series", () => {
  it('carries the three numbers and the core’s own words for them', () => {
    const points = loadSeriesPoints([state('2026-05-11', 18.2, 6.1, -12.1)])

    expect(points[0]).toMatchObject({
      acute: 18.2,
      chronic: 6.1,
      form: -12.1,
      acuteText: '18.2',
      chronicText: '6.1',
      formText: '-12.1',
    })
  })

  it('labels only some of the days, and the ends among them', () => {
    const states = Array.from({ length: 40 }, (_, index) =>
      state(`2026-05-${String(index + 1).padStart(2, '0')}`, 10, 5, -5),
    )
    const points = loadSeriesPoints(states)

    const labelled = points.filter((point) => point.label !== '')
    expect(labelled.length).toBeLessThanOrEqual(5)
    expect(points[0]?.label).not.toBe('')
    expect(points[points.length - 1]?.label).not.toBe('')
  })
})

describe("the days' bars", () => {
  it('lays the days on the calendar, so the gaps keep their places', () => {
    const bars = loadDayBars(
      [day('2026-05-01', 30), day('2026-05-05', 40)],
      '2026-05-01',
      '2026-05-07',
    )

    // One bar per calendar day, whether the record covers it or not.
    expect(bars).toHaveLength(7)
    expect(bars.map((bar) => bar.load)).toEqual([30, null, null, null, 40, null, null])
  })

  it('does not turn a day with no load into a day with a load of zero', () => {
    const bars = loadDayBars(
      [day('2026-05-01', 30), day('2026-05-02', null, 'nothing_weighable')],
      '2026-05-01',
      '2026-05-02',
    )

    expect(bars[0]?.load).toBe(30)
    // The absence of a value is not the value zero.
    expect(bars[1]?.load).toBeNull()
  })

  it('keeps a day whose load is zero as the value zero', () => {
    const bars = loadDayBars([day('2026-05-01', 0)], '2026-05-01', '2026-05-01')

    expect(bars[0]?.load).toBe(0)
  })

  it('has no bars when the window is not a window', () => {
    expect(loadDayBars([], 'not a date', '2026-05-01')).toHaveLength(0)
  })
})

describe("the scenario's projection", () => {
  it('carries one point per projected day, numbered from one', () => {
    const points = loadProjectionPoints([
      { day: 1, form: { raw: -10.2, text: '-10.2' }, ratio: { raw: 2.5, text: '2.50' } },
      { day: 2, form: { raw: -8.1, text: '-8.1' }, ratio: { raw: 2.22, text: '2.22' } },
    ])

    expect(points.map((point) => point.label)).toEqual(['1', '2'])
    expect(points[0]?.formText).toBe('-10.2')
  })
})
