/**
 * The narrow Progress: an index of the wall's cards.
 *
 * Six rows on Progress and five on the athlete profile, each one a card
 * of the wall with the athlete's answer in it, and a doorway into the
 * panel that answers it.
 *
 * The rules the rows answer to:
 *
 *   - the rows are the wall's cards, in the wall's order;
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
import { WALL_CARDS, cardRoute, cardsOf } from '../src/pages/taskAreas'
import { loadFixture, rollingFixture, streaksEmptyFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/**
 * The links of the index: the rows themselves.
 */
const QUESTION_LINKS = 'a[data-testid^="progress-index-"]'
const PROFILE_LINKS = 'a[data-testid^="profile-index-"]'

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
  it('asks the wall’s own six cards, in the wall’s order', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    const questions = [...index.querySelectorAll(QUESTION_LINKS)].map((node) =>
      node.getAttribute('data-testid'),
    )
    // The wall's cards, in the model's order: this week, the load, the
    // weeks, where the time went, the pace, and the bests.
    expect(questions).toEqual([
      'progress-index-week',
      'progress-index-load',
      'progress-index-weeks',
      'progress-index-time',
      'progress-index-pace',
      'progress-index-bests',
    ])

    expect(index.textContent).toContain(t('progress.wall.thisWeek.heading'))
    expect(index.textContent).toContain(t('progress.wall.load.heading'))
    expect(index.textContent).toContain(t('progress.wall.weeks.heading'))
    expect(index.textContent).toContain(t('progress.wall.time.heading'))
    expect(index.textContent).toContain(t('progress.wall.pace.heading'))
    expect(index.textContent).toContain(t('progress.wall.bests.heading'))
  })

  it('answers the week with the week and the difference', async () => {
    renderApp('/progress')
    // The core's own rendering: the distance and the difference.
    const text = await answer('week')
    expect(text).toContain('·')
    // And the window it covers.
    expect(await windowOf('week')).toContain('–')
  })

  it('answers the load with the load, as of the day it was read', async () => {
    renderApp('/progress')
    expect(await answer('load')).toBe('The last 7 days 19.7 · The difference -12.7')
    expect(await windowOf('load')).toBe('as of 13 May 2026')
  })

  it('answers the weeks with the distance and the runs', async () => {
    renderApp('/progress')
    expect(await answer('weeks')).toBe('58.4 km · 6 runs')
    expect(await windowOf('weeks')).toBe('23 Feb 2026 – 13 May 2026')
  })

  it('answers the pace with the pace at the heart rate', async () => {
    renderApp('/progress')
    // The core's own rendering of the pace and of the heart rate.
    expect(await answer('pace')).toBe('5:24 /km at 148 bpm')
    // And the week it belongs to.
    expect(await windowOf('pace')).toBe('The week of 4 May 2026')
  })

  it('answers the time with the split, or says why there is none', async () => {
    renderApp('/progress')
    // The fixture's runs carry no heart-rate samples, so the split has
    // no value: the row says so, in the core's own words, and shows no
    // number at all.
    expect(await answer('time')).toContain('none of the runs in this period recorded a heart rate')
  })

  it('answers the time with the split of the zones', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        models: rollingFixture.zones.models,
        source: rollingFixture.zones.source,
        unavailable: null,
        distribution: null,
        weeks: [],
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
    expect(await answer('time')).toBe(
      'Z1–Z2 low 6.28% · Z3 moderate 40.33% · Z4–Z5 high 53.39%',
    )
    expect(await windowOf('time')).toBe('The last 80 days')
  })

  it('answers the bests with the records', async () => {
    renderApp('/progress')
    expect(await answer('bests')).toContain('18.2 km')
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

describe('the index is the wall’s card list', () => {
  it('opens every card of the model from the two indexes', async () => {
    renderApp('/progress')
    renderApp('/profile')
    const index = await screen.findByTestId('progress-index')
    const profile = await screen.findByTestId('profile-index')

    // Every card of the wall has a doorway on the phone: the rows are
    // the cards, and the two indexes carry them between them.
    const hrefs = [
      ...[...index.querySelectorAll('a')].map((node) => node.getAttribute('href')),
      ...[...profile.querySelectorAll('a')].map((node) => node.getAttribute('href')),
    ]
    for (const card of WALL_CARDS) {
      const panel = cardRoute(card.id)
      if (panel.startsWith('/progress/') || panel.startsWith('/profile/')) {
        expect(hrefs).toContain(panel)
      }
    }
  })

  it('draws the cards of a destination in the model’s order', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    const rows = [...index.querySelectorAll('li > a[data-testid^="progress-index-"]')].map(
      (node) => node.getAttribute('data-testid'),
    )
    expect(rows).toEqual(cardsOf('progress').map((card) => `progress-index-${card.id}`))
  })

  it('names each row with the card’s own name', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    for (const card of cardsOf('progress')) {
      const row = index.querySelector(`[data-testid="progress-index-${card.id}"]`)
      expect(row?.textContent).toContain(t(card.titleKey))
    }
  })

  it('has no second index of the same cards', async () => {
    renderApp('/progress')
    const index = await screen.findByTestId('progress-index')

    // The areas' link lists are gone: the rows are the cards.
    expect(index.querySelector('nav[data-testid="progress-index-areas"]')).toBeNull()
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
    expect(await answer('load')).toContain("Your watch's zones have not come through")
  })

  it('says when there is no streak at all', async () => {
    vi.spyOn(api, 'insightsStreaks').mockResolvedValue(streaksEmptyFixture)
    renderApp('/profile')
    // A streak of zero days is no streak: the row says so.
    const node = await screen.findByTestId('profile-index-habit-answer')
    expect(node.textContent).toContain('no streak')
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
  it('opens the week panel on its own', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-week'))

    // The panel, and only the panel.
    const panel = await screen.findByTestId('progress-panel-week')
    expect(panel.querySelector('[data-testid="progress-wall-week"]')).toBeTruthy()
    // And the index is not there any more.
    expect(screen.queryByTestId('progress-index')).toBeNull()
  })

  it('opens the load panel on its own', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-load'))

    const panel = await screen.findByTestId('progress-panel-load')
    expect(panel.querySelector('[data-testid="progress-load"]')).toBeTruthy()
  })

  it('opens the pace and the time', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-pace'))
    let panel = await screen.findByTestId('progress-panel-pace')
    expect(panel.querySelector('[data-testid="progress-pace-window"]')).toBeTruthy()

    await userEvent.click(screen.getByTestId('progress-panel-back'))
    await userEvent.click(await screen.findByTestId('progress-index-time'))
    panel = await screen.findByTestId('progress-panel-zone-time')
    expect(panel.querySelector('[data-testid="progress-zones-distribution"]')).toBeTruthy()
  })

  it('goes back to the questions', async () => {
    renderApp('/progress')
    await userEvent.click(await screen.findByTestId('progress-index-load'))
    await screen.findByTestId('progress-panel-load')

    await userEvent.click(screen.getByTestId('progress-panel-back'))
    expect(await screen.findByTestId('progress-index')).toBeTruthy()
  })
})

