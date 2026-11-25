/**
 * The ledger: **the two blocks of zone time**.
 *
 * The card draws two blocks and never adds them up: the app's own zone
 * time, measured over **the time the athlete was running**, and the
 * watch's own zone minutes, measured over **the whole run**. The overlap
 * underneath them is where both bases exist, and it states the two
 * measurements side by side and the distance between them.
 *
 * The rules the test holds to:
 *
 *   - the two blocks are **two blocks**, each with its own basis;
 *   - the absence is **white space**, never a zero;
 *   - the source line is hoisted to the screen, and the card does not
 *     repeat it;
 *   - **nothing is added up**: the two bases never make one number.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { stimulusFixture } from '../src/fixtures'
import type { StimulusInsight } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The ledger with both bases, and the overlap between them. */
const WITH_BOTH: StimulusInsight = {
  ...stimulusFixture,
  kinds: [
    {
      key: 'low',
      label: 'Z1–Z2 low',
      time: { text: '1 h 2 m', raw: 3720 },
      share: { text: '60.00%', raw: 60 },
      runs: 3,
      last: null,
    },
    {
      key: 'moderate',
      label: 'Z3 moderate',
      time: { text: '2 h 4 m', raw: 7440 },
      share: { text: '30.00%', raw: 30 },
      runs: 2,
      last: null,
    },
    {
      key: 'high',
      label: 'Z4–Z5 high',
      time: { text: '1 h 4 m', raw: 3840 },
      share: { text: '10.00%', raw: 10 },
      runs: 1,
      last: null,
    },
  ],
  weeks: [
    {
      week_start: '2026-05-04',
      week_start_label: '4 May 2026',
      bands: [
        { key: 'low', label: 'Z1–Z2 low', time: { text: '1 h 2 m', raw: 3720 } },
        { key: 'moderate', label: 'Z3 moderate', time: { text: '2 h 4 m', raw: 7440 } },
        { key: 'high', label: 'Z4–Z5 high', time: null },
      ],
    },
  ],
  source_zones: [
    {
      zone: 1,
      time: { text: '2 h 4 m', raw: 7440 },
      share: { text: '50.00%', raw: 50 },
      runs: 3,
    },
    {
      zone: 4,
      time: { text: '1 h 4 m', raw: 3840 },
      share: { text: '16.00%', raw: 16 },
      runs: 1,
    },
  ],
  source_weeks: [
    {
      week_start: '2026-05-04',
      week_start_label: '4 May 2026',
      zones: [
        { text: '2 h 4 m', raw: 7440 },
        null,
        null,
        { text: '1 h 4 m', raw: 3840 },
        null,
      ],
    },
  ],
  overlap: {
    from: '2026-04-13',
    from_label: '13 Apr 2026',
    to: '2026-05-13',
    to_label: '13 May 2026',
    runs: 4,
    app_moving: { text: '3 h 12 m', raw: 11520 },
    source_elapsed: { text: '3 h 40 m', raw: 13200 },
    difference: { text: '28 m', raw: 1680 },
  },
}

describe('where the time went', () => {
  it('draws the two blocks, each with its own basis', async () => {
    vi.spyOn(api, 'insightsStimulus').mockResolvedValue(WITH_BOTH)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')
    await waitFor(() =>
      expect(card.querySelector('[data-testid="ledger-blocks"]')).toBeTruthy(),
    )

    // The app's own zone time, and the time it was measured over.
    expect(card.querySelector('[data-testid="ledger-app"]')?.textContent).toContain(
      'the time you were running',
    )
    // The watch's own zone minutes, and the whole run.
    expect(card.querySelector('[data-testid="ledger-source"]')?.textContent).toContain(
      'the whole run',
    )
  })

  it('draws the absence as white space', async () => {
    vi.spyOn(api, 'insightsStimulus').mockResolvedValue(WITH_BOTH)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')
    await waitFor(() =>
      expect(card.querySelector('[data-testid="ledger-app-weeks"]')).toBeTruthy(),
    )

    // The week that held no hard running: the high band's cell is empty,
    // and it is empty rather than a bar of zero.
    const cell = card.querySelector(
      '[data-testid="ledger-app-weeks-2026-05-04-high"]',
    )
    expect(cell).toBeTruthy()
    expect(cell?.getAttribute('style')).toContain('height: 0%')
  })

  it('reads the overlap as two measurements and a gap', async () => {
    vi.spyOn(api, 'insightsStimulus').mockResolvedValue(WITH_BOTH)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')
    const overlap = await screen.findByTestId('ledger-overlap')

    // The two bases stand side by side: the app's own and the watch's.
    expect(card.querySelector('[data-testid="ledger-overlap-app"]')?.textContent).toContain(
      '3 h 12 m',
    )
    expect(card.querySelector('[data-testid="ledger-overlap-source"]')?.textContent).toContain(
      '3 h 40 m',
    )
    // And the distance between them is stated.
    expect(overlap.querySelector('[data-testid="ledger-overlap-gap"]')?.textContent).toContain(
      '28 m',
    )
  })

  it('hoists the source line to the screen, and does not repeat it on the card', async () => {
    vi.spyOn(api, 'insightsStimulus').mockResolvedValue(WITH_BOTH)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')

    // The screen names the model it measured against.
    await waitFor(() =>
      expect(screen.getByTestId('progress-wall-zones-source')).toBeTruthy(),
    )
    expect(screen.getByTestId('progress-wall-zones-source').textContent).toContain('HR_MAX')
    // And the card does not repeat it.
    expect(card.querySelector('[data-testid="ledger-source-line"]')).toBeNull()
  })

  it('says why the ledger has no value at all', async () => {
    vi.spyOn(api, 'insightsStimulus').mockResolvedValue({
      ...WITH_BOTH,
      unavailable: 'no runs recorded',
      overlap: null,
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')

    await waitFor(() =>
      expect(card.querySelector('[data-testid="ledger-unavailable"]')).toBeTruthy(),
    )
    expect(card.querySelector('[data-testid="ledger-unavailable"]')?.textContent).toContain(
      'no runs recorded',
    )
  })
})
