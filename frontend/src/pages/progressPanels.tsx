/**
 * The panels of Progress.
 *
 * Each panel is a component of its own, so that the dashboard can lay
 * them out in its grid and the narrow layout can open **one** of them at
 * a time. A panel knows nothing about the screen around it: it takes the
 * answer it draws and shows it.
 *
 * The wording is the app's own and the core's own. A number the panel
 * shows is the core's rendering — "58.4 km", "5 h 11 m", "+40.2 km" — and
 * a panel that has no value says why, in the core's words, and shows no
 * number at all.
 */

import type { ReactNode } from 'react'
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
import type { LoadInsight, ProgressInsight, RenderedValue, RollingInsight } from '../lib/types'
import { formatCount, formatDateShort, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import { axisTick } from '../components/ChartAxis'

/**
 * The reasons the load can have no value, and the line each one gets.
 *
 * The reasons are the **core's** — the api passes the core's own reasons
 * on — and the line is the app's chrome. A reason the app does not know
 * gets the plain one: no value is still no value.
 */
export const LOAD_EMPTY: Record<string, CatalogueKey> = {
  no_zone_model: 'progress.load.empty.no_zone_model',
  no_max_heart_rate: 'progress.load.empty.no_max_heart_rate',
  unusable_zone_model: 'progress.load.empty.unusable_zone_model',
  no_history: 'progress.load.empty.no_history',
}

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

/** A chart card with a fixed, legible height. */
export function ChartCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="card px-4 py-3">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {title}
      </h2>
      {children}
    </section>
  )
}

/** One week of the weekly volume. */
export interface VolumePoint {
  label: string
  /** The first day of the week, so a week can be opened. */
  weekStart: string
  km: number | null
  distanceText: string | null
}

/** One week of the weekly median pace. */
export interface PacePoint {
  label: string
  paceValue: number | null
  paceText: string | null
  hrValue: number | null
  hrText: string | null
}

/** One week of the session count. */
export interface ConsistencyPoint {
  label: string
  sessions: number | null
}

/* ------------------------------------------------------------ the panels */

/**
 * The Eddington number: the number, the days still needed for the next
 * one and the longest distance in a day — over the window the screen is
 * showing. The window is named on the card, so the number is never read
 * apart from the period it covers.
 */
export function EddingtonPanel({ data }: { data: ProgressInsight }) {
  return (
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
      <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-eddington-window">
        {formatDateShort(data.period.from)} - {formatDateShort(data.period.to)}
      </p>
    </section>
  )
}

/**
 * The load, as values and a scenario.
 *
 * The card says what the numbers are, what window they describe and what
 * the model assumed — the coefficient, where the maximum heart rate came
 * from and the day the zone model was read, and the two windows. There is
 * no score, no grade, no verdict and no instruction: the band of the
 * ratio is named as a range, and the reader decides what it means.
 *
 * A load with no value says so, with the reason: never a zero.
 */
