/**
 * The wall's pictures.
 *
 * **A card is read as a picture first.** The drawing carries the fact,
 * the numbers are labels on the drawing, and the caption names the
 * period and the reason for an absence — and nothing else. A card that
 * cannot be drawn is merged or dropped.
 *
 * The rules these drawings follow:
 *
 *   - **The picture carries the number.** A distance is a column's
 *     height, a share is a bar's width, a day is a mark. The number is
 *     also in the text beside the drawing, because the picture is for
 *     the eye and the text is the fallback.
 *   - **Absence is drawn.** A day with no run is an empty column, a
 *     week with no run is a hole in the strip — never a bar of zero,
 *     which would read as "no time".
 *   - **Nothing is computed here.** Every value is the router's own
 *     rendering, or a fact about a date (the weekday of a day, the
 *     distance of a run) that the drawing reads off the answer.
 *   - **A colour never judges.** The drawing uses the app's tokens, and
 *     no colour says a thing was good or bad.
 */

import { useMemo } from 'react'
import { t } from '../i18n'
import { ABSENT, formatCount, formatDateShort, renderedText } from '../lib/format'
import type {
  ActivitySummary,
  ProfileFacet,
  RollingInsight,
  ZoneWeekRow,
} from '../lib/types'

/**
 * The words a movement is written in.
 *
 * **The movement is the core's.** The facet says which way it moved,
 * and the app writes the word — it never works the difference out.
 */
const MOVEMENT: Record<string, 'progress.rolling.trend.up.short' | 'progress.rolling.trend.down.short' | 'progress.rolling.trend.steady.short'> = {
  up: 'progress.rolling.trend.up.short',
  down: 'progress.rolling.trend.down.short',
  steady: 'progress.rolling.trend.steady.short',
}

/* ------------------------------------------------------------ the runs */

/** One run, as the pictures draw it. */
export interface PictureRun {
  /** The run's own id. */
  id: number
  /** The athlete's local date: "2026-05-10". */
  date: string
  /** The distance, in kilometres. Null when the run recorded none. */
  km: number | null
}

/**
 * The runs of the feed, as the pictures draw them.
 *
 * **The date and the distance are facts about the run**, and the
 * pictures read them: the weekday of a day is a calendar fact, and the
 * distance is the core's own number.
 */
export function pictureRuns(activities: ActivitySummary[] | undefined): PictureRun[] {
  return (activities ?? []).map((activity) => ({
    id: activity.id,
    date: activity.local_date,
    km: activity.distance.raw == null ? null : activity.distance.raw / 1000,
  }))
}

/** The runs of one day, and the day's distance. */
function runsOfDay(runs: PictureRun[], date: string): PictureRun[] {
  return runs.filter((run) => run.date === date)
}

/** The distance of one day, in kilometres. Null when the day holds no run. */
function distanceOfDay(runs: PictureRun[], date: string): number | null {
  const day = runsOfDay(runs, date)
  if (day.length === 0) return null
  return day.reduce((total, run) => total + (run.km ?? 0), 0)
}

/* ------------------------------------------------------- 1 · this week */

/**
 * This week, as a picture.
 *
 * **Seven columns, one per day**, each column's height the distance that
 * day held. The same days of the previous week stand beside them in the
 * pale colour, so the comparison is *in* the picture and not in a
 * sentence under it.
 *
 * A day with no run is an empty column: absence is drawn, and it is
 * never a bar of zero.
 */
