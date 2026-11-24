/**
 * The wall: the desktop's Progress and the desktop's athlete profile.
 *
 * The wide Progress is **the wall**: eleven cards, each one a question
 * with its answer, and a doorway into the panel that answers it. The
 * athlete profile has a wall of its own, and the two walls hold the
 * model between them: six cards on Progress, five on the athlete
 * profile.
 *
 * The tests hold four things:
 *
 *   - **Every card is on one of the walls, once.** Six on Progress, five
 *     on the athlete profile, and a card is on neither or both.
 *   - **Every card opens its panel.** The "open →" in the card's head is
 *     the doorway into the panel that answers it, and the route is the
 *     model's.
 *   - **The cards are drawn in the model's order.**
 *   - **No new numbers.** Every value on the wall is the router's own
 *     rendering, and a panel with no value says so and shows no number.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture } from '../src/fixtures'
import { t } from '../src/i18n'
import { DESTINATIONS } from '../src/app/destinations'
import { WALL_CARDS, cardRoute, cardsOf, panelDestination, panelOfCard } from '../src/pages/taskAreas'
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

/** The wall of a page, once it is there. */
async function wallOf(container: HTMLElement): Promise<HTMLElement> {
  return waitFor(() => {
    const wall = container.querySelector('[data-testid="progress-wall"]')
    if (!wall) throw new Error('the wall is not there yet')
    return wall as HTMLElement
  })
}

describe('the wall on Progress', () => {
  it('has eleven cards, six on Progress and five on the athlete profile', () => {
    expect(WALL_CARDS).toHaveLength(11)
    expect(cardsOf('progress')).toHaveLength(6)
    expect(cardsOf('profile')).toHaveLength(5)
  })

  it('keeps a card and the panel it opens on the same destination', () => {
    // A card that opens a panel drags that panel to its own side of the
    // split, and the two never disagree.
    for (const card of WALL_CARDS) {
      const panel = panelOfCard(card.id)
      if (!panel) continue
      expect(panelDestination(panel), `${card.id} opens ${panel}`).toBe(card.destination)
    }
  })

  it('draws the cards of the model, in the model’s order', async () => {
    for (const destination of ['progress', 'profile'] as const) {
      const { container, unmount } = renderApp(`/${destination}`)
      const wall = await wallOf(container)

      const cards = [...wall.querySelectorAll('section[data-testid^="progress-wall-"]')].map(
        (card) => card.getAttribute('data-testid'),
      )
      expect(cards).toEqual(cardsOf(destination).map((card) => cardTestId(card.id)))
      unmount()
    }
  })

  it('names each card with the catalogue’s own words, and nothing else', async () => {
    for (const destination of ['progress', 'profile'] as const) {
      const { container, unmount } = renderApp(`/${destination}`)
      const wall = await wallOf(container)

      for (const card of cardsOf(destination)) {
        const node = wall.querySelector(`[data-testid="${cardTestId(card.id)}"]`)
        const heading = node?.querySelector('h2')

        // The heading is the card's own name, and nothing more: no
        // number, no summary, no verdict.
        expect(heading?.textContent).toBe(t(card.titleKey))
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

    // The wall's six cards, one row each.
    expect(index.querySelectorAll('a[data-testid^="progress-index-"]')).toHaveLength(6)
  })

  it('shows the values the router answered with', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)
    const card = wall.querySelector('[data-testid="progress-wall-months"]')

    // The core's own words: the months the answer holds, and the window.
    expect(card?.textContent).toContain('May 2026')
  })

  it('has no number where the answer has none', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        models: [],
        source: null,
        unavailable: 'the runs in the window carry no heart-rate samples',
        shares: null,
        distribution: null,
        weeks: [],
      },
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-wall-time')

    // The reason is the answer's own words, and no share is drawn.
    expect(card.textContent).toContain('no heart-rate samples')
    expect(card.textContent).not.toContain('%')
  })
})

/**
 * The athlete profile: a destination of its own, with its own wall,
 * its own panels and its own doorway.
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

  it('keeps the index of its own five cards', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    expect(index.querySelectorAll('a[data-testid^="profile-index-"]')).toHaveLength(5)
  })

  it('opens its panels under the profile', async () => {
    const { container } = renderApp('/profile')
    const wall = await wallOf(container)

    // Every card of the profile opens a panel of the profile.
    for (const card of cardsOf('profile')) {
      const link = wall.querySelector(`[data-testid="${openTestId(card.id)}"]`)
      expect(link?.getAttribute('href')).toBe(cardRoute(card.id))
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
    renderApp('/profile/shape')
    const card = await screen.findByTestId('progress-shape')

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
    renderApp('/profile/shape')
    const card = await screen.findByTestId('progress-shape')

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
    renderApp('/profile/shape')
    const card = await screen.findByTestId('progress-shape')

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
