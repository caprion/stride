/**
 * The page header.
 *
 * The title, the date, and — when the data is old — the line that says so.
 * Every screen has one; they all say the same kind of thing.
 *
 * **The mock's own header**: a big title in the page's ink, and the
 * quiet line under it that says what the screen is about.
 */

import type { ReactNode } from 'react'

export function PageHeader({
  title,
  meta,
  action,
}: {
  title: string
  meta?: ReactNode
  action?: ReactNode
}) {
  return (
    <header className="mb-4 flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0">
        <h1 className="text-[26px] font-bold leading-tight tracking-[-0.3px] text-ink">
          {title}
        </h1>
        {meta ? <div className="sub">{meta}</div> : null}
      </div>
      {action}
    </header>
  )
}
