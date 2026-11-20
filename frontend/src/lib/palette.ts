/**
 * The palette.
 *
 * **One attribute on the root element.** The app's colours are CSS
 * variables, and the variables are the palette's, so `data-palette` on the
 * root element is the whole of the swap: every Tailwind class, every chart
 * and every surface follows it, and no component is rewritten.
 *
 * The choice is the athlete's, and it stays on the device: it is kept in
 * the WebView's own storage, beside the rest of the app's state, and it
 * never leaves the phone or the browser.
 *
 * It is deliberately **not** part of the api's settings: those are the
 * athlete's training — the goal and the pause — and they are stored in the
 * database, where the contract can see them. A colour scheme is a
 * property of this screen, not of the training.
 */

import { DEFAULT_PALETTE, palettes, type PaletteName } from '../../tokens.ts'

/** Where the choice is kept. It is device-local, like the rest of the UI. */
const STORAGE_KEY = 'stride.palette'

/** Whether a name is one of the palettes. */
export function isPalette(value: unknown): value is PaletteName {
  return typeof value === 'string' && Object.hasOwn(palettes, value)
}

/**
 * The palette the athlete chose, or the default.
 *
 * A stored value that is not a palette — an old build's name, a hand-edited
 * value — is not a palette, and the app falls back to the default rather
 * than painting itself in nothing.
 */
export function chosenPalette(): PaletteName {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY)
    return isPalette(stored) ? stored : DEFAULT_PALETTE
  } catch {
    // A browser with storage turned off is a browser that cannot remember
    // a colour scheme; it can still use one.
    return DEFAULT_PALETTE
  }
}

/**
 * The palette the app is drawn in.
 *
 * The attribute is on the root element, so the native shell can read the
 * same one: the phone's chrome and the web bundle it embeds are the same
 * palette by construction.
 */
export function applyPalette(name: PaletteName = chosenPalette()): void {
  const root = document.documentElement
  root.dataset.palette = name

  // The browser's own chrome follows the app: the theme colour is the
  // palette's deepest brand step, so the address bar is not a colour from
  // another palette.
  const theme = document.querySelector('meta[name="theme-color"]')
  if (theme) theme.setAttribute('content', palettes[name].brand[700])
}

/**
 * The athlete's choice: it is kept, and the app is painted in it.
 *
 * This is what a setting calls.
 */
export function setPalette(name: PaletteName): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, name)
  } catch {
    // Storage turned off: the palette holds for this session, which is
    // still better than not changing at all.
  }
  applyPalette(name)
}
