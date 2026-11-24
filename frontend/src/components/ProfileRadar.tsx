/**
 * The athlete's profile, as a radar.
 *
 * Three windows, one chart: the last thirty days, the last sixty and the
 * last ninety, each on its own polygon, each axis scaled to the
 * **largest of the three values on that axis**. So the chart says how
 * the windows differ in shape, and it says nothing about what any of the
 * numbers means — the scale is the athlete's own, not an anchor
 * somebody else chose.
 *
 * **The values are the router's.** The facets come from
 * `/insights/rolling`, and every number here is the core's own rendering
 * of it. The chart draws; it computes nothing.
 *
 * The drawing is SVG, made by the app's own chart helpers (`lib/chart.ts`):
 * the scale that puts the largest of the values on the outside, the path
 * of a polygon and the point on a circle. It is sized by its own viewBox, so
 * it shrinks with the card and never overflows it.
 */

import {
  polygonPath,
  radialPoint,
  scaleToLargest,
  seriesColour,
  type Point,
} from '../lib/chart'
import { t } from '../i18n'

/** One axis of the radar. */
export interface RadarAxis {
  /** The facet's key: "volume". */
  key: string
  /** The facet's name, in the core's words: "Sport variety". */
  label: string
  /** The comparable value in the window: the rate, or the average. Null when there is none. */
  value: number | null
  /** The value as the core renders it: "23.1 km a week". */
  text: string | null
}

/** One window of the chart. */
export interface RadarSeries {
  /** The series' own key: "30". */
  key: string
  /** The window's name, in the app's words: "last 30 days". */
  label: string
  /** The six axes, in the order they are drawn. */
  axes: RadarAxis[]
}

/**
 * The axes of the radar, in the order they are drawn.
 *
 * Six of the seven facets the answer carries: the days of the week are
 * not an axis, because a week has no shape.
 */
export const RADAR_AXES = [
  'volume',
  'consistency',
  'variety',
  'density',
  'duration',
  'intensity',
] as const

/** The box the drawing is made in. */
const VIEW_WIDTH = 280
const VIEW_HEIGHT = 240
const CENTRE_X = 140
const CENTRE_Y = 120
/** How far the outer ring is from the centre. */
const RADIUS = 70
/** How far the axis names sit from the centre. */
const LABEL_RADIUS = 84

/** The rings the axes are drawn over: a third, two thirds and all of it. */
const RINGS = [1 / 3, 2 / 3, 1]

/**
 * Where an axis' name sits, and which way it is written.
 *
 * The names on the right are written away from the centre and the names
 * on the left are written towards it, so no name runs over the chart.
 */
function labelAnchor(
  index: number,
  count: number,
): {
  x: number
  y: number
  anchor: 'start' | 'middle' | 'end'
} {
  const point = radialPoint(CENTRE_X, CENTRE_Y, LABEL_RADIUS, index, count)
  const angle = (Math.PI * 2 * index) / count - Math.PI / 2
  const cos = Math.cos(angle)
  if (cos > 0.2) return { x: point.x, y: point.y, anchor: 'start' }
  if (cos < -0.2) return { x: point.x, y: point.y, anchor: 'end' }
  return { x: point.x, y: point.y, anchor: 'middle' }
}

export function ProfileRadar({
  series,
  focus,
}: {
  series: RadarSeries[]
  /** The key of the window the page is about: it is drawn last. */
  focus?: string
}) {
  if (series.length === 0) return null

  const count = series[0]?.axes.length ?? 0
  if (count < 3) return null

  /**
   * Each axis' own scale: the largest of the windows' values on that
   * axis is the outside of the chart, and the others sit inside it.
   */
  const scales = series[0]!.axes.map((_axis, index) =>
    scaleToLargest(series.map((entry) => entry.axes[index]?.value ?? null)),
  )

  /** The points of one window's polygon. */
  const pointsOf = (entry: RadarSeries): Point[] =>
    entry.axes.map((axis, index) =>
      radialPoint(
        CENTRE_X,
        CENTRE_Y,
        RADIUS * scales[index]!.at(axis.value ?? 0),
        index,
        count,
      ),
    )

  /**
   * The order the windows are drawn in: the two the page is not about
   * first, and the focused one last, so that the shape the numbers
   * belong to is never buried under the others.
   */
  const drawn = [
    ...series.filter((entry) => entry.key !== focus),
    ...series.filter((entry) => entry.key === focus),
  ]

  return (
    <div data-testid="progress-radar">
      <svg
        viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
        preserveAspectRatio="xMidYMid meet"
        className="block h-auto w-full"
        role="img"
        aria-label={t('progress.wall.profile.alt')}
        data-testid="progress-radar-chart"
      >
        {/* the rings the axes are read against */}
        {RINGS.map((ring) => (
          <path
            key={ring}
            d={polygonPath(
              Array.from({ length: count }, (_, index) =>
                radialPoint(CENTRE_X, CENTRE_Y, RADIUS * ring, index, count),
              ),
            )}
            fill="none"
            stroke="var(--chart-grid)"
            strokeWidth="1"
          />
        ))}

        {/* the axes themselves */}
        {Array.from({ length: count }, (_, index) => {
          const point = radialPoint(CENTRE_X, CENTRE_Y, RADIUS, index, count)
          return (
            <line
              key={index}
              x1={CENTRE_X}
              y1={CENTRE_Y}
              x2={point.x}
              y2={point.y}
              stroke="var(--chart-grid)"
              strokeWidth="1"
            />
          )
        })}

        {/*
          the windows, one polygon each.

          **The fills are drawn first, all of them, and the lines over
          them**: a fill that covered another polygon's edge would hide
          where that polygon is. Each fill is a light wash of the
          polygon's own hue, and each line is thin and in that same hue.
        */}
        <g data-testid="progress-radar-fills">
          {drawn.map((entry) => {
            const points = pointsOf(entry)
            if (points.length < 3) return null
            return (
              <path
                key={`fill-${entry.key}`}
                d={polygonPath(points)}
                fill={seriesColour(series.indexOf(entry))}
                fillOpacity={0.1}
                stroke="none"
                data-testid={`progress-radar-fill-${entry.key}`}
              />
            )
          })}
        </g>
        {drawn.map((entry) => {
          const colour = seriesColour(series.indexOf(entry))
          const focused = entry.key === focus
          const points = pointsOf(entry)
          return (
            <path
              key={entry.key}
              d={polygonPath(points)}
              fill="none"
              stroke={colour}
              strokeWidth={focused ? 2.5 : 2}
              data-testid={`progress-radar-${entry.key}`}
            />
          )
        })}

        {/* the names of the axes */}
        {series[0]!.axes.map((axis, index) => {
          const place = labelAnchor(index, count)
          return (
            <text
              key={axis.key}
              x={place.x}
              y={place.y}
              textAnchor={place.anchor}
              dominantBaseline="middle"
              fontSize="9.5"
              fill="var(--ink-faint)"
              data-testid={`progress-radar-label-${axis.key}`}
            >
              {axis.label}
            </text>
          )
        })}
      </svg>

      {/*
        The legend: the windows, in the words the answers use. The chart
        draws the shape; the legend names what the shapes are.
      */}
      <ul className="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1">
        {series.map((entry) => (
          <li key={entry.key} className="inline-flex items-center gap-1.5">
            <i
              className="inline-block h-2.5 w-2.5 rounded-[3px]"
              style={{ background: seriesColour(series.indexOf(entry)) }}
            />
            <span className="text-[11.5px] text-ink-muted">{entry.label}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
