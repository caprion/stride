/**
 * A chart's vertical axis.
 *
 * The labels are the core's own rendering of the number — "148 bpm",
 * "5:24 /km" — so every label carries its unit, and the app never
 * prints a domain number on its own.
 *
 * **The axis is rendered by the chart itself.** Recharts reads its own
 * children to find the axes, so a `YAxis` inside a wrapper of this app's
 * is not an axis at all — it is a component with no size, and it renders
 * nothing. So this file holds the tick and the labels, and each chart
 * writes its own `<YAxis>`.
 */

import { axisLabels, type AxisTick } from '../lib/chart'

/** What a tick is given by the chart. */
interface TickProps {
  x?: number
  y?: number
  payload?: { value?: number | string }
  /** The core's words, by the chart's number. */
  labels: Map<number, string>
}

/**
 * One tick.
 *
 * The chart hands it the position and the value; the label comes from
 * the core's own words.
 */
export function AxisTick({ x = 0, y = 0, payload, labels }: TickProps) {
  const value = Number(payload?.value)
  const text = labels.get(value) ?? ''
  return (
    <text
      x={x}
      y={y}
      dy={4}
      textAnchor="end"
      fill="#61788D"
      fontSize={11}
      data-testid="chart-axis-label"
    >
      {text}
    </text>
  )
}

/**
 * The tick element for a chart's axis.
 *
 * The ticks are the values the data holds, and the labels are the
 * core's words for them.
 */
export function axisTick(ticks: AxisTick[]) {
  return <AxisTick labels={axisLabels(ticks)} />
}
