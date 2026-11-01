/**
 * The first-run page.
 *
 * It stands alone — no chrome, no tabs, no destinations — it states the
 * promise, and it takes the key in a field that exists on the desktop
 * only. The key never lands in the URL, and the page disappears once the
 * app is connected.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('the first-run page', () => {
  it('stands alone: no chrome, no tabs, no destinations', async () => {
    window.history.pushState({}, '', '/?fixtures=fresh')
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="key-input"]')).toBeTruthy(),
    )

    expect(container.querySelector('.shell')).toBeNull()
    expect(container.querySelectorAll('[data-destination]')).toHaveLength(0)
    expect(container.querySelector('[data-nav="tabbar"]')).toBeNull()
    expect(container.querySelector('[data-nav="rail"]')).toBeNull()
  })

  it('states the promise in one line, before the field', async () => {
    window.history.pushState({}, '', '/?fixtures=fresh')
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="key-input"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="connect-promise"]')?.textContent).toBe(
      'Your data stays on this phone.',
    )
    // The promise comes before the field.
    const promise = container.querySelector('[data-testid="connect-promise"]') as HTMLElement
    const field = container.querySelector('[data-testid="key-input"]') as HTMLElement
    expect(
      promise.compareDocumentPosition(field) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('takes the key in a masked field, with the two actions', async () => {
    window.history.pushState({}, '', '/?fixtures=fresh')
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="key-input"]')).toBeTruthy(),
    )

    const field = container.querySelector('[data-testid="key-input"]') as HTMLInputElement
    expect(field.type).toBe('password')
    expect(container.querySelector('[data-testid="key-test"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="key-connect"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="connect-not-now"]')).toBeTruthy()
  })

  it('keeps the key out of the URL', async () => {
    const user = userEvent.setup()
    window.history.pushState({}, '', '/?fixtures=fresh')
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="key-input"]')).toBeTruthy(),
    )

    await user.type(
      container.querySelector('[data-testid="key-input"]') as HTMLElement,
      'a-key',
    )
    await user.click(screen.getByTestId('key-test'))

    expect(window.location.search).not.toContain('a-key')
    expect(window.location.href).not.toContain('a-key')
  })

  it('leaves the app in an honest empty state when the athlete says "Not now"', async () => {
    const user = userEvent.setup()
    window.history.pushState({}, '', '/?fixtures=fresh')
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="key-input"]')).toBeTruthy(),
    )

    await user.click(screen.getByTestId('connect-not-now'))

    // The app opens on Today, and Today says what is missing.
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-empty"]')).toBeTruthy(),
    )
    expect(container.textContent).toContain("Connect your Garmin key to see today's numbers")
  })

  it('disappears once the app is connected', async () => {
    // The fixture set's default is a connected app.
    const { container } = renderApp('/connect')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-session"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="key-input"]')).toBeNull()
  })
})
