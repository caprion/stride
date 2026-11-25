/**
 * The wall: the desktop's Progress.
 *
 * **The desktop is a workspace.** Every panel the router answers is on
 * one screen, in the mock's order and in the mock's card shape: a name,
 * the answer, and the way into the panel that answers it. The phone keeps
 * the index — six questions, one at a time — and this is the other
 * arrangement of the same screen.
 *
 * The rules the cards follow:
 *
 *   - **No new numbers.** Every value on the wall is an answer the
 *     router already gave. A panel with no value says so, in the
 *     answer's own words, and shows no number at all.
 *   - **Every card opens a panel.** The "open →" in the card's head is
 *     the doorway the mock's drawer is: it opens the panel that answers
 *     the card, beside it.
 *   - **The card is the mock's card**: a name in the corner, the answer
 *     under it, and the window it covers named with it.
 *
 * The answers come from the same hooks the panels read — `useProgress` for
 * the ones that follow the range, `useWall` for the wall's own — so a
 * number on the wall and the same number on its panel are one number.
 */

import { Link } from 'react-router-dom'
import { Fragment, type ReactNode } from 'react'
import { endOfMonth, format, isValid, parseISO } from 'date-fns'
import { t } from '../i18n'
import {
  ABSENT,
  formatCount,
  formatDateMedium,
  formatDateShort,
  renderedText,
} from '../lib/format'
import { runsPath, shiftDate } from '../lib/history'
import { axisTicks } from '../lib/chart'
import type {
  RollingInsight,
  StreakInsight,
  TodayInsight,
  YearDay,
} from '../lib/types'
import {
  AdherencePanel,
  ChallengePanel,
  ConsistencyPanel,
  LoadPanel,
  VolumePanel,
  hour,
} from './progressPanels'
import { LadderCard } from './ladder'
import { LedgerCard } from './ledger'
import {
  BestsPicture,
  HabitPicture,
  SchedulePicture,
  ShapeGauges,
  WeekPicture,
  daysOf,
  pictureRuns,
} from '../components/wallPictures'
import type { ProgressAnswers } from './useProgress'
import type { WallAnswers } from './useWall'
import { cardsOf, WALL_CARDS, cardRoute, type WallCardId } from './taskAreas'

/* ------------------------------------------------ the ways into the runs */

/**
 * The name a link into the runs behind a mark carries.
 *
 * **The link says what it opens.** A mark on a chart is a shape, and a
 * shape has no words in it, so the name carries the date and what is
 * behind it — "The runs of Sun 10 May". The reader does not have to
 * hover to find out, and a screen reader says the same words.
 */
function runsName(kind: 'day' | 'week' | 'month', when: string): string {
  if (kind === 'day') return t('progress.wall.runs.day', { date: when })
  if (kind === 'week') return t('progress.wall.runs.week', { date: when })
  return t('progress.wall.runs.month', { month: when })
}

/**
 * A week, as the app measures it: the Monday and the six days after it.
 *
 * The same seven days the weekly volume chart uses, so a week on one
 * card and a week on another are one week.
 */
function weekEnd(weekStart: string): string {
  return shiftDate(weekStart, 6)
}

/** A month, as the app measures it: the first of the month to the last. */
function monthRange(key: string): { from: string; to: string } {
  const first = `${key}-01`
  const date = parseISO(first)
  if (!isValid(date)) return { from: first, to: first }
  return { from: first, to: format(endOfMonth(date), 'yyyy-MM-dd') }
}

/**
 * A card of the wall.
 *
 * The mock's card: a name in the corner, the way into the panel beside
 * it, and the answer under. A card that spans two columns of the wall is
 * a wide one — the mock's calendar, its table, its chart.
 */
