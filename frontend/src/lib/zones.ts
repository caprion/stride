/**
 * The athlete's zones, as the app draws them.
 *
 * **One palette, one zone.** The band on the run detail's Effort panel and
 * the band behind the run film's timeline are the same bands of the same
 * model, and they are the same colour: a colour that meant one zone on one
 * screen and another on the next would be a second place where the zones
 * are decided.
 *
 * The five bands run from the quietest to the loudest, in the product's own
 * azure — and the fifth is amber, because nothing in this product is red:
 * a band is a fact about a run, never a verdict about the athlete.
 */

/** The five bands, in the order the athlete's model numbers them. */
export const ZONE_COLOURS = [
  'var(--brand-200)',
  'var(--brand-300)',
  'var(--brand-500)',
  'var(--brand-600)',
  'var(--accent-500)',
]

/**
 * The space **outside** the zones.
 *
 * A reading at or below the first floor, or no reading at all. It is a
 * name and not a verdict, and it is the app's own quiet grey.
 */
export const OUTSIDE_ZONE_COLOUR = 'var(--chart-grid)'

/**
 * The colour of a band.
 *
 * `null` — a reading outside the zones — is the space outside them.
 */
export function zoneColour(zone: number | null): string {
  if (zone === null || zone < 0 || zone >= ZONE_COLOURS.length) {
    return OUTSIDE_ZONE_COLOUR
  }
  return ZONE_COLOURS[zone] as string
}