export function WeekPicture({
  runs,
  days,
  previousDays,
  difference,
}: {
  /** The runs of the window, the previous week included. */
  runs: PictureRun[]
  /** The days of this week, in order. */
  days: string[]
  /** The days of the previous week, the same elapsed days. */
  previousDays: string[]
  /** The difference, as the core renders it. */
  difference: { text: string; raw: number | null } | null
}) {
  const current = days.map((date) => distanceOfDay(runs, date))
  const previous = previousDays.map((date) => distanceOfDay(runs, date))
  const furthest = Math.max(...current.concat(previous).filter((value) => value !== null), 1)

  const column = (value: number | null, kind: 'now' | 'before') => {
    const height = value === null ? 0 : (value / furthest) * 100
    const style = {
      height: `${height}%`,
      background:
        value === null
          ? 'var(--surface-sunken)'
          : kind === 'now'
            ? 'var(--chart-line)'
            : 'var(--chart-line-alt)',
    }
    return <span className="block w-full rounded-t-[2px]" style={style} />
  }

  return (
    <div data-testid="progress-picture-week">
      <div className="flex items-baseline justify-between gap-3">
        <span className="text-[11.5px] text-ink-muted">{t('progress.wall.week.now')}</span>
        <span className="text-[11.5px] text-ink-muted">{t('progress.wall.week.previous')}</span>
      </div>
      <div
        className="mt-1 flex h-[92px] items-end gap-1"
        role="img"
        aria-label={t('progress.wall.week.picture')}
      >
        {days.map((date, index) => (
          <div key={date} className="flex h-full flex-1 items-end gap-[2px]">
            <span className="flex h-full flex-1 items-end">
              {column(previous[index] ?? null, 'before')}
            </span>
            <span className="flex h-full flex-1 items-end">
              {column(current[index] ?? null, 'now')}
            </span>
          </div>
        ))}
      </div>
      <div className="mt-1 flex gap-1">
        {days.map((date) => (
          <span
            key={date}
            className="flex-1 text-center text-[10px] leading-tight text-ink-faint"
          >
            {formatDateShort(date)}
          </span>
        ))}
      </div>
      {difference ? (
        <p className="mt-1 text-[12.5px] tabular-nums text-ink-muted" data-testid="progress-picture-week-difference">
          {t('progress.wall.week.difference', { difference: difference.text })}
        </p>
      ) : null}
    </div>
  )
}

/* ----------------------------------------------- 2 · where the time went */

/**
 * Where the time went, as one bar.
 *
 * **One bar, printed once.** The five zones are the five parts of it,
 * and the roll-up — the low, moderate and high shares — is *read off*
 * the bar, not printed beside it: a second drawing of one fact is not a
 * second fact, it is a second thing to disagree with the first.
 */
export function ZonePicture({
  rolling,
}: {
  /** The rolling answer: the window's split of the training. */
  rolling: RollingInsight | undefined
}) {
  const distribution = rolling?.zones.distribution ?? []

  if (distribution.length === 0) {
    return (
      <p className="text-[13px] text-ink-muted" data-testid="progress-picture-zones-empty">
        {rolling?.zones.unavailable ?? t('common.error')}
      </p>
    )
  }

  const colours = [
    'var(--chart-line-alt)',
    'var(--brand-300)',
    'var(--chart-line)',
    'var(--chart-reference)',
    'var(--accent-700)',
  ]

  return (
    <div data-testid="progress-picture-zones">
      <div
        className="flex h-4 w-full overflow-hidden rounded-[3px]"
        role="img"
        aria-label={t('progress.wall.time.picture')}
      >
        {distribution.map((zone, index) => (
          <span
            key={zone.zone}
            className="h-full"
            style={{
              width: `${Math.max(0, Math.min(100, zone.share?.raw ?? 0))}%`,
              background: colours[index % colours.length],
            }}
            data-testid={`progress-picture-zone-${zone.zone}`}
          />
        ))}
      </div>
      {/* The zones are named under the bar, with their own numbers: the
          labels of the picture, not a second sentence about it. */}
      <ul className="mt-2 space-y-1">
        {distribution.map((zone) => (
          <li key={zone.zone} className="flex items-baseline justify-between gap-3">
            <span className="text-[12.5px] text-ink-soft">
              {zone.zone}
              {zone.range ? (
                <span className="ml-1 text-[11px] text-ink-faint">{zone.range}</span>
              ) : null}
            </span>
            <span className="text-[12.5px] tabular-nums text-ink">
              {renderedText(zone.time)}
              <span className="ml-1 text-ink-muted">{renderedText(zone.share)}</span>
            </span>
          </li>
        ))}
      </ul>
      {/* The roll-up is not printed again: the low, moderate and high
          shares are **read off** the bar, and a second drawing of one
          fact is not a second fact. */}
    </div>
  )
}

