/**
 * Activity card — the Strava feed row.
 *
 * Layout: athlete + title header, optional route thumbnail, then the stat
 * strip (distance / pace / elevation / time) and the sport icon.
 */

import { Link } from 'react-router-dom'
import {
  Activity as ActivityIcon,
  Clock,
  Flame,
  Heart,
  Mountain,
  Route as RouteIcon,
  Timer,
} from 'lucide-react'
import type { ActivitySummary } from '../types'
import { RouteThumb } from './RouteThumb'
import { Badge, cn } from './ui'
import {
  formatDateTime,
  formatDistanceShort,
  formatDuration,
  formatElevation,
  formatPace,
  sportLabel,
  SPORT_COLOR,
} from '../lib/format'

function Stat({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode
  label: string
  value: string
}) {
  return (
    <span className="inline-flex items-center gap-1.5 whitespace-nowrap">
      <span className="text-ink-faint">{icon}</span>
      <span className="font-semibold text-ink">{value}</span>
      <span className="hidden text-ink-muted sm:inline">{label}</span>
    </span>
  )
}

export function ActivityCard({
  activity,
  showRoute = true,
  className,
}: {
  activity: ActivitySummary
  showRoute?: boolean
  className?: string
}) {
  const color = SPORT_COLOR[activity.sport_type] ?? '#fc5200'

  return (
    <article
      className={cn(
        'overflow-hidden rounded-card border border-edge bg-surface shadow-card transition-shadow hover:shadow-lift',
        className,
      )}
    >
      <div className="flex flex-col sm:flex-row">
        <div className="min-w-0 flex-1 p-4">
          <header className="mb-3 flex items-start gap-3">
            <span
              className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-white"
              style={{ background: color }}
            >
              <ActivityIcon size={15} strokeWidth={2.4} />
            </span>
            <div className="min-w-0 flex-1">
              <Link
                to={`/activities/${activity.id}`}
                className="block truncate text-[15px] font-semibold leading-snug text-ink hover:text-brand-600"
              >
                {activity.name}
              </Link>
              <p className="mt-0.5 text-[13px] text-ink-muted">
                {formatDateTime(activity.start_time)}
                <span className="mx-1.5 text-ink-faint">·</span>
                {sportLabel(activity.sport_type)}
              </p>
            </div>
            {activity.source === 'demo' ? <Badge tone="info">Demo</Badge> : null}
          </header>

          <div className="flex flex-wrap items-center gap-x-5 gap-y-2 text-[13px]">
            <Stat
              icon={<RouteIcon size={14} />}
              label="km"
              value={formatDistanceShort(activity.distance_m)}
            />
            <Stat
              icon={<Timer size={14} />}
              label="/km"
              value={`${formatPace(activity.pace_s_per_km)}`}
            />
            <Stat
              icon={<Mountain size={14} />}
              label="elev"
              value={formatElevation(activity.elevation_gain_m)}
            />
            <Stat
              icon={<Clock size={14} />}
              label="time"
              value={formatDuration(activity.moving_time_s)}
            />
            {activity.avg_hr ? (
              <Stat icon={<Heart size={14} />} label="bpm" value={String(activity.avg_hr)} />
            ) : null}
            {activity.calories ? (
              <Stat icon={<Flame size={14} />} label="kcal" value={String(activity.calories)} />
            ) : null}
          </div>
        </div>

        {showRoute && activity.has_geo ? (
          <Link
            to={`/activities/${activity.id}`}
            className="block shrink-0 border-t border-edge sm:w-[190px] sm:border-l sm:border-t-0"
            aria-label={`View ${activity.name} route`}
          >
            <RouteThumb points={activity.route_preview} className="h-28 w-full sm:h-full sm:min-h-[132px]" />
          </Link>
        ) : null}
      </div>
    </article>
  )
}

/** Compact variant used in the calendar day cells and sidebars. */
export function ActivityRow({
  activity,
  className,
}: {
  activity: ActivitySummary
  className?: string
}) {
  return (
    <Link
      to={`/activities/${activity.id}`}
      className={cn(
        'flex items-center gap-3 rounded-md px-2 py-1.5 transition-colors hover:bg-surface-soft',
        className,
      )}
    >
      <span
        className="h-2 w-2 shrink-0 rounded-full"
        style={{ background: SPORT_COLOR[activity.sport_type] ?? '#fc5200' }}
      />
      <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-ink">
        {activity.name}
      </span>
      <span className="shrink-0 text-[12px] text-ink-muted">
        {formatDistanceShort(activity.distance_m)} km
      </span>
    </Link>
  )
}
