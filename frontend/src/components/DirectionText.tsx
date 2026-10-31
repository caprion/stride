/**
 * The direction word.
 *
 * Three words, from a closed set. The colour is a reading aid only —
 * the word itself always carries the meaning, so the app never depends on
 * colour to say something (and never uses red).
 */

import type { Direction } from '../lib/types'

const TONE: Record<Direction, string> = {
  'better than usual': 'text-better',
  'about usual': 'text-usual',
  'could be better': 'text-could',
}

export function DirectionText({ direction }: { direction: Direction }) {
  return (
    <span className={`font-medium ${TONE[direction]}`} data-direction={direction}>
      {direction}
    </span>
  )
}
