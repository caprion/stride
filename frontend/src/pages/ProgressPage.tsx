/**
 * Progress — proof that the running is working.
 *
 * In order: the fitness headline (pace at the same heart rate, now against
 * six weeks ago), forgiving week-by-week consistency, best runs, race
 * ranges, milestones, and links to the deeper screens (Stats, Form).
 *
 * Every comparison is against the athlete's past self. Nothing here is red,
 * and a quiet week is a grace week, not a broken streak.
 */

import { useQuery } from '@tanstack/react-query'
import { ArrowRight, Award, Flame, Gauge, Medal, TrendingUp, Trophy } from 'lucide-react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { formatDate, formatDistanceShort, formatDuration, formatPace } from '../lib/format'
import type { BestRun, ConsistencyWeek } from '../types_insights'
import { Card, CardHeader, EmptyState, PageHeader, Skeleton, cn } from '../components/ui'
import { ConnectPrompt } from '../components/ConnectPrompt'

const WEEK_COLOUR: Record<ConsistencyWeek['status'], string> = {
  active: 'bg-brand-500',
  grace: 'bg-brand-200',
  gap: 'bg-surface-sunken',
  current: 'border border-dashed border-brand-400',
}

function ConsistencyGrid({ weeks }: { weeks: ConsistencyWeek[] }) {
  return (
    <div className="flex flex-wrap gap-1" aria-label="The last 52 weeks">
      {weeks.map((week) => (
        <span
          key={week.start}
          title={`Week of ${week.start}: ${week.runs} run${week.runs === 1 ? '' : 's'}, ${formatDistanceShort(week.distance_m)} km${week.status === 'grace' ? ' (grace week)' : ''}`}
          className={cn('h-3.5 w-3.5 rounded-sm', WEEK_COLOUR[week.status])}
        />
      ))}
    </div>
  )
}

function BestTile({ best }: { best: BestRun }) {
  return (
    <Link
      to={`/activities/${best.activity_id}`}
      className="group bg-surface px-4 py-3 transition-colors hover:bg-surface-soft"
    >
      <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
        {best.label}+ {best.scope === 'this_year' ? 'this year' : 'all time'}
      </p>
      <p className="mt-1 text-[19px] font-semibold leading-none text-ink group-hover:text-brand-600">
        {formatPace(best.pace_s_km)}
        <span className="ml-1 text-[12px] font-normal text-ink-muted">/km</span>
      </p>
      <p className="mt-1.5 text-[12px] text-ink-muted">
        {formatDistanceShort(best.run_distance_m)} km · {formatDate(best.date)}
      </p>
    </Link>
  )
}

