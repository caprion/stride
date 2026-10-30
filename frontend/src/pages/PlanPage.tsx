import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { ACCENT_DATA, GRID, INK_MUTED, RAMP, SERIES, ZONE_COLORS } from '../lib/palette'
import type { PlanSessionWithResult, Verdict } from '../types_phase2'
import {
  formatDate,
  formatDistance,
  formatDistanceShort,
  formatPace,
  formatWeekKey,
  sessionTypeLabel,
} from '../lib/format'
import {
  Card,
  CardHeader,
  Badge,
  Button,
  EmptyState,
  PageHeader,
  Skeleton,
  StatTile,
  inputClass,
} from '../components/ui'
import { ChartFrame } from '../components/charts'
import { isPhone } from '../lib/surface'
import {
  inferredSource,
  isFutureSession,
  prescribedSource,
  verdictLabel,
  verdictTone,
} from '../lib/planSources'
import type { PlanSource, PlanSourceResult } from '../types_phase2'

const verdictNames: Record<Verdict, string> = {
  hit: 'Hit',
  missed_pace: 'Missed pace',
  short: 'Short',
  missed: 'Not run',
  substituted: 'Substituted',
  long: 'Longer than planned',
  bonus: 'Bonus',
  // A session that has not happened is not a failure, and it is not red.
  not_yet: 'Not yet',
}
const verdictColors: Record<Verdict, string> = {
  hit: ZONE_COLORS[1],
  missed_pace: ZONE_COLORS[3],
  short: ZONE_COLORS[3],
  missed: ZONE_COLORS[0],
  substituted: RAMP[1],
  long: ZONE_COLORS[3],
  bonus: ACCENT_DATA,
  // Grey: a session that has not happened yet.
  not_yet: ZONE_COLORS[0],
}
const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
// The weekday names, in the order the week runs.
const todayKey = () => new Date().toISOString().slice(0, 10)
const pace = (v: number | null | undefined) => (v == null ? '—' : `${formatPace(v)}/km`)
const sessionLabel = (s: PlanSessionWithResult) => s.title || sessionTypeLabel(s.session_type)

/**
 * A date, shifted by days.
 *
 * The dates are ISO days, so the shift is done in UTC: a session on
 * "2026-10-26" is that day wherever the phone is.
 */
function addDays(iso: string, days: number): string {
  const date = new Date(`${iso}T00:00:00Z`)
  date.setUTCDate(date.getUTCDate() + days)
  return date.toISOString().slice(0, 10)
}

/** The Monday of the week a date falls in. */
function startOfWeek(iso: string): string {
  const date = new Date(`${iso}T00:00:00Z`)
  const offset = (date.getUTCDay() + 6) % 7
  return addDays(iso, -offset)
}

/** One session, in the shape the screen draws. */
interface DisplaySession {
  key: string
  date: string
  label: string
  targetDistance: number | null
  paceLow: number | null
  paceHigh: number | null
  activity: {
    id: number
    name: string
    distance_m: number
    pace_s_per_km: number | null
  } | null
  result: PlanSourceResult | null
}

/** A session from the database, in the shape the screen draws. */
function displaySession(session: PlanSessionWithResult, index: number): DisplaySession {
  return {
    key: `db-${session.id}-${index}`,
    date: session.date,
    label: session.title || sessionTypeLabel(session.session_type),
    targetDistance: session.target_distance_m,
    paceLow: session.pace_low_s_km,
    paceHigh: session.pace_high_s_km,
    activity: session.activity
      ? {
          id: session.activity.id,
          name: session.activity.name,
          distance_m: session.activity.distance_m,
          pace_s_per_km: session.activity.pace_s_per_km,
        }
      : null,
    result: session.result ? { ...session.result } : null,
  }
}

