/**
 * A media query as state.
 *
 * Used where two layouts are genuinely different rather than merely
 * different in width — a table and a set of cards, for instance. Rendering
 * both and hiding one with CSS would double the DOM for no reason, and on a
 * phone that is 160 extra rows of nothing.
 */

import { useEffect, useState } from 'react'

export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() =>
    typeof window === 'undefined' ? false : window.matchMedia(query).matches,
  )

  useEffect(() => {
    const list = window.matchMedia(query)
    const update = () => setMatches(list.matches)
    update()
    list.addEventListener('change', update)
    return () => list.removeEventListener('change', update)
  }, [query])

  return matches
}
