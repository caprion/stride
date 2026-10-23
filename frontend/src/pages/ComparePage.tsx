import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { Search, X } from 'lucide-react'
import { api } from '../lib/api'
import type {
  CompareCandidate,
  CompareFormat,
  CompareParameter,
  CompareRun,
} from '../types_phase2'
import {
  formatDate,
  formatDistance,
  formatDuration,
  formatElevation,
  formatHr,
  formatNumber,
  formatPace,
} from '../lib/format'
import {
  Badge,
  Card,
  CardHeader,
  EmptyState,
  PageHeader,
  SegmentedControl,
  Skeleton,
  inputClass,
} from '../components/ui'

type ProfileMetric = 'hr' | 'pace_s_per_km' | 'cadence' | 'elevation'
const metricOptions: { value: ProfileMetric; label: string }[] = [
  { value: 'hr', label: 'Heart rate' },
  { value: 'pace_s_per_km', label: 'Pace' },
  { value: 'cadence', label: 'Cadence' },
  { value: 'elevation', label: 'Elevation' },
]
const colors = [
  '#fc5200',
  '#2d9cdb',
  '#1a9c5a',
  '#c14fb0',
  '#e6a700',
  '#6d7176',
  '#8a4bff',
  '#d93a2b',
]

function formatValue(value: number | null | undefined, format: CompareFormat) {
  if (value == null) return '—'
  switch (format) {
    case 'distance':
      return formatDistance(value)
    case 'duration':
      return formatDuration(value)
    case 'pace':
      return `${formatPace(value)} /km`
    case 'elevation':
      return formatElevation(value)
    case 'hr':
      return `${formatHr(value)} bpm`
    case 'number':
      return formatNumber(value)
  }
}

function candidateTitle(candidate: Pick<CompareCandidate, 'name' | 'local_date'>) {
  return `${formatDate(candidate.local_date)} · ${candidate.name}`
}

/** Round for display without trailing zeros. */
function round(value: number, digits = 1): number {
  const factor = 10 ** digits
  return Math.round(value * factor) / factor
}

function summarize(runs: CompareRun[], parameters: CompareParameter[]) {
  if (runs.length < 2) return ''
  const [first, second] = runs
  // Two runs often share a name ("Bengaluru Running"), so the date is what
  // actually distinguishes them.
  const label = (run: CompareRun) => `${run.name || 'Run'} (${formatDate(run.local_date)})`
  const runNames = [label(first), label(second)]
  const dateNames = [formatDate(first.local_date), formatDate(second.local_date)]
  const observations: string[] = []
  for (const parameter of parameters) {
    const a = first.values[parameter.key]
    const b = second.values[parameter.key]
    if (a == null || b == null || parameter.better === 'neutral') continue
    if (a === b) continue
    const better = parameter.better === 'lower' ? a < b : a > b
    const difference = Math.abs(a - b)
    // A percentage against a near-zero baseline is noise ("321% behind" for
    // a drift of 0.1 vs 0.4 bpm). State the absolute difference instead.
    const magnitude = Math.abs(b) >= 5 ? Math.abs((a - b) / b) * 100 : null
    const unit = parameter.unit ? ` ${parameter.unit}` : ''
    const detail =
      magnitude !== null && magnitude >= 1
        ? `${Math.round(magnitude)}% ${better ? 'better' : 'behind'}`
        : `${round(difference, 1)}${unit} ${better ? 'better' : 'worse'}`
    const winner = better ? runNames[0] : runNames[1]
    const other = better ? runNames[1] : runNames[0]
    observations.push(
      `${winner} had the stronger ${parameter.label.toLowerCase()} (${detail} than ${other}).`,
    )
    if (observations.length === 2) break
  }
  const availableMetrics = parameters.filter(
    (p) => first.values[p.key] != null && second.values[p.key] != null,
  )
  if (
    !observations.length ||
    availableMetrics.every((p) => first.values[p.key] === second.values[p.key])
  ) {
    return `${dateNames[0]} and ${dateNames[1]} are very similar across the available comparison metrics.`
  }
  return observations.join(' ')
}

