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
import type {
  ChallengeInsight,
  LoadInsight,
  ProgressInsight,
  ProfileWindow,
  RenderedValue,
  RollingInsight,
} from '../lib/types'
import { ChallengeGrid } from '../components/ChallengeGrid'
import { PROFILE_SCOPES } from './useWall'
import { ABSENT, formatCount, formatDateShort, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { AthleteShape, shapeWindows } from '../components/AthleteShape'
import { SHAPE_AXES } from '../lib/shape'
import { seriesColour } from '../lib/chart'
import { LoadDaysChart, LoadProjectionChart, LoadSeriesChart } from './loadCharts'

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
export function hour(value: number): string {
  return `${String(value).padStart(2, '0')}:00`
}

/**
 * A panel's own frame.
 *
 * **A panel is a card with a name, or it is nothing.** The wide Progress
 * draws the panels inside its own cards — the mock's shape, with the
 * mock's name — so a panel can be drawn without its frame: `bare` gives
 * the content alone, and the wall puts it in a card of its own.
 *
 * The test id is on the content either way, so a panel is found the same
 * whether it is drawn on its own or inside the wall.
 */
function PanelShell({
  title,
  testId,
  bare = false,
  children,
}: {
  title: string
  testId: string
  bare?: boolean
  children: ReactNode
}) {
  if (bare) return <div data-testid={testId}>{children}</div>
  return (
    <section className="card px-4 py-3" data-testid={testId}>
      <h2 className="mb-2 caps">
        {title}
      </h2>
      {children}
    </section>
  )
}

/** A chart card with a fixed, legible height. */
export function ChartCard({
  title,
  testId,
  bare = false,
  children,
}: {
  title: string
  testId?: string
  bare?: boolean
  children: ReactNode
}) {
  if (bare) return <div data-testid={testId}>{children}</div>
  return (
    <section className="card px-4 py-3" data-testid={testId}>
      <h2 className="mb-2 caps">

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
export function EddingtonPanel({
  data,
  bare = false,
}: {
  data: ProgressInsight
  bare?: boolean
}) {
  return (
    <PanelShell title={t('progress.eddington.heading')} testId="progress-eddington" bare={bare}>

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
    </PanelShell>
  )
}

/**
 * The challenges: the athlete's own, measured week by week.
 *
 * **A week is met or it is not.** The grid says what happened and the
 * streak counts the weeks in a row; neither is a grade, and a week that
 * did not reach the target is drawn quietly rather than crossed out.
 */
export function ChallengePanel({
  challenges,
  bare = false,
}: {
  challenges: ChallengeInsight | undefined
  bare?: boolean
}) {
  const rows = challenges?.challenges ?? []
  const weeks =
    rows.length > 0
      ? rows[0]!.weeks.map((week) => ({
          week_start: week.week_start,
          label: week.label,
        }))
      : []

  return (
    <PanelShell title={t('challenge.heading')} testId="progress-challenges" bare={bare}>
      {challenges?.unavailable ? (
        <p className="text-[13px] text-ink-muted" data-testid="progress-challenges-empty">
          {challenges.unavailable}
        </p>
      ) : null}
      {!challenges?.unavailable ? (
        <>
          <ChallengeGrid challenges={rows} weeks={weeks} />
          <p className="mt-2 text-[12px] text-ink-faint">{t('challenge.note')}</p>
        </>
      ) : null}
    </PanelShell>
  )
}

/**
 * The load, as values and a scenario.
 *
 * **The shape comes first.** The card opens with the chart of the two
 * weighted series and the form, because that is what the window looks
 * like, and the numbers come after it: a picture you can read at a
 * glance, and then the facts behind it. The scenario and the assumption
 * stay at the foot, where they have always been — the assumption
 * travels with the numbers it belongs to.
 *
 * The card says what the numbers are, what window they describe and what
 * the model assumed — the coefficient, where the maximum heart rate came
 * from and the day the zone model was read, and the two windows. There is
 * no score, no grade, no verdict and no instruction: the band of the
 * ratio is named as a range, and the reader decides what it means.
 *
 * A load with no value says so, with the reason: never a zero.
 */
export function LoadPanel({
  load,
  error,
  bare = false,
  detail = true,
}: {
  load: LoadInsight | undefined
  error: boolean
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
  /**
   * Whether the card carries the whole answer.
   *
   * The **card** is the headline: the shape of the load and the numbers
   * behind it. The **panel** is the answer: the same shape, and then the
   * texture under it and the scenario's own projection. The wall is a
   * grid of cards, and a card that carried every chart of its question
   * would be a wall of one.
   */
  detail?: boolean
}) {
  const series = load?.load ?? null
  const states = series?.states ?? []
  const lastState = states.length > 0 ? states[states.length - 1]! : undefined
  const week = series?.week ?? null

  return (
    <PanelShell title={t('progress.load.heading')} testId="progress-load" bare={bare}>

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
          {/* The shape, first: the two weighted series and the form,
              day by day. */}
          {load ? (
            <div className="mt-2">
              <LoadSeriesChart load={load} />
            </div>
          ) : null}
          <dl
            className="mt-3 grid grid-cols-2 gap-x-3 gap-y-2"
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
              <h3 className="mt-3 caps-sm">
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
          {/* The charts of the load. The answer already carried the
              day-by-day series and the seven days the scenario projects
              forward; these draw them, and they draw nothing the answer
              did not send. The card carries the shape; the panel carries
              the whole answer. */}
          {load && detail ? (
            <>
              <div className="mt-4">
                <LoadDaysChart load={load} />
              </div>
              {load.scenario ? (
                <div className="mt-4">
                  <LoadProjectionChart load={load} />
                </div>
              ) : null}
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
    </PanelShell>
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
  bare = false,
}: {
  rolling: RollingInsight | undefined
  error: boolean
  bare?: boolean
}) {
  return (
    <PanelShell title={t('progress.rolling.heading')} testId="progress-rolling" bare={bare}>

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
    </PanelShell>
  )
}

/**
 * How the training was distributed across the athlete's own zones:
 * low (Z1–Z2), moderate (Z3), high (Z4–Z5). The shares are facts about
 * the window, and each says which way it moved against the window before
 * it.
 *
 * **The split is a picture.** Three bars, one under each share: the
 * reader sees the shape of the split before the numbers — a third of
 * the time easy and a third of it hard reads as two thirds, and it does
 * not read that way in three separate figures.
 *
 * There is no target range here: a range that says how the split ought
 * to be is a prescription, and this product does not prescribe. The model
 * the split was measured against is named with the day it was read.
 */
export function ZonesPanel({
  rolling,
  bare = false,
}: {
  rolling: RollingInsight | undefined
  bare?: boolean
}) {
  return (
    <PanelShell title={t('progress.rolling.zones.heading')} testId="progress-zones" bare={bare}>

      {rolling?.zones.shares ? (
        <>
          <ul className="space-y-2.5" data-testid="progress-zones-shares">
            {rolling.zones.shares.map((share) => (
              <li key={share.key}>
                <div className="flex items-baseline justify-between gap-3">
                  <span className="whitespace-nowrap text-[13.5px] text-ink-soft">
                    {share.label}
                  </span>
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
                </div>
                {/* The bar is the share itself: the core's own number,
                    drawn at its own scale. */}
                <div
                  className="mt-1 h-1.5 w-full overflow-hidden rounded-full"
                  style={{ background: 'var(--chart-band)' }}
                >
                  <div
                    className="h-full rounded-full"
                    style={{
                      width: `${share.share?.raw ?? 0}%`,
                      background: 'var(--chart-line)',
                    }}
                    data-testid={`progress-zones-bar-${share.key}`}
                  />
                </div>
              </li>
            ))}
          </ul>
          {rolling.zones.source ? (
            <p className="mt-2 text-[11.5px] text-ink-faint" data-testid="progress-zones-model">
              {rolling.zones.source}
            </p>
          ) : null}
          {rolling.zones.models.length > 1 ? (
            /* The window was measured against more than one model: the
               hoisted line names them, and the detail is a click away. */
            <details className="mt-1 text-[11.5px] text-ink-faint" data-testid="progress-zones-models">
              <summary>{t('progress.rolling.zones.models')}</summary>
              <ul className="mt-1 space-y-0.5">
                {rolling.zones.models.map((model) => (
                  <li key={model.read_on}>{model.source}</li>
                ))}
              </ul>
            </details>
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
    </PanelShell>
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
export function ZonesDistributionPanel({
  rolling,
  bare = false,
}: {
  rolling: RollingInsight | undefined
  bare?: boolean
}) {
  return (
    <PanelShell
      title={t('progress.rolling.zones.distribution')}
      testId="progress-zones-distribution"
      bare={bare}
    >

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
          {rolling.zones.source ? (
            <p className="mt-2 text-[11.5px] text-ink-faint" data-testid="progress-zones-model">
              {rolling.zones.source}
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
    </PanelShell>

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
 *
 * **These are the window's own totals**: the distance of the window, the
 * days it carries a run — the number a reader asks for when they ask
 * "how far did I run in three months?". The **rates** the chart is drawn
 * from are on the shape, and the two never stand for one another.
 */
export function ProfilePanel({
  rolling,
  bare = false,
}: {
  rolling: RollingInsight | undefined
  bare?: boolean
}) {
  return (
    <PanelShell title={t('progress.profile.heading')} testId="progress-profile" bare={bare}>

      <ul className="space-y-1.5" data-testid="progress-profile-facets">
        {rolling?.profile.map((facet) => (
          <li key={facet.key} className="flex items-baseline justify-between gap-3">
            <span className="text-[13.5px] text-ink-soft">{facet.label}</span>
            <span className="flex items-baseline gap-2">
              <span
                className="text-[16px] font-semibold tabular-nums text-ink"
                data-testid={`progress-profile-${facet.key}`}
              >
                {renderedText(facet.total)}
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
      <p className="mt-0.5 text-[12px] text-ink-faint">{t('progress.profile.facets.note')}</p>
    </PanelShell>

  )
}

/** The weekly volume, as a chart. */
export function VolumePanel({
  volume,
  ticks,
  typicalWeek,
  onWeekClick,
  bare = false,
}: {
  volume: VolumePoint[]
  ticks: { value: number; text: string }[]
  typicalWeek: RenderedValue | null
  /** What happens when a week is opened: the runs of that week. */
  onWeekClick?: (weekStart: string) => void
  bare?: boolean
}) {
  return (
    <ChartCard title={t('progress.volume.heading')} testId="progress-volume" bare={bare}>
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
                  fill={
                    index === volume.length - 1 ? 'url(#partial-week)' : 'var(--chart-line)'
                  }
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
  bare = false,
}: {
  pace: PacePoint[]
  ticks: { value: number; text: string }[]
  /** The window the chart covers, named under it. */
  window?: string
  bare?: boolean
}) {
  return (
    <ChartCard title={t('progress.paceAtHr.heading')} testId="progress-pace-hr" bare={bare}>
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
export function BestsPanel({
  bests,
  bare = false,
}: {
  bests: ProgressInsight['bests']
  bare?: boolean
}) {
  return (
    <ChartCard title={t('progress.bests.heading')} testId="progress-bests" bare={bare}>
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
export function ConsistencyPanel({
  consistency,
  bare = false,
}: {
  consistency: ConsistencyPoint[]
  bare?: boolean
}) {
  return (
    <ChartCard
      title={t('progress.consistency.heading')}
      testId="progress-consistency"
      bare={bare}
    >
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
            <Bar
              name={t('progress.consistency.unit')}
              dataKey="sessions"
              fill="var(--brand-300)"
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </ChartCard>
  )
}

/** The plan's sessions, and how many of them were run. */
export function AdherencePanel({
  adherence,
  bare = false,
}: {
  adherence: ProgressInsight['adherence']
  bare?: boolean
}) {
  if (!adherence) return null
  return (
    <ChartCard title={t('progress.adherence.heading')} testId="progress-adherence" bare={bare}>
      <p className="text-[17px] font-semibold text-ink" data-testid="progress-adherence">
        {t('progress.adherence.count', {
          done: adherence.done,
          planned: adherence.planned,
        })}
      </p>
    </ChartCard>
  )
}

/**
 * The athlete's shape: six axes, three windows, one chart.
 *
 * The same six facets of the training — the volume, the consistency, the
 * sport variety, the density, the duration, the intensity — over three
 * windows, drawn on one chart. **Each axis is scaled to the largest of
 * the three windows' values**, so the three shapes are read against each
 * other and not against anybody's idea of a good week.
 *
 * **The axes are rates, and the core is where that happens.** The volume
 * is a distance a week and the consistency the share of the window's
 * days, so the windows can be read against each other at all. The
 * numbers under the chart are the core's own rendering of the same
 * numbers the chart draws, one row for each axis, and a facet with no
 * value says so by showing nothing.
 *
 * The table under the chart is the **focused window against the window
 * before it of the same length**: the equal-length comparison, and which
 * way each facet moved.
 */
export function ShapePanel({
  windows,
  error,
  days,
  onScope,
  bare = false,
}: {
  /** The three windows the profile is drawn over. */
  windows: ProfileWindow[]
  error: boolean
  /** The window the page is about, in days. */
  days: number
  /** The scope the athlete chooses. */
  onScope: (days: number) => void
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
}) {
  if (bare) {
    return (
      <div data-testid="progress-shape">
        <ProfileScopePicker days={days} onChange={onScope} />
        <ShapeBody windows={windows} error={error} days={days} />
      </div>
    )
  }

  return (
    <section className="card px-4 py-3" data-testid="progress-shape">
      <h2 className="mb-2 caps">{t('progress.shape.heading')}</h2>
      {/* The note is drawn by the body below, and the body is inside the
          panel: drawing it here as well drew it twice. */}
      <ProfileScopePicker days={days} onChange={onScope} />
      <ShapeBody windows={windows} error={error} days={days} />
    </section>
  )
}

/**
 * The windows the profile is drawn over.
 *
 * **Thirty days, sixty days, ninety days.** The chart draws all three of
 * them, one on the other; the picker says which of them the rest of the
 * page is about.
 */
export function ProfileScopePicker({
  days,
  onChange,
}: {
  days: number
  onChange: (days: number) => void
}) {
  return (
    <div className="mb-2 flex flex-wrap gap-1.5" data-testid="profile-scope">
      {PROFILE_SCOPES.map((scope) => (
        <button
          key={scope.days}
          type="button"
          className={`tap rounded-full border px-2.5 py-0.5 text-[11.5px] ${
            scope.days === days
              ? 'border-brand-600 bg-brand-600 font-semibold text-white'
              : 'border-edge text-ink-muted'
          }`}
          onClick={() => onChange(scope.days)}
          data-testid={`profile-scope-${scope.days}`}
        >
          {t(scope.labelKey)}
        </button>
      ))}
    </div>
  )
}

/**
 * One row of the table under the chart: an axis, the focused window's
 * value, the window before it, and which way it moved.
 *
 * The values are the **comparable** ones — the rate, or the average of
 * the runs — so that they are the numbers the chart draws.
 */
export interface ShapeRow {
  key: string
  label: string
  now: RenderedValue | null
  before: RenderedValue | null
  movement: string | null
}

/** The rows of the table under the chart, for the focused window. */
export function shapeRows(rolling: RollingInsight | undefined): ShapeRow[] {
  return SHAPE_AXES.map((key) => {
    const facet = rolling?.profile.find((entry) => entry.key === key)
    return {
      key,
      label: facet?.label ?? '',
      now: facet?.value ?? null,
      before: facet?.previous ?? null,
      movement: facet?.movement ?? null,
    }
  })
}

/**
 * The short words a movement is written in, for the table's own column.
 *
 * The word is the movement and nothing else: a share that grew and a
 * facet that fell are both "up" and "down". Which way is the better way
 * is not decided here.
 */
export const MOVEMENT: Record<string, CatalogueKey> = {
  up: 'progress.rolling.trend.up.short',
  down: 'progress.rolling.trend.down.short',
  steady: 'progress.rolling.trend.steady.short',
}

/** The shape's body: the chart, the legend and the numbers. */
export function ShapeBody({
  windows,
  error,
  days,
  table = true,
}: {
  /** The three windows the chart draws, one on the other. */
  windows: ProfileWindow[]
  error: boolean
  /** The window the page is about, in days. */
  days: number
  /**
   * Whether the numbers under the chart are drawn.
   *
   * The chart is the answer; the numbers are the detail under it. A card
   * that leads with the picture says the numbers on the panel it opens,
   * and keeps the card a picture.
   */
  table?: boolean
}) {
  const focused = windows.find((window) => window.days === days)?.rolling
  const rows = shapeRows(focused)
  const shapes = shapeWindows(windows)
  return (
    <>
      <p className="text-[12px] text-ink-muted">{t('progress.shape.note')}</p>
      {error && !focused ? (
        <p className="mt-2 text-[13px] text-ink-muted" data-testid="progress-shape-error">
          {t('common.error')}
        </p>
      ) : null}
      {!focused && !error ? (
        <p className="mt-2 text-[13px] text-ink-muted">{t('common.loading')}</p>
      ) : null}
      {focused ? (
        <>
          {/* The three windows, one on the other: the whole shape on one
              chart. */}
          <AthleteShape windows={shapes} focus={days} />
          {/* The windows, named: the chart is never read apart from the
              periods it covers. */}
          <ul className="mt-2 flex flex-wrap gap-x-4 gap-y-1" data-testid="progress-shape-legend">
            {windows.map((window, index) => (
              <li
                key={window.days}
                className="flex items-center gap-1.5 text-[11.5px] text-ink-muted"
              >
                <span
                  className="inline-block h-2.5 w-4 rounded-sm"
                  style={{ backgroundColor: seriesColour(index) }}
                />
                <span>
                  {window.rolling
                    ? t('progress.shape.legend', { days: window.days })
                    : t('progress.shape.empty')}
                </span>
              </li>
            ))}
          </ul>
          <p className="mt-1 text-[12px] text-ink-faint">{t('progress.shape.scale')}</p>
          <p className="mt-0.5 text-[12px] text-ink-faint">{t('progress.shape.rates')}</p>
          {/* The numbers, in the core's own words: a row for each axis,
              the focused window, the window before it of the same
              length, and which way it moved. */}
          {table ? (
            <table
              className="mt-3 w-full table-fixed text-[12px]"
              data-testid="progress-shape-values"
            >
              <thead>
                <tr className="border-b border-edge text-ink-muted">
                  <th scope="col" className="py-1.5 text-left font-medium">
                    {t('progress.shape.axis')}
                  </th>
                  <th scope="col" className="py-1.5 text-right font-medium">
                    {t('progress.shape.window', { days })}
                  </th>
                  <th scope="col" className="py-1.5 text-right font-medium">
                    {t('progress.shape.previous', { days })}
                  </th>
                  <th scope="col" className="py-1.5 text-right font-medium">
                    {t('progress.shape.movement')}
                  </th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => (
                  <tr key={row.key} className="border-b border-edge last:border-b-0">
                    <th scope="row" className="py-1.5 text-left font-medium text-ink-soft">
                      {row.label}
                    </th>
                    <td
                      className="py-1.5 text-right tabular-nums text-ink"
                      data-testid={`progress-shape-${row.key}-now`}
                    >
                      {renderedText(row.now)}
                    </td>
                    <td
                      className="py-1.5 text-right tabular-nums text-ink-muted"
                      data-testid={`progress-shape-${row.key}-before`}
                    >
                      {renderedText(row.before)}
                    </td>
                    <td
                      className="py-1.5 text-right tabular-nums text-ink-muted"
                      data-testid={`progress-shape-${row.key}-movement`}
                    >
                      {row.movement
                        ? t(MOVEMENT[row.movement] ?? 'progress.rolling.trend.steady.short')
                        : ABSENT}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : null}
        </>
      ) : null}
    </>
  )
}
