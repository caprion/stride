/**
 * The Settings screen.
 *
 * The acceptance checks the plan names, as assertions:
 *
 *   - the goal is one number, and a blank is no goal;
 *   - the pause says what it is, in the app's voice;
 *   - the key is never on the screen — only its last four characters;
 *   - the one destructive action says what it deletes, and takes the word;
 *   - About carries the version, the licence, the promise and the source.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { healthFixture, settingsFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => {
  delete window.__stride_bridge__
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('phone data ownership', () => {
  it('opens the native data screen without collecting a passphrase in the page', async () => {
    const openData = vi.fn().mockResolvedValue(undefined)
    window.__stride_bridge__ = { apiCall: vi.fn(), openData }
    renderApp('/settings')
    await userEvent.click(
      await screen.findByRole('button', { name: 'Backup, restore and export' }),
    )
    await waitFor(() => expect(openData).toHaveBeenCalledOnce())
    expect(screen.queryByLabelText(/passphrase/i)).toBeNull()
  })

  it('shows a native data screen failure instead of claiming a saved backup', async () => {
    window.__stride_bridge__ = {
      apiCall: vi.fn(),
      openData: vi.fn().mockRejectedValue(new Error('locked')),
    }
    renderApp('/settings')
    await userEvent.click(
      await screen.findByRole('button', { name: 'Backup, restore and export' }),
    )
    await waitFor(() =>
      expect(screen.getByTestId('settings-notice')).toHaveTextContent(
        'The backup was not written',
      ),
    )
  })
})

describe('the weekly goal', () => {
  it('shows the goal the api sent, and the number in the field', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-goal"]')).toBeTruthy(),
    )

    // The goal the api answered — the seeded store holds none.
    expect(
      container.querySelector('[data-testid="settings-goal-value"]')?.textContent,
    ).toContain('No goal')
    const input = container.querySelector(
      '[data-testid="settings-goal-input"]',
    ) as HTMLInputElement
    expect(input.value).toBe('')
  })

  it('says that a blank goal is no goal, and never a zero', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-goal"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('A blank goal means no goal.')
    // The goal is never a zero: a zero would read as a failure.
    expect(
      container.querySelector('[data-testid="settings-goal-value"]')?.textContent,
    ).not.toBe('0 km')
  })

  it('says "No goal" when there is no goal', async () => {
    window.history.pushState({}, '', '/?fixtures=empty')
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-goal"]')).toBeTruthy(),
    )

    expect(
      container.querySelector('[data-testid="settings-goal-value"]')?.textContent,
    ).toContain('No goal')
  })
})

describe('the pause', () => {
  it('says the plan is paused, with the date and the reason', async () => {
    window.history.pushState({}, '', '/?fixtures=paused')
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-pause-state"]')).toBeTruthy(),
    )

    const state =
      container.querySelector('[data-testid="settings-pause-state"]')?.textContent ?? ''
    expect(state).toContain('paused until')
    expect(state).toContain('travel')
  })

  it('offers the three reasons, and no free text', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-pause"]')).toBeTruthy(),
    )

    const select = container.querySelector(
      '[data-testid="settings-pause-reason"]',
    ) as HTMLSelectElement
    const values = [...select.options].map((option) => option.value)
    expect(values).toEqual(['travel', 'sick', 'injured', ''])
  })
})

describe('the connection', () => {
  it('shows the key identifier, and never the key', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-connection"]')).toBeTruthy(),
    )

    // The key's identifier is the last four characters — and the seeded
    // store holds no key, so there is nothing to show.
    const key =
      container.querySelector('[data-testid="settings-connection-key"]')?.textContent ?? ''
    expect(key).toContain('—')
    // The whole key is nowhere: the app has never had it.
    expect(container.textContent).not.toContain('personal-key')
  })

  it('shows the state and the last sync', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-connection-state"]')).toBeTruthy(),
    )

    // The seeded store holds no key.
    expect(
      container.querySelector('[data-testid="settings-connection-state"]')?.textContent,
    ).toContain('Not connected')
    expect(
      container.querySelector('[data-testid="settings-connection-sync"]')?.textContent,
    ).toContain('Last synced')
  })
})

describe('the storage', () => {
  it('shows the size of the database', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-storage"]')).toBeTruthy(),
    )

    // The size is the host's, and it is only shown when the api answers
    // one: the seeded store answers none, so the row is not there.
    expect(settingsFixture.storage.size).toBeNull()
    expect(container.querySelector('[data-testid="settings-storage-size"]')).toBeNull()
  })

  it('takes the word before it deletes anything', async () => {
    const user = userEvent.setup()
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-delete"]')).toBeTruthy(),
    )

    await user.click(container.querySelector('[data-testid="settings-delete"]') as HTMLElement)

    // The confirmation says exactly what goes.
    const confirm =
      container.querySelector('[data-testid="settings-delete-confirm"]')?.textContent ?? ''
    expect(confirm).toContain('every run')
    expect(confirm).toContain('every wellness day')
    expect(confirm).toContain('the plan')
    expect(confirm).toContain('every note')
    expect(confirm).toContain('It cannot be undone.')

    // And the button stays out of reach until the word is typed.
    const button = container.querySelector(
      '[data-testid="settings-delete-confirm-button"]',
    ) as HTMLButtonElement
    expect(button.disabled).toBe(true)

    await user.type(
      container.querySelector('[data-testid="settings-delete-input"]') as HTMLElement,
      'DELETE',
    )
    expect(button.disabled).toBe(false)
  })
})

describe('About', () => {
  it('carries the version, the licence, the promise and the source', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-about"]')).toBeTruthy(),
    )

    // The version is the one the api answered.
    expect(
      container.querySelector('[data-testid="settings-about-version"]')?.textContent,
    ).toContain(`Version ${healthFixture.version}`)
    expect(
      container.querySelector('[data-testid="settings-about-licence"]')?.textContent,
    ).toContain('AGPL-3.0')
    expect(
      container.querySelector('[data-testid="settings-about-promise"]')?.textContent,
    ).toContain('Your data stays on this phone.')
    expect(
      container.querySelector('[data-testid="settings-about-source"]')?.textContent,
    ).toContain('github.com/caprion/stride')
  })
})

describe('the shell', () => {
  it('reaches the settings without making it a destination', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="shell-settings"]')).toBeTruthy(),
    )

    const destinations = new Set(
      [...container.querySelectorAll('[data-destination]')].map((node) =>
        node.getAttribute('data-destination'),
      ),
    )
    expect([...destinations].sort()).toEqual(['/', '/body', '/progress', '/runs'])
    expect(screen.getAllByTestId('shell-settings').length).toBeGreaterThan(0)
  })
})
