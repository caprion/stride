/**
 * The charts of the training load.
 *
 * **The numbers were already there.** The router's answer carried the
 * day-by-day series and the seven days the scenario projects, and the
 * panel showed the last value of each and nothing else. These charts
 * draw what the answer already carried — and they draw nothing it did
 * not send.
 *
 * The rules they rest on are tested in `load-series.test.ts`: the days
 * keep their places, a day with no load is not a day with a load of zero,
 * and the scale always holds the zero. What is checked here is that the
 * panel draws them, that the legend names the three lines, and that
 * **there is no verdict anywhere on the card**.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { loadFixture, rollingFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

describe('the charts of the training load', () => {
  it('draws the series, the days and the projection', async () => {
    // The panel is the whole answer; the card is the headline.
    renderApp('/progress/load')
    const card = await screen.findByTestId('progress-load')

    // The shape of the training, over the window the screen is showing.
    expect(card.querySelector('[data-testid="progress-load-series"]')).toBeTruthy()
    // The texture under it: the load of each day.
    expect(card.querySelector('[data-testid="progress-load-days"]')).toBeTruthy()
    // And what the scenario expects.
    expect(card.querySelector('[data-testid="progress-load-projection"]')).toBeTruthy()
  })

  it('gives the wall card the headline and the panel the whole answer', async () => {
    const { unmount } = renderApp('/progress')
    const card = await screen.findByTestId('progress-load')
    // The card on the wall: the shape and the numbers behind it.
    expect(card.querySelector('[data-testid="progress-load-series"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="progress-load-days"]')).toBeNull()
    unmount()

    renderApp('/progress/load')
    const panel = await screen.findByTestId('progress-load')
    expect(panel.querySelector('[data-testid="progress-load-days"]')).toBeTruthy()
    expect(panel.querySelector('[data-testid="progress-load-projection"]')).toBeTruthy()
  })

  it('names the three lines it draws', async () => {
    renderApp('/progress/load')
    const card = await screen.findByTestId('progress-load')

    expect(card.querySelector('[data-testid="progress-load-legend-chronic"]')?.textContent).toBe(
      'The last 6 weeks',
    )
    expect(card.querySelector('[data-testid="progress-load-legend-acute"]')?.textContent).toBe(
      'The last 7 days',
    )
    expect(card.querySelector('[data-testid="progress-load-legend-form"]')?.textContent).toBe(
      'The difference',
    )
  })

  it('says what the charts show, and what the empty bars are', async () => {
    renderApp('/progress/load')
    const card = await screen.findByTestId('progress-load')

    expect(card.querySelector('[data-testid="progress-load-series-note"]')?.textContent).toContain(
      'The difference is positive above the line',
    )
    // A day with no load and a day with a load of zero look the same on
    // the chart, and are not the same thing.
    expect(card.querySelector('[data-testid="progress-load-days-note"]')?.textContent).toContain(
      'nothing to weigh',
    )
    // And the projection says what it assumes.
    expect(
      card.querySelector('[data-testid="progress-load-projection-note"]')?.textContent,
    ).toContain('7 days')
  })

  it('has no verdict on the card, and no band name', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')
    const text = (card.textContent ?? '').toLowerCase()

    for (const word of ['risk', 'optimal', 'ready', 'should', 'must', 'fresh', 'fatigued']) {
      expect(text).not.toContain(word)
    }
  })

  it('draws no chart when the load has no value', async () => {
    vi.spyOn(api, 'insightsLoad').mockResolvedValue({
      ...loadFixture,
      load: null,
      scenario: null,
      unavailable: 'no_zone_model',
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    // No value, not a zero: there is no chart to draw.
    expect(card.querySelector('[data-testid="progress-load-series"]')).toBeNull()
    expect(card.querySelector('[data-testid="progress-load-days"]')).toBeNull()
    expect(card.querySelector('[data-testid="progress-load-projection"]')).toBeNull()
  })
})

/**
 * The graphs come before the text.
 *
 * **The card opens with the picture.** The shape of the training is
 * what the window looks like, and a reader takes that in before they
 * read a number; the numbers are the facts behind the picture, and they
 * come after it.
 */
describe('the shape before the numbers', () => {
  it('draws the chart of the load before the values', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    const chart = card.querySelector('[data-testid="progress-load-series"]')
    const values = card.querySelector('[data-testid="progress-load-values"]')
    expect(chart).toBeTruthy()
    expect(values).toBeTruthy()
    // The chart comes first in the card, not after the numbers.
    expect(
      chart!.compareDocumentPosition(values!) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('draws the split of the training as bars', async () => {
    // The router's own answer carries the split; the fixture's rolling
    // answer has none, so the card would have nothing to draw.
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        ...rollingFixture.zones,
        shares: [
          {
            key: 'low',
            label: 'Z1–Z2 low',
            share: { raw: 6.11, text: '6.11%' },
            previous_share: { raw: 5.89, text: '5.89%' },
            trend: 'up',
          },
          {
            key: 'moderate',
            label: 'Z3 moderate',
            share: { raw: 39.51, text: '39.51%' },
            previous_share: { raw: 27.1, text: '27.10%' },
            trend: 'up',
          },
          {
            key: 'high',
            label: 'Z4–Z5 high',
            share: { raw: 54.37, text: '54.37%' },
            previous_share: { raw: 67.01, text: '67.01%' },
            trend: 'down',
          },
        ],
      },
    })
    renderApp('/progress/zone-time')
    const card = await screen.findByTestId('progress-zones')

    // The three shares are bars as well as numbers.
    expect(card.querySelector('[data-testid="progress-zones-bar-low"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="progress-zones-bar-moderate"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="progress-zones-bar-high"]')).toBeTruthy()
    // And the numbers are still there, in the core's own words.
    expect(card.querySelector('[data-testid="progress-zones-high"]')?.textContent).toContain('%')
  })
})
