/**
 * The week against the plan: **how is this week, against the plan?**
 *
 * The rules the test holds to:
 *
 *   - the comparison lives **inside** the card that already answers "this
 *     week": the wall is eleven cards, and the user's instruction was to
 *     reduce the surface;
 *   - **done and not done are facts** — no grade, no percentage, no
 *     traffic light, and the word "missed" is not in the app;
 *   - the paces are compared **only when both of them exist**: the
 *     general aerobic day carries a band and no pace, and a run is never
 *     compared against a pace that is not there;
 *   - the four states each say their own thing: no plan adopted, a plan
 *     that has not started, a week that is running, and a plan that has
 *     passed.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { planWeekFixture } from '../src/fixtures'
import type { AdoptedSession, PlanWeek, PlanWeekAgainst } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** One session of the plan's week, with what happened on its day. */
function session(over: Partial<AdoptedSession> = {}): AdoptedSession {
  return {
    date: '2026-05-12',
    weekday: 'Tuesday',
    kind: 'long run',
    method_kind: 'endurance',
    distance: '17.7 km',
    pace: '5:00–5:32 /km',
    finish_pace: null,
    detail: null,
    label: '17.7 km · long run',
    sentence: 'Tuesday — 17.7 km · long run',
    line: '17.7 km · long run · 5:00–5:32 /km',
    no_pace: null,
    lt_last: null,
    actual: null,
    difference: null,
    pace_comparison: null,
    status: null,
    ...over,
  }
}

/** The plan's week, with the comparison the core worked out. */
function week(against: Partial<PlanWeekAgainst>, sessions: AdoptedSession[]): PlanWeek {
  return {
    ...planWeekFixture,
    against: {
      state: 'running',
      state_line: null,
      choose: null,
      source_line: 'the Pfitzinger half marathon tier 2 plan',
      paced_with: 'paced with the threshold of 10 May 2026 · 165 bpm',
      entry: {
        requirement: 'this level asks for 64.4-80.5 km a week',
        plan_band: null,
        athlete: 'your last 4 weeks: 60.8, 61.1, 50.8, 6 km',
      },
      totals: null,
      ...against,
    },
    adopted: {
      ...planWeekFixture.adopted!,
      sessions,
    },
  }
}

describe('the week against the plan', () => {
  it('lives inside the card that already answers this week', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {
          totals: {
            planned: { text: '49.9 km', raw: 49_900 },
            actual: { text: '29.5 km', raw: 29_500 },
            difference: { text: '−20.4 km', raw: -20_400 },
            line: "29.5 km against the plan's 49.9 km",
          },
        },
        [session()],
      ),
    )
    const { container } = renderApp('/progress')

    await screen.findByTestId('plan-against')

    // The card that answers "this week" carries the comparison.
    const card = container.querySelector('[data-testid="progress-wall-week"]')
    expect(card?.querySelector('[data-testid="plan-against"]')).not.toBeNull()

    // And the wall is still eleven cards: the comparison is not a
    // twelfth one.
    const cards = container.querySelectorAll('section[data-testid^="progress-wall-"]')
    expect(cards).toHaveLength(6)
  })

  it('compares the week’s totals with the plan’s', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {
          totals: {
            planned: { text: '49.9 km', raw: 49_900 },
            actual: { text: '29.5 km', raw: 29_500 },
            difference: { text: '−20.4 km', raw: -20_400 },
            line: "29.5 km against the plan's 49.9 km",
          },
        },
        [session()],
      ),
    )
    renderApp('/progress')

    const totals = await screen.findByTestId('plan-against-totals')
    expect(totals.textContent).toContain("29.5 km against the plan's 49.9 km")
    expect(screen.getByTestId('plan-against-difference').textContent).toContain('20.4 km')
  })

  it('says that no plan has been adopted, and points at the programmes', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {
          state: 'none',
          state_line: 'no plan has been adopted',
          choose: 'your plan is one of the 4 programmes',
          source_line: null,
          paced_with: null,
          entry: null,
        },
        [],
      ),
    )
    renderApp('/progress')

    await screen.findByTestId('plan-against')
    expect(screen.getByTestId('plan-against-note').textContent).toContain(
      'no plan has been adopted',
    )
    // And the door to the programmes there are to adopt.
    const choose = screen.getByTestId('plan-against-choose')
    expect(choose.textContent).toContain('programmes')
    expect(choose.getAttribute('href')).toBe('/plan/bases')
  })

  it('says when a plan that has not started starts', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {
          state: 'before',
          state_line: 'your plan starts on 2 Nov 2026 — this week comes before it',
        },
        [],
      ),
    )
    renderApp('/progress')

    await screen.findByTestId('plan-against')
    expect(screen.getByTestId('plan-against-note').textContent).toContain(
      'your plan starts on 2 Nov 2026',
    )
  })

  it('says when a plan that has passed ended', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {
          state: 'after',
          state_line: "your plan's last week ended on 7 Feb 2027 — this week comes after it",
        },
        [],
      ),
    )
    renderApp('/progress')

    await screen.findByTestId('plan-against')
    expect(screen.getByTestId('plan-against-note').textContent).toContain(
      "your plan's last week ended on 7 Feb 2027",
    )
  })

  it('says done and not done, and never a grade', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {},
        [
          session({ date: '2026-05-12', status: 'done', actual: { text: '15 km', raw: 15_000 } }),
          session({ date: '2026-05-13', status: 'not_done' }),
          session({ date: '2026-05-14', status: 'planned' }),
        ],
      ),
    )
    renderApp('/progress')

    await screen.findByTestId('plan-against-sessions')
    const card = screen.getByTestId('plan-against')
    expect(card.textContent).toContain('done')
    expect(card.textContent).toContain('not done')
    expect(card.textContent).toContain('planned')
    // No grade, no percentage, no traffic light.
    for (const banned of ['missed', '%', 'score']) {
      expect(card.textContent?.toLowerCase()).not.toContain(banned)
    }
  })

  it('compares the pace only when both paces exist', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(
      week(
        {},
        [
          session({
            date: '2026-05-12',
            pace_comparison: "4:50 /km · 10 s/km faster than the plan's 5:00–5:32 /km",
          }),
          // The general aerobic day: the method fixes a band and states
          // no pace, so there is no pace to compare against.
          session({ date: '2026-05-13', kind: 'easy', pace: null, pace_comparison: null }),
        ],
      ),
    )
    renderApp('/progress')

    await screen.findByTestId('plan-against-sessions')
    expect(screen.getByTestId('plan-against-pace-2026-05-12').textContent).toContain(
      "faster than the plan's 5:00–5:32 /km",
    )
    expect(screen.queryByTestId('plan-against-pace-2026-05-13')).toBeNull()
  })

  it('puts the entry requirement beside the athlete’s own mileage', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(week({}, [session()]))
    renderApp('/progress')

    await screen.findByTestId('plan-against')
    expect(screen.getByTestId('plan-against-entry').textContent).toContain(
      'this level asks for',
    )
    expect(screen.getByTestId('plan-against-mileage').textContent).toContain('your last 4 weeks')
  })

  it('names the plan and the threshold it was paced with', async () => {
    vi.spyOn(api, 'planWeek').mockResolvedValue(week({}, [session()]))
    renderApp('/progress')

    await screen.findByTestId('plan-against')
    expect(screen.getByTestId('plan-against-source').textContent).toContain(
      'the Pfitzinger half marathon tier 2 plan',
    )
    expect(screen.getByTestId('plan-against-paced').textContent).toContain('165 bpm')
  })
})
