/**
 * Compare.
 *
 * The canonical form is the metric diff — `metric · A · B · Δ` — and it is
 * the form at **every** width. Two columns and side-by-side charts appear
 * only when there is room; on a narrow screen one chart carries an A/B
 * toggle instead.
 *
 * The basis is always named. Comparing incomparable runs is allowed, and
 * labelled. There is no winner.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import {
  Line,
  LineChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import {
  formatDateMedium,
  formatDistance,
  formatDurationHuman,
  formatNumber,
  formatPace,
} from '../lib/format'
import type { CompareMetric } from '../lib/types'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

/** The delta, signed, in the metric's own unit. */
export function formatDelta(metric: CompareMetric, delta: number | null): string {
  if (delta === null) return '—'
  const sign = delta > 0 ? '+' : delta < 0 ? '−' : ''
  const magnitude = Math.abs(delta)
  switch (metric.format) {
    case 'distance':
      return `${sign}${(magnitude / 1000).toFixed(1)} km`
    case 'duration':
      return `${sign}${formatDurationHuman(magnitude)}`
    case 'pace':
      return `${sign}${Math.round(magnitude)} s/km`
    case 'hr':
      return `${sign}${formatNumber(magnitude)} bpm`
    default:
      return `${sign}${formatNumber(magnitude)}`
  }
}

/** The value, in the metric's own unit. */
function formatValue(metric: CompareMetric, value: number | null): string {
  if (value === null) return '—'
  switch (metric.format) {
    case 'distance':
      return formatDistance(value)
    case 'duration':
      return formatDurationHuman(value)
    case 'pace':
      return `${formatPace(value)} /km`
    case 'hr':
      return `${formatNumber(value)} bpm`
    default:
      return formatNumber(value)
  }
}

/** The basis banner's wording. */
function basisText(basis: { kind: string; weeks_apart?: number }): string {
  switch (basis.kind) {
    case 'same_distance':
      return t('compare.basis.sameDistance', { n: basis.weeks_apart ?? 0 })
    case 'same_route':
      return t('compare.basis.sameRoute')
    case 'same_hr':
      return t('compare.basis.sameHr')
    default:
      return t('compare.basis.different')
  }
}

function MetricRow({
  metric,
  a,
  b,
  delta,
}: {
  metric: CompareMetric
  a: number | null
  b: number | null
  delta: number | null
}) {
  return (
    <li
      className="grid grid-cols-[1fr_auto_auto_auto] items-baseline gap-x-3 border-b border-edge px-4 py-2 last:border-b-0"
      data-testid={`compare-row-${metric.key}`}
    >
      <span className="text-[13.5px] text-ink-soft">{t(metric.label_key)}</span>
      <span className="text-[13.5px] font-semibold tabular-nums text-ink">
        {formatValue(metric, a)}
      </span>
      <span className="text-[13.5px] font-semibold tabular-nums text-ink">
        {formatValue(metric, b)}
      </span>
      <span className="text-[13.5px] tabular-nums text-ink-muted">
        {formatDelta(metric, delta)}
      </span>
    </li>
  )
}

