/**
 * The direction word.
 *
 * The core writes the word; the app colours it. The colour is a reading
 * aid only — the word itself always carries the meaning, so the app never
 * depends on colour to say something (and never uses red).
 */

import { DIRECTION_WORDS } from '../i18n'

const TONE: Record<string, string> = {
  'better than usual': 'text-better',
  'about usual': 'text-usual',
  'could be better': 'text-could',
}

/**
 * The word the core sent, in the tone it belongs to.
 *
 * A word outside the closed set gets the neutral tone rather than a made
 * up one: the set is closed, and an unknown word is a mistake, not a
 * fourth meaning.
 */
export function DirectionText({ text }: { text: string }) {
  const known = (DIRECTION_WORDS as readonly string[]).includes(text)
  return (
    <span
      className={`font-medium ${known ? (TONE[text] ?? 'text-ink-soft') : 'text-ink-soft'}`}
      data-direction={text}
    >
      {text}
    </span>
  )
}
