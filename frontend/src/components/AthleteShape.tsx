/**
 * The athlete's shape: six axes, two windows, one chart.
 *
 * **The shape is the athlete's own.** Each axis is one of the core's own
 * facets — the volume, the consistency, the sport variety, the density,
 * the duration, the intensity — and each axis is scaled to **the larger of
 * the two windows' values**. So the two shapes are read against each
 * other and not against anybody's idea of a good week: there is no fixed
 * anchor here, and nothing on the chart is a grade.
 *
 * **The drawing is the app's own**: an SVG, with the scale and the path
 * from the app's chart helpers, and the axes named in the core's own
 * words. It is measured before it is painted, the labels shrink before
 * the chart does, and the chart never runs past the edge of its column.
 *
 * **Nothing here is a number the app computed.** Every value on the
 * chart and in the table below it is the core's own rendering — "58.4 km",
 * "148 bpm" — and an axis with no value is left out of the shape rather
 * than drawn as a zero.
 */

import { useMemo, useRef } from 'react'
import { t } from '../i18n'
import { linearScale, polygonPath, textWidth, type Point } from '../lib/chart'
import { useWidth } from '../lib/useWidth'
import type { RenderedValue, RollingInsight } from '../lib/types'

/**
 * The six axes, in the order the chart draws them.
 *
 * The order is the app's; the names are the core's — the answer carries
 * the label of every facet, and the chart draws the label the answer
 * gave.
 */
const AXES = [
  'volume',
  'consistency',
  'variety',
  'density',
  'duration',
  'intensity',
] as const

/** One axis: the core's own name for it, and the two windows' values. */
export interface ShapeAxis {
  /** The facet's key: "volume", "consistency" and so on. */
  key: string
  /** The facet's name, in the core's own words: "Sport variety". */
  label: string
  /** The value in the shorter window. */
  short: RenderedValue | null
  /** The value in the longer window. */
  long: RenderedValue | null
}

/**
 * The axes of the two windows, side by side.
 *
 * The values are the core's: the app reads them and draws them, and it
 * never derives one from another.
 */
export function shapeAxes(
  short: RollingInsight | undefined,
  long: RollingInsight | undefined,
): ShapeAxis[] {
  const facet = (answer: RollingInsight | undefined, key: string) =>
    answer?.profile.find((entry) => entry.key === key)
  return AXES.map((key) => {
    const near = facet(short, key)
    const far = facet(long, key)
    return {
      key,
      label: near?.label ?? far?.label ?? '',
      short: near?.value ?? null,
      long: far?.value ?? null,
    }
  })
}

/** The room between the chart and its labels. */
const GAP = 6

/** The room between the labels and the edge of the drawing. */
const MARGIN = 2

/** The smallest chart the app will draw: below this the labels rule. */
const MIN_RADIUS = 64

/** The font sizes the labels may take, largest first. */
const FONT_SIZES = [11, 10, 9, 8]

/**
 * The angle of an axis, in radians.
 *
 * The first axis is at the top and the axes run clockwise, so the shape
 * is read the way the facets are listed.
 */
function angleOf(index: number, count: number): number {
  return ((-90 + (360 / count) * index) * Math.PI) / 180
}

/**
 * The radius the chart may have, so that its labels fit beside it.
 *
 * The labels may shrink — they are the first thing to give way — but the
 * chart may not overflow: the drawing is measured, and the radius is the
 * one that keeps every label inside the drawing's own edge.
 */
function fittedRadius(size: number, labels: string[], fontSize: number): number {
  const centre = size / 2
  let radius = centre - 1
  for (let index = 0; index < labels.length; index += 1) {
    const angle = angleOf(index, labels.length)
    const cos = Math.cos(angle)
    const sin = Math.sin(angle)
    const width = textWidth(labels[index] ?? '', fontSize)
    let limit: number
    if (cos > 0.05) {
      // The label is set to the right of the axis: it needs its own width.
      limit = (size - MARGIN - centre - width) / cos - GAP
    } else if (cos < -0.05) {
      // And to the left of it.
      limit = (centre - MARGIN - width) / -cos - GAP
    } else if (sin < 0) {
      // The label sits above the chart.
      limit = centre - MARGIN - fontSize - GAP
    } else {
      // And below it.
      limit = size - MARGIN - centre - fontSize - GAP
    }
    radius = Math.min(radius, limit)
  }
  return Math.max(0, radius)
}

/**
 * The chart, its radius and the size of its labels.
 *
 * The labels shrink first: a smaller label is still read, and a chart
 * that runs past the edge of the column is not read at all.
 */
function layout(size: number, labels: string[]): { radius: number; fontSize: number } {
  let chosen = { radius: 0, fontSize: FONT_SIZES[FONT_SIZES.length - 1] as number }
  for (const fontSize of FONT_SIZES) {
    const radius = fittedRadius(size, labels, fontSize)
    chosen = { radius, fontSize }
    if (radius >= MIN_RADIUS) break
  }
  return chosen
}

