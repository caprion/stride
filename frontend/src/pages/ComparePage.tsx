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
import { Link, useSearchParams } from 'react-router-dom'
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
import { canDeliverFile, downloadFile, fileName } from '../lib/download'
import { formatDateMedium, renderedText } from '../lib/format'
import { comparisonDocument } from '../lib/share'
import { useStaleness } from '../lib/staleness'
import type { CompareCandidate, CompareMetric, CompareSeriesPoint } from '../lib/types'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { PageHeader } from '../ui/layout/PageHeader'
import { QueryError } from '../components/QueryError'
import { historyReturn } from '../lib/history'

/** One row of the diff: the metric, A, B and the delta — the core's words. */
function MetricRow({ metric }: { metric: CompareMetric }) {
  return (
    <tr
      className="border-b border-edge last:border-b-0"
      data-testid={`compare-row-${metric.key}`}
    >
      <th scope="row" className="px-2 py-3 text-left text-[12px] font-medium text-ink-soft">
        {t(metric.label_key)}
      </th>
      <td className="px-2 py-3 text-right text-[12px] font-semibold tabular-nums text-ink">
        {renderedText(metric.a)}
      </td>
      <td className="px-2 py-3 text-right text-[12px] font-semibold tabular-nums text-ink">
        {renderedText(metric.b)}
      </td>
      <td className="px-2 py-3 text-right text-[12px] tabular-nums text-ink-muted">
        {renderedText(metric.delta)}
      </td>
    </tr>
  )
}

