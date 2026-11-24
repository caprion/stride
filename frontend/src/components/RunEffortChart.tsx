/**
 * The run's one chart.
 *
 * **One run is one picture.** The pace and the heart rate are drawn over
 * the run's own elapsed time, with the athlete's own zone bands behind
 * the heart rate, and **one cursor** crosses both. The altitude is on
 * the same time axis but keeps **its own scale and its own metres
 * label**, and it is off by default: the ground is scenery, not an
 * effort signal.
 *
 * The rules the drawing follows:
 *
 *   - **Time is the x-axis.** Every observation the core makes about a
 *     run is a difference of times, and a stopped stretch covers no
 *     distance but is real elapsed time — on a time axis it is a gap,
 *     which is what it is.
 *   - **The bands stay behind the heart rate.** The athlete's own
 *     configuration, drawn on their own data.
 *   - **The two effort signals share the band and are read apart.** The
 *     pace and the heart rate have different units and different
 *     meanings, so they have their own axes and their own colours, and
 *     the chart says which is which.
 *   - **The altitude never shares the effort band.** Metres and seconds
 *     per kilometre would flatten one curve into a straight line and
 *     imply a comparison the data does not support.
 *   - **The numbers are the core's.** Every label and every tooltip is
 *     the api's own rendering; the app draws and formats nothing.
 */

