/**
 * The wall: the desktop's Progress and the desktop's athlete profile.
 *
 * The wide Progress is **the wall**: every panel the router answers, on
 * one screen, in the mock's cards — a name, the answer, and the way into
 * the panel that answers it. The athlete profile has a wall of its own,
 * and the two walls hold the model between them: the work's areas on
 * Progress, the long pattern's areas on the athlete profile.
 *
 * The tests hold four things:
 *
 *   - **The areas are drawn.** Each wall is the model's task areas of
 *     its own destination, in the model's order.
 *   - **Every card is on one of the walls, once.** No card is lost, none
 *     is twice, and the test ids are the ones the wall has always had.
 *   - **Every card opens its panel.** The "open →" in the card's head is
 *     the doorway the mock's drawer is, and the route is the model's.
 *   - **No new numbers.** Every value on the wall is the router's own
 *     rendering, and a panel with no value says so and shows no number.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { t } from '../src/i18n'
import { DESTINATIONS } from '../src/app/destinations'
import { TASK_AREAS, WALL_CARDS, areasOf, cardRoute, panelDestination, panelOfCard } from '../src/pages/taskAreas'
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
 * The links of an index that are not the areas' own links: the picture's
 * doorway, and the questions' rows.
 */
const QUESTION_LINKS = 'a[data-testid^="progress-index-"]:not([data-testid^="progress-index-area-"])'

/** The wall of a page, once it is there. */
async function wallOf(container: HTMLElement): Promise<HTMLElement> {
  return waitFor(() => {
    const wall = container.querySelector('[data-testid="progress-wall"]')
    if (!wall) throw new Error('the wall is not there yet')
    return wall as HTMLElement
  })
}

