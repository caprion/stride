/**
 * The route.
 *
 * The API sends the route as points — the run's own coordinates, in the
 * order the run went through them — and the app draws them: the map on
 * the run's own page, a thumbnail in the feed at the wide widths, and a
 * mini thumbnail on Today where there is room for it.
 *
 * **No map library, no tiles, no server.** The route never leaves the
 * device, so the map does not either: the drawing is made here, from the
 * points the API sent.
 */

import { waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('the route', () => {
  it('draws the run\'s route, with a marker where it started and where it ended', async () => {
    window.history.pushState({}, '', '/?fixtures=route')
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-route"]')).toBeTruthy(),
    )

    // The drawing is the app's own SVG, and the two markers are in it.
    const map = container.querySelector('[data-testid="route-map"] svg')
    expect(map).toBeTruthy()
    expect(container.querySelector('[data-testid="route-start"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="route-end"]')).toBeTruthy()
    // And it is described for anyone who cannot see it.
    expect(map?.getAttribute('aria-label')).toContain('from the start to the finish')
  })

  it('draws nothing when the api sends no points', async () => {
    // The seeded store holds no FIT file, so the router answers no
    // points — and the app draws nothing rather than inventing a route.
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-detail"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="activity-route"]')).toBeNull()
    expect(container.querySelector('[data-testid="route-map"]')).toBeNull()
  })

  it('shows a thumbnail in the feed, and a mini one on Today', async () => {
    window.history.pushState({}, '', '/?fixtures=route')

    const runs = renderApp('/runs')
    await waitFor(() => expect(runs.container.querySelector('[data-testid="run-101"]')).toBeTruthy())
    expect(runs.container.querySelector('[data-testid="run-route"]')).toBeTruthy()

    const today = renderApp('/')
    await waitFor(() =>
      expect(today.container.querySelector('[data-testid="today-last-run"]')).toBeTruthy(),
    )
    expect(today.container.querySelector('[data-testid="today-route"]')).toBeTruthy()
  })
})
