/**
 * Progress.
 *
 * The trajectory, forgivingly. A week with nothing in it is a gap in the
 * line, not a failure. There is no percentage without a real baseline, no
 * grade, and no counter for a week that had no plan.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import {
  Bar,
  BarChart,
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
import { formatCount, formatDateShort, renderedText } from '../lib/format'
import { useStaleness } from '../lib/staleness'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

/** A chart card with a fixed, legible height. */
function ChartCard({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="card px-4 py-3">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {title}
      </h2>
      {children}
    </section>
  )
}

/**
 * The ranges the charts can show.
 *
 * **The range is the chart's own business** — the api sends the weeks it
 * has, and the app draws the window the athlete asks for. A chart with
 * less behind it shows less; it never invents a week.
 */
const RANGES = [4, 8, 12, 26] as const

export function ProgressPage() {
  const progress = useQuery({ queryKey: ['insights-progress'], queryFn: api.insightsProgress })
  const stale = useStaleness()
  const [range, setRange] = useState<number>(12)
  const data = progress.data

  if (!data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  // Nothing to draw yet: the screen says so, and invents no numbers.
  if (data.weekly_volume.length === 0) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <div className="card px-4 py-8 text-center" data-testid="progress-empty">
          <p className="text-[15px] font-semibold text-ink">{t('progress.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('progress.empty.body')}</p>
        </div>
      </div>
    )
  }

  const volume = data.weekly_volume
    .slice(-range)
    .map((week) => ({
      ...week,
      km: (week.distance.raw ?? 0) / 1000,
      distanceText: week.distance.text,
      label: formatDateShort(week.week_start),
    }))
  const pace = data.pace_at_hr
    .slice(-range)
    .map((week) => ({
      ...week,
      label: formatDateShort(week.week_start),
      paceValue: week.pace?.raw ?? null,
      paceText: week.pace?.text ?? null,
      hrValue: week.heart_rate?.raw ?? null,
      hrText: week.heart_rate?.text ?? null,
    }))
  const consistency = data.consistency
    .slice(-range)
    .map((week) => ({
      ...week,
      label: formatDateShort(week.week_start),
    }))

  return (
    <div>
      <PageHeader
        title={t('progress.heading')}
        meta={stale ? <span className="text-ink-faint">{stale}</span> : null}
      />

      {/* ------------------------------------------------- the headline */}
      <section className="card px-4 py-3" data-testid="progress-headline">
        {/* The headline is the core's sentence (core-contract §8). */}
        <p className="text-[19px] font-semibold leading-tight text-ink">
          {data.headline}
        </p>
        {data.headline_note ? (
          <p className="mt-0.5 text-[13.5px] text-ink-muted">{data.headline_note}</p>
        ) : null}
        {/*
          Not enough history: the charts show what there is, and the note
          says how much more the core wants. No percentage is drawn — a
          percentage from a near-zero baseline is the old bug.
        */}
        {data.weeks_needed !== null ? (
          <p className="mt-1 text-[13px] text-ink-muted" data-testid="progress-needs-more">
            {t('progress.needsMore', { n: data.weeks_needed })}
          </p>
        ) : null}
      </section>

      {/* --------------------------------------------- the range selector */}
      <div className="mt-3 flex flex-wrap items-center gap-2" data-testid="progress-range">
        <span className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('progress.range.label')}
        </span>
        <div className="flex flex-wrap gap-1">
          {RANGES.map((weeks) => (
            <button
              key={weeks}
              type="button"
              onClick={() => {
                setRange(weeks)
              }}
              data-testid={`progress-range-${weeks}`}
              className={`tap rounded-md px-2.5 text-[12.5px] font-semibold ${
                range === weeks ? 'bg-brand-50 text-brand-700' : 'text-ink-muted'
              }`}
            >
              {t('progress.range.weeks', { n: weeks })}
            </button>
          ))}
        </div>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-3 progress-grid">
        {/* --------------------------------------------- weekly volume */}
        <ChartCard title={t('progress.volume.heading')}>
          <div style={{ width: '100%', height: 180 }}>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={volume} margin={{ top: 4, right: 4, bottom: 0, left: -18 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis
                  dataKey="label"
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  interval="preserveStartEnd"
                />
                <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={44} />
                <Tooltip
                  formatter={(_value, _name, item) =>
                    (item?.payload as { distanceText?: string } | undefined)?.distanceText ?? ''
                  }
                />
                {data.typical_week !== null ? (
                  <ReferenceLine
                    y={(data.typical_week.raw ?? 0) / 1000}
                    stroke="#93A6B7"
                    strokeDasharray="4 3"
                  />
                ) : null}
                <Bar dataKey="km" fill="#1E5E96" />
              </BarChart>
            </ResponsiveContainer>
          </div>
          {/* The band, named below the chart rather than inside it: a
              label on the line itself is cut off on a narrow screen. */}
          {data.typical_week !== null ? (
            <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-typical-week">
              {t('progress.typicalWeek')} · {renderedText(data.typical_week)}
            </p>
          ) : null}
        </ChartCard>

        {/* -------------------------------------------- pace at heart rate */}
        <ChartCard title={t('progress.paceAtHr.heading')}>
          <div style={{ width: '100%', height: 180 }}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={pace} margin={{ top: 4, right: 4, bottom: 0, left: -6 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis
                  dataKey="label"
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  width={44}
                  domain={['dataMin - 10', 'dataMax + 10']}
                />
                <Tooltip
                  formatter={(_value, name, item) => {
                    const payload = item?.payload as
                      | { paceText?: string | null; hrText?: string | null }
                      | undefined
                    // The tooltip shows the core's words.
                    if (name === 'pace') return payload?.paceText ?? ''
                    return payload?.hrText ?? ''
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="paceValue"
                  stroke="#1E5E96"
                  strokeWidth={2}
                  dot={false}
                  connectNulls={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </ChartCard>

        {/* ------------------------------------------ bests & milestones */}
        <ChartCard title={t('progress.bests.heading')}>
          <ul className="space-y-1.5">
            {data.bests.map((best) => (
              <li key={best.key} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{t(best.key)}</span>
                <span className="text-[13.5px] font-semibold tabular-nums text-ink">
                  {best.value} · {formatDateShort(best.date)}
                </span>
              </li>
            ))}
          </ul>
        </ChartCard>

        {/* ---------------------------------------------- consistency */}
        <ChartCard title={t('progress.consistency.heading')}>
          <div style={{ width: '100%', height: 140 }}>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={consistency} margin={{ top: 4, right: 4, bottom: 0, left: -24 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis
                  dataKey="label"
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  width={36}
                  allowDecimals={false}
                />
                <Tooltip />
                <Bar dataKey="sessions" fill="#7BA8CD" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </ChartCard>

        {/* ------------------------------------------------- adherence */}
        {data.adherence ? (
          <ChartCard title={t('progress.adherence.heading')}>
            <p className="text-[17px] font-semibold text-ink" data-testid="progress-adherence">
              {t('progress.adherence.count', {
                done: data.adherence.done,
                planned: data.adherence.planned,
              })}
            </p>
          </ChartCard>
        ) : null}
      </div>

      <p className="mt-4 text-[12.5px] text-ink-faint">
        {formatCount(data.weekly_volume.length)} weeks ·{' '}
        {t('progress.sessionsPerWeek', {
          n: data.sessions_per_week?.text ?? formatCount(null),
        })}
      </p>
    </div>
  )
}
