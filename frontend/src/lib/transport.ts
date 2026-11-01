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
 *
 * **The answer is the body, and the status is the status.**
 *
 * That is the contract's rule (`docs/contract/api-paths.md`, "The shape of
 * an answer"), and this file is where it is kept:
 *
 *   - nothing here expects an envelope. The body is the body: it is
 *     parsed and handed over as it is, and the app's DTO is that body.
 *   - a failure is a failure. The desktop's HTTP layer puts the status in
 *     the status line; the phone's bridge rejects the call. Neither one
 *     answers with a body that says `{"error": …}` and a 200, and this
 *     file never reads such a body as an answer.
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
  /** The Garmin session ended; the key can be exchanged once more. */
  | 'needs_reconnect'
  /** The request could not be read. */
  | 'bad_request'
  | 'not_found'
  /** The phone's screen lock: the shell refuses the data routes. */
  | 'locked'
  | 'transport'
  | 'unknown'

/** The kinds a router may answer a failure with. */
export const API_ERROR_KINDS: readonly ApiErrorKind[] = [
  'no_plan',
  'stale_data',
  'not_enough_history',
  'garmin_rejected',
  'needs_reconnect',
  'bad_request',
  'not_found',
  'locked',
  'unknown',
]

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

/**
 * The host this app is running on.
 *
 * The desktop talks to the router over loopback HTTP; the phone talks to
 * the Rust core in the same process. The two hosts differ in more than the
 * transport — the Garmin key belongs to a different place on each — so
 * the app asks this question rather than sniffing the device.
 */
export type Host = 'native' | 'http'

/** The host, as the bridge sees it. */
export function currentHost(): Host {
  return hasNativeBridge() ? 'native' : 'http'
}

/**
 * The native transport: the Rust core, in the same process.
 *
 * The plugin answers with the response body as a JSON string, and it
 * rejects when the request failed. So:
 *
 *   - a resolved call is the body, and the body is parsed as it is — no
 *     envelope is unwrapped, because there is none;
 *   - a rejected call is a failure. It becomes a typed `ApiError` with
 *     the status the plugin reported ("404", "403", "400"), and it never
 *     reaches the app as data.
 */
export const nativeTransport = async (request: ApiRequest): Promise<unknown> => {
  const bridge = window.__stride_bridge__
  if (!bridge) {
    throw new ApiError('transport', 'the native bridge is not installed', 0)
  }
  let answer: string
  try {
    answer = await bridge.apiCall({
      method: request.method,
      path: request.path,
      body: request.body,
    })
  } catch (reason) {
    // **A failure is a failure.** The rejection carries the status; it is
    // never read as an answer.
    throw failureFrom(reason)
  }
  if (answer === null || answer === undefined || answer === '') return null
  try {
    return JSON.parse(answer)
  } catch {
    throw new ApiError('transport', 'the native bridge answered with no JSON', 0)
  }
}

/**
 * The failure the plugin reported.
 *
 * The plugin rejects with a message and a code — "400" for a route that
 * carries the key, "403" for a locked phone, "404" for a route that is
 * not in the frozen table — so the status comes across with the message
 * and the app can tell one failure from another. A rejection with no code
 * is a transport failure.
 */
function failureFrom(reason: unknown): ApiError {
  if (reason instanceof ApiError) return reason
  const status = statusOf(reason)
  return new ApiError(kindForStatus(status, 'native'), messageOf(reason), status)
}

/** The status a rejection carries, if it carries one. */
function statusOf(reason: unknown): number {
  if (typeof reason !== 'object' || reason === null) return 0
  const record = reason as { code?: unknown; status?: unknown }
  for (const value of [record.code, record.status]) {
    const parsed =
      typeof value === 'number'
        ? value
        : typeof value === 'string' && value.trim() !== ''
          ? Number(value)
          : Number.NaN
    if (Number.isFinite(parsed) && parsed >= 100 && parsed < 600) return parsed
  }
  return 0
}

/** The HTTP transport: the desktop rig and the browser. */
export const httpTransport = async (request: ApiRequest): Promise<unknown> => {
  let response: Response
  try {
    response = await fetch(request.path, {
      method: request.method,
      headers: request.body ? { 'content-type': 'application/json' } : undefined,
      body: request.body ? JSON.stringify(request.body) : undefined,
    })
  } catch (reason) {
    // The request never arrived. That is a failure too, and it is the
    // transport's own.
    throw new ApiError('transport', messageOf(reason), 0)
  }
  if (!response.ok) {
    // **The status is in the status line.** A failure is reported as a
    // failure; the body of a failure is only read for its code and its
    // message.
    const failure = await failureBody(response)
    throw new ApiError(
      kindForCode(failure.code, response.status, 'http'),
      failure.detail ?? `the request failed (${response.status})`,
      response.status,
    )
  }
  const text = await response.text()
  if (!text) return null
  try {
    return JSON.parse(text)
  } catch {
    throw new ApiError('transport', 'the answer was not JSON', 0)
  }
}

/** The message a failure carries, if it carries one. */
function messageOf(reason: unknown): string {
  if (reason instanceof Error && reason.message) return reason.message
  if (typeof reason === 'string' && reason.length > 0) return reason
  return 'the request failed'
}

/**
 * What a status means.
 *
 * The two hosts share the statuses but not every meaning: a 403 on the
 * desktop is a refused credential, and a 403 on the phone is a locked
 * screen — the shell refuses the data routes before the core is asked
 * for anything.
 */
function kindForStatus(status: number, host: Host): ApiErrorKind {
  if (status === 404) return 'not_found'
  if (status === 409) return 'not_enough_history'
  if (status === 401) return 'garmin_rejected'
  if (status === 403) return host === 'native' ? 'locked' : 'garmin_rejected'
  return 'unknown'
}

/**
 * The kind a failure carries.
 *
 * The router's answer names its kind — `no_plan`, `stale_data`,
 * `not_enough_history`, `garmin_rejected` — and the app renders each one
 * as its own sentence. The status alone cannot tell them apart: a plan
 * that is not there and a run that is not there are both a 404.
 *
 * The phone's bridge carries the status and the message but not the
 * code, so on the phone the kind falls back to the status.
 */
function kindForCode(
  code: string | undefined,
  status: number,
  host: Host,
): ApiErrorKind {
  if (code && (API_ERROR_KINDS as readonly string[]).includes(code)) {
    return code as ApiErrorKind
  }
  return kindForStatus(status, host)
}

/** A failure's code and its message, as the router sent them. */
async function failureBody(response: Response): Promise<{ code?: string; detail?: string }> {
  try {
    const body = (await response.json()) as {
      code?: string
      detail?: string
      error?: { code?: string; detail?: string }
    }
    return {
      code: body.error?.code ?? body.code,
      detail: body.error?.detail ?? body.detail,
    }
  } catch {
    return {}
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
