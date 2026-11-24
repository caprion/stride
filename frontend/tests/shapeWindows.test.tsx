/**
 * The athlete profile: three windows, superimposed.
 *
 * **One picture of the whole shape.** Thirty days, sixty days and ninety
 * days are drawn on one chart, one polygon each, and the axes are
 * **rates** — so the windows can be read against each other at all. The
 * totals of the three windows are the same in the fixture, and the
 * picture is not: that is the whole point of the rates.
 *
 * The picker's job has changed with it: the chart always shows all three
 * windows, and the picker says which of them the rest of the page is
 * about.
 */

import { fireEvent, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { renderApp } from './helpers'

/**
 * The chart asks for a size, and jsdom has no layout engine.
 *
 * The observer is stubbed with a size the drawing can be measured
 * against: the shape is drawn at 640 pixels wide.
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

/** The polygon of one window. */
function polygon(panel: HTMLElement, days: number): SVGPathElement | null {
  return panel.querySelector(`[data-testid="progress-shape-window-${days}"]`)
}

describe('the three windows on one chart', () => {
  it('draws one polygon for each window, each in a hue of its own', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() => expect(polygon(panel, 30)).toBeTruthy())

    const strokes = [30, 60, 90].map(
      (days) => polygon(panel, days)?.getAttribute('style') ?? '',
    )
    // Three windows, three colours: the reader can tell the shapes apart.
    expect(new Set(strokes).size).toBe(3)
    for (const stroke of strokes) expect(stroke).toContain('stroke')
  })

  it('draws the axes as rates, so the windows can be read against each other', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() => expect(polygon(panel, 30)).toBeTruthy())

    // The three windows hold the **same distance** — 58.4 km — so a chart
    // of totals would draw one shape and say nothing. The rates are
    // different, and the chart shows that: the thirty-day window has the
    // most distance a week, and the ninety-day one the least.
    const thirty = pointsOf(polygon(panel, 30))
    const sixty = pointsOf(polygon(panel, 60))
    const ninety = pointsOf(polygon(panel, 90))

    const thirtyVolume = radiusOf(thirty[0]!, CENTRE)
    const sixtyVolume = radiusOf(sixty[0]!, CENTRE)
    const ninetyVolume = radiusOf(ninety[0]!, CENTRE)

    expect(thirtyVolume).toBeGreaterThan(sixtyVolume)
    expect(sixtyVolume).toBeGreaterThan(ninetyVolume)
    // And in the ratio the rates are in: 13.6, 6.8 and 4.5 km a week.
    expect(sixtyVolume / thirtyVolume).toBeCloseTo(6.8 / 13.6, 1)
    expect(ninetyVolume / thirtyVolume).toBeCloseTo(4.5 / 13.6, 1)
  })

  it('draws the focused window last, so it is not buried', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() => expect(polygon(panel, 30)).toBeTruthy())

    const paths = [...panel.querySelectorAll('[data-testid^="progress-shape-window-"]')]
    // The default focus is the ninety-day window, and it is drawn last.
    expect(paths[paths.length - 1]?.getAttribute('data-testid')).toBe(
      'progress-shape-window-90',
    )
    // And it is drawn with a heavier line than the others.
    const focused = paths[paths.length - 1]?.getAttribute('stroke-width')
    const other = paths[0]?.getAttribute('stroke-width')
    expect(Number(focused)).toBeGreaterThan(Number(other))
  })

  it('says which window each polygon is, in the legend', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    const legend = panel.querySelector('[data-testid="progress-shape-legend"]')
    expect(legend?.textContent).toContain('last 30 days')
    expect(legend?.textContent).toContain('last 60 days')
    expect(legend?.textContent).toContain('last 90 days')
  })

  it('the picker chooses the window the page is about, and not the chart', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() => expect(polygon(panel, 30)).toBeTruthy())

    // The table is about the ninety-day window to begin with.
    expect(panel.textContent).toContain('The last 90 days')

    // The picker moves it to the thirty-day window.
    fireEvent.click(screen.getByTestId('profile-scope-30'))
    await waitFor(() => expect(panel.textContent).toContain('The last 30 days'))

    // And the chart still draws all three windows: the picture is the
    // whole, and the picker is the focus.
    expect(polygon(panel, 30)).toBeTruthy()
    expect(polygon(panel, 60)).toBeTruthy()
    expect(polygon(panel, 90)).toBeTruthy()
  })

  it('tells the reader which way each facet moved', async () => {
    renderApp('/profile')
    const panel = await screen.findByTestId('progress-shape')
    await waitFor(() => expect(polygon(panel, 30)).toBeTruthy())

    // The table compares the focused window with the window before it of
    // the same length, and says which way each facet moved.
    expect(panel.textContent).toContain('the 90 days before')
    expect(
      panel.querySelector('[data-testid="progress-shape-volume-movement"]')?.textContent,
    ).toBe('up')
    expect(
      panel.querySelector('[data-testid="progress-shape-intensity-movement"]')?.textContent,
    ).toBe('the same')
    expect(
      panel.querySelector('[data-testid="progress-shape-duration-movement"]')?.textContent,
    ).toBe('down')
  })
})
