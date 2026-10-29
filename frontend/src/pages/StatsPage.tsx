/**
 * Statistics — "Totals & gains" plus trend surfaces.
 *
 * Two deliberate sections, switched by a segmented control:
 *   Totals : all-time gains · personal records · sport split · HR zones
 *   Trends : weekly/monthly volume · pace trend · yearly heatmap
 */

import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Activity as ActivityIcon,
  CalendarDays,
  Flame,
  Heart,
  Mountain,
  Route as RouteIcon,
  Timer,
  TrendingUp,
} from 'lucide-react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { ACCENT_DATA, ACCENT_DEEP, SURFACE_SUNKEN } from '../lib/palette'
import type { Bucket, HeatmapDay } from '../types'
import {
  formatDate,
  formatDistance,
  formatDistanceShort,
  formatElevation,
  formatHr,
  formatNumber,
  formatPaceWithUnit,
  sportLabel,
  SPORT_COLOR,
} from '../lib/format'
import { MonthlyBars, PaceTrendChart, VolumeChart, ZoneBarChart } from '../components/charts'
import { ConnectPrompt } from '../components/ConnectPrompt'
import {
  Card,
  CardHeader,
  Divider,
  EmptyState,
  PageHeader,
  SegmentedControl,
  Skeleton,
  StatTile,
  cn,
} from '../components/ui'

/* ------------------------------------------------------------------ heatmap */

/** Ordered shades, lightest to darkest. */
const HEAT_STEPS = [ACCENT_DATA, ACCENT_DATA, ACCENT_DEEP, ACCENT_DATA, ACCENT_DATA]

const WEEKDAY_LABELS = ['Mon', '', 'Wed', '', 'Fri', '', 'Sun']

/**
 * Moving time in "203h 12m" form. Rounds to whole minutes first so we never
 * render a nonsensical "202h 60m".
 */
function formatMovingTime(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const totalMinutes = Math.max(0, Math.round(seconds / 60))
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  if (hours && minutes) return `${hours}h ${minutes}m`
  if (hours) return `${hours}h`
  return `${minutes}m`
}

interface HeatCell {
  date: string
  inYear: boolean
  distanceM: number
  count: number
}

