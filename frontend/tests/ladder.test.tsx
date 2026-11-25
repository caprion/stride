/**
 * The ladder: the pace card, and the curve behind it.
 *
 * **The card is the picture.** The rungs are drawn with their values and
 * their bands, the caption names the race the paces came from, and the
 * rules open beside the rung they belong to.
 *
 * The rules the drawings follow:
 *
 *   - a rung with no pace is a hollow marker, and the reason is written
 *     where the pace would have been;
 *   - the source line is hoisted once per screen, and the lines behind
 *     it are behind a click;
 *   - the curve is a second reading of the same athlete, on its own
 *     panel, and it never judges.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { curveFixture, pacesFixture } from '../src/fixtures'
import type { PacesInsight } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/**
 * The ladder with one rung's pace missing.
 *
 * **Absence is not zero**: the band still stands, and the pace says
 * where it would come from.
 */
const LADDER_NO_PACE: PacesInsight = {
  ...pacesFixture,
  rungs: pacesFixture.rungs.map((rung, index) =>
    index === 0
      ? {
          ...rung,
          pace: {
            value: null,
            fast_s_per_km: null,
            slow_s_per_km: null,
            missing: 'no runs at that heart rate yet',
          },
        }
      : rung,
  ),
}

describe('the pace card', () => {
  it('carries the ladder', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    expect(card.querySelector('[data-testid="ladder-picture"]')).toBeTruthy()
    // The rungs are the card's own rows, with their values.
    expect(card.querySelector('[data-testid="ladder-rung-lactate_threshold"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="ladder-value-lactate_threshold"]')?.textContent).toContain(
      '/km',
    )
  })

  it('opens the pace panel, not a page of its own', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-pace-link')

    expect(link.getAttribute('href')).toBe('/progress/pace')
  })

  it('opens the curve from the pace card', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-wall-pace-curve')

    expect(link.getAttribute('href')).toBe('/progress/curve')
  })

  it('names the race the paces came from', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    // The caption is the core's own words: the race, and the chain.
    const caption = card.querySelector('[data-testid="ladder-caption"]')
    expect(caption?.textContent).toContain('10K')
    expect(caption?.textContent).toContain('VDOT')
  })

  it('keeps a rung’s rule behind a click, with its citation', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    const rule = card.querySelector('[data-testid="ladder-rule-lactate_threshold"]')
    expect(rule).toBeTruthy()
    // The rule is a disclosure: it is closed until it is asked for.
    expect(rule?.tagName).toBe('DETAILS')
    expect(rule?.hasAttribute('open')).toBe(false)
    // And the wording it opens with is the core's own, with its
    // citation.
    expect(rule?.textContent).toContain('ch. 1, ch. 7')
  })

  it('says what is missing, and draws no band of zero', async () => {
    vi.spyOn(api, 'insightsPaces').mockResolvedValue(LADDER_NO_PACE)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    const value = card.querySelector('[data-testid="ladder-value-lactate_threshold"]')
    expect(value?.textContent).toContain('no runs at that heart rate yet')
    // The rung's own band still stands.
    expect(card.querySelector('[data-testid="ladder-hr-lactate_threshold"]')).toBeTruthy()
  })

  it('names the period the screen is showing', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    // The period, in the runner's own words, with the dates behind it.
    const window = card.querySelector('[data-testid="ladder-window"]')
    expect(window?.textContent).toContain('the last 12 weeks')
  })

  it('says why the ladder has no value at all', async () => {
    vi.spyOn(api, 'insightsPaces').mockResolvedValue({
      ...pacesFixture,
      rungs: [],
      unavailable: 'no race time entered — the paces have no value yet',
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    expect(card.querySelector('[data-testid="ladder-unavailable"]')?.textContent).toContain(
      'no race time entered',
    )
  })
})

describe('the pace panel', () => {
  it('carries the ladder, with the source line hoisted', async () => {
    renderApp('/progress/pace')
    const panel = await screen.findByTestId('progress-panel-pace')

    expect(panel.querySelector('[data-testid="ladder-picture"]')).toBeTruthy()
    // The source line is hoisted here: the panel is its own screen.
    expect(panel.querySelector('[data-testid="ladder-source"]')).toBeTruthy()
  })

  it('is where a saved /pace link lands', async () => {
    renderApp('/pace')

    // The page is gone, and the link lands on the answer that replaced
    // it — the same question, on the screen that owns it.
    const panel = await screen.findByTestId('progress-panel-pace')
    expect(panel.querySelector('[data-testid="ladder-picture"]')).toBeTruthy()
  })
})

describe('the effort curve', () => {
  it('draws the curve, the races and the sessions', async () => {
    renderApp('/progress/curve')
    const card = await screen.findByTestId('curve-card')

    expect(card.querySelector('[data-testid="curve-picture"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="curve-line"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="curve-sessions"]')).toBeTruthy()
  })

  it('names the races it holds', async () => {
    renderApp('/progress/curve')
    const races = await screen.findByTestId('curve-races')

    expect(races.textContent).toContain('10K')
  })

  it('says that the two axes are not converted into one another', async () => {
    renderApp('/progress/curve')
    const card = await screen.findByTestId('curve-card')

    expect(card.textContent).toContain('two different numbers')
  })

  it('has no curve when the athlete has no race time', async () => {
    vi.spyOn(api, 'insightsPacingCurve').mockResolvedValue({
      ...curveFixture,
      curve: [],
      unavailable: 'no race time entered — the paces have no value yet',
    })
    renderApp('/progress/curve')
    const card = await screen.findByTestId('curve-card')

    expect(card.querySelector('[data-testid="curve-unavailable"]')?.textContent).toContain(
      'no race time entered',
    )
    expect(card.querySelector('[data-testid="curve-line"]')).toBeNull()
  })
})
