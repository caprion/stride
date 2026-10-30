/**
 * Feed — the activity feed.
 *
 * A filter bar (sport · sort · free-text search) over a paginated list of
 * activity cards, with a narrow summary column on the right.
 */

import { useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Activity,
  ChevronLeft,
  ChevronRight,
  Flame,
  Mountain,
  Route,
  Search,
  SearchX,
  Timer,
  TrendingUp,
} from 'lucide-react'
import { api } from '../lib/api'
import { ACCENT_DATA } from '../lib/palette'
import {
  formatDistanceShort,
  formatElevation,
  formatNumber,
  formatPace,
  sportLabel,
  SPORT_COLOR,
} from '../lib/format'
import { ActivityCard } from '../components/ActivityCard'
import { ConnectPrompt } from '../components/ConnectPrompt'
import { isPhone } from '../lib/surface'
import {
  Button,
  Card,
  CardHeader,
  EmptyState,
  PageHeader,
  SegmentedControl,
  Skeleton,
  StatTile,
  Toolbar,
  inputClass,
} from '../components/ui'

const PER_PAGE = 12

type SportFilter = 'all' | 'running' | 'trail_running' | 'treadmill_running' | 'walking'
type SortKey = 'date' | 'distance' | 'elevation'

const SPORT_OPTIONS: { label: string; value: SportFilter }[] = [
  { label: 'All', value: 'all' },
  { label: 'Run', value: 'running' },
  { label: 'Trail', value: 'trail_running' },
  { label: 'Treadmill', value: 'treadmill_running' },
  { label: 'Walk', value: 'walking' },
]

const SORT_OPTIONS: { label: string; value: SortKey }[] = [
  { label: 'Recent', value: 'date' },
  { label: 'Distance', value: 'distance' },
  { label: 'Elevation', value: 'elevation' },
]

