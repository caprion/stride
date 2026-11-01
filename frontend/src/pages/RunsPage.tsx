/**
 * Runs — the feed.
 *
 * Every row carries a sentence about what the run meant; a row without one
 * is a spreadsheet. The feed is windowed by date and loads more on demand.
 * There is no "missed" badge anywhere: activities are facts.
 *
 * The states:
 *
 *   - **empty** — "No activities yet", with the two actions that fill it:
 *     the key and the file;
 *   - **importing** — an inline row on the list, never a modal;
 *   - **a duplicate** — one line, once: "Already have this run (12 Oct,
 *     10.2 km)".
 */

import { useInfiniteQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import { formatDateShort, renderedText, sportLabel } from '../lib/format'
import { useStaleness } from '../lib/staleness'
import type { ActivitySummary } from '../lib/types'
import { t } from '../i18n'
import { FitImport } from '../components/FitImport'
import { PageHeader } from '../ui/layout/PageHeader'

/** The chips under a run's title. */
function Chips({ activity }: { activity: ActivitySummary }) {
  const chips = [
    renderedText(activity.distance),
    activity.pace ? renderedText(activity.pace) : null,
    activity.heart_rate ? renderedText(activity.heart_rate) : null,
  ].filter((chip): chip is string => chip !== null && chip !== '')

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
            {formatDateShort(activity.local_date)} · {renderedText(activity.duration)}
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
        <Link
          to={`/runs/${activity.id}`}
          className="tap-inline text-[12.5px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('runs.addNote')}
        </Link>
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
  // The feed is windowed by date: a page of forty runs, and the next
  // page when the athlete asks for it. It never loads everything.
  const feed = useInfiniteQuery({
    queryKey: ['activities'],
    queryFn: ({ pageParam }) =>
      api.activities({ limit: 40, cursor: pageParam as string | undefined }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
  })
  const stale = useStaleness()

  const rows = feed.data?.pages.flatMap((page) => page.activities) ?? []
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
        meta={stale ? <span className="text-ink-faint">{stale}</span> : null}
      />

      {/* The import entry: a picker on the phone, a drop target on the
          desktop. */}
      <div className="mb-3">
        <FitImport />
      </div>

      {rows.length === 0 ? (
        <div className="card px-4 py-8 text-center" data-testid="runs-empty">
          <p className="text-[15px] font-semibold text-ink">{t('runs.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('runs.empty.body')}</p>
          <button
            type="button"
            className="tap mt-3 inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft"
            onClick={() => {
              void connectGarmin()
            }}
          >
            {t('today.state.noDataAction')}
          </button>
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
                    distance: renderedText(group.totals.distance),
                    time: renderedText(group.totals.duration),
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
          {feed.hasNextPage ? (
            <button
              type="button"
              className="tap w-full rounded-md border border-edge bg-surface py-2 text-[13px] font-semibold text-ink-soft"
              data-testid="runs-load-more"
              onClick={() => {
                feed.fetchNextPage()
              }}
            >
              {t('runs.loadMore')}
            </button>
          ) : null}
        </div>
      )}
    </div>
  )
}
