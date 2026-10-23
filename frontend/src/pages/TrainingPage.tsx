/**
 * Training log — a real month calendar plus a selected-day detail panel.
 *
 * Composition:
 *   main column : month calendar · selected-day activities
 *   side column : month summary · weekly volume · streak readout
 */

import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Activity as ActivityIcon,
  CalendarDays,
  ChevronLeft,
  ChevronRight,
  Flame,
  Moon,
  Route as RouteIcon,
  Timer,
  TrendingUp,
} from 'lucide-react'
import { api } from '../lib/api'
import type { CalendarDay } from '../types'
import { formatDistance, formatDistanceShort, formatNumber } from '../lib/format'
import { ActivityCard } from '../components/ActivityCard'
import { VolumeChart } from '../components/charts'
import {
  Button,
  Card,
  CardHeader,
  EmptyState,
  PageHeader,
  Skeleton,
  StatTile,
  cn,
} from '../components/ui'

const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

/** Moving time as "9h 41m" — rounded to whole minutes so we never show "60m". */
function formatMovingTime(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return '—'
  const totalMinutes = Math.max(0, Math.round(seconds / 60))
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  if (hours && minutes) return `${hours}h ${minutes}m`
  if (hours) return `${hours}h`
  return `${minutes}m`
}

/** One rendered cell of the month grid. */
interface MonthCell {
  key: string
  date: string
  day: number
  inMonth: boolean
  distanceM: number
  activityCount: number
}

function toDateKey(year: number, month: number, day: number): string {
  const mm = String(month).padStart(2, '0')
  const dd = String(day).padStart(2, '0')
  return `${year}-${mm}-${dd}`
}

function daysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate()
}

/** Monday-first weekday index of the 1st of the month (0 = Monday). */
function leadingBlanks(year: number, month: number): number {
  const weekday = new Date(year, month - 1, 1).getDay() // 0 = Sunday
  return (weekday + 6) % 7
}

function monthLabel(year: number, month: number): string {
  return new Date(year, month - 1, 1).toLocaleDateString(undefined, {
    month: 'long',
    year: 'numeric',
  })
}

function buildCells(
  year: number,
  month: number,
  daysByDate: Map<string, CalendarDay>,
): MonthCell[] {
  const total = daysInMonth(year, month)
  const lead = leadingBlanks(year, month)
  const cells: MonthCell[] = []

  // Leading days of the previous month.
  const prevMonth = month === 1 ? 12 : month - 1
  const prevYear = month === 1 ? year - 1 : year
  const prevTotal = daysInMonth(prevYear, prevMonth)
  for (let i = lead - 1; i >= 0; i -= 1) {
    const day = prevTotal - i
    cells.push({
      key: `prev-${day}`,
      date: toDateKey(prevYear, prevMonth, day),
      day,
      inMonth: false,
      distanceM: 0,
      activityCount: 0,
    })
  }

  // The month itself.
  for (let day = 1; day <= total; day += 1) {
    const date = toDateKey(year, month, day)
    const entry = daysByDate.get(date)
    cells.push({
      key: date,
      date,
      day,
      inMonth: true,
      distanceM: entry?.distance_m ?? 0,
      activityCount: entry?.activities.length ?? 0,
    })
  }

  // Trailing days of the next month.
  const nextMonth = month === 12 ? 1 : month + 1
  const nextYear = month === 12 ? year + 1 : year
  const used = lead + total
  const trail = (7 - (used % 7)) % 7
  for (let day = 1; day <= trail; day += 1) {
    cells.push({
      key: `next-${day}`,
      date: toDateKey(nextYear, nextMonth, day),
      day,
      inMonth: false,
      distanceM: 0,
      activityCount: 0,
    })
  }

  return cells
}

/** Colour tier for a cell: none / small / medium / large. */
function volumeTier(distanceM: number, maxM: number): 'none' | 'small' | 'medium' | 'large' {
  if (!distanceM || distanceM <= 0) return 'none'
  if (maxM <= 0) return 'small'
  const ratio = distanceM / maxM
  if (ratio <= 0.25) return 'small'
  if (ratio <= 0.6) return 'medium'
  return 'large'
}

