/**
 * The athlete's facets: **the axes of the shape**.
 *
 * Six of the seven facets the answer carries, in the order they are
 * drawn: the volume, the consistency, the sport variety, the density,
 * the duration and the intensity. The days of the week are not an axis,
 * because a week has no shape.
 *
 * **The list lives here, and not in a chart.** The radar is gone — a
 * polygon over fixed anchors is a claim about a shape, and the anchors
 * are somebody else's coaching assumptions — but the axes are still
 * needed: the shape is drawn over them, and the table under the drawing
 * is named by them.
 *
 * The names themselves are the core's: the key is the core's key, and
 * the label is the core's label. The app draws; it names nothing.
 */

/** The axes of the athlete's shape, in the order they are drawn. */
export const SHAPE_AXES = [
  'volume',
  'consistency',
  'variety',
  'density',
  'duration',
  'intensity',
] as const

/** The key of an axis of the shape. */
export type ShapeAxisKey = (typeof SHAPE_AXES)[number]