describe('the wall on Progress', () => {
  it('has every card of the model in exactly one area', () => {
    // The model's own rule: every card is in one area, and no card is
    // in two.
    const areas = new Map<string, string[]>()
    for (const area of TASK_AREAS) {
      for (const id of area.cards) {
        areas.set(id, [...(areas.get(id) ?? []), area.id])
      }
    }

    expect([...areas.keys()].sort()).toEqual(WALL_CARDS.map((card) => card.id).sort())
    for (const [id, owners] of areas) {
      expect(owners, `${id} is in one area`).toHaveLength(1)
    }
  })

  it('keeps a card and the panel it opens on the same destination', () => {
    // A card that opens a panel drags that panel to its own side of the
    // split, and the two never disagree.
    for (const area of TASK_AREAS) {
      for (const id of area.cards) {
        const panel = panelOfCard(id)
        if (!panel) continue
        expect(panelDestination(panel), `${id} opens ${panel}`).toBe(area.destination)
      }
    }
  })

  it('draws the areas of the model, in the model’s order', async () => {
    const { container } = renderApp('/progress')
    const wall = await wallOf(container)

    // The areas are the model's, and they are drawn in the order the
    // model gives them: what is happening now, then how it compares,
    // then the pace and the results, and the doors out.
    const areas = [...wall.querySelectorAll('section[data-testid^="progress-area-"]')]
    expect(areas.map((area) => area.getAttribute('data-testid'))).toEqual(
      areasOf('progress').map((area) => `progress-area-${area.id}`),
    )
  })

  it('draws the areas of the model on the athlete profile too', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)

    const areas = [...wall.querySelectorAll('section[data-testid^="progress-area-"]')]
    expect(areas.map((area) => area.getAttribute('data-testid'))).toEqual(
      areasOf('profile').map((area) => `progress-area-${area.id}`),
    )
  })

  it('names each area with the catalogue’s own words, and nothing else', async () => {
    for (const destination of ['progress', 'profile'] as const) {
      const { container, unmount } = renderApp(`/${destination}`)
      const wall = await wallOf(container)

      for (const area of TASK_AREAS.filter((entry) => entry.destination === destination)) {
        const node = wall.querySelector(`[data-testid="progress-area-${area.id}"]`)
        const heading = node?.querySelector('h2')

        // The heading is the area's name, and nothing more: no number, no
        // summary, no verdict.
        expect(heading?.textContent).toBe(t(area.titleKey))
      }
      unmount()
    }
  })

  it('has every card of the model, exactly once, across the two walls', async () => {
    const progress = renderApp('/progress')
    const profile = renderApp('/profile')
    const progressWall = await wallOf(progress.container)
    const profileWall = await wallOf(profile.container)

    for (const card of WALL_CARDS) {
      // The card is a section of its own, and it is on one of the two
      // walls once — not on both, and not on neither.
      const cards = [
        ...progressWall.querySelectorAll(`section[data-testid="${cardTestId(card.id)}"]`),
        ...profileWall.querySelectorAll(`section[data-testid="${cardTestId(card.id)}"]`),
      ]
      expect(cards).toHaveLength(1)
    }
  })

  it('draws the cards of an area in the model’s order', async () => {
    for (const destination of ['progress', 'profile'] as const) {
      const { container, unmount } = renderApp(`/${destination}`)
      const wall = await wallOf(container)

      for (const area of TASK_AREAS.filter((entry) => entry.destination === destination)) {
        const node = wall.querySelector(`[data-testid="progress-area-${area.id}"]`)
        const cards = [...(node?.querySelectorAll('section[data-testid^="progress-wall-"]') ?? [])]

        // The area holds the model's cards for that area, and it holds
        // them in the model's order.
        expect(cards.map((card) => card.getAttribute('data-testid'))).toEqual(
          area.cards.map((id) => cardTestId(id)),
        )
      }
      unmount()
    }
  })

  it('has a card for every panel, and each card opens it', async () => {
    const progress = renderApp('/progress')
    const profile = renderApp('/profile')
    const progressWall = await wallOf(progress.container)
    const profileWall = await wallOf(profile.container)

    for (const card of WALL_CARDS) {
      const link =
        progressWall.querySelector(`[data-testid="${openTestId(card.id)}"]`) ??
        profileWall.querySelector(`[data-testid="${openTestId(card.id)}"]`)

      // The doorway is where the model says it is.
      expect(link?.getAttribute('href')).toBe(cardRoute(card.id))
    }
  })

  it('keeps the index for the phone', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // The six questions. The athlete profile's picture is not here: it
    // is the profile's own, and the profile has an index of its own.
    expect(index.querySelectorAll(QUESTION_LINKS)).toHaveLength(6)
  })

  it('shows the values the router answered with', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)
    const card = wall.querySelector('[data-testid="progress-wall-year"]')

    // The core's own words: the months the answer holds, and the window.
    expect(card?.textContent).toContain('12 months')
    expect(card?.textContent).toContain('1 Jun 2025')
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
 * The athlete profile: a destination of its own, with its own wall, its
 * own panels and its own doorway.
 */
describe('the athlete profile', () => {
  it('is the fifth destination, and the nav says so', async () => {
    const { container } = renderApp('/profile')

    // The destination set is the app's, and the athlete profile is one
    // of them.
    const destinations = [
      ...new Set(
        [...container.querySelectorAll('[data-destination]')].map((node) =>
          node.getAttribute('data-destination'),
        ),
      ),
    ]
    expect(destinations.sort()).toEqual(['/', '/body', '/profile', '/progress', '/runs'])
    expect(DESTINATIONS).toHaveLength(5)

    // And it is named with the catalogue's own words.
    const link = container.querySelector('[data-destination="/profile"]')
    expect(link?.textContent).toBe(t('nav.profile'))
  })

  it('puts the athlete profile above the questions, as a picture', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    // The picture is the first thing on the narrow layout: it answers
    // "what does my training look like?" in one glance.
    const shape = index.querySelector('[data-testid="profile-index-shape"]')
    expect(shape).toBeTruthy()
    expect(shape?.querySelector('[data-testid="progress-shape-chart"]')).toBeTruthy()
    // And it opens the panel that has the numbers under it.
    expect(shape?.querySelector('[data-testid="profile-index-shape-open"]')?.getAttribute('href')).toBe(
      '/profile/shape',
    )
  })

  it('opens its panels under the profile', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)

    // Every card of the profile's areas opens a panel of the profile —
    // and the one card that opens a page of its own (the year, which is
    // the year as a film) says so.
    for (const area of areasOf('profile')) {
      for (const id of area.cards) {
        const link = wall.querySelector(`[data-testid="${openTestId(id)}"]`)
        const route = cardRoute(id)
        if (route.startsWith('/profile/')) {
          expect(link?.getAttribute('href')).toBe(route)
        } else {
          expect(route).toBe('/year')
        }
      }
    }
  })

  it('keeps the old /progress links working', async () => {
    renderApp('/progress/shape')

    // The panel is the athlete profile's own, and it is where the old
    // link lands.
    const panel = await screen.findByTestId('profile-panel-shape')
    expect(panel.querySelector('[data-testid="progress-shape"]')).toBeTruthy()

    // And the back link goes back to where the reader came from, and
    // says so in its own words.
    const back = screen.getByTestId('profile-panel-back')
    expect(back.getAttribute('href')).toBe('/progress')
    expect(back.textContent).toBe(t('progress.back'))
  })

  it('sends the reader back to the profile when the profile opened it', async () => {
    renderApp('/profile/shape')
    await screen.findByTestId('profile-panel-shape')

    const back = screen.getByTestId('profile-panel-back')
    expect(back.getAttribute('href')).toBe('/profile')
    expect(back.textContent).toBe(t('progress.profile.back'))
  })
})

