/**
 * The transport, and the rule the contract states:
 *
 *     the answer is the body, and the status is the status.
 *
 * (`docs/contract/api-paths.md`, "The shape of an answer".)
 *
 * These tests hold the two hosts to it: the body is the body, with
 * nothing wrapped around it, and a failure is a failure on both of them.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'
import { ApiError, httpTransport, nativeTransport } from '../src/lib/transport'

const request = { method: 'GET' as const, path: '/api/v1/insights/today' }

afterEach(() => {
  delete (window as { __stride_bridge__?: unknown }).__stride_bridge__
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe('the native transport', () => {
  it('hands the app the body, with nothing around it', async () => {
    window.__stride_bridge__ = {
      apiCall: vi.fn().mockResolvedValue('{"value_text":"6 h 10 m","direction_text":"about usual"}'),
    }

    const answer = await nativeTransport(request)

    // The body is the answer. There is no envelope to unwrap, because
    // there is no envelope: the plugin hands the same bytes the desktop
    // puts on the wire.
    expect(answer).toEqual({ value_text: '6 h 10 m', direction_text: 'about usual' })
  })

  it('turns a rejection into a failure, and never into data', async () => {
    // The plugin rejects with a message and a code — "403" for a locked
    // phone.
    window.__stride_bridge__ = {
      apiCall: vi
        .fn()
        .mockRejectedValue(Object.assign(new Error('the phone is locked'), { code: '403' })),
    }

    const failure = await nativeTransport(request).catch((reason: unknown) => reason)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).kind).toBe('locked')
    expect((failure as ApiError).status).toBe(403)
  })

  it('a body that is not JSON is a failure, not an answer', async () => {
    window.__stride_bridge__ = { apiCall: vi.fn().mockResolvedValue('not json at all') }

    const failure = await nativeTransport(request).catch((reason: unknown) => reason)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).kind).toBe('transport')
  })

  it('a bridge that is not there is a failure too', async () => {
    const failure = await nativeTransport(request).catch((reason: unknown) => reason)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).kind).toBe('transport')
  })
})

describe('the http transport', () => {
  it('puts the status in the status line, and reports a failure as one', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      json: async () => ({ detail: 'no such run' }),
      text: async () => '',
    })
    vi.stubGlobal('fetch', fetchMock)

    const failure = await httpTransport({
      method: 'GET',
      path: '/api/v1/activities/9',
    }).catch((reason: unknown) => reason)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).kind).toBe('not_found')
    expect((failure as ApiError).status).toBe(404)
    expect((failure as ApiError).message).toBe('no such run')
  })

  it('the body of a success is the answer', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        text: async () => '{"items":[]}',
      }),
    )

    const answer = await httpTransport({ method: 'GET', path: '/api/v1/activities' })

    expect(answer).toEqual({ items: [] })
  })

  it('a request that never arrived is a failure too', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('the server is not there')))

    const failure = await httpTransport({
      method: 'GET',
      path: '/api/v1/activities',
    }).catch((reason: unknown) => reason)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).kind).toBe('transport')
  })
})
