import type { VitalsSnapshot } from '../lib/types'
import { t } from '../i18n'
import { formatDateShort } from '../lib/format'
import { Sparkline } from './Sparkline'
import { VitalRow } from './VitalRow'

const signals = [
  { key: 'sleep', label: 'vital.sleep' },
  { key: 'hrv', label: 'vital.hrv' },
  { key: 'resting_hr', label: 'vital.restingHr' },
] as const

export function VitalReadings({
  snapshot,
  layout,
  muted = false,
}: {
  snapshot: VitalsSnapshot
  layout: 'cards' | 'rows'
  muted?: boolean
}) {
  const readings = signals.map(({ key, label }) => {
    const vital = snapshot.vitals.find((reading) => reading.key === key)
    const history = snapshot.history[key]
    const hasHistory = history.some((point) => Number.isFinite(point.value))
    return {
      key,
      label: vital?.label ?? t(label),
      vital,
      history,
      hasHistory,
      visible: vital?.value_text != null || hasHistory,
    }
  })
  const available = readings.filter((reading) => reading.visible)
  const additional = readings.filter((reading) => !reading.visible)

  return (
    <>
      {available.length > 0 ? (
        <section
          className={`${layout === 'cards' ? 'signal-grid' : 'card mt-3 px-4 py-1'} ${muted ? 'opacity-60' : ''}`}
          data-testid={layout === 'cards' ? 'body-signals' : 'today-vitals'}
        >
          {available.map((reading) => {
            const first = reading.history[0]
            const last = reading.history[reading.history.length - 1]
            const chart = reading.hasHistory ? (
              <div className="mt-2">
                <Sparkline
                  values={reading.history.map((point) => point.value)}
                  band={reading.vital?.band ?? null}
                  height={layout === 'cards' ? 44 : undefined}
                  label={t('body.history.label', { n: reading.history.length })}
                />
                {(layout === 'cards' || reading.vital?.value_text == null) && first && last ? (
                  <p className="mt-2 flex justify-between gap-2 text-[11px] text-ink-muted">
                    <span>{formatDateShort(first.date)}</span>
                    <span>{formatDateShort(last.date)}</span>
                  </p>
                ) : null}
              </div>
            ) : null
            return (
              <div key={reading.key} className={layout === 'cards' ? 'card px-4 py-2' : ''}>
                {reading.vital ? (
                  <VitalRow vital={reading.vital}>{chart}</VitalRow>
                ) : (
                  <div className="py-3" data-vital={reading.key}>
                    <p className="text-[13px] font-semibold text-ink-muted">{reading.label}</p>
                    {chart}
                  </div>
                )}
              </div>
            )
          })}
        </section>
      ) : null}
      {additional.length > 0 ? (
        <details
          className="mt-3 text-[13px] text-ink-muted"
          data-testid={
            available.length === 0 && layout === 'rows'
              ? 'today-no-vitals'
              : 'additional-measurements'
          }
        >
          <summary className="tap-inline cursor-pointer">
            {t('body.additional.heading')}
          </summary>
          <p className="mt-1">
            {available.length === 0 ? t('today.state.noVitals') : null}
            {available.length === 0 ? ' ' : null}
            {t('body.additional.body')}
          </p>
          <ul className="mt-1 space-y-1">
            {additional.map((reading) => (
              <li key={reading.key}>{reading.label}</li>
            ))}
          </ul>
        </details>
      ) : null}
    </>
  )
}