export function LoadPanel({ load, error }: { load: LoadInsight | undefined; error: boolean }) {
  const series = load?.load ?? null
  const states = series?.states ?? []
  const lastState = states.length > 0 ? states[states.length - 1]! : undefined
  const week = series?.week ?? null

  return (
    <section className="card px-4 py-3" data-testid="progress-load">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.load.heading')}
      </h2>
      {error && !load ? (
        <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
      ) : null}
      {!load ? <p className="text-[13px] text-ink-muted">{t('common.loading')}</p> : null}
      {series ? (
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
          {load?.scenario ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="progress-load-band">
              {t('progress.load.band', { band: load.scenario.band })}
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
          {load?.scenario ? (
            <>
              <p
                className="mt-3 text-[12.5px] text-ink-soft"
                data-testid="progress-load-scenario"
              >
                {load.scenario.statement}
              </p>
              <p
                className="mt-1 text-[11.5px] text-ink-faint"
                data-testid="progress-load-assumption"
              >
                {load.scenario.assumption}
              </p>
            </>
          ) : null}
        </>
      ) : null}
      {load && !series ? (
        /* No value is the absence of the value: the card says why,
           and shows no number at all. */
        <p className="text-[13px] text-ink-muted" data-testid="progress-load-empty">
          {t(LOAD_EMPTY[load.unavailable ?? ''] ?? 'progress.load.empty.other')}
        </p>
      ) : null}
      {load ? (
        <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-load-window">
          {t('progress.load.window', {
            from: load.window.from_label,
            to: load.window.to_label,
          })}
        </p>
      ) : null}
    </section>
  )
}

/**
 * The window against the window before it.
 *
 * A comparison, not a verdict: the two numbers side by side and the
 * difference between them. The card names the window it describes, so a
 * number is never read apart from the period it covers.
 */
export function RollingPanel({
  rolling,
  error,
}: {
  rolling: RollingInsight | undefined
  error: boolean
}) {
  return (
    <section className="card px-4 py-3" data-testid="progress-rolling">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.heading')}
      </h2>
      {error && !rolling ? (
        <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
      ) : null}
      {!rolling ? <p className="text-[13px] text-ink-muted">{t('common.loading')}</p> : null}
      {rolling ? (
        <>
          <p className="text-[12px] text-ink-muted" data-testid="progress-rolling-window">
            {t('progress.rolling.window', { days: rolling.window.days })} ·{' '}
            {t('progress.rolling.range', {
              from: rolling.window.from_label,
              to: rolling.window.to_label,
            })}
          </p>
          <dl className="mt-2 space-y-2" data-testid="progress-rolling-values">
            {[
              {
                key: 'distance',
                label: t('vital.distance'),
                current: renderedText(rolling.distance.current),
                previous: renderedText(rolling.distance.previous),
                difference: renderedText(rolling.distance.difference),
              },
              {
                key: 'duration',
                label: t('vital.time'),
                current: renderedText(rolling.duration.current),
                previous: renderedText(rolling.duration.previous),
                difference: renderedText(rolling.duration.difference),
              },
              {
                key: 'sessions',
                label: t('progress.rolling.sessions'),
                current: formatCount(rolling.sessions.current),
                previous: formatCount(rolling.sessions.previous),
                difference: renderedText(rolling.sessions.difference),
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
                  {t('progress.rolling.before', { days: rolling.window.days })}: {row.previous}
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
  )
}

/**
 * How the training was distributed across the athlete's own zones:
 * low (Z1–Z2), moderate (Z3), high (Z4–Z5). The shares are facts about
 * the window, and each says which way it moved against the window before
 * it.
 *
 * There is no target range here: a range that says how the split ought
 * to be is a prescription, and this product does not prescribe. The model
 * the split was measured against is named with the day it was read.
 */
export function ZonesPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-zones">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.zones.heading')}
      </h2>
      {rolling?.zones.shares ? (
        <>
          <ul className="space-y-1.5" data-testid="progress-zones-shares">
            {rolling.zones.shares.map((share) => (
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
                        days: rolling.window.days,
                      })}
                    </span>
                  ) : null}
                </span>
              </li>
            ))}
          </ul>
          {rolling.zones.model?.source ? (
            <p className="mt-2 text-[11.5px] text-ink-faint" data-testid="progress-zones-model">
              {rolling.zones.model.source}
            </p>
          ) : null}
          <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-zones-window">
            {t('progress.rolling.window', { days: rolling.window.days })}
          </p>
        </>
      ) : null}
      {rolling && !rolling.zones.shares ? (
        /* No value is the absence of the value: the card says why,
           and shows no number at all. */
        <p className="text-[13px] text-ink-muted" data-testid="progress-zones-empty">
          {rolling.zones.unavailable ?? t('common.error')}
        </p>
      ) : null}
    </section>
  )
}

/**
 * The distribution of the time across the athlete's own zones: how long
 * they spent in each of the five, and what share of the time in the
 * zones that was.
 *
 * The ranges are the model's own floors, and the model is named below — a
 * band that cannot say where it came from is not inspectable.
 */
