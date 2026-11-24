/**
 * The pace: **the cloud and the bands**.
 *
 * The pace page is gone. Its two panels — the pace at a heart rate, and
 * where the time went by pace — are drawn on Progress's **The pace**
 * card, and on the panel behind it.
 *
 * The rules the drawings follow:
 *
 *   - the cloud is one point a week, and a week with no pace is not a
 *     point;
 *   - the bands are the answer's own, and a window with no samples says
 *     so instead of drawing a band of zero;
 *   - the time in the bands too small to draw is named, not lost.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { paceFixture } from '../src/fixtures'
import { paceCloud } from '../src/pages/pacePanels'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/**
 * The bands, for the state in which the pace is reviewed.
 *
 * **The values are invented**, like every other value in the fixture set:
 * the seeded store holds no FIT file, so the router's own answer for the
 * window carries no bands at all.
 */
const BANDS = {
  width_s_per_km: 30,
  total: { text: '3 h 12 m', raw: 11520 },
  distance: { text: '18 km', raw: 18000 },
  rest: { text: '0 m', raw: 0 },
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

describe('the pace card', () => {
  it('carries the cloud and the bands', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-pace')

    expect(card.querySelector('[data-testid="pace-cloud"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="pace-bands"]')).toBeTruthy()
  })

  it('opens the pace panel, not a page of its own', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-pace-link')

    expect(link.getAttribute('href')).toBe('/progress/pace')
  })
})

describe('the pace panel', () => {
  it('carries the cloud and the bands', async () => {
    renderApp('/progress/pace')

    expect(await screen.findByTestId('pace-pace-at-hr')).toBeTruthy()
    expect(await screen.findByTestId('pace-bands-panel')).toBeTruthy()
  })

  it('is where a saved /pace link lands', async () => {
    renderApp('/pace')

    // The page is gone, and the link lands on the answer that replaced
    // it — the same question, on the screen that owns it.
    const panel = await screen.findByTestId('progress-panel-pace')
    expect(panel.querySelector('[data-testid="pace-pace-at-hr"]')).toBeTruthy()
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
    const { container } = renderApp('/progress/pace')
    await screen.findByTestId('pace-pace-at-hr')

    const cloud = container.querySelector('[data-points]')
    expect(cloud?.getAttribute('data-points')).toBe('1')
  })

  it('says why there are no bands when the window holds no samples', async () => {
    renderApp('/progress/pace')
    const bands = await screen.findByTestId('pace-bands-panel')

    // The reason is the core's own words, and no band of zero is drawn.
    expect(bands.querySelector('[data-testid="pace-bands-empty"]')?.textContent).toContain(
      'none of the runs in this period recorded a pace',
    )
    expect(bands.querySelector('[data-testid="pace-bands-rows"]')).toBeNull()
  })

  it('draws the bands the answer carries', async () => {
    vi.spyOn(api, 'insightsPace').mockResolvedValue({
      ...paceFixture,
      bands: BANDS,
      unavailable: null,
    })
    renderApp('/progress/pace')
    const rows = await screen.findByTestId('pace-bands-rows')

    // The labels are the core's own words, and the bar is the share.
    expect(rows.textContent).toContain('4:00–4:30 /km')
    expect(rows.textContent).toContain('4:30–5:00 /km')
    expect(rows.textContent).toContain('3 h')
    const bar = rows.querySelector('[data-testid="pace-bands-bar-270"]')
    expect(bar?.getAttribute('style')).toContain('93.75%')
  })

  it('says how much time is in the bands too small to draw', async () => {
    vi.spyOn(api, 'insightsPace').mockResolvedValue({
      ...paceFixture,
      bands: { ...BANDS, rest: { text: '41 m', raw: 2460 } },
      unavailable: null,
    })
    renderApp('/progress/pace')
    const bands = await screen.findByTestId('pace-bands-panel')

    // The time is not lost: it is beside the bands, not in one of them.
    expect(bands.querySelector('[data-testid="pace-bands-rest"]')?.textContent).toContain('41 m')
    expect(bands.querySelector('[data-testid="pace-bands-rest"]')?.textContent).toContain(
      'The smaller bands',
    )
  })
})