function DayCell({
  cell,
  maxM,
  selected,
  onSelect,
}: {
  cell: MonthCell
  maxM: number
  selected: boolean
  onSelect: (date: string) => void
}) {
  const tier = volumeTier(cell.distanceM, maxM)
  const ratio = maxM > 0 ? Math.min(1, cell.distanceM / maxM) : 0
  const clickable = cell.inMonth && cell.activityCount > 0

  const tierClass =
    tier === 'large'
      ? 'bg-brand-500 text-white'
      : tier === 'medium'
        ? 'bg-brand-100 text-brand-800'
        : tier === 'small'
          ? 'bg-brand-50 text-brand-700'
          : 'bg-surface text-ink-soft hover:bg-surface-soft'

  return (
    <button
      type="button"
      onClick={clickable ? () => onSelect(cell.date) : undefined}
      disabled={!clickable}
      aria-pressed={selected}
      aria-label={`${cell.date}${
        cell.activityCount
          ? `, ${formatDistance(cell.distanceM)}, ${cell.activityCount} activities`
          : ''
      }`}
      className={cn(
        'relative flex h-[86px] flex-col items-stretch rounded-md border p-1.5 text-left transition-colors',
        cell.inMonth ? 'border-edge' : 'border-transparent bg-surface-soft',
        !cell.inMonth && 'opacity-45',
        tierClass,
        clickable && 'hover:border-brand-400',
        selected && 'ring-2 ring-brand-600 ring-offset-1',
      )}
    >
      <span className="flex items-center justify-between gap-1">
        <span
          className={cn(
            'text-[12px] font-semibold leading-none',
            tier === 'large' ? 'text-white' : 'text-ink-soft',
          )}
        >
          {cell.day}
        </span>
        {cell.activityCount > 0 ? (
          <span className="flex items-center gap-0.5">
            {Array.from({ length: Math.min(3, cell.activityCount) }).map((_, index) => (
              <span
                key={index}
                className={cn(
                  'h-1 w-1 rounded-full',
                  tier === 'large' ? 'bg-white/90' : 'bg-brand-500',
                )}
              />
            ))}
          </span>
        ) : null}
      </span>

      {cell.activityCount > 0 ? (
        <>
          <span
            className={cn(
              'mt-1 text-[11px] font-semibold leading-none',
              tier === 'large' ? 'text-white' : 'text-ink',
            )}
          >
            {formatDistanceShort(cell.distanceM)}
            <span className="ml-0.5 text-[9px] font-medium opacity-70">km</span>
          </span>
          {/* Volume bar — a non-colour cue for how much was run. */}
          <span className="mt-auto block h-1 w-full overflow-hidden rounded-full bg-black/10">
            <span
              className={cn(
                'block h-full rounded-full',
                tier === 'large' ? 'bg-white' : 'bg-brand-500',
              )}
              style={{ width: `${Math.max(12, ratio * 100)}%` }}
            />
          </span>
        </>
      ) : (
        <span className="mt-auto block h-1 w-full" />
      )}
    </button>
  )
}

