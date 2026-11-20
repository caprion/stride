/**
 * The cockpit.
 *
 * Today is a cockpit: **the hero** (the latest run), **where you stand**
 * (three rows) and **what's on the table** (the plan's sessions and the
 * night's readings). These are the acceptance checks the item names, as
 * assertions rather than as a read-through:
 *
 *   - the hero shows the latest run;
 *   - the three rows carry the right numbers;
 *   - the empty states are honest — a day with no run, and a week with no
 *     runs;
 *   - the zone ribbon is drawn from the athlete's own zones;
 *   - and nothing on the screen is a verdict, a score or a command.
 *
 * The numbers are the api's and the core's: the app shows them as they
 * arrive, and these tests are how we know it does.
 */

import { screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { streaksFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => {
  vi.restoreAllMocks()
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('the hero', () => {
  it('shows the latest run: its name, its distance, its time, its pace, its heart rate and its elevation', async () => {
    renderApp('/')
    const hero = await screen.findByTestId('today-hero')

    expect(hero.textContent).toContain('The latest run')
    expect(hero.textContent).toContain('Run')
    expect(hero.textContent).toContain('12.4 km')
    expect(hero.textContent).toContain('1 h 7 m')
    expect(hero.textContent).toContain('5:24 /km')
    expect(hero.textContent).toContain('148 bpm')
    expect(hero.textContent).toContain('42 m')
    // The day the run happened: the hero is the *latest* run, not a run
    // of today.
    expect(hero.textContent).toContain('10 May 2026')
  })

  it('opens the run itself, until the film lands', async () => {
    const { container } = renderApp('/')
    await screen.findByTestId('today-hero')
    const action = container.querySelector('[data-testid="today-hero-open"]')
    expect(action?.getAttribute('href')).toBe('/runs/101')
  })

  it('draws the route the way the run detail draws it, with the app\'s own disclosure', async () => {
    window.history.pushState({}, '', '/?fixtures=route')
    const { container } = renderApp('/')
    await screen.findByTestId('today-hero')

    const route = container.querySelector('[data-testid="today-hero-route"]')
    expect(route).toBeTruthy()
    // The GPS outline is drawn by the app itself, and the Street map is
    // behind the app's own disclosure — the same two the run detail
    // offers.
    expect(container.querySelector('[data-testid="route-map"]')).toBeTruthy()
    expect(route?.textContent).toContain('OpenStreetMap')
  })
})

describe('the zone ribbon', () => {
  it('draws the athlete\'s own zones, with the core\'s words for the times', async () => {
    window.history.pushState({}, '', '/?fixtures=zones')
    const { container } = renderApp('/')
    await screen.findByTestId('today-hero')

    const ribbon = await screen.findByTestId('zone-ribbon')
    // The bands are the athlete's own model, and the times are the
    // core's rendering of them.
    expect(ribbon.textContent).toContain('Z1 6 m')
    expect(ribbon.textContent).toContain('Z3 18 m')
    // And the time outside the zones is said too.
    expect(container.textContent).toContain('Time outside your zones')
  })

  it('draws nothing when the run has no heart-rate samples', async () => {
    const { container } = renderApp('/')
    await screen.findByTestId('today-hero')
    // The seeded store holds no file, so the answer has no bands and the
    // app draws no ribbon rather than an empty one.
    expect(container.querySelector('[data-testid="zone-ribbon"]')).toBeNull()
  })
})

describe('where you stand', () => {
  it('carries this week, the form and the habit', async () => {
    const { container } = renderApp('/')
    await screen.findByTestId('today-standing')

    // This week: the distance, and the difference against the same days
    // last week.
    expect(container.querySelector('[data-testid="today-week-distance"]')?.textContent).toBe(
      '0 km',
    )
    expect(container.querySelector('[data-testid="today-week-against"]')?.textContent).toContain(
      '−21.1 km',
    )
    expect(container.querySelector('[data-testid="today-week-against"]')?.textContent).toContain(
      '4 May – 6 May',
    )

    // Form: the load's own state, with the ratio and the band.
    expect(container.querySelector('[data-testid="today-form-value"]')?.textContent).toBe('-12.7')
    expect(container.querySelector('[data-testid="today-form-ratio"]')?.textContent).toContain(
      'the acute-to-chronic ratio is 2.82',
    )
    expect(container.querySelector('[data-testid="today-form-band"]')?.textContent).toContain(
      'the band is 0.8 to 1.3',
    )

    // The habit: the streak, the months and the Eddington number.
    expect(container.querySelector('[data-testid="today-habit-streak"]')?.textContent).toBe(
      'no streak',
    )
    expect(container.querySelector('[data-testid="today-habit-months"]')?.textContent).toBe(
      '1 month in a row',
    )
    expect(container.querySelector('[data-testid="today-habit-eddington"]')?.textContent).toContain(
      'Eddington 5',
    )
  })

  it('shows the streak the api answered with, and the day it started', async () => {
    vi.spyOn(api, 'insightsStreaks').mockResolvedValue({
      ...streaksFixture,
      days: {
        current: {
          count: 12,
          start: '2026-05-01',
          start_label: '1 May 2026',
          end: null,
          end_label: null,
        },
        longest: streaksFixture.days.longest,
      },
    })
    const { container } = renderApp('/')
    await screen.findByTestId('today-standing')

    expect(container.querySelector('[data-testid="today-habit-streak"]')?.textContent).toBe(
      '12 days in a row',
    )
    expect(container.querySelector('[data-testid="today-habit-since"]')?.textContent).toBe(
      'since 1 May 2026',
    )
  })
})

describe('the empty states', () => {
  it('is honest about a day with no run: the hero is the latest run, and it says which day', async () => {
    renderApp('/')
    const hero = await screen.findByTestId('today-hero')
    // The last run was on the 10th and today is the 13th: the hero names
    // the day, and it does not pretend the run happened today.
    expect(hero.textContent).toContain('10 May 2026')
    expect(hero.textContent).not.toContain('13 May 2026')
  })

  it('is honest about a week with no runs: the distance is zero, and the difference says so', async () => {
    const { container } = renderApp('/')
    await screen.findByTestId('today-standing')

    // Nothing was run this week: the distance is zero — the distance
    // covered, which is a fact — and the difference against the same days
    // last week is the core's own.
    expect(container.querySelector('[data-testid="today-week-distance"]')?.textContent).toBe(
      '0 km',
    )
    expect(container.querySelector('[data-testid="today-week-against"]')?.textContent).toContain(
      '−21.1 km',
    )
    // And the seven days of the week are there, all of them without a
    // run: the recorded days, not an inferred habit.
    const days = container.querySelectorAll('[data-testid="today-week-days"] li')
    expect(days).toHaveLength(7)
  })

  it('says nothing at all when the store holds nothing', async () => {
    window.history.pushState({}, '', '/?fixtures=empty')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-empty"]')).toBeTruthy(),
    )
    expect(container.querySelector('[data-testid="today-hero"]')).toBeNull()
    expect(container.querySelector('[data-testid="today-standing"]')).toBeNull()
  })
})

describe('the voice of the cockpit', () => {
  it('is not a verdict, a score or a command', async () => {
    const { container } = renderApp('/')
    await screen.findByTestId('today-standing')
    const text = (container.textContent ?? '').toLowerCase()
    for (const word of ['ready', 'should', 'must', 'good', 'bad', 'missed', 'score', 'recommend']) {
      expect(text).not.toContain(word)
    }
  })
})
