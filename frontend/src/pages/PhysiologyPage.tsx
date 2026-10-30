import type { RecoverySignal } from '../types_insights'
import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Area,
  AreaChart,
  Bar,
  CartesianGrid,
  ComposedChart,
  Line,
  LineChart,
  ReferenceArea,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import {
  ACCENT_DATA,
  BRAND,
  GRID,
  INK_FAINT,
  INK_MUTED,
  RAMP,
  ZONE_COLORS,
} from '../lib/palette'
import { formatDate } from '../lib/format'
import { Card, CardHeader, EmptyState, PageHeader, Skeleton } from '../components/ui'
import { isPhone } from '../lib/surface'
import { ConnectPrompt } from '../components/ConnectPrompt'
import { ChartFrame } from '../components/charts'
import { RangeSelector, sinceDate } from '../components/RangeSelector'

const gradeColor: Record<string, string> = {
  peak: ZONE_COLORS[1],
  good: RAMP[1],
  fair: ZONE_COLORS[3],
  low: ZONE_COLORS[4],
  unknown: INK_FAINT,
}
/**
 * "Normal" comes from the core, through `/insights/today`: one definition,
 * so Body and Today cannot disagree. Nothing here re-derives a threshold.
 */
function inNormal(signals: RecoverySignal[], key: string): boolean | null {
  const signal = signals.find((s) => s.key === key)
  return signal ? signal.normal : null
}

function normalHeadline(signals: RecoverySignal[]): string {
  if (!signals.length) return 'Building your baseline'
  const normal = signals.filter((s) => s.normal).length
  return `${normal} of ${signals.length} signals in your normal range`
}

const SIGNAL_NAME: Record<string, string> = {
  sleep: 'sleep',
  hrv: 'HRV',
  resting_hr: 'resting HR',
}

function normalAdvice(signals: RecoverySignal[]): string | null {
  if (!signals.length) return null
  const off = signals.filter((s) => !s.normal)
  if (!off.length) return 'Recovery looks normal. Train as you planned.'
  const sentence = `${off.map((s) => SIGNAL_NAME[s.key] ?? s.key).join(' and ')} outside your normal range: go easy today, or take the day.`
  return sentence.charAt(0).toUpperCase() + sentence.slice(1)
}

const num = (v: number | null | undefined, suffix = '') =>
  v == null ? '—' : `${Math.round(v * 10) / 10}${suffix}`

// The fields these three charts actually draw. A year of full daily rows is
// 180 KB of JSON, and a year of readiness rows with their drivers is 210 KB
// — none of it drawn. The drivers are still shown for today, which comes
// from the latest endpoint.
//
// The daily rows are the desktop's: the table of Garmin's own metrics. The
// phone does not draw them, so the phone does not ask for them.
const DAILY_FIELDS = 'date,body_battery_avg,stress_avg,acute_load,chronic_load,steps'
const READINESS_FIELDS = 'date,score,grade'
const HRV_FIELDS = 'date,hrv_ms,baseline,resting_hr,rhr_baseline'

// The readiness drivers the Body screen draws.
//
// The score is computed over seven — the stress, the body battery and the
// load count too, and the score does not change. Three of the seven are
// Garmin's own metrics, and they come off the phone's screen.
//
// The reference: `services.readiness.SHOWN_DRIVERS`.
const SHOWN_DRIVERS = ['sleep', 'sleep_score', 'hrv', 'resting_hr']
const NoRows = ({ loading, title }: { loading: boolean; title: string }) =>
  loading ? (
    <div className="p-4">
      <Skeleton className="h-48" />
    </div>
  ) : (
    <EmptyState title={title} description="There is no recorded data for this range yet." />
  )

/**
 * The year of readiness, as one strip.
 *
 * A circle per day meant 365 SVG nodes — and 367 on a real history, each
 * with its own group wrapper. It was also hard to read: at that density the
 * dots overlap into a smear.
 *
 * A strip of colour reads better *and* costs one DOM node: the grades are
 * merged into runs and drawn as a single gradient with hard stops. It is the
 * same idea as a year heatmap, and it survives being a hundred pixels wide.
 */
