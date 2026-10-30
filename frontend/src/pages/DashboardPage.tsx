/**
 * Today — what to do now, and nothing else.
 *
 * Composition:
 *   today's call (headline · reason · guardrail · the week so far)
 *   ├─ the planned session, when a prescribed plan has one (desktop)
 *   ├─ last night, in one line, linking to Body
 *   └─ the last three runs, each with what it meant
 *
 * Everything that used to sit under it (weekly and monthly volume, pace
 * trend, records, HR zones, by-sport) is on Progress and Stats. Today
 * answers one question: should I run, and how much?
 */

import { useQuery } from '@tanstack/react-query'
import { ArrowRight, HeartPulse, TrendingUp } from 'lucide-react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import type { PlanToday } from '../types_phase2'
import { formatDistance, formatPace } from '../lib/format'
import { ActivityCard } from '../components/ActivityCard'
import { ConnectPrompt } from '../components/ConnectPrompt'
import { TodayCall, TodayCallSkeleton } from '../components/TodayCall'
import { isPhone } from '../lib/surface'
import { usePause, useWeeklyGoal } from '../lib/goal'
import { Badge, Card, CardHeader, PageHeader, Skeleton } from '../components/ui'

function greeting(name: string | null): string {
  const hour = new Date().getHours()
  const part = hour < 12 ? 'Good morning' : hour < 18 ? 'Good afternoon' : 'Good evening'
  const first = name?.trim().split(/\s+/)[0]
  return first ? `${part}, ${first}` : part
}

/** The verdict words a prescribed plan uses, for the badge. */
const VERDICT_WORDS: Record<string, string> = {
  hit: 'Done',
  missed_pace: 'Off pace',
  short: 'Short',
  missed: 'Not run',
  substituted: 'Swapped',
  long: 'Longer than planned',
  not_yet: 'Not yet',
}

/**
 * The planned session, when a written plan has one today. Desktop only: the
 * phone has no plan, and its call already says what to do.
 */
function PlannedSession({ data }: { data: PlanToday }) {
  const session = data.session
  if (!session) return null
  const result = session.result
  const label = session.title || session.session_type
  return (
    <Card className="mb-5 p-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
            On your plan
          </p>
          <p className="mt-1 flex flex-wrap items-center gap-2 text-[17px] font-semibold capitalize text-ink">
            {label}
            {result ? (
              <Badge
                tone={result.verdict === 'hit' ? 'positive' : 'neutral'}
                className="normal-case tracking-normal"
              >
                {VERDICT_WORDS[result.verdict] ?? result.verdict}
              </Badge>
            ) : null}
          </p>
          <p className="mt-1 text-[13px] text-ink-muted">
            {session.target_distance_m != null
              ? `${formatDistance(session.target_distance_m)} planned`
              : 'No distance target'}
            {session.pace_low_s_km != null && session.pace_high_s_km != null
              ? ` · ${formatPace(session.pace_low_s_km)}–${formatPace(session.pace_high_s_km)}/km`
              : ''}
          </p>
          <p className="mt-2 text-[14px] text-ink">{data.recommendation}</p>
        </div>
        <Link
          to="/plan"
          className="inline-flex items-center gap-1 text-[13px] font-semibold text-brand-600 hover:underline"
        >
          Full plan
          <ArrowRight size={13} strokeWidth={2.2} />
        </Link>
      </div>
    </Card>
  )
}

export function DashboardPage() {
  const [goalKm] = useWeeklyGoal()
  const [pause] = usePause()
  const phone = isPhone()

  const todayQuery = useQuery({
    queryKey: ['insights-today'],
    queryFn: api.insightsToday,
    retry: false,
    staleTime: 60_000,
  })
  const summaryQuery = useQuery({
    queryKey: ['summary'],
    queryFn: api.summary,
    staleTime: 60_000,
  })
  const chipsQuery = useQuery({
    queryKey: ['insights-chips', 60],
    queryFn: () => api.insightsChips(60),
    retry: false,
    staleTime: 60_000,
  })
  const authQuery = useQuery({
    queryKey: ['auth'],
    queryFn: api.authStatus,
    staleTime: 300_000,
  })
  const planTodayQuery = useQuery({
    queryKey: ['plan-today'],
    queryFn: api.planToday,
    retry: false,
    enabled: !phone,
  })

  const today = todayQuery.data
  const recent = (summaryQuery.data?.recent ?? []).slice(0, 3)
  const chips = chipsQuery.data?.items ?? {}
  const athlete = authQuery.data?.athlete as Record<string, unknown> | undefined
  const name =
    (athlete?.full_name as string | undefined) || (athlete?.name as string | undefined) || null
  const normal = today?.recovery_normal

  return (
    <div className="mx-auto max-w-3xl">
      <PageHeader title={greeting(name)} />

      {todayQuery.isLoading ? (
        <TodayCallSkeleton />
      ) : today ? (
        today.state === 'empty' ? (
          <Card className="mb-5">
            <ConnectPrompt what="your runs" />
          </Card>
        ) : (
          <TodayCall data={today} pause={pause} goalKm={goalKm} />
        )
      ) : null}

      {!phone && planTodayQuery.data?.has_session ? (
        <PlannedSession data={planTodayQuery.data} />
      ) : null}

      {normal ? (
        <Link to="/physiology">
          <Card className="mb-5 flex items-center gap-3 px-5 py-3 transition-shadow hover:shadow-lift">
            <HeartPulse size={18} className="text-brand-600" />
            <p className="flex-1 text-[14px] text-ink">
              Last night: {normal[0]} of {normal[1]} recovery signals in your normal range
            </p>
            <ArrowRight size={14} className="text-ink-faint" />
          </Card>
        </Link>
      ) : null}

      <section>
        <CardHeader
          className="px-0 pt-0"
          title="Recent runs"
          action={
            <Link
              to="/feed"
              className="text-[13px] font-semibold text-brand-600 hover:text-brand-700"
            >
              All runs
            </Link>
          }
        />
        {summaryQuery.isLoading ? (
          <div className="space-y-3">
            {Array.from({ length: 3 }).map((_, index) => (
              <Skeleton key={index} className="h-[104px] w-full rounded-card" />
            ))}
          </div>
        ) : recent.length === 0 ? (
          today?.state === 'empty' ? null : (
            <Card>
              <ConnectPrompt what="your runs" />
            </Card>
          )
        ) : (
          <div className="space-y-3">
            {recent.map((activity) => (
              <ActivityCard
                key={activity.id}
                activity={activity}
                showRoute={false}
                chip={chips[String(activity.id)]}
              />
            ))}
          </div>
        )}
      </section>

      <Link
        to="/progress"
        className="mt-6 inline-flex items-center gap-1.5 text-[13px] font-semibold text-brand-600 hover:underline"
      >
        <TrendingUp size={14} />
        See your progress
      </Link>
    </div>
  )
}
