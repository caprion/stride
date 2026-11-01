/**
 * The key, and the host that keeps it.
 *
 * The plan's acceptance check is that the key never appears in any
 * WebView-reachable surface, and the phone's bridge holds it: the key
 * routes are refused there, and the native setup screen owns the key
 * instead.
 *
 * So the app has two key surfaces, and it must never show the desktop's
 * field on the phone. These tests hold that line.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'
import { hasKeyField, keySurface } from '../src/lib/connect'

/** A key that is obviously not a key. */
const A_KEY = 'test-key'

/**
 * The client and its error type, from the same module registry.
 *
 * The modules are reloaded so that the client picks up the bridge the
 * test installs — and the error type has to come from that same
 * registry, or `instanceof` would compare two different classes.
 */
async function loadClient() {
  vi.resetModules()
  const { api } = await import('../src/lib/api')
  const { ApiError } = await import('../src/lib/transport')
  return { api, ApiError }
}

afterEach(() => {
  delete (window as { __stride_bridge__?: unknown }).__stride_bridge__
  vi.unstubAllGlobals()
  vi.resetModules()
  // The other tests run against the fixture set; these talk to a host.
  window.history.pushState({}, '', '/')
})

describe('where the key is typed', () => {
  it('the desktop has a field', () => {
    expect(keySurface('http')).toEqual({ kind: 'field' })
    expect(hasKeyField('http')).toBe(true)
  })

  it('the phone has the native screen, and no field at all', () => {
    expect(keySurface('native')).toEqual({ kind: 'native-screen' })
    expect(hasKeyField('native')).toBe(false)
  })
})

describe('the key on the phone', () => {
  it('never crosses the bridge', async () => {
    const call = vi.fn().mockRejectedValue(new Error('the key is not in the browser'))
    window.__stride_bridge__ = { apiCall: call }
    const { api, ApiError } = await loadClient()

    await expect(api.setGarminPersonalKey(A_KEY)).rejects.toBeInstanceOf(ApiError)

    // The call refused here, so nothing was sent: the key stayed in the
    // app's hands and never reached the page.
    expect(call).not.toHaveBeenCalled()
  })

  it('opens the native setup screen, and sends no key with it', async () => {
    // The plugin opens its screen and *then* refuses the call, with a
    // 400. That refusal is how the phone says "the screen is open".
    const call = vi
      .fn()
      .mockRejectedValue(Object.assign(new Error('the key is not in the browser'), { code: '400' }))
    window.__stride_bridge__ = { apiCall: call }
    const { api } = await loadClient()

    await expect(api.openGarminKeySetup()).resolves.toBeUndefined()

    expect(call).toHaveBeenCalledTimes(1)
    const sent = call.mock.calls[0]?.[0] as { method: string; path: string; body?: unknown }
    expect(sent.method).toBe('POST')
    expect(sent.path).toContain('/auth/garmin/personal-key')
    expect(sent.body).toBeUndefined()
    expect(JSON.stringify(sent)).not.toContain(A_KEY)
  })

  it('a screen that fails to open is a failure', async () => {
    const call = vi
      .fn()
      .mockRejectedValue(Object.assign(new Error('the phone is locked'), { code: '403' }))
    window.__stride_bridge__ = { apiCall: call }
    const { api, ApiError } = await loadClient()

    await expect(api.openGarminKeySetup()).rejects.toBeInstanceOf(ApiError)
  })
})

describe('the key on the desktop', () => {
  it('goes to the desktop’s own server', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => '{"ok":true}',
    })
    vi.stubGlobal('fetch', fetchMock)
    const { api } = await loadClient()

    await api.setGarminPersonalKey(A_KEY)

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toContain('/auth/garmin/personal-key')
    expect(init.body).toContain(A_KEY)
  })
})
