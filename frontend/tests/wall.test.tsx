/**
 * The wall: the desktop's Progress.
 *
 * The wide Progress is **the wall**: every panel the router answers, on
 * one screen, in the mock's cards — a name, the answer, and the way into
 * the panel that answers it.
 *
 * The tests hold four things:
 *
 *   - **The areas are drawn.** The wall is the model's task areas, in
 *     the model's order, each with its own heading.
 *   - **Every card is on the wall, once.** No card is lost, none is
 *     twice, and the test ids are the ones the wall has always had.
 *   - **Every card opens its panel.** The "open →" in the card's head is
 *     the doorway the mock's drawer is, and the route is the model's.
 *   - **No new numbers.** Every value on the wall is the router's own
 *     rendering, and a panel with no value says so and shows no number.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { t } from '../src/i18n'
import { TASK_AREAS, WALL_CARDS, cardRoute } from '../src/pages/taskAreas'
import { renderApp } from './helpers'

/**
 * The chart asks for a size, and jsdom has no layout engine.
 *
 * The observer is stubbed with a size the drawing can be measured
 * against: the assertions are about what the drawing says rather than
 * how many pixels wide it is.
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

/** The test id of a card of the wall. */
function cardTestId(id: string): string {
  return `progress-wall-${id}`
}

/**
 * The test id of a card's "open →".
 *
 * The wall names its doorways after the card they open, and the pace has
 * always had a name of its own.
 */
const OPEN_TEST_IDS: Record<string, string> = { pace: 'progress-pace-link' }

function openTestId(id: string): string {
  return OPEN_TEST_IDS[id] ?? `progress-wall-open-${id}`
}

/**
 * The links of the index that are not the areas' own links: the picture's
 * doorway, and the six questions' rows.
 */
const QUESTION_LINKS = 'a[data-testid^="progress-index-"]:not([data-testid^="progress-index-area-"])'

describe('the wall on Progress', () => {
  it('draws the areas of the model, in the model’s order', async () => {
    renderApp('/progress')
    const wall = await screen.findByTestId('progress-wall')

    // The areas are the model's, and they are drawn in the order the
    // model gives them: what is happening now, then how it compares,
    // then the shape of the last few months, then the habits.
    const areas = [...wall.querySelectorAll('section[data-testid^="progress-area-"]')]
    expect(areas.map((area) => area.getAttribute('data-testid'))).toEqual(
      TASK_AREAS.map((area) => `progress-area-${area.id}`),
    )
  })

  it('names each area with the catalogue’s own words, and nothing else', async () => {
    renderApp('/progress')
    await screen.findByTestId('progress-wall')

    for (const area of TASK_AREAS) {
      const node = await screen.findByTestId(`progress-area-${area.id}`)
      const heading = node.querySelector('h2')

      // The heading is the area's name, and nothing more: no number, no
      // summary, no verdict.
      expect(heading?.textContent).toBe(t(area.titleKey))
    }
  })

  it('has every card of the wall, exactly once', async () => {
    renderApp('/progress')
    const wall = await screen.findByTestId('progress-wall')

    for (const card of WALL_CARDS) {
      // The card is a section of its own, and it is on the wall once.
      const cards = wall.querySelectorAll(`section[data-testid="${cardTestId(card.id)}"]`)
      expect(cards).toHaveLength(1)
    }
  })

  it('draws the cards of an area in the model’s order', async () => {
    renderApp('/progress')
    await screen.findByTestId('progress-wall')

    for (const area of TASK_AREAS) {
      const node = await screen.findByTestId(`progress-area-${area.id}`)
      const cards = [...node.querySelectorAll('section[data-testid^="progress-wall-"]')]

      // The area holds the model's cards for that area, and it holds
      // them in the model's order.
      expect(cards.map((card) => card.getAttribute('data-testid'))).toEqual(
        area.cards.map((id) => cardTestId(id)),
      )
    }
  })

  it('has a card for every panel, and each card opens it', async () => {
    renderApp('/progress')
    const wall = await screen.findByTestId('progress-wall')

    for (const card of WALL_CARDS) {
      const link = wall.querySelector(`[data-testid="${openTestId(card.id)}"]`)

      // The doorway is where the model says it is.
      expect(link?.getAttribute('href')).toBe(cardRoute(card.id))
    }
  })

  it('keeps the index for the phone', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // The six questions, and the picture above them.
    expect(index.querySelectorAll(QUESTION_LINKS)).toHaveLength(7)
  })

  it('puts the athlete profile above the questions, as a picture', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // The picture is the first thing on the narrow layout: it answers
    // "what does my training look like?" in one glance.
    const shape = index.querySelector('[data-testid="progress-index-shape"]')
    expect(shape).toBeTruthy()
    expect(shape?.querySelector('[data-testid="progress-shape-chart"]')).toBeTruthy()
    // And it opens the panel that has the numbers under it.
    expect(shape?.querySelector('[data-testid="progress-index-shape-open"]')?.getAttribute('href')).toBe(
      '/progress/shape',
    )
  })

  it('shows the values the router answered with', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-year')

    // The core's own words: the months the answer holds, and the window.
    expect(card.textContent).toContain('12 months')
    expect(card.textContent).toContain('1 Jun 2025')
  })

  it('has no number where the answer has none', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        model: null,
        unavailable: 'the runs in the window carry no heart-rate samples',
        shares: null,
        distribution: null,
      },
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-zones')

    // The reason is the answer's own words, and no share is drawn.
    expect(card.textContent).toContain('no heart-rate samples')
    expect(card.textContent).not.toContain('%')
  })
})

/**
 * The athlete profile: two windows on one chart.
 *
 * The radar draws the two polygons and the table under it holds the
 * numbers behind them. Every number is the core's own rendering, and the
 * two windows are named with the **answer's** count of days.
 */
describe('the athlete profile on Progress', () => {
  it('draws the two windows and names them', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    // One polygon for each window: the window, and the window before it.
    expect(card.querySelector('[data-testid="progress-shape-now"]')).toBeTruthy()
    expect(card.querySelector('[data-testid="progress-shape-before"]')).toBeTruthy()

    // The windows are named with the answer's own count of days, and
    // they are the **same length** — the window, and the window before
    // it. The card says what it answered.
    expect(card.textContent).toContain('The last 91 days')
    expect(card.textContent).toContain('the 91 days before')
  })

  it("names the six axes in the core's own words", async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    // The six axes are named, each one in the core's own words.
    const labels = [...card.querySelectorAll('[data-testid="progress-shape-label"]')].map(
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
    expect(card.textContent).toContain('Sport variety')
    // The week days are not an axis: a week has no shape.
    expect(card.querySelector('[data-testid="progress-shape-label-week_days"]')).toBeNull()
  })

  it('holds the values behind the shape', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-profile')

    // The core's rendering of each facet, in the window's own units.
    expect(
      card.querySelector('[data-testid="progress-shape-volume-now"]')?.textContent,
    ).toBe('58.4 km')
    expect(
      card.querySelector('[data-testid="progress-shape-intensity-now"]')?.textContent,
    ).toBe('148 bpm')
    expect(
      card.querySelector('[data-testid="progress-shape-variety-now"]')?.textContent,
    ).toBe('1')
  })
})
