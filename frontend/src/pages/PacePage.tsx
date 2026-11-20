/**
 * The pace.
 *
 * Three ways of reading the same window: the training week by week, the
 * pace at a heart rate, and where the time went by pace.
 *
 * **The page says what happened.** The pace trend is a line, the bands are
 * a distribution, and there is no target, no "too slow" and no verdict
 * anywhere on it.
 *
 * The window is the Progress window, and the same range selector chooses
 * it: a number is only a fact against a stated period, so the card names
 * the window it describes.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import {
  Bar,
  CartesianGrid,
  ComposedChart,
  Line,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { axisTicks, calendarWeekStarts } from '../lib/chart'
import { formatCount, formatDateShort, renderedText } from '../lib/format'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { PageHeader } from '../ui/layout/PageHeader'
import { useStaleness } from '../lib/staleness'
import type { TrainingWeek } from '../lib/types'

/**
 * The ranges the page can show.
 *
 * The same ranges the Progress screen offers, so the two screens are read
 * the same way.
 */
const RANGES = [4, 8, 12, 26, 52] as const

/** One week of the chart: the distance, the long run inside it, the pace. */
export interface TrainingChartWeek {
  /** The week, as the axis writes it. */
  label: string
  /** The week's distance, in kilometres. */
  km: number | null
  /** The long run's distance, in kilometres — the bar inside the bar. */
  longestKm: number | null
  /** The rest of the week, in kilometres. */
  restKm: number | null
  /** The week's pace, in seconds per kilometre. */
  paceValue: number | null
  /** The core's own words for the numbers. */
  distanceText: string | null
  longestText: string | null
  paceText: string | null
}

/** One point of the cloud: a week, its pace and its heart rate. */
export interface PaceCloudPoint {
  /** The week, as the tooltip writes it. */
  label: string
  /** The week's median heart rate. */
  hrValue: number
  /** The week's median pace, in seconds per kilometre. */
  paceValue: number
  /** The core's own words for the numbers. */
  hrText: string
  paceText: string
}

/**
 * The weeks of the window, as the training chart draws them.
 *
 * **The long run is inside the bar:** the week's distance is the whole bar
 * and the long run is the part of it that is drawn in the other colour, so
 * the two are never added up twice. A week with no run has no bar at all —
 * a gap in the chart, not a bar of zero.
 */
export function trainingWeeks(
  weeks: TrainingWeek[],
  calendar: string[],
): TrainingChartWeek[] {
  return calendar.map((weekStart) => {
    const week = weeks.find((entry) => entry.week_start === weekStart)
    const km = week?.distance.raw == null ? null : week.distance.raw / 1000
    const longestKm = week?.longest?.distance.raw == null ? null : week.longest.distance.raw / 1000
    const restKm =
      km === null || longestKm === null ? null : Math.max(0, km - longestKm)
    return {
      label: formatDateShort(weekStart),
      km,
      longestKm,
      restKm,
      paceValue: week?.pace?.raw ?? null,
      distanceText: week?.distance.text ?? null,
      longestText: week?.longest?.distance.text ?? null,
      paceText: week?.pace?.text ?? null,
    }
  })
}

/**
 * The cloud: one point a week, the pace that was held at that week's
 * median heart rate.
 *
 * A week with no heart rate is not a point: the cloud draws what the
 * answer carries.
 */
export function paceCloud(weeks: TrainingWeek[]): PaceCloudPoint[] {
  return weeks
    .filter(
      (week) => week.pace?.raw != null && week.heart_rate?.raw != null,
    )
    .map((week) => ({
      label: formatDateShort(week.week_start),
      hrValue: week.heart_rate!.raw!,
      paceValue: week.pace!.raw!,
      hrText: week.heart_rate!.text,
      paceText: week.pace!.text,
    }))
}

/** A chart card with a fixed, legible height. */
function ChartCard({
  title,
  note,
  children,
  testId,
}: {
  title: string
  note: string
  children: React.ReactNode
  testId: string
}) {
  return (
    <section className="card px-4 py-3" data-testid={testId}>
      <h2 className="mb-1 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {title}
      </h2>
      <p className="mb-2 text-[13px] text-ink-muted">{note}</p>
      {children}
    </section>
  )
}

/** The legend under the training chart. */
function Legend({ swatches }: { swatches: { colour: string; label: string }[] }) {
  return (
    <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1">
      {swatches.map((swatch) => (
        <span
          key={swatch.label}
          className="flex items-center gap-1.5 text-[12px] text-ink-muted"
        >
          <i
            className="inline-block h-2 w-3 rounded-[2px]"
            style={{ background: swatch.colour }}
            aria-hidden="true"
          />
          {swatch.label}
        </span>
      ))}
    </div>
  )
}

