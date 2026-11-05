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
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import { formatDateShort, renderedText, sportLabel } from '../lib/format'
import { useStaleness } from '../lib/staleness'
import type { ActivitySummary } from '../lib/types'
import { t } from '../i18n'
import { FitImport } from '../components/FitImport'
import { RouteMap } from '../components/RouteMap'
import { PageHeader } from '../ui/layout/PageHeader'
import { QueryError } from '../components/QueryError'

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

function ActivityRow({
  activity,
  selected,
  disabled,
  toggle,
}: {
  activity: ActivitySummary
  selected: boolean
  disabled: boolean
  toggle: () => void
}) {
  return (
    <li className="border-b border-edge last:border-b-0">
      <Link
        to={`/runs/${activity.id}`}
        data-testid={`run-${activity.id}`}
        className="tap block px-4 py-3 transition-colors hover:bg-surface-soft"
      >
        {/*
          The row, and — at the wide widths, where there is room for it —
          the route as a thumbnail. The thumbnail is the API's points
          drawn small; there is no map server behind it.
        */}
        <div className="flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
              <span className="text-[14px] font-semibold text-ink">
                {activity.name || sportLabel(activity.sport_type)}
              </span>
              <span className="text-[12.5px] text-ink-faint">
                {formatDateShort(activity.local_date)} · {renderedText(activity.duration)}
              </span>
            </div>
            <Chips activity={activity} />
            {/*
              The story is never empty: the core always has a sentence,
              and a row without one is the spreadsheet the plan forbids.
            */}
            <p className="mt-1 line-clamp-2 text-[13px] text-ink-soft">{activity.story}</p>
          </div>
          {activity.geo_points.length > 1 ? (
            <span className="hidden w-[104px] shrink-0 lg:block" data-testid="run-route">
              <RouteMap points={activity.geo_points} variant="thumb" />
            </span>
          ) : null}
        </div>
      </Link>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 px-4 pb-3 pt-1">
        <label className="tap-inline gap-2 text-[12.5px] text-ink-muted">
          <input
            type="checkbox"
            checked={selected}
            disabled={disabled}
            onChange={toggle}
            aria-label={t('runs.select', { date: formatDateShort(activity.local_date) })}
          />
          {t('compare.heading')}
        </label>
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
  const [search, setSearch] = useState('')
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [selected, setSelected] = useState<number[]>([])
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

  const loaded = feed.data?.pages.flatMap((page) => page.activities) ?? []
  const rows = loaded.filter(
    (activity) =>
      (!from || activity.local_date >= from) &&
      (!to || activity.local_date <= to) &&
      `${activity.name} ${sportLabel(activity.sport_type)} ${activity.local_date}`
        .toLowerCase()
        .includes(search.trim().toLowerCase()),
  )
  const today = loaded[0]?.local_date ?? new Date().toISOString().slice(0, 10)

  const groups: {
    label: string
    totals: ActivitySummary['group']
    activities: ActivitySummary[]
  }[] = []
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

      {loaded.length > 0 ? (
        <section className="card mb-3 px-4 py-3">
          <div className="run-filters">
            <label className="flex min-w-0 flex-col gap-1 text-[12px] text-ink-muted">
              {t('runs.search')}
              <input
                type="search"
                className="tap min-w-0 rounded-md border border-edge bg-surface px-3 text-[14px] text-ink"
                value={search}
                onChange={(event) => setSearch(event.target.value)}
              />
            </label>
            <label className="flex min-w-0 flex-col gap-1 text-[12px] text-ink-muted">
              {t('runs.from')}
              <input
                type="date"
                className="tap min-w-0 rounded-md border border-edge bg-surface px-2 text-ink"
                value={from}
                onChange={(event) => setFrom(event.target.value)}
              />
            </label>
            <label className="flex min-w-0 flex-col gap-1 text-[12px] text-ink-muted">
              {t('runs.to')}
              <input
                type="date"
                className="tap min-w-0 rounded-md border border-edge bg-surface px-2 text-ink"
                value={to}
                onChange={(event) => setTo(event.target.value)}
              />
            </label>
          </div>
          <p className="mt-2 text-[12px] text-ink-muted">{t('runs.filterScope')}</p>
        </section>
      ) : null}
      {selected.length > 0 ? (
        <section
          className="compare-tray card mb-3 flex flex-wrap items-center gap-3 border-brand-200 bg-brand-50 px-4 py-2"
          aria-label={t('runs.compareSelected')}
        >
          <p className="mr-auto text-[13px] font-semibold text-brand-700">
            {t('runs.selected', { n: selected.length })}
          </p>
          {selected.length === 2 ? (
            <Link
              className="tap-inline rounded-md bg-brand-600 px-3 text-[13px] font-semibold text-white"
              to={`/compare/${selected[0]}?with=${selected[1]}`}
            >
              {t('runs.compareSelected')}
            </Link>
          ) : null}
          <button
            type="button"
            className="tap text-[12px] text-brand-700 underline"
            onClick={() => setSelected([])}
          >
            {t('runs.clearSelection')}
          </button>
        </section>
      ) : null}
      {/* The import entry: a picker on the phone, a drop target on the
          desktop. */}
      <div className="mb-3">
        <FitImport />
      </div>

      {feed.isError ? (
        <QueryError
          retry={() => {
            void feed.refetch()
          }}
        />
      ) : null}
      {feed.isPending ? (
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      ) : loaded.length === 0 && !feed.isError ? (
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
      ) : loaded.length > 0 ? (
        <div className="space-y-4">
          {rows.length === 0 ? (
            <p className="card px-4 py-5 text-[14px] text-ink-muted">{t('runs.noMatches')}</p>
          ) : null}
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
                  <ActivityRow
                    key={activity.id}
                    activity={activity}
                    selected={selected.includes(activity.id)}
                    disabled={selected.length === 2 && !selected.includes(activity.id)}
                    toggle={() =>
                      setSelected((previous) =>
                        previous.includes(activity.id)
                          ? previous.filter((id) => id !== activity.id)
                          : [...previous, activity.id].slice(0, 2),
                      )
                    }
                  />
                ))}
              </ul>
            </section>
          ))}
          {feed.hasNextPage ? (
            <button
              type="button"
              className="tap w-full rounded-md border border-edge bg-surface py-2 text-[13px] font-semibold text-ink-soft"
              data-testid="runs-load-more"
              disabled={feed.isFetchingNextPage}
              onClick={() => {
                feed.fetchNextPage()
              }}
            >
              {t('runs.loadMore')}
            </button>
          ) : null}
        </div>
      ) : null}
    </div>
  )
}
