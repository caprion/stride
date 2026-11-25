/**
 * Adopting a programme: **the four, and the choice**.
 *
 * The rules the test holds to:
 *
 *   - the picker holds **the four programmes and nothing else** — the
 *     80/20 base is a spec, and there is no template for it;
 *   - the tier's **entry requirement stands beside the athlete's own
 *     mileage**, unranked: the app never compares them;
 *   - the adoption names the programme, the race and the start date.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { basesFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The picker, once it is there. */
async function picker() {
  renderApp('/plan/bases')
  return screen.findByTestId('plan-bases')
}

describe('adopting a programme', () => {
  it('offers the four programmes of the catalogue', async () => {
    const list = await picker()

    expect(list.querySelectorAll('[data-testid^="plan-base-pfitzinger-"]')).toHaveLength(4)
    // The 80/20 base is not in the picker: there is no template for it.
    expect(screen.queryByTestId('plan-base-pick-eighty_twenty-10k-tier-1')).toBeNull()
    expect(basesFixture.bases).toHaveLength(4)
  })

  it('puts the entry requirement beside the athlete’s own mileage', async () => {
    const list = await picker()

    const entry = list.querySelector('[data-testid="plan-base-entry-pfitzinger-10k-tier-1"]')
    // The requirement is the programme's own line.
    expect(entry?.textContent).toContain('this level asks for')
    // And the athlete's own mileage stands beside it.
    expect(
      list.querySelector('[data-testid="plan-base-mileage-pfitzinger-10k-tier-1"]')?.textContent,
    ).toContain('your last')
  })

  it('never compares the requirement with the mileage', async () => {
    const list = await picker()

    // No colour, no grade, no comparison: the two facts are stated.
    expect(list.textContent).not.toMatch(/below|above|enough|not enough/i)
  })

  it('adopts the programme the athlete picks', async () => {
    const adopt = vi.spyOn(api, 'adoptPlan').mockResolvedValue({
      adopted: true,
      plan: {
        programme_id: 'pfitzinger-10k-tier-1',
        programme_version: '1',
        label: '10K tier 1',
        method: 'Pfitzinger',
        tier: 'tier 1',
        goal_distance: { text: '10 km', raw: 10_000 },
        goal_label: '10K',
        race_date: '2026-08-01',
        race_date_label: '1 Aug 2026',
        start_date: '2026-05-11',
        end_date: '2026-08-09',
        adopted_on: '2026-05-13',
        source: 'prescribed',
        pacing: {
          paced: true,
          threshold: null,
          rule: null,
          missing: null,
          threshold_line: null,
          rule_line: null,
        },
        entry: { requirement: null, plan_band: null, athlete: null },
      },
      generated_at: '2026-05-13T06:30:00Z',
    })
    await picker()

    await userEvent.click(await screen.findByTestId('plan-base-pick-pfitzinger-10k-tier-1'))
    const form = await screen.findByTestId('plan-adopt-form')

    // The race and the start are the athlete's own to set.
    await userEvent.type(
      form.querySelector('[data-testid="plan-adopt-race-date"]') as HTMLInputElement,
      '2026-08-01',
    )
    await userEvent.click(screen.getByTestId('plan-adopt-submit'))

    await waitFor(() => expect(adopt).toHaveBeenCalled())
    expect(adopt.mock.calls[0]?.[0]).toMatchObject({
      programme_id: 'pfitzinger-10k-tier-1',
      race_date: '2026-08-01',
    })
  })

  it('says when the catalogue holds no programme', async () => {
    vi.spyOn(api, 'planBases').mockResolvedValue({
      ...basesFixture,
      bases: [],
      unavailable: 'no programme to adopt yet',
    })
    renderApp('/plan/bases')

    await waitFor(() =>
      expect(screen.getByTestId('plan-bases-unavailable')).toBeTruthy(),
    )
    expect(screen.getByTestId('plan-bases-unavailable').textContent).toContain(
      'no programme to adopt yet',
    )
  })
})
