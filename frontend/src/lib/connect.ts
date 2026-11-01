/**
 * Where the Garmin key lives.
 *
 * The key never belongs to the app. The desktop's server keeps it on the
 * desktop; the phone's Keystore keeps it on the phone. So there are two
 * key surfaces, and the app has to offer the right one:
 *
 *   - **the desktop**: a field. The key goes to the desktop's server, and
 *     the app holds it only while it is being sent.
 *   - **the phone**: the native setup screen. The bridge refuses the key
 *     routes — the plan's acceptance check is that the key never appears
 *     in any WebView-reachable surface — and opens the native screen
 *     instead, where the key is typed into the app and straight into the
 *     Keystore.
 *
 * **The field is not shown on the phone at all.** A field that cannot be
 * typed into is a trap: it looks like the way to do something and is not.
 * So the decision lives here, in one place, and the screens ask it rather
 * than working it out for themselves.
 */

import { currentHost, type Host } from './transport'
import { api } from './api'

/** How the key is entered on a host. */
export type KeySurface =
  /** The desktop: a field, and the key goes to the desktop's server. */
  | { kind: 'field' }
  /** The phone: the native setup screen owns the key. */
  | { kind: 'native-screen' }

/**
 * The key surface of a host.
 *
 * The phone's answer is the native screen, and it is the phone's own
 * screen: the app never shows the key field there, and never types the
 * key.
 */
export function keySurface(host: Host = currentHost()): KeySurface {
  return host === 'native' ? { kind: 'native-screen' } : { kind: 'field' }
}

/** True when this host types the key in the app's own field. */
export function hasKeyField(host: Host = currentHost()): boolean {
  return keySurface(host).kind === 'field'
}

/**
 * Connect Garmin, on this host.
 *
 * The phone opens the native setup screen: the key belongs to the app and
 * to the Keystore, and it never enters the WebView. The desktop has the
 * field, and the Settings screen owns it — so there this call only says
 * which surface it is, and the screen shows the field.
 */
export async function connectGarmin(): Promise<KeySurface['kind']> {
  if (keySurface().kind === 'native-screen') {
    await api.openGarminKeySetup()
    return 'native-screen'
  }
  return 'field'
}
