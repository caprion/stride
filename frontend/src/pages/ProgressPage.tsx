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
import type { CatalogueKey } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { PageHeader } from '../ui/layout/PageHeader'
import { QueryError } from '../components/QueryError'
import { runsPath, shiftDate } from '../lib/history'

/**
 * The reasons the load can have no value, and the line each one gets.
 *
 * The reasons are the **core's** — the api passes the core's own reasons
 * on — and the line is the app's chrome. A reason the app does not know
 * gets the plain one: no value is still no value.
 */
const LOAD_EMPTY: Record<string, CatalogueKey> = {
  no_zone_model: 'progress.load.empty.no_zone_model',
  no_max_heart_rate: 'progress.load.empty.no_max_heart_rate',
  unusable_zone_model: 'progress.load.empty.unusable_zone_model',
  no_history: 'progress.load.empty.no_history',
}

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

/**
 * The words a share's movement is written in.
 *
 * The share says which way it moved against the window before it, and
 * nothing more: "up", "down", "steady". It is not a verdict, and the
 * reader decides what a move means.
 */
const TREND: Record<string, CatalogueKey> = {
  up: 'progress.rolling.trend.up',
  down: 'progress.rolling.trend.down',
  steady: 'progress.rolling.trend.steady',
}

/** An hour of the athlete's own local time, as a clock reads it. */
function hour(value: number): string {
  return `${String(value).padStart(2, '0')}:00`
}

