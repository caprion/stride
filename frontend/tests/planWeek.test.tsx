/**
 * The plan's week: **the kinds of run, the distances and the paces**.
 *
 * The rules the test holds to:
 *
 *   - the week is read from the **adopted** plan — the dated copy — and
 *     its sessions carry their kinds, their distances and their paces;
 *   - **the paces carry the threshold they were worked out from**: the
 *     threshold and the day it was read stand above them;
 *   - a session with no pace says why, and shows no pace.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { planWeekFixture } from '../src/fixtures'
import type { AdoptedWeek, PlanWeek } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The adopted plan's week, with its paces. */
const ADOPTED: AdoptedWeek = {
  week: 8,
  weeks_to_go: 3,
  label: '3 weeks to go',
  start_date: '2026-05-11',
  end_date: '2026-05-17',
  sessions: [
    {
      date: '2026-05-12',
      weekday: 'Tuesday',
      kind: 'tempo',
      method_kind: 'LT',
      distance: '17.7 km',
      pace: '4:16–4:21 /km',
      finish_pace: null,
      detail: '2 × 20 min, jog 3',
      label: '17.7 km · tempo',
      line: '17.7 km · tempo · 4:16–4:21 /km',
      no_pace: null,
      lt_last: null,
    },
    {
      date: '2026-05-14',
      weekday: 'Thursday',
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
  entry: {
    requirement: 'this level asks for 64.4-80.5 km a week',
    plan_band: null,
    athlete: 'your last 2 weeks: 6.3, 52.1 km',
  },
  shift: null,
}

/** The plan's week, with the adopted plan in force. */
const WEEK: PlanWeek = {
  ...planWeekFixture,
  adopted: ADOPTED,
}

describe('the plan’s week', () => {
  it('shows the kinds of run, the distances and the paces', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(WEEK)
    renderApp('/plan')

    const card = await screen.findByTestId('plan-adopted-week')

    // The runner's wording, the method's own name beside it, and the
    // distance.
    expect(card.textContent).toContain('tempo')
    expect(card.textContent).toContain('LT')
    expect(card.textContent).toContain('17.7 km')
    // And the pace.
    expect(screen.getByTestId('plan-week-pace-2026-05-12').textContent).toContain('4:16–4:21 /km')
  })

  it('names the threshold the paces were worked out from', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(WEEK)
    renderApp('/plan')

    const card = await screen.findByTestId('plan-adopted-week')

    // The threshold and the day it was read stand above the paces.
    expect(card.querySelector('[data-testid="plan-week-paced-with"]')?.textContent).toContain(
      '165 bpm',
    )
    expect(card.querySelector('[data-testid="plan-week-paced-with"]')?.textContent).toContain(
      '10 May 2026',
    )
  })

  it('says why a session has no pace', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(WEEK)
    renderApp('/plan')

    const card = await screen.findByTestId('plan-adopted-week')

    // The general-aerobic day has no pace: the method fixes a band and
    // no pace, and the row says so.
    expect(screen.getByTestId('plan-week-no-pace-2026-05-14').textContent).toContain(
      'heart-rate band',
    )
    expect(card.querySelector('[data-testid="plan-week-pace-2026-05-14"]')).toBeNull()
  })

  it('puts the entry requirement beside the athlete’s own mileage', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(WEEK)
    renderApp('/plan')

    await screen.findByTestId('plan-adopted-week')

    expect(screen.getByTestId('plan-week-entry').textContent).toContain('this level asks for')
    expect(screen.getByTestId('plan-week-mileage').textContent).toContain('your last')
  })

  it('draws no adopted week when no plan is in force', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue({ ...planWeekFixture, adopted: null })
    renderApp('/plan')

    await screen.findByTestId('plan-week')
    expect(screen.queryByTestId('plan-adopted-week')).toBeNull()
  })
})