export function TrainingPage() {
  const today = useMemo(() => new Date(), [])
  const [year, setYear] = useState(today.getFullYear())
  const [month, setMonth] = useState(today.getMonth() + 1)

  const todayKey = toDateKey(today.getFullYear(), today.getMonth() + 1, today.getDate())
  const [selectedDate, setSelectedDate] = useState<string | null>(todayKey)

  const calendarQuery = useQuery({
    queryKey: ['calendar', year, month],
    queryFn: () => api.calendar(year, month),
  })

  const weeklyQuery = useQuery({
    queryKey: ['weekly', 12],
    queryFn: () => api.weekly(12),
  })

  const summaryQuery = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })

  const calendar = calendarQuery.data
  const daysByDate = useMemo(() => {
    const map = new Map<string, CalendarDay>()
    for (const day of calendar?.days ?? []) map.set(day.date, day)
    return map
  }, [calendar])

  const cells = useMemo(() => buildCells(year, month, daysByDate), [year, month, daysByDate])

  const maxDistance = useMemo(
    () => (calendar?.days ?? []).reduce((max, day) => Math.max(max, day.distance_m), 0),
    [calendar],
  )

  const monthMovingTime = useMemo(
    () => (calendar?.days ?? []).reduce((sum, day) => sum + (day.moving_time_s ?? 0), 0),
    [calendar],
  )

  // The selected day may live in the month we are not looking at any more.
  const selectedDay = selectedDate ? (daysByDate.get(selectedDate) ?? null) : null

  const goToMonth = (delta: number) => {
    const next = new Date(year, month - 1 + delta, 1)
    setYear(next.getFullYear())
    setMonth(next.getMonth() + 1)
    setSelectedDate(null)
  }

  const goToToday = () => {
    setYear(today.getFullYear())
    setMonth(today.getMonth() + 1)
    setSelectedDate(todayKey)
  }

  const weeklyItems = weeklyQuery.data?.items ?? []
  const streaks = summaryQuery.data?.streaks

  return (
    <div>
      <PageHeader
        title="Training log"
        description="Plan the month, review each session and keep the streak alive."
      />

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_330px] xl:grid-cols-[minmax(0,1fr)_360px]">
        {/* -------------------------------------------------- main column */}
        <div className="min-w-0 space-y-6">
          {/* ------------------------------------------ month summary */}
          <Card className="px-5 py-5">
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                {monthLabel(year, month)} summary
              </p>
              <p className="text-[12px] text-ink-muted">
                {calendarQuery.isLoading
                  ? 'Loading…'
                  : `${formatNumber(calendar?.activity_count ?? 0)} activit${
                      (calendar?.activity_count ?? 0) === 1 ? 'y' : 'ies'
                    } logged`}
              </p>
            </div>
            <div className="mt-3 grid grid-cols-3 gap-4 sm:gap-8">
              <StatTile
                label="Distance"
                value={formatDistanceShort(calendar?.total_distance_m ?? 0)}
                unit="km"
                icon={<RouteIcon size={13} strokeWidth={2} className="text-brand-500" />}
                tone="brand"
              />
              <StatTile
                label="Activities"
                value={formatNumber(calendar?.activity_count ?? 0)}
                hint="Sessions logged"
                icon={<ActivityIcon size={13} strokeWidth={2} className="text-ink-faint" />}
              />
              <StatTile
                label="Moving time"
                value={formatMovingTime(monthMovingTime)}
                hint="Time on the move"
                icon={<Timer size={13} strokeWidth={2} className="text-ink-faint" />}
              />
            </div>
          </Card>

          {/* ----------------------------------------------- calendar */}
          <Card className="overflow-hidden">
            <div className="flex items-center justify-between gap-3 px-5 pt-4 pb-3">
              <div>
                <h2 className="text-[17px] font-semibold leading-tight text-ink">
                  {monthLabel(year, month)}
                </h2>
                <p className="mt-0.5 text-[12px] text-ink-muted">
                  {calendar
                    ? `${formatNumber(calendar.activity_count)} activit${
                        calendar.activity_count === 1 ? 'y' : 'ies'
                      } · ${formatDistance(calendar.total_distance_m)}`
                    : ' '}
                </p>
              </div>
              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => goToMonth(-1)}
                  aria-label="Previous month"
                >
                  <ChevronLeft size={16} strokeWidth={2} />
                </Button>
                <Button variant="secondary" size="sm" onClick={goToToday}>
                  Today
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => goToMonth(1)}
                  aria-label="Next month"
                >
                  <ChevronRight size={16} strokeWidth={2} />
                </Button>
              </div>
            </div>

            {calendarQuery.isLoading ? (
              <div className="px-5 pb-5">
                <div className="mb-1.5 grid grid-cols-7 gap-1.5">
                  {WEEKDAYS.map((label) => (
                    <div
                      key={label}
                      className="pb-1 text-center text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint"
                    >
                      {label}
                    </div>
                  ))}
                </div>
                <div className="grid grid-cols-7 gap-1.5">
                  {Array.from({ length: 35 }).map((_, index) => (
                    <Skeleton key={index} className="h-[86px] w-full rounded-md" />
                  ))}
                </div>
              </div>
            ) : (
              <div className="px-3 pb-4 sm:px-5">
                <div className="mb-1.5 grid grid-cols-7 gap-1.5">
                  {WEEKDAYS.map((label) => (
                    <div
                      key={label}
                      className="pb-1 text-center text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint"
                    >
                      {label}
                    </div>
                  ))}
                </div>
                <div className="grid grid-cols-7 gap-1.5">
                  {cells.map((cell) => (
                    <DayCell
                      key={cell.key}
                      cell={cell}
                      maxM={maxDistance}
                      selected={cell.date === selectedDate}
                      onSelect={setSelectedDate}
                    />
                  ))}
                </div>

                {/* legend */}
                <div className="mt-3 flex flex-wrap items-center gap-3 text-[11px] text-ink-muted">
                  <span className="font-semibold uppercase tracking-[0.07em] text-ink-faint">
                    Volume
                  </span>
                  <span className="inline-flex items-center gap-1.5">
                    <span className="h-2.5 w-2.5 rounded-[3px] border border-edge bg-surface" />
                    None
                  </span>
                  <span className="inline-flex items-center gap-1.5">
                    <span className="h-2.5 w-2.5 rounded-[3px] bg-brand-50" />
                    Light
                  </span>
                  <span className="inline-flex items-center gap-1.5">
                    <span className="h-2.5 w-2.5 rounded-[3px] bg-brand-100" />
                    Medium
                  </span>
                  <span className="inline-flex items-center gap-1.5">
                    <span className="h-2.5 w-2.5 rounded-[3px] bg-brand-500" />
                    Large
                  </span>
                  <span className="inline-flex items-center gap-1.5">
                    <span className="h-1.5 w-4 rounded-full bg-brand-500" />
                    Bar length = distance
                  </span>
                </div>
              </div>
            )}
          </Card>

          {/* ------------------------------------------ selected day */}
          <section>
            <CardHeader
              className="px-0 pt-0"
              title={
                selectedDay
                  ? new Date(`${selectedDay.date}T00:00:00`).toLocaleDateString(undefined, {
                      weekday: 'long',
                      day: 'numeric',
                      month: 'long',
                    })
                  : selectedDate
                    ? new Date(`${selectedDate}T00:00:00`).toLocaleDateString(undefined, {
                        weekday: 'long',
                        day: 'numeric',
                        month: 'long',
                      })
                    : 'Day detail'
              }
              subtitle={
                selectedDay
                  ? `${formatDistance(selectedDay.distance_m)} · ${formatMovingTime(
                      selectedDay.moving_time_s,
                    )} moving time`
                  : selectedDate
                    ? 'Outside the current month'
                    : 'Pick a day in the calendar to see the session.'
              }
            />

            {calendarQuery.isLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-[104px] w-full rounded-card" />
                <Skeleton className="h-[104px] w-full rounded-card" />
              </div>
            ) : selectedDay && selectedDay.activities.length > 0 ? (
              <div className="space-y-3">
                {selectedDay.activities.map((activity) => (
                  <ActivityCard key={activity.id} activity={activity} showRoute={false} />
                ))}
              </div>
            ) : (
              <Card>
                <EmptyState
                  icon={
                    selectedDay ? (
                      <Moon size={22} strokeWidth={1.8} />
                    ) : (
                      <CalendarDays size={22} strokeWidth={1.8} />
                    )
                  }
                  title={selectedDay ? 'Rest day' : 'No day selected'}
                  description={
                    selectedDay
                      ? 'No sessions were logged on this date. A good day to recover.'
                      : 'Choose a highlighted day in the calendar to see the activities you logged.'
                  }
                />
              </Card>
            )}
          </section>
        </div>

        {/* ------------------------------------------------- side column */}
        <div className="min-w-0 space-y-6">
          {/* ---------------------------------------------- weekly volume */}
          {weeklyQuery.isLoading ? (
            <Card className="overflow-hidden">
              <div className="px-5 pt-4 pb-3">
                <Skeleton className="h-4 w-32" />
              </div>
              <div className="px-2 pb-3" style={{ height: 220 }}>
                <Skeleton className="h-full w-full" />
              </div>
            </Card>
          ) : weeklyItems.length === 0 ? (
            <Card>
              <CardHeader title="Weekly volume" />
              <EmptyState
                icon={<TrendingUp size={22} strokeWidth={1.8} />}
                title="No weekly volume yet"
                description="Charted as soon as your first runs are synced."
              />
            </Card>
          ) : (
            <VolumeChart
              data={weeklyItems}
              height={220}
              title="Weekly volume"
              subtitle="Last 12 weeks"
            />
          )}

          {/* ---------------------------------------------------- streak */}
          <Card className="px-5 py-5">
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
                  Streak
                </p>
                <p className="mt-1 text-[26px] font-semibold leading-none text-ink">
                  {formatNumber(streaks?.current_weeks ?? 0)}
                  <span className="ml-1 text-[14px] font-medium text-ink-muted">
                    {streaks?.current_weeks === 1 ? 'week' : 'weeks'}
                  </span>
                </p>
                <p className="mt-1.5 text-[12px] text-ink-muted">
                  Longest {formatNumber(streaks?.longest_weeks ?? 0)} weeks ·{' '}
                  {formatNumber(streaks?.total_active_weeks ?? 0)} active weeks
                </p>
              </div>
              <span
                className={cn(
                  'flex h-10 w-10 items-center justify-center rounded-full',
                  (streaks?.current_weeks ?? 0) > 0
                    ? 'bg-brand-50 text-brand-600'
                    : 'bg-surface-sunken text-ink-faint',
                )}
              >
                <Flame size={20} strokeWidth={2} />
              </span>
            </div>
            <div className="mt-4">
              <div className="h-1.5 w-full overflow-hidden rounded-full bg-surface-sunken">
                <div
                  className="h-full rounded-full bg-brand-500"
                  style={{
                    width: `${Math.min(
                      100,
                      ((streaks?.current_weeks ?? 0) /
                        Math.max(1, streaks?.longest_weeks ?? 1)) *
                        100,
                    )}%`,
                  }}
                />
              </div>
              <p className="mt-1.5 text-[11px] text-ink-faint">
                {streaks?.current_week_key
                  ? `Current week ${streaks.current_week_key.replace(/^(\d{4})-W(\d+)$/, 'W$2')}`
                  : 'No active week yet'}
              </p>
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}
