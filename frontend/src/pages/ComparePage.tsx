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
import { formatDateMedium, renderedText } from '../lib/format'
import type { CompareMetric, CompareSeriesPoint } from '../lib/types'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

/** One row of the diff: the metric, A, B and the delta — the core's words. */
function MetricRow({ metric }: { metric: CompareMetric }) {
  return (
    <li
      className="grid grid-cols-[1fr_auto_auto_auto] items-baseline gap-x-3 border-b border-edge px-4 py-2 last:border-b-0"
      data-testid={`compare-row-${metric.key}`}
    >
      <span className="text-[13.5px] text-ink-soft">{t(metric.label_key)}</span>
      <span className="text-[13.5px] font-semibold tabular-nums text-ink">
        {renderedText(metric.a)}
      </span>
      <span className="text-[13.5px] font-semibold tabular-nums text-ink">
        {renderedText(metric.b)}
      </span>
      <span className="text-[13.5px] tabular-nums text-ink-muted">
        {renderedText(metric.delta)}
      </span>
    </li>
  )
}

function EffortChart({
  series,
  name,
}: {
  series: CompareSeriesPoint[]
  name: string
}) {
  const data = series.map((point) => ({
    // The chart's own scale: minutes into the run.
    minute: Math.round(point.offset_s / 60),
    hr: point.heart_rate?.raw ?? null,
    hrText: point.heart_rate?.text ?? null,
    pace: point.pace?.raw ?? null,
    paceText: point.pace?.text ?? null,
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
            <Tooltip
              formatter={(_value, name, item) => {
                const payload = item?.payload as
                  | { hrText?: string | null; paceText?: string | null }
                  | undefined
                // The tooltip shows the core's words, not the chart's
                // numbers.
                if (name === 'hr') return payload?.hrText ?? ''
                return payload?.paceText ?? ''
              }}
            />
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

  /**
   * Which run is A is the API's business.
   *
   * Swapping the columns changes the question — "the other way round" is a
   * different comparison — and the delta's sign is the core's arithmetic.
   * So the app asks the API again, and shows the answer as it comes.
   */
  const other = id === '101' ? '102' : '101'
  const a = swapped ? id : other
  const b = swapped ? other : id

  const comparison = useQuery({
    queryKey: ['compare', a, b],
    queryFn: () => api.compare(a, b),
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

  const runA = data.a
  const runB = data.b
  const rows = data.metrics
  const chartSeries = { a: data.series.a, b: data.series.b }

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
        {data.basis_text}
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
          {rows.map((metric) => (
            <MetricRow key={metric.key} metric={metric} />
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
              {renderedText(run.distance)} · {formatDateMedium(run.date)}
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
