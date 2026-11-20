/**
 * The flyby: the run's route with a camera that follows the runner.
 *
 * **The app's own map.** The route is the athlete's own, drawn on this
 * device, and the camera follows the runner over it: the track is coloured
 * by the pace, the marker moves, and the shape of the run does not change
 * while the film plays.
 *
 * **The street tiles are behind an explicit action**, exactly as they are
 * on the run detail: the athlete asks for the OpenStreetMap tiles, and the
 * app says what they reveal. Without them the flyby is the athlete's own
 * route on its own, and nothing leaves the device.
 *
 * The runner's place on the route is found the way the athlete ran it: the
 * distance the watch recorded at that moment, as a share of the run's
 * distance, along the route the watch recorded. It is an estimate — the
 * GPS line and the watch's distance are two measurements of the same run —
 * and it is close enough to keep the marker on the runner's line.
 */

import { useEffect, useMemo, useRef, useState } from 'react'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import { t } from '../i18n'
import { RouteMap, type GeoPoint } from './RouteMap'
import { sampleAt } from './RunFilm'
import { robustRange } from '../lib/chart'
import type { FilmSample } from '../lib/types'
import { token } from '../lib/tokens'

/**
 * The colours of the pace, from the slowest to the fastest.
 *
 * The pace is the athlete's own signal and it is amber everywhere in the
 * app: on the timeline's axis and on the track. The steps run from the
 * palest to the deepest, so the fastest part of the run is the darkest.
 */
const PACE_RAMP = [
  '--accent-100',
  '--accent-300',
  '--accent-400',
  '--accent-500',
  '--accent-600',
  '--accent-700',
]

/** The zoom the camera follows at. */
const FOLLOW_ZOOM = 16

/** The flyby. */
export function FlybyMap({
  points,
  samples,
  playhead,
}: {
  points: readonly GeoPoint[]
  samples: FilmSample[]
  playhead: number
}) {
  const container = useRef<HTMLDivElement>(null)
  const map = useRef<L.Map | null>(null)
  const marker = useRef<L.CircleMarker | null>(null)
  /** The route's own geometry: the positions, the lengths along it, its
   * total length and the run's distance. */
  const track = useRef<{
    positions: L.LatLng[]
    lengths: number[]
    total: number
    distance: number
  } | null>(null)
  const [street, setStreet] = useState(false)
  const [failed, setFailed] = useState(false)

  /** The positions the route holds, and the distance the run covered. */
  const route = useMemo(() => {
    const positions = points
      .filter(
        ([lat, lon]) =>
          Number.isFinite(lat) &&
          Number.isFinite(lon) &&
          Math.abs(lat) <= 90 &&
          Math.abs(lon) <= 180,
      )
      .map(([lat, lon]) => L.latLng(lat, lon))
    const distance = samples[samples.length - 1]?.distance?.raw ?? 0
    return { positions, distance }
  }, [points, samples])

  /** The map, the track and the marker: drawn once. */
  useEffect(() => {
    const node = container.current
    if (!node || route.positions.length < 2) return

    const instance = L.map(node, {
      zoomAnimation: false,
      fadeAnimation: false,
      markerZoomAnimation: false,
      scrollWheelZoom: false,
      // The renderer draws the track and the marker in one SVG, and the
      // SVG is the map's own size: the default leaves a margin of ten per
      // cent around it, and a margin is a box that reaches past the edge
      // of the window.
      renderer: L.svg({ padding: 0 }),
    })

    // The length of the route, measured with the map's own distance.
    const lengths: number[] = [0]
    for (let index = 1; index < route.positions.length; index += 1) {
      const previous = route.positions[index - 1] as L.LatLng
      const current = route.positions[index] as L.LatLng
      lengths.push((lengths[index - 1] as number) + instance.distance(previous, current))
    }
    const total = lengths[lengths.length - 1] ?? 0
    track.current = { positions: route.positions, lengths, total, distance: route.distance }

    // The track, coloured by the pace, drawn once.
    instance.fitBounds(L.latLngBounds(route.positions), {
      padding: [24, 24],
      maxZoom: FOLLOW_ZOOM,
      animate: false,
    })
    const ramp = PACE_RAMP.map((name) => token(name))
    for (const run of paceRuns(route.positions, lengths, total, route.distance, samples, ramp)) {
      L.polyline(run.positions, { color: run.colour, weight: 4 }).addTo(instance)
    }

    // Where the run started, and where it ended: the app's own marks.
    L.circleMarker(route.positions[0] as L.LatLng, {
      radius: 5,
      color: token('--brand-600'),
      fillOpacity: 1,
    }).addTo(instance)
    L.circleMarker(route.positions[route.positions.length - 1] as L.LatLng, {
      radius: 6,
      color: token('--brand-600'),
      fillColor: token('--surface'),
      fillOpacity: 1,
    }).addTo(instance)

    marker.current = L.circleMarker(route.positions[0] as L.LatLng, {
      radius: 6,
      color: token('--ink'),
      fillColor: token('--ink'),
      fillOpacity: 1,
      weight: 2,
    }).addTo(instance)

    // The camera starts on the runner, and follows it from there.
    instance.setView(route.positions[0] as L.LatLng, FOLLOW_ZOOM, { animate: false })
    map.current = instance

    const observer = new ResizeObserver(() => instance.invalidateSize({ animate: false }))
    observer.observe(node)
    return () => {
      observer.disconnect()
      marker.current = null
      track.current = null
      map.current = null
      instance.stop()
      instance.remove()
    }
  }, [route, samples])

  /** The street tiles: the athlete's own action, the app's own layer. */
  useEffect(() => {
    const instance = map.current
    if (!instance || !street || failed) return
    const layer = L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
      referrerPolicy: 'strict-origin-when-cross-origin',
    })
      .on('tileerror', () => setFailed(true))
      .addTo(instance)
    return () => {
      instance.removeLayer(layer)
    }
  }, [street, failed])

  /** The runner's place on the route, and the camera that follows it. */
  useEffect(() => {
    const instance = map.current
    const runner = marker.current
    const geometry = track.current
    if (!instance || !runner || !geometry) return
    const position = positionAt(geometry, samples, playhead)
    if (!position) return
    runner.setLatLng(position)
    instance.panTo(position, { animate: false, duration: 0 })
  }, [playhead, samples])

  if (route.positions.length < 2) {
    return (
      <div className="rounded-md border border-edge bg-brand-50">
        <RouteMap points={points as GeoPoint[]} />
      </div>
    )
  }

  return (
    <div>
      <div className="mb-2 flex flex-wrap gap-2">
        <button
          type="button"
          className={`tap rounded-md border border-edge px-3 text-[13px] ${!street || failed ? 'bg-brand-50 text-brand-700' : ''}`}
          aria-pressed={!street || failed}
          data-testid="film-outline"
          onClick={() => {
            setFailed(false)
            setStreet(false)
          }}
        >
          {t('activity.route.offline')}
        </button>
        <button
          type="button"
          className={`tap rounded-md border border-edge px-3 text-[13px] ${street && !failed ? 'bg-brand-50 text-brand-700' : ''}`}
          aria-pressed={street && !failed}
          data-testid="film-street"
          onClick={() => {
            setFailed(false)
            setStreet(true)
          }}
        >
          {t('activity.route.street')}
        </button>
      </div>
      {failed ? (
        <p className="mb-2 text-[13px] text-ink-muted" role="alert" data-testid="film-street-failed">
          {t('activity.route.failed')}
        </p>
      ) : null}
      <div
        ref={container}
        className="street-map relative z-0 h-[280px] w-full overflow-hidden rounded-md bg-brand-50"
        data-testid="film-flyby"
      />
      <p className="mt-2 text-[12px] text-ink-muted">{t('activity.route.startEnd')}</p>
      <p className="mt-1 text-[12px] text-ink-muted">{t('activity.route.disclosure')}</p>
    </div>
  )
}

