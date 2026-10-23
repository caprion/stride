/**
 * Dashboard — the athlete's home surface.
 *
 * Composition:
 *   hero stat row (full width)
 *   ├─ main column : weekly volume · pace trend · recent activities
 *   └─ side column : personal records · monthly volume · HR zones · sports
 */

// Note: `npm run typecheck` failures outside this file belong to whoever owns
// the shared files — report them rather than editing shared modules.

import { useQuery } from '@tanstack/react-query'
import {
  Activity,
  CalendarDays,
  Flame,
  Mountain,
  Route,
  Target,
  Timer,
  TrendingUp,
} from 'lucide-react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import {
  formatDistance,
  formatDistanceShort,
  formatElevation,
  formatNumber,
  formatPace,
  formatPaceWithUnit,
  formatDate,
  sportLabel,
  SPORT_COLOR,
} from '../lib/format'
import { ActivityCard } from '../components/ActivityCard'
import { MonthlyBars, PaceTrendChart, VolumeChart, ZoneBarChart } from '../components/charts'
import {
  Button,
  Card,
  CardHeader,
  EmptyState,
  PageHeader,
  Skeleton,
  StatTile,
} from '../components/ui'

/** Weekly distance goal used for the progress bar. */
const WEEK_TARGET_M = 50_000

function greeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 18) return 'Good afternoon'
  return 'Good evening'
}

function HeroSkeleton() {
  return (
    <Card className="p-6">
      <div className="grid gap-x-6 gap-y-5 sm:grid-cols-2 lg:grid-cols-5">
        {Array.from({ length: 5 }).map((_, index) => (
          <div key={index} className="space-y-2">
            <Skeleton className="h-3 w-20" />
            <Skeleton className="h-7 w-24" />
            <Skeleton className="h-3 w-28" />
          </div>
        ))}
      </div>
    </Card>
  )
}

function HeroStat({
  label,
  value,
  unit,
  hint,
  icon,
  tone,
  children,
}: {
  label: string
  value: string
  unit?: string
  hint?: string
  icon: React.ReactNode
  tone?: 'default' | 'brand' | 'positive'
  children?: React.ReactNode
}) {
  return (
    <div className="flex flex-col">
      <StatTile label={label} value={value} unit={unit} hint={hint} icon={icon} tone={tone} />
      {children ? <div className="mt-3">{children}</div> : null}
    </div>
  )
}

