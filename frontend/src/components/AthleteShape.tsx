/**
 * The athlete's shape: six axes, three windows, one chart.
 *
 * **The shape is the athlete's own.** Each axis is one of the core's own
 * facets — the volume, the consistency, the sport variety, the density,
 * the duration, the intensity — and each axis is scaled to **the largest
 * of the three windows' values**. So the three shapes are read against
 * each other and not against anybody's idea of a good week: there is no
 * fixed anchor here, and nothing on the chart is a grade.
 *
 * **The three windows are drawn one on the other.** Thirty days, sixty
 * days and ninety days, each in a hue of its own, each a polygon of its
 * own: one picture of the whole shape, instead of two shapes the reader
 * has to hold in their head.
 *
 * **The axes are rates, and the core is where that happens.** The volume
 * is a distance a week and the consistency is the share of the window's
 * days, so a ninety-day window and a thirty-day window can be read
 * against each other at all. The intensity, the duration and the density
 * are averages of the runs inside the window, and the variety is a count
 * of the athlete's sports: they do not grow with the window, and they
 * are the same number in all three.
 *
 * **The drawing is the app's own**: an SVG, with the scale and the path
 * from the app's chart helpers, and the axes named in the core's own
 * words. It is measured before it is painted, the labels shrink before
 * the chart does, and the chart never runs past the edge of its column.
 *
 * **Nothing here is a number the app computed.** Every value on the
 * chart and in the table below it is the core's own rendering — "23.1 km
 * a week", "148 bpm" — and an axis with no value is left out of the shape
 * rather than drawn as a zero.
 */

import { useMemo, useRef } from 'react'
import { t } from '../i18n'
import { linearScale, polygonPath, scaleToLargest, seriesColour, textWidth, type Point } from '../lib/chart'
import { useWidth } from '../lib/useWidth'
import type { ProfileWindow, RenderedValue } from '../lib/types'
import { RADAR_AXES } from './ProfileRadar'

/**
 * One axis of the shape: the core's own name for it, and one window's
 * value.
 */
export interface ShapeAxis {
  /** The facet's key: "volume", "consistency" and so on. */
  key: string
  /** The facet's name, in the core's own words: "Sport variety". */
  label: string
  /** The comparable value: the rate, or the average of the runs. */
  value: RenderedValue | null
}

/**
 * One window of the shape: its six axes, and the days it covers.
 */
export interface ShapeWindow {
  /** How many days the window covers: 30, 60 or 90. */
  days: number
  /** The six axes, in the order they are drawn. */
  axes: ShapeAxis[]
}

/**
 * The six axes of every window, side by side.
 *
 * The values are the core's: the app reads them and draws them, and it
 * never derives one from another.
 */
export function shapeWindows(windows: ProfileWindow[]): ShapeWindow[] {
  return windows.map((window) => ({
    days: window.days,
    axes: RADAR_AXES.map((key) => {
      const facet = window.rolling?.profile.find((entry) => entry.key === key)
      return {
        key,
        label: facet?.label ?? '',
        value: facet?.value ?? null,
      }
    }),
  }))
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

/**
 * The athlete's shape, drawn.
 *
 * `focus` is the window the page is about: it is drawn **last**, on top
 * of the other two, so the shape the numbers below the chart belong to is
 * never buried under the others.
 */
export function AthleteShape({
  windows,
  focus,
}: {
  windows: ShapeWindow[]
  /** The days of the window the page is about. */
  focus: number
}) {
  const host = useRef<HTMLDivElement>(null)
  const width = useWidth(host)

  /** The drawing is square, and it is never wider than its column. */
  const size = Math.min(width, 340)
  const labels = windows[0]?.axes.map((axis) => axis.label) ?? []
  const { radius, fontSize } = layout(size, labels)
  const centre = size / 2

  /**
   * Each axis' own scale: the largest of the three windows' values on
   * that axis is the outside of the chart.
   */
  const scales = useMemo(
    () =>
      (windows[0]?.axes ?? []).map((_axis, index) =>
        scaleToLargest(windows.map((window) => window.axes[index]?.value?.raw ?? null)),
      ),
    [windows],
  )

  /**
   * The shape of one window.
   *
   * An axis with no value is left out: a shape is not drawn through a
   * number that does not exist.
   */
  const pointsOf = (window: ShapeWindow): Point[] => {
    const points: Point[] = []
    window.axes.forEach((axis, index) => {
      const value = axis.value?.raw
      if (value === null || value === undefined || !Number.isFinite(value)) return
      const scale = scales[index]
      if (!scale || scale.max <= 0) return
      const along = linearScale([0, scale.max], [0, radius])(value)
      const angle = angleOf(index, window.axes.length)
      points.push({
        x: centre + along * Math.cos(angle),
        y: centre + along * Math.sin(angle),
      })
    })
    return points
  }

  /**
   * The order the windows are drawn in: the two the page is not about
   * first, and the focused one last.
   */
  const drawn = [
    ...windows.filter((window) => window.days !== focus),
    ...windows.filter((window) => window.days === focus),
  ]

  /** The rings: the frame the shapes are read against. */
  const rings = [0.25, 0.5, 0.75, 1].map((share) =>
    (windows[0]?.axes ?? []).map((_axis, index) => {
      const angle = angleOf(index, windows[0]?.axes.length ?? 0)
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
          {(windows[0]?.axes ?? []).map((axis, index) => {
            const angle = angleOf(index, windows[0]?.axes.length ?? 0)
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
            The three shapes, one on the other.

            **The fills are drawn first, all of them, and the lines are
            drawn over them.** A fill that covered another polygon's
            edge would hide where that polygon is, and the reader has to
            be able to see each of the three. Each fill is a light wash
            of the polygon's own hue, and each line is thin and in that
            same hue.
          */}
          <g data-testid="progress-shape-fills">
            {drawn.map((window) => {
              const points = pointsOf(window)
              if (points.length < 2) return null
              return (
                <path
                  key={`fill-${window.days}`}
                  d={polygonPath(points)}
                  fill={seriesColour(windows.indexOf(window))}
                  fillOpacity={0.1}
                  stroke="none"
                  data-testid={`progress-shape-fill-${window.days}`}
                />
              )
            })}
          </g>
          {drawn.map((window) => {
            const points = pointsOf(window)
            if (points.length < 2) return null
            const colour = seriesColour(windows.indexOf(window))
            const focused = window.days === focus
            return (
              <path
                key={`window-${window.days}`}
                d={polygonPath(points)}
                fill="none"
                strokeWidth={focused ? 2.5 : 2}
                style={{ stroke: colour }}
                data-testid={`progress-shape-window-${window.days}`}
              />
            )
          })}

          {/* The focused window's own points, so its shape can be read. */}
          {drawn
            .filter((window) => window.days === focus)
            .flatMap((window) => pointsOf(window))
            .map((point, index) => (
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
          {(windows[0]?.axes ?? []).map((axis, index) => {
            const angle = angleOf(index, windows[0]?.axes.length ?? 0)
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
