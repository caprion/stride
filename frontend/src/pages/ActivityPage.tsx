/**
 * Activity detail — the flagship page.
 *
 * Layout mirrors Strava: a full-width interactive route map, the title block,
 * a tight stat strip, stream charts, a splits table, heart-rate zones and
 * previous/next navigation between neighbouring activities.
 *
 * Every value comes from the typed API client (`api.activity`, `api.activities`,
 * `api.patchActivity`); every visual comes from the shared kit in `components/`.
 */

import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CircleMarker, MapContainer, Polyline, TileLayer } from 'react-leaflet'
import { latLngBounds, type LatLngBounds } from 'leaflet'
import { Link, useParams } from 'react-router-dom'
import {
  ExternalLink,
  Activity as ActivityIcon,
  CalendarDays,
  ChevronLeft,
  ChevronRight,
  Flame,
  Footprints,
  Gauge,
  MapPin,
  Mountain,
  Pencil,
  Route as RouteIcon,
  Save,
  Timer,
  Watch,
  X,
  Zap,
} from 'lucide-react'
import { api } from '../lib/api'
import type { ActivityDetail, ActivitySummary, Lap, Stream } from '../types'
import type { Fingerprint } from '../types_phase2'
import {
  formatDateLong,
  formatDateTime,
  formatDistance,
  formatDistanceShort,
  formatDuration,
  formatElevation,
  formatHr,
  formatNumber,
  formatPace,
  formatPaceWithUnit,
  paceToMinutes,
  sportLabel,
  SPORT_COLOR,
} from '../lib/format'
import { StreamChart } from '../components/charts'
import {
  Badge,
  Button,
  Card,
  CardHeader,
  Divider,
  EmptyState,
  Skeleton,
  StatTile,
  cn,
  inputClass,
} from '../components/ui'

/* ------------------------------------------------------------------ consts */

const BRAND = '#fc5200'
const HR_COLOR = '#fc5200'
const PACE_COLOR = '#c14fb0'
const ELEVATION_COLOR = '#2d9cdb'

const TILE_URL = '/api/v1/tiles/{z}/{x}/{y}.png'
const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'

const MAP_HEIGHT = 'h-[360px] sm:h-[400px] lg:h-[420px]'

/* ------------------------------------------------------------------ helpers */

interface Point {
  x: number
  value: number | null
}

function toStreamPoints(stream: Stream | undefined): Point[] {
  if (!stream) return []
  return stream.seconds.map((second, index) => ({
    x: second,
    value: stream.values[index] ?? null,
  }))
}

function toPacePoints(stream: Stream | undefined): Point[] {
  // Pace arrives in seconds per km; the chart works in minutes per km.
  return toStreamPoints(stream).map((point) => ({
    x: point.x,
    value: point.value === null ? null : paceToMinutes(point.value),
  }))
}

function average(values: (number | null)[]): number | null {
  const clean = values.filter((v): v is number => v !== null && Number.isFinite(v))
  if (!clean.length) return null
  return clean.reduce((sum, v) => sum + v, 0) / clean.length
}

function lapElevation(lap: Lap): number | null {
  if (lap.elevation_gain_m !== null && lap.elevation_gain_m !== undefined) {
    return lap.elevation_gain_m
  }
  if (
    lap.start_elevation_m !== null &&
    lap.start_elevation_m !== undefined &&
    lap.end_elevation_m !== null &&
    lap.end_elevation_m !== undefined
  ) {
    return lap.end_elevation_m - lap.start_elevation_m
  }
  return null
}

/* --------------------------------------------------------------- route map */

