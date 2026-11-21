/**
 * The athlete's profile, as a radar.
 *
 * Two windows, one chart: the last thirty days and the last ninety, each
 * on its own polygon, each axis scaled to the **larger of the two values
 * on that axis**. So the chart says how the two windows differ in shape,
 * and it says nothing about what any of the numbers means — the scale is
 * the athlete's own, not an anchor somebody else chose.
 *
 * **The values are the router's.** The facets come from
 * `/insights/rolling`, and every number here is the core's own rendering
 * of it. The chart draws; it computes nothing.
 *
 * The drawing is SVG, made by the app's own chart helpers (`lib/chart.ts`):
 * the scale that puts the larger of two values on the outside, the path of
 * a polygon and the point on a circle. It is sized by its own viewBox, so
 * it shrinks with the card and never overflows it.
 */

import { polygonPath, radialPoint, scaleToLargest, type Point } from '../lib/chart'
import { t } from '../i18n'

/** One axis of the radar. */
export interface RadarAxis {
  /** The facet's key: "volume". */
  key: string
  /** The facet's name, in the core's words: "Sport variety". */
  label: string
  /** The value in the window, in the axis's own unit. Null when there is none. */
  value: number | null
  /** The value as the core renders it: "58.4 km". */
  text: string | null
}

/** One window of the chart. */
export interface RadarSeries {
  /** The series' own key: "short". */
  key: string
  /** The window's name, in the app's words: "The last 28 days". */
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

/** The colour of each series. */
const COLOURS: Record<string, string> = {
  short: 'var(--brand-500)',
  long: 'var(--accent-500)',
}

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

export function ProfileRadar({ series }: { series: RadarSeries[] }) {
  if (series.length === 0) return null

  const count = series[0]?.axes.length ?? 0
  if (count < 3) return null

  /**
   * Each axis' own scale: the larger of the two windows' values on that
   * axis is the outside of the chart, and the other one sits inside it.
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

        {/* the two windows, one polygon each */}
        {series.map((entry, index) => {
          const colour = COLOURS[entry.key] ?? 'var(--chart-line)'
          const points = pointsOf(entry)
          return (
            <path
              key={entry.key}
              d={polygonPath(points)}
              fill={colour}
              fillOpacity={index === 0 ? 0.18 : 0.08}
              stroke={colour}
              strokeWidth="2"
              strokeDasharray={index === 0 ? undefined : '5 3'}
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
        The legend: the two windows, in the words the answers use. The
        chart draws the shape; the legend names what the shapes are.
      */}
      <ul className="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1">
        {series.map((entry) => (
          <li key={entry.key} className="inline-flex items-center gap-1.5">
            <i
              className="inline-block h-2.5 w-2.5 rounded-[3px]"
              style={{ background: COLOURS[entry.key] ?? 'var(--chart-line)' }}
            />
            <span className="text-[11.5px] text-ink-muted">{entry.label}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
