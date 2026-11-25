/**
 * The Garmin controls, on both hosts.
 *
 * The rules the test holds to:
 *
 *   - the **door to the older history is on the phone too**. The athlete
 *     asks for the months the phone has not checked, on the phone, and
 *     not only on the desktop — the phone's ledger is as empty as the
 *     rig's was without it;
 *   - the door asks for the **history** and not for a plain refresh;
 *   - the door is there **only when there is older history left to
 *     check**, so it never promises work that is not waiting.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { syncStatusFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => {
  vi.restoreAllMocks()
  delete window.__stride_bridge__
  window.history.pushState({}, '', '/?fixtures=1')
})

/** A connected phone or rig, with older history still to check. */
function connected() {
  vi.spyOn(api, 'authStatus').mockResolvedValue({
    connected: true,
    method: 'personal',
    key_source: 'keystore',
    key_last4: 'cdef',
    last_sync_at: '2026-05-13T06:30:00Z',
    generated_at: '2026-05-13T06:30:00Z',
  })
  vi.spyOn(api, 'syncStatus').mockResolvedValue({
    ...syncStatusFixture,
    more_pending: true,
  })
}

describe('the older history', () => {
  it('is on the phone: the athlete asks for the months the phone has not checked', async () => {
    connected()
    const syncRun = vi.spyOn(api, 'syncRun').mockResolvedValue({ queued: true })
    // The phone: the native bridge is installed, so the app is on the
    // phone and not on the rig.
    window.__stride_bridge__ = { apiCall: vi.fn() }

    renderApp('/settings')
    const door = await screen.findByTestId('sync-older')
    await userEvent.click(door)

    // **The history, and not a plain refresh.**
    await waitFor(() => expect(syncRun).toHaveBeenCalledWith(true))
  })

  it('asks for the history on the rig too', async () => {
    connected()
    const syncRun = vi.spyOn(api, 'syncRun').mockResolvedValue({ queued: true })

    renderApp('/settings')
    const door = await screen.findByTestId('sync-older')
    await userEvent.click(door)

    await waitFor(() => expect(syncRun).toHaveBeenCalledWith(true))
  })

  it('is not there when there is no older history to check', async () => {
    connected()
    vi.spyOn(api, 'syncStatus').mockResolvedValue({
      ...syncStatusFixture,
      more_pending: false,
    })

    renderApp('/settings')
    await screen.findByTestId('settings-connection-state')
    expect(screen.queryByTestId('sync-older')).toBeNull()
  })
})
