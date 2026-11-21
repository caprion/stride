/**
 * The width of an element, measured as it changes.
 *
 * The app's charts are drawn by the app itself, so they are drawn at the
 * width they will be shown at: a chart that is drawn at the wrong width
 * and corrected afterwards flickers, and a chart that overflows its
 * column is a chart the reader cannot read.
 *
 * The observer keeps the width right when the window changes, and the
 * measurement is taken before the first paint — `useLayoutEffect` — so
 * the drawing is right the first time it is shown.
 */

import { useLayoutEffect, useState } from 'react'
import type { RefObject } from 'react'

export function useWidth(ref: RefObject<HTMLDivElement>): number {
  const [width, setWidth] = useState(0)
  useLayoutEffect(() => {
    const node = ref.current
    if (!node) return
    const measure = () => {
      const next = Math.round(node.getBoundingClientRect().width)
      setWidth((current) => (current === next ? current : next))
    }
    measure()
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      const next = Math.round(
        entry?.contentRect?.width ?? node.getBoundingClientRect().width,
      )
      setWidth((current) => (current === next ? current : next))
    })
    observer.observe(node)
    return () => observer.disconnect()
  }, [ref])
  return width
}
