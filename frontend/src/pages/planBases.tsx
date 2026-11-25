/**
 * Adopting a programme: **the four, and the choice**.
 *
 * A programme is **an intent**, not a baseline: it is what the athlete
 * adopts, and **the app never suggests one**. The four programmes of
 * the catalogue are offered, and the athlete picks.
 *
 * ## The entry requirement, beside the athlete's own mileage
 *
 * The tier's entry requirement is the book's own — "this level asks for
 * 40.2-56.3 km a week" — and the athlete's own mileage is theirs — "your
 * last 2 weeks: 6.3, 52.1 km". The two stand **side by side, unranked**:
 * the app states them and never compares them, never marks one as the
 * right one, and never says which level fits.
 *
 * ## The 80/20 base is not here
 *
 * Its schedule has not been read cell by cell, so there is no 80/20
 * template in the catalogue: **the picker holds the four programmes and
 * nothing else**. The app holds one base at a time.
 *
 * ## The adoption
 *
 * The paces are worked out **once**, here, from the threshold in force
 * on the day of the adoption, and they are stored with the plan. The
 * card shows the plan that is in force: its programme, its tier, its
 * dates, what it was paced with, and the entry requirement beside the
 * athlete's own mileage.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { t } from '../i18n'
import { api } from '../lib/api'
import { ApiError } from '../lib/transport'
import { formatDateMedium } from '../lib/format'
import type { AdoptionInsight, BaseRow } from '../lib/types'
import { PageHeader } from '../ui/layout/PageHeader'

/**
 * One programme, with the entry requirement beside the athlete's own
 * mileage.
 *
 * **Two facts, side by side, unranked.** The requirement is the
 * programme's own line and the mileage is the athlete's own; the card
 * states them and never compares them.
 */
function ProgrammeCard({
  base,
  mileage,
  chosen,
  onChoose,
}: {
  base: BaseRow
  /** The athlete's own mileage, in the core's own words. */
  mileage: string | null
  /** Whether this programme is the one the athlete is looking at. */
  chosen: boolean
  /** The athlete's act of choosing it. */
  onChoose: () => void
}) {
  return (
    <div
      className={`rounded-md border px-3 py-2.5 ${chosen ? 'border-brand-600' : 'border-edge'}`}
      data-testid={`plan-base-${base.id}`}
    >
      <div className="flex items-baseline justify-between gap-2">
        <span className="text-[14px] font-semibold text-ink" data-testid={`plan-base-label-${base.id}`}>
          {base.label}
        </span>
        <span className="text-[11.5px] text-ink-muted">{base.method}</span>
      </div>
      <p className="mt-0.5 text-[12px] text-ink-muted">
        {base.distance.text} · {t('plan.bases.weeks', { n: base.weeks })}
      </p>
      {/*
        The entry requirement and the athlete's own mileage: **two facts,
        side by side, unranked**. The app never compares them.
      */}
      <dl className="mt-2 space-y-1" data-testid={`plan-base-entry-${base.id}`}>
        <div>
          <dt className="text-[11px] text-ink-muted">{t('plan.bases.entry')}</dt>
          <dd className="text-[12.5px] text-ink-soft">{base.entry_line ?? ''}</dd>
        </div>
        {mileage ? (
          <div>
            <dt className="text-[11px] text-ink-muted">{t('plan.bases.mileage')}</dt>
            <dd className="text-[12.5px] text-ink-soft" data-testid={`plan-base-mileage-${base.id}`}>
              {mileage}
            </dd>
          </div>
        ) : null}
      </dl>
      <button
        type="button"
        className="tap mt-2 text-[12px] font-semibold text-brand-700 underline"
        data-testid={`plan-base-pick-${base.id}`}
        onClick={onChoose}
      >
        {t('plan.bases.pick')}
      </button>
    </div>
  )
}

/**
 * The plan that is in force.
 *
 * **The plan's own facts**: its programme, its tier, its dates, what it
 * was paced with, and the entry requirement beside the athlete's own
 * mileage.
 */
