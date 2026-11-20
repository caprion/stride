/**
 * The heatmap.
 *
 * The surface is behind an explicit action, it says in the app's own words
 * that the aggregation is made on the device, it shows an honest empty
 * state — and it asks nobody for anything: every request it makes is to
 * this app's own router, and nothing goes out to a third party.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { heatmapEmptyFixture, heatmapFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => {
  vi.restoreAllMocks()
  window.history.pushState({}, '', '/?fixtures=1')
})

/** A fetch that answers with the heatmap's own fixture. */
function stubFetch(body: unknown = heatmapFixture) {
  return vi.spyOn(globalThis, 'fetch').mockImplementation(async () => {
    return {
      ok: true,
      status: 200,
      text: async () => JSON.stringify(body),
      json: async () => body,
    } as unknown as Response
  })
}

/**
 * The heatmap, asked for over the app's own transport.
 *
 * The fixture mode answers from the fixture set instead of the network, so
 * the address is cleared first: the question here is what the screen asks
 * for and where it asks.
 */
function renderOverTheNetwork(body: unknown = heatmapFixture) {
  const requests = stubFetch(body)
  window.history.pushState({}, '', '/heatmap')
  const view = renderApp('/heatmap')
  return { ...view, requests }
}

describe('the heatmap', () => {
  it('is behind an explicit action on the Runs screen', async () => {
    renderApp('/runs?fixtures=1')

    const action = await screen.findByTestId('runs-heatmap')
    expect(action).toHaveAttribute('href', '/heatmap')
    // And the heatmap is not on the run list itself: the feed has no grid
    // on it.
    expect(screen.queryByTestId('heatmap-grid')).toBeNull()
  })

  it('says that the grid is made on this device and that nothing is sent', async () => {
    renderOverTheNetwork()

    const disclosure = await screen.findByTestId('heatmap-disclosure')
    expect(disclosure).toHaveTextContent('made on this device')
    expect(disclosure).toHaveTextContent('Nothing is sent anywhere')
  })

  it('draws the cells the answer carries, and counts them', async () => {
    const { container } = renderOverTheNetwork()

    await screen.findByTestId('heatmap-grid')
    const cells = container.querySelectorAll('[data-testid="heatmap-cell"]')
    expect(cells).toHaveLength(heatmapFixture.cells.length)
    // The busiest cell is the one the answer says it is.
    const busiest = Math.max(...heatmapFixture.cells.map((cell) => cell.runs))
    expect(
      [...cells].some(
        (cell) => Number(cell.getAttribute('data-runs')) === busiest,
      ),
    ).toBe(true)
    // And the summary names the runs and the cells.
    expect(screen.getByTestId('heatmap-summary')).toHaveTextContent('6 runs')
    expect(screen.getByTestId('heatmap-summary')).toHaveTextContent('5 cells')
  })

  it('shows an empty state when no run carries a position', async () => {
    renderOverTheNetwork(heatmapEmptyFixture)

    const empty = await screen.findByTestId('heatmap-empty')
    expect(empty).toHaveTextContent('No run has a saved route yet')
    // The disclosure is still there: the answer is empty, not the promise.
    expect(screen.getByTestId('heatmap-disclosure')).toHaveTextContent(
      'Nothing is sent anywhere',
    )
    // And there is no grid to draw.
    expect(screen.queryByTestId('heatmap-grid')).toBeNull()
  })

  it('asks for nothing outside this device', async () => {
    const { requests } = renderOverTheNetwork()

    await screen.findByTestId('heatmap-grid')
    await waitFor(() => expect(requests).toHaveBeenCalled())
    for (const call of requests.mock.calls) {
      const url = String(call[0])
      // Every request is to this app's own router, and it is relative: it
      // cannot be a request to anywhere else.
      expect(url.startsWith('/api/v1/')).toBe(true)
      expect(url).not.toMatch(/^https?:\/\//)
    }
  })

  it('asks the api for the grid, and the api answers with the cells', async () => {
    const { requests } = renderOverTheNetwork()

    await screen.findByTestId('heatmap-grid')
    const asked = requests.mock.calls.map((call) => String(call[0]))
    expect(asked).toContain('/api/v1/insights/heatmap')
  })
})