/** A session from a plan source, in the shape the screen draws. */
function displaySourceSession(
  session: NonNullable<PlanSource['sessions']>[number],
  index: number,
): DisplaySession {
  return {
    key: `src-${session.date}-${session.title ?? session.session_type}-${index}`,
    date: session.date,
    label: session.title || sessionTypeLabel(session.session_type),
    targetDistance: session.target_distance_m,
    paceLow: session.pace_low_s_km,
    paceHigh: session.pace_high_s_km,
    activity: session.activity ?? null,
    result: session.result
      ? { ...session.result }
      : session.verdict
        ? {
            verdict: session.verdict,
            verdict_label: session.verdict_label ?? null,
            message: session.message ?? null,
          }
        : null,
  }
}

/** Render an explanation, as the engine wrote it. */
const renderCause = (r: { cause?: string | null; cause_confidence?: number | null }) =>
  // The engine's sentences already carry their own confidence ("Likely
  // influenced by…", "No measured factor explains this one."), so they are
  // shown as written. Wrapping them in "Might be …?" produced broken English.
  r.cause ?? ''

/**
 * The verdict of a session, as a badge.
 *
 * **A session that has not happened yet reads "Not yet", in a neutral
 * badge.** A session that has not happened is not a failure, whatever the
 * verdict says — and the badge is grey, never red.
 */
function VerdictBadge({
  date,
  result,
  className,
}: {
  date: string
  result: PlanSourceResult | null
  className?: string
}) {
  if (isFutureSession(date, todayKey())) {
    return (
      <Badge tone="neutral" className={className}>
        Not yet
      </Badge>
    )
  }
  if (!result) return <Badge className={className}>Pending</Badge>
  return (
    <Badge tone={verdictTone(result.verdict)} className={className}>
      {verdictLabel(result)}
    </Badge>
  )
}

