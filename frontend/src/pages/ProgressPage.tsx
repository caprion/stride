/**
 * Progress.
 *
 * The trajectory, forgivingly. A week with nothing in it is a gap in the
 * line, not a failure. There is no percentage without a real baseline, no
 * grade, and no counter for a week that had no plan.
 */

import { useQuery } from '@tanstack/react-query'
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
import { formatDateShort, formatNumber } from '../lib/format'
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

export function ProgressPage() {
  const progress = useQuery({ queryKey: ['insights-progress'], queryFn: api.insightsProgress })
  const data = progress.data

  if (!data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const volume = data.weekly_volume.map((week) => ({
    ...week,
    km: week.distance_m / 1000,
    label: formatDateShort(week.week_start),
  }))
  const pace = data.pace_at_hr.map((week) => ({
    ...week,
    label: formatDateShort(week.week_start),
  }))
  const consistency = data.consistency.map((week) => ({
    ...week,
    label: formatDateShort(week.week_start),
  }))

  return (
    <div>
      <PageHeader title={t('progress.heading')} />

      {/* ------------------------------------------------- the headline */}
      <section className="card px-4 py-3" data-testid="progress-headline">
        {/* The headline is the core's sentence (core-contract §8). */}
        <p className="text-[19px] font-semibold leading-tight text-ink">
          {data.headline}
        </p>
        {data.headline_note ? (
          <p className="mt-0.5 text-[13.5px] text-ink-muted">{data.headline_note}</p>
        ) : null}
      </section>

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
                <Tooltip formatter={(value: number) => `${value.toFixed(1)} km`} />
                {data.typical_week_m !== null ? (
                  <ReferenceLine
                    y={data.typical_week_m / 1000}
                    stroke="#93A6B7"
                    strokeDasharray="4 3"
                    label={{
                      value: t('progress.typicalWeek'),
                      position: 'insideTopRight',
                      fontSize: 10,
                      fill: '#93A6B7',
                    }}
                  />
                ) : null}
                <Bar dataKey="km" fill="#1E5E96" />
              </BarChart>
            </ResponsiveContainer>
          </div>
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
                <Tooltip />
                <Line
                  type="monotone"
                  dataKey="pace_s_per_km"
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
        {formatNumber(data.weekly_volume.length)} weeks ·{' '}
        {t('progress.sessionsPerWeek', {
          n: Math.round(
            data.consistency.reduce((sum, week) => sum + week.sessions, 0) /
              Math.max(1, data.consistency.length),
          ),
        })}
      </p>
    </div>
  )
}
