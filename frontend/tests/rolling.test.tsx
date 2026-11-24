/**
 * The rolling view on Progress, and the streak on Today.
 *
 * The rolling card is a **comparison**: the window's numbers, the numbers
 * of the window before it, and the difference between them. It names the
 * window it describes, so a number is never read apart from the period
 * it covers.
 *
 * The streak is a count and a date. There is no verdict on either card:
 * the tests read the values, and they read what is *not* there too — a
 * split with no value shows no number at all, and a streak of zero days
 * is no streak.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { rollingFixture, streaksEmptyFixture, streaksFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

describe('the rolling window on Progress', () => {
  it('shows the window, the window before it and the difference', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-rolling')
    // The card is there before its numbers are: the window line is what
    // the test waits for.
    await screen.findByTestId('progress-rolling-window')

    // The core's own words, not a number the app formatted.
    expect(card.querySelector('[data-testid="progress-rolling-distance"]')?.textContent).toBe(
      '58.4 km',
    )
    expect(card.querySelector('[data-testid="progress-rolling-duration"]')?.textContent).toBe(
      '5 h 11 m',
    )
    expect(card.querySelector('[data-testid="progress-rolling-sessions"]')?.textContent).toBe('6')

    // The window before it, and the difference: a comparison, not a
    // verdict.
    expect(card.textContent).toContain('The 80 days before')
    expect(card.textContent).toContain('Difference')
  })

  it('names the window it describes', async () => {
    renderApp('/progress')
    await screen.findByTestId('progress-rolling')
    const window = (await screen.findByTestId('progress-rolling-window')).textContent

    // The api's own rendering of a date, and the length of the window.
    expect(window).toContain('23 Feb 2026')
    expect(window).toContain('13 May 2026')
    expect(window).toContain('80')
  })

  it('follows the range the screen is showing', async () => {
    const asked = vi.spyOn(api, 'insightsRolling')
    renderApp('/progress')
    await screen.findByTestId('progress-rolling')

    await userEvent.click(screen.getByTestId('progress-range-4'))
    await waitFor(() => expect(asked).toHaveBeenCalledWith(4))
  })
})

describe('the split of the zones on Progress', () => {
  it('has no value when the runs carry no heart-rate samples', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-zones')

    // The reason is the core's own words.
    expect(card.querySelector('[data-testid="progress-zones-empty"]')?.textContent).toContain(
      'no heart-rate samples',
    )
    // No value, not a zero: the card shows no share at all.
    expect(card.querySelector('[data-testid="progress-zones-shares"]')).toBeNull()
    expect(card.querySelector('[data-testid="progress-zones-low"]')).toBeNull()
  })

  it('names the model the split was measured against', async () => {
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
            share: { text: '72.53%', raw: 72.53 },
            previous_share: { text: '70.10%', raw: 70.1 },
            trend: 'up',
          },
        ],
      },
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-zones')

    // The share, and which way it moved against the window before it.
    expect(card.querySelector('[data-testid="progress-zones-low"]')?.textContent).toBe('72.53%')
    expect(card.textContent).toContain('up on the 80 days before')
    // The model, with the day it was read.
    expect(card.querySelector('[data-testid="progress-zones-model"]')?.textContent).toContain(
      'read on 10 May 2026',
    )
    // And no verdict anywhere on the card.
    const text = (card.textContent ?? '').toLowerCase()
    for (const word of ['optimal', 'imbalance', 'polarised', 'should', 'must']) {
      expect(text).not.toContain(word)
    }
  })
})

describe('the zone distribution on Progress', () => {
  it('has no value when the runs carry no heart-rate samples', async () => {
    renderApp('/progress')
    const card = await screen.findByTestId('progress-zones-distribution')

    // The reason is the core's own words.
    expect(
      card.querySelector('[data-testid="progress-zones-distribution-empty"]')?.textContent,
    ).toContain('no heart-rate samples')
    // No value, not a zero: there is no distribution at all.
    expect(card.querySelector('[data-testid="progress-zones-distribution-rows"]')).toBeNull()
  })

  it('shows the time in each zone, its share and the model', async () => {
    vi.spyOn(api, 'insightsRolling').mockResolvedValue({
      ...rollingFixture,
      zones: {
        models: rollingFixture.zones.models,
        source: rollingFixture.zones.source,
        unavailable: null,
        shares: rollingFixture.zones.shares,
        distribution: [
          {
            zone: 'Z1',
            range: '106-121 bpm',
            time: { text: '1 h 20 m', raw: 4800 },
            share: { text: '72.53%', raw: 72.53 },
          },
          {
            zone: 'Z2',
            range: '121-135 bpm',
            time: { text: '12 m', raw: 720 },
            share: { text: '10.81%', raw: 10.81 },
          },
        ],
      },
    })
    renderApp('/progress')
    const card = await screen.findByTestId('progress-zones-distribution')

    // The zone, its range, the time in it and its share.
    expect(card.textContent).toContain('Z1')
    expect(card.textContent).toContain('106-121 bpm')
    expect(card.textContent).toContain('1 h 20 m')
    expect(card.textContent).toContain('72.53%')
    // And the model the split was measured against.
    expect(card.querySelector('[data-testid="progress-zones-model"]')?.textContent).toContain(
      'read on 10 May 2026',
    )
  })
})

describe('the distance breakdown on the athlete profile', () => {
  it('buckets the runs by how far they went', async () => {
    renderApp('/profile')
    const card = await screen.findByTestId('progress-distances')

    // The buckets are the athlete's own scale: the longest run in the
    // fixture is 16.1 km, so the buckets are 4 km wide.
    expect(card.textContent).toContain('0-4 km')
    expect(card.textContent).toContain('16-20 km')
    // The run of 12.4 km is in the 12-16 bucket.
    expect(card.textContent).toContain('12.4 km')
    // And the card names the window it describes.
    // The profile's own scope: the athlete profile's cards are all read
    // over the same window, and not over the page's range.
    expect(card.querySelector('[data-testid="progress-distances-window"]')?.textContent).toContain(
      '90 days',
    )
  })
})

describe('the training profile on the athlete profile', () => {
  it('shows the seven facets as values, and no shape', async () => {
    renderApp('/profile')
    const card = await screen.findByTestId('progress-profile')

    // The facets, each with its own value.
    expect(card.querySelector('[data-testid="progress-profile-volume"]')?.textContent).toBe(
      '58.4 km',
    )
    expect(card.querySelector('[data-testid="progress-profile-consistency"]')?.textContent).toBe(
      '6',
    )
    expect(card.querySelector('[data-testid="progress-profile-intensity"]')?.textContent).toBe(
      '148 bpm',
    )
    expect(card.querySelector('[data-testid="progress-profile-duration"]')?.textContent).toBe(
      '49 m',
    )
    expect(card.querySelector('[data-testid="progress-profile-density"]')?.textContent).toBe('52 m')
    expect(card.querySelector('[data-testid="progress-profile-variety"]')?.textContent).toBe('1')
    expect(card.querySelector('[data-testid="progress-profile-week_days"]')?.textContent).toBe('5')
    // The facet names are the core's.
    expect(card.textContent).toContain('Sport variety')
    // And each says which way it moved against the window before it. The
    // window before the window is the 91 days that end the day the window
    // begins, so it carries its own runs: some facets moved up, some down,
    // and the ones that did not move say so.
    expect(card.textContent).toContain('up on the 90 days before')
    expect(card.textContent).toContain('down on the 90 days before')
    expect(card.textContent).toContain('the same as the 90 days before')
    // No polygon, no score, no grade.
    const text = (card.textContent ?? '').toLowerCase()
    for (const word of ['radar', 'profile score', 'shape score', 'excellent', 'very active']) {
      expect(text).not.toContain(word)
    }
  })
})

describe('the splits and the tables on the athlete profile', () => {
  it('places the runs on their weekday', async () => {
    renderApp('/profile')
    const card = await screen.findByTestId('progress-weekday')
    expect(card.textContent).toContain('Monday')
    // The run of the Monday and how far it went.
    expect(card.textContent).toContain('16.1 km')
  })

  it('names the hours of the daytime split', async () => {
    renderApp('/profile')
    const card = await screen.findByTestId('progress-daytime')
    expect(card.textContent).toContain('Morning')
    // The cut points, in the athlete's own local time.
    expect(card.textContent).toContain('06:00 – 12:00')
    expect(card.textContent).toContain('Night')
  })

  it('holds the months and the years that have runs', async () => {
    renderApp('/profile')
    const months = await screen.findByTestId('progress-months')
    expect(months.textContent).toContain('May 2026')
    expect(months.textContent).toContain('58.4 km')
    expect(months.textContent).toContain('February 2026')

    const years = await screen.findByTestId('progress-years')
    expect(years.textContent).toContain('2026')
    expect(years.textContent).toContain('76.6 km')
  })
})

describe('the streak on Today', () => {
  it('shows the count and the day it started', async () => {
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
    renderApp('/')
    const row = await screen.findByTestId('today-row-habit')

    // The streak that is still going: twelve days, and the day it
    // started.
    expect(row.querySelector('[data-testid="today-habit-streak"]')?.textContent).toBe(
      '12 days in a row',
    )
    expect(row.textContent).toContain('since 1 May 2026')
    // And the months: the run of months the habit has run.
    expect(row.querySelector('[data-testid="today-habit-months"]')?.textContent).toBe(
      '1 month in a row',
    )
  })

  it('has no streak when the history holds none', async () => {
    vi.spyOn(api, 'insightsStreaks').mockResolvedValue(streaksEmptyFixture)
    renderApp('/')
    const row = await screen.findByTestId('today-row-habit')

    // A streak of zero days is no streak: the row says so and shows no
    // number.
    expect(row.querySelector('[data-testid="today-habit-streak"]')?.textContent).toBe('no streak')
    expect(row.querySelector('[data-testid="today-habit-months"]')?.textContent).toBe(
      'no months in a row',
    )
    // And no verdict: a streak is a count and a date.
    const text = (row.textContent ?? '').toLowerCase()
    for (const word of ['consistent', 'good', 'bad', 'should']) {
      expect(text).not.toContain(word)
    }
  })

  it('shows the streak the router answered with', async () => {
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
        longest: {
          count: 9,
          start: '2026-04-20',
          start_label: '20 Apr 2026',
          end: '2026-04-28',
          end_label: '28 Apr 2026',
        },
      },
    })
    renderApp('/')
    const row = await screen.findByTestId('today-row-habit')

    expect(row.querySelector('[data-testid="today-habit-streak"]')?.textContent).toBe(
      '12 days in a row',
    )
    expect(row.textContent).toContain('since 1 May 2026')
  })
})
