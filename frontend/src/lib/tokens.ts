/**
 * The tokens, read at the moment they are needed.
 *
 * **A component never names a colour.** It names a token — a Tailwind class
 * or a CSS variable. That rule holds for the app's own markup, where the
 * variable can simply be written into the style, and it has to hold for
 * the drawings too: the film is an SVG, and an SVG's presentation
 * attributes cannot carry a variable, so those are given the variable
 * through the style, where it works.
 *
 * There is one place a variable cannot go at all: the map library, which
 * writes colours into the drawing itself as attributes. For that one the
 * variable is **read from the page** and the value handed over — so the map
 * is still painted in the app's own palette, and still follows the
 * athlete's chosen palette, because it reads whatever the page has.
 */

/** The value of a token on the page, as the page has it. */
export function token(name: string, fallback = 'transparent'): string {
  if (typeof document === 'undefined') return fallback
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim()
  return value || fallback
}