function WallCard({
  title,
  to,
  wide = false,
  testId,
  openTestId,
  children,
}: {
  title: string
  /** Where the card's "open →" goes. */
  to?: string
  /** Whether the card spans two columns of the wall. */
  wide?: boolean
  /** The card's own test id. */
  testId?: string
  /** The test id of the card's "open →" link. */
  openTestId?: string
  children: ReactNode
}) {
  return (
    <section className={`card px-4 py-3 ${wide ? 'progress-card--wide' : ''}`} data-testid={testId}>
      <div className="mb-2 flex items-baseline justify-between gap-3">
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {title}
        </h2>
        {to ? (
          <Link
            to={to}
            className="tap-inline shrink-0 text-[12.5px] font-semibold text-brand-700 underline"
            data-testid={
              openTestId ??
              (testId
                ? `progress-wall-open-${testId.replace(/^progress-wall-/, '')}`
                : undefined)
            }
          >
            {t('progress.wall.open')}
          </Link>
        ) : null}
      </div>
      {children}
    </section>
  )
}

/** The window a card's numbers cover, named under them. */
function Window({ children }: { children: ReactNode }) {
  return <p className="mt-2 text-[12px] text-ink-faint">{children}</p>
}

/* ---------------------------------------------------------- the answers */

/**
 * The activity heatmap: the year's days, coloured by how far the day went.
 *
 * The bands are the mock's own, and they are named in the legend under
 * the drawing, so the colour is never a mystery.
 */
const DISTANCE_BANDS = [
  { key: 'progress.wall.calendar.none', colour: 'var(--surface-sunken)' },
  { key: 'progress.wall.calendar.l1', colour: 'var(--brand-100)' },
  { key: 'progress.wall.calendar.l2', colour: 'var(--brand-300)' },
  { key: 'progress.wall.calendar.l3', colour: 'var(--brand-500)' },
  { key: 'progress.wall.calendar.l4', colour: 'var(--brand-700)' },
] as const

/** The band's a day's distance falls in. */
function distanceBand(distance: number | null): number {
  if (distance === null) return 0
  if (distance < 5000) return 1
  if (distance < 10000) return 2
  if (distance < 15000) return 3
  return 4
}

export function DistanceCalendar({ days }: { days: YearDay[] }) {
  if (days.length === 0) return null

  // The first day's weekday decides where the grid starts: the weeks are
  // the athlete's own, Monday through Sunday.
  const first = new Date(`${days[0]?.date ?? ''}T00:00:00`)
  const offset = Number.isNaN(first.getTime()) ? 0 : (first.getDay() + 6) % 7
  const columns = Math.ceil((offset + days.length) / 7)

  return (
    <svg
      viewBox={`0 0 ${columns} 7`}
      preserveAspectRatio="xMinYMin meet"
      className="block h-auto w-full"
      role="group"
      aria-label={t('progress.wall.calendar.legend')}
      data-testid="progress-wall-calendar"
    >
      {days.map((day, index) => {
        const column = Math.floor((offset + index) / 7)
        const row = (offset + index) % 7
        const band = distanceBand(day.distance?.raw ?? null)
        const rect = (
          <rect
            x={column}
            y={row}
            width={0.86}
            height={0.86}
            rx={0.12}
            fill={DISTANCE_BANDS[band]?.colour ?? 'var(--surface-sunken)'}
            data-testid="progress-wall-calendar-day"
            data-date={day.date}
          >
            <title>
              {t('year.day.title', {
                date: formatDateMedium(day.date),
                distance: day.distance?.text ?? t('year.day.none'),
                high: day.distance?.text ?? t('year.day.noMix'),
              })}
            </title>
          </rect>
        )
        // A day with no run in it is a quiet mark: absence is not a door.
        if (day.sessions <= 0) return <Fragment key={day.date}>{rect}</Fragment>
        // A day with a run in it opens the runs of that day.
        return (
          <Link
            key={day.date}
            to={runsPath(day.date)}
            aria-label={runsName('day', formatDateShort(day.date))}
            data-testid={`progress-wall-calendar-${day.date}`}
          >
            {rect}
          </Link>
        )
      })}
    </svg>
  )
}

/** The legend of the calendar: the bands, named. */
export function DistanceLegend() {
  return (
    <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1">
      {DISTANCE_BANDS.map((band) => (
        <span
          key={band.key}
          className="inline-flex items-center gap-1 text-[11.5px] text-ink-muted"
        >
          <i
            className="inline-block h-2.5 w-2.5 rounded-[3px]"
            style={{ background: band.colour }}
          />
          {t(band.key)}
        </span>
      ))}
    </div>
  )
}

