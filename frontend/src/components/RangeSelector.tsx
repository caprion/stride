/**
 * A range selector.
 *
 * A year of daily points is not a chart, it is a smear — and on a phone it
 * is more points than pixels. Every trend in the app gets one of these, so
 * "how far back am I looking" is the same control everywhere.
 *
 * Oura and Whoop do exactly this, and the reason is not fashion: a year of
 * sleep bars cannot be read at any width.
 */

import { SegmentedControl } from './ui'

export const RANGE_OPTIONS = [
  { label: '30 days', value: 30 },
  { label: '3 months', value: 90 },
  { label: 'Year', value: 365 },
]

export function RangeSelector({
  value,
  onChange,
  className,
}: {
  value: number
  onChange: (value: number) => void
  className?: string
}) {
  return (
    <SegmentedControl
      options={RANGE_OPTIONS}
      value={value}
      onChange={(next) => onChange(next as number)}
      className={className}
    />
  )
}

/** The date `days` before today, as YYYY-MM-DD. */
export function sinceDate(days: number): string {
  const date = new Date()
  date.setDate(date.getDate() - days)
  return date.toISOString().slice(0, 10)
}
