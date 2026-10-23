/**
 * Map tile policy.
 *
 * The browser never talks to a third party. Tiles are fetched by this app's own
 * server (which identifies itself to OpenStreetMap with a proper User-Agent),
 * cached on disk, and served back from the same origin — so the athlete's IP
 * address is never handed to a tile provider, and the map keeps working when
 * the network is flaky.
 */

export const TILE_URL = '/api/v1/tiles/{z}/{x}/{y}.png'

export const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'

export function tileLayer(): string {
  return TILE_URL
}

/** True when the map should render its own backdrop instead of tiles. */
export function tilesEnabled(): boolean {
  return true
}
