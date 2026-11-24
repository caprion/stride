/**
 * The narrow Progress: an index of the questions.
 *
 * Seven rows, each one a question with the athlete's answer in it, and
 * a doorway into the panel that answers it.
 *
 * The rules the rows answer to:
 *
 *   - every number is the answer to that question, in the core's own
 *     rendering;
 *   - every row names the window its answer covers;
 *   - a row with no answer says why, in the core's words, and shows no
 *     number at all;
 *   - nothing on a row is a verdict.
 */

import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { t } from '../src/i18n'
import { TASK_AREAS, WALL_CARDS, areasOf, cardRoute } from '../src/pages/taskAreas'
import { loadFixture, rollingFixture, streaksEmptyFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/**
 * The links of the index that are not the areas' own links: the picture's
 * doorway, and the six questions' rows.
 */
const QUESTION_LINKS =
  'a[data-testid^="progress-index-"]:not([data-testid^="progress-index-area-"])'

/** The text of one row's answer. */
async function answer(id: string): Promise<string> {
  const node = await screen.findByTestId(`progress-index-${id}-answer`)
  return node.textContent ?? ''
}

/** The text of one row's window. */
async function windowOf(id: string): Promise<string> {
  const node = await screen.findByTestId(`progress-index-${id}-window`)
  return node.textContent ?? ''
}

describe('the index of the questions', () => {
  it('asks six questions, and each one is a question the app already asks', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    const questions = [...index.querySelectorAll(QUESTION_LINKS)].map((node) =>
      node.getAttribute('data-testid'),
    )
    // The six questions themselves. Each one is a panel the app already
    // has. The athlete profile's picture is not here: it is the
    // profile's own, and the profile has an index of its own.
    expect(questions).toEqual([
      'progress-index-window',
      'progress-index-load',
      'progress-index-year',
      'progress-index-pace',
      'progress-index-zones',
      'progress-index-habit',
    ])

    expect(index.textContent).toContain('This window against the window before')
    expect(index.textContent).toContain('The training load')
    expect(index.textContent).toContain('The year')
    expect(index.textContent).toContain('The pace')
    expect(index.textContent).toContain('Your zones')
    expect(index.textContent).toContain('The habit')
  })

  it('answers the window with the window and the difference', async () => {
    renderApp('/progress')
    // The core's own rendering: "58.4 km" and "+31 km".
    expect(await answer('window')).toBe('58.4 km · +31 km')
    // And the window it covers.
    expect(await windowOf('window')).toBe('The last 80 days · 23 Feb 2026 – 13 May 2026')
  })

  it('answers the load with the load, as of the day it was read', async () => {
    renderApp('/progress')
    expect(await answer('load')).toBe('Acute load 19.7 · Form -12.7')
    expect(await windowOf('load')).toBe('as of 13 May 2026')
  })

  it('answers the year with the year, and says how many months are recorded', async () => {
    renderApp('/progress')
    expect(await answer('year')).toBe('76.6 km · 7 runs')
    expect(await windowOf('year')).toBe('2026 · 2 months with runs')
  })

  it('answers the pace with the pace at the heart rate', async () => {
    renderApp('/progress')
    // The core's own rendering of the pace and of the heart rate.
    expect(await answer('pace')).toBe('5:24 /km at 148 bpm')
    // And the week it belongs to.
    expect(await windowOf('pace')).toBe('The week of 4 May 2026')
  })

  it('answers the zones with the split, or says why there is none', async () => {
    renderApp('/progress')
    // The fixture's runs carry no heart-rate samples, so the split has
    // no value: the row says so, in the core's own words, and shows no
    // number at all.
    expect(await answer('zones')).toContain('no heart-rate samples')
  })

  it('answers the zones with the split of the zones', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        models: rollingFixture.zones.models,
        source: rollingFixture.zones.source,
        unavailable: null,
        distribution: null,
        shares: [
          {
            key: 'low',
            label: 'Z1–Z2 low',
            share: { text: '6.28%', raw: 6.28 },
            previous_share: { text: '5.73%', raw: 5.73 },
            trend: 'up',
          },
          {
            key: 'moderate',
            label: 'Z3 moderate',
            share: { text: '40.33%', raw: 40.33 },
            previous_share: { text: '26.23%', raw: 26.23 },
            trend: 'up',
          },
          {
            key: 'high',
            label: 'Z4–Z5 high',
            share: { text: '53.39%', raw: 53.39 },
            previous_share: { text: '68.04%', raw: 68.04 },
            trend: 'down',
          },
        ],
      },
    })
    renderApp('/progress')
    expect(await answer('zones')).toBe(
      'Z1–Z2 low 6.28% · Z3 moderate 40.33% · Z4–Z5 high 53.39%',
    )
    expect(await windowOf('zones')).toBe('The last 80 days')
  })

  it('answers the habit with the streak and the Eddington number', async () => {
    renderApp('/progress')
    // The fixture holds no current streak of days, so the row says so
    // rather than showing a zero.
    expect(await answer('habit')).toBe('no streak · 2 weeks · Eddington 5')
    // And the window the Eddington number covers.
    expect(await windowOf('habit')).toBe('The Eddington number over 23 Feb 2026 – 13 May 2026')
  })

  it('draws no verdict on a row', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')
    const text = (index.textContent ?? '').toLowerCase()
    for (const word of ['good', 'bad', 'poor', 'should', 'must', 'optimal', 'consistent']) {
      expect(text).not.toContain(word)
    }
  })
})

