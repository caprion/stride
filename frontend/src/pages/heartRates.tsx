/**
 * The athlete's own heart rates.
 *
 * **One small fact, entered once, and the app takes care of it.** The
 * athlete declares their **maximum heart rate**, their **resting heart
 * rate**, or **both** — each with where it came from and, when they know
 * it, the day it was measured. The app derives the bands from the pair and
 * uses them.
 *
 * ## What the card shows
 *
 * * the **declaration in force**: each number, its source and its dates;
 * * the **bands it produced**, and the **watch's own floors** beside them,
 *   with the **difference** stated boundary by boundary — a fact, and never
 *   a correction;
 * * the **history**: the earlier declarations, each with its own date, and
 *   never deleted.
 *
 * **The wording is the core's.** The labels of the two numbers, the five
 * sources, the source lines and the reasons come from the api, which asks
 * the core's catalogue for them. The app writes no sentence of its own
 * here.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { t } from '../i18n'
import { api } from '../lib/api'
import { ApiError } from '../lib/transport'
import { formatDateMedium } from '../lib/format'
import type { HeartRatesInsight } from '../lib/types'

/**
 * The five bands, as the two sets of floors are drawn beside each other.
 *
 * **The difference is a fact.** The card states it and says nothing about
 * which set of floors is the right one.
 */