/** The tooltip of the training chart: the week's own numbers. */
function WeekTooltip({
  active,
  payload,
}: {
  active?: boolean
  payload?: { payload: TrainingChartWeek }[]
}) {
  if (!active || !payload || payload.length === 0) return null
  const week = payload[0]!.payload
  return (
    <div className="rounded-md border border-edge bg-surface px-2 py-1 text-[12px] shadow-card">
      <p className="font-semibold text-ink">{week.label}</p>
      {week.distanceText ? <p className="text-ink-muted">{week.distanceText}</p> : null}
      {week.longestText ? (
        <p className="text-ink-muted">
          {t('pace.weeks.legend.longest')} · {week.longestText}
        </p>
      ) : null}
      {week.paceText ? <p className="text-ink-muted">{week.paceText}</p> : null}
    </div>
  )
}

/** The tooltip of the cloud: the week's own numbers. */
function CloudTooltip({
  active,
  payload,
}: {
  active?: boolean
  payload?: { payload: PaceCloudPoint }[]
}) {
  if (!active || !payload || payload.length === 0) return null
  const point = payload[0]!.payload
  return (
    <div className="rounded-md border border-edge bg-surface px-2 py-1 text-[12px] shadow-card">
      <p className="font-semibold text-ink">{point.label}</p>
      <p className="text-ink-muted">{point.paceText}</p>
      <p className="text-ink-muted">{point.hrText}</p>
    </div>
  )
}

