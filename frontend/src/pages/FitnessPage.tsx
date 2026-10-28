import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import {
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { ACCENT_DATA, GRID, INK_FAINT, INK_MUTED } from '../lib/palette'
import type { Fingerprint } from '../types_phase2'
import { formatDate, formatDistance } from '../lib/format'
import { ZoneBarChart } from '../components/charts'
import {
  Badge,
  Card,
  CardHeader,
  EmptyState,
  PageHeader,
  SegmentedControl,
  Skeleton,
} from '../components/ui'

type Metric =
  | 'cadence_avg'
  | 'cadence_cv'
  | 'hr_drift_pct'
  | 'decoupling_pct'
  | 'efficiency_first'
  | 'pace_cv'
  | 'form_score'
const metrics: { value: Metric; label: string; simple: string }[] = [
  { value: 'hr_drift_pct', label: 'HR drift', simple: 'HR drift' },
  { value: 'decoupling_pct', label: 'Aerobic decoupling', simple: 'decoupling' },
  { value: 'cadence_avg', label: 'Cadence', simple: 'cadence' },
  { value: 'cadence_cv', label: 'Cadence stability', simple: 'cadence stability' },
  // TODO(shared): Add pace_drift_pct to the fitness/trends metric contract and backend.
  { value: 'efficiency_first', label: 'Efficiency factor', simple: 'efficiency' },
  { value: 'pace_cv', label: 'Pace consistency', simple: 'pace consistency' },
  { value: 'form_score', label: 'Form score', simple: 'form score' },
]
const display = (v: number | null | undefined, suffix = '') =>
  v == null ? '—' : `${Number(v).toFixed(1)}${suffix}`
const favorable = (
  value: number | null,
  baseline: number | null,
  better: 'lower' | 'higher',
) =>
  value == null || baseline == null
    ? 'text-ink-muted'
    : (better === 'lower' ? value <= baseline : value >= baseline)
      ? 'text-positive'
      : 'text-negative'
export default function FitnessPage() {
  const [metric, setMetric] = useState<Metric>('hr_drift_pct')
  const trend = useQuery({
    queryKey: ['fitness-trend', metric],
    queryFn: () => api.fitnessTrends(metric),
    retry: false,
  })
  const fingerprints = useQuery({
    queryKey: ['fitness-fingerprint'],
    queryFn: () => api.fitnessFingerprint(),
    retry: false,
  })
  const zones = useQuery({
    queryKey: ['fitness-zones'],
    queryFn: () => api.fitnessZones(),
    retry: false,
  })
  const gear = useQuery({ queryKey: ['gear-list'], queryFn: api.gearList, retry: false })
  const data = trend.data
  const metricLabel = metrics.find((m) => m.value === metric)?.simple ?? metric
  const trendPct = data?.trend_pct
  const improving = data
    ? data.better === 'lower'
      ? (trendPct ?? 0) < 0
      : (trendPct ?? 0) > 0
    : false
  const baseline = data?.baseline ?? null
  const getBaseline = (key: 'decoupling_pct' | 'cadence_cv', items: Fingerprint[]) => {
    const vals = items.map((i) => i[key]).filter((v): v is number => v != null)
    return vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : null
  }
  const rows = fingerprints.data?.items ?? []
  const decoupleBaseline = getBaseline('decoupling_pct', rows)
  const cvBaseline = getBaseline('cadence_cv', rows)
  const excludedWalking = rows.reduce((sum, item) => sum + (item.walking_time_s ?? 0), 0)
  const excludedStanding = rows.reduce((sum, item) => sum + (item.stopped_time_s ?? 0), 0)
  return (
    <div className="mx-auto max-w-7xl p-5">
      <PageHeader
        title="Running fingerprint"
        description="The mechanics that make your running yours — and how they change over time."
      />
      <Card className="mb-4">
        <CardHeader
          title="Metric trend"
          subtitle={
            data
              ? `${data.label} · ${data.better === 'lower' ? 'lower is better' : 'higher is better'} · baseline ${display(data.baseline)} ${data.unit}`
              : 'Choose a metric to explore its recent trend.'
          }
          action={
            <SegmentedControl
              options={metrics.map((m) => ({ label: m.label, value: m.value }))}
              value={metric}
              onChange={setMetric}
            />
          }
        />
        {trend.isLoading ? (
          <div className="p-4">
            <Skeleton className="h-64" />
          </div>
        ) : data ? (
          <div className="grid gap-3 p-4 lg:grid-cols-[1fr_250px]">
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart
                  data={data.points.map((p) => ({
                    ...p,
                    label: formatDate(p.date).slice(0, 6),
                  }))}
                >
                  <CartesianGrid stroke={GRID} vertical={false} />
                  <XAxis dataKey="label" tick={{ fontSize: 10, fill: INK_MUTED }} />
                  <YAxis tick={{ fontSize: 10, fill: INK_MUTED }} />
                  <Tooltip />
                  <ReferenceLine
                    y={baseline ?? undefined}
                    stroke={INK_FAINT}
                    strokeDasharray="4 3"
                    label={{ value: 'Baseline', fontSize: 10, fill: INK_MUTED }}
                  />
                  <Line
                    dataKey="value"
                    name={data.label}
                    stroke={ACCENT_DATA}
                    strokeWidth={2.5}
                    dot={{ r: 3 }}
                    connectNulls
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
            <div className="flex flex-col justify-center rounded-lg bg-surface-soft p-5">
              <p className="text-xs font-semibold uppercase tracking-wide text-ink-muted">
                Change over recent weeks
              </p>
              <strong
                className={`mt-2 text-4xl font-bold ${trendPct == null ? 'text-ink' : improving ? 'text-positive' : 'text-negative'}`}
              >
                {trendPct == null ? '—' : `${trendPct > 0 ? '+' : ''}${trendPct.toFixed(1)}%`}
              </strong>
              <p className="mt-2 text-sm text-ink-soft">
                {trendPct == null
                  ? 'Not enough activity data to calculate a trend.'
                  : `${metricLabel} is ${Math.abs(trendPct).toFixed(0)}% ${trendPct < 0 ? 'down' : 'up'} over the recent period — ${improving ? 'moving in a favorable direction.' : 'worth watching across your next runs.'}`}
              </p>
              <Badge tone={improving ? 'positive' : 'warn'} className="mt-3 w-fit">
                {data.better === 'lower' ? 'Good when low' : 'Good when high'}
              </Badge>
            </div>
          </div>
        ) : (
          <EmptyState
            title="Trend unavailable"
            description="This metric may not have enough activity data yet, or the trend service is offline."
          />
        )}
      </Card>
      {/* ``min-w-0`` matters: a grid item's default minimum width is its
          content's, so without it the scrolling table refuses to shrink and
          the whole page overflows sideways on a phone. */}
      <div className="grid gap-4 xl:grid-cols-[1.5fr_1fr]">
        <div className="min-w-0 space-y-4">
          <Card>
            <CardHeader
              title="Recent fingerprints"
              subtitle="Compare mechanics run by run. Decoupling and cadence variation are better when lower."
            />
            {fingerprints.isLoading ? (
              <div className="space-y-2 p-4">
                {[1, 2, 3, 4].map((i) => (
                  <Skeleton key={i} className="h-9" />
                ))}
              </div>
            ) : rows.length ? (
              <>
                <p className="px-5 pb-3 text-[12px] text-ink-muted">
                  Running only — walking and standing are excluded from every number on this
                  page.
                  {excludedWalking + excludedStanding > 0
                    ? ` Across these runs: ${Math.round(excludedWalking / 60)}m walking and ${Math.round(excludedStanding / 60)}m standing excluded.`
                    : ''}
                </p>
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[900px] text-left text-xs">
                    <thead className="bg-surface-soft text-[10px] uppercase tracking-wide text-ink-muted">
                      <tr>
                        {[
                          'Date',
                          'Run',
                          'Distance',
                          'Cadence',
                          'Cadence CV',
                          'HR drift',
                          'Decoupling',
                          'Pace drift',
                          'Form',
                        ].map((h) => (
                          <th key={h} className="px-3 py-2.5">
                            {h}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-edge">
                      {rows.map((r) => (
                        <FingerprintRow
                          key={r.activity_id}
                          item={r}
                          cvBaseline={cvBaseline}
                          decoupleBaseline={decoupleBaseline}
                        />
                      ))}
                    </tbody>
                  </table>
                </div>
              </>
            ) : (
              <EmptyState
                title="No fingerprints yet"
                description="Run data with cadence and heart-rate streams will build your fingerprint."
              />
            )}
          </Card>
          <Card>
            <CardHeader
              title="Heart-rate zones"
              subtitle={
                zones.data
                  ? `Distribution across ${Math.round(zones.data.total_time_s / 60)} minutes`
                  : 'Time spent in each zone'
              }
            />
            {zones.isLoading ? (
              <div className="p-4">
                <Skeleton className="h-48" />
              </div>
            ) : zones.data?.zones.length ? (
              <ZoneBarChart zones={zones.data.zones} height={250} />
            ) : (
              <EmptyState
                title="No zone breakdown yet"
                description="Zone data appears when activities include heart-rate information."
              />
            )}
          </Card>
        </div>
        <div className="space-y-4">
          <Card>
            <CardHeader title="Shoe rotation" subtitle="Mileage toward retirement" />
            {gear.isLoading ? (
              <div className="space-y-3 p-4">
                {[1, 2].map((i) => (
                  <Skeleton key={i} className="h-20" />
                ))}
              </div>
            ) : gear.data?.items.length ? (
              <div className="space-y-3 p-4">
                {gear.data.items.map((g) => {
                  const pct =
                    g.retire_at_m > 0
                      ? Math.min(100, (g.distance_m / g.retire_at_m) * 100)
                      : null
                  return (
                    <div key={g.id} className="rounded-lg border border-edge p-3">
                      <div className="flex items-start justify-between gap-2">
                        <div>
                          <p className="font-semibold text-ink">{g.name}</p>
                          <p className="text-xs text-ink-muted">
                            {[g.brand, g.model].filter(Boolean).join(' ') || 'Running shoe'}
                          </p>
                        </div>
                        {g.retired || g.status === 'retired' ? (
                          <Badge tone="negative">Retired</Badge>
                        ) : (
                          <Badge tone="neutral">Active</Badge>
                        )}
                      </div>
                      <div className="mt-3 flex justify-between text-xs text-ink-soft">
                        <span>
                          {formatDistance(g.distance_m)} · {g.activity_count} runs
                        </span>
                        <span>
                          Retire at {g.retire_at_m > 0 ? formatDistance(g.retire_at_m) : '—'}
                        </span>
                      </div>
                      <div className="mt-2 h-2 overflow-hidden rounded-full bg-surface-sunken">
                        <div
                          className={`h-full rounded-full ${g.retired || g.status === 'retired' ? 'bg-negative' : 'bg-brand-500'}`}
                          style={{ width: `${pct ?? 0}%` }}
                        />
                      </div>
                    </div>
                  )
                })}
              </div>
            ) : (
              <EmptyState
                title="No gear recorded"
                description="Shoes will appear here when gear data is available."
              />
            )}
          </Card>
        </div>
      </div>
    </div>
  )
}

function FingerprintRow({
  item,
  cvBaseline,
  decoupleBaseline,
}: {
  item: Fingerprint
  cvBaseline: number | null
  decoupleBaseline: number | null
}) {
  return (
    <tr>
      <td className="whitespace-nowrap px-3 py-2.5">
        {item.local_date ? formatDate(item.local_date) : '—'}
      </td>
      <td className="px-3 py-2.5">
        <Link
          to={`/activities/${item.activity_id}`}
          className="font-semibold text-brand-600 hover:underline"
        >
          {item.activity_name || 'Run'}
        </Link>
      </td>
      <td className="px-3 py-2.5">{formatDistance(item.distance_m)}</td>
      <td className="px-3 py-2.5">{display(item.cadence_avg, ' spm')}</td>
      <td className={`px-3 py-2.5 ${favorable(item.cadence_cv, cvBaseline, 'lower')}`}>
        {display(item.cadence_cv, '%')}
      </td>
      <td className="px-3 py-2.5">{display(item.hr_drift_pct, '%')}</td>
      <td
        className={`px-3 py-2.5 ${favorable(item.decoupling_pct, decoupleBaseline, 'lower')}`}
      >
        {display(item.decoupling_pct, '%')}
      </td>
      <td className={`px-3 py-2.5 ${favorable(item.pace_drift_pct, null, 'lower')}`}>
        {display(item.pace_drift_pct, '%')}
      </td>
      <td className="px-3 py-2.5">{display(item.form_score)}</td>
    </tr>
  )
}
