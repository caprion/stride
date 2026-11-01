/**
 * The screens, rendered.
 *
 * These are the acceptance checks the plan names, as assertions rather
 * than as a read-through: the same values on Today and on Body, the
 * distance comparison appearing on exactly one fixture, and the swap
 * flipping every delta.
 *
 * The sentences they assert are the core's (core-contract §8): the app
 * shows them as they arrive, and these tests are how we know it does.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

function vitalReading(container: HTMLElement, key: string) {
  const row = container.querySelector(`[data-vital="${key}"]`)
  return {
    value: row?.querySelector(`[data-vital-value="${key}"]`)?.textContent?.trim(),
    direction: row?.querySelector('[data-direction]')?.textContent?.trim(),
  }
}

describe('Today', () => {
  it('leads with the vitals: value, direction, and the muted arithmetic', async () => {
    const { container } = renderApp('/')
    await waitFor(() => expect(container.querySelector('[data-vital="sleep"]')).toBeTruthy())

    expect(vitalReading(container, 'sleep')).toEqual({
      value: '6 h 10 m',
      direction: 'could be better',
    })
    expect(vitalReading(container, 'hrv')).toEqual({
      value: '48 ms',
      direction: 'could be better',
    })
    expect(vitalReading(container, 'resting_hr')).toEqual({
      value: '63 bpm',
      direction: 'about usual',
    })

    expect(container.textContent).toContain('1 h 10 m less than your usual')
    expect(container.textContent).toContain('14 below your usual')
    expect(container.textContent).toContain('5 above your usual')
    expect(container.textContent).toContain('6th night down')
    expect(container.textContent).toContain('3rd night down')
  })

  it('summarises the state as a count and a direction', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-state-line"]')).toBeTruthy(),
    )
    const line = container.querySelector('[data-testid="today-state-line"]')?.textContent
    expect(line).toContain('2 of 3 signals below your usual')
  })

  it('shows the plan’s session and its other options, unranked', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-session"]')).toBeTruthy(),
    )
    expect(container.textContent).toContain('Today — 10 km · easy')
    expect(container.textContent).toContain('from your imported plan')
    expect(container.textContent).toContain('Also in your plan')
  })

  it('says nothing that links an option to a vital', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-session"]')).toBeTruthy(),
    )
    const options = container.querySelector('[data-testid="today-options"]')?.textContent ?? ''
    expect(options.toLowerCase()).not.toContain('sleep')
    expect(options.toLowerCase()).not.toContain('hrv')
  })
})

describe('the distance comparison', () => {
  it('is absent within 10% of the target', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-session"]')).toBeTruthy(),
    )
    expect(container.querySelector('[data-testid="today-distance-comparison"]')).toBeNull()
  })

  it('appears, neutrally, beyond 10% of the target', async () => {
    window.history.pushState({}, '', '/?fixtures=over')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-distance-comparison"]')).toBeTruthy(),
    )
    const line = container.querySelector(
      '[data-testid="today-distance-comparison"]',
    )?.textContent
    expect(line).toContain('2.4 km more than the 10 km target')
    window.history.pushState({}, '', '/?fixtures=1')
  })
})

describe('Body', () => {
  it('renders the same values and directions as Today', async () => {
    const today = renderApp('/')
    await waitFor(() =>
      expect(today.container.querySelector('[data-vital="sleep"]')).toBeTruthy(),
    )
    const body = renderApp('/body')
    await waitFor(() =>
      expect(body.container.querySelector('[data-vital="sleep"]')).toBeTruthy(),
    )

    for (const key of ['sleep', 'hrv', 'resting_hr']) {
      expect(vitalReading(body.container, key)).toEqual(vitalReading(today.container, key))
    }
  })

  it('carries the patterns with their sample size', async () => {
    const { container } = renderApp('/body')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="body-patterns"]')).toBeTruthy(),
    )
    const patterns = container.querySelector('[data-testid="body-patterns"]')?.textContent ?? ''
    expect(patterns).toContain('your HRV has been below your usual for 3 nights')
    expect(patterns).toContain('the 5 other times')
  })
})

describe('Compare', () => {
  it('shows the metric diff, and the basis', async () => {
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )
    expect(container.querySelector('[data-testid="compare-basis"]')?.textContent).toContain(
      'Same distance, 3 weeks apart',
    )
    const pace = container.querySelector('[data-testid="compare-row-pace"]')?.textContent
    expect(pace).toContain('Pace')
    expect(container.textContent).toContain('Difference')
  })

  it('asks the API the other way round when A and B are swapped', async () => {
    const user = userEvent.setup()
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-row-pace"]')).toBeTruthy(),
    )

    // The delta's sign is the core's arithmetic, so the app cannot flip
    // it: it asks the API for the other order, and shows the answer.
    const before =
      container.querySelector('[data-testid="compare-row-pace"]')?.textContent ?? ''
    expect(before).toContain('-11 s/km')

    await user.click(screen.getByTestId('compare-swap'))

    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-row-pace"]')?.textContent).toContain(
        '+11 s/km',
      ),
    )
  })

  it('shows the values the API sent, word for word', async () => {
    const { container } = renderApp('/runs')
    await waitFor(() => expect(container.querySelector('[data-testid="run-101"]')).toBeTruthy())

    // "8.4 km", "42 m", "148 bpm", "5:24 /km" — the core's words, not the
    // app's arithmetic.
    const row = container.querySelector('[data-testid="run-101"]')?.textContent ?? ''
    expect(row).toContain('12.4 km')
    expect(row).toContain('1 h 7 m')
    expect(row).toContain('148 bpm')
    expect(row).toContain('5:24 /km')
  })
})

describe('the shell', () => {
  it('offers the same four destinations at every width', async () => {
    const { container } = renderApp('/')
    await waitFor(() => expect(container.querySelector('[data-nav="tabbar"]')).toBeTruthy())
    const destinations = new Set(
      [...container.querySelectorAll('[data-destination]')].map((node) =>
        node.getAttribute('data-destination'),
      ),
    )
    expect([...destinations].sort()).toEqual(['/', '/body', '/progress', '/runs'])
  })
})
