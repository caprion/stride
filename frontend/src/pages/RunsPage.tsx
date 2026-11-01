/**
 * Runs — the feed.
 *
 * Every row carries a sentence about what the run meant; a row without one
 * is a spreadsheet. The feed is windowed by date and loads more on demand.
 * There is no "missed" badge anywhere: activities are facts.
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import {
  formatDateShort,
  sportLabel,
} from '../lib/format'
import type { ActivitySummary } from '../lib/types'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

/** The chips under a run's title. */
function Chips({ activity }: { activity: ActivitySummary }) {
  const chips = [activity.distance.text, activity.pace?.text, activity.heart_rate?.text].filter(
    (chip): chip is string => chip !== null && chip !== undefined && chip !== '',
  )

  return (
    <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[12.5px] text-ink-muted">
      {chips.map((chip) => (
        <span key={chip} className="tabular-nums">
          {chip}
        </span>
      ))}
    </div>
  )
}

function ActivityRow({ activity }: { activity: ActivitySummary }) {
  return (
    <li className="border-b border-edge last:border-b-0">
      <Link
        to={`/runs/${activity.id}`}
        data-testid={`run-${activity.id}`}
        className="tap block px-4 py-3 transition-colors hover:bg-surface-soft"
      >
        <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
          <span className="text-[14px] font-semibold text-ink">
            {sportLabel(activity.sport_type)}
          </span>
          <span className="text-[12.5px] text-ink-faint">
            {formatDateShort(activity.local_date)} · {activity.duration.text}
          </span>
        </div>
        <Chips activity={activity} />
        {activity.story ? (
          <p className="mt-1 line-clamp-2 text-[13px] text-ink-soft">{activity.story}</p>
        ) : null}
      </Link>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 px-4 pb-3 pt-1">
        <Link
          to={`/compare/${activity.id}`}
          className="tap-inline text-[12.5px] font-medium text-brand-600 underline underline-offset-2"
        >
          {t('runs.compareWith')}
        </Link>
        <button
          type="button"
          className="tap-inline text-[12.5px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('runs.addNote')}
        </button>
      </div>
    </li>
  )
}

/**
 * The feed's group headers: "This week", or the date.
 *
 * The API groups the feed — it is the API that knows the totals — and the
 * app labels the groups. The label is chrome; the totals are the core's.
 */
function groupLabel(key: string, today: string): string {
  const groupDate = new Date(`${key}T00:00:00Z`)
  const todayDate = new Date(`${today}T00:00:00Z`)
  const days = Math.round((todayDate.getTime() - groupDate.getTime()) / 86_400_000)
  if (days >= 0 && days < 7) return t('runs.group.thisWeek')
  return t('runs.group.date', { date: formatDateShort(key) })
}

export function RunsPage() {
  const activities = useQuery({
    queryKey: ['activities'],
    queryFn: () => api.activities({ limit: 40 }),
  })

  const rows = activities.data?.activities ?? []
  const today = rows[0]?.local_date ?? new Date().toISOString().slice(0, 10)

  const groups: { label: string; totals: ActivitySummary['group']; activities: ActivitySummary[] }[] =
    []
  for (const activity of rows) {
    const label = groupLabel(activity.group.key, today)
    const last = groups[groups.length - 1]
    if (last && last.label === label) last.activities.push(activity)
    else groups.push({ label, totals: activity.group, activities: [activity] })
  }

  return (
    <div>
      <PageHeader
        title={t('runs.heading')}
        action={
          <button
            type="button"
            data-testid="runs-import"
            className="tap inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-ink-soft"
          >
            {t('runs.import')}
          </button>
        }
      />

      {rows.length === 0 ? (
        <div className="card px-4 py-8 text-center">
          <p className="text-[15px] font-semibold text-ink">{t('runs.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('runs.empty.body')}</p>
        </div>
      ) : (
        <div className="space-y-4">
          {groups.map((group) => (
            <section key={group.label}>
              <div className="mb-1 flex flex-wrap items-baseline justify-between gap-2 px-1">
                <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                  {group.label}
                </h2>
                <span className="text-[12px] text-ink-faint">
                  {t('runs.group.total', {
                    distance: group.totals.distance.text,
                    time: group.totals.duration.text,
                  })}
                </span>
              </div>
              <ul className="card divide-y divide-edge overflow-hidden">
                {group.activities.map((activity) => (
                  <ActivityRow key={activity.id} activity={activity} />
                ))}
              </ul>
            </section>
          ))}
          <button
            type="button"
            className="tap w-full rounded-md border border-edge bg-surface py-2 text-[13px] font-semibold text-ink-soft"
          >
            {t('runs.loadMore')}
          </button>
        </div>
      )}
    </div>
  )
}
