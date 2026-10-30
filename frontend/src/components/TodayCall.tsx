/**
 * Today's call: one headline, one reason, one guardrail, and the week so far.
 *
 * This is the first thing on the app. It answers "should I run today, and
 * how much?" in words, from the athlete's own runs and last night's
 * recovery. No score leads it: a 0-100 number invites anxiety and argument,
 * a sentence with its reason invites a decision.
 */

import { Link } from 'react-router-dom'
import { ArrowRight, PauseCircle, ShieldCheck } from 'lucide-react'
import type { TodayInsight, WeekSoFar } from '../types_insights'
import { formatDistanceShort } from '../lib/format'
import { PAUSE_LABEL, type Pause, weeklyTarget } from '../lib/goal'
import { Badge, Card, Skeleton, cn } from './ui'
import { chipTone } from './ActivityCard'

const STATE_LABEL: Record<TodayInsight['state'], string> = {
  empty: 'Today',
  done: 'Done today',
  go: 'Today',
  easy: 'Take it easy',
  nudge: 'Today',
  comeback: 'Comeback',
}

export function TodayCallSkeleton() {
  return (
    <Card className="mb-5 p-5">
      <Skeleton className="h-3 w-20" />
      <Skeleton className="mt-3 h-7 w-3/4" />
      <Skeleton className="mt-3 h-4 w-1/2" />
    </Card>
  )
}

export function TodayCall({
  data,
  pause,
  goalKm,
}: {
  data: TodayInsight
  pause: Pause | null
  goalKm: number | null
}) {
  const last = data.last_run
  const paused = pause != null && data.state !== 'done'
  return (
    <Card className="mb-5 overflow-hidden">
      <div className="p-5">
        <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          {paused ? PAUSE_LABEL[pause.reason] : STATE_LABEL[data.state]}
        </p>
        {paused ? (
          <>
            <p className="mt-1 flex items-center gap-2 text-[22px] font-bold leading-tight tracking-tight text-ink">
              <PauseCircle size={20} className="text-ink-muted" />
              Paused until {pause.until}
            </p>
            <p className="mt-2 text-[14px] text-ink-soft">
              No nudges, and your consistency streak waits for you. Rest is part of training.
            </p>
          </>
        ) : (
          <>
            <p className="mt-1 text-[22px] font-bold leading-tight tracking-tight text-ink">
              {data.headline}
            </p>
            {data.reason ? (
              <p className="mt-2 text-[14px] leading-snug text-ink-soft">{data.reason}</p>
            ) : null}
            {data.guardrail ? (
              <p className="mt-3 flex items-start gap-2 rounded-md bg-surface-soft px-3 py-2 text-[13px] leading-snug text-ink">
                <ShieldCheck size={15} className="mt-0.5 shrink-0 text-brand-600" />
                {data.guardrail}
              </p>
            ) : null}
          </>
        )}
        {last && data.state !== 'empty' ? (
          <Link
            to={`/activities/${last.run.id}`}
            className="mt-4 flex items-center gap-2 text-[13px] text-ink-muted hover:text-brand-600"
          >
            <span className="truncate">
              Last run: {last.run.name} · {formatDistanceShort(last.run.distance_m)} km ·{' '}
              {last.run.date}
            </span>
            {last.story.chip !== 'Usual run' ? (
              <Badge
                tone={chipTone(last.story.tone)}
                className="shrink-0 normal-case tracking-normal"
              >
                {last.story.chip}
              </Badge>
            ) : null}
            <ArrowRight size={13} className="shrink-0" />
          </Link>
        ) : null}
      </div>
      {data.state !== 'empty' ? <WeekStrip week={data.week} goalKm={goalKm} /> : null}
    </Card>
  )
}

const DAY_LETTERS = ['M', 'T', 'W', 'T', 'F', 'S', 'S']

/**
 * The week as seven dots and one bar. The bar measures against the
 * athlete's goal when they set one, otherwise their usual week, and says
 * which. It never shows red and never says "missed".
 */
export function WeekStrip({ week, goalKm }: { week: WeekSoFar; goalKm: number | null }) {
  const target = weeklyTarget(goalKm, week.usual_distance_m)
  const km = week.distance_m / 1000
  const pct = target ? Math.min(100, (km / target.km) * 100) : 0
  const close = target ? pct >= 80 : false
  return (
    <div className="border-t border-edge px-5 py-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex gap-1.5">
          {week.days.map((day, index) => (
            <div key={day.date} className="flex flex-col items-center gap-1">
              <span
                title={day.ran ? `${formatDistanceShort(day.distance_m)} km` : day.date}
                className={cn(
                  'h-3 w-3 rounded-full',
                  day.ran
                    ? 'bg-brand-500'
                    : day.future
                      ? 'border border-dashed border-edge'
                      : 'bg-surface-sunken',
                )}
              />
              <span className="text-[10px] text-ink-faint">{DAY_LETTERS[index]}</span>
            </div>
          ))}
        </div>
        <div className="min-w-0 flex-1 text-right">
          <p className="text-[15px] font-semibold text-ink">
            {formatDistanceShort(week.distance_m)} km
            <span className="ml-1 text-[12px] font-normal text-ink-muted">
              {week.runs} run{week.runs === 1 ? '' : 's'} this week
            </span>
          </p>
          {target ? (
            <p
              className={cn(
                'text-[11px]',
                close ? 'font-semibold text-brand-700' : 'text-ink-faint',
              )}
            >
              {target.source === 'goal' ? 'goal' : 'your usual week'} {target.km} km
            </p>
          ) : null}
        </div>
      </div>
      {target ? (
        <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-surface-sunken">
          <div className="h-full rounded-full bg-brand-500" style={{ width: `${pct}%` }} />
        </div>
      ) : null}
    </div>
  )
}
