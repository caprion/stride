/**
 * The Progress screen.
 *
 * The Eddington card: the number, the days still needed for the next one
 * and the longest distance in a day — and the window the card says they
 * cover. A number without its period is not a fact, so the window is
 * asserted with the values.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { loadFixture, progressFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

describe('the Eddington number on Progress', () => {
  it('shows the number, the days to the next one and the longest day', async () => {
    const { container } = renderApp('/progress')
    const card = await screen.findByTestId('progress-eddington')

    expect(
      card.querySelector('[data-testid="progress-eddington-number"]')?.textContent,
    ).toBe('5')
    expect(card.querySelector('[data-testid="progress-eddington-days"]')?.textContent).toBe('1')
    expect(card.querySelector('[data-testid="progress-eddington-longest"]')?.textContent).toBe(
      '16.1 km',
    )
    // The core's own words for the longest day, not a number the app
    // formatted.
    expect(container.textContent).toContain('16.1 km')
  })

  it('names the window the number covers', async () => {
    const { container } = renderApp('/progress')
    const card = await screen.findByTestId('progress-eddington')
    const window = card.querySelector('[data-testid="progress-eddington-window"]')?.textContent

    expect(window).toContain('23 Feb')
    expect(window).toContain('13 May')
    // And it is the window the totals beside it describe.
    expect(container.querySelector('[data-testid="progress-summary"]')?.textContent).toBeTruthy()
  })

  it('has no number when the window holds no distance', async () => {
    vi.spyOn(api, 'insightsProgress').mockResolvedValue({
      ...progressFixture,
      eddington: null,
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-eddington')

    expect(
      card.querySelector('[data-testid="progress-eddington-empty"]')?.textContent,
    ).toContain('No recorded distance in this window')
    // No value, not a zero: the card shows no number at all.
    expect(card.querySelector('[data-testid="progress-eddington-number"]')).toBeNull()
    expect(card.textContent).not.toContain('Longest distance in a day')
  })
})

/**
 * The training load on Progress.
 *
 * The card is **values and a scenario**: the two weighted series, the
 * form, the ratio, the week's three numbers, and what a run of rest days
 * is expected to do with them — with the assumption the numbers rest on
 * attached. There is no score, no grade and no verdict, and the band of
 * the ratio is named as a range.
 *
 * A load with no value says why, and shows no number at all: absence is
 * not zero.
 */
describe('the training load on Progress', () => {
  it('shows the values, the scenario and the assumption it rests on', async () => {
    const { container } = renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    // The two weighted series, the form and the ratio — the core's own
    // rendering, not a number the app formatted.
    expect(card.querySelector('[data-testid="progress-load-acute"]')?.textContent).toBe('19.7')
    expect(card.querySelector('[data-testid="progress-load-chronic"]')?.textContent).toBe('7.0')
    expect(card.querySelector('[data-testid="progress-load-form"]')?.textContent).toBe('-12.7')
    expect(card.querySelector('[data-testid="progress-load-ratio"]')?.textContent).toBe('2.82')

    // The week's three numbers.
    expect(card.querySelector('[data-testid="progress-load-impulse"]')?.textContent).toBe('177')
    expect(card.querySelector('[data-testid="progress-load-monotony"]')?.textContent).toBe('0.83')
    expect(card.querySelector('[data-testid="progress-load-strain"]')?.textContent).toBe('148')

    // The band is a range, and it is named as one.
    expect(card.querySelector('[data-testid="progress-load-band"]')?.textContent).toContain(
      '0.8 to 1.3',
    )

    // The scenario, in the core's words.
    const scenario = card.querySelector('[data-testid="progress-load-scenario"]')?.textContent
    expect(scenario).toContain('acute-to-chronic ratio')
    expect(scenario).toContain('0.8 to 1.3')

    // And the assumption with it: the coefficient, where the maximum
    // heart rate came from and the day it was read, and the windows.
    const assumption = card.querySelector('[data-testid="progress-load-assumption"]')?.textContent
    expect(assumption).toContain('Banister k = 1.92')
    expect(assumption).toContain('read on 10 May 2026')
    expect(assumption).toContain('chronic window 42 days')

    // And no verdict anywhere on the card.
    const text = card.textContent ?? ''
    for (const word of ['risk', 'optimal', 'ready', 'should', 'must']) {
      expect(text.toLowerCase()).not.toContain(word)
    }
    expect(container.querySelector('[data-testid="progress-load"]')).toBeTruthy()
  })

  it('names the window it describes', async () => {
    const { container } = renderApp('/progress')
    const card = await screen.findByTestId('progress-load')
    const window = card.querySelector('[data-testid="progress-load-window"]')?.textContent

    // The api's own rendering of a date.
    expect(window).toContain('23 Feb 2026')
    expect(window).toContain('13 May 2026')
    // And it is the window the card's values describe.
    expect(container.querySelector('[data-testid="progress-load-as-of"]')?.textContent).toContain(
      '13 May 2026',
    )
  })

  it('has no numbers when the load has no value', async () => {
    vi.spyOn(api, 'insightsLoad').mockResolvedValue({
      ...loadFixture,
      load: null,
      scenario: null,
      unavailable: 'no_zone_model',
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-load')

    // The reason is the core's, in the app's words.
    expect(card.querySelector('[data-testid="progress-load-empty"]')?.textContent).toContain(
      'No zone model has been read',
    )
    // No value, not a zero: the card shows no number at all.
    expect(card.querySelector('[data-testid="progress-load-acute"]')).toBeNull()
    expect(card.querySelector('[data-testid="progress-load-ratio"]')).toBeNull()
    expect(card.querySelector('[data-testid="progress-load-scenario"]')).toBeNull()
  })

  it('follows the range the screen is showing', async () => {
    const asked = vi.spyOn(api, 'insightsLoad')
    renderApp('/progress')
    await screen.findByTestId('progress-load')

    await userEvent.click(screen.getByTestId('progress-range-4'))
    await waitFor(() => expect(asked).toHaveBeenCalledWith(4))
  })
})
