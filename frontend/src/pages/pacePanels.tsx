/**
 * The pace, drawn: **the cloud and the bands**.
 *
 * These two panels used to live on a page of their own. They are the
 * pace's two readings — the pace that was held at a heart rate, and
 * where the time went by pace — and the page is gone: the pace is
 * drawn on Progress's **The pace** card, and on the panel behind it.
 *
 * **The page was the third home of the pace.** The run detail has the
 * run's own pace, the wall has the trend, and the pace's own page had
 * these two. A number that lives in three places is three facts to keep
 * in step, and the page was the one a reader had to find.
 *
 * The rules the drawings follow:
 *
 *   - **The pace is read from the runs that are saved**, and a week with
 *     no pace is not a point: the cloud draws what the answer carries.
 *   - **The medians are separate.** A week's pace and a week's heart
 *     rate are two medians of two sets of runs, and the chart says so
 *     rather than pretending they were matched.
 *   - **A band with no value is not a band of zero.** The bands are the
 *     shape of the running, and the time too small for a band is named
 *     as it is.
 */

import {
  CartesianGrid,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { axisTicks } from '../lib/chart'
import { formatDateShort, renderedText } from '../lib/format'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import type { PaceBands, TrainingWeek } from '../lib/types'

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
 * The cloud: one point a week, the pace that was held at that week's
 * median heart rate.
 *
 * A week with no heart rate is not a point: the cloud draws what the
 * answer carries.
 */
export function paceCloud(weeks: TrainingWeek[]): PaceCloudPoint[] {
  return weeks
    .filter((week) => week.pace?.raw != null && week.heart_rate?.raw != null)
    .map((week) => ({
      label: formatDateShort(week.week_start),
      hrValue: week.heart_rate!.raw!,
      paceValue: week.pace!.raw!,
      hrText: week.heart_rate!.text,
      paceText: week.pace!.text,
    }))
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

/**
 * The pace against the heart rate, as a scatter.
 *
 * **One point a week**, and the two axes are the two medians the answer
 * carries. The pace axis is reversed, so a faster pace reads higher —
 * the way a runner reads a pace, not the way a number reads.
 */
export function PaceCloudChart({
  weeks,
  bare = false,
}: {
  /** The weeks of the window. */
  weeks: TrainingWeek[]
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
}) {
  const cloud = paceCloud(weeks)
  if (cloud.length === 0) return null

  const heartRateTicks = axisTicks(
    cloud.map((point) => ({ value: point.hrValue, text: point.hrText })),
  )
  const cloudPaceTicks = axisTicks(
    cloud.map((point) => ({ value: point.paceValue, text: point.paceText })),
  )

  const chart = (
    <>
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
            {/* Reversed: a faster pace reads higher. */}
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
      {/* The medians are separate, and the chart says so: a week with a
          pace and a week with a heart rate are not a matched effort. */}
      <p className="mt-2 text-[12px] text-ink-muted">{t('progress.paceNote')}</p>
    </>
  )

  if (bare) return <div data-testid="pace-cloud">{chart}</div>

  return (
    <section className="card py-3" data-testid="pace-pace-at-hr">
      <h2 className="mb-1 caps">{t('pace.paceAtHr.heading')}</h2>
      <p className="lede">{t('pace.paceAtHr.note')}</p>
      {chart}
    </section>
  )
}

/**
 * Where the time went, by pace.
 *
 * **Every sample of every run, bucketed by the pace it was held at.** The
 * time in the bands too small to draw is named as it is: it is not
 * lost, and it is not folded into a band that did not hold it.
 */
export function PaceBandsPanel({
  bands,
  unavailable,
  bare = false,
}: {
  /** The bands of the window, or null when the window holds no samples. */
  bands: PaceBands | null
  /** Why there are no bands, in the core's own words. */
  unavailable: string | null
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
}) {
  const body = bands ? (
    <>
      <p className="mb-2 text-[12px] text-ink-faint" data-testid="pace-bands-total">
        {t('pace.bands.total')} · {renderedText(bands.total)} · {renderedText(bands.distance)}
      </p>
      <ul className="space-y-1.5" data-testid="pace-bands-rows">
        {bands.rows.map((row) => (
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
        {/* The time in the bands too small to draw: the bands are the
            shape of the running, and the rest of the time is not lost —
            it is here. */}
        {bands.rest && (bands.rest.raw ?? 0) > 0 ? (
          <li data-testid="pace-bands-rest">
            <div className="flex items-baseline justify-between gap-3">
              <span className="text-[13.5px] text-ink-muted">{t('pace.bands.rest')}</span>
              <span className="text-[13.5px] tabular-nums text-ink-muted">
                {renderedText(bands.rest)}
              </span>
            </div>
          </li>
        ) : null}
      </ul>
    </>
  ) : (
    /* No value is the absence of the whole result: the panel says why,
       and shows no band of zero. */
    <p className="text-[13px] text-ink-muted" data-testid="pace-bands-empty">
      {unavailable ?? t('common.error')}
    </p>
  )

  if (bare) return <div data-testid="pace-bands">{body}</div>

  return (
    <section className="card py-3" data-testid="pace-bands-panel">
      <h2 className="mb-1 caps">{t('pace.bands.heading')}</h2>
      <p className="lede">{t('pace.bands.note')}</p>
      {body}
    </section>
  )
}