export function ProgressPage() {
  const query = useQuery({
    queryKey: ['insights-progress'],
    queryFn: api.insightsProgress,
    retry: false,
    staleTime: 60_000,
  })
  const data = query.data

  if (query.isLoading) {
    return (
      <div className="mx-auto max-w-3xl space-y-4">
        <PageHeader title="Progress" />
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-32 w-full rounded-card" />
        ))}
      </div>
    )
  }

  if (!data || data.milestones.total_runs === 0) {
    return (
      <div className="mx-auto max-w-3xl">
        <PageHeader title="Progress" />
        <Card>
          <ConnectPrompt what="your progress" />
        </Card>
      </div>
    )
  }

  const { fitness, consistency, milestones } = data
  const bests = data.bests.filter(
    (best) =>
      best.scope === 'all_time' ||
      !data.bests.some(
        (other) =>
          other.scope === 'all_time' &&
          other.label === best.label &&
          other.activity_id === best.activity_id,
      ),
  )
  const nextTier = milestones.next_tier_km
  const totalKm = milestones.total_m / 1000

  return (
    <div className="mx-auto max-w-3xl space-y-5">
      <PageHeader title="Progress" description="You against your past self, and nobody else." />

      {/* ------------------------------------------------------- fitness */}
      <Card className="p-5">
        <p className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          <TrendingUp size={13} /> Fitness
        </p>
        {fitness ? (
          <>
            <p className="mt-1 text-[20px] font-bold leading-tight tracking-tight text-ink">
              {fitness.headline}
            </p>
            <p className="mt-2 text-[12px] text-ink-muted">
              Median of {fitness.recent_runs} runs in the last six weeks against{' '}
              {fitness.earlier_runs} in the six before. Heat, hills and wrist-HR glitches move
              single runs, so this only reads the trend.
            </p>
          </>
        ) : (
          <p className="mt-1 text-[15px] text-ink-soft">
            Four runs with heart rate in each of the last two six-week stretches and this
            becomes a fitness trend.
          </p>
        )}
      </Card>

      {/* --------------------------------------------------- consistency */}
      <Card className="p-5">
        <p className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          <Flame size={13} /> Consistency
        </p>
        <p className="mt-1 text-[20px] font-bold leading-tight tracking-tight text-ink">
          {consistency.current_weeks > 0
            ? `${consistency.current_weeks} week${consistency.current_weeks === 1 ? '' : 's'} in a row`
            : 'A new run of weeks starts with your next run'}
        </p>
        <p className="mt-1 text-[13px] text-ink-muted">
          A week counts with {consistency.runs_per_week} run
          {consistency.runs_per_week === 1 ? '' : 's'}.{' '}
          {consistency.grace_available
            ? 'One quiet week in six is a grace week and keeps the streak.'
            : 'Grace week used: the next quiet week ends the run.'}{' '}
          Best ever: {consistency.best_weeks} weeks.
        </p>
        <div className="mt-4">
          <ConsistencyGrid weeks={consistency.weeks} />
        </div>
        <p className="mt-2 text-[11px] text-ink-faint">
          {consistency.active_weeks_this_year} weeks this year · light squares are grace weeks
        </p>
      </Card>

      {/* --------------------------------------------------------- bests */}
      <Card>
        <CardHeader
          title={
            <span className="flex items-center gap-1.5">
              <Trophy size={15} /> Best runs
            </span>
          }
          subtitle="Fastest average pace over at least each distance"
        />
        {bests.length === 0 ? (
          <EmptyState
            icon={<Medal size={22} strokeWidth={1.8} />}
            title="No 5K runs yet"
            description="Run 5 km or more and your bests start here."
          />
        ) : (
          <div className="grid grid-cols-2 gap-px overflow-hidden rounded-b-card bg-edge">
            {bests.map((best) => (
              <BestTile key={`${best.label}-${best.scope}`} best={best} />
            ))}
          </div>
        )}
      </Card>

      {/* --------------------------------------------------- race ranges */}
      {data.race_estimates.length > 0 ? (
        <Card className="p-5">
          <p className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
            <Gauge size={13} /> Race range today
          </p>
          <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-4">
            {data.race_estimates.map((race) => (
              <div key={race.label}>
                <p className="text-[12px] text-ink-muted">{race.label}</p>
                <p className="text-[15px] font-semibold text-ink">
                  {formatDuration(race.low_s)}–{formatDuration(race.high_s)}
                </p>
              </div>
            ))}
          </div>
          <p className="mt-3 text-[12px] text-ink-muted">
            {data.race_basis} Longer distances assume you have trained for them.
          </p>
        </Card>
      ) : null}

      {/* ---------------------------------------------------- milestones */}
      <Card className="p-5">
        <p className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          <Award size={13} /> Milestones
        </p>
        <p className="mt-1 text-[20px] font-bold leading-tight tracking-tight text-ink">
          {Math.round(totalKm).toLocaleString()} km over {milestones.total_runs} runs
        </p>
        <p className="mt-1 text-[13px] text-ink-muted">
          {milestones.runner_since
            ? `Running since ${formatDate(milestones.runner_since)}. `
            : ''}
          Longest run {formatDistanceShort(milestones.longest_m)} km. This year{' '}
          {formatDistanceShort(milestones.this_year_m)} km in {milestones.this_year_runs} runs.
        </p>
        {nextTier ? (
          <div className="mt-3">
            <div className="h-1.5 overflow-hidden rounded-full bg-surface-sunken">
              <div
                className="h-full rounded-full bg-brand-500"
                style={{ width: `${Math.min(100, (totalKm / nextTier) * 100)}%` }}
              />
            </div>
            <p className="mt-1 text-[11px] text-ink-faint">
              {Math.round(nextTier - totalKm).toLocaleString()} km to{' '}
              {nextTier.toLocaleString()} km
            </p>
          </div>
        ) : null}
      </Card>

      {/* ------------------------------------------------------- deeper */}
      <div className="flex flex-wrap gap-4 pb-4">
        <Link
          to="/stats"
          className="inline-flex items-center gap-1 text-[13px] font-semibold text-brand-600 hover:underline"
        >
          All stats and volume <ArrowRight size={13} />
        </Link>
        <Link
          to="/fitness"
          className="inline-flex items-center gap-1 text-[13px] font-semibold text-brand-600 hover:underline"
        >
          Running form <ArrowRight size={13} />
        </Link>
      </div>
    </div>
  )
}
