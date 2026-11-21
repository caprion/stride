/**
 * The zone ribbon.
 *
 * The athlete's own zones, drawn as one bar: each band is a slice, and
 * the slice is as wide as the time the run spent in it. It is the run's
 * own picture of where its heart rate sat — not a grade, and not a
 * target.
 *
 * **The words are the core's**: the band names, the ranges and the times
 * are the api's own, and the app shows them as they arrive. The only
 * thing the app works out is the **width** of a slice, which is geometry
 * — a part of a bar — and it is worked out from the same numbers the
 * words were rendered from.
 *
 * A run with no heart-rate samples has no ribbon: the answer says why,
 * and the app draws nothing rather than an empty bar.
 */

import { t } from '../i18n'
import { renderedText } from '../lib/format'
import type { FitnessZones } from '../lib/types'

/**
 * The five bands, in the app's own colours: azure to amber.
 *
 * The same scale the run detail draws its bands with — the app's own
 * palette, named — so the ribbon and the chart say the same thing about
 * the same run.
 */
const BAND_CLASSES = [
  'bg-brand-200',
  'bg-brand-300',
  'bg-brand-500',
  'bg-brand-600',
  'bg-accent-500',
]

export function ZoneRibbon({ zones }: { zones: FitnessZones | null | undefined }) {
  const bands = zones?.zones ?? []
  const withTime = bands.filter((band) => (band.time?.raw ?? 0) > 0)
  // No time in any band: there is no ribbon to draw.
  if (withTime.length === 0) return null

  const total = withTime.reduce((sum, band) => sum + (band.time?.raw ?? 0), 0)

  return (
    <div className="mt-3" data-testid="zone-ribbon">
      <p className="caps">
        {t('today.hero.zones')}
      </p>
      <div className="mt-1 flex h-2.5 w-full overflow-hidden rounded-full bg-brand-50">
        {bands.map((band, index) => {
          const seconds = band.time?.raw ?? 0
          if (seconds <= 0) return null
          return (
            <span
              key={band.label}
              data-testid={`zone-ribbon-${band.label}`}
              title={`${band.label} · ${renderedText(band.time)}`}
              className={BAND_CLASSES[index % BAND_CLASSES.length]}
              style={{ width: `${(100 * seconds) / total}%` }}
            />
          )
        })}
      </div>
      {/*
        The legend: the bands that carry time, with the core's own words
        for it. A band with no time is not in the legend — there is
        nothing to say about it.
      */}
      <p className="mt-1.5 text-[12px] text-ink-muted" data-testid="zone-ribbon-legend">
        {withTime.map((band) => `${band.label} ${renderedText(band.time)}`).join(' · ')}
      </p>
      {zones?.outside ? (
        <p className="mt-0.5 text-[11.5px] text-ink-faint">
          {t('activity.zones.missing')} · {renderedText(zones.outside)}
        </p>
      ) : null}
    </div>
  )
}
