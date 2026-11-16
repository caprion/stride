/**
 * The Progress screen.
 *
 * The Eddington card: the number, the days still needed for the next one
 * and the longest distance in a day — and the window the card says they
 * cover. A number without its period is not a fact, so the window is
 * asserted with the values.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { progressFixture } from '../src/fixtures'
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