/**
 * The pace, as the mock draws it: a bar for each week of the window.
 *
 * The weeks are the answer's own, and the bar's height is the week's
 * distance against the furthest week of the window.
 */
export function PaceBars({
  pace,
}: {
  pace: {
    week_start: string
    /** How many runs the week holds. */
    sessions: number
    pace: { raw: number | null; text: string } | null
  }[]
}) {
  /**
   * **The pace, and not the distance.**
   *
   * The card is titled "The pace" and the preview is the first thing in
   * it, so the preview has to be the pace: a preview of the distance
   * under a pace title answers a different question from the one the
   * card asks. The bars are the pace, and a faster week is a taller
   * one — the pace is a time, and a shorter one is further along.
   */
  const paces = pace
    .map((week) => week.pace?.raw ?? null)
    .filter((value): value is number => value !== null && Number.isFinite(value))
  const slowest = paces.length > 0 ? Math.max(...paces) : 0
  const fastest = paces.length > 0 ? Math.min(...paces) : 0
  const span = slowest - fastest

  return (
    <div className="mt-2 flex h-11 items-end gap-[2px]" data-testid="progress-wall-pace-bars">
      {pace.map((week) => {
        const value = week.pace?.raw ?? null
        // A week with no run has no pace, and a pace that cannot be read
        // against the others is not drawn at all.
        const share =
          value === null || span <= 0 ? null : (slowest - value) / span
        const height = `${share === null ? 3 : Math.max(6, share * 100)}%`
        const className = `min-w-[2px] flex-1 rounded-t-[2px] ${share === null ? 'bg-surface-sunken' : 'bg-brand-400'}`
        const title = `${week.week_start}${week.pace ? ` · ${week.pace.text}` : ''}`
        // A week with no run in it is a quiet mark: absence is not a door.
        if (week.sessions <= 0) {
          return (
            <span
              key={week.week_start}
              className={className}
              style={{ height }}
              title={title}
            />
          )
        }
        // A week with a run in it opens the runs of that week.
        return (
          <Link
            key={week.week_start}
            to={runsPath(week.week_start, weekEnd(week.week_start))}
            aria-label={runsName('week', formatDateShort(week.week_start))}
            data-testid={`progress-wall-pace-${week.week_start}`}
            className={`block ${className}`}
            style={{ height }}
            title={title}
          />
        )
      })}
    </div>
  )
}

/**
 * This week against last week.
 *
 * The week to date, the same days of the week before, and the difference
 * between them — the core's own rendering of it, sign and all. It is a
 * comparison and not a verdict.
 */
export function WeekBody({ today }: { today: TodayInsight | undefined }) {
  if (!today) return <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  const week = today.week
  const days = week.days
  const from = days[0]?.date ?? ''
  const to = days[days.length - 1]?.date ?? ''

  return (
    <div data-testid="progress-wall-week">
      <div>
        <span className="text-[12px] text-ink-muted">{t('progress.wall.week.distance')}</span>
        <p className="text-[26px] font-semibold leading-tight tabular-nums text-ink">
          {renderedText(week.distance)}
        </p>
        {week.difference ? (
          <p
            className="text-[12.5px] text-ink-muted"
            data-testid="progress-wall-week-difference"
          >
            {renderedText(week.difference)}
          </p>
        ) : null}
      </div>
      <dl className="mt-2 space-y-1.5">
        <div className="flex items-baseline justify-between gap-3">
          <dt className="text-[12.5px] text-ink-soft">{t('progress.wall.week.sessions')}</dt>
          <dd className="text-[15px] font-semibold tabular-nums text-ink">
            {formatCount(week.sessions)}
          </dd>
        </div>
        <div className="flex items-baseline justify-between gap-3">
          <dt className="text-[12.5px] text-ink-soft">{t('progress.wall.week.before')}</dt>
          <dd className="text-[15px] font-semibold tabular-nums text-ink">
            {renderedText(week.previous.distance)} · {formatCount(week.previous.sessions)}
          </dd>
        </div>
      </dl>
      <Window>
        {t('progress.wall.week.window', {
          from: formatDateMedium(from),
          to: formatDateMedium(to),
          beforeFrom: formatDateMedium(week.previous.from),
          beforeTo: formatDateMedium(week.previous.to),
        })}
      </Window>
    </div>
  )
}