function BandsPicture({ data }: { data: HeartRatesInsight }) {
  const declared = data.bands
  const watch = data.watch
  if (!declared && !watch) return null
  const difference = data.difference ?? []
  return (
    <div className="mt-2" data-testid="heart-rates-bands">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h3 className="caps-sm">{t('heartRates.card.bands')}</h3>
        {declared ? (
          <span className="text-[11.5px] text-ink-muted" data-testid="heart-rates-bands-source">
            {declared.source ?? ''}
          </span>
        ) : null}
      </div>
      {declared ? (
        <p className="text-[11.5px] text-ink-faint" data-testid="heart-rates-range">
          {declared.range_label}
        </p>
      ) : null}
      <table className="mt-1 w-full text-[12px]">
        <thead>
          <tr className="text-left text-ink-faint">
            <th className="font-medium">{t('heartRates.card.band')}</th>
            <th className="font-medium">{declared?.label ?? ''}</th>
            <th className="font-medium">{watch?.label ?? ''}</th>
            <th className="font-medium">{t('heartRates.card.difference')}</th>
          </tr>
        </thead>
        <tbody>
          {[1, 2, 3, 4, 5].map((zone) => {
            const row = difference.find((item) => item.zone === zone)
            return (
              <tr key={zone} className="border-t border-edge" data-testid={`heart-rates-band-${zone}`}>
                <td className="py-0.5 text-ink-muted">Z{zone}</td>
                <td className="py-0.5 tabular-nums text-ink">
                  {row ? `${row.declared_bpm} bpm` : ''}
                </td>
                <td className="py-0.5 tabular-nums text-ink-muted">
                  {row ? `${row.watch_bpm} bpm` : ''}
                </td>
                <td className="py-0.5 tabular-nums text-ink-muted">
                  {row?.difference?.text ?? ''}
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
      {watch ? (
        <p className="mt-1 text-[11.5px] text-ink-faint" data-testid="heart-rates-watch-source">
          {watch.source ?? ''}
        </p>
      ) : null}
    </div>
  )
}

/**
 * The card.
 *
 * The declaration in force comes first, then the bands it produced beside
 * the watch's own floors, then the two fields the athlete changes, and
 * then the history.
 */
export function HeartRatesCard() {
  const query = useQuery({
    queryKey: ['athlete-heart-rates'],
    queryFn: api.athleteHeartRates,
  })
  const client = useQueryClient()
  const [max, setMax] = useState('')
  const [maxSource, setMaxSource] = useState('typed')
  const [maxMeasured, setMaxMeasured] = useState('')
  const [resting, setResting] = useState('')
  const [restingSource, setRestingSource] = useState('typed')
  const [restingMeasured, setRestingMeasured] = useState('')

  const declare = useMutation({
    mutationFn: (body: Parameters<typeof api.declareHeartRates>[0]) =>
      api.declareHeartRates(body),
    onSuccess: () => {
      setMax('')
      setResting('')
      setMaxMeasured('')
      setRestingMeasured('')
      void client.invalidateQueries({ queryKey: ['athlete-heart-rates'] })
    },
  })

  const data: HeartRatesInsight | undefined = query.data
  const failure = declare.error instanceof ApiError ? declare.error : null
  const sources = data?.sources ?? []
  const kinds = data?.kinds ?? []
  const declaration = data?.declaration ?? null
  // The labels of the two kinds are the core's: "your maximum heart rate"
  // and "your resting heart rate".
  const maxLabel = kinds.find((kind) => kind.key === 'max')?.label ?? ''
  const restingLabel = kinds.find((kind) => kind.key === 'resting')?.label ?? ''

  const save = () => {
    const body: Parameters<typeof api.declareHeartRates>[0] = {}
    const maxValue = Number(max)
    if (max.trim() !== '' && Number.isFinite(maxValue) && maxValue > 0) {
      body.max_heart_rate_bpm = maxValue
      body.max_source = maxSource
      if (maxMeasured) body.max_measured_on = maxMeasured
    }
    const restingValue = Number(resting)
    if (resting.trim() !== '' && Number.isFinite(restingValue) && restingValue > 0) {
      body.resting_heart_rate_bpm = restingValue
      body.resting_source = restingSource
      if (restingMeasured) body.resting_measured_on = restingMeasured
    }
    if (body.max_heart_rate_bpm === undefined && body.resting_heart_rate_bpm === undefined) return
    declare.mutate(body)
  }

  return (
    <div data-testid="heart-rates-card">
      {query.isError && !data ? (
        <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
      ) : null}
      {!data ? (
        <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
      ) : (
        <>
          {/* The declaration in force: the numbers, their sources and
              their dates. */}
          {declaration ? (
            <div className="mb-2" data-testid="heart-rates-in-force">
              {declaration.max ? (
                <div data-testid="heart-rates-max">
                  <p className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
                    {declaration.max.label}
                  </p>
                  <p className="text-[19px] font-semibold tabular-nums text-ink">
                    {declaration.max.value?.text ?? ''}
                  </p>
                  <p className="text-[11.5px] text-ink-muted">
                    {declaration.max.source_label}
                    {declaration.max.measured_on_label
                      ? ` · ${declaration.max.measured_on_label}`
                      : ''}
                  </p>
                </div>
              ) : null}
              {declaration.resting ? (
                <div className="mt-1" data-testid="heart-rates-resting">
                  <p className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
                    {declaration.resting.label}
                  </p>
                  <p className="text-[19px] font-semibold tabular-nums text-ink">
                    {declaration.resting.value?.text ?? ''}
                  </p>
                  <p className="text-[11.5px] text-ink-muted">
                    {declaration.resting.source_label}
                    {declaration.resting.measured_on_label
                      ? ` · ${declaration.resting.measured_on_label}`
                      : ''}
                  </p>
                </div>
              ) : null}
              <p className="mt-1 text-[11.5px] text-ink-faint" data-testid="heart-rates-entered">
                {declaration.entered}
              </p>
            </div>
          ) : (
            <p className="text-[13px] text-ink-muted" data-testid="heart-rates-none">
              {data.unavailable ?? t('heartRates.card.none')}
            </p>
          )}

          {/* The reason there are no bands, when there are none. */}
          {data.unavailable && declaration ? (
            <p className="text-[12px] text-ink-muted" data-testid="heart-rates-unavailable">
              {data.unavailable}
            </p>
          ) : null}

          {/* The bands, and the watch's own floors beside them. */}
          <BandsPicture data={data} />

          {/* The two fields. Either may be left empty. */}
          <div className="mt-3 space-y-2">
            <div className="grid gap-2 sm:grid-cols-3">
              <label className="block">
                <span className="block caps">{maxLabel}</span>
                <input
                  type="number"
                  inputMode="numeric"
                  value={max}
                  data-testid="heart-rates-max-input"
                  onChange={(event) => setMax(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                />
              </label>
              <label className="block">
                <span className="block caps">{t('heartRates.card.source')}</span>
                <select
                  value={maxSource}
                  data-testid="heart-rates-max-source"
                  onChange={(event) => setMaxSource(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                >
                  {sources.map((source) => (
                    <option key={source.key} value={source.key}>
                      {source.label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="block">
                <span className="block caps">{t('heartRates.card.measured')}</span>
                <input
                  type="date"
                  value={maxMeasured}
                  data-testid="heart-rates-max-measured"
                  onChange={(event) => setMaxMeasured(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                />
              </label>
            </div>
            <div className="grid gap-2 sm:grid-cols-3">
              <label className="block">
                <span className="block caps">{restingLabel}</span>
                <input
                  type="number"
                  inputMode="numeric"
                  value={resting}
                  data-testid="heart-rates-resting-input"
                  onChange={(event) => setResting(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                />
              </label>
              <label className="block">
                <span className="block caps">{t('heartRates.card.source')}</span>
                <select
                  value={restingSource}
                  data-testid="heart-rates-resting-source"
                  onChange={(event) => setRestingSource(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                >
                  {sources.map((source) => (
                    <option key={source.key} value={source.key}>
                      {source.label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="block">
                <span className="block caps">{t('heartRates.card.measured')}</span>
                <input
                  type="date"
                  value={restingMeasured}
                  data-testid="heart-rates-resting-measured"
                  onChange={(event) => setRestingMeasured(event.target.value)}
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink"
                />
              </label>
            </div>
            <button
              type="button"
              className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
              disabled={declare.isPending}
              data-testid="heart-rates-save"
              onClick={save}
            >
              {t('heartRates.card.save')}
            </button>
          </div>

          {declare.isSuccess ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="heart-rates-saved">
              {t('heartRates.card.saved')}
            </p>
          ) : null}
          {failure ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="heart-rates-refused">
              {t('heartRates.card.refused')}
            </p>
          ) : null}

          {/* The declarations that came before: the dated history. */}
          {data.history.length > 0 ? (
            <div className="mt-3" data-testid="heart-rates-history">
              <h3 className="caps-sm">{t('heartRates.card.history')}</h3>
              <ul className="mt-1 space-y-0.5">
                {data.history.map((entry) => (
                  <li key={entry.id} className="text-[12px] text-ink-muted">
                    {formatDateMedium(entry.entered_on)}
                    {entry.max?.value?.text ? ` · ${entry.max.value.text}` : ''}
                    {entry.resting?.value?.text ? ` · ${entry.resting.value.text}` : ''}
                    {entry.in_force ? (
                      <span className="ml-1 font-semibold text-brand-700">
                        {t('heartRates.card.inUse')}
                      </span>
                    ) : null}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </>
      )}
    </div>
  )
}
