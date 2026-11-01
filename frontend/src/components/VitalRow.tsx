/**
 * A vital, as a row.
 *
 * Label, value, direction — in that order, with the arithmetic behind the
 * direction, muted. Nothing is scored, nothing is coloured as a problem,
 * and the row is the same on Today and on Body.
 *
 * Every word on this row is the core's: the label, the value, the
 * direction, the arithmetic and the travel all arrive rendered
 * (core-contract §8). The row is the chrome that holds them.
 */

import type { VitalReading } from '../lib/types'
import { DirectionText } from './DirectionText'

/** The middot that separates the parts of the line. */
function Separator() {
  return (
    <span className="text-ink-faint" aria-hidden="true">
      ·
    </span>
  )
}

export function VitalRow({
  vital,
  children,
}: {
  vital: VitalReading
  children?: React.ReactNode
}) {
  return (
    <div className="py-3" data-vital={vital.key}>
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
        <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
          {vital.label}
        </span>
        <span className="text-[17px] font-semibold text-ink" data-vital-value={vital.key}>
          {vital.value_text ?? '—'}
        </span>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[12.5px]">
        {vital.direction_text ? <DirectionText text={vital.direction_text} /> : null}
        {vital.deviation_text ? (
          <>
            <Separator />
            <span className="text-ink-muted">{vital.deviation_text}</span>
          </>
        ) : null}
        {!vital.direction_text && vital.status_text ? (
          <>
            <Separator />
            <span className="text-ink-muted">{vital.status_text}</span>
          </>
        ) : null}
        {vital.travel_text ? (
          <>
            <Separator />
            <span className="text-ink-faint">{vital.travel_text}</span>
          </>
        ) : null}
      </div>
      {children}
    </div>
  )
}
