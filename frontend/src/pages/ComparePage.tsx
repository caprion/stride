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
 *
 * **The runs are the athlete's own.** The screen asks the candidates route
 * which runs there are, and the pair it compares is the one the athlete
 * came from and the one they chose out of that list. There is no id
 * written into this file, and no pair the app made up.
 */

import { useQuery } from '@tanstack/react-query'
import { useMemo, useState } from 'react'
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
import { axisTicks } from '../lib/chart'
import { formatDateMedium, renderedText } from '../lib/format'
import { useStaleness } from '../lib/staleness'
import type { CompareCandidate, CompareMetric, CompareSeriesPoint } from '../lib/types'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
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

function EffortChart({ series, name }: { series: CompareSeriesPoint[]; name: string }) {
  const data = series.map((point) => ({
    // The chart's own scale: minutes into the run.
    minute: Math.round(point.offset_s / 60),
    hr: point.heart_rate?.raw ?? null,
    hrText: point.heart_rate?.text ?? null,
    pace: point.pace?.raw ?? null,
    paceText: point.pace?.text ?? null,
  }))
  // The axis's labels are the core's own words: "148 bpm".
  const ticks = axisTicks(
    series.map((point) => ({
      value: point.heart_rate?.raw ?? null,
      text: point.heart_rate?.text ?? null,
    })),
  )
  return (
    <figure className="m-0">
      <figcaption className="mb-1 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {name}
      </figcaption>
      <div style={{ width: '100%', height: 150 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
            <CartesianGrid stroke="#E4EBF0" vertical={false} />
            <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks)}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={['dataMin - 5', 'dataMax + 5']}
            />
            <Tooltip
              formatter={(_value, _name, item) => {
                const payload = item?.payload as { hrText?: string | null } | undefined
                // The tooltip shows the core's words, not the chart's
                // numbers.
                return payload?.hrText ?? ''
              }}
            />
            <Line
              type="monotone"
              name={t('vital.heartRate')}
              dataKey="hr"
              stroke="#1E5E96"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </figure>
  )
}

/**
 * The runs to compare with, in the order they are offered.
 *
 * The order is the list's, not the basis: **the api names the basis** —
 * "Same distance, 3 weeks apart" or "Different distances — read with care"
 * — and the app does not decide what is comparable. What the app does is
 * leave out the run the athlete is already on, and put the nearest
 * distances first, because that is the run most likely to be worth
 * comparing with.
 */
function rankCandidates(
  candidates: CompareCandidate[],
  current: CompareCandidate | null,
): CompareCandidate[] {
  const currentDistance = current?.distance.raw
  return candidates
    .filter((candidate) => current === null || candidate.id !== current.id)
    .filter((candidate) => candidate.distance.raw !== null)
    .map((candidate) => ({
      candidate,
      // How far the two distances are from each other, in metres.
      gap:
        currentDistance === null || currentDistance === undefined
          ? Number.MAX_SAFE_INTEGER
          : Math.abs((candidate.distance.raw ?? 0) - currentDistance),
    }))
    .sort((left, right) => {
      if (left.gap !== right.gap) return left.gap - right.gap
      // The newer run first, when the distances are the same.
      return right.candidate.date.localeCompare(left.candidate.date)
    })
    .map((entry) => entry.candidate)
}

export function ComparePage({ id }: { id: string }) {
  const [swapped, setSwapped] = useState(false)
  const [chart, setChart] = useState<'a' | 'b'>('a')
  const [chosen, setChosen] = useState<string | null>(null)
  const [picking, setPicking] = useState(false)
  const stale = useStaleness()

  /**
   * The athlete's real runs.
   *
   * The candidates route is the one that knows them; the app asks.
   */
  const candidates = useQuery({
    queryKey: ['compare-candidates'],
    queryFn: () => api.compareCandidates(id),
  })

  const current = useMemo(
    () => (candidates.data ?? []).find((candidate) => String(candidate.id) === id) ?? null,
    [candidates.data, id],
  )
  const others = useMemo(
    () => rankCandidates(candidates.data ?? [], current),
    [candidates.data, current],
  )
  const partner = useMemo(
    () => others.find((candidate) => String(candidate.id) === chosen) ?? others[0] ?? null,
    [others, chosen],
  )

  /**
   * The pair.
   *
   * A is the earlier run and B the later, so the delta reads forward in
   * time; the swap turns it round.
   */
  const pair = useMemo(() => {
    if (!current || !partner) return null
    const earlier = current.date <= partner.date ? current : partner
    const later = current.date <= partner.date ? partner : current
    return swapped
      ? { a: later.id, b: earlier.id }
      : { a: earlier.id, b: later.id }
  }, [current, partner, swapped])

  const comparison = useQuery({
    queryKey: ['compare', pair?.a, pair?.b],
    queryFn: () => api.compare(pair?.a ?? '', pair?.b ?? ''),
    enabled: pair !== null,
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

  // Not enough to compare: the honest version of the empty state, where a
  // comparison is impossible. No numbers are invented to fill the gap.
  if (!runA || !runB || rows.length === 0) {
    return (
      <div>
        <PageHeader title={t('compare.heading')} />
        <div className="card px-4 py-8 text-center" data-testid="compare-none">
          <p className="text-[15px] font-semibold text-ink">{t('compare.state.none')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('compare.state.noneBody')}</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <PageHeader
        title={t('compare.heading')}
        meta={stale ? <span className="text-ink-faint">{stale}</span> : null}
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
      <div className="mt-3 grid grid-cols-1 gap-3 compare-grid" data-testid="compare-runs">
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
          <div className={chart === 'a' ? '' : 'compare-hidden'} data-testid="compare-chart-a-panel">
            <EffortChart
              series={chartSeries.a}
              name={`${t('compare.column.a')} · ${t('vital.heartRate')}`}
            />
          </div>
          <div className={chart === 'b' ? '' : 'compare-hidden'} data-testid="compare-chart-b-panel">
            <EffortChart
              series={chartSeries.b}
              name={`${t('compare.column.b')} · ${t('vital.heartRate')}`}
            />
          </div>
        </div>
      </section>

      {/* --------------------------------------------- the other runs */}
      <div className="mt-3">
        {picking ? (
          <section className="card px-4 py-3" data-testid="compare-picker">
            <div className="flex items-center justify-between gap-3">
              <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                {t('compare.chooseOther')}
              </h2>
              <button
                type="button"
                className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
                onClick={() => {
                  setPicking(false)
                }}
              >
                {t('common.close')}
              </button>
            </div>
            <ul className="mt-2">
              {others.map((candidate) => (
                <li key={candidate.id} className="border-b border-edge last:border-b-0">
                  <button
                    type="button"
                    data-testid={`compare-candidate-${candidate.id}`}
                    className="tap flex w-full items-baseline justify-between gap-3 py-2 text-left"
                    onClick={() => {
                      setChosen(String(candidate.id))
                      setPicking(false)
                    }}
                  >
                    <span className="text-[14px] text-ink">
                      {formatDateMedium(candidate.date)}
                    </span>
                    <span className="text-[13.5px] tabular-nums text-ink-muted">
                      {renderedText(candidate.distance)}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        ) : (
          <button
            type="button"
            data-testid="compare-choose"
            className="tap-inline text-[13px] font-medium text-brand-600 underline underline-offset-2"
            onClick={() => {
              setPicking(true)
            }}
          >
            {t('compare.chooseOther')}
          </button>
        )}
      </div>
    </div>
  )
}