function ReadinessStrip({ items }: { items: { date: string; grade: string }[] }) {
  const { gradient, first, last } = useMemo(() => {
    if (!items.length) return { gradient: '', first: '', last: '' }
    const runs: { grade: string; from: number; to: number }[] = []
    items.forEach((item, index) => {
      const run = runs[runs.length - 1]
      if (run && run.grade === item.grade) run.to = index + 1
      else runs.push({ grade: item.grade, from: index, to: index + 1 })
    })
    const total = items.length
    const gradient =
      'linear-gradient(90deg, ' +
      runs
        .map((run) => {
          const from = ((run.from / total) * 100).toFixed(2)
          const to = ((run.to / total) * 100).toFixed(2)
          return `${gradeColor[run.grade] || INK_FAINT} ${from}% ${to}%`
        })
        .join(', ') +
      ')'
    return {
      gradient,
      first: items[0].date,
      last: items[items.length - 1].date,
    }
  }, [items])

  if (!gradient) return null

  return (
    <div className="px-1 pb-1 pt-2">
      <div
        className="h-2 w-full rounded-full"
        style={{ backgroundImage: gradient }}
        role="img"
        aria-label="Readiness grade for each day, left to right"
      />
      <div className="mt-1 flex justify-between text-[10px] text-ink-faint">
        <span>{formatDate(first)}</span>
        <span>{formatDate(last)}</span>
      </div>
    </div>
  )
}

