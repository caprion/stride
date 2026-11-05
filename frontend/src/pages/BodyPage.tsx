/**
 * Body — the evidence behind the morning readout.
 *
 * The same three signals Today shows, with the history behind them and
 * the band they are measured against. The patterns live here and nowhere
 * else: they are facts about the past, and they always carry their
 * sample size.
 *
 * The pattern sentences are the core's (core-contract §8) — "your HRV has
 * been below your usual for 3 nights" — and they are shown as they
 * arrive. The app does not phrase them again.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { api } from '../lib/api'
import { useStaleness } from '../lib/staleness'
import { t } from '../i18n'
import { VitalReadings } from '../components/VitalReadings'
import { SleepComposition } from '../components/SleepComposition'
import { QueryError } from '../components/QueryError'
import { PageHeader } from '../ui/layout/PageHeader'
import { DateNavigator } from '../components/DateNavigator'
import { runsPath, validDate } from '../lib/history'

export function BodyPage() {
  const [params, setParams] = useSearchParams()
  const date = validDate(params.get('date'))
  const setDate = (next: string) => {
    setParams((previous) => {
      if (next) previous.set('date', next)
      else previous.delete('date')
      return previous
    })
  }
  const [days, setDays] = useState(28)
  const vitals = useQuery({
    queryKey: date || days !== 28 ? ['physiology-vitals', date, days] : ['physiology-vitals'],
    queryFn: () => api.vitals({ date: date || undefined, days }),
  })
  const stale = useStaleness()
  const snapshot = vitals.data

  if (vitals.isError && !snapshot) {
    return (
      <div>
        <PageHeader title={t('body.heading')} />
        <section className="card px-4 py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void vitals.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      </div>
    )
  }

  if (!snapshot) {
    return (
      <div>
        <PageHeader title={t('body.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const controls = (
    <>
      <DateNavigator date={date || snapshot.date} onDate={setDate} />
      <div className="mb-3 flex flex-wrap items-end gap-3">
        <label className="flex flex-col gap-1 text-[13px] text-ink-muted">
          {t('body.history.range')}
          <select
            value={days}
            onChange={(event) => setDays(Number(event.target.value))}
            className="tap rounded-md border border-edge bg-surface px-2 text-ink"
          >
            {[7, 28, 90, 365].map((range) => (
              <option key={range} value={range}>
                {t('body.history.days', { n: range })}
              </option>
            ))}
          </select>
        </label>
        {date ? (
          <button
            className="tap-inline text-[13px] font-medium text-brand-700 underline"
            type="button"
            onClick={() => setDate('')}
          >
            {t('body.history.latest')}
          </button>
        ) : null}
        <Link
          to={runsPath(date || snapshot.date)}
          className="tap-inline ml-auto text-[13px] font-medium text-brand-700 underline"
        >
          {t('body.history.runs')}
        </Link>
      </div>
    </>
  )

  // No wellness yet: the screen says what is missing and what to do,
  // and it draws no chart of nothing.
  if (
    !snapshot.vitals.some((vital) => vital.value_text !== null) &&
    !Object.values(snapshot.history).some((history) =>
      history.some((point) => Number.isFinite(point.value)),
    )
  ) {
    return (
      <div>
        <PageHeader
          title={t('body.heading')}
          meta={
            <>
              {t('body.question')}
              {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
            </>
          }
        />
        {controls}
        <div className="card px-4 py-8 text-center" data-testid="body-empty">
          <p className="text-[15px] font-semibold text-ink">{t('body.state.empty')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('body.state.emptyBody')}</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <PageHeader
        title={t('body.heading')}
        meta={
          <>
            {t('body.question')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />
      {controls}
      {vitals.isError ? (
        <QueryError
          retry={() => {
            void vitals.refetch()
          }}
        />
      ) : null}
      <SleepComposition date={snapshot.date} />

      {/* --------------------------------------------- the three signals */}
      <VitalReadings snapshot={snapshot} layout="cards" onDate={setDate} />

      {/* --------------------------------------------- how to read this */}
      <section className="card mt-3 px-4 py-3">
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('body.howToRead.heading')}
        </h2>
        <p className="mt-1 text-[13.5px] text-ink-soft">{t('body.howToRead.band')}</p>
        <p className="text-[13.5px] text-ink-soft">{t('body.howToRead.inside')}</p>
      </section>

      {/* ------------------------------------------------------ patterns */}
      {snapshot.patterns.length > 0 ? (
        <section className="card mt-3 px-4 py-3" data-testid="body-patterns">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('body.patterns.heading')}
          </h2>
          <ul className="mt-1.5 space-y-1.5">
            {snapshot.patterns.map((pattern) => (
              <li key={pattern.id} className="text-[14px] text-ink-soft">
                {pattern.text}
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </div>
  )
}