export default function ComparePage() {
  const [selectedIds, setSelectedIds] = useState<number[]>([])
  const [search, setSearch] = useState('')
  const [metric, setMetric] = useState<ProfileMetric>('hr')
  const candidatesQuery = useQuery({
    queryKey: ['compare-candidates'],
    queryFn: () => api.compareCandidates(),
    retry: false,
  })
  const comparisonQuery = useQuery({
    queryKey: ['compare-runs', selectedIds],
    queryFn: () => api.compareRuns(selectedIds),
    enabled: selectedIds.length >= 2,
    retry: false,
  })
  const candidates = candidatesQuery.data?.items ?? []
  const comparison = comparisonQuery.data
  const runs = comparison?.runs ?? []
  const filteredCandidates = useMemo(() => {
    const query = search.trim().toLowerCase()
    return candidates.filter(
      (candidate) =>
        !selectedIds.includes(candidate.id) &&
        (!query || `${candidate.name} ${candidate.local_date}`.toLowerCase().includes(query)),
    )
  }, [candidates, search, selectedIds])
  const chartRuns = runs.filter((run) => run.profile.some((point) => point[metric] != null))
  const chartRows = useMemo(() => {
    const byDistance = new Map<number, Record<string, number | null>>()
    for (const run of chartRuns) {
      for (const point of run.profile) {
        const row = byDistance.get(point.pct) ?? { distancePct: point.pct }
        row[String(run.id)] = point[metric]
        byDistance.set(point.pct, row)
      }
    }
    return [...byDistance.entries()].sort(([a], [b]) => a - b).map(([, row]) => row)
  }, [chartRuns, metric])

  function toggleCandidate(id: number) {
    setSelectedIds((current) =>
      current.includes(id)
        ? current.filter((selected) => selected !== id)
        : current.length < 8
          ? [...current, id]
          : current,
    )
  }

  const bestValues = (parameter: CompareParameter) => {
    if (parameter.better === 'neutral') return new Set<number>()
    const values = runs
      .map((run) => run.values[parameter.key])
      .filter((value): value is number => value != null)
    if (!values.length) return new Set<number>()
    const best = parameter.better === 'lower' ? Math.min(...values) : Math.max(...values)
    return new Set(
      runs.filter((run) => run.values[parameter.key] === best).map((run) => run.id),
    )
  }

  return (
    <div className="min-h-full bg-surface-soft p-4 md:p-6">
      <PageHeader
        title="Compare runs"
        description="See how your running has changed across your history."
      />
      {candidatesQuery.isError ? (
        <Card className="mb-4">
          <EmptyState
            title="Runs could not be loaded"
            description="Try refreshing the page in a moment."
          />
        </Card>
      ) : null}
      <div className="grid grid-cols-1 items-start gap-4 xl:grid-cols-[minmax(0,1fr)_340px]">
        <main className="min-w-0 space-y-4">
          {selectedIds.length === 0 ? (
            <Card>
              <CardHeader
                title="Start with a comparison"
                subtitle="Pick at least two runs from your history."
              />
              {candidatesQuery.isLoading ? (
                <div className="space-y-2 p-4">
                  {[1, 2, 3].map((n) => (
                    <Skeleton key={n} className="h-12" />
                  ))}
                </div>
              ) : (
                <div className="space-y-1 px-3 pb-3">
                  {candidates.slice(0, 3).map((candidate) => (
                    <button
                      key={candidate.id}
                      onClick={() => toggleCandidate(candidate.id)}
                      className="flex w-full items-center justify-between rounded-md p-3 text-left hover:bg-surface-soft"
                    >
                      <span>
                        <span className="block text-sm font-semibold text-ink">
                          {candidateTitle(candidate)}
                        </span>
                        <span className="text-xs text-ink-muted">
                          {formatDistance(candidate.distance_m)} ·{' '}
                          {formatPace(candidate.pace_s_per_km)} /km ·{' '}
                          {candidate.avg_hr == null ? '—' : `${formatHr(candidate.avg_hr)} bpm`}
                        </span>
                      </span>
                      <span className="text-sm font-semibold text-brand-600">Add</span>
                    </button>
                  ))}
                  {!candidates.length && !candidatesQuery.isLoading ? (
                    <EmptyState
                      title="No runs available"
                      description="Runs will appear here after they have synced."
                    />
                  ) : null}
                </div>
              )}
            </Card>
          ) : selectedIds.length === 1 ? (
            <Card className="p-5">
              <p className="text-sm font-semibold text-ink">Choose one more run to compare.</p>
              <p className="mt-1 text-xs text-ink-muted">
                Your selected run is ready; add another from the list.
              </p>
            </Card>
          ) : comparisonQuery.isLoading ? (
            <Card>
              <CardHeader
                title="Building comparison"
                subtitle="Aligning the runs by distance."
              />
              <div className="space-y-3 p-4">
                <Skeleton className="h-48" />
                <Skeleton className="h-32" />
              </div>
            </Card>
          ) : comparisonQuery.isError ? (
            <Card>
              <EmptyState
                title="Comparison could not be loaded"
                description="Try removing a run and adding it again."
              />
            </Card>
          ) : comparison ? (
            <>
              <Card className="overflow-hidden">
                <CardHeader
                  title="Run comparison"
                  subtitle="Rows with a green marker show the strongest value for that parameter."
                />
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[620px] text-left text-xs">
                    <thead className="bg-surface-soft text-[10px] uppercase tracking-wide text-ink-muted">
                      <tr>
                        <th className="sticky left-0 bg-surface-soft px-4 py-2.5">Parameter</th>
                        {runs.map((run, index) => (
                          <th key={run.id} className="min-w-[140px] px-3 py-2.5">
                            <span
                              className="block"
                              style={{ color: colors[index % colors.length] }}
                            >
                              {formatDate(run.local_date)}
                            </span>
                            <span className="mt-0.5 block max-w-[180px] truncate normal-case tracking-normal text-ink-soft">
                              {run.name}
                            </span>
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-edge">
                      {comparison.parameters.map((parameter) => {
                        const winners = bestValues(parameter)
                        return (
                          <tr key={parameter.key}>
                            <th className="sticky left-0 bg-surface px-4 py-2.5 font-medium text-ink-soft">
                              {parameter.label}
                              {parameter.unit ? (
                                <span className="ml-1 text-ink-faint">({parameter.unit})</span>
                              ) : null}
                            </th>
                            {runs.map((run) => {
                              const value = run.values[parameter.key]
                              const winner = value != null && winners.has(run.id)
                              return (
                                <td
                                  key={run.id}
                                  className={`px-3 py-2.5 font-semibold tabular-nums ${winner ? 'bg-emerald-50 text-emerald-800' : 'text-ink'}`}
                                >
                                  {winner ? (
                                    <span className="mr-1 text-emerald-600" aria-label="Best">
                                      ●
                                    </span>
                                  ) : null}
                                  {formatValue(value, parameter.format)}
                                </td>
                              )
                            })}
                          </tr>
                        )
                      })}
                    </tbody>
                  </table>
                </div>
              </Card>
              <Card>
                <CardHeader
                  title="Profiles over distance"
                  subtitle="Curves use each run's own distance, normalized from start to finish."
                  action={
                    <SegmentedControl
                      value={metric}
                      onChange={(value) => setMetric(value as ProfileMetric)}
                      options={metricOptions}
                    />
                  }
                />
                {chartRows.length ? (
                  <div className="h-[290px] px-2 pb-4">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart
                        data={chartRows}
                        margin={{ top: 8, right: 16, left: 0, bottom: 4 }}
                      >
                        <CartesianGrid stroke="#ececec" vertical={false} />
                        <XAxis
                          dataKey="distancePct"
                          type="number"
                          domain={[0, 100]}
                          tickFormatter={(value) => `${value}%`}
                          tick={{ fontSize: 11, fill: '#6d7176' }}
                          tickLine={false}
                          axisLine={{ stroke: '#ececec' }}
                        />
                        <YAxis
                          tick={{ fontSize: 11, fill: '#6d7176' }}
                          tickLine={false}
                          axisLine={false}
                          width={52}
                          reversed={metric === 'pace_s_per_km'}
                          tickFormatter={(value) =>
                            metric === 'pace_s_per_km'
                              ? formatPace(value)
                              : String(Math.round(value))
                          }
                        />
                        <Tooltip
                          labelFormatter={(value) => `${Number(value).toFixed(0)}% of run`}
                          formatter={(value, name) => {
                            const run = runs.find((item) => String(item.id) === String(name))
                            const display =
                              metric === 'pace_s_per_km'
                                ? `${formatPace(Number(value))} /km`
                                : metric === 'hr'
                                  ? `${formatHr(Number(value))} bpm`
                                  : metric === 'elevation'
                                    ? formatElevation(Number(value))
                                    : `${formatNumber(Number(value))} spm`
                            return [display, run?.name ?? name]
                          }}
                        />
                        <Line dataKey="distancePct" hide />
                        {chartRuns.map((run) => (
                          <Line
                            key={run.id}
                            dataKey={String(run.id)}
                            name={String(run.id)}
                            stroke={
                              colors[
                                runs.findIndex((item) => item.id === run.id) % colors.length
                              ]
                            }
                            strokeWidth={2}
                            dot={false}
                            connectNulls={false}
                            activeDot={{ r: 3 }}
                            isAnimationActive={false}
                          />
                        ))}
                      </LineChart>
                    </ResponsiveContainer>
                    <div className="flex flex-wrap justify-center gap-x-4 gap-y-1 text-[11px]">
                      {chartRuns.map((run) => (
                        <span
                          key={run.id}
                          className="inline-flex items-center gap-1.5 text-ink-muted"
                        >
                          <i
                            className="h-2 w-2 rounded-full"
                            style={{
                              background:
                                colors[
                                  runs.findIndex((item) => item.id === run.id) % colors.length
                                ],
                            }}
                          />
                          {formatDate(run.local_date)} · {run.name}
                        </span>
                      ))}
                    </div>
                  </div>
                ) : (
                  <EmptyState
                    title={`No ${metricOptions.find((option) => option.value === metric)?.label.toLowerCase()} profiles available`}
                    description="These runs do not include values for this stream."
                  />
                )}
              </Card>
              <Card className="p-4">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-ink-muted">
                  What stands out
                </p>
                <p className="mt-1 text-sm leading-relaxed text-ink-soft">
                  {summarize(runs, comparison.parameters)}
                </p>
              </Card>
              {runs.some((run) => run.profile.length === 0) ? (
                <p className="px-1 text-xs text-ink-muted">
                  Curves are unavailable for{' '}
                  {runs
                    .filter((run) => !run.profile.length)
                    .map((run) => run.name)
                    .join(', ')}{' '}
                  because no profile streams were recorded.
                </p>
              ) : null}
              {comparison.missing.length ? (
                <p className="px-1 text-xs text-ink-muted">
                  {comparison.missing.length} selected{' '}
                  {comparison.missing.length === 1 ? 'run was' : 'runs were'} unavailable and
                  omitted.
                </p>
              ) : null}
            </>
          ) : null}
        </main>
        <aside className="space-y-4 xl:sticky xl:top-4">
          <Card>
            <CardHeader
              title={`Selected runs · ${selectedIds.length}/8`}
              subtitle="Choose runs from anywhere in your history."
            />
            <div className="flex flex-wrap gap-2 px-4 pb-3">
              {selectedIds.map((id, index) => {
                const run = runs.find((item) => item.id === id)
                const candidate = candidates.find((item) => item.id === id)
                const label = run?.name ?? candidate?.name ?? `Run ${id}`
                return (
                  <span
                    key={id}
                    className="inline-flex max-w-full items-center gap-1 rounded-full bg-surface-soft px-2.5 py-1 text-[11px] text-ink-soft"
                  >
                    <i
                      className="h-2 w-2 shrink-0 rounded-full"
                      style={{ background: colors[index % colors.length] }}
                    />
                    <span className="max-w-[190px] truncate">{label}</span>
                    <button
                      aria-label={`Remove ${label}`}
                      onClick={() => toggleCandidate(id)}
                      className="ml-0.5 rounded-full p-0.5 hover:bg-surface-sunken"
                    >
                      <X size={12} />
                    </button>
                  </span>
                )
              })}
            </div>
            <div className="px-4 pb-3">
              <div className="relative">
                <Search
                  size={15}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-ink-faint"
                />
                <input
                  className={`${inputClass} pl-9`}
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Search runs"
                  aria-label="Search runs"
                />
              </div>
            </div>
            <div className="max-h-[520px] overflow-y-auto border-t border-edge">
              {candidatesQuery.isLoading ? (
                <div className="space-y-2 p-3">
                  {[1, 2, 3, 4].map((n) => (
                    <Skeleton key={n} className="h-14" />
                  ))}
                </div>
              ) : filteredCandidates.length ? (
                filteredCandidates.map((candidate) => (
                  <button
                    key={candidate.id}
                    disabled={selectedIds.length >= 8}
                    onClick={() => toggleCandidate(candidate.id)}
                    className="flex w-full items-start justify-between gap-2 border-b border-edge px-4 py-3 text-left last:border-0 hover:bg-surface-soft disabled:opacity-50"
                  >
                    <span className="min-w-0">
                      <span className="block truncate text-[13px] font-semibold text-ink">
                        {candidate.name}
                      </span>
                      <span className="mt-0.5 block text-[11px] text-ink-muted">
                        {formatDate(candidate.local_date)} ·{' '}
                        {formatDistance(candidate.distance_m)}
                      </span>
                      <span className="mt-0.5 block text-[11px] text-ink-muted">
                        {formatPace(candidate.pace_s_per_km)} /km ·{' '}
                        {candidate.avg_hr == null ? '—' : `${formatHr(candidate.avg_hr)} bpm`}
                      </span>
                    </span>
                    <span className="shrink-0 space-y-1 text-right">
                      {candidate.has_profile ? (
                        <Badge tone="positive">Profile</Badge>
                      ) : (
                        <Badge tone="neutral">No profile</Badge>
                      )}
                      <span className="block text-[10px] text-brand-600">Add</span>
                    </span>
                  </button>
                ))
              ) : (
                <EmptyState
                  title={search ? 'No matching runs' : 'No more runs'}
                  description={
                    search ? 'Try another name or date.' : 'All available runs are selected.'
                  }
                />
              )}
            </div>
          </Card>
        </aside>
      </div>
    </div>
  )
}
