/**
 * The page header.
 *
 * The title, the date, and — when the data is old — the line that says so.
 * Every screen has one; they all say the same kind of thing.
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
        <h1 className="text-[24px] font-bold leading-tight tracking-tight text-ink">{title}</h1>
        {meta ? <div className="mt-0.5 text-[13px] text-ink-muted">{meta}</div> : null}
      </div>
      {action}
    </header>
  )
}
