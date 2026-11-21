/**
 * The athlete's shape.
 *
 * Six axes of the core's own facets, two windows, one chart. The chart
 * draws the two shapes against each other: each axis is scaled to the
 * larger of the two windows' values, so the reader compares the windows
 * and not a fixed anchor.
 *
 * The values are the core's own rendering, and nothing on the card is a
 * score, a grade or a verdict.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { renderApp } from './helpers'

/**
 * The chart asks for a size, and jsdom has no layout engine.
 *
 * The observer is stubbed with a size the drawing can be measured
 * against: the shape is drawn at 640 pixels wide, and the assertions are
 * about what the drawing says rather than how many pixels wide it is.
 */
class SizedResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}

  observe(target: Element) {
    this.callback(
      [
        {
          target,
          contentRect: { width: 640, height: 300 } as DOMRectReadOnly,
        } as unknown as DOMRectReadOnly,
      ] as unknown as ResizeObserverEntry[],
      this as unknown as ResizeObserver,
    )
  }

  unobserve() {}

  disconnect() {}
}

globalThis.ResizeObserver = SizedResizeObserver as never

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
  vi.restoreAllMocks()
})

/** The points of a drawn path, as `[x, y]` pairs. */
function pointsOf(path: SVGPathElement | null): [number, number][] {
  const d = path?.getAttribute('d') ?? ''
  return d
    .split(/[ML]/)
    .map((piece) => piece.trim())
    .filter((piece) => piece.length > 0)
    .map((piece) => {
      const [x, y] = piece.replace('Z', '').split(',')
      return [Number(x), Number(y)] as [number, number]
    })
}

/** The distance of a point from the middle of the drawing. */
function radiusOf(point: [number, number], centre: number): number {
  return Math.hypot(point[0] - centre, point[1] - centre)
}

describe("the athlete's shape", () => {
  it('is on the wall of panels, and draws the two windows', async () => {
    renderApp('/progress')
    const panel = await screen.findByTestId('progress-shape')

    // The chart itself, drawn by the app.
    const svg = panel.querySelector('[data-testid="progress-shape-svg"]')
    expect(svg).toBeTruthy()
    // Both windows are drawn on it.
    expect(panel.querySelector('[data-testid="progress-shape-short"]')).toBeTruthy()
    expect(panel.querySelector('[data-testid="progress-shape-long"]')).toBeTruthy()

    // The axes are named in the core's own words.
    const labels = [...panel.querySelectorAll('[data-testid="progress-shape-label"]')].map(
      (node) => node.textContent,
    )
    expect(labels).toEqual([
      'Volume',
      'Consistency',
      'Sport variety',
      'Density',
      'Duration',
      'Intensity',
    ])
  })

  it('shows the core\'s own numbers, and the windows they cover', async () => {
    renderApp('/progress')
    const panel = await screen.findByTestId('progress-shape')

    // The values are the core's rendering: "58.4 km", "148 bpm".
    expect(panel.querySelector('[data-testid="progress-shape-volume-short"]')?.textContent).toBe(
      '58.4 km',
    )
    expect(panel.querySelector('[data-testid="progress-shape-intensity-short"]')?.textContent).toBe(
      '148 bpm',
    )
    expect(panel.querySelector('[data-testid="progress-shape-consistency-short"]')?.textContent).toBe(
      '6',
    )
    // And the two windows are named.
    expect(panel.textContent).toContain('The last 28 days')
    expect(panel.textContent).toContain('The last 91 days')
    // And how the axes are scaled, so the shape is not read as a score.
    expect(panel.textContent).toContain('Each axis is scaled to the larger of the two windows.')
  })

  it('scales every axis to the larger of the two windows', async () => {
    // The longer window holds more: its volume is the larger of the two,
    // so it reaches the edge of the chart and the shorter window reaches
    // the share of it that its own volume is.
    const bigger = {
      ...rollingFixture,
      window: { ...rollingFixture.window, days: 91 },
      profile: rollingFixture.profile.map((facet) =>
        facet.key === 'volume' ? { ...facet, value: { text: '76.6 km', raw: 76600 } } : facet,
      ),
    }
    vi.spyOn(api, 'insightsRolling').mockImplementation(async (weeks = 12) =>
      weeks === 13 ? bigger : rollingFixture,
    )

    renderApp('/progress')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() =>
      expect(panel.querySelector('[data-testid="progress-shape-long"]')).toBeTruthy(),
    )

    const short = pointsOf(panel.querySelector('[data-testid="progress-shape-short"]'))
    const long = pointsOf(panel.querySelector('[data-testid="progress-shape-long"]'))
    // The volume is the first axis, and it is at the top of the chart.
    const centre = 640 / 2
    expect(radiusOf(short[0]!, centre)).toBeLessThan(radiusOf(long[0]!, centre))
    // And every other axis is the same in the two windows, so the two
    // shapes meet there.
    expect(radiusOf(short[1]!, centre)).toBeCloseTo(radiusOf(long[1]!, centre), 5)
  })

  it('opens on its own, like the other panels', async () => {
    renderApp('/progress/shape')
    const panel = await screen.findByTestId('progress-panel-shape')
    expect(panel.querySelector('[data-testid="progress-shape"]')).toBeTruthy()
    // And the chart is on it.
    expect(panel.querySelector('[data-testid="progress-shape-svg"]')).toBeTruthy()
  })

  it('draws no verdict', async () => {
    renderApp('/progress')
    const panel = await screen.findByTestId('progress-shape')
    const text = (panel.textContent ?? '').toLowerCase()
    for (const word of ['score', 'grade', 'excellent', 'very active', 'optimal', 'good', 'bad']) {
      expect(text).not.toContain(word)
    }
  })
})
