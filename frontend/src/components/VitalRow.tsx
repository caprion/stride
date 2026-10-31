/**
 * A vital, as a row.
 *
 * Label, value, direction — in that order, with the unit written out and
 * the arithmetic behind the direction, muted. Nothing is scored, nothing
 * is coloured as a problem, and the row is the same on Today and on Body.
 */

import { formatDeviation, formatDurationHuman } from '../lib/format'
import type { VitalKey, VitalValue } from '../lib/types'
import { t } from '../i18n'
import { DirectionText } from './DirectionText'

const LABEL: Record<
  VitalKey,
  'vital.sleep.label' | 'vital.hrv.label' | 'vital.restingHr.label'
> = {
  sleep: 'vital.sleep.label',
  hrv: 'vital.hrv.label',
  resting_hr: 'vital.restingHr.label',
}

/** The value, with its unit written out. */
export function vitalValue(vital: VitalValue): string {
  if (vital.value === null) return '—'
  if (vital.key === 'sleep') return formatDurationHuman(vital.value)
  if (vital.key === 'hrv') return `${vital.value} ms`
  return `${vital.value} bpm`
}

/** The arithmetic, in words: "14 below your usual". */
export function deviationText(vital: VitalValue): string | null {
  if (vital.deviation === null) return null
  const magnitude = formatDeviation(vital.key, vital.deviation)
  if (magnitude === null) return null
  const above = vital.deviation > 0
  if (vital.key === 'sleep') {
    return t(above ? 'delta.moreThan' : 'delta.lessThan', { value: magnitude })
  }
  return t(above ? 'delta.above' : 'delta.below', { value: magnitude })
}

/** The direction of travel, in words. */
export function travelText(vital: VitalValue): string | null {
  if (vital.travel === 'steady') return t('travel.steady')
  const days = vital.travel_days ?? 1
  if (vital.travel === 'down') {
    return t('travel.nightDown', { n: ordinal(days) })
  }
  if (days <= 1) {
    return t('travel.nightUp', { n: ordinal(1) })
  }
  return t('travel.nightsUp', { n: days })
}

/** "1st", "2nd", "3rd", "4th". */
export function ordinal(value: number): string {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const remainder = value % 100
  const suffix = suffixes[(remainder - 20) % 10] ?? suffixes[remainder] ?? 'th'
  return `${value}${suffix}`
}

export function VitalRow({
  vital,
  children,
}: {
  vital: VitalValue
  children?: React.ReactNode
}) {
  const deviation = deviationText(vital)
  const travel = travelText(vital)

  return (
    <div className="py-3" data-vital={vital.key}>
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
        <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
          {t(LABEL[vital.key])}
        </span>
        <span className="text-[17px] font-semibold text-ink" data-vital-value={vital.key}>
          {vitalValue(vital)}
        </span>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[12.5px]">
        <DirectionText direction={vital.direction} />
        {deviation ? (
          <>
            <span className="text-ink-faint" aria-hidden="true">
              ·
            </span>
            <span className="text-ink-muted">{deviation}</span>
          </>
        ) : (
          <>
            <span className="text-ink-faint" aria-hidden="true">
              ·
            </span>
            <span className="text-ink-muted">{t('today.state.notEnoughHistory')}</span>
          </>
        )}
        {travel ? (
          <>
            <span className="text-ink-faint" aria-hidden="true">
              ·
            </span>
            <span className="text-ink-faint">{travel}</span>
          </>
        ) : null}
      </div>
      {children}
    </div>
  )
}