function ProgressBar({ pct }: { pct: number }) {
  const clamped = Math.max(0, Math.min(100, pct))
  return (
    <div
      className="h-1.5 w-full overflow-hidden rounded-full bg-surface-sunken"
      role="progressbar"
      aria-valuenow={Math.round(clamped)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div className="h-full rounded-full bg-brand-500" style={{ width: `${clamped}%` }} />
    </div>
  )
}

function ChartSkeleton({ height = 240 }: { height?: number }) {
  return (
    <Card className="overflow-hidden">
      <div className="px-5 pt-4 pb-3">
        <Skeleton className="h-4 w-36" />
        <Skeleton className="mt-2 h-3 w-52" />
      </div>
      <div className="px-2 pb-3" style={{ height }}>
        <Skeleton className="h-full w-full" />
      </div>
    </Card>
  )
}

export function DashboardPage() {
  const summaryQuery = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const weeklyQuery = useQuery({
    queryKey: ['weekly', 16],
    queryFn: () => api.weekly(16),
  })

  const paceQuery = useQuery({
    queryKey: ['pace-trends', 12],
    queryFn: () => api.paceTrends(12),
  })

  const monthlyQuery = useQuery({
    queryKey: ['monthly', 12],
    queryFn: () => api.monthly(12),
  })

  const recordsQuery = useQuery({
    queryKey: ['personal-records'],
    queryFn: api.personalRecords,
  })

  const zonesQuery = useQuery({
    queryKey: ['hr-zones', 4],
    queryFn: () => api.hrZones(4),
  })

  const summary = summaryQuery.data
  const week = summary?.this_week
  const month = summary?.this_month
  const totals = summary?.totals
  const streaks = summary?.streaks

  const weeklyItems = weeklyQuery.data?.items ?? []
  const paceItems = paceQuery.data?.items ?? []
  const monthlyItems = monthlyQuery.data?.items ?? []
  const records = recordsQuery.data?.items ?? []
  const zones = zonesQuery.data?.zones ?? []
  const recent = summary?.recent ?? []

  const weekDistance = week?.distance_m ?? 0
  const weekPct = WEEK_TARGET_M > 0 ? (weekDistance / WEEK_TARGET_M) * 100 : 0

  return (
    <div>
      <PageHeader
        title={`${greeting()}, athlete`}
        description="Your training at a glance — volume, pace and consistency this week."
        actions={
          <>
            <Link to="/feed">
              <Button variant="secondary" size="md">
                Browse activities
              </Button>
            </Link>
            <Link to="/training">
              <Button size="md">
                <CalendarDays size={15} strokeWidth={2} />
                Training log
              </Button>
            </Link>
          </>
        }
      />

      {/* ------------------------------------------------------ hero stats */}
      {summaryQuery.isLoading ? (
        <HeroSkeleton />
      ) : summaryQuery.isError || !summary ? (
        <Card>
          <EmptyState
            icon={<Activity size={22} strokeWidth={1.8} />}
            title="Couldn't load your training summary"
            description="The stats service did not respond. Try again in a moment."
            action={
              <Button variant="secondary" onClick={() => summaryQuery.refetch()}>
                Retry
              </Button>
            }
          />
        </Card>
      ) : (
        <Card className="px-6 py-6">
          <div className="grid gap-x-6 gap-y-6 sm:grid-cols-2 lg:grid-cols-5">
            <HeroStat
              label="This week"
              value={formatDistanceShort(weekDistance)}
              unit="km"
              hint={`${week?.activity_count ?? 0} run${week?.activity_count === 1 ? '' : 's'} · goal ${formatDistanceShort(WEEK_TARGET_M)} km`}
              icon={<Route size={13} strokeWidth={2} className="text-brand-500" />}
              tone="brand"
            >
              <ProgressBar pct={weekPct} />
              <p className="mt-1.5 text-[11px] font-medium uppercase tracking-[0.07em] text-ink-faint">
                {Math.round(weekPct)}% of weekly goal
              </p>
            </HeroStat>

            <HeroStat
              label="This month"
              value={formatDistanceShort(month?.distance_m ?? 0)}
              unit="km"
              hint={month?.label ?? '—'}
              icon={<CalendarDays size={13} strokeWidth={2} className="text-ink-faint" />}
            />

            <HeroStat
              label="Total elevation"
              value={formatElevation(totals?.elevation_gain_m)}
              hint={`${formatDistanceShort(totals?.distance_m)} km all time`}
              icon={<Mountain size={13} strokeWidth={2} className="text-ink-faint" />}
            />

            <HeroStat
              label="Week streak"
              value={formatNumber(streaks?.current_weeks ?? 0)}
              unit="weeks"
              hint={
                streaks && streaks.current_weeks > 0
                  ? `Longest ${streaks.longest_weeks} weeks`
                  : 'Log a run to start one'
              }
              icon={<Flame size={13} strokeWidth={2} className="text-ink-faint" />}
              tone={streaks && streaks.current_weeks > 0 ? 'positive' : 'default'}
            />

            <HeroStat
              label="Average pace"
              value={formatPace(week?.avg_pace_s_per_km ?? null)}
              unit="/km"
              hint={`This week · ${formatPaceWithUnit(month?.avg_pace_s_per_km ?? null)} this month`}
              icon={<Timer size={13} strokeWidth={2} className="text-ink-faint" />}
            />
          </div>
        </Card>
      )}

      {/* --------------------------------------------------------- columns */}
      <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_320px] xl:grid-cols-[minmax(0,1fr)_352px]">
        {/* ------------------------------------------------- main column */}
        <div className="min-w-0 space-y-6">
          {weeklyQuery.isLoading ? (
            <ChartSkeleton />
          ) : weeklyItems.length === 0 ? (
            <Card>
              <EmptyState
                icon={<TrendingUp size={22} strokeWidth={1.8} />}
                title="No weekly volume yet"
                description="Once your first runs are synced, your weekly volume chart will appear here."
              />
            </Card>
          ) : (
            <VolumeChart
              data={weeklyItems}
              height={260}
              title="Weekly volume"
              subtitle="Last 16 weeks — distance per ISO week"
            />
          )}

          {paceQuery.isLoading ? (
            <ChartSkeleton />
          ) : paceItems.length === 0 ? (
            <Card>
              <EmptyState
                icon={<Timer size={22} strokeWidth={1.8} />}
                title="Not enough pace data"
                description="We need a couple of weeks of runs before we can chart your pace trend."
              />
            </Card>
          ) : (
            <PaceTrendChart data={paceItems} height={240} />
          )}

          <section>
            <CardHeader
              className="px-0 pt-0"
              title="Recent activities"
              subtitle={
                recent.length > 0 ? `Your last ${Math.min(recent.length, 8)} activities` : undefined
              }
              action={
                <Link
                  to="/feed"
                  className="text-[13px] font-semibold text-brand-600 hover:text-brand-700"
                >
                  View all
                </Link>
              }
            />
            {summaryQuery.isLoading ? (
              <div className="space-y-3">
                {Array.from({ length: 3 }).map((_, index) => (
                  <Skeleton key={index} className="h-[104px] w-full rounded-card" />
                ))}
              </div>
            ) : recent.length === 0 ? (
              <Card>
                <EmptyState
                  icon={<Activity size={22} strokeWidth={1.8} />}
                  title="No activities yet"
                  description="Sync your watch or import a FIT file to get started."
                />
              </Card>
            ) : (
              <div className="space-y-3">
                {recent.slice(0, 8).map((activity) => (
                  <ActivityCard key={activity.id} activity={activity} showRoute={false} />
                ))}
              </div>
            )}
          </section>
        </div>

        {/* ------------------------------------------------- side column */}
        <div className="min-w-0 space-y-6">
          {/* personal records */}
          <Card>
            <CardHeader
              title="Personal records"
              subtitle={records.length > 0 ? `${records.length} records` : undefined}
            />
            {recordsQuery.isLoading ? (
              <div className="grid grid-cols-2 gap-3 px-5 pb-5">
                {Array.from({ length: 6 }).map((_, index) => (
                  <Skeleton key={index} className="h-16 w-full rounded-md" />
                ))}
              </div>
            ) : records.length === 0 ? (
              <EmptyState
                icon={<Target size={22} strokeWidth={1.8} />}
                title="No records yet"
                description="Personal records appear once you have a few activities in the bank."
              />
            ) : (
              <div className="grid grid-cols-2 gap-px overflow-hidden rounded-b-card bg-edge">
                {records.map((record) => (
                  <Link
                    key={`${record.label}-${record.activity_id}`}
                    to={`/activities/${record.activity_id}`}
                    className="group bg-surface px-4 py-3 transition-colors hover:bg-surface-soft"
                  >
                    <p className="truncate text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                      {record.label}
                    </p>
                    <p className="mt-1 text-[19px] font-semibold leading-none text-ink group-hover:text-brand-600">
                      {record.display}
                    </p>
                    <p className="mt-1.5 truncate text-[12px] text-ink-muted">
                      {record.activity_name}
                    </p>
                    <p className="text-[11px] text-ink-faint">{formatDate(record.activity_date)}</p>
                  </Link>
                ))}
              </div>
            )}
          </Card>

          {/* monthly volume */}
          {monthlyQuery.isLoading ? (
            <ChartSkeleton height={200} />
          ) : monthlyItems.length === 0 ? (
            <Card>
              <EmptyState
                icon={<CalendarDays size={22} strokeWidth={1.8} />}
                title="No monthly volume"
                description="Monthly totals will show up as your history builds."
              />
            </Card>
          ) : (
            <MonthlyBars data={monthlyItems} height={200} />
          )}

          {/* hr zones */}
          {zonesQuery.isLoading ? (
            <ChartSkeleton height={220} />
          ) : zones.length === 0 ? (
            <Card>
              <EmptyState
                icon={<Flame size={22} strokeWidth={1.8} />}
                title="No heart-rate data"
                description="Wear a heart-rate monitor and we will chart your training zones."
              />
            </Card>
          ) : (
            <ZoneBarChart zones={zones} height={220} />
          )}

          {/* sport breakdown */}
          {summary && summary.sport_breakdown.length > 0 ? (
            <Card>
              <CardHeader title="By sport" subtitle="All time" />
              <div className="space-y-3 px-5 pb-5">
                {summary.sport_breakdown.map((row) => {
                  const total = summary.totals.distance_m || 1
                  const pct = (row.distance_m / total) * 100
                  return (
                    <div key={row.sport_type}>
                      <div className="flex items-baseline justify-between gap-3">
                        <span className="inline-flex items-center gap-1.5 text-[13px] font-medium text-ink">
                          <span
                            className="h-2 w-2 rounded-full"
                            style={{
                              background: SPORT_COLOR[row.sport_type] ?? '#fc5200',
                            }}
                          />
                          {sportLabel(row.sport_type)}
                        </span>
                        <span className="text-[13px] font-semibold text-ink">
                          {formatDistanceShort(row.distance_m)} km
                        </span>
                      </div>
                      <div className="mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-surface-sunken">
                        <div
                          className="h-full rounded-full"
                          style={{
                            width: `${Math.max(2, Math.min(100, pct))}%`,
                            background: SPORT_COLOR[row.sport_type] ?? '#fc5200',
                          }}
                        />
                      </div>
                      <p className="mt-1 text-[11px] text-ink-faint">
                        {row.activity_count} activities · {formatElevation(row.elevation_gain_m)} climbed
                      </p>
                    </div>
                  )
                })}
              </div>
            </Card>
          ) : null}

          {summary && summary.sport_breakdown.length === 0 ? (
            <Card>
              <EmptyState
                icon={<Activity size={22} strokeWidth={1.8} />}
                title="Nothing recorded yet"
                description="Your sport breakdown appears after the first synced activity."
              />
            </Card>
          ) : null}
        </div>
      </div>

      {/* ----------------------------------------------------- footer note */}
      {summary ? (
        <p className="mt-6 text-[12px] text-ink-faint">
          Totals cover {formatNumber(summary.totals.activity_count)} activities across{' '}
          {formatDistance(summary.totals.distance_m)} and{' '}
          {formatDurationLabel(summary.totals.moving_time_s)} of moving time.
        </p>
      ) : null}
    </div>
  )
}

/** Local helper: moving time in a compact "1,234h" style label. */
function formatDurationLabel(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const hours = Math.round(seconds / 3600)
  return `${formatNumber(hours)} h`
}
