/**
 * The pace page.
 *
 * Three panels: the training week by week, the pace at a heart rate, and
 * where the time went by pace.
 *
 * The page says what happened. There is no target, no "too slow" and no
 * verdict on it, and the window every number belongs to is named on the
 * screen.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { paceFixture } from '../src/fixtures'
import { paceCloud, trainingWeeks } from '../src/pages/PacePage'
import { calendarWeekStarts } from '../src/lib/chart'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/**
 * The bands, for the state in which the page is reviewed.
 *
 * **The values are invented**, like every other value in the fixture set:
 * the seeded store holds no FIT file, so the router's own answer for the
 * window carries no bands at all.
 */
const BANDS = {
  width_s_per_km: 30,
  total: { text: '3 h 12 m', raw: 11520 },
  distance: { text: '18 km', raw: 18000 },
  rows: [
    {
      from: 240,
      to: 270,
      label: '4:00–4:30 /km',
      time: { text: '12 m', raw: 720 },
      distance: { text: '3 km', raw: 3000 },
      share: { text: '6.25%', raw: 6.25 },
    },
    {
      from: 270,
      to: 300,
      label: '4:30–5:00 /km',
      time: { text: '3 h', raw: 10800 },
      distance: { text: '15 km', raw: 15000 },
      share: { text: '93.75%', raw: 93.75 },
    },
  ],
}

describe('the pace page', () => {
  it('renders the three panels', async () => {
    renderApp('/pace')

    expect(await screen.findByTestId('pace-weeks')).toBeTruthy()
    expect(await screen.findByTestId('pace-pace-at-hr')).toBeTruthy()
    expect(await screen.findByTestId('pace-bands')).toBeTruthy()
  })

  it('names the window every number belongs to', async () => {
    renderApp('/pace')
    const window = await screen.findByTestId('pace-window')

    expect(window.textContent).toContain('23 Feb 2026')
    expect(window.textContent).toContain('13 May 2026')
  })

  it('draws the long run inside the week’s bar', () => {
    const weeks = trainingWeeks(
      paceFixture.weeks,
      calendarWeekStarts(paceFixture.date, 12),
    )
    const week = weeks.find((entry) => entry.km === 52.1)

    expect(week).toBeTruthy()
    // The long run is a part of the week's bar, never beside it.
    expect(week!.longestKm).toBe(16.1)
    expect(week!.restKm).toBeCloseTo(52.1 - 16.1, 5)
    expect(week!.longestKm! + week!.restKm!).toBeCloseTo(week!.km!, 5)
  })

  it('leaves a week with no run as a gap, not a bar of zero', () => {
    const weeks = trainingWeeks(
      paceFixture.weeks,
      calendarWeekStarts(paceFixture.date, 12),
    )
    const gap = weeks.find((entry) => entry.label === 'Mon 23 Feb')

    expect(gap).toBeTruthy()
    expect(gap!.km).toBeNull()
    expect(gap!.longestKm).toBeNull()
  })

  it('moves the pace-at-heart-rate cloud with the window', async () => {
    // The window's own weeks: two of them, and both carry a pace and a
    // heart rate.
    const wide = paceCloud(paceFixture.weeks)
    expect(wide.length).toBe(2)

    vi.spyOn(api, 'insightsPace').mockResolvedValue({
      ...paceFixture,
      weeks: [paceFixture.weeks[1]!],
    })
    const { container } = renderApp('/pace')
    await screen.findByTestId('pace-pace-at-hr')

    const cloud = container.querySelector('[data-points]')
    expect(cloud?.getAttribute('data-points')).toBe('1')
  })

  it('says why there are no bands when the window holds no samples', async () => {
    renderApp('/pace')
    const bands = await screen.findByTestId('pace-bands')

    // The reason is the core's own words, and no band of zero is drawn.
    expect(
      bands.querySelector('[data-testid="pace-bands-empty"]')?.textContent,
    ).toContain('no moving samples')
    expect(bands.querySelector('[data-testid="pace-bands-rows"]')).toBeNull()
  })

  it('draws the bands the answer carries', async () => {
    vi.spyOn(api, 'insightsPace').mockResolvedValue({
      ...paceFixture,
      bands: BANDS,
      unavailable: null,
    })
    renderApp('/pace')
    const rows = await screen.findByTestId('pace-bands-rows')

    // The labels are the core's own words, and the bar is the share.
    expect(rows.textContent).toContain('4:00–4:30 /km')
    expect(rows.textContent).toContain('4:30–5:00 /km')
    expect(rows.textContent).toContain('3 h')
    const bar = rows.querySelector('[data-testid="pace-bands-bar-270"]')
    expect(bar?.getAttribute('style')).toContain('93.75%')
  })
})

describe('the pace page on Progress', () => {
  it('is reached from the Progress screen', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-pace-link')

    expect(link.getAttribute('href')).toBe('/pace')
  })
})