/* ------------------------------------- 2b · the ledger's week matrix */

/**
 * The weeks × bands matrix.
 *
 * **The weeks of the window are the columns, the bands are the rows**,
 * and each cell is filled in proportion to the time that band held that
 * week. A week with no time in a band is **white space** — not a zero-
 * width bar, which would read as "no time", and not a missing cell,
 * which would read as "not known". It is a hole, and the hole is the
 * finding.
 *
 * Each row is scaled to its own busiest week, so the shape of a small
 * row is still readable.
 */
export function LedgerMatrix({
  weeks,
  rows,
}: {
  /** The window's calendar weeks, Monday first. */
  weeks: string[]
  /** The weeks that hold a run, and the time in each band. */
  rows: ZoneWeekRow[]
}) {
  const bands = ['low', 'moderate', 'high'] as const
  const byWeek = new Map(rows.map((row) => [row.week_start, row]))
  // Each row's own busiest week: the scale that makes the row readable.
  const busiest = bands.map((band) =>
    Math.max(
      0,
      ...rows.map((row) => {
        const found = row.bands.find((entry) => entry.key === band)
        return found?.time?.raw ?? 0
      }),
    ),
  )

  if (weeks.length === 0) return null

  return (
    <div data-testid="progress-picture-matrix">
      <div className="flex gap-[3px]">
        <div className="flex w-[74px] shrink-0 flex-col justify-between text-[11px] text-ink-muted">
          {bands.map((band) => (
            <span key={band} className="flex h-[18px] items-center">
              {rows[0]?.bands.find((entry) => entry.key === band)?.label ?? band}
            </span>
          ))}
        </div>
        <div className="flex flex-1 gap-[3px]">
          {weeks.map((week) => {
            const row = byWeek.get(week)
            return (
              <div key={week} className="flex flex-1 flex-col justify-between">
                {bands.map((band, bandIndex) => {
                  const found = row?.bands.find((entry) => entry.key === band)
                  const seconds = found?.time?.raw ?? null
                  const scale = busiest[bandIndex] ?? 0
                  const share =
                    seconds === null || scale <= 0
                      ? 0
                      : Math.max(0, Math.min(1, seconds / scale))
                  return (
                    <span
                      key={band}
                      className="flex h-[18px] items-end"
                      title={found?.time?.text ?? undefined}
                    >
                      <span
                        className="block w-full rounded-[1px]"
                        style={{
                          height: seconds === null ? '0%' : `${Math.max(8, share * 100)}%`,
                          background: 'var(--chart-line)',
                        }}
                        data-testid={`progress-picture-matrix-${week}-${band}`}
                      />
                    </span>
                  )
                })}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

/**
 * The longest runs, as a bar series.
 *
 * **The largest is held out**: it is drawn in the other colour, so the
 * reader sees at a glance which run of the window was the longest. The
 * bars are the runs themselves, and their lengths are the runs' own
 * distances.
 */
export function LongRunsSeries({ runs }: { runs: PictureRun[] }) {
  const withDistance = runs
    .filter((run): run is PictureRun & { km: number } => run.km !== null)
    .sort((a, b) => b.km - a.km)
    .slice(0, 5)
    .reverse()
  if (withDistance.length === 0) return null
  const furthest = Math.max(...withDistance.map((run) => run.km), 1)

  return (
    <div data-testid="progress-picture-long-runs">
      <div className="flex h-[42px] items-end gap-1">
        {withDistance.map((run, index) => (
          <div
            key={run.id}
            className="flex h-full flex-1 items-end"
            title={formatDateShort(run.date)}
          >
            <span
              className="block w-full rounded-t-[2px]"
              style={{
                height: `${(run.km / furthest) * 100}%`,
                background:
                  index === withDistance.length - 1 ? 'var(--chart-reference)' : 'var(--brand-300)',
              }}
              data-testid={`progress-picture-long-run-${run.id}`}
            />
          </div>
        ))}
      </div>
    </div>
  )
}

/* -------------------------------------------- 6 · bests & milestones */

/**
 * The records, as a picture.
 *
 * **Distance against date**: every run is a dot, and the three records
 * are marks of their own shapes, so the meaning is carried by the shape
 * and not only by the colour. Each record is labelled with its distance
 * and its date.
 */
export function BestsPicture({
  runs,
  bests,
}: {
  /** The runs of the window. */
  runs: PictureRun[]
  /** The records, as the answer carries them. */
  bests: { key: string; value: string; date: string }[]
}) {
  if (runs.length === 0) {
    return (
      <p className="text-[13px] text-ink-muted" data-testid="progress-picture-bests-empty">
        {t('progress.wall.bests.empty')}
      </p>
    )
  }

  const dates = runs.map((run) => run.date).sort()
  const from = dates[0] ?? ''
  const to = dates[dates.length - 1] ?? ''
  const days = Math.max(1, daysBetween(from, to))
  const furthest = Math.max(
    ...runs.map((run) => run.km ?? 0),
    ...bests.map((best) => Number.parseFloat(best.value) || 0),
    1,
  )

  const xOf = (date: string) => (daysBetween(from, date) / days) * 100
  const yOf = (km: number) => 100 - (km / furthest) * 100

  return (
    <div data-testid="progress-picture-bests">
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" className="block h-[120px] w-full">
        {/* The grid the dots are read against. */}
        {[0, 25, 50, 75, 100].map((line) => (
          <line
            key={line}
            x1="0"
            x2="100"
            y1={line}
            y2={line}
            style={{ stroke: 'var(--chart-grid)' }}
            strokeWidth="0.5"
            vectorEffect="non-scaling-stroke"
          />
        ))}
        {runs.map((run) =>
          run.km === null ? null : (
            <circle
              key={run.id}
              cx={xOf(run.date)}
              cy={yOf(run.km)}
              r="1.2"
              style={{ fill: 'var(--brand-300)' }}
              data-testid="progress-picture-bests-run"
            />
          ),
        )}
        {bests.map((best) => {
          const km = Number.parseFloat(best.value) || 0
          return (
            <circle
              key={best.key}
              cx={xOf(best.date)}
              cy={yOf(km)}
              r="2.4"
              style={{ fill: 'var(--chart-reference)', stroke: 'var(--surface)' }}
              strokeWidth="0.8"
              vectorEffect="non-scaling-stroke"
              data-testid={`progress-picture-bests-${best.key}`}
            />
          )
        })}
      </svg>
      <ul className="mt-1 space-y-0.5">
        {bests.map((best) => (
          <li
            key={best.key}
            className="flex items-baseline justify-between gap-3 text-[12.5px]"
          >
            <span className="text-ink-soft">{t(best.key as never)}</span>
            <span className="tabular-nums text-ink">
              {best.value}
              <span className="ml-1 text-ink-muted">{formatDateShort(best.date)}</span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  )
}

/** The days between two dates, as the pictures count them. */
function daysBetween(from: string, to: string): number {
  const start = Date.parse(`${from}T00:00:00Z`)
  const end = Date.parse(`${to}T00:00:00Z`)
  if (Number.isNaN(start) || Number.isNaN(end)) return 1
  return Math.max(1, Math.round((end - start) / 86_400_000))
}

/* --------------------------------------------------- 7 · your shape */

/**
 * The athlete's shape, as seven gauges.
 *
 * **Seven bars, and no polygon.** A polygon over fixed anchors is a
 * claim about a shape, and the anchors are somebody else's coaching
 * assumptions; seven bars on a common track are not a claim, they are
 * the values. Each gauge carries its own value and the way it moved
 * against the window before it.
 */
export function ShapeGauges({
  facets,
}: {
  /** The seven facets of the window. */
  facets: ProfileFacet[]
}) {
  if (facets.length === 0) {
    return (
      <p className="text-[13px] text-ink-muted" data-testid="progress-picture-shape-empty">
        {t('progress.wall.shape.empty')}
      </p>
    )
  }

  return (
    <ul className="space-y-2" data-testid="progress-picture-shape">
      {facets.map((facet) => {
        const value = facet.value?.raw ?? null
        const previous = facet.previous?.raw ?? null
        // The gauge is scaled to the larger of the two readings of the
        // same facet: the window and the window before it.
        const scale = Math.max(value ?? 0, previous ?? 0, 1)
        const share = value === null ? 0 : Math.max(0, Math.min(1, value / scale))
        return (
          <li key={facet.key} className="flex items-center gap-3">
            <span className="w-[86px] shrink-0 text-[12px] text-ink-soft">{facet.label}</span>
            <span className="h-2 flex-1 overflow-hidden rounded-full bg-surface-sunken">
              <span
                className="block h-full rounded-full bg-brand-500"
                style={{ width: `${share * 100}%` }}
                data-testid={`progress-picture-shape-${facet.key}`}
              />
            </span>
            <span className="w-[92px] shrink-0 text-right text-[12px] tabular-nums text-ink">
              {renderedText(facet.value)}
            </span>
            <span className="w-[64px] shrink-0 text-right text-[11px] tabular-nums text-ink-muted">
              {facet.movement
                ? t(MOVEMENT[facet.movement] ?? 'progress.rolling.trend.steady.short')
                : ABSENT}
            </span>
          </li>
        )
      })}
    </ul>
  )
}

/* ------------------------------------------------ 9 · the schedule */

/**
 * The schedule, as a picture.
 *
 * **The athlete's week, drawn.** Seven rows, one per day of the week,
 * each row's bar the distance that weekday held; and under it the four
 * times of day the core measures, each with its own bar.
 *
 * **The hour of a run is not in the answer**, so the grid is the seven
 * days and the four times of day the core itself measures — a
 * picture of the shape of the week, and not a second table of numbers.
 */
export function SchedulePicture({
  rolling,
}: {
  /** The rolling answer: the window's weekday and time-of-day splits. */
  rolling: RollingInsight | undefined
}) {
  const weekdays = rolling?.weekday ?? []
  const dayparts = rolling?.daytime ?? []

  if (weekdays.length === 0 && dayparts.length === 0) {
    return (
      <p className="text-[13px] text-ink-muted" data-testid="progress-picture-schedule-empty">
        {t('progress.rolling.noRows')}
      </p>
    )
  }

  const furthest = Math.max(
    ...weekdays.map((day) => day.distance.raw ?? 0),
    ...dayparts.map((part) => part.distance.raw ?? 0),
    1,
  )

  const row = (label: string, km: number, note: string | null, key: string) => (
    <li key={key} className="flex items-center gap-3">
      {/* The label of a time of day carries its window — "Morning
          (04:00–12:00)" — so the column is wide enough for the hours to
          sit beside the word. */}
      <span className="w-[118px] shrink-0 text-[12px] leading-tight text-ink-soft">{label}</span>
      <span className="h-2 flex-1 overflow-hidden rounded-full bg-surface-sunken">
        <span
          className="block h-full rounded-full bg-brand-400"
          style={{ width: `${(km / furthest) * 100}%` }}
          data-testid={`progress-picture-schedule-${key}`}
        />
      </span>
      <span className="w-[74px] shrink-0 text-right text-[12px] tabular-nums text-ink">
        {note}
      </span>
    </li>
  )

  return (
    <div data-testid="progress-picture-schedule">
      <ul className="space-y-1.5">
        {weekdays.map((day) =>
          row(
            day.weekday,
            day.distance.raw ?? 0,
            renderedText(day.distance),
            `day-${day.weekday}`,
          ),
        )}
      </ul>
      <p className="mt-2 text-[11.5px] text-ink-faint">{t('progress.wall.schedule.timeOfDay')}</p>
      <ul className="mt-1 space-y-1.5">
        {dayparts.map((part) =>
          row(part.label, part.distance.raw ?? 0, renderedText(part.distance), `part-${part.key}`),
        )}
      </ul>
    </div>
  )
}

/* ------------------------------------------------- 10 · the habit */

/**
 * The habit, as a picture.
 *
 * **Two drawings.** The streak is a row of day marks — one for each day
 * of the window, filled where the day held a run — and the Eddington
 * number is a ladder of ticks, one per distance, each as tall as the
 * number of days that reached it.
 *
 * A day with no run is an empty mark: absence is drawn.
 */
export function HabitPicture({
  runs,
  from,
  to,
  streak,
  eddington,
}: {
  /** The runs of the window. */
  runs: PictureRun[]
  /** The first day of the window. */
  from: string
  /** The last day of the window. */
  to: string
  /** The current streak, as the answer carries it. */
  streak: { count: number; start_label: string | null } | null
  /** The Eddington number, as the answer carries it. */
  eddington: {
    number: number
    days_needed_for_next: number
    longest_day: { text: string; raw: number | null } | null
  } | null
}) {
  const days = useMemo(() => {
    const list: { date: string; ran: boolean }[] = []
    const total = daysBetween(from, to)
    for (let index = 0; index < total; index += 1) {
      const date = addDays(from, index)
      list.push({ date, ran: runsOfDay(runs, date).length > 0 })
    }
    return list
  }, [runs, from, to])

  // The days that reached each distance: a day reaches k when the runs
  // of that day add up to k.
  const daily = useMemo(() => {
    const totals = new Map<string, number>()
    for (const run of runs) {
      totals.set(run.date, (totals.get(run.date) ?? 0) + (run.km ?? 0))
    }
    return totals
  }, [runs])

  const ladder = useMemo(() => {
    const furthest = Math.max(0, ...daily.values())
    const steps: { km: number; days: number }[] = []
    for (let km = 1; km <= Math.floor(furthest); km += 1) {
      const days = [...daily.values()].filter((value) => value >= km).length
      steps.push({ km, days })
    }
    return steps
  }, [daily])

  const tallest = Math.max(1, ...ladder.map((step) => step.days))

  return (
    <div data-testid="progress-picture-habit">
      {/* The streak: a mark for every day of the window. */}
      <div className="flex flex-wrap gap-[2px]" role="img" aria-label={t('progress.wall.habit.picture')}>
        {days.map((day) => (
          <span
            key={day.date}
            className={`h-[7px] w-[7px] rounded-[1px] ${day.ran ? 'bg-brand-500' : 'bg-surface-sunken'}`}
            data-testid={`progress-picture-habit-day-${day.date}`}
          />
        ))}
      </div>
      <p className="mt-1 text-[12.5px] tabular-nums text-ink-muted" data-testid="progress-picture-habit-streak">
        {streak
          ? t('progress.wall.habit.streak', {
              count: formatCount(streak.count),
              since: streak.start_label ?? '',
            })
          : t('today.streak.none')}
      </p>

      {/* The Eddington: a ladder of ticks, one per distance. */}
      {eddington && ladder.length > 0 ? (
        <>
          <div className="mt-3 flex h-[54px] items-end gap-[2px]">
            {ladder.map((step) => (
              <span
                key={step.km}
                className={`flex-1 rounded-t-[2px] ${
                  step.km === eddington.number ? 'bg-accent-500' : 'bg-brand-200'
                }`}
                style={{ height: `${(step.days / tallest) * 100}%` }}
                data-testid={`progress-picture-habit-ladder-${step.km}`}
              />
            ))}
          </div>
          <p className="mt-1 text-[12.5px] tabular-nums text-ink-muted" data-testid="progress-picture-habit-eddington">
            {t('progress.wall.habit.eddington', { n: formatCount(eddington.number) })}
          </p>
        </>
      ) : (
        <p className="mt-3 text-[13px] text-ink-muted" data-testid="progress-picture-habit-empty">
          {t('progress.eddington.empty')}
        </p>
      )}
    </div>
  )
}

/** The days of a span, from the first to the last. */
export function daysOf(from: string, to: string): string[] {
  const days: string[] = []
  const total = daysBetween(from, to)
  for (let index = 0; index < total; index += 1) days.push(addDays(from, index))
  return days
}

/** A date, a number of days on. */
function addDays(date: string, days: number): string {
  const start = Date.parse(`${date}T00:00:00Z`)
  if (Number.isNaN(start)) return date
  const next = new Date(start + days * 86_400_000)
  return next.toISOString().slice(0, 10)
}