export function ZonesDistributionPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-zones-distribution">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.zones.distribution')}
      </h2>
      {rolling?.zones.distribution ? (
        <>
          <ul className="space-y-1.5" data-testid="progress-zones-distribution-rows">
            {rolling.zones.distribution.map((zone) => (
              <li key={zone.zone}>
                <div className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">
                    {zone.zone}
                    {zone.range ? (
                      <span className="ml-1 text-[11.5px] text-ink-faint">{zone.range}</span>
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
          {rolling.zones.model?.source ? (
            <p className="mt-2 text-[11.5px] text-ink-faint" data-testid="progress-zones-model">
              {rolling.zones.model.source}
            </p>
          ) : null}
        </>
      ) : null}
      {rolling && !rolling.zones.distribution ? (
        <p
          className="text-[13px] text-ink-muted"
          data-testid="progress-zones-distribution-empty"
        >
          {rolling.zones.unavailable ?? t('common.error')}
        </p>
      ) : null}
    </section>
  )
}

/**
 * The runs by how far they went: the buckets, their runs, their distance,
 * their climb and their time. The bucket size is taken from the longest
 * run in the window, so the buckets are the athlete's own scale.
 */
export function DistancesPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-distances">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.distances.heading')}
      </h2>
      <p className="text-[12px] text-ink-faint" data-testid="progress-distances-window">
        {t('progress.rolling.window', { days: rolling?.window.days ?? 0 })}
      </p>
      <ul className="mt-1.5 space-y-1">
        {rolling?.distances.map((bucket) => (
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
  )
}

/**
 * Seven facets of the athlete's shape over the window: the volume, the
 * consistency, the intensity, the duration, the density, the sport variety
 * and the days of the week.
 *
 * **Not a radar.** A polygon over fixed anchors is a claim about a
 * shape, and the anchors are somebody else's coaching assumptions. These
 * are values, and each says which way it moved against the window before
 * it. The reader draws the shape.
 */
export function ProfilePanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-profile">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.profile.heading')}
      </h2>
      <ul className="space-y-1.5" data-testid="progress-profile-facets">
        {rolling?.profile.map((facet) => (
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
                    days: rolling?.window.days ?? 0,
                  })}
                </span>
              ) : null}
            </span>
          </li>
        ))}
      </ul>
      <p className="mt-2 text-[12px] text-ink-faint" data-testid="progress-profile-window">
        {t('progress.rolling.window', { days: rolling?.window.days ?? 0 })}
      </p>
    </section>
  )
}

