/**
 * Today: **the plan's options, with the plan's pacing**.
 *
 * The rules the test holds to:
 *
 *   - the day's sessions carry **the pace the plan was worked out with**;
 *   - the pacing is named **once**, above the sessions: the threshold
 *     and the day it was read;
 *   - a session with no pace says why, and shows none.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { planWeekFixture, todayFixture } from '../src/fixtures'
import type { AdoptedWeek } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The adopted plan's week, with one session on the day. */
const ADOPTED: AdoptedWeek = {
  week: 8,
  weeks_to_go: 3,
  label: '3 weeks to go',
  start_date: '2026-05-11',
  end_date: '2026-05-17',
  sessions: [
    {
      date: '2026-05-13',
      weekday: 'Wednesday',
      kind: 'tempo',
      method_kind: 'LT',
      distance: '17.7 km',
      pace: '4:16–4:21 /km',
      finish_pace: null,
      detail: null,
      label: '17.7 km · tempo',
      line: '17.7 km · tempo · 4:16–4:21 /km',
      no_pace: null,
      lt_last: null,
    },
    {
      date: '2026-05-15',
      weekday: 'Friday',
      kind: 'easy',
      method_kind: 'GA',
      distance: '8 km',
      pace: null,
      finish_pace: null,
      detail: null,
      label: '8 km · easy',
      line: null,
      no_pace: 'the method fixes a heart-rate band for this run and no pace',
      lt_last: null,
    },
  ],
  source: 'prescribed',
  source_line: 'the Pfitzinger half marathon tier 2 plan',
  pacing: {
    paced: true,
    threshold: {
      value: '165 bpm',
      date: '2026-05-10',
      date_label: '10 May 2026',
      source: 'watch_zone_model',
      base: 'pfitzinger',
    },
    rule: null,
    missing: null,
    threshold_line: 'paced with the threshold of 10 May 2026 · 165 bpm',
    rule_line: null,
  },
  entry: { requirement: null, plan_band: null, athlete: null },
  shift: null,
}

describe('today’s plan', () => {
  it('shows the day’s sessions with the plan’s pacing', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue({ ...planWeekFixture, adopted: ADOPTED })
    renderApp('/')

    await screen.findByTestId('today-table')

    // The pacing is named once, with the threshold and the day it was
    // read.
    const pacing = await screen.findByTestId('today-pacing')
    expect(pacing.textContent).toContain('165 bpm')
    expect(pacing.textContent).toContain('10 May 2026')
  })

  it('puts the pace beside the session it belongs to', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue({ ...planWeekFixture, adopted: ADOPTED })
    renderApp('/')

    await screen.findByTestId('today-table')

    // The day the plan has a tempo session for.
    const pace = await screen.findByTestId('today-pace-2026-05-13')
    expect(pace.textContent).toContain('4:16–4:21 /km')
  })

  it('shows no pacing when no plan is in force', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue({ ...planWeekFixture, adopted: null })
    renderApp('/')

    await screen.findByTestId('today-table')
    expect(screen.queryByTestId('today-pacing')).toBeNull()
  })

  it('keeps the day’s own sentence', async () => {
    vi.spyOn(api, 'insightsToday').mockResolvedValue(todayFixture)
    renderApp('/')

    await screen.findByTestId('today-session')
    expect(screen.getByTestId('today-session').textContent).toContain(todayFixture.planned!.sentence)
  })
})