export function FeedPage() {
  const [sport, setSport] = useState<SportFilter>('all')
  const [sort, setSort] = useState<SortKey>('date')
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')
  const [page, setPage] = useState(1)

  // Debounce the free-text search so we do not hammer the API per keystroke.
  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(search.trim()), 350)
    return () => window.clearTimeout(timer)
  }, [search])

  // Any change of filter returns the user to the first page.
  useEffect(() => {
    setPage(1)
  }, [sport, sort, debounced])

  const sportParam = sport === 'all' ? undefined : sport

  // One chip per run: what it meant, in two or three words.
  const chipsQuery = useQuery({
    queryKey: ['insights-chips', 3650],
    queryFn: () => api.insightsChips(3650),
    retry: false,
    staleTime: 300_000,
  })

  const activitiesQuery = useQuery({
    queryKey: ['activities', page, sportParam ?? 'all', sort, debounced],
    queryFn: () =>
      api.activities({
        page,
        per_page: PER_PAGE,
        sport: sportParam,
        q: debounced || undefined,
        sort,
      }),
  })

  const summaryQuery = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const summary = summaryQuery.data
  const week = summary?.this_week

  const items = activitiesQuery.data?.items ?? []
  const total = activitiesQuery.data?.total ?? 0
  const pages = activitiesQuery.data?.pages ?? 1
  const currentPage = activitiesQuery.data?.page ?? page

  const hasFilters = sport !== 'all' || debounced.length > 0
  const filtersActive = hasFilters || sort !== 'date'

  const clearFilters = () => {
    setSport('all')
    setSort('date')
    setSearch('')
    setDebounced('')
    setPage(1)
  }

  const totalDistance = useMemo(
    () => items.reduce((sum, item) => sum + (item.distance_m ?? 0), 0),
    [items],
  )

  return (
    <div>
      <PageHeader
        title={isPhone() ? 'Runs' : 'Activity feed'}
        description={
          activitiesQuery.isLoading
            ? 'Loading your activities…'
            : `${formatNumber(total)} activit${total === 1 ? 'y' : 'ies'} match your filters`
        }
      />

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_300px] xl:grid-cols-[minmax(0,1fr)_320px]">
        {/* ------------------------------------------------- main column */}
        <div className="min-w-0">
          {/* filter bar */}
          <Card className="mb-4 px-4 py-3">
            <Toolbar>
              <div className="flex min-w-0 flex-1 items-center gap-2">
                <span className="relative block w-full max-w-xs">
                  <Search
                    size={15}
                    strokeWidth={2}
                    className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-ink-faint"
                  />
                  <input
                    type="search"
                    value={search}
                    onChange={(event) => setSearch(event.target.value)}
                    placeholder="Search activities, dates, descriptions…"
                    aria-label="Search activities"
                    className={`${inputClass} pl-9`}
                  />
                </span>
              </div>
              <SegmentedControl
                options={SPORT_OPTIONS}
                value={sport}
                onChange={(value) => setSport(value)}
              />
              <SegmentedControl
                options={SORT_OPTIONS}
                value={sort}
                onChange={(value) => setSort(value)}
              />
            </Toolbar>
          </Card>

          {/* list */}
          {activitiesQuery.isLoading ? (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, index) => (
                <Skeleton key={index} className="h-[132px] w-full rounded-card" />
              ))}
            </div>
          ) : activitiesQuery.isError ? (
            <Card>
              <EmptyState
                icon={<Activity size={22} strokeWidth={1.8} />}
                title="Couldn't load the feed"
                description="The activity service did not respond. Give it another go."
                action={
                  <Button variant="secondary" onClick={() => activitiesQuery.refetch()}>
                    Retry
                  </Button>
                }
              />
            </Card>
          ) : items.length === 0 ? (
            <Card>
              {/*
                Three different empties: the filters hid something, or
                there is nothing of this sport yet, or there is nothing at
                all — and nothing at all is the one that says what to do.
              */}
              {filtersActive ? (
                <EmptyState
                  icon={<SearchX size={22} strokeWidth={1.8} />}
                  title={
                    debounced
                      ? `No activities match “${debounced}”`
                      : `No ${sportLabel(sport).toLowerCase()}s yet`
                  }
                  description={
                    debounced
                      ? 'Try a different keyword, or widen the sport filter to see more.'
                      : 'Nothing of this sport has been synced yet. Try another sport or clear the filters.'
                  }
                  action={
                    <Button variant="secondary" onClick={clearFilters}>
                      Clear filters
                    </Button>
                  }
                />
              ) : (
                <ConnectPrompt what="your runs" />
              )}
            </Card>
          ) : (
            <div className="space-y-3">
              {items.map((activity) => (
                <ActivityCard
                  key={activity.id}
                  activity={activity}
                  showRoute
                  chip={chipsQuery.data?.items[String(activity.id)]}
                />
              ))}
            </div>
          )}

          {/* pagination */}
          {!activitiesQuery.isLoading && !activitiesQuery.isError && items.length > 0 ? (
            <Card className="mt-4 px-4 py-3">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <p className="text-[13px] text-ink-muted">
                  Showing{' '}
                  <span className="font-semibold text-ink">
                    {(currentPage - 1) * PER_PAGE + 1}–
                    {(currentPage - 1) * PER_PAGE + items.length}
                  </span>{' '}
                  of <span className="font-semibold text-ink">{formatNumber(total)}</span> ·{' '}
                  {formatDistanceShort(totalDistance)} km on this page
                </p>
                <div className="flex items-center gap-2">
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={currentPage <= 1}
                    onClick={() => setPage((value) => Math.max(1, value - 1))}
                  >
                    <ChevronLeft size={15} strokeWidth={2} />
                    Prev
                  </Button>
                  <span className="px-1 text-[13px] font-medium text-ink-muted">
                    Page {currentPage} of {pages}
                  </span>
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={currentPage >= pages}
                    onClick={() => setPage((value) => Math.min(pages, value + 1))}
                  >
                    Next
                    <ChevronRight size={15} strokeWidth={2} />
                  </Button>
                </div>
              </div>
            </Card>
          ) : null}
        </div>

        {/* ------------------------------------------------- side column */}
        <aside className="min-w-0 space-y-4">
          <Card className="px-5 py-5">
            <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
              This week
            </p>
            {summaryQuery.isLoading ? (
              <div className="mt-3 space-y-2">
                <Skeleton className="h-8 w-24" />
                <Skeleton className="h-3 w-32" />
              </div>
            ) : (
              <>
                <p className="mt-1.5 text-[30px] font-bold leading-none text-ink">
                  {formatDistanceShort(week?.distance_m ?? 0)}
                  <span className="ml-1 text-[14px] font-medium text-ink-muted">km</span>
                </p>
                <p className="mt-1.5 text-[13px] text-ink-muted">
                  {week?.activity_count ?? 0} activit
                  {(week?.activity_count ?? 0) === 1 ? 'y' : 'ies'} ·{' '}
                  {formatElevation(week?.elevation_gain_m)} climbed
                </p>
                <div className="mt-4 grid grid-cols-2 gap-4 border-t border-edge pt-4">
                  <StatTile
                    label="Avg pace"
                    value={formatPace(week?.avg_pace_s_per_km ?? null)}
                    unit="/km"
                    icon={<Timer size={12} strokeWidth={2} className="text-ink-faint" />}
                  />
                  <StatTile
                    label="Avg HR"
                    value={week?.avg_hr ? String(Math.round(week.avg_hr)) : '—'}
                    unit="bpm"
                    icon={<Flame size={12} strokeWidth={2} className="text-ink-faint" />}
                  />
                </div>
              </>
            )}
          </Card>

          <Card>
            <CardHeader title="Your sports" subtitle="All time" />
            {summaryQuery.isLoading ? (
              <div className="space-y-2 px-5 pb-5">
                {Array.from({ length: 3 }).map((_, index) => (
                  <Skeleton key={index} className="h-4 w-full" />
                ))}
              </div>
            ) : summary && summary.sport_breakdown.length > 0 ? (
              <ul className="px-5 pb-5">
                {summary.sport_breakdown.map((row) => (
                  <li
                    key={row.sport_type}
                    className="flex items-center justify-between gap-3 border-b border-edge py-2 last:border-b-0"
                  >
                    <span className="inline-flex min-w-0 items-center gap-2 text-[13px] font-medium text-ink">
                      <span
                        className="h-2 w-2 shrink-0 rounded-full"
                        style={{ background: SPORT_COLOR[row.sport_type] ?? ACCENT_DATA }}
                      />
                      <span className="truncate">{sportLabel(row.sport_type)}</span>
                    </span>
                    <span className="shrink-0 text-[13px] text-ink-muted">
                      <span className="font-semibold text-ink">
                        {formatDistanceShort(row.distance_m)}
                      </span>{' '}
                      km · {row.activity_count}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <EmptyState
                icon={<Route size={20} strokeWidth={1.8} />}
                title="Nothing yet"
                description="Sport totals appear after your first sync."
              />
            )}
          </Card>

          <Card className="px-5 py-5">
            <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
              Consistency
            </p>
            {summaryQuery.isLoading ? (
              <Skeleton className="mt-3 h-12 w-full" />
            ) : (
              <>
                <div className="mt-3 flex items-baseline gap-1.5">
                  <span className="text-[30px] font-bold leading-none text-ink">
                    {formatNumber(summary?.streaks.current_weeks ?? 0)}
                  </span>
                  <span className="text-[14px] font-medium text-ink-muted">
                    week{summary?.streaks.current_weeks === 1 ? '' : 's'} in a row
                  </span>
                </div>
                <p className="mt-2 flex items-center gap-1.5 text-[13px] text-ink-muted">
                  <TrendingUp size={14} strokeWidth={2} className="text-brand-500" />
                  Longest streak {formatNumber(summary?.streaks.longest_weeks ?? 0)} weeks
                </p>
                <p className="mt-1 flex items-center gap-1.5 text-[13px] text-ink-muted">
                  <Mountain size={14} strokeWidth={2} className="text-ink-faint" />
                  {formatNumber(summary?.totals.elevation_gain_m ?? 0)} m climbed all time
                </p>
              </>
            )}
          </Card>
        </aside>
      </div>
    </div>
  )
}