/**
 * The streaks: the days, the weeks and the months, and the longest run
 * of each.
 */
export function StreaksBody({ streaks }: { streaks: StreakInsight | undefined }) {
  if (!streaks) return <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  const rows = [
    { key: 'days', label: t('progress.wall.streaks.days'), streak: streaks.days.current },
    { key: 'weeks', label: t('progress.wall.streaks.weeks'), streak: streaks.weeks.current },
    { key: 'months', label: t('progress.wall.streaks.months'), streak: streaks.months.current },
  ]
  const longestDays = streaks.days.longest?.count ?? null
  const longestMonths = streaks.months.longest?.count ?? null

  return (
    <div data-testid="progress-wall-streaks">
      <dl className="space-y-1.5">
        {rows.map((row) => (
          <div key={row.key} className="flex items-baseline justify-between gap-3">
            <dt className="text-[13px] text-ink-soft">{row.label}</dt>
            <dd
              className="text-[17px] font-semibold tabular-nums text-ink"
              data-testid={`progress-wall-streak-${row.key}`}
            >
              {row.streak ? formatCount(row.streak.count) : t('today.streak.none')}
            </dd>
          </div>
        ))}
      </dl>
      {longestDays !== null || longestMonths !== null ? (
        <p
          className="mt-2 text-[11.5px] text-ink-faint"
          data-testid="progress-wall-streaks-longest"
        >
          {t('progress.wall.streaks.longest', {
            days: longestDays === null ? ABSENT : formatCount(longestDays),
            months: longestMonths === null ? ABSENT : formatCount(longestMonths),
          })}
        </p>
      ) : null}
    </div>
  )
}

/**
 * A period on the chart: the mark, and where it opens when the period
 * holds runs.
 *
 * A period with nothing in it is a quiet mark: absence is not a door.
 */
interface PeriodMark {
  label: string
  km: number | null
  text: string | null
  /** Where the mark opens, when the period holds runs. */
  to?: string
  /** The name the link carries. */
  name?: string
  /** The test id of the link. */
  linkTestId?: string
}

/**
 * A period's distance, as the mock's chart: a bar for each period.
 *
 * The chart is the app's own drawing, and it is HTML rather than SVG —
 * a drawing that scales with its card must not scale its words with it.
 * The axis's labels are the core's own words ("58.4 km"), so no number
 * on the axis is a bare number, and the whole period is in the tooltip.
 */
function PeriodBars({
  rows,
  testId,
  label,
}: {
  rows: PeriodMark[]
  testId: string
  label: string
}) {
  const ticks = axisTicks(
    rows.map((row) => ({ value: row.km, text: row.text })),
    3,
  )
  const furthest = Math.max(...rows.map((row) => row.km ?? 0), 1)
  const plot = 96

  return (
    <div data-testid={testId} aria-label={label}>
      <div className="flex gap-2">
        {/* the axis: the periods' own values, with their units */}
        <div className="relative w-[54px] shrink-0" style={{ height: plot }}>
          {ticks.map((tick) => (
            <span
              key={tick.value}
              className="absolute right-0 -translate-y-1/2 text-[10px] tabular-nums text-ink-faint"
              style={{ bottom: `${(tick.value / furthest) * 100}%` }}
            >
              {tick.text}
            </span>
          ))}
        </div>
        {/* the bars */}
        <div className="flex flex-1 items-end gap-1" style={{ height: plot }}>
          {rows.map((row) => (
            <div
              key={row.label}
              className="flex h-full flex-1 flex-col justify-end"
              title={`${row.label}: ${row.text ?? ABSENT}`}
            >
              <div
                className="relative rounded-t-[3px] bg-brand-400"
                style={{ height: `${row.km ? (row.km / furthest) * 100 : 0}%` }}
                data-testid={`${testId}-bar`}
              >
                {/*
                  The mark is the doorway: the link covers the bar and
                  nothing else, so the chart still reads as a chart.
                */}
                {row.to ? (
                  <Link
                    to={row.to}
                    aria-label={row.name}
                    data-testid={row.linkTestId}
                    className="absolute inset-0 block"
                  />
                ) : null}
              </div>
            </div>
          ))}
        </div>
      </div>
      {/* the periods, under their own bars */}
      <div className="mt-1 flex gap-1 pl-[62px]">
        {rows.map((row) => (
          <span
            key={row.label}
            className="flex-1 text-center text-[10px] leading-tight text-ink-faint"
            title={row.label}
          >
            {row.label}
          </span>
        ))}
      </div>
    </div>
  )
}

