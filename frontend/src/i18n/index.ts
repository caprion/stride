/**
 * The typed string catalogue's public face.
 *
 * `t()` is the only way a component may produce a user-facing string. The
 * catalogue's types come along with the key, so a `direction` word cannot be
 * swapped for a `status` line by accident, and the lint can read the same
 * file the app renders from.
 *
 * The domain's sentences are not here: they arrive from the core already
 * rendered (core-contract §8), and the app shows them as they stand.
 */

import { catalogue, type CatalogueKey, type StringType } from './catalogue'
import { DIRECTION_WORDS } from './catalogue'

export { DIRECTION_WORDS, catalogue }
export type { CatalogueKey, StringType }

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