/** The athlete's shape, drawn. */
export function AthleteShape({
  short,
  long,
}: {
  short: RollingInsight | undefined
  long: RollingInsight | undefined
}) {
  const host = useRef<HTMLDivElement>(null)
  const width = useWidth(host)
  const axes = useMemo(() => shapeAxes(short, long), [short, long])

  /** The drawing is square, and it is never wider than its column. */
  const size = Math.min(width, 340)
  const { radius, fontSize } = layout(size, axes.map((axis) => axis.label))
  const centre = size / 2

  /**
   * The shape of one window.
   *
   * Each axis is scaled to the larger of the two windows' values, so the
   * two shapes are read against each other. An axis with no value is left
   * out: a shape is not drawn through a number that does not exist.
   */
  const pointsOf = (which: 'short' | 'long'): Point[] => {
    const points: Point[] = []
    axes.forEach((axis, index) => {
      const value = which === 'short' ? axis.short : axis.long
      const other = which === 'short' ? axis.long : axis.short
      const largest = Math.max(value?.raw ?? 0, other?.raw ?? 0)
      if (!value || value.raw === null || largest <= 0) return
      const along = linearScale([0, largest], [0, radius])(value.raw)
      const angle = angleOf(index, axes.length)
      points.push({
        x: centre + along * Math.cos(angle),
        y: centre + along * Math.sin(angle),
      })
    })
    return points
  }

  const shortPoints = pointsOf('short')
  const longPoints = pointsOf('long')

  /** The rings: the frame the shapes are read against. */
  const rings = [0.25, 0.5, 0.75, 1].map((share) =>
    axes.map((_, index) => {
      const angle = angleOf(index, axes.length)
      const along = radius * share
      return { x: centre + along * Math.cos(angle), y: centre + along * Math.sin(angle) }
    }),
  )

  return (
    <div ref={host} className="mt-2 w-full" data-testid="progress-shape-chart">
      {size > 0 && radius > 0 ? (
        <svg
          viewBox={`0 0 ${size} ${size}`}
          width={size}
          height={size}
          role="img"
          aria-label={t('progress.shape.alt')}
          data-testid="progress-shape-svg"
        >
          {/* The rings and the spokes: the frame, in the app's own grid. */}
          {rings.map((ring, index) => (
            <path
              key={`ring-${index}`}
              d={polygonPath(ring)}
              fill="none"
              strokeWidth={1}
              style={{ stroke: 'var(--chart-grid)' }}
            />
          ))}
          {axes.map((axis, index) => {
            const angle = angleOf(index, axes.length)
            return (
              <line
                key={`spoke-${axis.key}`}
                x1={centre}
                y1={centre}
                x2={centre + radius * Math.cos(angle)}
                y2={centre + radius * Math.sin(angle)}
                strokeWidth={1}
                style={{ stroke: 'var(--chart-grid)' }}
              />
            )
          })}

          {/*
            The two shapes. The shorter window is filled in first, and
            the longer one is drawn over it: when the two windows hold
            the same shape, the quiet dashed edge of the longer one is
            still there to be seen.
          */}
          {shortPoints.length > 1 ? (
            <path
              d={polygonPath(shortPoints)}
              strokeWidth={2}
              style={{
                fill: 'var(--chart-line)',
                fillOpacity: 0.18,
                stroke: 'var(--chart-line)',
              }}
              data-testid="progress-shape-short"
            />
          ) : null}
          {longPoints.length > 1 ? (
            <path
              d={polygonPath(longPoints)}
              strokeWidth={1.5}
              strokeDasharray="4 3"
              style={{
                fill: 'var(--chart-line-alt)',
                fillOpacity: 0.1,
                stroke: 'var(--chart-line-alt)',
              }}
              data-testid="progress-shape-long"
            />
          ) : null}
          {shortPoints.map((point, index) => (
            <circle
              key={`marker-${index}`}
              cx={point.x}
              cy={point.y}
              r={3}
              strokeWidth={1.5}
              style={{ fill: 'var(--chart-reference)', stroke: 'var(--surface)' }}
            />
          ))}

          {/* The axes' names, in the core's own words. */}
          {axes.map((axis, index) => {
            const angle = angleOf(index, axes.length)
            const cos = Math.cos(angle)
            const sin = Math.sin(angle)
            const along = radius + GAP
            const anchor = cos > 0.05 ? 'start' : cos < -0.05 ? 'end' : 'middle'
            const offset = sin < -0.5 ? 4 : sin > 0.5 ? 12 : 4
            return (
              <text
                key={`label-${axis.key}`}
                x={centre + along * cos}
                y={centre + along * sin + offset}
                textAnchor={anchor}
                fontSize={fontSize}
                style={{ fill: 'var(--ink-muted)' }}
                data-testid="progress-shape-label"
              >
                {axis.label}
              </text>
            )
          })}
        </svg>
      ) : null}
    </div>
  )
}