/**
 * The narrow athlete profile: the profile's five cards.
 *
 * The same rules as Progress's index: every number is the answer to the
 * question, every row names the window it covers, and nothing on a row
 * is a verdict.
 */
describe('the index of the athlete profile', () => {
  it('asks the profile’s own five cards, in the model’s order', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    const questions = [...index.querySelectorAll(PROFILE_LINKS)].map((node) =>
      node.getAttribute('data-testid'),
    )
    expect(questions).toEqual([
      'profile-index-shape',
      'profile-index-months',
      'profile-index-schedule',
      'profile-index-habit',
      'profile-index-goals',
    ])
  })

  it('answers each one', async () => {
    renderApp('/profile')

    // The core's own numbers: the volume of the window, the months of
    // the history, the days of the week the runs fall on, the streak and
    // the Eddington number, and the weeks of the challenges.
    expect((await screen.findByTestId('profile-index-shape-answer')).textContent).toContain(
      '58.4 km',
    )
    expect((await screen.findByTestId('profile-index-months-answer')).textContent).toContain(
      'May 2026',
    )
    expect((await screen.findByTestId('profile-index-schedule-answer')).textContent).toContain(
      'Monday',
    )
    expect((await screen.findByTestId('profile-index-habit-answer')).textContent).toContain(
      '5 km on at least 5 days',
    )
    expect((await screen.findByTestId('profile-index-goals-answer')).textContent).toBe(
      '2 of 3 weeks',
    )
  })

  it('opens its panels under the profile', async () => {
    renderApp('/profile')
    const index = await screen.findByTestId('profile-index')

    const hrefs = [...index.querySelectorAll('a')].map((node) => node.getAttribute('href'))
    expect(hrefs).toContain('/profile/shape')
    expect(hrefs).toContain('/profile/months')
    expect(hrefs).toContain('/profile/weekday')
    expect(hrefs).toContain('/profile/habit')
    expect(hrefs).toContain('/profile/challenges')
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
