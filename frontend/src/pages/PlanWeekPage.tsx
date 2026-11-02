/**
 * The Plan week.
 *
 * The plan is the intent the engine measures against, so it has to be
 * inspectable — but it is not a place the athlete lives in. It is reached
 * from Today and from the feed, and it goes back where it came from.
 *
 * The status of a session is a plain fact: done · planned · not done.
 * There is no red, and the word "missed" is not in the app's vocabulary.
 *
 * The states:
 *
 *   - **no plan at all** — the screen says so, and offers the two ways a
 *     plan arrives: imported, or inferred from the athlete's own history;
 *   - **a baseline plan** — named as the source it is. A baseline is what
 *     the athlete usually does, so it never produces a "not done";
 *   - **an imported plan** — named as the source it is;
 *   - **paused** — the app says the plan is paused, in its own voice.
 */

import { useQuery } from '@tanstack/react-query'
import { format, parseISO } from 'date-fns'
import { api } from '../lib/api'
import { ApiError } from '../lib/transport'
import { formatDateMedium, formatDateShort, renderedText } from '../lib/format'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
import { PlanImport } from '../components/PlanImport'
import { PageHeader } from '../ui/layout/PageHeader'

/** The source banner's wording. */
function sourceText(source: PlanSession['source']): string {
  switch (source) {
    case 'baseline':
      return t('plan.source.baseline')
    case 'prescribed':
      return t('plan.source.prescribed')
    default:
      return t('plan.source.typed')
  }
}

/** Why the plan is the plan it is. */
function sourceWhy(source: PlanSession['source']): string {
  switch (source) {
    case 'baseline':
      return t('plan.source.why.baseline')
    case 'prescribed':
      return t('plan.source.why.prescribed')
    default:
      return t('plan.source.why.typed')
  }
}

/** The plain status of a session. */
function statusText(status: PlanSession['status']): string {
  switch (status) {
    case 'done':
      return t('plan.status.done')
    case 'not_done':
      return t('plan.status.notDone')
    default:
      return t('plan.status.planned')
  }
}

/** The seven days of the week, in order. */
function weekDays(start: string): string[] {
  const first = parseISO(start)
  return Array.from({ length: 7 }, (_, index) =>
    format(new Date(first.getTime() + index * 86_400_000), 'yyyy-MM-dd'),
  )
}

const DOT: Record<PlanSession['status'], string> = {
  done: 'bg-brand-500',
  planned: 'bg-accent-400',
  not_done: 'bg-ink-faint',
}

