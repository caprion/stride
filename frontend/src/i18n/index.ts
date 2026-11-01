/**
 * The typed string catalogue's public face.
 *
 * `t()` is the only way a component may produce a user-facing string. The
 * catalogue's types come along with the key, so a `direction` word cannot be
 * swapped for a `status` line by accident, and the lint can read the same
 * file the app renders from.
 */

import {
  DIRECTION_WORDS,
  catalogue,
  directions,
  type CatalogueKey,
  type DirectionKey,
  type DirectionWord,
  type StringType,
} from './catalogue'

export { DIRECTION_WORDS, catalogue, directions }
export type { CatalogueKey, DirectionKey, DirectionWord, StringType }

/** The keys of the catalogue that carry a given register. */
export type KeysOfType<T extends StringType> = {
  [K in CatalogueKey]: (typeof catalogue)[K]['type'] extends T ? K : never
}[CatalogueKey]

export type Params = Record<string, string | number>

/**
 * Fill a catalogue entry's placeholders.
 *
 * A placeholder with no matching parameter is left in place rather than
 * silently emptied — a visible `{distance}` is a bug the lint and the
 * tests can see.
 */
function fill(text: string, params?: Params): string {
  if (!params) return text
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  )
}

/** Render a catalogue entry. */
export function t(key: CatalogueKey, params?: Params): string {
  return fill(catalogue[key].text, params)
}

/** Render one of the three closed direction words. */
export function directionText(key: DirectionKey): DirectionWord {
  return directions[key].text
}

/**
 * The direction word for a core-provided direction.
 *
 * The core owns the value; the app only chooses the word that carries it.
 * An unknown value falls back to "about usual" rather than inventing a
 * fourth word — the set is closed.
 */
export function directionWord(direction: string): DirectionWord {
  const match = DIRECTION_WORDS.find((word) => word === direction)
  return match ?? 'about usual'
}