function AdoptedPlan({ adoption }: { adoption: AdoptionInsight }) {
  const plan = adoption.plan
  if (!plan) return null
  return (
    <section className="card mb-3 px-4 py-3" data-testid="plan-adopted">
      <h2 className="caps-sm">{t('plan.bases.adopted')}</h2>
      <p className="mt-1 text-[16px] font-semibold text-ink" data-testid="plan-adopted-label">
        {plan.label}
      </p>
      <dl className="mt-1.5 space-y-1 text-[12.5px]">
        <div className="flex items-baseline justify-between gap-3">
          <dt className="text-ink-muted">{t('plan.bases.race')}</dt>
          <dd className="text-ink" data-testid="plan-adopted-race">
            {formatDateMedium(plan.race_date)}
          </dd>
        </div>
        <div className="flex items-baseline justify-between gap-3">
          <dt className="text-ink-muted">{t('plan.bases.start')}</dt>
          <dd className="text-ink" data-testid="plan-adopted-start">
            {formatDateMedium(plan.start_date)}
          </dd>
        </div>
        {plan.pacing.threshold_line ? (
          <div className="flex items-baseline justify-between gap-3">
            <dt className="text-ink-muted">{t('plan.week.pacedWith')}</dt>
            <dd className="text-ink" data-testid="plan-adopted-paced">
              {plan.pacing.threshold_line}
            </dd>
          </div>
        ) : plan.pacing.rule_line ? (
          <div className="flex items-baseline justify-between gap-3">
            <dt className="text-ink-muted">{t('plan.week.pacedWith')}</dt>
            <dd className="text-ink" data-testid="plan-adopted-paced">
              {plan.pacing.rule_line}
            </dd>
          </div>
        ) : null}
        {plan.entry.requirement ? (
          <div className="flex items-baseline justify-between gap-3">
            <dt className="text-ink-muted">{t('plan.bases.entry')}</dt>
            <dd className="text-ink">{plan.entry.requirement}</dd>
          </div>
        ) : null}
        {plan.entry.athlete ? (
          <div className="flex items-baseline justify-between gap-3">
            <dt className="text-ink-muted">{t('plan.bases.mileage')}</dt>
            <dd className="text-ink" data-testid="plan-adopted-mileage">
              {plan.entry.athlete}
            </dd>
          </div>
        ) : null}
      </dl>
    </section>
  )
}

/**
 * The page: the four programmes, and the adoption.
 */
export function PlanBasesPage() {
  const bases = useQuery({ queryKey: ['plan-bases'], queryFn: () => api.planBases() })
  const adoption = useQuery({ queryKey: ['plan-adopt'], queryFn: () => api.planAdopted() })
  const client = useQueryClient()
  const [chosen, setChosen] = useState<string | null>(null)
  const [raceDate, setRaceDate] = useState('')
  const [startDate, setStartDate] = useState('')

  const adopt = useMutation({
    mutationFn: (body: Parameters<typeof api.adoptPlan>[0]) => api.adoptPlan(body),
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: ['plan-adopt'] })
      void client.invalidateQueries({ queryKey: ['plan-bases'] })
    },
  })

  const list = bases.data?.bases ?? []
  const failure = adopt.error instanceof ApiError ? adopt.error : null

  return (
    <div>
      <PageHeader title={t('plan.bases.heading')} />
      {adoption.data ? <AdoptedPlan adoption={adoption.data} /> : null}

      {bases.isError ? (
        <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
      ) : null}
      {bases.data?.unavailable ? (
        <p className="text-[13px] text-ink-muted" data-testid="plan-bases-unavailable">
          {bases.data.unavailable}
        </p>
      ) : null}
      {!bases.data ? (
        <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
      ) : (
        <div className="grid gap-2 sm:grid-cols-2" data-testid="plan-bases">
          {list.map((base) => (
            <ProgrammeCard
              key={base.id}
              base={base}
              mileage={bases.data?.athlete_mileage ?? null}
              chosen={chosen === base.id}
              onChoose={() => setChosen(base.id)}
            />
          ))}
        </div>
      )}

      {chosen ? (
        <section className="card mt-3 px-4 py-3" data-testid="plan-adopt-form">
          <h2 className="caps-sm">{t('plan.bases.pick')}</h2>
          <div className="mt-1.5 flex flex-wrap items-end gap-3">
            <label className="flex flex-col gap-1 text-[11.5px] text-ink-muted">
              {t('plan.bases.race')}
              <input
                type="date"
                className="tap rounded-md border border-edge bg-surface px-2 py-1 text-[13px] text-ink"
                value={raceDate}
                data-testid="plan-adopt-race-date"
                onChange={(event) => setRaceDate(event.target.value)}
              />
            </label>
            <label className="flex flex-col gap-1 text-[11.5px] text-ink-muted">
              {t('plan.bases.start')}
              <input
                type="date"
                className="tap rounded-md border border-edge bg-surface px-2 py-1 text-[13px] text-ink"
                value={startDate}
                data-testid="plan-adopt-start-date"
                onChange={(event) => setStartDate(event.target.value)}
              />
            </label>
            <button
              type="button"
              className="tap rounded-md bg-brand-600 px-4 py-1.5 text-[13px] font-semibold text-white disabled:opacity-50"
              data-testid="plan-adopt-submit"
              disabled={adopt.isPending || raceDate === ''}
              onClick={() => {
                adopt.mutate({
                  programme_id: chosen,
                  race_date: raceDate,
                  ...(startDate ? { start_date: startDate } : {}),
                })
              }}
            >
              {t('plan.bases.pick')}
            </button>
          </div>
          {failure ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="plan-adopt-refused">
              {failure.message}
            </p>
          ) : null}
        </section>
      ) : null}
    </div>
  )
}