/** What usually happened on each day of the week. */
export function WeekdayPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-weekday">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.weekday.heading')}
      </h2>
      <p className="text-[12px] text-ink-faint">{t('progress.rolling.history')}</p>
      <ul className="mt-1.5 space-y-1">
        {rolling?.weekday.map((day) => (
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
  )
}

/** What usually happened in each time of day. */
export function DaytimePanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-daytime">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.daytime.heading')}
      </h2>
      <p className="text-[12px] text-ink-faint">{t('progress.rolling.history')}</p>
      <ul className="mt-1.5 space-y-1.5">
        {rolling?.daytime.map((bucket) => (
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
  )
}

/** What each month of the history holds. */
export function MonthsPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-months">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.months.heading')}
      </h2>
      {rolling && rolling.months.length === 0 ? (
        <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
      ) : null}
      <ul className="space-y-1">
        {rolling?.months.map((month) => (
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
  )
}

/** What each year of the history holds. */
export function YearsPanel({ rolling }: { rolling: RollingInsight | undefined }) {
  return (
    <section className="card px-4 py-3" data-testid="progress-years">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {t('progress.rolling.years.heading')}
      </h2>
      <p className="text-[12px] text-ink-faint">{t('progress.rolling.history')}</p>
      {rolling && rolling.years.length === 0 ? (
        <p className="text-[13px] text-ink-muted">{t('progress.rolling.noRows')}</p>
      ) : null}
      <ul className="space-y-1">
        {rolling?.years.map((year) => (
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
  )
}

/** The weekly volume, as a chart. */
export function VolumePanel({
  volume,
  ticks,
  typicalWeek,
  onWeekClick,
}: {
  volume: VolumePoint[]
  ticks: { value: number; text: string }[]
  typicalWeek: RenderedValue | null
  /** What happens when a week is opened: the runs of that week. */
  onWeekClick?: (weekStart: string) => void
}) {
  return (
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
                <rect width="6" height="6" fill="var(--chart-band)" />
                <rect width="3" height="6" fill="var(--chart-line)" />
              </pattern>
            </defs>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
              interval="preserveStartEnd"
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks)}
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
            {typicalWeek !== null ? (
              <ReferenceLine
                y={(typicalWeek.raw ?? 0) / 1000}
                stroke="var(--chart-axis)"
                strokeDasharray="4 3"
              />
            ) : null}
            <Bar
              name={t('vital.distance')}
              dataKey="km"
              fill="var(--chart-line)"
              isAnimationActive={false}
              cursor="pointer"
              onClick={(_entry, index) => {
                const week = volume[index]?.weekStart
                if (week) onWeekClick?.(week)
              }}
            >
              {volume.map((week, index) => (
                <Cell
                  key={week.label}
                  fill={index === volume.length - 1 ? 'url(#partial-week)' : 'var(--chart-line)'}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
      <p className="mt-2 text-[12px] text-ink-muted">{t('progress.partial')}</p>
      {/* The band, named below the chart rather than inside it: a
          label on the line itself is cut off on a narrow screen. */}
      {typicalWeek !== null ? (
        <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-typical-week">
          {t('progress.typicalWeek')} · {renderedText(typicalWeek)}
        </p>
      ) : null}
    </ChartCard>
  )
}

/** The weekly median pace at a heart rate, as a chart. */
export function PaceAtHrPanel({
  pace,
  ticks,
  window,
}: {
  pace: PacePoint[]
  ticks: { value: number; text: string }[]
  /** The window the chart covers, named under it. */
  window?: string
}) {
  return (
    <ChartCard title={t('progress.paceAtHr.heading')}>
      <div style={{ width: '100%', height: 180 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={pace} margin={{ top: 4, right: 4, bottom: 0, left: -6 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
              interval="preserveStartEnd"
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks)}
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
              stroke="var(--chart-line)"
              strokeWidth={2}
              dot={false}
              connectNulls={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      <p className="mt-2 text-[12px] text-ink-muted">{t('progress.paceNote')}</p>
      {window ? (
        <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-pace-window">
          {window}
        </p>
      ) : null}
    </ChartCard>
  )
}

/** The bests and the milestones. */
export function BestsPanel({ bests }: { bests: ProgressInsight['bests'] }) {
  return (
    <ChartCard title={t('progress.bests.heading')}>
      <ul className="space-y-1.5">
        {bests.map((best) => (
          <li key={best.key} className="flex items-baseline justify-between gap-3">
            <span className="text-[13.5px] text-ink-soft">{t(best.key)}</span>
            <span className="text-[13.5px] font-semibold tabular-nums text-ink">
              {best.value} · {formatDateShort(best.date)}
            </span>
          </li>
        ))}
      </ul>
    </ChartCard>
  )
}

/** The sessions a week, as a chart. */
export function ConsistencyPanel({ consistency }: { consistency: ConsistencyPoint[] }) {
  return (
    <ChartCard title={t('progress.consistency.heading')}>
      <div style={{ width: '100%', height: 140 }}>
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={consistency} margin={{ top: 4, right: 4, bottom: 0, left: -24 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
              interval="preserveStartEnd"
            />
            <YAxis
              tick={{ fontSize: 11, fill: 'var(--chart-axis)' }}
              width={64}
              allowDecimals={false}
              label={{
                value: t('progress.consistency.unit'),
                angle: -90,
                position: 'insideLeft',
                style: { fontSize: 11, fill: 'var(--chart-axis)' },
              }}
            />
            <Tooltip />
            <Bar name={t('progress.consistency.unit')} dataKey="sessions" fill="var(--brand-300)" />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </ChartCard>
  )
}

/** The plan's sessions, and how many of them were run. */
export function AdherencePanel({ adherence }: { adherence: ProgressInsight['adherence'] }) {
  if (!adherence) return null
  return (
    <ChartCard title={t('progress.adherence.heading')}>
      <p className="text-[17px] font-semibold text-ink" data-testid="progress-adherence">
        {t('progress.adherence.count', {
          done: adherence.done,
          planned: adherence.planned,
        })}
      </p>
    </ChartCard>
  )
}
