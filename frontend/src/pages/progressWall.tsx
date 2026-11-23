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
import {
  ProfileRadar,
  RADAR_AXES,
  type RadarAxis,
  type RadarSeries,
} from '../components/ProfileRadar'
import { axisTicks } from '../lib/chart'
import { YearCalendar } from '../components/YearCalendar'
import type {
  ActivitySummary,
  RollingInsight,
  StreakInsight,
  TodayInsight,
  VitalsSnapshot,
  YearDay,
} from '../lib/types'
import {
  AdherencePanel,
  ProfileScopePicker,
  ChallengePanel,
  BestsPanel,
  ConsistencyPanel,
  EddingtonPanel,
  LoadPanel,
  ProfilePanel,
  RollingPanel,
  ShapePanel,
  VolumePanel,
  ZonesDistributionPanel,
  ZonesPanel,
  hour,
} from './progressPanels'
import type { ProgressAnswers } from './useProgress'
import type { WallAnswers } from './useWall'
import { areasOf, WALL_CARDS, cardRoute, type WallCardId } from './taskAreas'

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

/** The mock's big number: the answer at the top of a card. */
function BigValue({ children }: { children: ReactNode }) {
  return (
    <p className="text-[22px] font-semibold leading-tight tabular-nums text-ink">{children}</p>
  )
}

/** The mock's small note: what the card is, under the number. */
function Note({ children }: { children: ReactNode }) {
  return <p className="mt-1 text-[12.5px] leading-snug text-ink-muted">{children}</p>
}

/** The window a card's numbers cover, named under them. */
function Window({ children }: { children: ReactNode }) {
  return <p className="mt-2 text-[12px] text-ink-faint">{children}</p>
}

/* ---------------------------------------------------------- the answers */

/**
 * The six axes of the athlete profile, in the order they are drawn.
 *
 * The facets are the answer's own — the key, the name and the value are
 * the core's — and the order is the chart's.
 */
function axesOf(rolling: RollingInsight, field: 'value' | 'previous'): RadarAxis[] {
  return RADAR_AXES.map((key) => {
    const facet = rolling.profile.find((entry) => entry.key === key)
    const value = field === 'value' ? facet?.value : facet?.previous
    return {
      key,
      label: facet?.label ?? key,
      value: value?.raw ?? null,
      text: value?.text ?? null,
    }
  })
}

/**
 * The athlete profile: two windows on one chart.
 *
 * The radar draws the two polygons and the table under it holds the
 * numbers behind them, so a reader who cannot read the shape can still
 * read the values — the core's own rendering of each one.
 */
