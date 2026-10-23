/**
 * Route thumbnail — an inline SVG rendering of an activity's GPS track.
 *
 * Feed cards show dozens of these at once. A real map tile layer per card would
 * be far too heavy (and needs network access), so we draw the track against a
 * subtle vector "map" backdrop. The full interactive map is reserved for the
 * activity detail page.
 */

import { useMemo } from 'react'
import { cn } from './ui'

interface Props {
  points: [number, number][] | null | undefined
  className?: string
  strokeWidth?: number
  showMarkers?: boolean
}

export function RouteThumb({
  points,
  className,
  strokeWidth = 3,
  showMarkers = true,
}: Props) {
  const geometry = useMemo(() => {
    if (!points || points.length < 2) return null

    const lats = points.map((p) => p[0])
    const lons = points.map((p) => p[1])
    const minLat = Math.min(...lats)
    const maxLat = Math.max(...lats)
    const minLon = Math.min(...lons)
    const maxLon = Math.max(...lons)

    // Guard against a degenerate (zero-area) track.
    const spanLat = maxLat - minLat || 0.001
    const spanLon = maxLon - minLon || 0.001

    const W = 100
    const H = 100
    const pad = 12

    const project = (lat: number, lon: number): [number, number] => {
      const x = pad + ((lon - minLon) / spanLon) * (W - pad * 2)
      // Latitude grows northwards; SVG y grows downwards.
      const y = H - pad - ((lat - minLat) / spanLat) * (H - pad * 2)
      return [x, y]
    }

    const projected = points.map(([lat, lon]) => project(lat, lon))

    // On a closed loop the finish sits on the start; drawing both markers
    // would just stack them, so only show the finish when it is distinct.
    const start = projected[0]
    const end = projected[projected.length - 1]
    const spread = Math.max(W - pad * 2, 1)
    const endFarEnough =
      Math.abs(end[0] - start[0]) > spread * 0.04 ||
      Math.abs(end[1] - start[1]) > spread * 0.04

    // Simplify: keep every point but drop collinear runs so the path stays light.
    let path = ''
    let drawn = 0
    for (let i = 0; i < projected.length; i += 1) {
      const [x, y] = projected[i]
      path += `${drawn === 0 ? 'M' : 'L'}${x.toFixed(2)},${y.toFixed(2)} `
      drawn += 1
    }

    return {
      path,
      start,
      end,
      endFarEnough,
      W,
      H,
    }
  }, [points])

  if (!geometry) {
    return (
      <div
        className={cn(
          'flex items-center justify-center bg-surface-soft text-[12px] text-ink-faint',
          className,
        )}
      >
        No route
      </div>
    )
  }

  return (
    <svg
      viewBox={`0 0 ${geometry.W} ${geometry.H}`}
      preserveAspectRatio="xMidYMid slice"
      className={cn('bg-[#f4f1ec]', className)}
      role="img"
      aria-label="Route map"
    >
      <defs>
        <pattern id="grid" width="12" height="12" patternUnits="userSpaceOnUse">
          <path d="M12 0 L0 0 0 12" fill="none" stroke="#e6e1d8" strokeWidth="0.5" />
        </pattern>
        <linearGradient id="route-grad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#ff7a33" />
          <stop offset="100%" stopColor="#fc5200" />
        </linearGradient>
      </defs>
      <rect width={geometry.W} height={geometry.H} fill="url(#grid)" />
      {/* Park / water blobs for a map-like texture. */}
      <ellipse cx="18" cy="80" rx="22" ry="16" fill="#e3ecdf" opacity="0.9" />
      <ellipse cx="84" cy="22" rx="18" ry="13" fill="#dfeaf2" opacity="0.9" />
      <path d={geometry.path} fill="none" stroke="#ffffff" strokeWidth={strokeWidth + 2.4} strokeLinecap="round" strokeLinejoin="round" opacity="0.95" />
      <path d={geometry.path} fill="none" stroke="url(#route-grad)" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round" />
      {showMarkers ? (
        <>
          <circle cx={geometry.start[0]} cy={geometry.start[1]} r={strokeWidth * 0.9} fill="#ffffff" stroke="#1a9c5a" strokeWidth={strokeWidth * 0.6} />
          {geometry.endFarEnough ? (
            <circle cx={geometry.end[0]} cy={geometry.end[1]} r={strokeWidth * 0.9} fill="#ffffff" stroke="#d93a2b" strokeWidth={strokeWidth * 0.6} />
          ) : null}
        </>
      ) : null}
    </svg>
  )
}