export default function PhysiologyPage() {
  const [days, setDays] = useState(90)
  const since = useMemo(() => sinceDate(days), [days])
  const phone = isPhone()

  const todayInsight = useQuery({
    queryKey: ['insights-today'],
    queryFn: api.insightsToday,
    retry: false,
    staleTime: 60_000,
  })
  const signals = todayInsight.data?.recovery_signals ?? []
  const sources = useQuery({
    queryKey: ['physiology-sources'],
    queryFn: api.physiologySources,
    retry: false,
  })
  const enabled = sources.data?.available === true
  const latest = useQuery({
    queryKey: ['physiology-latest'],
    queryFn: api.physiologyLatest,
    enabled,
    retry: false,
  })
  const sleep = useQuery({
    queryKey: ['physiology-sleep'],
    queryFn: () => api.physiologySleep(),
    enabled,
    retry: false,
  })
  const hrv = useQuery({
    queryKey: ['physiology-hrv'],
    queryFn: () => api.physiologyHrv(HRV_FIELDS),
    enabled,
    retry: false,
  })
  const readiness = useQuery({
    queryKey: ['physiology-readiness'],
    queryFn: () => api.physiologyReadiness(READINESS_FIELDS),
    enabled,
    retry: false,
  })
  // The daily rows are the desktop's table. The phone does not draw it, so
  // the phone does not pay for it.
  const daily = useQuery({
    queryKey: ['physiology-daily'],
    queryFn: () => api.physiologyDaily(DAILY_FIELDS),
    enabled: enabled && !phone,
    retry: false,
  })
  const available = sources.data?.available === true
  const current = latest.data?.readiness
  const drivers = useMemo(() => {
    const all = current?.drivers ?? []
    // The score counts seven drivers; the phone draws four of them.
    return phone ? all.filter((driver) => SHOWN_DRIVERS.includes(driver.key)) : all
  }, [current, phone])
  return (
    <div className="mx-auto max-w-7xl p-5">
      {/*
        One name everywhere: **Body**.

        The phone tab was already called Body, and the desktop said
        Physiology for the same screen. Two names for one screen is a bug
        in the vocabulary, not a difference between the surfaces: the
        route stays `/physiology`, but nobody has to learn a second word
        for the page they already reach from Today.
      */}
      <PageHeader
        title="Body"
        description={
          phone
            ? 'Your readiness, your sleep, and your resting heart rate.'
            : 'Your recovery state, the signals behind it, and how they are moving.'
        }
      />
      {sources.isLoading ? (
        <Card>
          <div className="space-y-3 p-5">
            <Skeleton className="h-7 w-1/3" />
            <Skeleton className="h-16" />
          </div>
        </Card>
      ) : !available ? (
        phone ? (
          // The phone says one thing: connect, and the data follows. The
          // Plan tab is not on the phone, so there is nothing to promise
          // here about it.
          <Card className="mb-4">
            <ConnectPrompt what="your health data" />
          </Card>
        ) : (
          <Card className="mb-4">
            <CardHeader
              title="Health data isn’t connected"
              subtitle={
                sources.data?.reason ||
                (sources.error
                  ? 'The physiology source could not be reached.'
                  : 'Stride cannot reach the health endpoints yet.')
              }
            />
            <div className="px-5 pb-5">
              <p className="mb-3 text-sm text-ink-soft">
                Connect Garmin with the account health access enabled, then sync again.
                Readiness, sleep, HRV and resting heart rate require this connection.
              </p>
              {sources.data?.auth_method && (
                <p className="mb-3 text-xs text-ink-muted">
                  Current connection: {sources.data.auth_method}
                </p>
              )}
              {sources.data?.endpoints?.length ? (
                <div className="space-y-2">
                  {sources.data.endpoints.map((e) => (
                    <div
                      key={e.name}
                      className="flex items-center justify-between rounded-md bg-surface-soft px-3 py-2 text-sm"
                    >
                      <span>{e.name.replace(/_/g, ' ')}</span>
                      <span className="text-xs font-medium text-ink-muted">
                        {e.available ? 'Available' : 'Not connected'}
                      </span>
                    </div>
                  ))}
                </div>
              ) : null}
              {!sources.data && (
                <p className="text-sm text-ink-muted">
                  The source status endpoint is unavailable. No health charts are shown until
                  availability can be confirmed.
                </p>
              )}
              <div className="mt-4 flex flex-wrap items-center gap-3 border-t border-edge pt-4">
                <Link
                  to="/settings"
                  className="inline-flex h-9 items-center rounded-md bg-brand-500 px-4 text-[14px] font-semibold text-white hover:bg-brand-600"
                >
                  Connect Garmin
                </Link>
                <Link
                  to="/plan"
                  className="text-[13px] font-medium text-brand-600 hover:underline"
                >
                  Plan vs actual works without it &rarr;
                </Link>
              </div>
              <p className="mt-3 text-xs text-ink-muted">
                Once connected, this page will show your readiness score with the drivers behind
                it, sleep stages, and HRV against your 30-day baseline.
              </p>
            </div>
          </Card>
        )
      ) : (
        <>
          <Card className="mb-4">
            <CardHeader
              title="Today’s readiness"
              subtitle={
                current
                  ? `Updated for ${formatDate(current.date)}`
                  : 'The score, and the signals behind it.'
              }
            />
            {latest.isLoading ? (
              <div className="p-5">
                <Skeleton className="h-24" />
              </div>
            ) : current ? (
              <div className="grid gap-5 p-5 lg:grid-cols-[220px_1fr]">
                <div className="flex flex-col justify-center rounded-xl bg-surface-soft p-5">
                  {/*
                    The headline is the athlete's normal, not a 0-100 number:
                    which recovery signals are inside their own range. The
                    score is still there, small, for whoever wants it.
                  */}
                  <strong className="text-2xl font-bold leading-tight text-ink">
                    {normalHeadline(signals)}
                  </strong>
                  <p className="mt-3 text-sm text-ink-soft">
                    {normalAdvice(signals) ??
                      current.rationale ??
                      'Readiness reflects your recent recovery signals.'}
                  </p>
                </div>
                <div>
                  <p className="mb-3 text-xs font-semibold uppercase tracking-wide text-ink-muted">
                    What’s driving it
                  </p>
                  {drivers.length ? (
                    <div className="space-y-3">
                      {drivers.map((d) => {
                        const ok = inNormal(signals, d.key)
                        return (
                          <div key={d.key}>
                            <div className="flex items-center justify-between gap-3 text-sm">
                              <span className="flex items-center gap-1.5 font-semibold">
                                {ok != null ? (
                                  <span
                                    className={`h-2 w-2 rounded-full ${ok ? 'bg-positive' : 'bg-warn'}`}
                                    title={
                                      ok ? 'In your normal range' : 'Outside your normal range'
                                    }
                                  />
                                ) : null}
                                {d.label}
                              </span>
                              <span className="text-ink-muted">
                                {num(d.value)}
                                {d.unit ? (
                                  <span className="text-ink-faint"> {d.unit}</span>
                                ) : null}
                                {d.baseline != null && (
                                  <span className="text-ink-faint">
                                    {' '}
                                    · baseline {num(d.baseline)}
                                    {d.delta != null && ` · Δ ${num(d.delta)}`}
                                  </span>
                                )}
                              </span>
                            </div>
                            {d.detail && (
                              <p className="mt-1 text-xs text-ink-muted">{d.detail}</p>
                            )}
                          </div>
                        )
                      })}
                    </div>
                  ) : (
                    <EmptyState
                      title="No readiness drivers"
                      description="The latest score has no driver breakdown."
                    />
                  )}
                </div>
              </div>
            ) : (
              <EmptyState
                title="No readiness score yet"
                description="A score will appear when Garmin has synced recent health data."
              />
            )}
          </Card>
          <div className="grid gap-4 xl:grid-cols-2">
            <ChartFrame
              title="Sleep"
              subtitle={`Sleep stages by night · baseline ${sleep.data?.items.length ? `${Math.round((sleep.data.items.reduce((n, p) => n + (p.sleep_s ?? 0), 0) / sleep.data.items.length / 3600) * 10) / 10} h average` : '—'}`}
              height={280}
              action={<RangeSelector value={days} onChange={setDays} className="self-start" />}
            >
              {sleep.isLoading ? (
                <div className="p-3">
                  <Skeleton className="h-52" />
                </div>
              ) : sleep.data?.items.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <ComposedChart
                    data={sleep.data.items
                      .filter((p) => p.date >= since)
                      .map((p) => ({
                        ...p,
                        label: formatDate(p.date).slice(0, 6),
                        deep: (p.deep_sleep_s ?? 0) / 3600,
                        rem: (p.rem_s ?? 0) / 3600,
                        light: (p.light_sleep_s ?? 0) / 3600,
                        awake: (p.awake_s ?? 0) / 3600,
                      }))}
                  >
                    <CartesianGrid stroke={GRID} vertical={false} />
                    <XAxis dataKey="label" tick={{ fontSize: 10, fill: INK_MUTED }} />
                    <YAxis yAxisId="sleep" tick={{ fontSize: 10, fill: INK_MUTED }} unit=" h" />
                    <YAxis
                      yAxisId="score"
                      orientation="right"
                      domain={[0, 100]}
                      tick={{ fontSize: 10, fill: INK_MUTED }}
                    />
                    <Tooltip />
                    <Bar
                      yAxisId="sleep"
                      stackId="sleep"
                      dataKey="deep"
                      name="Deep"
                      fill={RAMP[4]}
                    />
                    <Bar
                      yAxisId="sleep"
                      stackId="sleep"
                      dataKey="rem"
                      name="REM"
                      fill={RAMP[3]}
                    />
                    <Bar
                      yAxisId="sleep"
                      stackId="sleep"
                      dataKey="light"
                      name="Light"
                      fill={BRAND[300]}
                    />
                    <Bar
                      yAxisId="sleep"
                      stackId="sleep"
                      dataKey="awake"
                      name="Awake"
                      fill={BRAND[100]}
                    />
                    <Line
                      yAxisId="score"
                      dataKey="sleep_score"
                      name="Sleep score"
                      stroke={ACCENT_DATA}
                      dot={false}
                    />
                  </ComposedChart>
                </ResponsiveContainer>
              ) : (
                <NoRows loading={false} title="No sleep data yet" />
              )}
            </ChartFrame>
            <ChartFrame
              title="HRV and resting heart rate"
              subtitle={`HRV baseline ${num(hrv.data?.baseline, ' ms')} · Resting HR baseline ${num(hrv.data?.items[hrv.data.items.length - 1]?.rhr_baseline, ' bpm')}`}
              height={280}
            >
              {hrv.isLoading ? (
                <div className="p-3">
                  <Skeleton className="h-52" />
                </div>
              ) : hrv.data?.items.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart
                    data={hrv.data.items
                      .filter((p) => p.date >= since)
                      .map((p) => ({
                        ...p,
                        label: formatDate(p.date).slice(0, 6),
                      }))}
                  >
                    <CartesianGrid stroke={GRID} vertical={false} />
                    <XAxis dataKey="label" tick={{ fontSize: 10, fill: INK_MUTED }} />
                    <YAxis yAxisId="hrv" tick={{ fontSize: 10, fill: INK_MUTED }} unit=" ms" />
                    <YAxis
                      yAxisId="rhr"
                      orientation="right"
                      tick={{ fontSize: 10, fill: INK_MUTED }}
                      unit=" bpm"
                    />
                    <Tooltip />
                    {hrv.data.baseline != null && (
                      <ReferenceArea
                        yAxisId="hrv"
                        y1={hrv.data.baseline * 0.95}
                        y2={hrv.data.baseline * 1.05}
                        fill={INK_FAINT}
                        fillOpacity={0.14}
                        label={{ value: '30-day baseline band', fontSize: 9, fill: INK_MUTED }}
                      />
                    )}
                    <Line
                      yAxisId="hrv"
                      dataKey="baseline"
                      name="HRV 30-day baseline"
                      stroke={INK_FAINT}
                      strokeDasharray="4 3"
                      dot={false}
                    />
                    <Line
                      yAxisId="hrv"
                      dataKey="hrv_ms"
                      name="HRV"
                      stroke={ACCENT_DATA}
                      strokeWidth={2}
                      dot={false}
                    />
                    <Line
                      yAxisId="rhr"
                      dataKey="rhr_baseline"
                      name="RHR baseline"
                      stroke={RAMP[3]}
                      strokeDasharray="4 3"
                      dot={false}
                    />
                    <Line
                      yAxisId="rhr"
                      dataKey="resting_hr"
                      name="Resting HR"
                      stroke={RAMP[1]}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              ) : (
                <NoRows loading={false} title="No HRV or resting heart rate data" />
              )}
            </ChartFrame>
            <ChartFrame
              title="Readiness trend"
              subtitle="Daily score, coloured by grade"
              height={260}
            >
              {readiness.isLoading ? (
                <div className="p-3">
                  <Skeleton className="h-48" />
                </div>
              ) : readiness.data?.items.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart
                    data={readiness.data.items
                      .filter((p) => p.date >= since)
                      .map((p) => ({
                        ...p,
                        label: formatDate(p.date).slice(0, 6),
                      }))}
                  >
                    <CartesianGrid stroke={GRID} vertical={false} />
                    <XAxis dataKey="label" tick={{ fontSize: 10, fill: INK_MUTED }} />
                    <YAxis domain={[0, 100]} tick={{ fontSize: 10, fill: INK_MUTED }} />
                    <Tooltip />
                    <Area
                      dataKey="score"
                      name="Readiness"
                      stroke={ACCENT_DATA}
                      fill={ACCENT_DATA}
                      fillOpacity={0.12}
                      dot={false}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              ) : (
                <NoRows loading={false} title="No readiness history" />
              )}
              {readiness.data?.items.length ? (
                <ReadinessStrip
                  items={readiness.data.items.map((p) => ({
                    date: p.date,
                    grade: p.grade ?? 'unknown',
                  }))}
                />
              ) : null}
            </ChartFrame>
            {phone ? null : (
              <Card>
                <CardHeader
                  title="Body battery, stress and load"
                  subtitle="Daily context from Garmin"
                />
                {daily.isLoading ? (
                  <div className="space-y-2 p-4">
                    {[1, 2, 3, 4].map((x) => (
                      <Skeleton key={x} className="h-8" />
                    ))}
                  </div>
                ) : daily.data?.items.length ? (
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-xs">
                      <thead className="bg-surface-soft text-[10px] uppercase text-ink-muted">
                        <tr>
                          {[
                            'Date',
                            'Battery avg',
                            'Stress avg',
                            'Acute / chronic',
                            'Steps',
                          ].map((h) => (
                            <th key={h} className="px-3 py-2">
                              {h}
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-edge">
                        {daily.data.items
                          .slice(-10)
                          .reverse()
                          .map((d) => (
                            <tr key={d.date}>
                              <td className="px-3 py-2">{formatDate(d.date)}</td>
                              <td className="px-3 py-2">{num(d.body_battery_avg)}</td>
                              <td className="px-3 py-2">{num(d.stress_avg)}</td>
                              <td className="px-3 py-2">
                                {num(d.acute_load)} / {num(d.chronic_load)}
                              </td>
                              <td className="px-3 py-2">
                                {d.steps == null ? '—' : d.steps.toLocaleString()}
                              </td>
                            </tr>
                          ))}
                      </tbody>
                    </table>
                  </div>
                ) : (
                  <EmptyState title="No daily physiology recorded" />
                )}
              </Card>
            )}
          </div>
        </>
      )}
    </div>
  )
}
