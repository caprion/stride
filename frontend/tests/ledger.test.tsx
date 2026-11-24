/**
 * The ledger: **where the time went**.
 *
 * The card is the one bar, the weeks × bands matrix and the longest
 * runs — and the **source line is hoisted**: it is named once, on the
 * screen, and no card repeats it.
 *
 * The rules the test holds to:
 *
 *   - the split is printed **once**: the roll-up is read off the bar,
 *     not printed beside it;
 *   - the matrix's absence is **white space**, never a zero;
 *   - the source line is on the screen, and not on the card.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The rolling answer with a split in it. */
const WITH_SPLIT = {
  ...rollingFixture,
  zones: {
    models: rollingFixture.zones.models,
    source: rollingFixture.zones.source,
    unavailable: null,
    distribution: [
      {
        zone: 'Z1',
        range: '106-121 bpm',
        time: { text: '1 h 2 m', raw: 3720 },
        share: { text: '20.00%', raw: 20 },
      },
      {
        zone: 'Z2',
        range: '122-137 bpm',
        time: { text: '2 h 4 m', raw: 7440 },
        share: { text: '40.00%', raw: 40 },
      },
      {
        zone: 'Z3',
        range: '138-153 bpm',
        time: { text: '2 h 20 m', raw: 8400 },
        share: { text: '30.00%', raw: 30 },
      },
      {
        zone: 'Z4',
        range: '154-169 bpm',
        time: { text: '1 h 4 m', raw: 3840 },
        share: { text: '9.00%', raw: 9 },
      },
      {
        zone: 'Z5',
        range: '170 bpm and above',
        time: { text: '0 m', raw: 0 },
        share: { text: '1.00%', raw: 1 },
      },
    ],
    shares: [
      {
        key: 'low',
        label: 'Z1–Z2 low',
        share: { text: '60.00%', raw: 60 },
        previous_share: { text: '58.00%', raw: 58 },
        trend: 'up',
      },
      {
        key: 'moderate',
        label: 'Z3 moderate',
        share: { text: '30.00%', raw: 30 },
        previous_share: { text: '31.00%', raw: 31 },
        trend: 'down',
      },
      {
        key: 'high',
        label: 'Z4–Z5 high',
        share: { text: '10.00%', raw: 10 },
        previous_share: { text: '11.00%', raw: 11 },
        trend: 'down',
      },
    ],
    weeks: [
      {
        week_start: '2026-05-04',
        week_start_label: '4 May 2026',
        sessions: 1,
        bands: [
          { key: 'low', label: 'Z1–Z2 low', time: { text: '1 h 2 m', raw: 3720 } },
          { key: 'moderate', label: 'Z3 moderate', time: { text: '2 h 4 m', raw: 7440 } },
          { key: 'high', label: 'Z4–Z5 high', time: null },
        ],
      },
    ],
  },
}

describe('where the time went', () => {
  it('draws the split once, and the matrix and the longest runs beside it', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue(WITH_SPLIT)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')
    await waitFor(() =>
      expect(card.querySelector('[data-testid="progress-picture-matrix"]')).toBeTruthy(),
    )

    // The one bar, with the five zones named under it.
    expect(card.querySelector('[data-testid="progress-picture-zones"]')).toBeTruthy()
    // The weeks × bands matrix.
    expect(card.querySelector('[data-testid="progress-picture-matrix"]')).toBeTruthy()
    // The longest runs.
    expect(card.querySelector('[data-testid="progress-picture-long-runs"]')).toBeTruthy()
    // The roll-up is read off the bar, not printed beside it.
    expect(card.querySelector('[data-testid="progress-picture-zones-rollup"]')).toBeNull()
  })

  it('draws the absence as white space', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue(WITH_SPLIT)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')
    await waitFor(() =>
      expect(card.querySelector('[data-testid="progress-picture-matrix"]')).toBeTruthy(),
    )

    // The week that held no hard running: the high band's cell is empty,
    // and it is empty rather than a bar of zero.
    const cell = card.querySelector('[data-testid="progress-picture-matrix-2026-05-04-high"]')
    expect(cell).toBeTruthy()
    expect(cell?.getAttribute('style')).toContain('height: 0%')
  })

  it('hoists the source line to the screen, and does not repeat it on the card', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue(WITH_SPLIT)
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')

    // The screen names the model it measured against.
    await waitFor(() =>
      expect(screen.getByTestId('progress-wall-zones-source')).toBeTruthy(),
    )
    expect(screen.getByTestId('progress-wall-zones-source').textContent).toContain('HR_MAX')
    // And the card does not repeat it.
    expect(card.querySelector('[data-testid="progress-wall-zones-source"]')).toBeNull()
  })
})
