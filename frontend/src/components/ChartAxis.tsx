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
  /**
   * Whether the quantity on the axis is **dimensionless**.
   *
   * The training load has no unit: the core renders it as the number
   * itself, because a load is not measured in anything — it is the
   * share of a threshold hour. Such a tick carries no unit, and it says
   * so, so that the gate which checks every axis for one can tell a bare
   * number the **core** rendered from a bare number the **app** printed
   * on its own. The second is the mistake; the first is the model.
   */
  dimensionless?: boolean
}

/**
 * One tick.
 *
 * The chart hands it the position and the value; the label comes from
 * the core's own words.
 */
export function AxisTick({ x = 0, y = 0, payload, labels, dimensionless = false }: TickProps) {
  const value = Number(payload?.value)
  const text = labels.get(value) ?? ''
  return (
    <text
      x={x}
      y={y}
      dy={4}
      textAnchor="end"
      fill="var(--chart-axis)"
      fontSize={11}
      data-testid="chart-axis-label"
      data-dimensionless={dimensionless ? 'true' : undefined}
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
export function axisTick(ticks: AxisTick[], options?: { dimensionless?: boolean }) {
  return (
    <AxisTick labels={axisLabels(ticks)} dimensionless={options?.dimensionless ?? false} />
  )
}

/**
 * The tick of a chart's **horizontal** axis.
 *
 * **The first and the last sit flush with the ends of the chart.** A tick
 * in the middle is centred on its day, and a label at the very edge of
 * the chart would hang over it: the last date of a window is half a word
 * long, and it would be cut in half by the edge of the card.
 *
 * The ends are anchored to their own end, and the rest to their middle.
 * The chart hands the tick its index and the number of ticks there are,
 * so the two ends are known without the chart counting anything itself.
 */
export function EdgeTick({
  x = 0,
  y = 0,
  payload,
  index = 0,
  visibleTicksCount = 0,
}: {
  x?: number
  y?: number
  payload?: { value?: string | number }
  index?: number
  visibleTicksCount?: number
}) {
  const anchor = index === 0 ? 'start' : index === visibleTicksCount - 1 ? 'end' : 'middle'
  return (
    <text
      x={x}
      y={y}
      textAnchor={anchor}
      fill="var(--chart-axis)"
      fontSize={11}
      data-testid="chart-date-label"
    >
      {payload?.value === undefined ? '' : String(payload.value)}
    </text>
  )
}