export function ProgressPage() {
  const [range, setRange] = useState<number>(12)
  const [chosenWeek, setChosenWeek] = useState('')
  const navigate = useNavigate()
  const progress = useQuery({
    queryKey: ['insights-progress', range],
    queryFn: () => api.insightsProgress(range),
  })
  // The load has a route of its own, and it follows the same range: the
  // number is never read apart from the control that changes it.
  const load = useQuery({
    queryKey: ['insights-load', range],
    queryFn: () => api.insightsLoad(range),
  })
  // The rolling view follows the same range for the same reason.
  const rolling = useQuery({
    queryKey: ['insights-rolling', range],
    queryFn: () => api.insightsRolling(range),
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
   * The load's own answer.
   *
   * The state of the two series at the last day of the series is the
   * load's present, and the seven-day values are the week. The state is
   * the last day the record covers, which is not always today: the card
   * says which day it is.
   */
  const loadStates = load.data?.load?.states ?? []
  const lastState = loadStates.length > 0 ? loadStates[loadStates.length - 1]! : undefined
  const week = load.data?.load?.week ?? null

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
        {/* ------------------------------------------ the Eddington number */}
        {/*
          The number, the days still needed for the next one and the
          longest distance in a day — over the window the screen is
          showing. The window is named on the card, so the number is never
          read apart from the period it covers.
        */}
        <section className="card px-4 py-3" data-testid="progress-eddington">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.eddington.heading')}
          </h2>
          {data.eddington ? (
            <>
              <p
                className="text-[24px] font-semibold tabular-nums text-brand-700"
                data-testid="progress-eddington-number"
              >
                {formatCount(data.eddington.number)}
              </p>
              <p className="mt-1 text-[12px] text-ink-muted">{t('progress.eddington.note')}</p>
              <ul className="mt-3 space-y-1.5">
                <li className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">
                    {t('progress.eddington.daysToNext')}
                  </span>
                  <span
                    className="text-[13.5px] font-semibold tabular-nums text-ink"
                    data-testid="progress-eddington-days"
                  >
                    {formatCount(data.eddington.days_needed_for_next)}
                  </span>
                </li>
                <li className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">
                    {t('progress.eddington.longestDay')}
                  </span>
                  <span
                    className="text-[13.5px] font-semibold tabular-nums text-ink"
                    data-testid="progress-eddington-longest"
                  >
                    {renderedText(data.eddington.longest_day)}
                  </span>
                </li>
              </ul>
            </>
          ) : (
            /* No distance in the window: there is no number, and the card
               says so rather than showing a zero. */
            <p className="text-[13px] text-ink-muted" data-testid="progress-eddington-empty">
              {t('progress.eddington.empty')}
            </p>
          )}
          <p
            className="mt-2 text-[12px] text-ink-faint"
            data-testid="progress-eddington-window"
          >
            {formatDateShort(data.period.from)} - {formatDateShort(data.period.to)}
          </p>
        </section>

        {/* ------------------------------------------ the training load */}
        {/*
          The load, as values and a scenario.

          The card says what the numbers are, what window they describe
          and what the model assumed — the coefficient, where the maximum
          heart rate came from and the day the zone model was read, and
          the two windows. There is no score, no grade, no verdict and no
          instruction: the band of the ratio is named as a range, and the
          reader decides what it means.

          A load with no value says so, with the reason: never a zero.
        */}
        <section className="card px-4 py-3" data-testid="progress-load">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.load.heading')}
          </h2>
          {load.isError && !load.data ? (
            <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
          ) : null}
          {!load.data ? (
            <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
          ) : null}
          {load.data?.load ? (
            <>
              {lastState ? (
                <p className="text-[12px] text-ink-muted" data-testid="progress-load-as-of">
                  {t('progress.load.asOf', { date: lastState.label })}
                </p>
              ) : null}
              <dl
                className="mt-2 grid grid-cols-2 gap-x-3 gap-y-2"
                data-testid="progress-load-values"
              >
                {[
                  { key: 'acute', label: t('progress.load.acute'), value: lastState?.acute },
                  {
                    key: 'chronic',
                    label: t('progress.load.chronic'),
                    value: lastState?.chronic,
                  },
                  { key: 'form', label: t('progress.load.form'), value: lastState?.form },
                  { key: 'ratio', label: t('progress.load.ratio'), value: lastState?.ratio },
                ].map(({ key, label, value }) => (
                  <div key={key}>
                    <dt className="text-[12px] text-ink-muted">{label}</dt>
                    <dd
                      className="mt-0.5 text-[19px] font-semibold tabular-nums text-ink"
                      data-testid={`progress-load-${key}`}
                    >
                      {renderedText(value ?? null)}
                    </dd>
                  </div>
                ))}
              </dl>
              {load.data.scenario ? (
                <p className="mt-2 text-[12px] text-ink-muted" data-testid="progress-load-band">
                  {t('progress.load.band', { band: load.data.scenario.band })}
                </p>
              ) : null}
              {week ? (
                <>
                  <h3 className="mt-3 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                    {t('progress.load.week.heading')}
                  </h3>
                  <dl className="mt-1.5 grid grid-cols-2 gap-x-3 gap-y-2">
                    {[
                      {
                        key: 'impulse',
                        label: t('progress.load.impulse'),
                        value: week.impulse,
                      },
                      {
                        key: 'monotony',
                        label: t('progress.load.monotony'),
                        value: week.monotony,
                      },
                      { key: 'strain', label: t('progress.load.strain'), value: week.strain },
                    ].map(({ key, label, value }) => (
                      <div key={key}>
                        <dt className="text-[12px] text-ink-muted">{label}</dt>
                        <dd
                          className="mt-0.5 text-[16px] font-semibold tabular-nums text-ink"
                          data-testid={`progress-load-${key}`}
                        >
                          {renderedText(value)}
                        </dd>
                      </div>
                    ))}
                  </dl>
                </>
              ) : null}
              {/* The scenario and its assumption: the core's own words,
                  and the assumption travels with them. */}
              {load.data.scenario ? (
                <>
                  <p
                    className="mt-3 text-[12.5px] text-ink-soft"
                    data-testid="progress-load-scenario"
                  >
                    {load.data.scenario.statement}
                  </p>
                  <p
                    className="mt-1 text-[11.5px] text-ink-faint"
                    data-testid="progress-load-assumption"
                  >
                    {load.data.scenario.assumption}
                  </p>
                </>
              ) : null}
            </>
          ) : null}
          {load.data && !load.data.load ? (
            /* No value is the absence of the value: the card says why,
               and shows no number at all. */
            <p className="text-[13px] text-ink-muted" data-testid="progress-load-empty">
              {t(LOAD_EMPTY[load.data.unavailable ?? ''] ?? 'progress.load.empty.other')}
            </p>
          ) : null}
          {load.data ? (
            <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-load-window">
              {t('progress.load.window', {
                from: load.data.window.from_label,
                to: load.data.window.to_label,
              })}
            </p>
          ) : null}
        </section>

        {/* --------------------------------------- the rolling window */}
        {/*
          The window against the window before it.

          A comparison, not a verdict: the two numbers side by side and
          the difference between them. The card names the window it
          describes, so a number is never read apart from the period it
          covers.
        */}
        <section className="card px-4 py-3" data-testid="progress-rolling">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.heading')}
          </h2>
          {rolling.isError && !rolling.data ? (
            <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
          ) : null}
          {!rolling.data ? (
            <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
          ) : null}
          {rolling.data ? (
            <>
              <p className="text-[12px] text-ink-muted" data-testid="progress-rolling-window">
                {t('progress.rolling.window', { days: rolling.data.window.days })} ·{' '}
                {t('progress.rolling.range', {
                  from: rolling.data.window.from_label,
                  to: rolling.data.window.to_label,
                })}
              </p>
              <dl className="mt-2 space-y-2" data-testid="progress-rolling-values">
                {[
                  {
                    key: 'distance',
                    label: t('vital.distance'),
                    current: renderedText(rolling.data.distance.current),
                    previous: renderedText(rolling.data.distance.previous),
                    difference: renderedText(rolling.data.distance.difference),
                  },
                  {
                    key: 'duration',
                    label: t('vital.time'),
                    current: renderedText(rolling.data.duration.current),
                    previous: renderedText(rolling.data.duration.previous),
                    difference: renderedText(rolling.data.duration.difference),
                  },
                  {
                    key: 'sessions',
                    label: t('progress.rolling.sessions'),
                    current: formatCount(rolling.data.sessions.current),
                    previous: formatCount(rolling.data.sessions.previous),
                    difference: renderedText(rolling.data.sessions.difference),
                  },
                ].map((row) => (
                  <div key={row.key}>
                    <dt className="text-[12px] text-ink-muted">{row.label}</dt>
                    <dd
                      className="text-[19px] font-semibold tabular-nums text-ink"
                      data-testid={`progress-rolling-${row.key}`}
                    >
                      {row.current}
                    </dd>
                    <p className="text-[12px] text-ink-muted">
                      {t('progress.rolling.before', { days: rolling.data.window.days })}:{' '}
                      {row.previous}
                    </p>
                    <p className="text-[12px] text-ink-muted">
                      {t('progress.rolling.difference')}: {row.difference}
                    </p>
                  </div>
                ))}
              </dl>
            </>
          ) : null}
        </section>

        {/* ------------------------------------------ the split of zones */}
        {/*
          How the training was distributed across the athlete's own
          zones: low (Z1–Z2), moderate (Z3), high (Z4–Z5). The shares
          are facts about the window, and each says which way it moved
          against the window before it.

          There is no target range here: a range that says how the split
          ought to be is a prescription, and this product does not
          prescribe. The model the split was measured against is named
          with the day it was read.
        */}
        <section className="card px-4 py-3" data-testid="progress-zones">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.zones.heading')}
          </h2>
          {rolling.data?.zones.shares ? (
            <>
              <ul className="space-y-1.5" data-testid="progress-zones-shares">
                {rolling.data.zones.shares.map((share) => (
                  <li key={share.key} className="flex items-baseline justify-between gap-3">
                    <span className="text-[13.5px] text-ink-soft">{share.label}</span>
                    <span className="flex items-baseline gap-2">
                      <span
                        className="text-[16px] font-semibold tabular-nums text-ink"
                        data-testid={`progress-zones-${share.key}`}
                      >
                        {renderedText(share.share)}
                      </span>
                      {share.trend ? (
                        <span className="text-[11.5px] text-ink-muted">
                          {t(TREND[share.trend] ?? 'progress.rolling.trend.steady', {
                            days: rolling.data.window.days,
                          })}
                        </span>
                      ) : null}
                    </span>
                  </li>
                ))}
              </ul>
              {rolling.data.zones.model?.source ? (
                <p
                  className="mt-2 text-[11.5px] text-ink-faint"
                  data-testid="progress-zones-model"
                >
                  {rolling.data.zones.model.source}
                </p>
              ) : null}
              <p
                className="mt-1 text-[12px] text-ink-faint"
                data-testid="progress-zones-window"
              >
                {t('progress.rolling.window', { days: rolling.data.window.days })}
              </p>
            </>
          ) : null}
          {rolling.data && !rolling.data.zones.shares ? (
            /* No value is the absence of the value: the card says why,
               and shows no number at all. */
            <p className="text-[13px] text-ink-muted" data-testid="progress-zones-empty">
              {rolling.data.zones.unavailable ?? t('common.error')}
            </p>
          ) : null}
        </section>

        {/* --------------------------------------- the zone distribution */}
        {/*
          The distribution of the time across the athlete's own zones:
          how long they spent in each of the five, and what share of the
          time in the zones that was.

          The ranges are the model's own floors, and the model is named
          below — a band that cannot say where it came from is not
          inspectable.
        */}
        <section className="card px-4 py-3" data-testid="progress-zones-distribution">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.zones.distribution')}
          </h2>
          {rolling.data?.zones.distribution ? (
            <>
              <ul className="space-y-1.5" data-testid="progress-zones-distribution-rows">
                {rolling.data.zones.distribution.map((zone) => (
                  <li key={zone.zone}>
                    <div className="flex items-baseline justify-between gap-3">
                      <span className="text-[13.5px] text-ink-soft">
                        {zone.zone}
                        {zone.range ? (
                          <span className="ml-1 text-[11.5px] text-ink-faint">
                            {zone.range}
                          </span>
                        ) : null}
                      </span>
                      <span className="flex items-baseline gap-2 text-[13.5px] tabular-nums text-ink">
                        <span>{renderedText(zone.time)}</span>
                        <span className="text-ink-muted">{renderedText(zone.share)}</span>
                      </span>
                    </div>
                    {/* The bar is the share itself: the width is the
                        proportion of the time in the zones. */}
                    <div className="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-brand-50">
                      <div
                        className="h-full rounded-full bg-brand-600"
                        style={{
                          width: `${Math.max(0, Math.min(100, zone.share?.raw ?? 0))}%`,
                        }}
                        data-testid={`progress-zones-bar-${zone.zone}`}
                      />
                    </div>
                  </li>
                ))}
              </ul>
              {rolling.data.zones.model?.source ? (
                <p
                  className="mt-2 text-[11.5px] text-ink-faint"
                  data-testid="progress-zones-model"
                >
                  {rolling.data.zones.model.source}
                </p>
              ) : null}
            </>
          ) : null}
          {rolling.data && !rolling.data.zones.distribution ? (
            <p
              className="text-[13px] text-ink-muted"
              data-testid="progress-zones-distribution-empty"
            >
              {rolling.data.zones.unavailable ?? t('common.error')}
            </p>
          ) : null}
        </section>

        {/* --------------------------------------- the distance breakdown */}
        {/*
          The runs by how far they went: the buckets, their runs, their
          distance, their climb and their time. The bucket size is taken
          from the longest run in the window, so the buckets are the
          athlete's own scale.
        */}
        <section className="card px-4 py-3" data-testid="progress-distances">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.distances.heading')}
          </h2>
          <p className="text-[12px] text-ink-faint" data-testid="progress-distances-window">
            {t('progress.rolling.window', { days: rolling.data?.window.days ?? 0 })}
          </p>
          <ul className="mt-1.5 space-y-1">
            {rolling.data?.distances.map((bucket) => (
              <li key={bucket.key} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{bucket.label}</span>
                <span className="flex items-baseline gap-2 text-[13.5px] tabular-nums text-ink">
                  <span className="text-ink-muted">{formatCount(bucket.sessions)}</span>
                  <span>{renderedText(bucket.distance)}</span>
                </span>
              </li>
            ))}
          </ul>
        </section>

        {/* -------------------------------------------- the training profile */}
        {/*
          Seven facets of the athlete's shape over the window: the
          volume, the consistency, the intensity, the duration, the
          density, the sport variety and the days of the week.

          **Not a radar.** A polygon over fixed anchors is a claim about a
          shape, and the anchors are somebody else's coaching
          assumptions. These are values, and each says which way it moved
          against the window before it. The reader draws the shape.
        */}
        <section className="card px-4 py-3" data-testid="progress-profile">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.profile.heading')}
          </h2>
          <ul className="space-y-1.5" data-testid="progress-profile-facets">
            {rolling.data?.profile.map((facet) => (
              <li key={facet.key} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{facet.label}</span>
                <span className="flex items-baseline gap-2">
                  <span
                    className="text-[16px] font-semibold tabular-nums text-ink"
                    data-testid={`progress-profile-${facet.key}`}
                  >
                    {renderedText(facet.value)}
                  </span>
                  {facet.movement ? (
                    <span className="text-[11.5px] text-ink-muted">
                      {t(TREND[facet.movement] ?? 'progress.rolling.trend.steady', {
                        days: rolling.data?.window.days ?? 0,
                      })}
                    </span>
                  ) : null}
                </span>
              </li>
            ))}
          </ul>
          <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-profile-window">
            {t('progress.rolling.window', { days: rolling.data?.window.days ?? 0 })}
          </p>
        </section>

        {/* -------------------------------------------- the weekday split */}
        <section className="card px-4 py-3" data-testid="progress-weekday">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.weekday.heading')}
          </h2>
          <p className="text-[12px] text-ink-faint">{t('progress.rolling.history')}</p>
          <ul className="mt-1.5 space-y-1">
            {rolling.data?.weekday.map((day) => (
              <li key={day.weekday} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{day.weekday}</span>
                <span className="flex items-baseline gap-2 text-[13px] tabular-nums text-ink">
                  <span className="text-ink-muted">{formatCount(day.sessions)}</span>
                  <span>{renderedText(day.distance)}</span>
                </span>
              </li>
            ))}
          </ul>
        </section>

        {/* --------------------------------------------- the daytime split */}
        <section className="card px-4 py-3" data-testid="progress-daytime">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.daytime.heading')}
          </h2>
          <p className="text-[12px] text-ink-faint">{t('progress.rolling.history')}</p>
          <ul className="mt-1.5 space-y-1.5">
            {rolling.data?.daytime.map((bucket) => (
              <li key={bucket.key}>
                <div className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">{bucket.label}</span>
                  <span className="text-[13.5px] font-semibold tabular-nums text-ink">
                    {renderedText(bucket.distance)}
                  </span>
                </div>
                <p className="text-[11.5px] text-ink-faint">
                  {t('progress.rolling.hours', {
                    from: hour(bucket.from_hour),
                    to: hour(bucket.to_hour),
                  })}{' '}
                  · {formatCount(bucket.sessions)}
                </p>
              </li>
            ))}
          </ul>
        </section>

        {/* ------------------------------------------------- the months */}
        <section className="card px-4 py-3" data-testid="progress-months">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.months.heading')}
          </h2>
          {rolling.data && rolling.data.months.length === 0 ? (
            <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
          ) : null}
          <ul className="space-y-1">
            {rolling.data?.months.map((month) => (
              <li key={month.key} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{month.label}</span>
                <span className="flex items-baseline gap-2 text-[13.5px] tabular-nums text-ink">
                  <span className="text-ink-muted">{formatCount(month.sessions)}</span>
                  <span>{renderedText(month.distance)}</span>
                </span>
              </li>
            ))}
          </ul>
        </section>


        {/* ------------------------------------------------- the year */}
        {/*
          The year, as a film: the twelve months, the calendar of the
          days coloured by the zones they were run in, and what changed
          against the athlete's own past.

          It is a page of its own, reached from here. The four
          destinations stay the four, and the page says nothing about
          where the athlete ran.
        */}
        <section className="card px-4 py-3" data-testid="progress-year">
          <h2 className="mb-1 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('year.heading')}
          </h2>
          <p className="text-[13px] text-ink-soft">{t('year.intro')}</p>
          <Link
            className="tap-inline mt-2 inline-block text-[13px] font-medium text-brand-700 underline"
            to="/year"
            data-testid="progress-year-link"
          >
            {t('year.open')}
          </Link>
        </section>
        {/* -------------------------------------------------- the years */}
        <section className="card px-4 py-3" data-testid="progress-years">
          <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.rolling.years.heading')}
          </h2>
          {rolling.data && rolling.data.years.length === 0 ? (
            <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
          ) : null}
          <ul className="space-y-1">
            {rolling.data?.years.map((year) => (
              <li key={year.key} className="flex items-baseline justify-between gap-3">
                <span className="text-[13.5px] text-ink-soft">{year.label}</span>
                <span className="flex items-baseline gap-2 text-[13.5px] tabular-nums text-ink">
                  <span className="text-ink-muted">{formatCount(year.sessions)}</span>
                  <span>{renderedText(year.distance)}</span>
                </span>
              </li>
            ))}
          </ul>
        </section>

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