function buildHeatCells(year: number, byDate: Map<string, HeatmapDay>): HeatCell[] {
  const first = new Date(year, 0, 1)
  const offset = (first.getDay() + 6) % 7 // Monday-first
  const start = new Date(year, 0, 1 - offset)
  const cells: HeatCell[] = []
  for (let i = 0; i < 53 * 7; i += 1) {
    const date = new Date(start.getFullYear(), start.getMonth(), start.getDate() + i)
    const key = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
      date.getDate(),
    ).padStart(2, '0')}`
    const entry = byDate.get(key)
    cells.push({
      date: key,
      inYear: date.getFullYear() === year,
      distanceM: entry?.distance_m ?? 0,
      count: entry?.activity_count ?? 0,
    })
  }
  return cells
}

function heatColor(distanceM: number, maxM: number, inYear: boolean): string {
  if (!inYear) return 'transparent'
  if (!distanceM) return SURFACE_SUNKEN
  if (maxM <= 0) return HEAT_STEPS[0]
  const ratio = distanceM / maxM
  if (ratio <= 0.25) return HEAT_STEPS[0]
  if (ratio <= 0.5) return HEAT_STEPS[1]
  if (ratio <= 0.75) return HEAT_STEPS[2]
  if (ratio < 1) return HEAT_STEPS[3]
  return HEAT_STEPS[4]
}

function Heatmap({
  year,
  onYearChange,
}: {
  year: number
  onYearChange: (year: number) => void
}) {
  const heatmapQuery = useQuery({
    queryKey: ['heatmap', year],
    queryFn: () => api.heatmap(year),
  })

  const byDate = useMemo(() => {
    const map = new Map<string, HeatmapDay>()
    for (const day of heatmapQuery.data?.days ?? []) map.set(day.date, day)
    return map
  }, [heatmapQuery.data])

  const cells = useMemo(() => buildHeatCells(year, byDate), [year, byDate])

  const maxDistance = useMemo(
    () =>
      (heatmapQuery.data?.days ?? []).reduce((max, day) => Math.max(max, day.distance_m), 0),
    [heatmapQuery.data],
  )

  const activeDays = (heatmapQuery.data?.days ?? []).filter((day) => day.distance_m > 0).length
  const totalDistance = (heatmapQuery.data?.days ?? []).reduce(
    (sum, day) => sum + day.distance_m,
    0,
  )

  const currentYear = new Date().getFullYear()
  const yearOptions = [currentYear, currentYear - 1, currentYear - 2].map((value) => ({
    label: String(value),
    value,
  }))

  // Month labels sit above the first week column that contains the month.
  const monthLabels = useMemo(() => {
    const labels: { index: number; label: string }[] = []
    const seen = new Set<number>()
    for (let week = 0; week < 53; week += 1) {
      const cell = cells[week * 7]
      if (!cell) break
      const date = new Date(`${cell.date}T00:00:00`)
      if (date.getFullYear() !== year) continue
      const month = date.getMonth()
      if (seen.has(month)) continue
      seen.add(month)
      labels.push({
        index: week,
        label: date.toLocaleDateString(undefined, { month: 'short' }),
      })
    }
    return labels
  }, [cells, year])

  return (
    <Card className="overflow-hidden">
      <div className="flex flex-wrap items-start justify-between gap-3 px-5 pt-4 pb-3">
        <div>
          <h3 className="text-[15px] font-semibold leading-tight text-ink">Year in runs</h3>
          <p className="mt-0.5 text-[13px] text-ink-muted">
            {heatmapQuery.isLoading
              ? 'Loading…'
              : `${formatNumber(activeDays)} active days · ${formatDistance(totalDistance)}`}
          </p>
        </div>
        <SegmentedControl
          options={yearOptions}
          value={year}
          onChange={(value) => onYearChange(value)}
        />
      </div>

      {heatmapQuery.isLoading ? (
        <div className="px-5 pb-5">
          <Skeleton className="h-[120px] w-full" />
        </div>
      ) : activeDays === 0 ? (
        <ConnectPrompt
          what="your stats"
          emptyTitle={`No runs logged in ${year}`}
          emptyDescription="Once activities are synced for this year, the grid fills in."
        />
      ) : (
        <div className="px-5 pb-5">
          <div className="overflow-x-auto pb-2">
            <div className="min-w-[640px]">
              {/* month labels */}
              <div className="relative mb-1 h-4">
                {monthLabels.map((label) => (
                  <span
                    key={`${label.index}-${label.label}`}
                    className="absolute text-[10px] font-medium text-ink-faint"
                    style={{ left: `${(label.index / 53) * 100}%` }}
                  >
                    {label.label}
                  </span>
                ))}
              </div>

              <div className="flex gap-2">
                {/* weekday axis */}
                <div className="flex w-7 shrink-0 flex-col gap-[3px]">
                  {WEEKDAY_LABELS.map((label, index) => (
                    <span
                      key={index}
                      className="flex h-[10px] items-center text-[9px] font-medium leading-none text-ink-faint"
                    >
                      {label}
                    </span>
                  ))}
                </div>

                {/* the grid: 7 weekday rows × 53 week columns */}
                <div
                  className="grid grid-flow-col grid-rows-7 gap-[3px]"
                  style={{ gridTemplateColumns: 'repeat(53, minmax(0, 1fr))' }}
                >
                  {cells.map((cell) => (
                    <div
                      key={cell.date}
                      title={
                        cell.inYear
                          ? `${cell.date} — ${formatDistance(cell.distanceM)}, ${
                              cell.count
                            } activit${cell.count === 1 ? 'y' : 'ies'}`
                          : undefined
                      }
                      className="h-[10px] w-full rounded-[2px]"
                      style={{
                        background: heatColor(cell.distanceM, maxDistance, cell.inYear),
                        opacity: cell.inYear ? 1 : 0,
                      }}
                    />
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* legend */}
          <div className="mt-3 flex items-center justify-end gap-2 text-[11px] text-ink-muted">
            <span>Less</span>
            {[SURFACE_SUNKEN, ...HEAT_STEPS].map((color) => (
              <span
                key={color}
                className="h-[10px] w-[10px] rounded-[2px]"
                style={{ background: color }}
              />
            ))}
            <span>More</span>
          </div>
        </div>
      )}
    </Card>
  )
}

/* --------------------------------------------------------------- fragments */

function PeriodRow({
  label,
  bucket,
  emphasis,
}: {
  label: string
  bucket: Bucket | undefined
  emphasis?: boolean
}) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-6 gap-y-1 py-2.5">
      <span
        className={cn(
          'w-24 shrink-0 text-[11px] font-semibold uppercase tracking-[0.07em]',
          emphasis ? 'text-brand-600' : 'text-ink-muted',
        )}
      >
        {label}
      </span>
      <span className="text-[17px] font-semibold leading-none text-ink">
        {formatDistanceShort(bucket?.distance_m ?? 0)}
        <span className="ml-1 text-[12px] font-medium text-ink-muted">km</span>
      </span>
      <span className="text-[13px] text-ink-soft">
        {formatNumber(bucket?.activity_count ?? 0)} activit
        {(bucket?.activity_count ?? 0) === 1 ? 'y' : 'ies'}
      </span>
      <span className="text-[13px] text-ink-soft">
        {formatElevation(bucket?.elevation_gain_m ?? 0)} gain
      </span>
      <span className="text-[13px] text-ink-soft">
        {formatMovingTime(bucket?.moving_time_s ?? 0)} moving
      </span>
      <span className="text-[13px] text-ink-soft">
        {formatPaceWithUnit(bucket?.avg_pace_s_per_km ?? null)} pace
      </span>
      <span className="text-[13px] text-ink-soft">
        {formatHr(bucket?.avg_hr ?? null)} bpm avg HR
      </span>
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

/* -------------------------------------------------------------------- page */

export function StatsPage() {
  const [section, setSection] = useState<'totals' | 'trends'>('totals')
  const [heatmapYear, setHeatmapYear] = useState(() => new Date().getFullYear())

  const summaryQuery = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const recordsQuery = useQuery({
    queryKey: ['personal-records'],
    queryFn: api.personalRecords,
  })

  const weeklyQuery = useQuery({
    queryKey: ['weekly', 26],
    queryFn: () => api.weekly(26),
  })

  const monthlyQuery = useQuery({
    queryKey: ['monthly', 12],
    queryFn: () => api.monthly(12),
  })

  const paceQuery = useQuery({
    queryKey: ['pace-trends', 26],
    queryFn: () => api.paceTrends(26),
  })

  const zonesQuery = useQuery({
    queryKey: ['hr-zones', 12],
    queryFn: () => api.hrZones(12),
  })

  const summary = summaryQuery.data
  const totals = summary?.totals
  const records = recordsQuery.data?.items ?? []
  const weeklyItems = weeklyQuery.data?.items ?? []
  const monthlyItems = monthlyQuery.data?.items ?? []
  const paceItems = paceQuery.data?.items ?? []
  const zones = zonesQuery.data?.zones ?? []
  const sports = summary?.sport_breakdown ?? []

  return (
    <div>
      <PageHeader
        title="Statistics"
        description="All-time gains, personal records and the trends behind them."
        actions={
          <SegmentedControl
            options={[
              { label: 'Totals', value: 'totals' },
              { label: 'Trends', value: 'trends' },
            ]}
            value={section}
            onChange={(value) => setSection(value)}
          />
        }
      />

      {section === 'totals' ? (
        <div className="space-y-6">
          {/* ------------------------------------------ totals & gains */}
          <Card className="px-6 py-6">
            <div className="flex items-baseline justify-between gap-3">
              <h2 className="text-[15px] font-semibold text-ink">Totals &amp; gains</h2>
              <span className="text-[12px] text-ink-muted">{totals?.label ?? 'All time'}</span>
            </div>

            {summaryQuery.isLoading ? (
              <div className="mt-5 grid gap-x-6 gap-y-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
                {Array.from({ length: 6 }).map((_, index) => (
                  <div key={index} className="space-y-2">
                    <Skeleton className="h-3 w-20" />
                    <Skeleton className="h-7 w-24" />
                    <Skeleton className="h-3 w-28" />
                  </div>
                ))}
              </div>
            ) : summaryQuery.isError || !totals ? (
              <EmptyState
                icon={<ActivityIcon size={22} strokeWidth={1.8} />}
                title="Couldn't load your statistics"
                description="The stats service did not respond. Try again in a moment."
              />
            ) : (
              <div className="mt-5 grid gap-x-6 gap-y-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
                <StatTile
                  label="Total distance"
                  value={formatDistanceShort(totals.distance_m)}
                  unit="km"
                  hint={`${formatDistanceShort(summary?.this_year.distance_m ?? 0)} km in ${
                    summary?.this_year.label ?? 'this year'
                  }`}
                  icon={<RouteIcon size={13} strokeWidth={2} className="text-brand-500" />}
                  tone="brand"
                />
                <StatTile
                  label="Elevation gain"
                  value={formatElevation(totals.elevation_gain_m)}
                  hint={`${formatNumber(Math.round(totals.elevation_gain_m))} m climbed`}
                  icon={<Mountain size={13} strokeWidth={2} className="text-ink-faint" />}
                />
                <StatTile
                  label="Moving time"
                  value={formatMovingTime(totals.moving_time_s)}
                  hint={`${formatNumber(Math.round(totals.moving_time_s / 3600))} hours on the move`}
                  icon={<Timer size={13} strokeWidth={2} className="text-ink-faint" />}
                />
                <StatTile
                  label="Activities"
                  value={formatNumber(totals.activity_count)}
                  hint="Logged sessions"
                  icon={<ActivityIcon size={13} strokeWidth={2} className="text-ink-faint" />}
                />
                <StatTile
                  label="Average pace"
                  value={formatPaceWithUnit(totals.avg_pace_s_per_km)}
                  hint="Across every activity"
                  icon={<TrendingUp size={13} strokeWidth={2} className="text-ink-faint" />}
                />
                <StatTile
                  label="Average HR"
                  value={formatHr(totals.avg_hr)}
                  unit="bpm"
                  hint="Average heart rate"
                  icon={<Heart size={13} strokeWidth={2} className="text-ink-faint" />}
                />
              </div>
            )}
          </Card>

          {/* ------------------------------------------------- periods */}
          {summary ? (
            <Card className="px-6 py-5">
              <h2 className="text-[15px] font-semibold text-ink">By period</h2>
              <div className="mt-2 divide-y divide-edge">
                <PeriodRow label="This year" bucket={summary.this_year} emphasis />
                <PeriodRow label="This month" bucket={summary.this_month} />
                <PeriodRow label="This week" bucket={summary.this_week} />
              </div>
            </Card>
          ) : null}

          {/* ------------------------------------------- personal records */}
          <Card className="overflow-hidden">
            <CardHeader
              title="Personal records"
              subtitle={records.length > 0 ? `${records.length} records` : undefined}
            />
            {recordsQuery.isLoading ? (
              <div className="grid grid-cols-2 gap-3 px-5 pb-5 sm:grid-cols-3">
                {Array.from({ length: 6 }).map((_, index) => (
                  <Skeleton key={index} className="h-[104px] w-full rounded-md" />
                ))}
              </div>
            ) : records.length === 0 ? (
              <ConnectPrompt
                what="your stats"
                emptyTitle="No records yet"
                emptyDescription="Personal records appear once you have a few activities in the bank."
              />
            ) : (
              <div className="grid grid-cols-2 gap-px bg-edge sm:grid-cols-3">
                {records.map((record) => (
                  <Link
                    key={`${record.label}-${record.activity_id}`}
                    to={`/activities/${record.activity_id}`}
                    className="group bg-surface px-4 py-3.5 transition-colors hover:bg-surface-soft"
                  >
                    <p className="truncate text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                      {record.label}
                    </p>
                    <p className="mt-1 text-[21px] font-semibold leading-none text-ink group-hover:text-brand-600">
                      {record.display}
                    </p>
                    <p className="mt-2 truncate text-[12px] font-medium text-ink-soft">
                      {record.activity_name}
                    </p>
                    <p className="text-[11px] text-ink-faint">
                      {formatDate(record.activity_date)}
                    </p>
                  </Link>
                ))}
              </div>
            )}
          </Card>

          {/* ---------------------------------------------- sport split */}
          <Card className="overflow-hidden">
            <CardHeader title="By sport" subtitle="All time" />
            {summaryQuery.isLoading ? (
              <div className="space-y-2 px-5 pb-5">
                {Array.from({ length: 3 }).map((_, index) => (
                  <Skeleton key={index} className="h-9 w-full" />
                ))}
              </div>
            ) : sports.length === 0 ? (
              <EmptyState
                icon={<ActivityIcon size={22} strokeWidth={1.8} />}
                title="Nothing recorded yet"
                description="Your sport breakdown appears after the first synced activity."
              />
            ) : (
              /* The table has five columns. On a phone it scrolls sideways
                 rather than pushing the page off the edge. */
              <div className="overflow-x-auto">
                <table className="w-full min-w-[520px] text-left">
                  <thead>
                    <tr className="border-b border-edge text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                      <th className="px-5 py-2 font-semibold">Sport</th>
                      <th className="px-3 py-2 text-right font-semibold">Activities</th>
                      <th className="px-3 py-2 text-right font-semibold">Distance</th>
                      <th className="px-3 py-2 text-right font-semibold">Elevation</th>
                      <th className="px-5 py-2 text-right font-semibold">Moving time</th>
                    </tr>
                  </thead>
                  <tbody>
                    {sports.map((row) => (
                      <tr key={row.sport_type} className="border-b border-edge last:border-0">
                        <td className="px-5 py-2.5">
                          <span className="inline-flex items-center gap-2 text-[13px] font-medium text-ink">
                            <span
                              className="h-2.5 w-2.5 rounded-full"
                              style={{ background: SPORT_COLOR[row.sport_type] ?? ACCENT_DATA }}
                            />
                            {sportLabel(row.sport_type)}
                          </span>
                        </td>
                        <td className="px-3 py-2.5 text-right text-[13px] text-ink-soft">
                          {formatNumber(row.activity_count)}
                        </td>
                        <td className="px-3 py-2.5 text-right text-[13px] font-semibold text-ink">
                          {formatDistance(row.distance_m)}
                        </td>
                        <td className="px-3 py-2.5 text-right text-[13px] text-ink-soft">
                          {formatElevation(row.elevation_gain_m)}
                        </td>
                        <td className="px-5 py-2.5 text-right text-[13px] text-ink-soft">
                          {formatMovingTime(row.moving_time_s)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </Card>

          {/* ------------------------------------------------- hr zones */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_300px]">
            {zonesQuery.isLoading ? (
              <ChartSkeleton height={230} />
            ) : zones.length === 0 ? (
              <Card>
                <EmptyState
                  icon={<Flame size={22} strokeWidth={1.8} />}
                  title="No heart-rate data"
                  description="Wear a heart-rate monitor and we will chart your training zones."
                />
              </Card>
            ) : (
              <ZoneBarChart zones={zones} height={230} />
            )}

            <Card className="px-5 py-5">
              <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                Heart rate
              </p>
              <div className="mt-3 space-y-4">
                <div>
                  <p className="text-[12px] text-ink-muted">Peak heart rate</p>
                  <p className="text-[24px] font-semibold leading-none text-ink">
                    {formatHr(zonesQuery.data?.peak_hr ?? null)}
                    <span className="ml-1 text-[13px] font-medium text-ink-muted">bpm</span>
                  </p>
                </div>
                <Divider />
                <div>
                  <p className="text-[12px] text-ink-muted">Time in zones</p>
                  <p className="text-[24px] font-semibold leading-none text-ink">
                    {formatMovingTime(zonesQuery.data?.total_time_s ?? 0)}
                  </p>
                  <p className="mt-1 text-[12px] text-ink-muted">
                    Last {formatNumber(zonesQuery.data?.weeks ?? 12)} weeks
                  </p>
                </div>
              </div>
            </Card>
          </div>
        </div>
      ) : (
        <div className="space-y-6">
          {/* -------------------------------------------- weekly volume */}
          {weeklyQuery.isLoading ? (
            <ChartSkeleton height={250} />
          ) : weeklyItems.length === 0 ? (
            <Card>
              <EmptyState
                icon={<TrendingUp size={22} strokeWidth={1.8} />}
                title="No weekly volume yet"
                description="Charted as soon as your first runs are synced."
              />
            </Card>
          ) : (
            <VolumeChart
              data={weeklyItems}
              height={250}
              title="Weekly volume"
              subtitle="Last 26 weeks — distance per ISO week"
            />
          )}

          {/* ------------------------------------------- monthly volume */}
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

          {/* --------------------------------------------- pace trend */}
          {paceQuery.isLoading ? (
            <ChartSkeleton height={240} />
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

          {/* ------------------------------------------------- heatmap */}
          <Heatmap year={heatmapYear} onYearChange={setHeatmapYear} />
        </div>
      )}
    </div>
  )
}