describe('the areas of the index', () => {
  it('opens every panel of the model from the two indexes', async () => {
    renderApp('/progress')
    renderApp('/profile')
    const index = await screen.findByTestId('progress-index')
    const profile = await screen.findByTestId('profile-index')

    // Every card of the wall has a doorway on the phone now: the
    // questions' own rows count, the picture counts, and the areas carry
    // the rest — and the two indexes carry them between them.
    const hrefs = [
      ...[...index.querySelectorAll('a')].map((node) => node.getAttribute('href')),
      ...[...profile.querySelectorAll('a')].map((node) => node.getAttribute('href')),
    ]
    for (const area of TASK_AREAS) {
      for (const id of area.cards) {
        expect(hrefs).toContain(cardRoute(id))
      }
    }
  })

  it("draws the areas in the model's order, and leaves out the ones with nothing in them", async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // The panels the index opens already: the six questions' own panels.
    const opened = new Set(
      [...index.querySelectorAll(QUESTION_LINKS)].map((node) => node.getAttribute('href')),
    )

    const sections = [
      ...index.querySelectorAll('nav[data-testid="progress-index-areas"] > section'),
    ]
    const drawn = sections.map((section) => section.querySelector('h3')?.textContent ?? '')

    // The model's areas of this destination, in the model's order, minus
    // the ones that have nothing left to open.
    const expected = areasOf('progress')
      .filter((area) => area.cards.some((id) => !opened.has(cardRoute(id))))
      .map((area) => t(area.titleKey))

    expect(drawn).toEqual(expected)
  })

  it('gives every panel a link, under the name the card gives it', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    for (const section of index.querySelectorAll(
      'nav[data-testid="progress-index-areas"] > section',
    )) {
      const links = [...section.querySelectorAll('a')]
      // An area with nothing in it is not drawn, so a drawn one has a
      // link under it.
      expect(links.length).toBeGreaterThan(0)

      for (const link of links) {
        const id = link.getAttribute('data-testid')?.replace('progress-index-area-', '')
        const card = WALL_CARDS.find((entry) => entry.id === id)
        expect(card).toBeTruthy()
        // The panel's own name, and the card's own route.
        expect(link.textContent).toBe(t(card!.titleKey))
        expect(link.getAttribute('href')).toBe(cardRoute(card!.id))
      }
    }
  })

  it('carries no answers and no numbers in the areas', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')
    const areas = index.querySelector('nav[data-testid="progress-index-areas"]')

    // The questions carry the answers; the wall's cards carry the
    // numbers. The areas are a way in and nothing more.
    expect(areas?.querySelectorAll('[data-testid$="-answer"]').length).toBe(0)
    expect(areas?.querySelectorAll('[data-testid$="-window"]').length).toBe(0)
    expect(areas?.textContent ?? '').not.toMatch(/\d/)
  })
})

describe('the empty states of the index', () => {
  it('says why the load has no value, and shows no number', async () => {
    vi.spyOn(api, 'insightsLoad').mockResolvedValue({
      ...loadFixture,
      load: null,
      scenario: null,
      unavailable: 'no_zone_model',
    })
    renderApp('/progress')
    expect(await answer('load')).toContain('No zone model has been read')
  })

  it('says when there is no streak at all', async () => {
    vi.spyOn(api, 'insightsStreaks').mockResolvedValue(streaksEmptyFixture)
    renderApp('/progress')
    // A streak of zero days is no streak: the row says so.
    expect(await answer('habit')).toContain('no streak')
  })

  it('says so when there is nothing recorded at all', async () => {
    vi.spyOn(api, 'insightsProgress').mockResolvedValue({
      ...(await import('../src/fixtures')).progressFixture,
      weekly_volume: [],
    })
    const { container } = renderApp('/progress')
    await screen.findByTestId('progress-empty')
    // The index is not there: there is nothing to ask about.
    expect(container.querySelector('[data-testid="progress-index"]')).toBeNull()
  })
})

