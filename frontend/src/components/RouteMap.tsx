/**
 * The route, drawn from the points the API sent.
 *
 * **No map library, no tiles, no server.** The route is the athlete's own
 * data and it never leaves the device, so the map does not either: the
 * points are drawn as a polyline over their own bounding box, in an SVG
 * the app draws itself. There is no tile server to reach, no account to
 * hold, and nothing a third party could see.
 *
 * The app draws geometry and nothing else. The points are the API's, in
 * the order the API sent them, and the shape is the shape they make —
 * the app does not smooth them, shorten them or invent a street name.
 *
 * Three sizes, because the plan asks for three: the map on the run's own
 * page, a thumbnail in the feed at wide widths, and a mini thumbnail on
 * Today where there is room for it.
 */

import { t } from '../i18n'

/** A point, as the API sends it: `[latitude, longitude]`. */
export type GeoPoint = readonly [number, number]

/** The three sizes the route is drawn at. */
export type RouteVariant = 'full' | 'thumb' | 'mini'

/**
 * The box each size draws in.
 *
 * The width is the same for all three — the drawing is scaled to fit —
 * and the height is the aspect: 16:9 for the map (the plan's banner at
 * compact), 3:2 for the feed's thumbnail and 2:1 for the mini one.
 */
const BOXES: Record<RouteVariant, { width: number; height: number; className: string }> = {
  full: { width: 100, height: 56.25, className: 'aspect-video' },
  thumb: { width: 100, height: 66.67, className: 'aspect-[3/2]' },
  mini: { width: 100, height: 50, className: 'aspect-[2/1]' },
}

/** The room the drawing keeps from the edge, in the box's own units. */
const PADDING = 10

export function RouteMap({
  points,
  variant = 'full',
  label,
}: {
  points: readonly GeoPoint[]
  variant?: RouteVariant
  label?: string
}) {
  const drawn = points.filter(
    (point) => Number.isFinite(point[0]) && Number.isFinite(point[1]),
  ) as GeoPoint[]
  // A route needs two points to be a route. With fewer, there is nothing
  // to draw, and the app draws nothing.
  if (drawn.length < 2) return null

  const box = BOXES[variant]

  /**
   * A degree of longitude is shorter than a degree of latitude away from
   * the equator, so the drawing scales the two the way the ground does —
   * otherwise every route comes out narrower than it ran.
   */
  const meanLatitude = drawn.reduce((sum, point) => sum + point[0], 0) / drawn.length
  const longitudeScale = Math.cos((meanLatitude * Math.PI) / 180)

  const xs = drawn.map((point) => point[1] * longitudeScale)
  const ys = drawn.map((point) => point[0])
  const minX = Math.min(...xs)
  const maxX = Math.max(...xs)
  const minY = Math.min(...ys)
  const maxY = Math.max(...ys)
  const spanX = maxX - minX || 1e-9
  const spanY = maxY - minY || 1e-9

  // One scale for both axes: the shape of the route is the shape it ran.
  const scale = Math.min(
    (box.width - 2 * PADDING) / spanX,
    (box.height - 2 * PADDING) / spanY,
  )
  const left = (box.width - spanX * scale) / 2
  const top = (box.height - spanY * scale) / 2
  const x = (index: number) => left + (xs[index] as number - minX) * scale
  // North is up: the latitude grows upwards, the drawing's y grows down.
  const y = (index: number) => top + (maxY - (ys[index] as number)) * scale

  const line = drawn
    .map((_, index) => `${x(index).toFixed(2)},${y(index).toFixed(2)}`)
    .join(' ')
  const last = drawn.length - 1
  const startX = x(0)
  const startY = y(0)
  const endX = x(last)
  const endY = y(last)

  return (
    <div className={box.className} data-testid="route-map">
      <svg
        viewBox={`0 0 ${box.width} ${box.height}`}
        preserveAspectRatio="xMidYMid meet"
        className="block h-full w-full"
        role="img"
        aria-label={label ?? t('activity.route.alt')}
      >
        {/*
          The line is drawn in the screen's own pixels, not in the
          drawing's: `non-scaling-stroke` keeps it the same weight in the
          map and in the thumbnails, whatever the box is scaled to.
        */}
        <polyline
          points={line}
          fill="none"
          stroke="#1E5E96"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          vectorEffect="non-scaling-stroke"
        />
        {/*
          The start: a filled dot. The end: the same dot, hollow — a ring
          of the brand's own dark, with the page's white inside it. Both
          are drawn the same way, so they are the same size on every
          screen: a zero-length line with a round cap is a dot.
        */}
        <path
          d={`M ${startX} ${startY} L ${startX} ${startY}`}
          stroke="#1E5E96"
          strokeWidth={8}
          strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          data-testid="route-start"
        />
        <path
          d={`M ${endX} ${endY} L ${endX} ${endY}`}
          stroke="#0D3A63"
          strokeWidth={11}
          strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
        />
        <path
          d={`M ${endX} ${endY} L ${endX} ${endY}`}
          stroke="#FFFFFF"
          strokeWidth={6}
          strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          data-testid="route-end"
        />
      </svg>
    </div>
  )
}
