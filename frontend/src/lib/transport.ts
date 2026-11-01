/**
 * The transport.
 *
 * There are two hosts behind one interface:
 *
 *   - the desktop and the browser: `fetch` against the router that serves
 *     this very bundle;
 *   - the phone: the native bridge, which hands the path to the Rust core
 *     in the same process. The phone talks to no server.
 *
 * The transport is chosen once, at start-up, and never changes its mind.
 * The surface — the typed client in `api.ts` — is the same on both hosts,
 * so a screen cannot drift between them.
 */

export type HttpMethod = 'GET' | 'POST'

export interface ApiRequest {
  method: HttpMethod
  /** The path, with the API base already on it. */
  path: string
  body?: unknown
}

/**
 * The native bridge.
 *
 * The Android shell installs this on the window object; the shape is
 * deliberately tiny — one method, JSON in, JSON out — because the key
 * never crosses it.
 */
export interface StrideNativeBridge {
  apiCall(request: { method: HttpMethod; path: string; body?: unknown }): Promise<string>
}

declare global {
  interface Window {
    __stride_bridge__?: StrideNativeBridge
  }
}

/** A typed business failure, as the contract names them. */
export type ApiErrorKind =
  | 'no_plan'
  | 'stale_data'
  | 'not_enough_history'
  | 'garmin_rejected'
  | 'not_found'
  | 'transport'
  | 'unknown'

export class ApiError extends Error {
  readonly kind: ApiErrorKind
  readonly status: number

  constructor(kind: ApiErrorKind, message: string, status = 0) {
    super(message)
    this.name = 'ApiError'
    this.kind = kind
    this.status = status
  }
}

/** True when the native bridge is installed. */
export function hasNativeBridge(): boolean {
  return (
    typeof window !== 'undefined' && typeof window.__stride_bridge__?.apiCall === 'function'
  )
}

/** The native transport: the Rust core, in the same process. */
export const nativeTransport = async (request: ApiRequest): Promise<unknown> => {
  const bridge = window.__stride_bridge__
  if (!bridge) {
    throw new ApiError('transport', 'the native bridge is not installed', 0)
  }
  const answer = await bridge.apiCall({
    method: request.method,
    path: request.path,
    body: request.body,
  })
  return answer ? JSON.parse(answer) : null
}

/** The HTTP transport: the desktop rig and the browser. */
export const httpTransport = async (request: ApiRequest): Promise<unknown> => {
  const response = await fetch(request.path, {
    method: request.method,
    headers: request.body ? { 'content-type': 'application/json' } : undefined,
    body: request.body ? JSON.stringify(request.body) : undefined,
  })
  if (!response.ok) {
    throw new ApiError(
      kindForStatus(response.status),
      await errorText(response),
      response.status,
    )
  }
  const text = await response.text()
  return text ? JSON.parse(text) : null
}

function kindForStatus(status: number): ApiErrorKind {
  if (status === 404) return 'not_found'
  if (status === 401 || status === 403) return 'garmin_rejected'
  if (status === 409) return 'not_enough_history'
  return 'unknown'
}

async function errorText(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as { detail?: string; error?: string }
    return body.detail ?? body.error ?? `the request failed (${response.status})`
  } catch {
    return `the request failed (${response.status})`
  }
}

/**
 * The transport the app uses.
 *
 * The build-time flag `VITE_NATIVE` only says "the native bridge is
 * expected to be there" — it never decides a layout, and it is not a
 * device sniff.
 */
export function selectTransport(): (request: ApiRequest) => Promise<unknown> {
  if (import.meta.env.VITE_NATIVE === 'true' && hasNativeBridge()) {
    return nativeTransport
  }
  if (hasNativeBridge()) return nativeTransport
  return httpTransport
}
