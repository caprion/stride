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
import { loadFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

describe('the charts of the training load', () => {
  it('draws the series, the days and the projection', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    // The shape of the training, over the window the screen is showing.
    expect(card.querySelector('[data-testid="progress-load-series"]')).toBeTruthy()
    // The texture under it: the load of each day.
    expect(card.querySelector('[data-testid="progress-load-days"]')).toBeTruthy()
    // And what the scenario expects.
    expect(card.querySelector('[data-testid="progress-load-projection"]')).toBeTruthy()
  })

  it('names the three lines it draws', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    expect(card.querySelector('[data-testid="progress-load-legend-chronic"]')?.textContent).toBe(
      'Chronic load',
    )
    expect(card.querySelector('[data-testid="progress-load-legend-acute"]')?.textContent).toBe(
      'Acute load',
    )
    expect(card.querySelector('[data-testid="progress-load-legend-form"]')?.textContent).toBe(
      'Form',
    )
  })

  it('says what the charts show, and what the empty bars are', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    expect(card.querySelector('[data-testid="progress-load-series-note"]')?.textContent).toContain(
      'The form is positive above the line',
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