export function PlanWeekPage() {
  const plan = useQuery({ queryKey: ['plan-week'], queryFn: api.planWeek })
  const settings = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const data = plan.data

  // The plan is not there: the screen says so, in the app's voice, and
  // offers the two ways a plan arrives. A 404 on this route is the
  // router saying "there is no plan" — the phone's bridge carries the
  // status and not the code, so both are read as the same fact.
  const failure = plan.error instanceof ApiError ? plan.error : null
  const noPlan =
    plan.isError && (failure?.kind === 'no_plan' || failure?.kind === 'not_found')

  if (noPlan) {
    return (
      <div>
        <PageHeader title={t('plan.heading')} />
        <div className="card px-4 py-6 text-center" data-testid="plan-none">
          <p className="text-[15px] font-semibold text-ink">{t('plan.state.none')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('plan.state.noneBody')}</p>
          <div className="mt-3 flex justify-center">
            <PlanImport />
          </div>
        </div>
      </div>
    )
  }

  if (!data) {
    return (
      <div>
        <PageHeader title={t('plan.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const days = weekDays(data.week_start)
  const byDate = new Map(data.sessions.map((session) => [session.date, session]))
  // The athlete's day, as the api counts it — not the app's clock.
  const today = days.includes(data.today) ? data.today : (days[0] as string)
  const pause = settings.data?.pause
  const paused = pause
    ? pause.reason
      ? t('common.paused.untilReason', {
          date: formatDateShort(pause.end_date),
          reason: t(
            pause.reason === 'travel'
              ? 'settings.pause.reason.travel'
              : pause.reason === 'sick'
                ? 'settings.pause.reason.sick'
                : 'settings.pause.reason.injured',
          ),
        })
      : t('common.paused.until', { date: formatDateShort(pause.end_date) })
    : null

  return (
    <div>
      <PageHeader title={t('plan.heading')} meta={formatDateMedium(data.week_start)} />

      {/* -------------------------------------------- the source banner */}
      <div
        className="card px-4 py-2.5 text-[13.5px] font-medium text-ink-soft"
        data-testid="plan-source"
      >
        {sourceText(data.source)}
        {data.block_name ? <span className="text-ink-faint"> · {data.block_name}</span> : null}
        <p className="mt-0.5 text-[12.5px] font-normal text-ink-faint">
          {sourceWhy(data.source)}
        </p>
      </div>

      {/* -------------------------------------------------- the week strip */}
      <section className="card mt-3 px-2 py-3" data-testid="plan-week">
        <div className="grid grid-cols-7 gap-1">
          {days.map((day) => {
            const planned = byDate.get(day)
            const letter = format(parseISO(day), 'EEEEE')
            const isToday = day === today
            return (
              <div
                key={day}
                className={`flex min-h-[44px] flex-col items-center justify-center gap-1 rounded-md px-0.5 text-center ${
                  isToday ? 'bg-brand-50' : ''
                }`}
                data-testid={`plan-day-${day}`}
              >
                <span className="text-[12px] font-semibold text-ink-soft">{letter}</span>
                <span
                  className={`h-2 w-2 rounded-full ${planned ? DOT[planned.status] : 'bg-surface-sunken'}`}
                  aria-hidden="true"
                />
                <span className="text-[10.5px] text-ink-faint">
                  {planned?.target ? renderedText(planned.target) : ''}
                </span>
              </div>
            )
          })}
        </div>
      </section>

      {/* -------------------------------------------------- the day detail */}
      <section className="card mt-3 px-4 py-3" data-testid="plan-day-detail">
        {(() => {
          const planned = byDate.get(today)
          if (!planned) {
            return <p className="text-[13.5px] text-ink-muted">{t('today.state.noPlan')}</p>
          }
          return (
            <>
              <p className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                {t('plan.day.today')} · {formatDateMedium(planned.date)}
              </p>
              {/* The session, as the core wrote it. */}
              <p className="mt-1 text-[16px] font-semibold text-ink">
                {planned.sentence}
              </p>
              <p className="mt-0.5 text-[13.5px] text-ink-muted" data-testid="plan-status">
                {statusText(planned.status)}
                {planned.actual !== null
                  ? ` · ${t('plan.days.actual', { distance: renderedText(planned.actual) })}`
                  : ''}
              </p>
            </>
          )
        })()}
      </section>

      {/* ------------------------------------- the counts of the week */}
      {data.counts ? (
        <section className="card mt-3 px-4 py-3" data-testid="plan-counts">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('plan.adherence.heading')}
          </h2>
          {/*
            Three numbers, never a percentage: a percentage hides the
            count, and the count is the fact. **"Not done" is the plain
            fact** — the day went by and the session was not run.
          */}
          <p className="mt-1 flex flex-wrap items-baseline gap-x-3 gap-y-1 text-[15px] font-semibold text-ink">
            <span data-testid="plan-count-done">
              {t('plan.counts.done', { n: data.counts.done })}
            </span>
            <span className="text-ink-muted" data-testid="plan-count-planned">
              {t('plan.counts.planned', { n: data.counts.planned })}
            </span>
            <span className="text-ink-muted" data-testid="plan-count-not-done">
              {t('plan.counts.notDone', { n: data.counts.not_done })}
            </span>
          </p>
          {data.adherence ? (
            <p className="mt-1 text-[13px] text-ink-muted" data-testid="plan-adherence">
              {t('plan.adherence.count', {
                done: data.adherence.done,
                planned: data.adherence.planned,
              })}
            </p>
          ) : null}
        </section>
      ) : null}

      {/* --------------------------------------------- the plan is paused */}
      {paused ? (
        <p className="mt-3 text-[13.5px] text-ink-muted" data-testid="plan-paused">
          {paused}
        </p>
      ) : null}

      {/* ------------------------------------------------ sources, manage */}
      <section className="card mt-3 px-4 py-3" data-testid="plan-sources">
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('plan.sources.heading')}
        </h2>
        <p className="mt-1 text-[13px] text-ink-muted">{sourceWhy(data.source)}</p>
        <div className="mt-2">
          <PlanImport />
        </div>
      </section>
    </div>
  )
}