/** The pace page. */
export function PacePage() {
  const [range, setRange] = useState<number>(12)
  const pace = useQuery({
    queryKey: ['insights-pace', range],
    queryFn: () => api.insightsPace(range),
  })
  const stale = useStaleness()
  const data = pace.data

  if (pace.isError && !data) {
    return (
      <div>
        <PageHeader title={t('pace.heading')} />
        <section className="card px-4 py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void pace.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      </div>
    )
  }

  if (!data) {
    return (
      <div>
        <PageHeader title={t('pace.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  // Nothing to draw: the page says so and invents no numbers.
  if (data.weeks.length === 0) {
    return (
      <div>
        <PageHeader title={t('pace.heading')} />
        <div className="card px-4 py-8 text-center" data-testid="pace-empty">
          <p className="text-[15px] font-semibold text-ink">{t('pace.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('pace.empty.body')}</p>
        </div>
      </div>
    )
  }

  // The calendar weeks of the window: a week with no run is a gap in the
  // chart, not a missing week.
  const calendar = calendarWeekStarts(data.date, range)
  const weeks = trainingWeeks(data.weeks, calendar)
  const cloud = paceCloud(data.weeks)

  /**
   * The axes' labels.
   *
   * They are the core's own words — "38.6 km", "5:24 /km", "156 bpm" — so
   * every label carries its unit.
   */
  const distanceTicks = axisTicks(
    weeks.map((week) => ({ value: week.km, text: week.distanceText })),
  )
  const paceTicks = axisTicks(
    weeks.map((week) => ({ value: week.paceValue, text: week.paceText })),
  )
  const heartRateTicks = axisTicks(
    cloud.map((point) => ({ value: point.hrValue, text: point.hrText })),
  )
  const cloudPaceTicks = axisTicks(
    cloud.map((point) => ({ value: point.paceValue, text: point.paceText })),
  )

  return (
    <div>
      <PageHeader
        title={t('pace.heading')}
        meta={
          <>
            {t('pace.question')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />

      {/* --------------------------------------------- the range selector */}
      <div
        className="mb-3 flex flex-wrap items-center gap-2"
        data-testid="pace-range"
      >
        <span className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('progress.range.label')}
        </span>
        <div className="flex flex-wrap gap-1">
          {RANGES.map((weeks) => (
            <button
              key={weeks}
              type="button"
              onClick={() => {
                setRange(weeks)
              }}
              aria-pressed={range === weeks}
              data-testid={`pace-range-${weeks}`}
              className={`tap rounded-md px-2.5 text-[12.5px] font-semibold ${
                range === weeks ? 'bg-brand-50 text-brand-700' : 'text-ink-muted'
              }`}
            >
              {t('progress.range.weeks', { n: weeks })}
            </button>
          ))}
        </div>
      </div>

      {/* ------------------------------------------------ the window line */}
      <p
        className="mb-3 text-[12px] text-ink-faint"
        data-testid="pace-window"
      >
        {t('pace.window', {
          from: data.window.from_label,
          to: data.window.to_label,
        })}
      </p>

      {/* ------------------------------------------ the training, weekly */}
      <ChartCard
        title={t('pace.weeks.heading')}
        note={t('pace.weeks.note')}
        testId="pace-weeks"
      >
        <div style={{ width: '100%', height: 200 }}>
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart
              data={weeks}
              margin={{ top: 4, right: 52, bottom: 0, left: -18 }}
            >
              <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
              <XAxis
                dataKey="label"
                tick={{ fontSize: 11, fill: 'var(--ink-muted)' }}
                interval="preserveStartEnd"
              />
              <YAxis
                yAxisId="distance"
                ticks={distanceTicks.map((tick) => tick.value)}
                tick={axisTick(distanceTicks)}
                width={64}
                tickLine={false}
                axisLine={false}
                domain={[0, 'auto']}
              />
              {/* The pace on the other axis, and reversed: a faster pace is
                  a smaller number, and it reads higher on the chart. */}
              <YAxis
                yAxisId="pace"
                orientation="right"
                ticks={paceTicks.map((tick) => tick.value)}
                tick={axisTick(paceTicks)}
                width={56}
                tickLine={false}
                axisLine={false}
                reversed
                domain={['dataMin - 10', 'dataMax + 10']}
              />
              <Tooltip content={<WeekTooltip />} cursor={{ fill: 'var(--brand-50)' }} />
              {/* The long run is the bar inside the bar: the week's
                  distance is the whole, and the long run is the part of it
                  that is drawn in the other colour. */}
              <Bar
                yAxisId="distance"
                dataKey="longestKm"
                stackId="distance"
                fill="var(--brand-500)"
                isAnimationActive={false}
              />
              <Bar
                yAxisId="distance"
                dataKey="restKm"
                stackId="distance"
                fill="var(--brand-300)"
                isAnimationActive={false}
              />
              <Line
                yAxisId="pace"
                type="monotone"
                dataKey="paceValue"
                stroke="var(--accent-700)"
                strokeWidth={2}
                dot={false}
                connectNulls={false}
                isAnimationActive={false}
              />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
        <Legend
          swatches={[
            { colour: 'var(--brand-300)', label: t('pace.weeks.legend.distance') },
            { colour: 'var(--brand-500)', label: t('pace.weeks.legend.longest') },
            { colour: 'var(--accent-700)', label: t('pace.weeks.legend.pace') },
          ]}
        />
      </ChartCard>

      {/* ------------------------------------------- the pace at a heart rate */}
      <ChartCard
        title={t('pace.paceAtHr.heading')}
        note={t('pace.paceAtHr.note')}
        testId="pace-pace-at-hr"
      >
        <div style={{ width: '100%', height: 200 }} data-points={cloud.length}>
          <ResponsiveContainer width="100%" height="100%">
            <ScatterChart margin={{ top: 8, right: 16, bottom: 8, left: -6 }}>
              <CartesianGrid stroke="var(--chart-grid)" />
              <XAxis
                type="number"
                dataKey="hrValue"
                ticks={heartRateTicks.map((tick) => tick.value)}
                tick={axisTick(heartRateTicks)}
                tickLine={false}
                axisLine={false}
                domain={['dataMin - 4', 'dataMax + 4']}
              />
              {/* Reversed again: a faster pace reads higher. */}
              <YAxis
                type="number"
                dataKey="paceValue"
                ticks={cloudPaceTicks.map((tick) => tick.value)}
                tick={axisTick(cloudPaceTicks)}
                width={64}
                tickLine={false}
                axisLine={false}
                reversed
                domain={['dataMin - 10', 'dataMax + 10']}
              />
              <Tooltip content={<CloudTooltip />} cursor={{ strokeDasharray: '3 3' }} />
              <Scatter data={cloud} fill="var(--accent-600)" isAnimationActive={false} />
            </ScatterChart>
          </ResponsiveContainer>
        </div>
        {/* The medians are separate, and the page says so: a week with a
            pace and a week with a heart rate are not a matched effort. */}
        <p className="mt-2 text-[12px] text-ink-muted">{t('progress.paceNote')}</p>
      </ChartCard>

      {/* ------------------------------------------- where the time went */}
      <ChartCard
        title={t('pace.bands.heading')}
        note={t('pace.bands.note')}
        testId="pace-bands"
      >
        {data.bands ? (
          <>
            <p className="mb-2 text-[12px] text-ink-faint" data-testid="pace-bands-total">
              {t('pace.bands.total')} · {renderedText(data.bands.total)} ·{' '}
              {renderedText(data.bands.distance)}
            </p>
            <ul className="space-y-1.5" data-testid="pace-bands-rows">
              {data.bands.rows.map((row) => (
                <li key={`${row.from}-${row.to}`}>
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="text-[13.5px] text-ink-soft">{row.label}</span>
                    <span className="flex items-baseline gap-2 text-[13.5px] tabular-nums text-ink">
                      <span>{renderedText(row.time)}</span>
                      <span className="text-ink-muted">{renderedText(row.share)}</span>
                    </span>
                  </div>
                  {/* The bar is the band's own share of the window's time. */}
                  <div className="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-brand-50">
                    <div
                      className="h-full rounded-full bg-brand-600"
                      style={{
                        width: `${Math.max(0, Math.min(100, row.share?.raw ?? 0))}%`,
                      }}
                      data-testid={`pace-bands-bar-${row.from}`}
                    />
                  </div>
                </li>
              ))}
            </ul>
          </>
        ) : (
          /* No value is the absence of the whole result: the page says why,
             and shows no band of zero. */
          <p className="text-[13px] text-ink-muted" data-testid="pace-bands-empty">
            {data.unavailable ?? t('common.error')}
          </p>
        )}
      </ChartCard>

      {/* The window's own line: how much time it holds, read by pace. */}
      <p className="mt-4 text-[12.5px] text-ink-faint" data-testid="pace-summary">
        {data.bands
          ? t('pace.summary', {
              weeks: formatCount(data.window.weeks),
              time: renderedText(data.bands.total),
            })
          : t('progress.range.weeks', { n: data.window.weeks })}
      </p>
    </div>
  )
}