/**
 * The runs of the track that share a pace colour.
 *
 * The track is drawn as a few lines rather than a thousand: a line per
 * segment would be a thousand objects on the map, and the shape of the run
 * does not change while the film plays.
 */
function paceRuns(
  positions: L.LatLng[],
  lengths: number[],
  total: number,
  distance: number,
  samples: FilmSample[],
  ramp: string[],
): { colour: string; positions: L.LatLng[] }[] {
  const paces = samples
    .map((sample) => sample.pace?.raw)
    .filter((value): value is number => value !== null && value !== undefined)
  // The same scale the timeline draws: the middle of the readings, so one
  // walk break does not flatten the whole track into one colour.
  const [fastest, slowest] = robustRange(paces)

  const runs: { colour: string; positions: L.LatLng[] }[] = []
  for (let index = 1; index < positions.length; index += 1) {
    const along = total > 0 ? ((lengths[index] as number) / total) * distance : 0
    const colour = paceColour(paceAtDistance(samples, along), fastest, slowest, ramp)
    const last = runs[runs.length - 1]
    if (last && last.colour === colour) {
      last.positions.push(positions[index] as L.LatLng)
    } else {
      runs.push({
        colour,
        positions: [positions[index - 1] as L.LatLng, positions[index] as L.LatLng],
      })
    }
  }
  return runs
}

/** The pace at a distance along the run, from the samples. */
function paceAtDistance(samples: FilmSample[], metres: number): number | null {
  let found: number | null = null
  for (const sample of samples) {
    const distance = sample.distance?.raw
    if (distance === null || distance === undefined) continue
    if (distance > metres) break
    if (sample.pace?.raw !== null && sample.pace?.raw !== undefined) {
      found = sample.pace.raw
    }
  }
  return found
}

/** The colour of a pace: the app's own ramp, from the slowest to the fastest. */
function paceColour(
  pace: number | null,
  fastest: number,
  slowest: number,
  ramp: string[],
): string {
  if (pace === null || slowest <= fastest) return token('--brand-200')
  const share = (pace - fastest) / (slowest - fastest)
  const step = colourStep(1 - share, ramp.length)
  return ramp[Math.min(ramp.length - 1, Math.max(0, step))] as string
}

/**
 * A step of the colour scale, counted from zero.
 *
 * The drawing's own geometry: which band of the scale a value falls in.
 */
function colourStep(share: number, steps: number): number {
  return Math.floor(share * steps)
}

/**
 * Where the runner is, at a moment of the run.
 *
 * The distance the watch recorded at that moment, as a share of the run's
 * distance, along the route the watch recorded.
 */
function positionAt(
  geometry: { positions: L.LatLng[]; lengths: number[]; total: number; distance: number },
  samples: FilmSample[],
  playhead: number,
): L.LatLng | null {
  const sample = sampleAt(samples, playhead)
  const distance = sample?.distance?.raw ?? 0
  const share = geometry.distance > 0 ? distance / geometry.distance : 0
  const target = Math.min(geometry.total, Math.max(0, share * geometry.total))
  let index = 0
  while (
    index < geometry.lengths.length - 1 &&
    (geometry.lengths[index + 1] as number) < target
  ) {
    index += 1
  }
  return geometry.positions[index] ?? null
}
