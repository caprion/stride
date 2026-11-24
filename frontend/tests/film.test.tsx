/**
 * The run film.
 *
 * The surface is reached by an **explicit action** on the run detail, it
 * draws the run's own series, the playhead moves over it and the readout
 * follows it — and a run with no file says so instead of drawing nothing.
 *
 * The fixture the film is reviewed in is the **`film` variant**: the seeded
 * store holds no FIT file, so the router's own answer for it is the honest
 * empty one, and the film with a series is the fixture set's own.
 */

import { fireEvent, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

/**
 * The chart asks for a size, and jsdom has no layout engine.
 *
 * The observer is stubbed with a size the drawing can be measured against:
 * the film is drawn at 640 pixels wide, and the assertions are about what
 * the drawing says rather than how many pixels wide it is.
 */
class SizedResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}

  observe(target: Element) {
    this.callback(
      [
        {
          target,
          contentRect: { width: 640, height: 300 } as DOMRectReadOnly,
        } as ResizeObserverEntry,
      ],
      this as unknown as ResizeObserver,
    )
  }

  unobserve() {}

  disconnect() {}
}

globalThis.ResizeObserver = SizedResizeObserver as never

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
})

/** The film, in the state it is reviewed in. */
function renderFilm() {
  window.history.pushState({}, '', '/?fixtures=film')
  return renderApp('/runs/101/film')
}

describe('the run film', () => {
  it('is behind an explicit action on the run detail', async () => {
    renderApp('/runs/101')

    const action = await screen.findByTestId('activity-film')
    expect(action).toHaveAttribute('href', '/runs/101/film')
    // And the film is not on the run list itself.
    expect(screen.queryByTestId('film-svg')).toBeNull()
  })

  it('draws the run and says where you are', async () => {
    renderFilm()

    await screen.findByTestId('film-svg')
    const readout = screen.getByTestId('film-readout')
    // The six numbers the readout carries, in the app's own words.
    expect(readout).toHaveTextContent('The whole run')
    expect(readout).toHaveTextContent('Distance')
    expect(readout).toHaveTextContent('Pace')
    expect(readout).toHaveTextContent('Heart rate')
    expect(readout).toHaveTextContent('Ground')
    expect(readout).toHaveTextContent('Right now')
    // And the values are the core's own: "5:39 /km", "132 bpm", "118 m".
    expect(readout).toHaveTextContent('5:39 /km')
    expect(readout).toHaveTextContent('132 bpm')
    expect(readout).toHaveTextContent('118 m')
  })

  it('the playhead moves and the readout follows it', async () => {
    renderFilm()

    await screen.findByTestId('film-svg')
    const readout = screen.getByTestId('film-readout')
    expect(readout).toHaveTextContent('0 m')

    // The slider moves over the run's own samples: the twentieth is the
    // twentieth minute.
    fireEvent.change(screen.getByTestId('film-scrub'), { target: { value: '20' } })

    await waitFor(() => expect(readout).toHaveTextContent('20 m'))
    // The distance at that moment is the core's own rendering, and the
    // zone is the model's own label.
    expect(readout).toHaveTextContent('3.6 km')
    expect(readout).toHaveTextContent('Z3')
  })

  it('plays the run at sixty times its own time', async () => {
    renderFilm()

    const play = await screen.findByTestId('film-play')
    expect(play).toHaveTextContent('Play')
    fireEvent.click(play)
    await waitFor(() => expect(screen.getByTestId('film-play')).toHaveTextContent('Pause'))
  })

  it('a split is a place you can jump to', async () => {
    renderFilm()

    const split = await screen.findByTestId('film-split-2')
    fireEvent.click(split)
    await waitFor(() =>
      expect(screen.getByTestId('film-readout')).toHaveTextContent('18 m'),
    )
  })

  it('names the zones the run was measured against', async () => {
    renderFilm()

    const zones = await screen.findByTestId('film-zones')
    expect(zones).toHaveTextContent('Z1')
    expect(zones).toHaveTextContent('Z5')
    expect(zones).toHaveTextContent('106-121 bpm')
    // And the model's own source line, with the day it was read.
    expect(screen.getByTestId('film-zones-source')).toHaveTextContent('taken from your watch on 10 May 2026')
  })

  it('says that the street tiles come from OpenStreetMap', async () => {
    renderFilm()

    await screen.findByTestId('film')
    // The disclosure is the app's own wording, the same one the run
    // detail's Street map carries.
    const disclosures = screen.getAllByText(/OpenStreetMap/)
    expect(disclosures.length).toBeGreaterThan(0)
    expect(disclosures.some((node) => node.textContent?.includes('reveal the area'))).toBe(true)
  })

  it('says so when the run has no file to read', async () => {
    // The seeded store holds no FIT file, so the router's own answer for
    // the film is the honest one.
    renderApp('/runs/101/film')

    const empty = await screen.findByTestId('film-empty')
    expect(empty).toHaveTextContent('the run has no file to read')
    // And there is no chart to draw.
    expect(screen.queryByTestId('film-svg')).toBeNull()
  })
})