function RouteHero({
  points,
  activityKey,
}: {
  points: [number, number][]
  activityKey: number
}) {
  const geometry = useMemo(() => {
    const clean = points.filter(
      (p) => Array.isArray(p) && Number.isFinite(p[0]) && Number.isFinite(p[1]),
    ) as [number, number][]

    if (clean.length === 0) {
      return { positions: [] as [number, number][], bounds: null, center: null }
    }

    const bounds: LatLngBounds = latLngBounds(clean)
    const centre = bounds.getCenter()

    return {
      positions: clean,
      bounds,
      center: [centre.lat, centre.lng] as [number, number],
    }
  }, [points])

  if (geometry.positions.length === 0) {
    return (
      <Card className={cn('flex items-center justify-center overflow-hidden', MAP_HEIGHT)}>
        <div className="max-w-sm px-6 text-center">
          <MapPin size={26} strokeWidth={1.6} className="mx-auto text-ink-faint" />
          <p className="mt-3 text-[15px] font-semibold text-ink">No GPS route</p>
          <p className="mt-1 text-[13px] text-ink-muted">
            This activity was recorded without a GPS track — a treadmill session, a manual
            entry, or a watch that did not capture position data.
          </p>
        </div>
      </Card>
    )
  }

  const start = geometry.positions[0]
  const finish = geometry.positions[geometry.positions.length - 1]

  return (
    <Card className={cn('overflow-hidden', MAP_HEIGHT)}>
      <MapContainer
        key={activityKey}
        bounds={geometry.bounds ?? undefined}
        center={geometry.center ?? undefined}
        scrollWheelZoom={false}
        zoomControl={true}
        className="h-full w-full"
        style={{ height: '100%', width: '100%' }}
      >
        <TileLayer url={TILE_URL} attribution={TILE_ATTRIBUTION} />
        {/* White casing so the track stays legible over any basemap. */}
        <Polyline
          positions={geometry.positions}
          pathOptions={{ color: '#ffffff', weight: 8, opacity: 0.95, lineCap: 'round' }}
        />
        <Polyline
          positions={geometry.positions}
          pathOptions={{ color: BRAND, weight: 4, opacity: 1, lineCap: 'round' }}
        />
        <CircleMarker
          center={start}
          radius={7}
          pathOptions={{
            color: '#ffffff',
            weight: 2,
            fillColor: '#1a9c5a',
            fillOpacity: 1,
          }}
        />
        <CircleMarker
          center={finish}
          radius={7}
          pathOptions={{
            color: '#ffffff',
            weight: 2,
            fillColor: '#d93a2b',
            fillOpacity: 1,
          }}
        />
      </MapContainer>
    </Card>
  )
}

/* --------------------------------------------------------------- title block */

function TitleBlock({
  activity,
  onSaveName,
  saving,
}: {
  activity: ActivityDetail
  onSaveName: (name: string) => void
  saving: boolean
}) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(activity.name)

  const startEditing = () => {
    setDraft(activity.name)
    setEditing(true)
  }

  const commit = () => {
    const next = draft.trim()
    setEditing(false)
    if (next && next !== activity.name) onSaveName(next)
  }

  return (
    <Card className="px-5 py-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          {editing ? (
            <div className="flex flex-wrap items-center gap-2">
              <input
                value={draft}
                autoFocus
                onChange={(event) => setDraft(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') commit()
                  if (event.key === 'Escape') setEditing(false)
                }}
                aria-label="Activity name"
                className={cn(inputClass, 'max-w-md text-[18px] font-bold')}
              />
              <Button size="sm" onClick={commit} disabled={saving}>
                <Save size={14} strokeWidth={2} />
                Save
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setEditing(false)}>
                <X size={14} strokeWidth={2} />
                Cancel
              </Button>
            </div>
          ) : (
            <div className="flex items-start gap-2">
              <h1 className="min-w-0 text-[26px] font-bold leading-tight tracking-tight text-ink">
                {activity.name}
              </h1>
              <button
                type="button"
                onClick={startEditing}
                aria-label="Rename activity"
                title="Rename activity"
                className="mt-1 shrink-0 rounded p-1 text-ink-faint transition-colors hover:bg-surface-sunken hover:text-ink-soft"
              >
                <Pencil size={14} strokeWidth={2} />
              </button>
            </div>
          )}

          <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-2 text-[13px] text-ink-muted">
            <span className="inline-flex items-center gap-1.5">
              <CalendarDays size={14} strokeWidth={2} className="text-ink-faint" />
              {formatDateLong(activity.start_time || activity.local_date)}
            </span>
            <span className="inline-flex items-center gap-1.5">
              <Timer size={14} strokeWidth={2} className="text-ink-faint" />
              {formatDateTime(activity.start_time)}
            </span>
            {activity.device ? (
              <span className="inline-flex items-center gap-1.5">
                <Watch size={14} strokeWidth={2} className="text-ink-faint" />
                {activity.device}
              </span>
            ) : null}
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <span
              className="inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide"
              style={{
                background: `${SPORT_COLOR[activity.sport_type] ?? '#fc5200'}18`,
                color: SPORT_COLOR[activity.sport_type] ?? '#fc5200',
              }}
            >
              <ActivityIcon size={11} strokeWidth={2.4} />
              {sportLabel(activity.sport_type)}
            </span>
            {activity.source === 'demo' ? <Badge tone="info">Demo</Badge> : null}
            {activity.source && activity.source !== 'demo' ? (
              <Badge tone="neutral">{activity.source}</Badge>
            ) : null}
            {activity.gear ? <Badge tone="neutral">{activity.gear}</Badge> : null}
            {activity.is_trail ? <Badge tone="warn">Trail</Badge> : null}
            {activity.is_commute ? <Badge tone="neutral">Commute</Badge> : null}
          </div>
        </div>
      </div>

      {activity.description ? (
        <>
          <Divider className="my-4" />
          <p className="text-[14px] leading-relaxed text-ink-soft">{activity.description}</p>
        </>
      ) : null}
    </Card>
  )
}