export default function PlanPage() {
  const [weekDate, setWeekDate] = useState<string | null>(null)
  const [format, setFormat] = useState('markdown')
  const [content, setContent] = useState('')
  const [importMessage, setImportMessage] = useState<{
    warnings: string[]
    error: string | null
    name: string | null
  } | null>(null)
  const client = useQueryClient()
  const current = useQuery({
    queryKey: ['plan-current'],
    queryFn: api.planCurrentSummary,
    retry: false,
  })
  const today = useQuery({ queryKey: ['plan-today'], queryFn: api.planToday, retry: false })
  // Default to the week the plan is currently in, not to today — a plan
  // usually covers a block of weeks, and "today" is often outside it.
  const effectiveWeekDate = weekDate ?? current.data?.week?.week?.start_date ?? todayKey()

  // The window the screen talks about.
  //
  // The phone shows the near future and the recent past — the week before,
  // this week and the week after. The desktop keeps the whole block: it has
  // the room, and the block's history is worth reading there.
  const phone = isPhone()
  const todayIso = todayKey()
  const rangeStart = startOfWeek(effectiveWeekDate)
  // The three weeks the phone lists: twenty-one days, Monday to Sunday.
  const windowStart = addDays(rangeStart, -7)
  const windowEnd = addDays(rangeStart, 13)
  const week = useQuery({
    queryKey: ['plan-week', rangeStart],
    queryFn: () => api.planWeek(rangeStart),
    retry: false,
  })

  // The week before, for the phone's list of sessions.
  const previousWeek = useQuery({
    queryKey: ['plan-week', addDays(rangeStart, -7)],
    queryFn: () => api.planWeek(addDays(rangeStart, -7)),
    enabled: phone,
    retry: false,
  })

  // The week after, for the phone's list of sessions.
  const nextWeek = useQuery({
    queryKey: ['plan-week', addDays(rangeStart, 7)],
    queryFn: () => api.planWeek(addDays(rangeStart, 7)),
    enabled: phone,
    retry: false,
  })

  /**
   * The whole block, every session of it.
   *
   * The desktop keeps the full list — the block's history, with the reason
   * behind every session. The phone does not ask for it: it has three weeks
   * and no more, so it reads the trimmed summary of the block instead.
   */
  const fullPlan = useQuery({
    queryKey: ['plan-full'],
    queryFn: api.planCurrent,
    enabled: !phone,
    retry: false,
  })

  // The plan sources: the plan the athlete wrote down, and the plan they
  // already have. The route is new, so a failure here is not an error the
  // athlete sees — the screen falls back to the plan in the database.
  const sourcesQuery = useQuery({
    queryKey: ['plan-sources', phone ? windowStart : 'all', phone ? windowEnd : 'all'],
    queryFn: () =>
      api.planSources(phone ? windowStart : undefined, phone ? windowEnd : undefined),
    retry: false,
  })
  const sources = sourcesQuery.data ?? []
  const prescribed = prescribedSource(sources)
  const inferred = inferredSource(sources)
  // The plan the screen shows.
  //
  // The athlete's own imported plan comes first: they wrote it down. Then
  // Garmin's scheduled workouts. And the inferred plan — the usual week
  // their history already holds — when there is nothing else.
  //
  // Never two of them at once: they are judged with different words, and
  // mixing them would say something false.
  const hasImportedPlan = Boolean(current.data?.block)
  const garminSource = sources.find((source) => source.source === 'garmin') ?? null
  const active = hasImportedPlan ? null : (prescribed ?? inferred)
  const baseline = active && !active.prescribed ? active : null
  // The Garmin source may not be able to answer at all: it needs an
  // account. That is a sentence for the athlete, not an error.
  const garminReason =
    garminSource && garminSource.available === false ? garminSource.reason : null
  const adherence = useQuery({
    queryKey: ['plan-adherence', phone ? windowStart : 'all', phone ? windowEnd : 'all'],
    queryFn: () =>
      api.planAdherence(phone ? windowStart : undefined, phone ? windowEnd : undefined),
    retry: false,
  })
  const importing = useMutation({
    mutationFn: () => api.importPlan({ format, content }),
    onSuccess: (result) => {
      setImportMessage({ warnings: result.warnings, error: result.error, name: result.name })
      if (result.ok) {
        void client.invalidateQueries({ queryKey: ['plan-'] })
        void client.invalidateQueries({ queryKey: ['plan-current'] })
      }
    },
  })
  const detail = week.data
  const weekSessions = detail?.sessions ?? []
  // Every session of the block, for the desktop's full list.
  const blockSessions: DisplaySession[] = (fullPlan.data?.block?.phases ?? []).flatMap(
    (phase) => (phase.weeks ?? []).flatMap((w) => (w.sessions ?? []).map(displaySession)),
  )

  /**
   * The sessions the screen lists.
   *
   * The phone lists the week before, this week and the week after — the near
   * future and the recent past, twenty-one days. The desktop keeps the
   * whole list.
   *
   * The list carries one plan's words and one plan's ruler, never two:
   *
   * * a **prescription** — a plan somebody wrote down, or Garmin's
   *   scheduled workouts — judged *hit*, *missed*, *not yet*;
   * * a **baseline** — the athlete's own usual week, inferred from their
   *   history — judged *less than usual*, and never *missed*, because
   *   nobody wrote it down.
   */
  const listedSessions: DisplaySession[] = (
    baseline
      ? (baseline.sessions ?? []).map(displaySourceSession)
      : hasImportedPlan
        ? phone
          ? [
              ...(previousWeek.data?.sessions ?? []).map(displaySession),
              ...weekSessions.map(displaySession),
              ...(nextWeek.data?.sessions ?? []).map(displaySession),
            ]
          : // The desktop keeps the full list: the whole block. Until it has
            // arrived, the week on screen stands in for it.
            blockSessions.length
            ? blockSessions
            : weekSessions.map(displaySession)
        : // No plan was written down: Garmin's scheduled workouts are the
          // prescription, and they are what the phone lists.
          (prescribed?.sessions ?? []).map(displaySourceSession)
  )
    .slice()
    .sort((a, b) => a.date.localeCompare(b.date))
  const sessionsByDate = new Map(listedSessions.map((s) => [s.date, s]))
  // Today's session of the inferred plan, when the plan is inferred.
  const baselineToday = baseline
    ? (listedSessions.find((s) => s.date === todayIso) ?? null)
    : null
  // Whether the screen is judging Garmin's scheduled workouts rather than
  // the plan in the database. When it is, the Today card must speak about
  // those workouts and not about the plan in the database: one screen, one
  // comparison.
  const prescribedPlan = Boolean(prescribed) && !hasImportedPlan
  // Today's session of that prescription.
  const prescribedToday = prescribedPlan
    ? (listedSessions.find((s) => s.date === todayIso) ?? null)
    : null
  // The week the grid draws: the plan's week, or the calendar week the
  // screen is looking at when the plan has no week to offer.
  const start = detail?.week?.start_date ?? rangeStart
  const weekDays = start
    ? Array.from({ length: 7 }, (_, i) => {
        const d = new Date(`${start}T12:00:00`)
        d.setDate(d.getDate() + i)
        return d.toISOString().slice(0, 10)
      })
    : []
  const summary = adherence.data
  /**
   * The verdicts the panel draws.
   *
   * A session that has not happened yet is neither a hit nor a miss: it is
   * grey, and it is not in the adherence denominator. The bar is drawn
   * against the sessions that were due, so the grey never runs off the end
   * of it.
   */
  const dueVerdicts = (Object.keys(verdictNames) as Verdict[]).filter((v) => v !== 'not_yet')
  const dueRows = summary
    ? dueVerdicts.map((verdict) => ({ verdict, count: summary.by_verdict?.[verdict] ?? 0 }))
    : []
  const dueTotal = dueRows.reduce((sum, row) => sum + row.count, 0)
  // The sessions that have not happened yet, counted from the list the
  // screen shows. They are grey, and they do not count in the denominator.
  const notYetCount = listedSessions.filter(
    (s) => s.result?.verdict === 'not_yet' || (!s.result && isFutureSession(s.date, todayIso)),
  ).length
  // Whether the adherence has anything to say: a plan that was written down
  // and came due. A baseline has no missed sessions to count.
  const hasAdherence = Boolean(
    summary && (summary.planned_sessions > 0 || summary.extra_sessions > 0),
  )
  /**
   * Whether the adherence panel belongs on this screen at all.
   *
   * The adherence is measured against the plan in the database — a plan
   * somebody wrote down. When the screen is judging something else (the
   * athlete's usual week, or Garmin's scheduled workouts), those numbers
   * are about a different comparison, and two vocabularies on one screen
   * say something false.
   */
  const showsAdherence = hasImportedPlan
  // "No plan" means no plan at all: no block, and no source that could
  // offer one. A block that schedules nothing for today is a rest day, not
  // a missing plan.
  const noPlan = !current.isLoading && !current.data?.block && !current.error && !active
  // A plan that schedules nothing for today is a rest day, not a gap.
  const noSessionToday = Boolean(today.data?.has_plan) && !today.data?.has_session
  return (
    <div className="mx-auto max-w-7xl p-5">
      <PageHeader
        title={isPhone() ? 'Plan' : 'Plan vs actual'}
        description={
          current.data?.block ? (
            <>
              {current.data.block.block.name} ·{' '}
              {current.data.block.block.goal || 'Training block'} ·{' '}
              {formatDate(current.data.block.block.start_date)} –{' '}
              {current.data.block.block.end_date
                ? formatDate(current.data.block.block.end_date)
                : 'ongoing'}
            </>
          ) : (
            'See what the plan asked for, what happened, and why.'
          )
        }
        actions={
          <label className="text-xs text-ink-muted">
            Week starting{' '}
            <input
              aria-label="Week date"
              type="date"
              className={`${inputClass} mt-1`}
              value={effectiveWeekDate}
              onChange={(e) => setWeekDate(e.target.value)}
            />
          </label>
        }
      />
      {/*
        The source of the plan, and what it says about itself.
        *
        * A baseline says it is a baseline — nobody wrote it down, so
        * nothing in it can be missed. A prescription says it was written
        * down. The screen shows one of them, never both.
      */}
      {baseline ? (
        <Card className="mb-5">
          <CardHeader title="Your usual week" subtitle={baseline.description} />
          <div className="px-5 pb-5">
            {baseline.sufficient === false ? (
              <p className="text-[13px] leading-relaxed text-ink-soft">{baseline.reason}</p>
            ) : (
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                <StatTile
                  label="A usual week"
                  value={formatDistanceShort(baseline.weekly_volume_m ?? 0)}
                  unit="km"
                />
                <StatTile
                  label="Rest days"
                  value={(baseline.rest_days ?? []).join(', ') || '—'}
                />
                <StatTile label="Long run" value={baseline.long_run_day ?? '—'} />
                <StatTile label="Quality day" value={baseline.quality_day ?? '—'} />
              </div>
            )}
          </div>
        </Card>
      ) : active ? (
        <p className="mb-4 text-[13px] leading-snug text-ink-muted">{active.description}</p>
      ) : !hasImportedPlan && garminReason ? (
        /* The Garmin source needs an account. It says so, in a sentence. */
        <p className="mb-4 text-[13px] leading-snug text-ink-muted">{garminReason}</p>
      ) : null}
      {noPlan && (
        <Card className="mb-5">
          <EmptyState
            title="No plan yet"
            description={
              isPhone()
                ? 'Your Garmin workouts show up here once they sync.'
                : 'Import your training plan to compare planned sessions with your runs and understand what changed.'
            }
          />
        </Card>
      )}
      <div className="grid gap-4 xl:grid-cols-[1.5fr_1fr]">
        <Card>
          <CardHeader
            title="This week"
            subtitle={
              detail?.phase?.name ??
              (detail?.week
                ? `Week ${detail.week.week_number}`
                : 'Planned sessions alongside what you did')
            }
          />
          {week.isLoading ? (
            <div className="grid grid-cols-2 gap-2 p-4 md:grid-cols-7">
              {days.map((d) => (
                <Skeleton key={d} className="h-36" />
              ))}
            </div>
          ) : detail && weekDays.length ? (
            <div className="grid grid-cols-2 gap-2 p-4 md:grid-cols-7">
              {weekDays.map((date, index) => {
                const s = sessionsByDate.get(date)
                const r = s?.result ?? null
                const actual = s?.activity ?? null
                return (
                  <div
                    key={date}
                    className="min-h-36 rounded-lg border border-edge bg-surface-soft p-3"
                  >
                    <div className="flex justify-between text-xs font-semibold text-ink-muted">
                      <span>{days[index]}</span>
                      <span>{date.slice(8)}</span>
                    </div>
                    {s ? (
                      <>
                        <p className="mt-3 text-sm font-semibold capitalize text-ink">
                          {s.label}
                        </p>
                        <p className="mt-1 text-xs text-ink-muted">
                          Planned ·{' '}
                          {s.targetDistance == null ? '—' : formatDistance(s.targetDistance)}
                          <br />
                          {s.paceLow == null && s.paceHigh == null
                            ? 'Pace —'
                            : `${pace(s.paceLow)} – ${pace(s.paceHigh)}`}
                        </p>
                        <div className="my-2 border-t border-edge" />
                        <p className="text-xs text-ink-soft">
                          {actual ? (
                            <>
                              {actual.name}
                              <br />
                              {formatDistance(actual.distance_m)} ·{' '}
                              {actual.pace_s_per_km == null ? '—' : pace(actual.pace_s_per_km)}
                            </>
                          ) : (
                            <span className="text-ink-faint">
                              {isFutureSession(date, todayIso) ? 'Not yet' : 'No activity'}
                            </span>
                          )}
                        </p>
                        {r ? <VerdictBadge date={date} result={r} className="mt-2" /> : null}
                      </>
                    ) : (
                      <p className="mt-4 text-sm text-ink-faint">Rest</p>
                    )}
                  </div>
                )
              })}
            </div>
          ) : (
            <EmptyState
              title="No week available"
              description={
                isPhone()
                  ? 'Choose a week with a plan.'
                  : 'Choose a week with a plan, or import a plan below.'
              }
            />
          )}
        </Card>
        <div className="space-y-4">
          <Card>
            <CardHeader title="Today" />
            {baseline ? (
              baselineToday ? (
                <div className="space-y-3 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <span className="font-semibold capitalize">{baselineToday.label}</span>
                    <VerdictBadge date={baselineToday.date} result={baselineToday.result} />
                  </div>
                  <p className="text-sm text-ink-soft">
                    {baselineToday.result?.message ?? 'Your usual session for today.'}
                  </p>
                </div>
              ) : (
                /* The baseline's own words: a day with no usual session is
                 * not a missed session. */
                <div className="space-y-3 p-4">
                  <span className="font-semibold">No usual session today</span>
                </div>
              )
            ) : prescribedPlan ? (
              /* Garmin's own words: the scheduled workout, and how it went. */
              prescribedToday ? (
                <div className="space-y-3 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <span className="font-semibold capitalize">{prescribedToday.label}</span>
                    <VerdictBadge date={prescribedToday.date} result={prescribedToday.result} />
                  </div>
                  {prescribedToday.result?.message ? (
                    <p className="text-sm text-ink-soft">{prescribedToday.result.message}</p>
                  ) : null}
                </div>
              ) : (
                <div className="space-y-3 p-4">
                  <span className="font-semibold">Nothing scheduled today</span>
                </div>
              )
            ) : today.isLoading ? (
              <div className="space-y-3 p-4">
                <Skeleton className="h-8 w-1/2" />
                <Skeleton className="h-16" />
              </div>
            ) : today.data?.has_plan ? (
              <div className="space-y-3 p-4">
                <div className="flex items-center justify-between">
                  <span className="font-semibold capitalize">
                    {today.data.session
                      ? sessionLabel(today.data.session)
                      : noSessionToday
                        ? 'Rest day'
                        : 'Nothing planned'}
                  </span>
                  {today.data.readiness && today.data.readiness.score != null && (
                    <Badge tone={today.data.readiness.score >= 70 ? 'positive' : 'warn'}>
                      Readiness {Math.round(today.data.readiness.score)} ·{' '}
                      {today.data.readiness.grade}
                    </Badge>
                  )}
                </div>
                <p className="text-sm text-ink-soft">{today.data.recommendation}</p>
                {today.data.rationale && (
                  <p className="text-xs text-ink-muted">{today.data.rationale}</p>
                )}
              </div>
            ) : (
              <EmptyState
                title={today.error ? 'Today’s plan is unavailable' : 'No plan yet'}
                description={
                  isPhone()
                    ? ''
                    : 'Import a training plan to get a session recommendation here.'
                }
              />
            )}
          </Card>
          {/*
            v1 views the data. The phone does not author a plan: the import
            is the desktop's job, and the phone reads what it finds.
          */}
          {isPhone() ? null : (
            <Card>
              <CardHeader
                title="Import a plan"
                subtitle="Paste a training plan document to create a block."
              />
              <div className="space-y-3 p-4">
                <div className="flex gap-2">
                  <select
                    aria-label="Plan format"
                    className={inputClass}
                    value={format}
                    onChange={(e) => setFormat(e.target.value)}
                  >
                    <option value="markdown">Markdown</option>
                    <option value="csv">CSV</option>
                    <option value="json">JSON</option>
                  </select>
                  <Button
                    disabled={!content.trim() || importing.isPending}
                    onClick={() => importing.mutate()}
                  >
                    {importing.isPending ? 'Importing…' : 'Import'}
                  </Button>
                </div>
                <textarea
                  className={`${inputClass} min-h-32`}
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  placeholder="Paste your plan here"
                />
                <p className="text-xs text-ink-muted">
                  Supports Markdown week sections, CSV rows, or structured JSON.
                </p>
                {importMessage && (
                  <div className="rounded-md bg-surface-soft p-3 text-sm">
                    {importMessage.name && (
                      <p className="font-semibold">Imported {importMessage.name}</p>
                    )}
                    {importMessage.error && (
                      <p className="text-negative">{importMessage.error}</p>
                    )}
                    {importMessage.warnings.map((w, i) => (
                      <p key={i} className="text-amber-700">
                        {w}
                      </p>
                    ))}
                  </div>
                )}
                {importing.isError && (
                  <p className="text-sm text-negative">
                    Plan import is temporarily unavailable. Your text is still here; try again
                    shortly.
                  </p>
                )}
              </div>
            </Card>
          )}
        </div>
      </div>
      <div className="mt-4 grid gap-4 md:grid-cols-4">
        {/*
          The tiles are the prescription's numbers: *missed*, *extra*, the
          adherence against a plan somebody wrote down. A baseline has none
          of those words, so on a baseline screen they are not shown — and
          with no plan written down they would be four zeros anyway.
        */}
        {summary && showsAdherence && hasAdherence ? (
          <>
            <StatTile
              label="Adherence"
              value={summary.overall_pct == null ? '—' : `${Math.round(summary.overall_pct)}%`}
            />
            <StatTile
              label="Planned / done"
              value={`${summary.planned_km.toFixed(1)} / ${summary.actual_km.toFixed(1)} km`}
            />
            <StatTile
              label="Sessions"
              value={`${summary.completed_sessions} / ${summary.planned_sessions}`}
            />
            <StatTile
              label="Missed · extra"
              value={`${summary.missed_sessions} · ${summary.extra_sessions}`}
            />
          </>
        ) : adherence.isLoading ? (
          days.slice(0, 4).map((d) => <Skeleton key={d} className="h-20" />)
        ) : null}
      </div>
      <div className="mt-4 grid gap-4 xl:grid-cols-[1.6fr_1fr]">
        {showsAdherence && summary && (
          <>
            <ChartFrame title="Planned vs actual by week" subtitle="Kilometres per week">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart
                  data={summary.rows.map((r) => ({ ...r, label: formatWeekKey(r.week_key) }))}
                >
                  <CartesianGrid stroke={GRID} vertical={false} />
                  <XAxis dataKey="label" tick={{ fontSize: 10, fill: INK_MUTED }} />
                  <YAxis tick={{ fontSize: 10, fill: INK_MUTED }} unit=" km" />
                  <Tooltip />
                  <Legend />
                  <Bar dataKey="planned_km" name="Planned" fill={SERIES.planned} />
                  <Bar dataKey="actual_km" name="Actual" fill={ACCENT_DATA} />
                </BarChart>
              </ResponsiveContainer>
            </ChartFrame>
            <Card>
              <CardHeader title="Session verdicts" />
              <div className="p-4">
                {dueTotal === 0 && notYetCount === 0 ? (
                  <EmptyState title="No session results yet" />
                ) : (
                  <>
                    {/*
                      The bar is the adherence: the sessions that were due.
                      A session that has not happened yet is not in it — it is
                      grey, it is counted in the legend, and it is not a
                      failure.
                    */}
                    <div className="flex h-4 overflow-hidden rounded-full">
                      {dueRows.map((row) => (
                        <div
                          key={row.verdict}
                          title={`${verdictNames[row.verdict]}: ${row.count}`}
                          style={{
                            width: `${dueTotal ? (row.count / dueTotal) * 100 : 0}%`,
                            background: verdictColors[row.verdict],
                          }}
                        />
                      ))}
                    </div>
                    <div className="mt-3 grid grid-cols-2 gap-2">
                      {(Object.keys(verdictNames) as Verdict[]).map((v) => (
                        <div key={v} className="flex items-center gap-2 text-xs">
                          <i
                            className="h-2 w-2 rounded-full"
                            style={{ background: verdictColors[v] }}
                          />
                          {verdictNames[v]}{' '}
                          <b>
                            {v === 'not_yet' ? notYetCount : (summary.by_verdict?.[v] ?? 0)}
                          </b>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </div>
            </Card>
          </>
        )}
      </div>
      <Card className="mt-4">
        <CardHeader
          title="Sessions"
          subtitle={
            isPhone()
              ? 'Last week, this week and next week.'
              : 'Every result, with the reason behind it.'
          }
        />
        {week.isLoading ? (
          <div className="space-y-2 p-4">
            {[1, 2, 3].map((x) => (
              <Skeleton key={x} className="h-16" />
            ))}
          </div>
        ) : listedSessions.length ? (
          <div className="divide-y divide-edge">
            {listedSessions.map((s) => {
              const r = s.result
              // A session that has not happened yet is neither a hit nor a
              // miss: it reads "Not yet", in grey, whatever the payload says.
              const notYet =
                r?.verdict === 'not_yet' || (!r && isFutureSession(s.date, todayIso))
              return (
                <div
                  key={s.key}
                  className="grid gap-2 px-4 py-3 text-sm md:grid-cols-[1fr_1fr_1fr_auto]"
                >
                  <div>
                    <p className="font-semibold text-ink">
                      {formatDate(s.date)} · <span className="capitalize">{s.label}</span>
                    </p>
                    <p className="text-xs text-ink-muted">
                      Target {s.targetDistance == null ? '—' : formatDistance(s.targetDistance)}{' '}
                      · {pace(s.paceLow)}–{pace(s.paceHigh)}
                    </p>
                  </div>
                  <div className="text-ink-soft">
                    {s.activity ? (
                      <>
                        <Link
                          className="font-medium text-brand-600 hover:underline"
                          to={`/activities/${r?.activity_id ?? s.activity.id}`}
                        >
                          {s.activity.name}
                        </Link>
                        <p className="text-xs">
                          {formatDistance(s.activity.distance_m)} ·{' '}
                          {pace(s.activity.pace_s_per_km)}
                        </p>
                      </>
                    ) : (
                      'No matched activity'
                    )}
                  </div>
                  <div>
                    {notYet ? (
                      <span className="text-xs text-ink-faint">Has not happened yet.</span>
                    ) : r?.message ? (
                      <p className="text-xs text-ink-soft">{r.message}</p>
                    ) : r?.cause ? (
                      <>
                        <p className="text-xs font-semibold uppercase tracking-wide text-ink-muted">
                          Why this happened
                        </p>
                        <p className="text-xs text-ink-soft">{renderCause(r)}</p>
                      </>
                    ) : r?.verdict === 'hit' ? (
                      <span className="text-xs text-ink-faint">As planned</span>
                    ) : (
                      <span className="text-xs text-ink-faint">No explanation available</span>
                    )}
                  </div>
                  <div>
                    {r ? (
                      <VerdictBadge date={s.date} result={r} />
                    ) : isFutureSession(s.date, todayIso) ? (
                      <Badge tone="neutral">Not yet</Badge>
                    ) : (
                      <Badge>Pending</Badge>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <EmptyState
            title={isPhone() ? 'No sessions around this week' : 'No sessions for this week'}
            description={
              isPhone()
                ? 'Nothing planned around this week.'
                : 'Choose another week or import your plan.'
            }
          />
        )}
      </Card>
    </div>
  )
}
