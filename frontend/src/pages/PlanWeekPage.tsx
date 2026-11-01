/**
 * The Plan week.
 *
 * The plan is the intent the engine measures against, so it has to be
 * inspectable — but it is not a place the athlete lives in. It is reached
 * from Today and from the feed, and it goes back where it came from.
 *
 * The status of a session is a plain fact: done · planned · not done. There
 * is no red, and the word "missed" is not in the app's vocabulary.
 */

import { useQuery } from '@tanstack/react-query'
import { format, parseISO } from 'date-fns'
import { api } from '../lib/api'
import { formatDateMedium, formatDistance } from '../lib/format'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
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
  const data = plan.data

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
  // The day that is today, when the week contains it; otherwise the first
  // day of the week.
  const now = new Date().toISOString().slice(0, 10)
  const today = days.includes(now) ? now : (days[0] as string)

  return (
    <div>
      <PageHeader title={t('plan.heading')} meta={formatDateMedium(data.week_start)} />

      {/* -------------------------------------------- the source banner */}
      <p
        className="card px-4 py-2.5 text-[13.5px] font-medium text-ink-soft"
        data-testid="plan-source"
      >
        {sourceText(data.source)}
        {data.block_name ? <span className="text-ink-faint"> · {data.block_name}</span> : null}
      </p>

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
                  {planned?.target_distance_m
                    ? (planned.target_distance_m / 1000).toFixed(0)
                    : ''}
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
                {planned.actual_distance_m !== null
                  ? ` · ${t('plan.days.actual', { distance: formatDistance(planned.actual_distance_m) })}`
                  : ''}
              </p>
            </>
          )
        })()}
      </section>

      {/* --------------------------------------------------- the adherence */}
      {data.adherence ? (
        <section className="card mt-3 px-4 py-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('plan.adherence.heading')}
          </h2>
          <p className="mt-1 text-[17px] font-semibold text-ink">
            {t('plan.adherence.count', {
              done: data.adherence.done,
              planned: data.adherence.planned,
            })}
          </p>
        </section>
      ) : null}
    </div>
  )
}