import { useState } from 'react'
import {
  Area,
  AreaChart,
  CartesianGrid,
  ComposedChart,
  Line,
  ReferenceArea,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { t } from '../i18n'
import { axisTicks } from '../lib/chart'
import { axisTick } from './ChartAxis'
import type { ActivityDetail, ZoneBand } from '../lib/types'

/** The minute of the run, as the chart's x-axis writes it. */
function minuteTick(value: number): string {
  const minutes = Math.round(value)
  return `${minutes}′`
}

/**
 * The run's chart.
 *
 * The series are the run's own samples, thinned by the api, and the
 * bands are the athlete's own zones.
 */
export function RunEffortChart({
  samples,
  zones,
  runId,
}: {
  /** The run's samples, as the answer carries them. */
  samples: ActivityDetail['samples']
  /** The athlete's own zones, as the answer carries them. */
  zones: ZoneBand[]
  /** The run's own id: the charts that share a cursor share this. */
  runId: number
}) {
  const [paceOn, setPaceOn] = useState(true)
  const [altitudeOn, setAltitudeOn] = useState(false)

  const rows = samples.map((sample) => ({
    // The chart's own scale: minutes into the run.
    minute: sample.offset_s / 60,
    hr: sample.heart_rate?.raw ?? null,
    hrText: sample.heart_rate?.text ?? null,
    pace: sample.pace?.raw ?? null,
    paceText: sample.pace?.text ?? null,
    altitude: sample.altitude_m?.raw ?? null,
    altitudeText: sample.altitude_m?.text ?? null,
    kind: sample.kind,
  }))

  /**
   * The axes' labels.
   *
   * They are the core's own words — "148 bpm", "5:24 /km", "40 m" — so
   * every label carries its unit. A bare number on an axis would be the
   * app printing a domain number on its own.
   */
  const heartRateTicks = axisTicks(
    samples.map((sample) => ({ value: sample.heart_rate?.raw ?? null, text: sample.heart_rate?.text ?? null })),
  )
  const paceTicks = axisTicks(
    samples.map((sample) => ({ value: sample.pace?.raw ?? null, text: sample.pace?.text ?? null })),
  )
  const altitudeTicks = axisTicks(
    samples.map((sample) => ({ value: sample.altitude_m?.raw ?? null, text: sample.altitude_m?.text ?? null })),
  )

  const bands = zones.filter((band) => band.from_bpm !== null && band.to_bpm !== null)

  return (
    <div data-testid="activity-chart">
      {/*
        The toggles: the reader chooses which of the readings the chart
        carries. The heart rate is always there — it is the run's effort,
        and the bands behind it need it.
      */}
      <div className="mb-2 flex flex-wrap gap-2" data-testid="activity-chart-toggles">
        <button
          type="button"
          onClick={() => setPaceOn((on) => !on)}
          aria-pressed={paceOn}
          className={`tap rounded-md border px-2.5 py-0.5 text-[12px] ${
            paceOn ? 'border-brand-600 bg-brand-50 text-brand-700' : 'border-edge text-ink-muted'
          }`}
          data-testid="activity-chart-toggle-pace"
        >
          {t('vital.pace')}
        </button>
        <button
          type="button"
          onClick={() => setAltitudeOn((on) => !on)}
          aria-pressed={altitudeOn}
          className={`tap rounded-md border px-2.5 py-0.5 text-[12px] ${
            altitudeOn ? 'border-brand-600 bg-brand-50 text-brand-700' : 'border-edge text-ink-muted'
          }`}
          data-testid="activity-chart-toggle-altitude"
        >
          {t('activity.elevation.heading')}
        </button>
      </div>

      {/*
        The effort band: the pace and the heart rate over the elapsed
        time, with the athlete's own zones behind the heart rate.
      */}
      <div style={{ width: '100%', height: 190 }}>
        <ResponsiveContainer width="100%" height="100%">
          <ComposedChart data={rows} syncId={`effort-${runId}`} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="minute"
              type="number"
              domain={[0, 'dataMax']}
              tickCount={5}
              tickFormatter={minuteTick}
              tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
            />
            <YAxis
              yAxisId="hr"
              ticks={heartRateTicks.map((tick) => tick.value)}
              tick={axisTick(heartRateTicks)}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={['dataMin - 5', 'dataMax + 5']}
            />
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
            {/*
              The athlete's own zones: the bands the core computed, on
              the numbers the core rendered the words from. They are
              bands of the athlete's history, not a judgement — nothing
              here says what the run should have been.
            */}
            {bands.map((band) => (
              <ReferenceArea
                key={band.label}
                yAxisId="hr"
                y1={band.from_bpm ?? 0}
                y2={band.to_bpm ?? 0}
                fill="var(--chart-band)"
                fillOpacity={0.35}
                stroke="var(--chart-line-alt)"
                strokeOpacity={0.8}
              />
            ))}
            <Tooltip
              formatter={(_value, _name, item) => {
                const payload = item?.payload as
                  { hrText?: string | null; paceText?: string | null } | undefined
                // The tooltip shows the core's words, not the chart's
                // numbers.
                return payload?.hrText ?? payload?.paceText ?? ''
              }}
            />
            <Line
              yAxisId="hr"
              type="monotone"
              name={t('vital.heartRate')}
              dataKey="hr"
              stroke="var(--chart-line)"
              strokeWidth={2}
              dot={false}
              data-testid="activity-chart-heart-rate"
            />
            {paceOn ? (
              <Line
                yAxisId="pace"
                type="monotone"
                name={t('vital.pace')}
                dataKey="pace"
                stroke="var(--chart-reference)"
                strokeWidth={2}
                dot={false}
                data-testid="activity-chart-pace"
              />
            ) : null}
          </ComposedChart>
        </ResponsiveContainer>
      </div>

      {/*
        The ground: the altitude, on its own scale and its own label, and
        sharing the run's cursor. It is off by default — the ground under
        the run is scenery, and the reader asks for it.
      */}
      {altitudeOn ? (
        <div className="mt-3" data-testid="activity-chart-altitude">
          <p className="mb-1 text-[11.5px] text-ink-muted">{t('activity.elevation.heading')}</p>
          <div style={{ width: '100%', height: 90 }}>
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={rows} syncId={`effort-${runId}`} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
                <XAxis
                  dataKey="minute"
                  type="number"
                  domain={[0, 'dataMax']}
                  tickCount={5}
                  tickFormatter={minuteTick}
                  tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
                />
                <YAxis
                  ticks={altitudeTicks.map((tick) => tick.value)}
                  tick={axisTick(altitudeTicks)}
                  width={64}
                  tickLine={false}
                  axisLine={false}
                  domain={['dataMin - 5', 'dataMax + 5']}
                />
                <Tooltip
                  formatter={(_value, _name, item) => {
                    const payload = item?.payload as { altitudeText?: string | null } | undefined
                    return payload?.altitudeText ?? ''
                  }}
                />
                <Area
                  type="monotone"
                  name={t('activity.elevation.heading')}
                  dataKey="altitude"
                  stroke="var(--chart-line-alt)"
                  fill="var(--chart-band)"
                  isAnimationActive={false}
                  data-testid="activity-chart-altitude-area"
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>
      ) : null}
    </div>
  )
}
