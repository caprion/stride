/**
 * The activity detail page.
 *
 * What the run was, what it meant, and how it sat against the plan. The
 * numbers in the header, the numbers in the story and the numbers in the
 * chart are the same numbers.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
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
  formatElevation,
  formatNumber,
  formatPace,
  sportLabel,
} from '../lib/format'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

export function ActivityPage({ id }: { id: string }) {
  const [splitsOpen, setSplitsOpen] = useState(false)
  const [elevationOpen, setElevationOpen] = useState(false)

  const activity = useQuery({ queryKey: ['activity', id], queryFn: () => api.activity(id) })
  const data = activity.data

  if (!data) {
    return (
      <div>
        <PageHeader title={t('activity.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const samples = data.samples.map((sample) => ({
    minute: Math.round(sample.offset_s / 60),
    hr: sample.hr,
    pace: sample.pace_s_per_km,
  }))
  const elevation = data.elevation.map((point) => ({
    km: Number((point.distance_m / 1000).toFixed(1)),
    m: point.elevation_m,
  }))

  return (
    <div>
      <PageHeader
        title={sportLabel(data.sport_type)}
        meta={formatDateMedium(data.local_date)}
      />

      {/* ------------------------------------- the three headline numbers */}
      <section className="card px-4 py-3" data-testid="activity-detail">
        <div className="stat-strip">
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.distance')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {formatDistance(data.distance_m)}
            </span>
          </span>
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.time')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {formatDurationHuman(data.moving_time_s)}
            </span>
          </span>
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.pace')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {formatPace(data.pace_s_per_km)} /km
            </span>
          </span>
        </div>
      </section>

      {/* ------------------------------------------------------- the story */}
      {data.story ? (
        <section className="card mt-3 px-4 py-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.story.heading')}
          </h2>
          <p className="mt-1 text-[14.5px] text-ink-soft">{data.story}</p>
        </section>
      ) : null}

      {/* --------------------------------------------------------- effort */}
      <section className="card mt-3 px-4 py-3">
        <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('activity.effort.heading')}
        </h2>
        <div style={{ width: '100%', height: 150 }}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: -20 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={38} />
              <Tooltip />
              <Line type="monotone" dataKey="hr" stroke="#1E5E96" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        <div style={{ width: '100%', height: 120 }} className="mt-2">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: -20 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={38} />
              <Tooltip />
              <Line
                type="monotone"
                dataKey="pace"
                stroke="#D99A0B"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </section>

      {/* -------------------------------------------------------- splits */}
      <section className="card mt-3 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.splits.heading')}
          </h2>
          <button
            type="button"
            onClick={() => setSplitsOpen((open) => !open)}
            className="tap text-[13px] font-medium text-brand-600 underline underline-offset-2"
          >
            {splitsOpen ? t('activity.hideSplits') : t('activity.showSplits')}
          </button>
        </div>
        {splitsOpen ? (
          <ul className="mt-2">
            {data.splits.map((split) => (
              <li
                key={split.index}
                className="grid grid-cols-[1fr_auto_auto] items-baseline gap-x-3 border-b border-edge py-1.5 last:border-b-0"
              >
                <span className="text-[13px] text-ink-soft">{split.index} km</span>
                <span className="text-[13px] tabular-nums text-ink">
                  {formatDurationHuman(split.duration_s)}
                </span>
                <span className="text-[13px] tabular-nums text-ink-muted">
                  {split.avg_hr === null ? '—' : `${formatNumber(split.avg_hr)} bpm`}
                </span>
              </li>
            ))}
          </ul>
        ) : null}
      </section>

      {/* ------------------------------------------------------ elevation */}
      <section className="card mt-3 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.elevation.heading')}
          </h2>
          <button
            type="button"
            onClick={() => setElevationOpen((open) => !open)}
            className="tap text-[13px] font-medium text-brand-600 underline underline-offset-2"
          >
            {t('activity.showElevation')}
          </button>
        </div>
        {elevationOpen ? (
          <div style={{ width: '100%', height: 140 }} className="mt-2">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={elevation} margin={{ top: 4, right: 4, bottom: 0, left: -14 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis dataKey="km" tick={{ fontSize: 11, fill: '#61788D' }} />
                <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={40} />
                <Tooltip formatter={(value: number) => formatElevation(value)} />
                <Area type="monotone" dataKey="m" stroke="#3F6E9E" fill="#D6E6F3" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        ) : null}
      </section>

      {/* -------------------------------------------------- against the plan */}
      {data.plan_line ? (
        <section className="card mt-3 px-4 py-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.plan.heading')}
          </h2>
          <p className="mt-1 text-[14px] text-ink-soft">
            {t('activity.plan.line', {
              actual: formatDistance(data.plan_line.actual_distance_m),
              target: formatDistance(data.plan_line.target_distance_m),
            })}
          </p>
        </section>
      ) : null}

      {/* --------------------------------------------------------- actions */}
      <div className="mt-3 flex flex-wrap gap-4 pb-2">
        <a
          href={`/compare/${data.id}`}
          className="tap-inline text-[13px] font-medium text-brand-600 underline underline-offset-2"
        >
          {t('runs.compareWith')}
        </a>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('runs.addNote')}
        </button>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('activity.export')}
        </button>
      </div>
    </div>
  )
}
