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
import { formatDate, formatDistance, formatPace, formatWeekKey } from '../lib/format'
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

const tones: Record<Verdict, 'positive' | 'warn' | 'negative' | 'info' | 'brand'> = {
  hit: 'positive',
  missed_pace: 'warn',
  short: 'warn',
  missed: 'negative',
  substituted: 'info',
  bonus: 'brand',
}
const verdictNames: Record<Verdict, string> = {
  hit: 'Hit',
  missed_pace: 'Missed pace',
  short: 'Short',
  missed: 'Missed',
  substituted: 'Substituted',
  bonus: 'Bonus',
}
const verdictColors: Record<Verdict, string> = {
  hit: ZONE_COLORS[1],
  missed_pace: ZONE_COLORS[3],
  short: ZONE_COLORS[3],
  missed: ZONE_COLORS[4],
  substituted: RAMP[1],
  bonus: ACCENT_DATA,
}
const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
const todayKey = () => new Date().toISOString().slice(0, 10)
const pace = (v: number | null | undefined) => (v == null ? '—' : `${formatPace(v)}/km`)
const sessionLabel = (s: PlanSessionWithResult) => s.title || s.session_type

/**
 * Render an explanation with the right amount of confidence.
 *
 * A confidence of zero means the cause is a plain fact ("no activity was
 * logged") and must read as a statement. Only an *inferred* cause with low
 * confidence gets hedged.
 */
const renderCause = (r: { cause: string | null; cause_confidence: number | null }) => {
  const cause = r.cause
  if (!cause) return ''
  const confidence = r.cause_confidence ?? 0
  if (confidence >= 0.66) return cause
  if (confidence <= 0) return cause
  return `Might be ${cause.charAt(0).toLowerCase()}${cause.slice(1)}?`
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
  const week = useQuery({
    queryKey: ['plan-week', effectiveWeekDate],
    queryFn: () => api.planWeek(effectiveWeekDate),
    retry: false,
  })
  const adherence = useQuery({
    queryKey: ['plan-adherence'],
    queryFn: () => api.planAdherence(),
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
  const sessions = detail?.sessions ?? []
  const sessionsByDate = new Map(sessions.map((s) => [s.date, s]))
  const start = detail?.week?.start_date ?? detail?.sessions?.[0]?.date
  const weekDays = start
    ? Array.from({ length: 7 }, (_, i) => {
        const d = new Date(`${start}T12:00:00`)
        d.setDate(d.getDate() + i)
        return d.toISOString().slice(0, 10)
      })
    : []
  const summary = adherence.data
  // "No plan" means no training block exists. A block that schedules nothing
  // for today is a rest day, not a missing plan.
  const noPlan = !current.isLoading && !current.data?.block && !current.error
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
      {noPlan && (
        <Card className="mb-5">
          <EmptyState
            title="No plan yet"
            description={
              isPhone()
                ? 'A plan shows up here once it is imported.'
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
                const r = s?.result
                const actual = s?.activity
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
                          {sessionLabel(s)}
                        </p>
                        <p className="mt-1 text-xs text-ink-muted">
                          Planned ·{' '}
                          {s.target_distance_m == null
                            ? '—'
                            : formatDistance(s.target_distance_m)}
                          <br />
                          {s.pace_low_s_km == null && s.pace_high_s_km == null
                            ? 'Pace —'
                            : `${pace(s.pace_low_s_km)} – ${pace(s.pace_high_s_km)}`}
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
                              {date < todayKey() ? 'No activity' : 'Not yet'}
                            </span>
                          )}
                        </p>
                        {r && (
                          <Badge tone={tones[r.verdict]} className="mt-2">
                            {verdictNames[r.verdict]}
                          </Badge>
                        )}
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
            {today.isLoading ? (
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
        {summary ? (
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
        {summary && (
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
                {Object.entries(summary.by_verdict).every(([, v]) => !v) ? (
                  <EmptyState title="No session results yet" />
                ) : (
                  <>
                    <div className="flex h-4 overflow-hidden rounded-full">
                      {(Object.keys(verdictNames) as Verdict[]).map((v) => (
                        <div
                          key={v}
                          title={`${verdictNames[v]}: ${summary.by_verdict[v]}`}
                          style={{
                            width: `${summary.planned_sessions ? (summary.by_verdict[v] / summary.planned_sessions) * 100 : 0}%`,
                            background: verdictColors[v],
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
                          {verdictNames[v]} <b>{summary.by_verdict[v]}</b>
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
        <CardHeader title="Sessions" subtitle="Every result, with the reason behind it." />
        {week.isLoading ? (
          <div className="space-y-2 p-4">
            {[1, 2, 3].map((x) => (
              <Skeleton key={x} className="h-16" />
            ))}
          </div>
        ) : sessions.length ? (
          <div className="divide-y divide-edge">
            {sessions.map((s) => {
              const r = s.result
              return (
                <div
                  key={s.id}
                  className="grid gap-2 px-4 py-3 text-sm md:grid-cols-[1fr_1fr_1fr_auto]"
                >
                  <div>
                    <p className="font-semibold text-ink">
                      {formatDate(s.date)} ·{' '}
                      <span className="capitalize">{sessionLabel(s)}</span>
                    </p>
                    <p className="text-xs text-ink-muted">
                      Target{' '}
                      {s.target_distance_m == null ? '—' : formatDistance(s.target_distance_m)}{' '}
                      · {pace(s.pace_low_s_km)}–{pace(s.pace_high_s_km)}
                    </p>
                  </div>
                  <div className="text-ink-soft">
                    {s.activity ? (
                      <>
                        <Link
                          className="font-medium text-brand-600 hover:underline"
                          to={`/activities/${r?.activity_id}`}
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
                    {r?.cause ? (
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
                      <Badge tone={tones[r.verdict]}>{verdictNames[r.verdict]}</Badge>
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
            title="No sessions for this week"
            description="Choose another week or import your plan."
          />
        )}
      </Card>
    </div>
  )
}
