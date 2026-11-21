/**
 * The wall: the desktop's Progress.
 *
 * The wide Progress is **the wall**: every panel the router answers, on
 * one screen, in the mock's cards — a name, the answer, and the way into
 * the panel that answers it.
 *
 * The tests hold three things:
 *
 *   - **Every card opens its panel.** The "open →" in the card's head is
 *     the doorway the mock's drawer is.
 *   - **No new numbers.** Every value on the wall is the router's own
 *     rendering, and a panel with no value says so and shows no number.
 *   - **The phone keeps the index.** The six rows are still there, and
 *     they are still the only thing the phone shows.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The cards of the wall, and the panel each one opens. */
const CARDS: [card: string, open: string, panel: string][] = [
  ['year', 'progress-wall-open-year', '/year'],
  ['pace', 'progress-pace-link', '/pace'],
  ['activities', 'progress-wall-open-activities', '/runs'],
  ['load', 'progress-wall-open-load', '/progress/load'],
  ['volume', 'progress-wall-open-volume', '/progress/volume'],
  ['week', 'progress-wall-open-week', '/progress/week'],
  ['heatmap', 'progress-wall-open-heatmap', '/progress/heatmap'],
  ['streaks', 'progress-wall-open-streaks', '/progress/habit'],
  ['profile', 'progress-wall-open-profile', '/progress/profile'],
  ['facets', 'progress-wall-open-facets', '/progress/facets'],
  ['eddington', 'progress-wall-open-eddington', '/progress/eddington'],
  ['zones', 'progress-wall-open-zones', '/progress/zone-time'],
  ['split', 'progress-wall-open-split', '/progress/zones'],
  ['window', 'progress-wall-open-window', '/progress/window'],
  ['weekday', 'progress-wall-open-weekday', '/progress/weekday'],
  ['daytime', 'progress-wall-open-daytime', '/progress/daytime'],
  ['distances', 'progress-wall-open-distances', '/progress/distances'],
  ['months', 'progress-wall-open-months', '/progress/months'],
  ['years', 'progress-wall-open-years', '/progress/year'],
  ['bests', 'progress-wall-open-bests', '/progress/bests'],
  ['consistency', 'progress-wall-open-consistency', '/progress/consistency'],
  ['body', 'progress-wall-open-body', '/body'],
]

describe('the wall on Progress', () => {
  it('has a card for every panel, and each card opens it', async () => {
    renderApp('/progress')
    await screen.findByTestId('progress-wall')

    for (const [, open, panel] of CARDS) {
      const link = await screen.findByTestId(open)
      expect(link.getAttribute('href')).toBe(panel)
    }
  })

  it('keeps the index for the phone', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // Six questions, and no more.
    expect(index.querySelectorAll('a[data-testid^="progress-index-"]')).toHaveLength(6)
  })

  it('shows the values the router answered with', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-year')

    // The core's own words: the months the answer holds, and the window.
    expect(card.textContent).toContain('12 months')
    expect(card.textContent).toContain('1 Jun 2025')
  })

  it('has no number where the answer has none', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        model: null,
        unavailable: 'the runs in the window carry no heart-rate samples',
        shares: null,
        distribution: null,
      },
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-zones')

    // The reason is the answer's own words, and no share is drawn.
    expect(card.textContent).toContain('no heart-rate samples')
    expect(card.textContent).not.toContain('%')
  })
})

/**
 * The athlete profile: two windows on one chart.
 *
 * The radar draws the two polygons and the table under it holds the
 * numbers behind them. Every number is the core's own rendering, and the
 * two windows are named with the **answer's** count of days.
 */
describe('the athlete profile on Progress', () => {
  it('draws the two windows and names them', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    // One polygon for each window.
    expect(card.querySelector('[data-testid="progress-radar-short"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="progress-radar-long"]')).toBeTruthy()

    // The windows are named with the answer's own count of days: the
    // router's window is measured in weeks, and the card says what it
    // answered.
    expect(card.textContent).toContain('The last 28 days')
    expect(card.textContent).toContain('The last 84 days')
  })

  it("names the six axes in the core's own words", async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    for (const key of [
      'volume',
      'consistency',
      'variety',
      'density',
      'duration',
      'intensity',
    ]) {
      expect(card.querySelector(`[data-testid="progress-radar-label-${key}"]`)).toBeTruthy()
    }
    expect(card.textContent).toContain('Sport variety')
    // The week days are not an axis: a week has no shape.
    expect(card.querySelector('[data-testid="progress-radar-label-week_days"]')).toBeNull()
  })

  it('holds the values behind the shape', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    // The core's rendering of each facet, in the window's own units.
    expect(
      card.querySelector('[data-testid="progress-wall-radar-short-volume"]')?.textContent,
    ).toBe('58.4 km')
    expect(
      card.querySelector('[data-testid="progress-wall-radar-short-intensity"]')?.textContent,
    ).toBe('148 bpm')
    expect(
      card.querySelector('[data-testid="progress-wall-radar-short-variety"]')?.textContent,
    ).toBe('1')
  })
})
