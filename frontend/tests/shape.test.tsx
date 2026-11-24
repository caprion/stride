/**
 * The athlete's shape.
 *
 * Six axes of the core's own facets, three windows, one chart. The chart
 * draws the three shapes against each other: each axis is scaled to the
 * largest of the three windows' values, so the reader compares the
 * windows and not a fixed anchor.
 *
 * The values are the core's own rendering, and nothing on the card is a
 * score, a grade or a verdict.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { RADAR_AXES } from '../src/components/ProfileRadar'
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

/** The middle of the drawing: the chart is 340 pixels wide at most. */
const CENTRE = 340 / 2

describe("the athlete's shape", () => {
  it('is on the wall of panels, and draws the three windows', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')

    // The chart itself, drawn by the app.
    const svg = panel.querySelector('[data-testid="progress-shape-svg"]')
    expect(svg).toBeTruthy()
    // All three windows are drawn on it, one on the other: the last
    // thirty days, the last sixty and the last ninety.
    expect(panel.querySelector('[data-testid="progress-shape-window-30"]')).toBeTruthy()
    expect(panel.querySelector('[data-testid="progress-shape-window-60"]')).toBeTruthy()
    expect(panel.querySelector('[data-testid="progress-shape-window-90"]')).toBeTruthy()

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
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')

    // The values are the core's rendering, and they are the **rates**:
    // the volume a week and the consistency a share of the days, so that
    // the windows can be read against each other.
    expect(panel.querySelector('[data-testid="progress-shape-volume-now"]')?.textContent).toBe(
      '4.5 km a week',
    )
    expect(panel.querySelector('[data-testid="progress-shape-intensity-now"]')?.textContent).toBe(
      '148 bpm',
    )
    expect(panel.querySelector('[data-testid="progress-shape-consistency-now"]')?.textContent).toBe(
      '6.7% of the days',
    )
    // And the window before it, of the same length.
    expect(panel.querySelector('[data-testid="progress-shape-volume-before"]')?.textContent).toBe(
      '3.5 km a week',
    )
    // The three windows are named: the legend says which is which.
    const legend = panel.querySelector('[data-testid="progress-shape-legend"]')
    expect(legend?.textContent).toContain('last 30 days')
    expect(legend?.textContent).toContain('last 60 days')
    expect(legend?.textContent).toContain('last 90 days')
    // And how the axes are scaled, so the shape is not read as a score.
    expect(panel.textContent).toContain('Each axis is scaled to the largest of the three windows.')
    // And what the rates are, so a number is not read apart from its
    // unit.
    expect(panel.textContent).toContain(
      'The volume is a distance a week and the consistency the share of the days',
    )
  })

  it('scales every axis to the largest of the three windows', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() =>
      expect(panel.querySelector('[data-testid="progress-shape-window-30"]')).toBeTruthy(),
    )

    const thirty = pointsOf(panel.querySelector('[data-testid="progress-shape-window-30"]'))
    const sixty = pointsOf(panel.querySelector('[data-testid="progress-shape-window-60"]'))
    const ninety = pointsOf(panel.querySelector('[data-testid="progress-shape-window-90"]'))
    // The volume is the first axis, and it is at the top of the chart.
    // The thirty-day window has the most distance a week, so it reaches
    // the edge and the ninety-day one sits inside it.
    expect(radiusOf(thirty[0]!, CENTRE)).toBeGreaterThan(radiusOf(sixty[0]!, CENTRE))
    expect(radiusOf(sixty[0]!, CENTRE)).toBeGreaterThan(radiusOf(ninety[0]!, CENTRE))
    // And on the axis the three windows agree on — the intensity — the
    // three shapes meet, because each axis is scaled to the largest of
    // the three.
    const intensity = RADAR_AXES.indexOf('intensity')
    expect(radiusOf(ninety[intensity]!, CENTRE)).toBeCloseTo(radiusOf(thirty[intensity]!, CENTRE), 5)
    expect(radiusOf(ninety[intensity]!, CENTRE)).toBeCloseTo(radiusOf(sixty[intensity]!, CENTRE), 5)
  })

  it('opens on its own, like the other panels', async () => {
    renderApp('/profile/shape')
    const panel = await screen.findByTestId('profile-panel-shape')
    expect(panel.querySelector('[data-testid="progress-shape"]')).toBeTruthy()
    // And the chart is on it.
    expect(panel.querySelector('[data-testid="progress-shape-svg"]')).toBeTruthy()
  })

  it('opens from the old link too', async () => {
    renderApp('/progress/shape')
    const panel = await screen.findByTestId('profile-panel-shape')
    expect(panel.querySelector('[data-testid="progress-shape"]')).toBeTruthy()
  })

  it('draws no verdict', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    const text = (panel.textContent ?? '').toLowerCase()
    for (const word of ['score', 'grade', 'excellent', 'very active', 'optimal', 'good', 'bad']) {
      expect(text).not.toContain(word)
    }
  })

  it('is drawn from the answers the router gave', async () => {
    // The three windows are three answers, and the chart draws each of
    // them: a window the router did not answer has no polygon.
    vi.spyOn(api, 'insightsRollingDays').mockImplementation(async (days: number) => {
      if (days === 60) return { ...rollingFixture, profile: [] }
      return rollingFixture
    })

    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() =>
      expect(panel.querySelector('[data-testid="progress-shape-window-30"]')).toBeTruthy(),
    )
    // The window with no facets has no shape of its own.
    expect(panel.querySelector('[data-testid="progress-shape-window-60"]')).toBeNull()
    expect(panel.querySelector('[data-testid="progress-shape-window-90"]')).toBeTruthy()
  })
})