/* ---------------------------------------------------------------- stat strip */

function StatStrip({
  activity,
  fingerprint,
}: {
  activity: ActivityDetail
  fingerprint?: Fingerprint
}) {
  const hrDrift = fingerprint?.hr_drift_bpm
  const paceDrift = fingerprint?.pace_drift_pct
  const signed = (value: number) => `${value > 0 ? '+' : ''}${formatNumber(value, 0)}`
  return (
    <Card className="px-5 py-4">
      <div className="grid grid-cols-2 gap-x-5 gap-y-5 sm:grid-cols-4 lg:grid-cols-8">
        <StatTile
          label="Distance"
          value={formatDistanceShort(activity.distance_m)}
          unit="km"
          tone="brand"
          icon={<RouteIcon size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={formatDistance(activity.distance_m)}
        />
        <StatTile
          label="Pace"
          value={formatPace(activity.pace_s_per_km)}
          unit="/km"
          icon={<Gauge size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={`${formatNumber(activity.avg_speed_ms ? activity.avg_speed_ms * 3.6 : null, 1)} km/h`}
        />
        <StatTile
          label="Elevation"
          value={formatElevation(activity.elevation_gain_m)}
          icon={<Mountain size={12} strokeWidth={2} className="text-ink-faint" />}
          hint="climb"
        />
        <StatTile
          label="Moving time"
          value={formatDuration(activity.moving_time_s)}
          icon={<Timer size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={`elapsed ${formatDuration(activity.elapsed_time_s)}`}
        />
        <StatTile
          label="HR drift"
          value={hrDrift == null ? '—' : signed(hrDrift)}
          unit={hrDrift == null ? undefined : 'bpm'}
          icon={<Flame size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={
            `${hrDrift == null ? '' : hrDrift > 0 ? 'Heart rate rose across the run' : hrDrift < 0 ? 'Heart rate fell across the run' : 'Heart rate held steady across the run'}${activity.avg_hr != null ? `${hrDrift == null ? '' : ' · '}avg ${formatHr(activity.avg_hr)}${activity.max_hr != null ? ` · peak ${formatHr(activity.max_hr)}` : ''} bpm` : ''}` ||
            'No running HR data'
          }
        />
        <StatTile
          label="Pace drift"
          value={
            paceDrift == null ? '—' : `${paceDrift > 0 ? '+' : ''}${formatNumber(paceDrift, 1)}`
          }
          unit={paceDrift == null ? undefined : '%'}
          icon={<Gauge size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={
            paceDrift == null
              ? 'No running pace data'
              : `${Math.abs(paceDrift).toFixed(0)}% ${paceDrift > 0 ? 'slower' : 'faster'} in the second half`
          }
        />
        <StatTile
          label="Calories"
          value={formatNumber(activity.calories)}
          unit="kcal"
          icon={<Zap size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={
            activity.perceived_exertion
              ? `RPE ${formatNumber(activity.perceived_exertion)}`
              : undefined
          }
        />
        <StatTile
          label="Avg cadence"
          value={activity.avg_cadence === null ? '—' : formatNumber(activity.avg_cadence)}
          unit="spm"
          icon={<Footprints size={12} strokeWidth={2} className="text-ink-faint" />}
          hint={
            activity.max_cadence ? `max ${formatNumber(activity.max_cadence)} spm` : undefined
          }
        />
      </div>
    </Card>
  )
}

/* ------------------------------------------------------------------- splits */

type RunningLap = Lap & { elapsed_s?: number | null; running_distance_m?: number | null }

function SplitsTable({ laps }: { laps: RunningLap[] }) {
  const paces = laps
    .map((lap) => lap.pace_s_per_km)
    .filter((p): p is number => p !== null && p !== undefined && p > 0)

  const fastest = paces.length ? Math.min(...paces) : null
  const slowest = paces.length ? Math.max(...paces) : null
  const singleLap = paces.length < 2

  return (
    <Card className="overflow-hidden">
      <CardHeader
        title="Splits"
        subtitle={`${laps.length} lap${laps.length === 1 ? '' : 's'}`}
      />
      <div className="overflow-x-auto">
        <table className="w-full min-w-[640px] border-collapse text-left">
          <thead>
            <tr className="border-y border-edge bg-surface-soft text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
              <th className="px-5 py-2.5">Lap</th>
              <th className="px-3 py-2.5">Distance</th>
              <th className="px-3 py-2.5">Pace</th>
              <th className="px-3 py-2.5">Time</th>
              <th className="px-3 py-2.5">Elevation</th>
              <th className="px-3 py-2.5">Avg HR</th>
              <th className="px-5 py-2.5 text-right">Cadence</th>
            </tr>
          </thead>
          <tbody>
            {laps.map((lap) => {
              const isFastest =
                !singleLap &&
                lap.pace_s_per_km !== null &&
                lap.pace_s_per_km !== undefined &&
                lap.pace_s_per_km === fastest
              const isSlowest =
                !singleLap &&
                lap.pace_s_per_km !== null &&
                lap.pace_s_per_km !== undefined &&
                lap.pace_s_per_km === slowest
              const elevation = lapElevation(lap)

              return (
                <tr
                  key={lap.lap_index}
                  className={cn(
                    'border-b border-edge text-[13px] text-ink last:border-b-0',
                    isFastest && 'bg-brand-50',
                    isSlowest && 'bg-surface-sunken',
                  )}
                >
                  <td className="px-5 py-2.5 font-semibold text-ink">
                    <span className="inline-flex items-center gap-2">
                      {lap.lap_index}
                      {isFastest ? (
                        <Badge tone="brand" className="font-semibold">
                          Fastest
                        </Badge>
                      ) : null}
                      {isSlowest ? (
                        <Badge tone="neutral" className="font-semibold">
                          Slowest
                        </Badge>
                      ) : null}
                    </span>
                  </td>
                  <td className="px-3 py-2.5">
                    {lap.running_distance_m != null && lap.running_distance_m < 980
                      ? `${(lap.running_distance_m / 1000).toFixed(2)} km run`
                      : (lap.distance_label ?? formatDistance(lap.distance_m))}
                  </td>
                  <td className="px-3 py-2.5 font-semibold text-ink">
                    {lap.pace_label ?? formatPace(lap.pace_s_per_km)}
                  </td>
                  <td className="px-3 py-2.5 text-ink-soft">
                    <div>{lap.duration_label ?? formatDuration(lap.duration_s)}</div>
                    {lap.elapsed_s != null && lap.elapsed_s - lap.duration_s > 2 ? (
                      <div className="text-[11px] text-ink-faint">
                        +{formatDuration(lap.elapsed_s - lap.duration_s)} elapsed
                      </div>
                    ) : null}
                  </td>
                  <td className="px-3 py-2.5 text-ink-soft">
                    {elevation === null ? (
                      '—'
                    ) : (
                      <span className="inline-flex items-center gap-1">
                        <Mountain size={12} strokeWidth={2} className="text-ink-faint" />
                        {formatElevation(Math.abs(elevation))}
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2.5 text-ink-soft">
                    {lap.avg_hr === null || lap.avg_hr === undefined
                      ? '—'
                      : `${formatHr(lap.avg_hr)} bpm`}
                  </td>
                  <td className="px-5 py-2.5 text-right text-ink-soft">
                    {lap.avg_cadence === null || lap.avg_cadence === undefined
                      ? '—'
                      : `${formatNumber(lap.avg_cadence)} spm`}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </Card>
  )
}

/* -------------------------------------------------------------- hr zone table */

function HrZonesTable({ activity }: { activity: ActivityDetail }) {
  const total = activity.hr_zones.reduce((sum, zone) => sum + (zone.time_s || 0), 0)

  return (
    <Card className="overflow-hidden">
      <CardHeader title="Heart-rate zones" subtitle={`${formatDuration(total)} in zones`} />
      <table className="w-full border-collapse text-left">
        <thead>
          <tr className="border-y border-edge bg-surface-soft text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
            <th className="px-5 py-2.5">Zone</th>
            <th className="px-3 py-2.5">Range</th>
            <th className="px-3 py-2.5">Time</th>
            <th className="px-5 py-2.5 text-right">Share</th>
          </tr>
        </thead>
        <tbody>
          {activity.hr_zones.map((zone) => {
            const share = total > 0 ? ((zone.time_s || 0) / total) * 100 : 0
            return (
              <tr
                key={zone.zone_no}
                className="border-b border-edge text-[13px] last:border-b-0"
              >
                <td className="px-5 py-2.5 font-semibold text-ink">
                  Z{zone.zone_no} {zone.name}
                </td>
                <td className="px-3 py-2.5 text-ink-soft">
                  {zone.low_bpm !== null && zone.high_bpm !== null
                    ? `${zone.low_bpm}–${zone.high_bpm} bpm`
                    : zone.low_bpm !== null
                      ? `${zone.low_bpm}+ bpm`
                      : '—'}
                </td>
                <td className="px-3 py-2.5 text-ink-soft">
                  {zone.time_label ?? formatDuration(zone.time_s)}
                </td>
                <td className="px-5 py-2.5 text-right text-ink-soft">{share.toFixed(1)}%</td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </Card>
  )
}

/* ----------------------------------------------------------------- notes */

function NotesSection({
  activity,
  onSave,
  saving,
}: {
  activity: ActivityDetail
  onSave: (description: string) => void
  saving: boolean
}) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(activity.description ?? '')

  const start = () => {
    setDraft(activity.description ?? '')
    setEditing(true)
  }

  const commit = () => {
    setEditing(false)
    if (draft.trim() !== (activity.description ?? '')) onSave(draft.trim())
  }

  return (
    <Card className="px-5 py-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-[15px] font-semibold leading-tight text-ink">Description</h3>
          <p className="mt-0.5 text-[13px] text-ink-muted">
            How it felt, the route, the weather — the details you will want later.
          </p>
        </div>
        {!editing ? (
          <Button variant="secondary" size="sm" onClick={start}>
            <Pencil size={14} strokeWidth={2} />
            {activity.description ? 'Edit' : 'Add notes'}
          </Button>
        ) : null}
      </div>

      {editing ? (
        <div className="mt-4">
          <textarea
            value={draft}
            autoFocus
            rows={5}
            onChange={(event) => setDraft(event.target.value)}
            placeholder="How did it go?"
            className={cn(inputClass, 'resize-y leading-relaxed')}
          />
          <div className="mt-3 flex items-center gap-2">
            <Button size="sm" onClick={commit} disabled={saving}>
              <Save size={14} strokeWidth={2} />
              Save
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setEditing(false)}>
              <X size={14} strokeWidth={2} />
              Cancel
            </Button>
          </div>
        </div>
      ) : activity.description ? (
        <p className="mt-4 whitespace-pre-line text-[14px] leading-relaxed text-ink-soft">
          {activity.description}
        </p>
      ) : (
        <p className="mt-4 text-[13px] text-ink-faint">
          No description yet — add one to remember how this run went.
        </p>
      )}
    </Card>
  )
}

/* ------------------------------------------------------------- neighbours */

function NeighbourLink({
  activity,
  direction,
}: {
  activity: ActivitySummary
  direction: 'previous' | 'next'
}) {
  const isPrevious = direction === 'previous'
  return (
    <Link
      to={`/activities/${activity.id}`}
      className={cn(
        'flex items-center gap-3 rounded-card border border-edge bg-surface px-4 py-3 shadow-card transition-colors hover:border-brand-300 hover:bg-brand-50/40',
        isPrevious ? '' : 'flex-row-reverse text-right',
      )}
    >
      <span className="shrink-0 text-ink-faint">
        {isPrevious ? (
          <ChevronLeft size={18} strokeWidth={2} />
        ) : (
          <ChevronRight size={18} strokeWidth={2} />
        )}
      </span>
      <span className="min-w-0">
        <span className="block text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          {isPrevious ? 'Previous activity' : 'Next activity'}
        </span>
        <span className="block truncate text-[14px] font-semibold text-ink">
          {activity.name}
        </span>
        <span className="block text-[12px] text-ink-muted">
          {formatDateLong(activity.local_date)} · {formatDistanceShort(activity.distance_m)} km
        </span>
      </span>
    </Link>
  )
}

/* ------------------------------------------------------------------ loading */

function ActivitySkeleton() {
  return (
    <div className="space-y-4">
      <Skeleton className={cn('w-full rounded-card', MAP_HEIGHT)} />
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
        <Card className="space-y-3 px-5 py-5">
          <Skeleton className="h-7 w-2/3" />
          <Skeleton className="h-4 w-1/2" />
          <Skeleton className="h-4 w-1/3" />
          <Divider className="my-3" />
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            {Array.from({ length: 4 }).map((_, index) => (
              <Skeleton key={index} className="h-12 w-full" />
            ))}
          </div>
        </Card>
        <Card className="space-y-3 px-5 py-5">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-8 w-32" />
          <Skeleton className="h-4 w-40" />
        </Card>
      </div>
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {Array.from({ length: 3 }).map((_, index) => (
          <Skeleton key={index} className="h-[280px] w-full rounded-card" />
        ))}
      </div>
    </div>
  )
}

/* --------------------------------------------------------------------- page */

export function ActivityPage() {
  const { id } = useParams<{ id: string }>()
  const activityId = Number(id)
  const validId = Number.isInteger(activityId) && activityId > 0

  const queryClient = useQueryClient()

  const activityQuery = useQuery({
    queryKey: ['activity', activityId],
    queryFn: () => api.activity(activityId),
    enabled: validId,
    retry: (count, error) => {
      const status = (error as { status?: number }).status
      if (status === 404) return false
      return count < 1
    },
  })

  const fingerprintsQuery = useQuery({
    queryKey: ['fitness-fingerprint'],
    queryFn: () => api.fitnessFingerprint(),
    enabled: validId,
    retry: false,
  })

  const neighboursQuery = useQuery({
    queryKey: ['activities', 'neighbours', 200],
    queryFn: () => api.activities({ per_page: 200 }),
    staleTime: 60_000,
  })

  const patchMutation = useMutation({
    mutationFn: (patch: Record<string, unknown>) => api.patchActivity(activityId, patch),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['activity', activityId] })
    },
  })

  const activity = activityQuery.data

  const neighbours = useMemo(() => {
    const items = neighboursQuery.data?.items ?? []
    const sorted = [...items].sort((a, b) => {
      if (a.local_date === b.local_date) return a.id - b.id
      return a.local_date < b.local_date ? -1 : 1
    })
    const index = sorted.findIndex((item) => item.id === activityId)
    if (index === -1) return { previous: null, next: null }
    return {
      previous: index > 0 ? sorted[index - 1] : null,
      next: index < sorted.length - 1 ? sorted[index + 1] : null,
    }
  }, [neighboursQuery.data, activityId])

  /* ------------------------------------------------------------- states */

  if (!validId) {
    return (
      <Card>
        <EmptyState
          icon={<ActivityIcon size={22} strokeWidth={1.8} />}
          title="Activity not found"
          description="That link does not point at a valid activity."
          action={
            <Link to="/feed">
              <Button variant="secondary">Back to feed</Button>
            </Link>
          }
        />
      </Card>
    )
  }

  if (activityQuery.isLoading) {
    return <ActivitySkeleton />
  }

  if (activityQuery.isError || !activity) {
    const status = (activityQuery.error as { status?: number } | null)?.status
    const missing = status === 404

    return (
      <Card>
        <EmptyState
          icon={<ActivityIcon size={22} strokeWidth={1.8} />}
          title={missing ? 'Activity not found' : "Couldn't load this activity"}
          description={
            missing
              ? 'This activity is no longer in your history — it may have been removed by the sync.'
              : 'The activity service did not respond. Give it another go.'
          }
          action={
            <div className="flex items-center gap-2">
              <Button variant="secondary" onClick={() => activityQuery.refetch()}>
                Retry
              </Button>
              <Link to="/feed">
                <Button>Back to feed</Button>
              </Link>
            </div>
          }
        />
      </Card>
    )
  }

  /* --------------------------------------------------------------- data */

  const hrPoints = toStreamPoints(activity.streams.hr)
  const pacePoints = toPacePoints(activity.streams.pace)
  const elevationPoints = toStreamPoints(activity.streams.elevation)

  const avgHr = average(activity.streams.hr?.values ?? [])
  const avgPaceSeconds = average(activity.streams.pace?.values ?? [])
  const fingerprint = fingerprintsQuery.data?.items.find(
    (item) => item.activity_id === activityId,
  )
  const hrDrift = fingerprint?.hr_drift_bpm
  const paceDrift = fingerprint?.pace_drift_pct
  const hasExclusions = (activity.walking_time_s ?? 0) > 0 || (activity.stopped_time_s ?? 0) > 0
  const excludedParts = [
    activity.walking_time_s ? `${formatDuration(activity.walking_time_s)} walking` : null,
    activity.stopped_time_s ? `${formatDuration(activity.stopped_time_s)} standing` : null,
  ].filter((part): part is string => part !== null)

  const hasElevation = elevationPoints.some((point) => point.value !== null)

  return (
    <div className="space-y-4">
      {/* ------------------------------------------------- 1. route map */}
      <RouteHero points={activity.geo_points} activityKey={activity.id} />

      {/* ------------------------------------------ 2 + 3. title & stats */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
        <div className="space-y-4">
          <TitleBlock
            activity={activity}
            saving={patchMutation.isPending}
            onSaveName={(name) => patchMutation.mutate({ name })}
          />
          <StatStrip activity={activity} fingerprint={fingerprint} />
          {hasExclusions ? (
            <p className="px-1 text-[12px] text-ink-muted">
              <span className="font-semibold">Running only</span> ·{' '}
              {formatDistance(activity.running_distance_m)} of{' '}
              {formatDistance(activity.distance_m)} · {excludedParts.join(', ')} excluded
            </p>
          ) : null}
        </div>

        <aside className="space-y-4">
          <Card className="px-5 py-5">
            <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
              Details
            </p>
            <div className="mt-3 space-y-3">
              <div className="flex items-baseline justify-between gap-3">
                <span className="text-[13px] text-ink-muted">Avg speed</span>
                <span className="text-[15px] font-semibold text-ink">
                  {formatNumber(activity.avg_speed_ms ? activity.avg_speed_ms * 3.6 : null, 1)}{' '}
                  km/h
                </span>
              </div>
              <Divider />
              <div className="flex items-baseline justify-between gap-3">
                <span className="text-[13px] text-ink-muted">Max HR</span>
                <span className="text-[15px] font-semibold text-ink">
                  {formatHr(activity.max_hr)} bpm
                </span>
              </div>
              <Divider />
              <div className="flex items-baseline justify-between gap-3">
                <span className="text-[13px] text-ink-muted">Elapsed</span>
                <span className="text-[15px] font-semibold text-ink">
                  {formatDuration(activity.elapsed_time_s)}
                </span>
              </div>
              <Divider />
              <div className="flex items-baseline justify-between gap-3">
                <span className="text-[13px] text-ink-muted">Laps</span>
                <span className="text-[15px] font-semibold text-ink">
                  {formatNumber(activity.laps.length)}
                </span>
              </div>
              <Divider />
              <div className="flex items-baseline justify-between gap-3">
                <span className="text-[13px] text-ink-muted">GPS points</span>
                <span className="text-[15px] font-semibold text-ink">
                  {formatNumber(activity.geo_points.length)}
                </span>
              </div>
              {activity.garmin_url ? (
                <>
                  <Divider />
                  <div>
                    <span className="text-[13px] text-ink-muted">Share</span>
                    <a
                      href={activity.garmin_url}
                      target="_blank"
                      rel="noreferrer noopener"
                      className="mt-1 flex items-center gap-1 text-[13px] font-semibold text-brand-600 hover:underline"
                    >
                      Open in Garmin Connect
                      <ExternalLink size={13} strokeWidth={2.2} />
                    </a>
                    <p className="mt-1 text-[11px] leading-snug text-ink-muted">
                      Sharing points at Garmin. Who can open it is Garmin&rsquo;s call, not
                      ours.
                    </p>
                  </div>
                </>
              ) : null}
            </div>
          </Card>
        </aside>
      </div>

      {/* ------------------------------------------------------ 4. charts */}
      <section>
        <h2 className="mb-2 text-[15px] font-semibold text-ink">Analysis</h2>
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          {hrPoints.length ? (
            <StreamChart
              data={hrPoints}
              color={HR_COLOR}
              unit=" bpm"
              title="Heart rate"
              subtitle={`${hrDrift == null ? 'Heart rate drift unavailable' : `Heart rate ${hrDrift > 0 ? 'rose' : hrDrift < 0 ? 'fell' : 'held steady'} ${Math.round(Math.abs(hrDrift))} bpm across the run`} · avg ${avgHr === null ? '—' : formatHr(avgHr)}, peak ${formatHr(activity.max_hr)} · running only`}
              referenceLines={
                avgHr !== null
                  ? [{ value: avgHr, label: `avg ${formatHr(avgHr)}`, color: '#9aa0a6' }]
                  : undefined
              }
            />
          ) : (
            <Card>
              <CardHeader title="Heart rate" subtitle="No heart-rate data recorded" />
              <div className="px-5 pb-6 pt-2 text-[13px] text-ink-muted">
                This activity has no heart-rate stream.
              </div>
            </Card>
          )}

          {pacePoints.length ? (
            <StreamChart
              data={pacePoints}
              color={PACE_COLOR}
              unit=" min/km"
              title="Pace"
              subtitle={
                avgPaceSeconds !== null
                  ? `${paceDrift == null ? 'Pace drift unavailable' : `${Math.abs(paceDrift).toFixed(0)}% ${paceDrift > 0 ? 'slower' : 'faster'} in the second half`} · avg ${formatPaceWithUnit(avgPaceSeconds)} · running only`
                  : 'Running only · lower is faster'
              }
              tickFormatter={(v) => formatPace(v * 60)}
              valueFormatter={(v) => formatPaceWithUnit(v * 60)}
            />
          ) : (
            <Card>
              <CardHeader title="Pace" subtitle="No pace data recorded" />
              <div className="px-5 pb-6 pt-2 text-[13px] text-ink-muted">
                This activity has no pace stream.
              </div>
            </Card>
          )}

          {hasElevation ? (
            <StreamChart
              data={elevationPoints}
              color={ELEVATION_COLOR}
              unit=" m"
              title="Elevation"
              subtitle={`${formatElevation(activity.elevation_gain_m)} of climb`}
            />
          ) : null}
        </div>
      </section>

      {/* ------------------------------------------------------ 5. splits */}
      <section>
        <h2 className="mb-2 text-[15px] font-semibold text-ink">Splits</h2>
        {activity.laps.length ? (
          <SplitsTable laps={activity.laps as RunningLap[]} />
        ) : (
          <Card>
            <EmptyState
              icon={<RouteIcon size={20} strokeWidth={1.8} />}
              title="No lap data"
              description="This activity was recorded without per-kilometre splits."
            />
          </Card>
        )}
      </section>

      {/* -------------------------------------------------- 6. hr zones */}
      {activity.hr_zones.length ? (
        <section>
          <h2 className="mb-2 text-[15px] font-semibold text-ink">Intensity</h2>
          <HrZonesTable activity={activity} />
        </section>
      ) : null}

      {/* ------------------------------------------------- 7. description */}
      <section>
        <NotesSection
          activity={activity}
          saving={patchMutation.isPending}
          onSave={(description) => patchMutation.mutate({ description })}
        />
      </section>

      {/* --------------------------------------------- 8. prev / next nav */}
      {neighbours.previous || neighbours.next ? (
        <nav className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {neighbours.previous ? (
            <NeighbourLink activity={neighbours.previous} direction="previous" />
          ) : (
            <span />
          )}
          {neighbours.next ? (
            <NeighbourLink activity={neighbours.next} direction="next" />
          ) : null}
        </nav>
      ) : null}
    </div>
  )
}

export default ActivityPage
