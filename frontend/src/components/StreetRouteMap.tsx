import { useEffect, useRef, useState } from 'react'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import { t } from '../i18n'
import { RouteMap, type GeoPoint } from './RouteMap'

export function StreetRouteMap({ points }: { points: readonly GeoPoint[] }) {
  const [street, setStreet] = useState(false)
  const [failed, setFailed] = useState(false)
  const container = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!street || failed || !container.current) return
    const positions = points
      .filter(
        ([lat, lon]) =>
          Number.isFinite(lat) &&
          Number.isFinite(lon) &&
          Math.abs(lat) <= 90 &&
          Math.abs(lon) <= 180,
      )
      .map(([lat, lon]) => L.latLng(lat, lon))
    if (positions.length < 2) return
    const map = L.map(container.current, {
      zoomAnimation: false,
      fadeAnimation: false,
      markerZoomAnimation: false,
      scrollWheelZoom: false,
    })
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
      referrerPolicy: 'strict-origin-when-cross-origin',
    })
      .on('tileerror', () => setFailed(true))
      .addTo(map)
    const route = L.polyline(positions, { color: '#124A7C', weight: 4 }).addTo(map)
    L.circleMarker(positions[0]!, { radius: 5, color: '#124A7C', fillOpacity: 1 }).addTo(map)
    L.circleMarker(positions[positions.length - 1]!, {
      radius: 6,
      color: '#124A7C',
      fillColor: '#ffffff',
      fillOpacity: 1,
    }).addTo(map)
    map.fitBounds(route.getBounds(), { padding: [24, 24], maxZoom: 16, animate: false })
    const observer = new ResizeObserver(() => map.invalidateSize({ animate: false }))
    observer.observe(container.current)
    return () => {
      observer.disconnect()
      map.stop()
      map.remove()
    }
  }, [points, street, failed])

  return (
    <div>
      <div className="mb-2 flex flex-wrap gap-2">
        <button
          type="button"
          className={`tap rounded-md border border-edge px-3 text-[13px] ${!street || failed ? 'bg-brand-50 text-brand-700' : ''}`}
          aria-pressed={!street || failed}
          onClick={() => setStreet(false)}
        >
          {t('activity.route.offline')}
        </button>
        <button
          type="button"
          className={`tap rounded-md border border-edge px-3 text-[13px] ${street && !failed ? 'bg-brand-50 text-brand-700' : ''}`}
          aria-pressed={street && !failed}
          onClick={() => {
            setFailed(false)
            setStreet(true)
          }}
        >
          {t('activity.route.street')}
        </button>
      </div>
      {failed ? (
        <p className="mb-2 text-[13px] text-ink-muted" role="alert">
          {t('activity.route.failed')}
        </p>
      ) : null}
      {street && !failed ? (
        <div
          ref={container}
          className="street-map relative z-0 h-[280px] w-full overflow-hidden rounded-md"
          data-testid="street-map"
        />
      ) : (
        <div className="rounded-md border border-edge bg-brand-50">
          <RouteMap points={points} />
        </div>
      )}
      <p className="mt-2 text-[12px] text-ink-muted">{t('activity.route.startEnd')}</p>
      <p className="mt-1 text-[12px] text-ink-muted">{t('activity.route.disclosure')}</p>
    </div>
  )
}
