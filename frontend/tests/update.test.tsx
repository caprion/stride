/**
 * The run's update, on the run detail.
 *
 * **It is not the product's voice.** The panel says whose words they are,
 * it lists the checks the draft passed, and the athlete can dismiss it.
 * The app's own facts stay the app's, and the panel sits below them.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** An update, with its checks and its verdict. */
const update = {
  activity_id: 101,
  update: {
    writer: 'the writer',
    reviewer: 'the reviewer',
    payload_hash: 'a-hash',
    draft: 'The run was 12.4 km in 1 h 7 m, at an average of 148 bpm.',
    checks: [
      { check: 'groundedness', findings: [] },
      { check: 'direction', findings: [] },
      { check: 'register', findings: [] },
    ],
    verdict: 'ship',
    created_at: '2026-05-13T06:30:00Z',
  },
  generated_at: '2026-05-13T06:30:00Z',
}

describe("the run's update", () => {
  it('is attributed, and lists the checks it passed', async () => {
    vi.spyOn(api, 'activityUpdate').mockResolvedValue(update)
    renderApp('/runs/101')

    const panel = await screen.findByTestId('activity-update')

    // The model's own words.
    expect(panel.querySelector('[data-testid="activity-update-draft"]')?.textContent).toBe(
      'The run was 12.4 km in 1 h 7 m, at an average of 148 bpm.',
    )
    // And whose words they are.
    expect(panel.querySelector('[data-testid="activity-update-attribution"]')?.textContent).toBe(
      'Written by the writer, checked by the reviewer',
    )
    // The three checks are listed.
    const checks = panel.querySelector('[data-testid="activity-update-checks"]')
    expect(checks?.textContent).toContain('Every number is in the payload')
    expect(checks?.textContent).toContain('Every direction agrees with the payload')
    expect(checks?.textContent).toContain('No word the app does not use')
  })

  it('says plainly when the checks found nothing to change', async () => {
    vi.spyOn(api, 'activityUpdate').mockResolvedValue({
      ...update,
      update: { ...update.update, reviewer: '' },
    })
    renderApp('/runs/101')

    const panel = await screen.findByTestId('activity-update')

    expect(panel.querySelector('[data-testid="activity-update-attribution"]')?.textContent).toBe(
      'Written by the writer; the checks found nothing to change',
    )
  })

  it('shows what the model was given', async () => {
    vi.spyOn(api, 'activityUpdate').mockResolvedValue(update)
    vi.spyOn(api, 'activityUpdatePayload').mockResolvedValue({
      activity_id: 101,
      payload_hash: 'a-hash',
      payload: null,
      text: 'Distance — 10 km\nDuration — 50 m',
      unavailable: null,
      generated_at: '2026-05-13T06:30:00Z',
    })
    renderApp('/runs/101')

    const payload = await screen.findByTestId('activity-update-payload')
    expect(payload.textContent).toContain('Distance — 10 km')
  })

  it('says what a check found', async () => {
    vi.spyOn(api, 'activityUpdate').mockResolvedValue({
      ...update,
      update: {
        ...update.update,
        checks: [
          {
            check: 'groundedness',
            findings: ['the number 500 is not in the payload'],
          },
          { check: 'direction', findings: [] },
          { check: 'register', findings: [] },
        ],
      },
    })
    renderApp('/runs/101')

    const checks = await screen.findByTestId('activity-update-checks')
    expect(checks.textContent).toContain('the number 500 is not in the payload')
  })

  it('can be dismissed', async () => {
    const user = userEvent.setup()
    vi.spyOn(api, 'activityUpdate').mockResolvedValue(update)
    renderApp('/runs/101')

    await screen.findByTestId('activity-update')
    await user.click(screen.getByTestId('activity-update-dismiss'))

    await waitFor(() => {
      expect(screen.queryByTestId('activity-update')).toBeNull()
    })
    expect(screen.getByTestId('activity-update-dismissed')).toBeTruthy()

    // And it can come back.
    await user.click(screen.getByTestId('activity-update-show'))
    expect(await screen.findByTestId('activity-update')).toBeTruthy()
  })

  it('is not there when the run has no update', async () => {
    vi.spyOn(api, 'activityUpdate').mockResolvedValue({
      activity_id: 101,
      update: null,
      generated_at: '2026-05-13T06:30:00Z',
    })
    renderApp('/runs/101')

    await waitFor(() => {
      expect(screen.queryByTestId('activity-update')).toBeNull()
    })
  })
})

describe("the update's settings", () => {
  it('say the feature\'s state, the two models and the base URLs', async () => {
    renderApp('/settings')

    const group = await screen.findByTestId('settings-update')

    // The feature is off until the athlete turns it on.
    const state = await screen.findByTestId('settings-update-state')
    expect(state.textContent).toContain('Off')
    // The two models, and the two base URLs.
    expect(group.querySelector('[data-testid="settings-update-writer-model"]')).toBeTruthy()
    expect(group.querySelector('[data-testid="settings-update-writer-family"]')).toBeTruthy()
    expect(group.querySelector('[data-testid="settings-update-writer-url"]')).toBeTruthy()
    expect(group.querySelector('[data-testid="settings-update-reviewer-model"]')).toBeTruthy()
    expect(group.querySelector('[data-testid="settings-update-reviewer-family"]')).toBeTruthy()
    expect(group.querySelector('[data-testid="settings-update-reviewer-url"]')).toBeTruthy()
    // And whether anything may leave the machine.
    expect(group.querySelector('[data-testid="settings-update-remote"]')).toBeTruthy()
  })

  it('say plainly what would be sent', async () => {
    renderApp('/settings')

    const sent = await screen.findByTestId('settings-update-sent')
    expect(sent.textContent).toContain("The run's own facts")
    expect(sent.textContent).toContain('Never your positions, never your note, never your name.')
  })
})