function EffortChart({
  series,
  name,
  metric,
}: {
  series: CompareSeriesPoint[]
  name: string
  metric: 'heart_rate' | 'pace'
}) {
  const data = series.map((point) => ({
    // The chart's own scale: minutes into the run.
    minute: point.offset_s / 60,
    hr: point.heart_rate?.raw ?? null,
    hrText: point.heart_rate?.text ?? null,
    pace: point.pace?.raw ?? null,
    paceText: point.pace?.text ?? null,
  }))
  // The axis's labels are the core's own words: "148 bpm".
  const ticks = axisTicks(
    series.map((point) => ({
      value: point[metric]?.raw ?? null,
      text: point[metric]?.text ?? null,
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
            <XAxis
              dataKey="minute"
              type="number"
              domain={[0, 'dataMax']}
              tickCount={5}
              tick={{ fontSize: 11, fill: '#61788D' }}
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks)}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={['dataMin - 5', 'dataMax + 5']}
              reversed={metric === 'pace'}
            />
            <Tooltip
              formatter={(_value, _name, item) => {
                const payload = item?.payload as
                  { hrText?: string | null; paceText?: string | null } | undefined
                // The tooltip shows the core's words, not the chart's
                // numbers.
                return (metric === 'pace' ? payload?.paceText : payload?.hrText) ?? ''
              }}
            />
            <Line
              type="monotone"
              name={t(metric === 'pace' ? 'vital.pace' : 'vital.heartRate')}
              dataKey={metric === 'pace' ? 'pace' : 'hr'}
              stroke={metric === 'pace' ? '#D99A0B' : '#1E5E96'}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
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
  const [params, setParams] = useSearchParams()
  const [swapped, setSwapped] = useState(false)
  const [chart, setChart] = useState<'a' | 'b'>('a')
  const chosen = params.get('with')
  const returnTo = historyReturn(params.get('returnTo'))
  const [metric, setMetric] = useState<'heart_rate' | 'pace'>('heart_rate')
  const [picking, setPicking] = useState(false)
  const stale = useStaleness()
  const back = (
    <Link to={returnTo} className="tap-inline mb-2 text-[13px] text-brand-700 underline">
      {t('activity.back')}
    </Link>
  )

  /**
   * The athlete's real runs.
   *
   * The candidates route is the one that knows them; the app asks.
   */
  const candidates = useQuery({
    queryKey: ['compare-candidates', id],
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
    return swapped ? { a: later.id, b: earlier.id } : { a: earlier.id, b: later.id }
  }, [current, partner, swapped])

  const comparison = useQuery({
    queryKey: ['compare', pair?.a, pair?.b],
    queryFn: () => api.compare(pair?.a ?? '', pair?.b ?? ''),
    enabled: pair !== null,
  })
  const data = comparison.data

  if (candidates.isError || comparison.isError) {
    return (
      <>
        {back}
        <PageHeader title={t('compare.heading')} />
        <QueryError
          retry={() => {
            void candidates.refetch()
            if (pair) void comparison.refetch()
          }}
        />
      </>
    )
  }
  if (candidates.isSuccess && (!current || !partner)) {
    return (
      <>
        {back}
        <PageHeader title={t('compare.heading')} />
        <section className="card px-4 py-6" data-testid="compare-none">
          <p>{t('compare.state.none')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('compare.state.noneBody')}</p>
        </section>
      </>
    )
  }
  if (!data) {
    return (
      <div>
        {back}
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
        {back}
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
      {back}
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
        <table className="w-full table-fixed">
          <thead>
            <tr className="border-b border-edge bg-surface-soft text-[11px] text-ink-muted">
              <th scope="col" className="w-1/4 px-2 py-2 text-left font-semibold">
                {t('compare.column.metric')}
              </th>
              <th scope="col" className="w-1/4 px-2 py-2 text-right font-semibold">
                {t('compare.column.a')}
              </th>
              <th scope="col" className="w-1/4 px-2 py-2 text-right font-semibold">
                {t('compare.column.b')}
              </th>
              <th scope="col" className="w-1/4 px-2 py-2 text-right font-semibold">
                {t('compare.column.delta')}
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.map((metric) => (
              <MetricRow key={metric.key} metric={metric} />
            ))}
          </tbody>
        </table>
      </section>

      {/* ------------------------------------------------ the two runs */}
      <div className="compare-grid mt-3" data-testid="compare-runs">
        {[
          { run: runA, key: 'a' as const },
          { run: runB, key: 'b' as const },
        ].map(({ run, key }) => (
          <section key={key} className="card px-4 py-3" data-testid={`compare-run-${key}`}>
            <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              {t(key === 'a' ? 'compare.column.a' : 'compare.column.b')}
            </p>
            <p className="text-[15px] font-semibold text-ink">
              <Link
                to={`/runs/${run.id}?${new URLSearchParams({ returnTo })}`}
                className="tap-inline text-brand-700 underline"
              >
                {run.label}
              </Link>
            </p>
            <p className="text-[13px] text-ink-muted">
              {renderedText(run.distance)} · {formatDateMedium(run.date)}
            </p>
            {run.story ? <p className="mt-0.5 text-[13px] text-ink-soft">{run.story}</p> : null}
          </section>
        ))}
      </div>

      {/* --------------------------------------------------- the charts */}
      <section className="card mt-3 px-4 py-3">
        <div className="mb-2 flex flex-wrap items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.effort.heading')}
          </h2>
          <div className="flex items-center gap-2 text-[12px] text-ink-muted">
            <label htmlFor="compare-signal">{t('compare.chart.signal')}</label>
            <select
              id="compare-signal"
              className="tap rounded-md border border-edge bg-surface px-2 text-ink"
              value={metric}
              onChange={(event) =>
                setMetric(event.target.value === 'pace' ? 'pace' : 'heart_rate')
              }
            >
              <option value="heart_rate">{t('vital.heartRate')}</option>
              <option value="pace">{t('vital.pace')}</option>
            </select>
          </div>
          {/* The A/B toggle: one chart on a narrow screen, both when there
              is room. */}
          <div className="compare-toggle gap-1">
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
        <div className="compare-charts">
          <div
            className={chart === 'a' ? '' : 'compare-hidden'}
            data-testid="compare-chart-a-panel"
          >
            <EffortChart
              series={chartSeries.a}
              name={`${t('compare.column.a')} · ${t(metric === 'pace' ? 'vital.pace' : 'vital.heartRate')}`}
              metric={metric}
            />
          </div>
          <div
            className={chart === 'b' ? '' : 'compare-hidden'}
            data-testid="compare-chart-b-panel"
          >
            <EffortChart
              series={chartSeries.b}
              name={`${t('compare.column.b')} · ${t(metric === 'pace' ? 'vital.pace' : 'vital.heartRate')}`}
              metric={metric}
            />
          </div>
        </div>
      </section>

      {/* ------------------------------------------------- the actions */}
      <div className="mt-3 flex flex-wrap gap-4">
        {/*
          The share makes the comparison into a file the athlete keeps,
          and the host delivers it: a download on the desktop, the native
          share sheet on the phone. The phone's sheet is not there yet,
          so the control is only where it can deliver something.
        */}
        {canDeliverFile() ? (
          <button
            type="button"
            data-testid="compare-share"
            className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
            onClick={() => {
              downloadFile(
                fileName(t('compare.share.file'), 'html'),
                comparisonDocument(data),
                'text/html',
              )
            }}
          >
            {t('compare.share')}
          </button>
        ) : null}
      </div>

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
                      setParams((previous) => {
                        previous.set('with', String(candidate.id))
                        return previous
                      })
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