/**
 * The athlete profile: three windows on one chart.
 *
 * The chart draws the three polygons and the table under it holds the
 * numbers behind them. Every number is the core's own rendering, and the
 * windows are named with the **answer's** count of days.
 */
describe('the athlete profile on its own wall', () => {
  it('draws the three windows and names them', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)
    const card = wall.querySelector('[data-testid="progress-wall-profile"]')

    // One polygon for each window: the last thirty days, the last sixty
    // and the last ninety, drawn one on the other.
    expect(card?.querySelector('[data-testid="progress-shape-window-30"]')).toBeTruthy()
    expect(card?.querySelector('[data-testid="progress-shape-window-60"]')).toBeTruthy()
    expect(card?.querySelector('[data-testid="progress-shape-window-90"]')).toBeTruthy()

    // The windows are named: the legend says which is which, and the
    // table says which one the numbers are about.
    expect(card?.textContent).toContain('last 30 days')
    expect(card?.textContent).toContain('last 60 days')
    expect(card?.textContent).toContain('last 90 days')
    // The focused window and the window before it are named with the
    // answer's own count of days, and they are the **same length**.
    expect(card?.textContent).toContain('The last 90 days')
    expect(card?.textContent).toContain('the 90 days before')
  })

  it("names the six axes in the core's own words", async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)
    const card = wall.querySelector('[data-testid="progress-wall-profile"]')

    // The six axes are named, each one in the core's own words.
    const labels = [...(card?.querySelectorAll('[data-testid="progress-shape-label"]') ?? [])].map(
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
    expect(card?.textContent).toContain('Sport variety')
    // The week days are not an axis: a week has no shape.
    expect(card?.querySelector('[data-testid="progress-shape-label-week_days"]')).toBeNull()
  })

  it('holds the values behind the shape', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)
    const card = wall.querySelector('[data-testid="progress-wall-profile"]')

    // The core's rendering of each facet, in the window's own units. The
    // volume and the consistency are **rates** — a distance a week and a
    // share of the days — so that the windows can be read against each
    // other.
    expect(
      card?.querySelector('[data-testid="progress-shape-volume-now"]')?.textContent,
    ).toBe('4.5 km a week')
    expect(
      card?.querySelector('[data-testid="progress-shape-intensity-now"]')?.textContent,
    ).toBe('148 bpm')
    expect(
      card?.querySelector('[data-testid="progress-shape-variety-now"]')?.textContent,
    ).toBe('1')
    // And the table says which way each facet moved against the window
    // before it.
    expect(
      card?.querySelector('[data-testid="progress-shape-volume-movement"]')?.textContent,
    ).toBe('up')
    expect(
      card?.querySelector('[data-testid="progress-shape-intensity-movement"]')?.textContent,
    ).toBe('the same')
  })
})
