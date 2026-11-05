/**
 * Progress.
 *
 * The trajectory, forgivingly. A week with nothing in it is a gap in the
 * line, not a failure. There is no percentage without a real baseline, no
 * grade, and no counter for a week that had no plan.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import {
  Bar,
  BarChart,
  Cell,
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
import { axisTicks, calendarWeekStarts } from '../lib/chart'
import { formatCount, formatDateShort, renderedText } from '../lib/format'
import { useStaleness } from '../lib/staleness'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { PageHeader } from '../ui/layout/PageHeader'
import { QueryError } from '../components/QueryError'
import { runsPath, shiftDate } from '../lib/history'

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
const RANGES = [4, 8, 12, 26, 52] as const

export function ProgressPage() {
  const [range, setRange] = useState<number>(12)
  const [chosenWeek, setChosenWeek] = useState('')
  const navigate = useNavigate()
  const progress = useQuery({
    queryKey: ['insights-progress', range],
    queryFn: () => api.insightsProgress(range),
  })
  const stale = useStaleness()
  const data = progress.data

  if (progress.isError && !data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <section className="card px-4 py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void progress.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      </div>
    )
  }

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

  const weeks = calendarWeekStarts(data.date, range)
  const selectedWeek = weeks.includes(chosenWeek) ? chosenWeek : weeks[weeks.length - 1]!
  const volume = weeks.map((date) => {
    const week = data.weekly_volume.find((entry) => entry.week_start === date)
    return {
      label: formatDateShort(date),
      km: week?.distance.raw == null ? null : week.distance.raw / 1000,
      distanceText: week?.distance.text ?? null,
    }
  })
  const pace = weeks.map((date) => {
    const week = data.pace_at_hr.find((entry) => entry.week_start === date)
    return {
      label: formatDateShort(date),
      paceValue: week?.pace?.raw ?? null,
      paceText: week?.pace?.text ?? null,
      hrValue: week?.heart_rate?.raw ?? null,
      hrText: week?.heart_rate?.text ?? null,
    }
  })
  const consistency = weeks.map((date) => ({
    label: formatDateShort(date),
    sessions: data.consistency.find((entry) => entry.week_start === date)?.sessions ?? null,
  }))

  /**
   * The axes' labels.
   *
   * The volume's and the pace's are the core's own words — "38.6 km",
   * "5:12 /km" — so every label carries its unit. The sessions are a
   * count the api sends as a plain number, so their unit is the app's.
   */
  const volumeTicks = axisTicks(
    volume.map((week) => ({ value: week.km, text: week.distanceText })),
  )
  const paceTicks = axisTicks(
    pace.map((week) => ({ value: week.paceValue ?? null, text: week.paceText ?? null })),
  )

  return (
    <div>
      <PageHeader
        title={t('progress.heading')}
        meta={
          <>
            {t('progress.question')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />
      {progress.isError ? (
        <QueryError
          retry={() => {
            void progress.refetch()
          }}
        />
      ) : null}

      {/* ------------------------------------------------- the headline */}
      <section className="card px-4 py-3" data-testid="progress-headline">
        {/* The headline is the core's sentence (core-contract §8). */}
        <p className="text-[19px] font-semibold leading-tight text-ink">{data.headline}</p>
        <p className="mt-1 text-[12px] text-ink-muted">
          {formatDateShort(data.period.from)} - {formatDateShort(data.period.to)}
        </p>
        <dl
          className="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-4"
          data-testid="progress-summary"
        >
          {[
            { label: t('vital.distance'), value: renderedText(data.period.distance) },
            { label: t('vital.time'), value: renderedText(data.period.duration) },
            { label: t('progress.summary.sessions'), value: formatCount(data.period.sessions) },
            {
              label: t('progress.summary.weeks'),
              value: formatCount(data.period.recorded_weeks),
            },
          ].map(({ label, value }) => (
            <div key={label}>
              <dt className="text-[12px] text-ink-muted">{label}</dt>
              <dd className="mt-1 text-[24px] font-semibold tabular-nums text-brand-700">
                {value}
              </dd>
            </div>
          ))}
        </dl>
        <p className="mt-1 text-[13px] text-ink-muted">{t('progress.scope')}</p>
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
              aria-pressed={range === weeks}
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

      <div className="mt-3 flex flex-wrap items-end gap-3">
        <label className="flex flex-col gap-1 text-[12px] text-ink-muted">
          {t('progress.exploreWeek')}
          <select
            className="tap rounded-md border border-edge bg-surface px-2 text-[13px] text-ink"
            value={selectedWeek}
            onChange={(event) => setChosenWeek(event.target.value)}
          >
            {weeks
              .slice()
              .reverse()
              .map((date) => (
                <option key={date} value={date}>
                  {formatDateShort(date)}
                </option>
              ))}
          </select>
        </label>
        <Link
          className="tap-inline text-[13px] font-medium text-brand-700 underline"
          to={runsPath(selectedWeek, shiftDate(selectedWeek, 6))}
        >
          {t('progress.viewRuns')}
        </Link>
      </div>
      <div className="progress-grid mt-3">
        {/* --------------------------------------------- weekly volume */}
        <ChartCard title={t('progress.volume.heading')}>
          <div style={{ width: '100%', height: 180 }}>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={volume} margin={{ top: 4, right: 4, bottom: 0, left: -18 }}>
                <defs>
                  <pattern
                    id="partial-week"
                    width="6"
                    height="6"
                    patternUnits="userSpaceOnUse"
                    patternTransform="rotate(45)"
                  >
                    <rect width="6" height="6" fill="#D6E6F3" />
                    <rect width="3" height="6" fill="#1E5E96" />
                  </pattern>
                </defs>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis
                  dataKey="label"
                  tick={{ fontSize: 11, fill: '#61788D' }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  ticks={volumeTicks.map((tick) => tick.value)}
                  tick={axisTick(volumeTicks)}
                  width={64}
                  tickLine={false}
                  axisLine={false}
                  domain={[0, 'auto']}
                />
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
                <Bar
                  name={t('vital.distance')}
                  dataKey="km"
                  fill="#1E5E96"
                  isAnimationActive={false}
                  cursor="pointer"
                  onClick={(_entry, index) => {
                    const week = weeks[index]
                    if (week) navigate(runsPath(week, shiftDate(week, 6)))
                  }}
                >
                  {volume.map((week, index) => (
                    <Cell
                      key={week.label}
                      fill={index === volume.length - 1 ? 'url(#partial-week)' : '#1E5E96'}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
          <p className="mt-2 text-[12px] text-ink-muted">{t('progress.partial')}</p>
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
                  ticks={paceTicks.map((tick) => tick.value)}
                  tick={axisTick(paceTicks)}
                  width={64}
                  tickLine={false}
                  axisLine={false}
                  domain={['dataMin - 10', 'dataMax + 10']}
                />
                <Tooltip
                  formatter={(_value, _name, item) => {
                    const payload = item?.payload as
                      { paceText?: string | null; hrText?: string | null } | undefined
                    // The tooltip shows the core's words.
                    return t('progress.paceHr', {
                      pace: payload?.paceText ?? '',
                      hr: payload?.hrText ?? '',
                    })
                  }}
                />
                <Line
                  type="monotone"
                  name={t('vital.pace')}
                  dataKey="paceValue"
                  stroke="#1E5E96"
                  strokeWidth={2}
                  dot={false}
                  connectNulls={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
          <p className="mt-2 text-[12px] text-ink-muted">{t('progress.paceNote')}</p>
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
                  width={64}
                  allowDecimals={false}
                  label={{
                    value: t('progress.consistency.unit'),
                    angle: -90,
                    position: 'insideLeft',
                    style: { fontSize: 11, fill: '#61788D' },
                  }}
                />
                <Tooltip />
                <Bar name={t('progress.consistency.unit')} dataKey="sessions" fill="#7BA8CD" />
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
      <p className="mt-1 text-[12px] text-ink-muted">{t('progress.averageNote')}</p>
    </div>
  )
}