export function ProfileRadarBody({
  rolling,
  months,
  onScope,
}: {
  rolling: RollingInsight | undefined
  /** The window the profile is drawn over, in months. */
  months: number
  /** The scope the athlete chooses. */
  onScope: (months: number) => void
}) {
  const series: RadarSeries[] = []
  if (rolling) {
    // The window, and the window before it of the same length.
    series.push({
      key: 'now',
      label: t('progress.wall.profile.window', { days: rolling.window.days }),
      axes: axesOf(rolling, 'value'),
    })
    series.push({
      key: 'before',
      label: t('progress.wall.profile.previous', { days: rolling.window.days }),
      axes: axesOf(rolling, 'previous'),
    })
  }

  if (series.length === 0) {
    return <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  }

  return (
    <div data-testid="progress-wall-radar">
      <ProfileScopePicker months={months} onChange={onScope} />
      <ProfileRadar series={series} />
      <Note>{t('progress.wall.profile.note')}</Note>
      {series.length > 1 ? (
        <table className="mt-2 w-full table-fixed border-collapse text-[12px]">
          <thead>
            <tr>
              <th className="w-[34%] pb-1 text-left text-[10.5px] uppercase tracking-wide text-ink-faint">
                {t('progress.wall.profile.axis')}
              </th>
              {series.map((entry) => (
                <th
                  key={entry.key}
                  className="w-[33%] pb-1 text-right text-[10.5px] uppercase tracking-wide text-ink-faint"
                >
                  {entry.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {series[0]!.axes.map((axis, index) => (
              <tr key={axis.key} className="border-t border-edge">
                <td className="py-1 pr-2 text-left text-ink-soft">{axis.label}</td>
                {series.map((entry) => {
                  const other = entry.axes[index]
                  return (
                    <td
                      key={entry.key}
                      className="py-1 text-right font-semibold tabular-nums text-ink"
                      data-testid={`progress-wall-radar-${entry.key}-${axis.key}`}
                    >
                      {other?.text ?? ABSENT}
                    </td>
                  )
                })}
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </div>
  )
}

/**
 * The activity heatmap: the year's days, coloured by how far the day went.
 *
 * The bands are the mock's own, and they are named in the legend under
 * the drawing, so the colour is never a mystery.
 */
const DISTANCE_BANDS = [
  { key: 'progress.wall.heatmap.none', colour: 'var(--surface-sunken)' },
  { key: 'progress.wall.heatmap.l1', colour: 'var(--brand-100)' },
  { key: 'progress.wall.heatmap.l2', colour: 'var(--brand-300)' },
  { key: 'progress.wall.heatmap.l3', colour: 'var(--brand-500)' },
  { key: 'progress.wall.heatmap.l4', colour: 'var(--brand-700)' },
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
      aria-label={t('progress.wall.heatmap.legend')}
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
            data-testid={`progress-wall-heatmap-${day.date}`}
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
 * The most recent activities, as the mock's table: the date, the name,
 * the distance, the time and the heart rate.
 *
 * Every cell is the answer's own — the core's rendering of the distance,
 * the time and the heart rate — and a run with no heart rate says so.
 */
export function ActivitiesTable({ activities }: { activities: ActivitySummary[] }) {
  if (activities.length === 0) {
    return <p className="text-[13px] text-ink-muted">{t('runs.empty.title')}</p>
  }
  return (
    <table
      className="w-full table-fixed border-collapse text-[12.5px]"
      data-testid="progress-wall-activities-table"
    >
      <thead>
        <tr>
          {[
            { key: 'date', label: t('progress.wall.date'), width: 'w-[22%]' },
            { key: 'name', label: t('progress.wall.name'), width: 'w-[28%]' },
            { key: 'distance', label: t('vital.distance'), width: 'w-[16%]' },
            { key: 'time', label: t('vital.time'), width: 'w-[16%]' },
            { key: 'heart', label: t('vital.heartRate'), width: 'w-[18%]' },
          ].map((column) => (
            <th
              key={column.key}
              className={`${column.width} pb-1 text-left text-[10.5px] uppercase tracking-wide text-ink-faint`}
            >
              {column.label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {activities.map((activity) => (
          <tr key={activity.id} className="border-t border-edge align-top">
            <td className="py-1 pr-2 text-ink-soft">{formatDateShort(activity.local_date)}</td>
            <td className="py-1 pr-2 break-words text-ink-soft">{activity.name}</td>
            <td className="py-1 pr-2 tabular-nums text-ink">
              {renderedText(activity.distance)}
            </td>
            <td className="py-1 pr-2 tabular-nums text-ink">
              {renderedText(activity.duration)}
            </td>
            <td className="py-1 tabular-nums text-ink">
              {activity.heart_rate ? renderedText(activity.heart_rate) : ABSENT}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
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

/** The years of the history, as the mock's chart. */
export function YearsChart({ rolling }: { rolling: RollingInsight | undefined }) {
  const rows = (rolling?.years ?? []).map((year) => ({
    label: year.label,
    km: year.distance?.raw == null ? null : year.distance.raw / 1000,
    text: year.distance?.text ?? null,
  }))
  if (rows.length === 0)
    return <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
  return (
    <PeriodBars rows={rows} testId="progress-years" label={t('progress.wall.years.heading')} />
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
 * Last night, as the mock's card: each reading and how it sits against
 * the athlete's own usual.
 */
export function VitalsBody({ vitals }: { vitals: VitalsSnapshot | undefined }) {
  if (!vitals) return <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  const readings = vitals.vitals ?? []
  if (readings.length === 0) {
    return <p className="text-[13px] text-ink-muted">{t('today.state.noVitals')}</p>
  }
  return (
    <div data-testid="progress-wall-vitals">
      <dl className="space-y-1.5">
        {readings.map((reading) => (
          <div key={reading.key} className="flex items-baseline justify-between gap-3">
            <dt className="text-[12.5px] text-ink-soft">{reading.label}</dt>
            <dd className="text-right">
              <span className="text-[15px] font-semibold tabular-nums text-ink">
                {reading.value_text ?? ABSENT}
              </span>
              {reading.direction_text ? (
                <span className="ml-1.5 text-[11.5px] text-ink-muted">
                  {reading.direction_text}
                </span>
              ) : null}
            </dd>
          </div>
        ))}
      </dl>
      <Note>{t('progress.wall.body.note')}</Note>
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
  const { today, year, pace, activities, profile, profileScope, setProfileMonths, vitals } = wall

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
   * The cards of the wall, each one exactly once.
   *
   * The map is keyed by the card's own id, so a card that is missing is
   * a type error and a card that is nowhere on the wall cannot be drawn.
   * The name of a card and the route it opens both come from the model.
   */
  const cards: Record<WallCardId, ReactNode> = {
    /* -------------------------------------------- the training load */
    load: (
      <WallCard title={cardTitle('load')} to={cardRoute('load')} testId="progress-wall-load">
        {/* The card is the headline: the shape of the load and the
            numbers behind it. The panel carries the whole answer. */}
        <LoadPanel load={load.data} error={load.isError} bare detail={false} />
      </WallCard>
    ),

    /* --------------------------------------------- the weekly volume */
    volume: (
      <WallCard
        title={cardTitle('volume')}
        to={cardRoute('volume')}
        testId="progress-wall-volume"
      >
        <VolumePanel
          volume={answers.volume}
          ticks={answers.volumeTicks}
          typicalWeek={data?.typical_week ?? null}
          onWeekClick={onWeekClick}
          bare
        />
      </WallCard>
    ),

    /* ----------------------------------------------- the consistency */
    consistency: (
      <WallCard
        title={cardTitle('consistency')}
        to={cardRoute('consistency')}
        testId="progress-wall-consistency"
      >
        <ConsistencyPanel consistency={answers.consistency} bare />
      </WallCard>
    ),

    /* ---------------------------------------------- the time in zones */
    zones: (
      <WallCard
        title={cardTitle('zones')}
        to={cardRoute('zones')}
        testId="progress-wall-zones"
      >
        <ZonesDistributionPanel rolling={rolling.data} bare />
      </WallCard>
    ),

    /* ---------------------------------------------- the polarised split */
    split: (
      <WallCard
        title={cardTitle('split')}
        to={cardRoute('split')}
        testId="progress-wall-split"
      >
        <ZonesPanel rolling={rolling.data} bare />
      </WallCard>
    ),

    /* --------------------------------- this week against last week */
    week: (
      <WallCard title={cardTitle('week')} to={cardRoute('week')} testId="progress-wall-week">
        <WeekBody today={today.data} />
      </WallCard>
    ),

    /* --------------------------------------------- the rolling window */
    window: (
      <WallCard
        title={cardTitle('window')}
        to={cardRoute('window')}
        testId="progress-wall-window"
      >
        <RollingPanel rolling={rolling.data} error={rolling.isError} bare />
        <WindowWeeks weeks={answers.weeks} runs={windowRuns} />
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
        {pace.data ? (
          <>
            <BigValue>
              {t('progress.wall.pace.weeks', { n: formatCount(pace.data.window.weeks) })}
            </BigValue>
            <Note>{t('progress.wall.pace.note')}</Note>
            <PaceBars pace={pace.data.weeks} />
            <Window>
              {pace.data.window.from_label} – {pace.data.window.to_label}
            </Window>
          </>
        ) : (
          <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
        )}
      </WallCard>
    ),

    /* ------------------------------------------ bests & milestones */
    bests: (
      <WallCard
        title={cardTitle('bests')}
        to={cardRoute('bests')}
        testId="progress-wall-bests"
      >
        {data ? <BestsPanel bests={data.bests} bare /> : null}
      </WallCard>
    ),

    /* ------------------------------------------ the athlete profile */
    profile: (
      <WallCard
        title={cardTitle('profile')}
        to={cardRoute('profile')}
        testId="progress-wall-profile"
      >
        <ShapePanel
          rolling={profile.data}
          error={profile.isError}
          months={profileScope.months}
          onScope={setProfileMonths}
          bare
        />
      </WallCard>
    ),

    /* ------------------------------------------- the training profile */
    facets: (
      <WallCard
        title={cardTitle('facets')}
        to={cardRoute('facets')}
        testId="progress-wall-facets"
      >
        <ProfilePanel rolling={rolling.data} bare />
      </WallCard>
    ),

    /* -------------------------------------------------- the challenges */
    challenges: (
      <WallCard
        title={cardTitle('challenges')}
        to={cardRoute('challenges')}
        testId="progress-wall-challenges"
      >
        <ChallengePanel challenges={answers.challenges.data} bare />
      </WallCard>
    ),

    /* --------------------------------------------------- the adherence */
    adherence: data?.adherence ? (
      <WallCard
        title={cardTitle('adherence')}
        to={cardRoute('adherence')}
        testId="progress-wall-adherence"
      >
        <AdherencePanel adherence={data.adherence} bare />
      </WallCard>
    ) : null,

    /* ------------------------------------------------- the weekday split */
    weekday: (
      <WallCard
        title={cardTitle('weekday')}
        to={cardRoute('weekday')}
        wide
        testId="progress-wall-weekday"
      >
        <WeekdayTable rolling={rolling.data} />
      </WallCard>
    ),

    /* ------------------------------------------------- the daytime split */
    daytime: (
      <WallCard
        title={cardTitle('daytime')}
        to={cardRoute('daytime')}
        testId="progress-wall-daytime"
      >
        <DaytimeTable rolling={rolling.data} />
      </WallCard>
    ),

    /* --------------------------------------------- the distance breakdown */
    distances: (
      <WallCard
        title={cardTitle('distances')}
        to={cardRoute('distances')}
        testId="progress-wall-distances"
      >
        <DistancesTable rolling={rolling.data} />
      </WallCard>
    ),

    /* ------------------------------------------------- the year */
    year: (
      <WallCard title={cardTitle('year')} to={cardRoute('year')} wide testId="progress-wall-year">
        {year.data ? (
          <>
            <BigValue>
              {t('progress.wall.year.months', { n: formatCount(year.data.months.length) })}
            </BigValue>
            <Note>{t('progress.wall.year.note')}</Note>
            <Window>
              {year.data.window.from_label} – {year.data.window.to_label}
            </Window>
            <div className="mt-2">
              <YearCalendar days={year.data.days} />
            </div>
          </>
        ) : (
          <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
        )}
      </WallCard>
    ),

    /* -------------------------------------------- the activity heatmap */
    heatmap: (
      <WallCard
        title={cardTitle('heatmap')}
        to={cardRoute('heatmap')}
        wide
        testId="progress-wall-heatmap"
      >
        {year.data ? (
          <>
            <DistanceCalendar days={year.data.days} />
            <DistanceLegend />
          </>
        ) : (
          <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
        )}
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
        <MonthsChart rolling={rolling.data} />
      </WallCard>
    ),

    /* ---------------------------------------------------- the years */
    years: (
      <WallCard
        title={cardTitle('years')}
        to={cardRoute('years')}
        testId="progress-wall-years"
      >
        <YearsChart rolling={rolling.data} />
      </WallCard>
    ),

    /* --------------------------------------------------- the streaks */
    streaks: (
      <WallCard
        title={cardTitle('streaks')}
        to={cardRoute('streaks')}
        testId="progress-wall-streaks"
      >
        <StreaksBody streaks={streaks.data} />
      </WallCard>
    ),

    /* ----------------------------------------------- the Eddington number */
    eddington: (
      <WallCard
        title={cardTitle('eddington')}
        to={cardRoute('eddington')}
        testId="progress-wall-eddington"
      >
        {data ? <EddingtonPanel data={data} bare /> : null}
      </WallCard>
    ),

    /* ----------------------------------- the most recent activities */
    activities: (
      <WallCard
        title={cardTitle('activities')}
        to={cardRoute('activities')}
        wide
        testId="progress-wall-activities"
      >
        <ActivitiesTable activities={activities.data?.activities ?? []} />
      </WallCard>
    ),

    /* ------------------------------------------------- last night */
    body: (
      <WallCard title={cardTitle('body')} to={cardRoute('body')} testId="progress-wall-body">
        <VitalsBody vitals={vitals.data} />
      </WallCard>
    ),
  }

  return (
    <div className="progress-grid mt-3" data-testid="progress-wall">
      {/*
        The wall draws the model's areas, in the model's order. An area
        is a heading and nothing more: it names the group of cards and
        says nothing about them.
      */}
      {areasOf(destination).map((area) => (
        <section
          key={area.id}
          className="progress-area"
          data-testid={`progress-area-${area.id}`}
        >
          <h2 className="progress-area__heading text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t(area.titleKey)}
          </h2>
          <div className="progress-area__grid">
            {area.cards.map((id) => (
              <Fragment key={id}>{cards[id]}</Fragment>
            ))}
          </div>
        </section>
      ))}
    </div>
  )
}
