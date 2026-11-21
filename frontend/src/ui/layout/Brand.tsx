/**
 * The brand.
 *
 * One mark, one word. It is in the shell's chrome, and it is on the
 * first-run page — which stands alone, and has to say what it is.
 *
 * **The mark is the mock's mark**: the warm gradient square with the
 * app's glyph in it, and the word beside it in the mock's weight. The
 * mark takes its colours from the tokens, so it follows the palette the
 * athlete chose.
 */

import { Activity } from 'lucide-react'

export function Brand() {
  return (
    <span className="brand">
      <span className="brand__mark">
        <Activity size={16} strokeWidth={2.6} aria-hidden="true" />
      </span>
      <span className="brand__word">Stride</span>
    </span>
  )
}
