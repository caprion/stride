/**
 * The labels of a chart's vertical axis.
 *
 * The core renders the domain's numbers; the chart draws them. So an
 * axis's ticks are values the api sent, and their labels are the core's
 * own words — "148 bpm", "5:24 /km", "38.6 km" — never a bare number
 * the app printed on its own.
 *
 * The pace axis is the reason this file exists: the pace's raw number is
 * seconds per kilometre, and a chart that printed it would be showing a
 * domain number without its unit, which is the contract's §8 saying no.
 */

import { eachWeekOfInterval, format, parseISO, startOfWeek, subWeeks } from 'date-fns'

/** Calendar weeks, including dates for which the store has no observations. */
export function calendarWeekStarts(date: string, count: number): string[] {
  const end = startOfWeek(parseISO(date.slice(0, 10)), { weekStartsOn: 1 })
  return eachWeekOfInterval({ start: subWeeks(end, count - 1), end }, { weekStartsOn: 1 }).map(
    (week) => format(week, 'yyyy-MM-dd'),
  )
}

/** One tick of a chart's axis. */
export interface AxisTick {
  /** The chart's own number for the tick. */
  value: number
  /** The core's words for it: "148 bpm". */
  text: string
}

/**
 * The ticks of a chart's vertical axis.
 *
 * The values are the ones the data holds — so there is nothing on the
 * axis that the core did not write — and at most `count` of them are
 * drawn, so the labels never crowd each other.
 */
export function axisTicks(
  points: { value: number | null; text: string | null }[],
  count = 3,
): AxisTick[] {
  const unique = new Map<number, string>()
  for (const point of points) {
    if (point.value === null || point.text === null) continue
    if (!Number.isFinite(point.value)) continue
    if (!unique.has(point.value)) unique.set(point.value, point.text)
  }

  const sorted = [...unique.entries()]
    .sort((left, right) => left[0] - right[0])
    .map(([value, text]) => ({ value, text }))

  if (sorted.length <= count) return sorted

  // Evenly spread, and the ends are always there: the reader sees the
  // range the chart covers.
  const step = (sorted.length - 1) / (count - 1)
  const picked: AxisTick[] = []
  for (let index = 0; index < count; index += 1) {
    const tick = sorted[Math.round(index * step)] as AxisTick
    if (!picked.some((entry) => entry.value === tick.value)) picked.push(tick)
  }
  return picked
}

/** The labels of an axis, by the chart's own number. */
export function axisLabels(ticks: AxisTick[]): Map<number, string> {
  return new Map(ticks.map((tick) => [tick.value, tick.text]))
}

/** A point in a chart's own coordinates. */
export interface Point {
  x: number
  y: number
}

/**
 * A scale that puts the **largest of the values** on the outside.
 *
 * The radar draws two windows on one chart, and each axis is scaled to the
 * larger of the two values on that axis: the axis is the athlete's own
 * scale, not a fixed anchor anybody else chose. A scale that says what a
 * value means is the reader's business, and this one only says where the
 * bigger of two numbers sits.
 */
export interface Scale {
  /** The largest of the values the scale was given. */
  max: number
  /** Where a value sits on the scale: 0 at the centre, 1 at the edge. */
  at(value: number): number
}

/**
 * The scale that puts the largest of the values on the outside.
 *
 * The values that are not there are not zero: a window with no value on
 * an axis leaves that axis at the centre, and the chart says so.
 */
export function scaleToLargest(values: (number | null | undefined)[]): Scale {
  const numbers = values.filter(
    (value): value is number => typeof value === 'number' && Number.isFinite(value),
  )
  const max = numbers.length > 0 ? Math.max(...numbers) : 0
  return {
    max,
    at(value: number): number {
      if (!Number.isFinite(value) || max <= 0) return 0
      return Math.max(0, Math.min(1, value / max))
    },
  }
}

/** Two decimals: the path of a chart is read by a browser, not by a person. */
function round(value: number): number {
  return Math.round(value * 100) / 100
}

/**
 * The path of a closed polygon.
 *
 * The radar's two windows are polygons, and the browser draws the path
 * this returns.
 */
export function polygonPath(points: Point[]): string {
  if (points.length === 0) return ''
  const commands = points.map(
    (point, index) => `${index === 0 ? 'M' : 'L'}${round(point.x)},${round(point.y)}`,
  )
  return `${commands.join(' ')} Z`
}

/**
 * A point on a circle: the tip of an axis, or the end of a spoke.
 *
 * The first axis points up, and the axes go round clockwise — the way a
 * clock does, so the reader finds the same axis in the same place on
 * every chart.
 */
export function radialPoint(
  centreX: number,
  centreY: number,
  radius: number,
  index: number,
  count: number,
): Point {
  const angle = (Math.PI * 2 * index) / count - Math.PI / 2
  return {
    x: centreX + Math.cos(angle) * radius,
    y: centreY + Math.sin(angle) * radius,
  }
}

/**
 * The range a scale covers: the middle of the values.
 *
 * A run carries a few readings that are far outside the rest — a walk
 * break's pace of nineteen minutes a kilometre, a spike in the heart rate
 * at a crossing — and a scale that reaches them squashes the whole series
 * into the top of the chart. So the scale covers the middle ninety-six per
 * cent of the values, and the readings outside it are drawn past the edge
 * of the drawing, which is what a chart does with them.
 *
 * The values outside are not thrown away and not rounded: they are still
 * in the series, and the core's own words still say what they are.
 */
export function robustRange(values: number[]): [number, number] {
  if (values.length === 0) return [0, 1]
  const sorted = [...values].sort((left, right) => left - right)
  const low = sorted[Math.floor(sorted.length * 0.02)] as number
  const high = sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * 0.98))] as number
  return [low, high]
}
