/**
 * The year, as a film.
 *
 * The months render with their own facts, the calendar colours the days
 * by the zones they were run in, and the diff says what changed in the
 * core's own sentences.
 *
 * And two things the surface must never say: **where** the athlete ran,
 * and **whether** the year was any good.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { yearFixture } from '../src/fixtures'
import { renderApp } from './helpers'
import type { YearInsight } from '../src/lib/types'

afterEach(() => vi.restoreAllMocks())

/** An answer with three days: a hard one, an easy one and a day with no run. */
function threeDays(): YearInsight {
  return {
    ...yearFixture,
    days: [
      {
        date: '2026-05-10',
        sessions: 1,
        distance: { text: '10 km', raw: 10_000 },
        high: { text: '62.50%', raw: 62.5 },
      },
      {
        date: '2026-05-11',
        sessions: 1,
        distance: { text: '8 km', raw: 8_000 },
        high: { text: '4.00%', raw: 4 },
      },
      { date: '2026-05-12', sessions: 0, distance: null, high: null },
    ],
  }
}

describe('the year', () => {
  it('renders the twelve months with their own facts', async () => {
    renderApp('/year')
    await screen.findByTestId('year-months')

    const months = await screen.findAllByTestId('year-month')
    expect(months).toHaveLength(12)

    // The core's own rendering, not a number the app formatted.
    expect(screen.getByTestId('year-month-2026-05-distance').textContent).toBe('58.4 km')
    expect(screen.getByTestId('year-month-2026-05-runs').textContent).toBe('6')
    expect(screen.getByTestId('year-month-2026-05-longest').textContent).toBe('16.1 km')
  })

  it('colours the days by the share of the high zones', async () => {
    vi.spyOn(api, 'insightsYear').mockResolvedValue(threeDays())
    renderApp('/year')

    const grid = await screen.findByTestId('year-calendar')
    const days = grid.querySelectorAll('[data-testid="year-calendar-day"]')
    expect(days).toHaveLength(3)
    // The hard day is the deepest colour, the easy one the palest, and
    // the day with no run is the neutral one: no mix, not a mix of zero.
    expect(days[0]?.getAttribute('fill')).toBe('var(--accent-400)')
    expect(days[1]?.getAttribute('fill')).toBe('var(--chart-band)')
    expect(days[2]?.getAttribute('fill')).toBe('var(--surface-sunken)')
  })

  it('says what changed, in the core own sentences', async () => {
    renderApp('/year')
    const lines = await screen.findByTestId('year-changed-lines')
    expect(lines.textContent).toContain('+76.6 km against the period before')
    expect(lines.textContent).toContain('+7 recorded sessions')
    expect(lines.textContent).toContain('the longest run grew by 18.2 km')
  })

  it('names the window it describes', async () => {
    renderApp('/year')
    await screen.findByTestId('year-months')
    const text = document.body.textContent ?? ''
    expect(text).toContain('1 Jun 2025')
    expect(text).toContain('13 May 2026')
  })

  it('says so when nothing was recorded', async () => {
    vi.spyOn(api, 'insightsYear').mockResolvedValue({
      ...yearFixture,
      months: yearFixture.months.map((month) => ({ ...month, sessions: 0 })),
    })
    renderApp('/year')
    expect(await screen.findByTestId('year-empty')).toBeTruthy()
  })

  it('says nothing about where the athlete ran', async () => {
    renderApp('/year')
    await screen.findByTestId('year-calendar')
    const text = (document.body.textContent ?? '').toLowerCase()
    for (const word of ['latitude', 'longitude', 'gps', 'street', 'address', 'coordinates']) {
      expect(text).not.toContain(word)
    }
  })

  it('draws no verdict and no instruction', async () => {
    renderApp('/year')
    await screen.findByTestId('year-months')
    const text = (document.body.textContent ?? '').toLowerCase()
    for (const word of ['should', 'must', 'good', 'bad', 'poor', 'missed', 'optimal', 'ready']) {
      expect(text).not.toContain(word)
    }
  })
})
