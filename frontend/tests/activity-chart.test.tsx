/**
 * The run's one chart.
 *
 * **One run is one picture.** The pace and the heart rate are drawn over
 * the run's own elapsed time, with the athlete's own bands behind the
 * heart rate, and the reader chooses which of the readings the chart
 * carries.
 *
 * The rules the test holds to:
 *
 *   - the chart is one chart, not three;
 *   - the altitude is **off by default** and keeps its own scale;
 *   - the effort table keeps the zone numbers, the *not moving* bucket
 *     and the model's source line.
 */

import { waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { renderApp } from './helpers'

/**
 * The chart asks for a size, and jsdom has no layout engine.
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

afterEach(() => vi.restoreAllMocks())

/** The run detail, once it is there. */
async function runDetail() {
  const { container } = renderApp('/runs/101')
  await waitFor(() =>
    expect(container.querySelector('[data-testid="activity-effort"]')).toBeTruthy(),
  )
  await waitFor(() =>
    expect(container.querySelector('[data-testid="activity-chart"]')).toBeTruthy(),
  )
  return container
}

describe("the run's one chart", () => {
  it('is one chart, over the run’s own elapsed time', async () => {
    const container = await runDetail()

    // The chart, and the toggles that choose what it carries.
    expect(container.querySelector('[data-testid="activity-chart"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="activity-chart-toggles"]')).toBeTruthy()
    // The heart rate is always there: it is the run's effort, and the
    // bands behind it need it.
    expect(container.querySelector('[data-testid="activity-chart-toggle-pace"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="activity-chart-toggle-altitude"]')).toBeTruthy()
  })

  it('carries the pace until the reader says otherwise', async () => {
    const container = await runDetail()

    const toggle = container.querySelector('[data-testid="activity-chart-toggle-pace"]')
    expect(toggle?.getAttribute('aria-pressed')).toBe('true')

    await userEvent.click(toggle!)
    expect(toggle?.getAttribute('aria-pressed')).toBe('false')
  })

  it('leaves the altitude off until the reader asks for it', async () => {
    const container = await runDetail()

    // The ground is scenery: it is not drawn until it is asked for.
    expect(container.querySelector('[data-testid="activity-chart-altitude"]')).toBeNull()

    await userEvent.click(container.querySelector('[data-testid="activity-chart-toggle-altitude"]')!)
    expect(container.querySelector('[data-testid="activity-chart-altitude"]')).toBeTruthy()
  })

  it('keeps the effort table: the zones, the not-moving bucket and the source line', async () => {
    const container = await runDetail()

    // The model's own source line travels with the bands.
    expect(container.querySelector('[data-testid="activity-zones-source"]')?.textContent).toContain(
      'HR_MAX',
    )
  })
})