describe('opening a row', () => {
  it('opens the window panel on its own', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-window'))

    // The panel, and only the panel.
    const panel = await screen.findByTestId('progress-panel-window')
    expect(panel.querySelector('[data-testid="progress-rolling"]')).toBeTruthy()
    // And the index is not there any more.
    expect(screen.queryByTestId('progress-index')).toBeNull()
  })

  it('opens the load panel on its own', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-load'))

    const panel = await screen.findByTestId('progress-panel-load')
    expect(panel.querySelector('[data-testid="progress-load"]')).toBeTruthy()
  })

  it('opens the year, the pace, the zones and the habit', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-year'))
    // The year is a panel of the athlete profile, so it opens under the
    // profile — and the back link goes back to Progress, and says so.
    let panel = await screen.findByTestId('profile-panel-year')
    expect(panel.querySelector('[data-testid="progress-years"]')).toBeTruthy()

    await userEvent.click(screen.getByTestId('profile-panel-back'))
    await userEvent.click(await screen.findByTestId('progress-index-pace'))
    panel = await screen.findByTestId('progress-panel-pace')
    expect(panel.querySelector('[data-testid="progress-pace-window"]')).toBeTruthy()

    await userEvent.click(screen.getByTestId('progress-panel-back'))
    await userEvent.click(await screen.findByTestId('progress-index-zones'))
    panel = await screen.findByTestId('progress-panel-zones')
    expect(panel.querySelector('[data-testid="progress-zones"]')).toBeTruthy()

    await userEvent.click(screen.getByTestId('progress-panel-back'))
    await userEvent.click(await screen.findByTestId('progress-index-habit'))
    panel = await screen.findByTestId('profile-panel-habit')
    expect(panel.querySelector('[data-testid="progress-streaks"]')).toBeTruthy()
    expect(panel.querySelector('[data-testid="progress-eddington"]')).toBeTruthy()
  })

  it('goes back to the questions', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-window'))
    await screen.findByTestId('progress-panel-window')

    await userEvent.click(screen.getByTestId('progress-panel-back'))
    expect(await screen.findByTestId('progress-index')).toBeTruthy()
  })
})

/**
 * The narrow athlete profile: the picture, the questions, and the
 * areas' links.
 *
 * The same rules as Progress's index: every number is the answer to the
 * question, every row names the window it covers, and nothing on a row
 * is a verdict.
 */
describe('the index of the athlete profile', () => {
  const PROFILE_LINKS =
    'a[data-testid^="profile-index-"]:not([data-testid^="profile-index-area-"])'

  it('asks its own questions, and answers each one', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    // The picture, and the profile's own four questions.
    const questions = [...index.querySelectorAll(PROFILE_LINKS)].map((node) =>
      node.getAttribute('data-testid'),
    )
    expect(questions).toEqual([
      'profile-index-shape-open',
      'profile-index-goals',
      'profile-index-schedule',
      'profile-index-seasons',
      'profile-index-months',
    ])

    // The core's own numbers: the weeks of the challenges, the days of
    // the week the runs fall on, the months of the history and the
    // months in a row.
    expect((await screen.findByTestId('profile-index-goals-answer')).textContent).toBe(
      '2 of 3 weeks',
    )
    expect((await screen.findByTestId('profile-index-schedule-answer')).textContent).toContain(
      'Monday',
    )
    expect((await screen.findByTestId('profile-index-seasons-answer')).textContent).toContain(
      'May 2026',
    )
    expect((await screen.findByTestId('profile-index-months-answer')).textContent).toBe(
      '1 month in a row',
    )
  })

  it('opens its panels under the profile', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    const hrefs = [...index.querySelectorAll('a')].map((node) => node.getAttribute('href'))
    expect(hrefs).toContain('/profile/shape')
    expect(hrefs).toContain('/profile/challenges')
    expect(hrefs).toContain('/profile/weekday')
    expect(hrefs).toContain('/profile/months')
    expect(hrefs).toContain('/profile/habit')
    // And the areas carry the rest of the profile's panels.
    expect(hrefs).toContain('/profile/facets')
    expect(hrefs).toContain('/profile/heatmap')
    expect(hrefs).toContain('/profile/eddington')
  })

  it('opens a panel of the profile, and comes back to the profile', async () => {
    renderApp('/profile')
    await userEvent.click(await screen.findByTestId('profile-index-goals'))

    const panel = await screen.findByTestId('profile-panel-challenges')
    expect(panel.querySelector('[data-testid="challenge-grid"]')).toBeTruthy()

    // The back link goes back to the athlete profile, and says so.
    const back = screen.getByTestId('profile-panel-back')
    expect(back.getAttribute('href')).toBe('/profile')
    await userEvent.click(back)
    expect(await screen.findByTestId('profile-index')).toBeTruthy()
  })

  it('draws no verdict on a row', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')
    const text = (index.textContent ?? '').toLowerCase()
    for (const word of ['good', 'bad', 'poor', 'should', 'must', 'optimal', 'consistent']) {
      expect(text).not.toContain(word)
    }
  })
})