/** The months of the history, as the mock's chart. */
export function MonthsChart({ rolling }: { rolling: RollingInsight | undefined }) {
  const rows = (rolling?.months ?? []).map((month) => {
    const range = monthRange(month.key)
    // A month with no run in it is a quiet mark: absence is not a door.
    const hasRuns = month.sessions > 0
    return {
      label: month.label,
      km: month.distance?.raw == null ? null : month.distance.raw / 1000,
      text: month.distance?.text ?? null,
      to: hasRuns ? runsPath(range.from, range.to) : undefined,
      name: hasRuns ? runsName('month', month.label) : undefined,
      linkTestId: hasRuns ? `progress-wall-months-${month.key}` : undefined,
    }
  })
  if (rows.length === 0)
    return <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
  return (
    <PeriodBars
      rows={rows}
      testId="progress-months"
      label={t('progress.wall.months.heading')}
    />
  )
}

/**
 * The weekday split, as the mock's table: the day, the runs and how far
 * they went.
 */
export function WeekdayTable({ rolling }: { rolling: RollingInsight | undefined }) {
  const rows = rolling?.weekday ?? []
  if (rows.length === 0)
    return <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
  return (
    <table
      className="w-full table-fixed border-collapse text-[12.5px]"
      data-testid="progress-weekday"
    >
      <thead>
        <tr>
          <th className="w-[40%] pb-1 text-left text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('progress.wall.day')}
          </th>
          <th className="w-[24%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('progress.wall.distances.sessions')}
          </th>
          <th className="w-[36%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('vital.distance')}
          </th>
        </tr>
      </thead>
      <tbody>
        {rows.map((day) => (
          <tr key={day.weekday} className="border-t border-edge">
            <td className="py-1 pr-2 text-ink-soft">{day.weekday}</td>
            <td className="py-1 text-right tabular-nums text-ink-muted">
              {formatCount(day.sessions)}
            </td>
            <td className="py-1 text-right font-semibold tabular-nums text-ink">
              {renderedText(day.distance)}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}

/** The daytime split, as the mock's table: the hours, the runs, the distance. */
export function DaytimeTable({ rolling }: { rolling: RollingInsight | undefined }) {
  const rows = rolling?.daytime ?? []
  if (rows.length === 0)
    return <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
  return (
    <table
      className="w-full table-fixed border-collapse text-[12.5px]"
      data-testid="progress-daytime"
    >
      <thead>
        <tr>
          <th className="w-[46%] pb-1 text-left text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('progress.wall.timeOfDay')}
          </th>
          <th className="w-[22%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('progress.wall.distances.sessions')}
          </th>
          <th className="w-[32%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
            {t('vital.distance')}
          </th>
        </tr>
      </thead>
      <tbody>
        {rows.map((bucket) => (
          <tr key={bucket.key} className="border-t border-edge">
            <td className="py-1 pr-2 text-ink-soft">
              {bucket.label}
              <span className="block text-[11px] tabular-nums text-ink-faint">
                {t('progress.rolling.hours', {
                  from: hour(bucket.from_hour),
                  to: hour(bucket.to_hour),
                })}
              </span>
            </td>
            <td className="py-1 text-right tabular-nums text-ink-muted">
              {formatCount(bucket.sessions)}
            </td>
            <td className="py-1 text-right font-semibold tabular-nums text-ink">
              {renderedText(bucket.distance)}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}

/**
 * The distance breakdown, as the mock's table: the bucket, the runs in
 * it and how far they went.
 */
export function DistancesTable({ rolling }: { rolling: RollingInsight | undefined }) {
  const rows = rolling?.distances ?? []
  if (rows.length === 0)
    return <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
  return (
    <div data-testid="progress-distances">
      <table className="w-full table-fixed border-collapse text-[12.5px]">
        <thead>
          <tr>
            <th className="w-[40%] pb-1 text-left text-[10.5px] uppercase tracking-wide text-ink-faint">
              {t('vital.distance')}
            </th>
            <th className="w-[24%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
              {t('progress.wall.distances.sessions')}
            </th>
            <th className="w-[36%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint">
              {t('progress.wall.distances.total')}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((bucket) => (
            <tr key={bucket.key} className="border-t border-edge">
              <td className="py-1 pr-2 text-ink-soft">{bucket.label}</td>
              <td className="py-1 text-right tabular-nums text-ink-muted">
                {formatCount(bucket.sessions)}
              </td>
              <td className="py-1 text-right font-semibold tabular-nums text-ink">
                {renderedText(bucket.distance)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-distances-window">
        {t('progress.rolling.window', { days: rolling?.window.days ?? 0 })}
      </p>
    </div>
  )
}

/**
 * The weeks of the window, each one a doorway into that week's runs.
 *
 * The rolling window is the range the screen is showing, and its weeks
 * are the calendar weeks of that range — the same weeks the weekly
 * volume chart uses. A week with no run in it is a quiet mark:
 * absence is not a door.
 */
function WindowWeeks({
  weeks,
  runs,
}: {
  /** The weeks of the window, Monday first. */
  weeks: string[]
  /** How many runs each of them holds. */
  runs: Map<string, number>
}) {
  if (weeks.length === 0) return null
  return (
    <div className="mt-2">
      <p id="progress-wall-window-weeks-label" className="text-[11.5px] text-ink-faint">
        {t('progress.wall.window.weeks')}
      </p>
      <div
        className="mt-1 flex gap-[2px]"
        role="group"
        aria-labelledby="progress-wall-window-weeks-label"
        data-testid="progress-wall-window-weeks"
      >
        {weeks.map((week) => {
          const sessions = runs.get(week) ?? 0
          const className = `min-w-[2px] flex-1 rounded-[2px] ${sessions > 0 ? 'bg-brand-400' : 'bg-surface-sunken'}`
          const style = { height: '8px' }
          if (sessions <= 0) {
            return <span key={week} className={className} style={style} />
          }
          return (
            <Link
              key={week}
              to={runsPath(week, weekEnd(week))}
              aria-label={runsName('week', formatDateShort(week))}
              data-testid={`progress-wall-window-${week}`}
              className={`block ${className}`}
              style={style}
            />
          )
        })}
      </div>
    </div>
  )
}

/* ------------------------------------------------------------- the wall */

/**
 * The catalogue key that names each card.
 *
 * The name of a card is the model's business: the wall reads it from
 * the task areas, so the wall and the model cannot disagree about what
 * a card is called.
 */
const CARD_TITLE_KEYS = new Map(WALL_CARDS.map((card) => [card.id, card.titleKey] as const))

/** The name of a card, in the catalogue's own words. */
function cardTitle(id: WallCardId): string {
  const key = CARD_TITLE_KEYS.get(id)
  if (!key) {
    // Every id in the areas is in the cards: the model is one place, and
    // a card that is not in it has no name to draw.
    throw new Error(`the wall has no card named "${id}"`)
  }
  return t(key)
}

/**
 * The wall.
 *
 * The cards are the model's: `taskAreas.ts` says which cards belong
 * together and in what order, and the wall draws the areas in the
 * model's order. Every card still answers one question about the
 * athlete's history, and its "open →" is still the doorway into the
 * panel that answers it.
 */
export function ProgressWall({
  answers,
  wall,
  onWeekClick,
  destination = 'progress',
}: {
  answers: ProgressAnswers
  wall: WallAnswers
  onWeekClick: (weekStart: string) => void
  /**
   * Which destination's areas this wall draws.
   *
   * The athlete profile is a destination of its own, and the areas are
   * split between the two pages: the work on Progress, the long pattern
   * on the athlete profile.
   */
  destination?: 'progress' | 'profile'
}) {
  const { data, load, rolling, streaks } = answers
  const {
    year,
    paces,
    profile,
  } = wall

  /**
   * The weeks of the rolling window, and the runs in each of them.
   *
   * The window is the range the screen is showing, and the answer's own
   * count of the runs in each week is what says whether a week holds
   * anything at all.
   */
  const windowRuns = new Map<string, number>()
  answers.weeks.forEach((week, index) => {
    windowRuns.set(week, answers.consistency[index]?.sessions ?? 0)
  })

  /**
   * The runs the pictures are drawn from.
   *
   * **The feed is the answer for a picture about the runs themselves.**
   * The window's runs are the range the screen is showing; the profile's
   * runs are the profile's own window, and the two are never mixed.
   */
  const rangeRuns = pictureRuns(answers.windowRuns.data?.activities)

  const profileRuns = pictureRuns(wall.profileRuns.data?.activities)

  /**
   * This week, as a picture.
   *
   * **The comparison is in the picture.** The seven columns are this
   * week's days, and the same days of the previous week stand beside
   * them, so the difference is read off the drawing and not out of a
   * sentence.
   */
  const todayWeek = answers.today.data?.week
  const weekPicture = todayWeek ? (
    <WeekPicture
      runs={rangeRuns}
      days={todayWeek.days.map((day) => day.date)}
      previousDays={daysOf(todayWeek.previous.from, todayWeek.previous.to)}
      difference={todayWeek.difference ?? null}
    />
  ) : (
    <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  )

  /**
   * The cards of the wall, each one exactly once.
   *
   * The map is keyed by the card's own id, so a card that is missing is
   * a type error and a card that is nowhere on the wall cannot be drawn.
   * The name of a card and the route it opens both come from the model.
   */
  const cards: Record<WallCardId, ReactNode> = {
    /* --------------------------------- this week, and the window it is in */
    week: (
      <WallCard title={cardTitle('week')} to={cardRoute('week')} testId="progress-wall-week">
        {weekPicture}
      </WallCard>
    ),

    /* -------------------------------------------- the training load */
    load: (
      <WallCard title={cardTitle('load')} to={cardRoute('load')} testId="progress-wall-load">
        {/* The card is the headline: the shape of the load and the
            numbers behind it. The panel carries the whole answer. */}
        <LoadPanel load={load.data} error={load.isError} bare detail={false} />
      </WallCard>
    ),

    /* ------------------------------------ the weeks, and the sessions */
    weeks: (
      <WallCard title={cardTitle('weeks')} to={cardRoute('weeks')} testId="progress-wall-weeks">
        <VolumePanel
          volume={answers.volume}
          ticks={answers.volumeTicks}
          typicalWeek={data?.typical_week ?? null}
          onWeekClick={onWeekClick}
          bare
        />
        <div className="mt-3">
          <ConsistencyPanel consistency={answers.consistency} bare />
        </div>
        <WindowWeeks weeks={answers.weeks} runs={windowRuns} />
      </WallCard>
    ),

    /* ----------------------------------------- where the time went */
    time: (
      <WallCard title={cardTitle('time')} to={cardRoute('time')} wide testId="progress-wall-time">
        {/*
          The ledger: **the two blocks** — the app's own zone time, and
          the watch's own zone minutes — with the overlap underneath
          them. The source line is hoisted to the screen: it is named
          once, above the cards, and no card repeats it.
        */}
        <LedgerCard stimulus={answers.stimulus.data} error={answers.stimulus.isError} bare />
      </WallCard>
    ),

    /* ------------------------------------------------ the pace */
    pace: (
      <WallCard
        title={cardTitle('pace')}
        to={cardRoute('pace')}
        wide
        testId="progress-wall-pace"
        openTestId="progress-pace-link"
      >
        {/*
          The ladder: the anchors, the rungs with their values, their
          bands and their rules. **The card is the picture** — the rungs
          are drawn, and the caption names the race the paces came from.
        */}
        <LadderCard paces={paces.data} error={paces.isError} bare />
        {/*
          The drill-through: the curve is a second reading of the same
          athlete, and it has a panel of its own.
        */}
        <Link
          className="tap-inline mt-2 inline-block text-[12.5px] font-semibold text-brand-700 underline"
          to="/progress/curve"
          data-testid="progress-wall-pace-curve"
        >
          {t('progress.wall.curve.heading')}
        </Link>
      </WallCard>
    ),

    /* ------------------------------------------ bests & milestones */
    bests: (
      <WallCard
        title={cardTitle('bests')}
        to={cardRoute('bests')}
        testId="progress-wall-bests"
      >
        <BestsPicture runs={rangeRuns} bests={data?.bests ?? []} />
      </WallCard>
    ),

    /* ------------------------------------------- the athlete's shape */
    shape: (
      <WallCard title={cardTitle('shape')} to={cardRoute('shape')} testId="progress-wall-shape">
        {/* The facets are the profile's own facets, so they are read
            over the **profile's scope** and not the page's range. */}
        <ShapeGauges facets={profile?.profile ?? []} />
        <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-wall-shape-window">
          {t('progress.rolling.window', { days: profile?.window.days ?? 0 })}
        </p>
      </WallCard>
    ),

    /* --------------------------------------------------- the months */
    months: (
      <WallCard
        title={cardTitle('months')}
        to={cardRoute('months')}
        wide
        testId="progress-wall-months"
      >
        <MonthsChart rolling={profile} />
        {year.data ? (
          <div className="mt-3">
            <DistanceCalendar days={year.data.days} />
            <DistanceLegend />
          </div>
        ) : null}
      </WallCard>
    ),

    /* ------------------------------------------------ the schedule */
    schedule: (
      <WallCard
        title={cardTitle('schedule')}
        to={cardRoute('schedule')}
        wide
        testId="progress-wall-schedule"
      >
        <SchedulePicture rolling={profile} />
      </WallCard>
    ),

    /* --------------------------------------------------- the habit */
    habit: (
      <WallCard title={cardTitle('habit')} to={cardRoute('habit')} testId="progress-wall-habit">
        <HabitPicture
          runs={profileRuns}
          from={profile?.window.from ?? ''}
          to={profile?.window.to ?? ''}
          streak={streaks.data?.days.current ?? null}
          eddington={data?.eddington ?? null}
        />
      </WallCard>
    ),

    /* --------------------------------------------------- the goals */
    goals: (
      <WallCard title={cardTitle('goals')} to={cardRoute('goals')} testId="progress-wall-goals">
        <ChallengePanel challenges={answers.challenges.data} bare />
        {data?.adherence ? (
          <div className="mt-3">
            <AdherencePanel adherence={data.adherence} bare />
          </div>
        ) : null}
      </WallCard>
    ),
  }

  return (
    <div className="progress-grid mt-3" data-testid="progress-wall">
      {/*
        The source line, hoisted.

        **One per screen.** The zone model the window was measured
        against is named once, here, and the list of the models is behind
        a click — no card repeats it, and an answer that used a different
        model says so in place.
      */}
      {destination === 'progress' && rolling.data?.zones.source ? (
        <p
          className="col-span-full text-[11.5px] text-ink-faint"
          data-testid="progress-wall-zones-source"
        >
          {rolling.data.zones.source}
          {rolling.data.zones.models.length > 1 ? (
            <details className="ml-1 inline" data-testid="progress-wall-zones-models">
              <summary className="inline cursor-pointer">
                {t('progress.rolling.zones.models')}
              </summary>
              <ul className="mt-1 space-y-0.5">
                {rolling.data.zones.models.map((model) => (
                  <li key={model.read_on}>{model.source}</li>
                ))}
              </ul>
            </details>
          ) : null}
        </p>
      ) : null}
      {/*
        The wall draws the model's cards, in the model's order. A card
        is a question, and the panel behind it answers it: six cards on
        Progress, five on the athlete profile.
      */}
      {cardsOf(destination).map((card) => (
        <Fragment key={card.id}>{cards[card.id]}</Fragment>
      ))}
    </div>
  )
}
