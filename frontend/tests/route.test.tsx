/**
 * The route.
 *
 * The API sends the route as points — the run's own coordinates, in the
 * order the run went through them — and the app draws them.
 *
 * **On the run's own page the map is the flyby**: the track coloured by
 * the pace, the marker on the runner and the camera following, played
 * from the run's own samples. The app's own SVG map — the outline, with a
 * marker where the run started and a ring where it ended — is what the
 * feed and the cockpit draw, where a picture of the route is the whole
 * of what is wanted.
 */

import { waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('the route', () => {
  it("draws the flyby on the run's own page, and plays it", async () => {
    window.history.pushState({}, '', '/?fixtures=route')
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-flyby"]')).toBeTruthy(),
    )

    // The map is the flyby: the same map the film page draws.
    expect(container.querySelector('[data-testid="film-flyby"]')).toBeTruthy()
    // And it is played from the run's own page.
    expect(container.querySelector('[data-testid="activity-flyby-play"]')).toBeTruthy()
    // The film is still the deep view, and the run detail still opens it.
    expect(container.querySelector('[data-testid="activity-film"]')).toBeTruthy()
  })

  it('draws the run\'s outline in the feed, with a marker where it started and where it ended', async () => {
    window.history.pushState({}, '', '/?fixtures=route')
    const { container } = renderApp('/runs')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="run-route"]')).toBeTruthy(),
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
    expect(container.querySelector('[data-testid="activity-flyby"]')).toBeNull()
    expect(container.querySelector('[data-testid="route-map"]')).toBeNull()
  })

  it('shows a thumbnail in the feed, and the hero\'s route on Today', async () => {
    window.history.pushState({}, '', '/?fixtures=route')

    const runs = renderApp('/runs')
    await waitFor(() => expect(runs.container.querySelector('[data-testid="run-101"]')).toBeTruthy())
    expect(runs.container.querySelector('[data-testid="run-route"]')).toBeTruthy()

    const today = renderApp('/')
    await waitFor(() =>
      expect(today.container.querySelector('[data-testid="today-hero"]')).toBeTruthy(),
    )
    expect(today.container.querySelector('[data-testid="today-hero-route"]')).toBeTruthy()
  })
})