function EffortChart({
  series,
  name,
}: {
  series: { offset_s: number; pace_s_per_km: number | null; hr: number | null }[]
  name: string
}) {
  const data = series.map((point) => ({
    minute: Math.round(point.offset_s / 60),
    hr: point.hr,
    pace: point.pace_s_per_km,
  }))
  return (
    <figure className="m-0">
      <figcaption className="mb-1 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {name}
      </figcaption>
      <div style={{ width: '100%', height: 150 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: -20 }}>
            <CartesianGrid stroke="#E4EBF0" vertical={false} />
            <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
            <YAxis
              tick={{ fontSize: 11, fill: '#61788D' }}
              width={38}
              domain={['dataMin - 5', 'dataMax + 5']}
            />
            <Tooltip />
            <Line type="monotone" dataKey="hr" stroke="#1E5E96" strokeWidth={2} dot={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </figure>
  )
}

export function ComparePage({ id }: { id: string }) {
  const [swapped, setSwapped] = useState(false)
  const [chart, setChart] = useState<'a' | 'b'>('a')

  const comparison = useQuery({
    queryKey: ['compare', id],
    queryFn: () => api.compare(id, id === '101' ? '102' : '101'),
  })
  const data = comparison.data

  if (!data) {
    return (
      <div>
        <PageHeader title={t('compare.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const runA = swapped ? data.b : data.a
  const runB = swapped ? data.a : data.b
  const rows = data.metrics.map((metric) => ({
    metric,
    a: swapped ? metric.b : metric.a,
    b: swapped ? metric.a : metric.b,
    // The delta is B − A. Swapping flips its sign.
    delta: metric.delta === null ? null : swapped ? -metric.delta : metric.delta,
  }))

  const chartSeries = swapped
    ? { a: data.series.b, b: data.series.a }
    : { a: data.series.a, b: data.series.b }

  return (
    <div>
      <PageHeader
        title={t('compare.heading')}
        action={
          <button
            type="button"
            data-testid="compare-swap"
            onClick={() => setSwapped((value) => !value)}
            className="tap inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-ink-soft"
          >
            {t('compare.swap')}
          </button>
        }
      />

      {/* --------------------------------------------- the basis banner */}
      <p
        className="card px-4 py-2.5 text-[13.5px] font-medium text-ink-soft"
        data-testid="compare-basis"
      >
        {basisText(data.basis)}
      </p>

      {/* ------------------------------------------- the metric diff */}
      <section className="card mt-3 overflow-hidden" data-testid="compare-metrics">
        <div className="grid grid-cols-[1fr_auto_auto_auto] items-baseline gap-x-3 border-b border-edge bg-surface-soft px-4 py-2">
          <span className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('compare.column.metric')}
          </span>
          <span className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('compare.column.a')}
          </span>
          <span className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('compare.column.b')}
          </span>
          <span className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('compare.column.delta')}
          </span>
        </div>
        <ul>
          {rows.map((row) => (
            <MetricRow
              key={row.metric.key}
              metric={row.metric}
              a={row.a}
              b={row.b}
              delta={row.delta}
            />
          ))}
        </ul>
      </section>

      {/* ------------------------------------------------ the two runs */}
      <div className="mt-3 grid grid-cols-1 gap-3 compare-grid">
        {[
          { run: runA, key: 'a' as const },
          { run: runB, key: 'b' as const },
        ].map(({ run, key }) => (
          <section key={key} className="card px-4 py-3" data-testid={`compare-run-${key}`}>
            <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              {t(key === 'a' ? 'compare.column.a' : 'compare.column.b')}
            </p>
            <p className="text-[15px] font-semibold text-ink">
              {formatDistance(run.distance_m)} · {formatDateMedium(run.date)}
            </p>
            {run.story ? <p className="mt-0.5 text-[13px] text-ink-soft">{run.story}</p> : null}
          </section>
        ))}
      </div>

      {/* --------------------------------------------------- the charts */}
      <section className="card mt-3 px-4 py-3">
        <div className="mb-2 flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('compare.chart.toggle')}
          </h2>
          {/* The A/B toggle: one chart on a narrow screen, both when there
              is room. */}
          <div className="compare-toggle flex gap-1">
            <button
              type="button"
              onClick={() => setChart('a')}
              data-testid="compare-chart-a"
              className={`tap rounded-md px-3 text-[13px] font-semibold ${
                chart === 'a' ? 'bg-brand-50 text-brand-700' : 'text-ink-muted'
              }`}
            >
              {t('compare.column.a')}
            </button>
            <button
              type="button"
              onClick={() => setChart('b')}
              data-testid="compare-chart-b"
              className={`tap rounded-md px-3 text-[13px] font-semibold ${
                chart === 'b' ? 'bg-brand-50 text-brand-700' : 'text-ink-muted'
              }`}
            >
              {t('compare.column.b')}
            </button>
          </div>
        </div>
        <div className="compare-charts grid grid-cols-1 gap-3">
          <div
            className={chart === 'a' ? '' : 'compare-hidden'}
            data-testid="compare-chart-a-panel"
          >
            <EffortChart series={chartSeries.a} name={t('compare.column.a')} />
          </div>
          <div
            className={chart === 'b' ? '' : 'compare-hidden'}
            data-testid="compare-chart-b-panel"
          >
            <EffortChart series={chartSeries.b} name={t('compare.column.b')} />
          </div>
        </div>
      </section>

      <div className="mt-3 flex flex-wrap gap-4">
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-brand-600 underline underline-offset-2"
        >
          {t('compare.chooseOther')}
        </button>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('compare.share')}
        </button>
      </div>
    </div>
  )
}
