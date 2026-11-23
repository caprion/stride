/**
 * The heatmap's grid, drawn by the app itself.
 *
 * **No map library, no tiles, no server.** The cells are the athlete's own
 * data and they never leave the device, so the drawing does not either:
 * the app draws the rectangles the answer carries, over their own box, in
 * an SVG it draws itself. There is no tile server to reach and no account
 * to hold.
 *
 * **A cell is a count, not a place.** The drawing shades a cell by how
 * many runs went through it, and it names nothing: no street, no address,
 * no neighbourhood and no coordinate is written on the picture. A grid of
 * cells over a map the athlete can draw themselves is the whole design.
 */

import { t } from '../i18n'
import type { HeatmapCell, HeatmapExtent } from '../lib/types'

/** The box the drawing is made in. */
const BOX = 100

/** The room the drawing keeps from the edge, in the box's own units. */
const PADDING = 4

/**
 * The steps of the shading, from the quietest cell to the busiest.
 *
 * The product's own azure: a density, not a verdict. Nothing here is red,
 * because nothing here is a problem.
 */
const STEPS = [
  'var(--brand-100)',
  'var(--brand-200)',
  'var(--brand-400)',
  'var(--brand-600)',
]

export function HeatmapGrid({
  cells,
  extent,
}: {
  cells: HeatmapCell[]
  extent: HeatmapExtent
}) {
  if (cells.length === 0) return null

  /**
   * The cells the answer carries are the cells of the **picture**: the
   * places the running is. The grid is a global one at a fixed size, so
   * the box that holds every cell the athlete ever touched can be a
   * thousand kilometres across — and the cells they actually ran in are
   * then one pixel each, and the picture is blank.
   */
  const drawn = cells

  /**
   * A degree of longitude is shorter than a degree of latitude away from
   * the equator, so the drawing scales the two the way the ground does —
   * otherwise every grid comes out wider than it is tall.
   */
  const meanLatitude = (extent.north + extent.south) / 2
  const scale = Math.cos((meanLatitude * Math.PI) / 180)

  const west = Math.min(...drawn.map((cell) => cell.west)) * scale
  const east = Math.max(...drawn.map((cell) => cell.east)) * scale
  const minX = west
  const maxX = east
  const minY = Math.min(...drawn.map((cell) => cell.south))
  const maxY = Math.max(...drawn.map((cell) => cell.north))

  const spanX = maxX - minX || 1e-9
  const spanY = maxY - minY || 1e-9
  const fit = Math.min((BOX - 2 * PADDING) / spanX, (BOX - 2 * PADDING) / spanY)
  const left = (BOX - spanX * fit) / 2
  const top = (BOX - spanY * fit) / 2

  const x = (longitude: number) => left + (longitude * scale - minX) * fit
  // North is up: the latitude grows upwards, the drawing's y grows down.
  const y = (latitude: number) => top + (maxY - latitude) * fit

  const busiest = drawn.reduce((most, cell) => Math.max(most, cell.runs), 0)

  return (
    <svg
      viewBox={`0 0 ${BOX} ${BOX}`}
      preserveAspectRatio="xMidYMid meet"
      className="block h-full w-full"
      role="img"
      aria-label={t('heatmap.alt')}
      data-testid="heatmap-grid"
    >
      {drawn.map((cell) => {
        const x0 = x(cell.west)
        const y0 = y(cell.north)
        const x1 = x(cell.east)
        const y1 = y(cell.south)
        return (
          <rect
            key={`${cell.x}-${cell.y}`}
            x={x0}
            y={y0}
            // A hair more than the cell, so the cells of a row tile the row
            // without a seam of background between them.
            width={x1 - x0 + 0.5}
            height={y1 - y0 + 0.5}
            fill={shade(cell.runs, busiest)}
            data-testid="heatmap-cell"
            data-runs={cell.runs}
          />
        )
      })}
    </svg>
  )
}

/** The colour of a cell, by how many runs went through it.
 *
 * The busiest cell is the deepest one; a cell with one run in it is the
 * quietest. The scale is the answer's own: nothing is compared with
 * anything but the map it is on.
 */
function shade(runs: number, busiest: number): string {
  if (busiest <= 1) return STEPS[0] as string
  const ratio = (runs - 1) / (busiest - 1)
  const step = Math.round(ratio * (STEPS.length - 1))
  return STEPS[Math.min(STEPS.length - 1, Math.max(0, step))] as string
}
