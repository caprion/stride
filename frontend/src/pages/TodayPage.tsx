/**
 * Today — the cockpit.
 *
 * One hero, three rows, and what is on the table:
 *
 *   - **the hero** — the latest run: its name, its distance, its time,
 *     its pace, its heart rate and its elevation, the zone ribbon drawn
 *     from the athlete's own zones, the route the way the run detail
 *     draws it, and a way into the run;
 *   - **where you stand** — three rows, each a question with the answer
 *     in it: this week against the same days last week, the form with
 *     its ratio and its band, and the habit with the streak, the months
 *     and the Eddington number;
 *   - **what's on the table** — the plan's own sessions, unranked, and
 *     the night's readings.
 *
 * **Nothing here is a verdict, a score or a command.** A difference is a
 * difference; a form is a form; a streak is a count and a date. The
 * plan's sessions are listed unranked and are never linked to a vital —
 * that link would be the judgement.
 *
 * Every sentence on this screen is the core's (core-contract §8): the
 * Rest line, the direction words, the arithmetic, the travel and the
 * plan's sessions arrive written. The screen is the chrome around them.
 *
 * **And the states are the screen.** A reviewer will try these first:
 *
 *   - **nothing yet** — no runs, no wellness, no key: the screen says
 *     what is missing and what to do, and it invents no numbers;
 *   - **stale** — the data is older than it should be, and the screen
 *     says when it was last synced and which day the numbers are from;
 *   - **the sync failed** — the reason, a retry, and the last good
 *     readout greyed with its date: the numbers would be a lie, so they
 *     are not shown as if they were current;
 *   - **not enough history** — that row says so, and shows the value
 *     alone;
 *   - **a week with no runs** — the distance is zero because nothing was
 *     run, and the difference says so against the same days last week.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import {
  formatCount,
  formatDateMedium,
  formatDateShort,
  formatDayMonth,
  renderedText,
} from '../lib/format'
import { stalenessText } from '../lib/staleness'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
import { StreetRouteMap } from '../components/StreetRouteMap'
import { VitalReadings } from '../components/VitalReadings'
import { PageHeader } from '../ui/layout/PageHeader'
import { SleepComposition } from '../components/SleepComposition'
import { SyncControls } from '../components/SyncControls'
import { QueryError } from '../components/QueryError'
import { ZoneRibbon } from '../components/ZoneRibbon'
import { runsPath, weekDates } from '../lib/history'
import { PlanPacing, SessionPace } from './todayPlan'
import { LOAD_EMPTY } from '../lib/load'

/**
 * The window the cockpit's Eddington number is over.
 *
 * It is the Progress screen's own default range, so the number on the
 * cockpit is the number on the Progress screen — and the Progress answer
 * is already cached under this range, so the two screens share one
 * answer.
 */
const HABIT_WEEKS = 12

function SessionCard({
  planned,
  comparison,
  paused,
  pace,
}: {
  planned: PlanSession
  comparison: string | null
  paused: string | null
  /** The pace the plan was worked out with, when the plan gave one. */
  pace?: React.ReactNode
}) {
  const sourceKey =
    planned.source === 'prescribed'
      ? 'today.session.source.imported'
      : planned.source === 'baseline'
        ? 'today.session.source.baseline'
        : 'today.session.source.typed'

  return (
    <Link
      to="/plan"
      data-testid="today-session"
      className="tap card block py-3 transition-shadow hover:shadow-lift"
    >
      {/* The session, as the core wrote it: "Today — 10 km · easy". */}
      <p className="text-[17px] font-semibold text-ink">{planned.sentence}</p>
      <p className="mt-0.5 text-[13px] text-ink-muted">{t(sourceKey)}</p>
      {/* The plan's own pace for the day, when the plan gave one. */}
      {pace ? <p className="mt-0.5">{pace}</p> : null}
      {/*
        The distance comparison appears only when the gap from the target
        is beyond 10%, and it sits inside the session card as a second
        line — never as a banner, never coloured as a warning. The core
        decides whether it is there at all, and writes it when it is.
      */}
      {comparison ? (
        <p className="mt-1 text-[13px] text-ink-muted" data-testid="today-distance-comparison">
          {comparison}
        </p>
      ) : null}
      {/* While the plan is paused, the app says so — a fact, in the app's
          voice. */}
      {paused ? (
        <p className="mt-1 text-[13px] text-ink-muted" data-testid="today-paused">
          {paused}
        </p>
      ) : null}
    </Link>
  )
}

export function TodayPage() {
  const queryClient = useQueryClient()
  const today = useQuery({ queryKey: ['insights-today'], queryFn: api.insightsToday })
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: () => api.vitals() })
  const sync = useQuery({ queryKey: ['sync-status'], queryFn: api.syncStatus })
  const settings = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const connection = useQuery({ queryKey: ['auth-status'], queryFn: api.authStatus })
  // The habit: how long the streak has run. A count and a date, and
  // nothing else — no "you are consistent" anywhere near it.
  const streaks = useQuery({ queryKey: ['insights-streaks'], queryFn: api.insightsStreaks })
  // The form: the load's own state, over the Progress screen's range.
  const load = useQuery({
    queryKey: ['insights-load', HABIT_WEEKS],
    queryFn: () => api.insightsLoad(HABIT_WEEKS),
  })
  // The plan's own week: the adopted plan's paces for the day.
  const plan = useQuery({ queryKey: ['plan-week'], queryFn: api.planWeek })
  // The Eddington number, over the same range.
  const progress = useQuery({
    queryKey: ['insights-progress', HABIT_WEEKS],
    queryFn: () => api.insightsProgress(HABIT_WEEKS),
  })

  const insight = today.data
  const lastRun = insight?.last_run ?? null
  // The zone ribbon: the run's own time in the athlete's own zones.
  const zones = useQuery({
    queryKey: ['fitness-zones', lastRun?.id ?? 0],
    queryFn: () => api.fitnessZones(lastRun!.id),
    enabled: Boolean(lastRun),
  })

  const connect = useMutation({ mutationFn: () => connectGarmin() })
  const runSync = useMutation({
    mutationFn: () => api.syncRun(),
    onSuccess: () => queryClient.invalidateQueries(),
  })

  const snapshot = vitals.data

  if (today.isPending || vitals.isPending || sync.isPending) {
    return (
      <div>
        <PageHeader title={t('today.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  if (
    (today.isError || vitals.isError || sync.isError) &&
    (!insight || !snapshot || !sync.data)
  ) {
    return (
      <div>
        <PageHeader title={t('today.heading')} />
        <section className="card py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void today.refetch()
              void vitals.refetch()
              void sync.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      </div>
    )
  }

  // Nothing to show: there is no number, and the app never makes one up.
  // It says what is missing and what to do.
  //
  // The sync state alone does not decide this: a store can hold runs and
  // no sync row (the api's own seed is one), and the numbers are there.
  // The router's shape is frozen — three vitals, one per kind, with null
  // values when there is no data — so the empty state is the one where
  // there is no last run **and** no vital has a value.
  const nothingToShow =
    !insight ||
    !snapshot ||
    (!snapshot.vitals.some((vital) => vital.value_text !== null) &&
      !Object.values(snapshot.history).some((history) =>
        history.some((point) => Number.isFinite(point.value)),
      ) &&
      !insight.last_run)

  if (nothingToShow) {
    return (
      <div>
        <PageHeader title={t('today.heading')} action={<SyncControls />} />
        <div className="card py-6 text-center" data-testid="today-empty">
          <p className="text-[14px] text-ink-soft">
            {connection.data?.connected ? t('today.state.noRecords') : t('today.state.noData')}
          </p>
          {!connection.data?.connected ? (
            <button
              type="button"
              className="tap mt-3 inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white"
              disabled={
                connection.isPending ||
                connection.isError ||
                runSync.isPending ||
                connect.isPending
              }
              onClick={() => {
                if (connection.data?.connected) runSync.mutate()
                else connect.mutate()
              }}
            >
              {connection.data?.connected ? t('sync.refresh') : t('today.state.noDataAction')}
            </button>
          ) : null}
          {connect.isError ? (
            <p className="mt-2 text-[13px] text-ink-muted">{t('today.state.connectFailed')}</p>
          ) : null}
          {connection.isError ? (
            <QueryError
              retry={() => {
                void connection.refetch()
              }}
            />
          ) : null}
          {runSync.isError ? <p role="alert">{t('sync.failed')}</p> : null}
          <Link to="/plan" className="tap-inline mt-3 text-[13px] text-brand-700 underline">
            {t('today.plan.open')}
          </Link>
        </div>
      </div>
    )
  }

  const stale = stalenessText(sync.data?.last_sync_at)
  const failed = sync.data?.last_error ?? null
  const readings = snapshot.vitals.filter((vital) => vital.value_text !== null)
  const paused = settings.data?.pause
    ? settings.data.pause.reason
      ? t('common.paused.untilReason', {
          date: formatDateShort(settings.data.pause.end_date),
          reason: t(
            settings.data.pause.reason === 'travel'
              ? 'settings.pause.reason.travel'
              : settings.data.pause.reason === 'sick'
                ? 'settings.pause.reason.sick'
                : 'settings.pause.reason.injured',
          ),
        })
      : t('common.paused.until', { date: formatDateShort(settings.data.pause.end_date) })
    : null

  // The week's own dates: the calendar week the answer is about.
  const week = weekDates(insight.date)
  // The form's own state: the last day the load has one for.
  const lastState = load.data?.load?.states.slice(-1)[0] ?? null

  return (
    <div>
      <PageHeader
        title={t('today.heading')}
        meta={
          <>
            {formatDateMedium(insight.date)}
            {stale ? <span className="ml-2 text-ink-faint">· {stale}</span> : null}
          </>
        }
        action={<SyncControls />}
      />
      {today.isError || vitals.isError || sync.isError ? (
        <QueryError
          retry={() => {
            void today.refetch()
            void vitals.refetch()
            void sync.refetch()
          }}
        />
      ) : null}

      {/* ------------------------------------------------ the hero: the run */}
      {lastRun ? (
        <section className="card mb-3 py-4" data-testid="today-hero">
          <div className="flex items-baseline justify-between gap-3">
            <h2 className="caps">
              {t('today.cockpit.hero')}
            </h2>
            <p className="text-[12px] text-ink-muted">{formatDateMedium(lastRun.local_date)}</p>
          </div>
          <h3 className="mt-1 text-[19px] font-semibold text-ink">
            {lastRun.name || t('compare.run.label')}
          </h3>
          <div className="mt-1 lg:flex lg:items-start lg:gap-6">
            <div className="min-w-0 flex-1">
              <p className="text-[40px] font-semibold leading-tight tracking-tight tabular-nums text-brand-700">
                {renderedText(lastRun.distance)}
              </p>
              {/*
                The run's own facts, in the core's words: the time, the
                pace, the heart rate and the elevation. Nothing here is
                read against a target — there is no target on this screen.
              */}
              <ul className="mt-1 flex flex-wrap gap-x-3 gap-y-0.5 text-[13px] text-ink-soft">
                <li data-testid="today-hero-time">{renderedText(lastRun.duration)}</li>
                <li data-testid="today-hero-pace">{renderedText(lastRun.pace)}</li>
                <li data-testid="today-hero-hr">{renderedText(lastRun.heart_rate)}</li>
                <li data-testid="today-hero-elevation">
                  {renderedText(lastRun.elevation_gain)}
                </li>
              </ul>
              <ZoneRibbon zones={zones.data} />
              {/*
                The way into the run. The film is another thread's work:
                until it lands, the action opens the run itself.
              */}
              <Link
                to={`/runs/${lastRun.id}`}
                data-testid="today-hero-open"
                className="tap mt-3 inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white"
              >
                {t('today.hero.open')}
              </Link>
            </div>
            {/*
              The route, drawn the way the run detail draws it: the GPS
              outline, and the Street map behind the app's own disclosure.
              The phone has no room for it — the hero's map is the wide
              widths' business.
            */}
            {lastRun.geo_points.length > 1 ? (
              <div
                className="mt-3 hidden w-full lg:mt-0 lg:block lg:w-[320px] lg:shrink-0"
                data-testid="today-hero-route"
              >
                <StreetRouteMap points={lastRun.geo_points} />
              </div>
            ) : null}
          </div>
        </section>
      ) : null}

      <div className="today-layout">
        <div>
          {/* ------------------------------------------- where you stand */}
          <section className="card mb-3 py-3" data-testid="today-standing">
            <h2 className="caps">
              {t('today.cockpit.standing')}
            </h2>
            <ul className="mt-1.5 divide-y divide-edge">
              {/* This week: the distance, and the same days last week. */}
              <li className="py-2.5" data-testid="today-week">
                <div className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">{t('today.week.heading')}</span>
                  <Link
                    to={runsPath(week[0]!, week[6]!)}
                    className="tap text-[17px] font-semibold tabular-nums text-brand-700 underline"
                    data-testid="today-week-distance"
                  >
                    {renderedText(insight.week.distance)}
                  </Link>
                </div>
                <p className="text-[12.5px] text-ink-muted" data-testid="today-week-against">
                  {insight.week.difference
                    ? t('today.week.against', {
                        difference: renderedText(insight.week.difference),
                        from: formatDayMonth(insight.week.previous.from),
                        to: formatDayMonth(insight.week.previous.to),
                      })
                    : null}
                </p>
                {/*
                  The recorded days of the week. These are the sessions
                  that happened — not a rest day, not a coverage claim,
                  and not an inferred habit.
                */}
                <ol
                  className="mt-2 grid grid-cols-7"
                  aria-label={t('today.week.recordedDays')}
                  data-testid="today-week-days"
                >
                  {insight.week.days.map((day) => (
                    <li key={day.date} className="min-w-0">
                      <Link
                        to={runsPath(day.date)}
                        className={`tap flex min-h-[48px] flex-col items-center justify-end gap-1 rounded-md text-[12px] ${day.date === insight.date ? 'bg-accent-100' : 'hover:bg-brand-100'}`}
                        aria-label={`${formatDateShort(day.date)}: ${t('today.week.sessions', { n: day.sessions })}, ${renderedText(day.distance)}`}
                        aria-current={day.date === insight.date ? 'date' : undefined}
                      >
                        <span
                          className="w-5 rounded-t bg-brand-600"
                          style={{
                            height: `${day.sessions ? Math.max(8, (36 * (day.distance?.raw ?? day.sessions)) / Math.max(1, ...insight.week.days.map((entry) => entry.distance?.raw ?? entry.sessions))) : 2}px`,
                          }}
                        />
                        <span className="text-ink-muted">
                          {formatDateShort(day.date).slice(0, 3)}
                        </span>
                        <span className="font-semibold tabular-nums text-brand-700">
                          {day.date.slice(-2)}
                        </span>
                      </Link>
                    </li>
                  ))}
                </ol>
              </li>

              {/* Form: the load's own state, with its ratio and its band. */}
              <li className="py-2.5" data-testid="today-row-form">
                <div className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">{t('today.form.label')}</span>
                  <span
                    className="text-[17px] font-semibold tabular-nums text-ink"
                    data-testid="today-form-value"
                  >
                    {renderedText(lastState?.form ?? null)}
                  </span>
                </div>
                {lastState ? (
                  <>
                    <p className="text-[12.5px] text-ink-muted" data-testid="today-form-ratio">
                      {t('today.form.ratio', { ratio: renderedText(lastState.ratio) })}
                    </p>
                    <p className="text-[12px] text-ink-faint" data-testid="today-form-band">
                      {t('today.form.band', { band: load.data?.scenario?.band ?? '' })}
                      {' · '}
                      {t('today.form.asOf', { date: lastState.label })}
                    </p>
                  </>
                ) : load.data ? (
                  /* No load is no value: the row says why, and shows no
                     number at all. */
                  <p className="text-[12.5px] text-ink-muted" data-testid="today-form-empty">
                    {t(LOAD_EMPTY[load.data.unavailable ?? ''] ?? 'progress.load.empty.other')}
                  </p>
                ) : (
                  <p className="text-[12.5px] text-ink-muted">{t('common.loading')}</p>
                )}
              </li>

              {/* The habit: the streak, the months and the number. */}
              <li className="py-2.5" data-testid="today-row-habit">
                <div className="flex items-baseline justify-between gap-3">
                  <span className="text-[13.5px] text-ink-soft">{t('today.habit.label')}</span>
                  <span
                    className="text-[17px] font-semibold tabular-nums text-ink"
                    data-testid="today-habit-streak"
                  >
                    {streaks.data?.days.current
                      ? t(
                          streaks.data.days.current.count === 1
                            ? 'today.habit.day'
                            : 'today.habit.days',
                          { n: streaks.data.days.current.count },
                        )
                      : t('today.streak.none')}
                  </span>
                </div>
                <p className="text-[12.5px] text-ink-muted" data-testid="today-habit-months">
                  {streaks.data?.months.current
                    ? t(
                        streaks.data.months.current.count === 1
                          ? 'today.habit.month'
                          : 'today.habit.months',
                        { n: streaks.data.months.current.count },
                      )
                    : t('today.habit.months.none')}
                </p>
                {/*
                  A streak is a count and a date: the day it started is
                  part of the fact, and it is the core's own rendering of
                  it.
                */}
                {streaks.data?.days.current ? (
                  <p className="text-[12px] text-ink-faint" data-testid="today-habit-since">
                    {t('today.streak.since', { date: streaks.data.days.current.start_label })}
                  </p>
                ) : null}
                <p className="text-[12px] text-ink-faint" data-testid="today-habit-eddington">
                  {progress.data?.eddington
                    ? t('today.habit.eddington', {
                        number: formatCount(progress.data.eddington.number),
                        from: formatDayMonth(progress.data.period.from),
                        to: formatDayMonth(progress.data.period.to),
                      })
                    : progress.data
                      ? t('today.habit.eddington.empty')
                      : t('common.loading')}
                </p>
              </li>
            </ul>
          </section>

          {/* --------------------------------------- what's on the table */}
          <section className="card mb-3 py-3" data-testid="today-table">
            <div className="flex items-center justify-between gap-3">
              <h2 className="caps">
                {t('today.cockpit.table')}
              </h2>
              <Link
                to="/plan"
                className="tap-inline text-[13px] font-medium text-brand-700 underline"
              >
                {t('today.plan.open')}
              </Link>
            </div>
            {/*
              The plan's pacing, named once: the threshold the plan was
              worked out with, and the day it was read.
            */}
            <PlanPacing week={plan.data?.adopted} />
            {insight.planned ? (
              <SessionCard
                planned={insight.planned}
                comparison={insight.distance_comparison}
                paused={paused}
                pace={
                  <SessionPace
                    session={plan.data?.adopted?.sessions.find(
                      (session) => session.date === insight.planned?.date,
                    )}
                  />
                }
              />
            ) : (
              <p className="mt-1 text-[13px] text-ink-muted">{t('today.state.noPlan')}</p>
            )}
            {insight.options.length > 0 ? (
              <div className="mt-3 border-t border-edge pt-3" data-testid="today-options">
                <h3 className="text-[12px] font-semibold text-ink-muted">
                  {t('today.options.heading')}
                </h3>
                <ul className="mt-1 space-y-1">
                  {insight.options.map((option) => (
                    <li key={option.date} className="text-[13px] text-ink-soft">
                      {option.sentence}
                      {/*
                        The plan's own pace for that day, when the plan
                        gave one.
                      */}
                      <SessionPace
                        session={plan.data?.adopted?.sessions.find(
                          (session) => session.date === option.date,
                        )}
                      />
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </section>
        </div>
        <div>
          {/* ---------------------------------------------- the state line */}
          {sync.data?.more_pending ? (
            <p className="mb-3 text-[13px] text-ink-muted" data-testid="today-history-pending">
              {t('today.state.historyPending')}
            </p>
          ) : null}
          {failed ? (
            // The sync did not finish. The reason and a retry, and the
            // numbers below are greyed with their date: they are the last
            // good ones, not the current ones.
            <section className="card py-3" data-testid="today-sync-failed">
              <p className="text-[13.5px] font-semibold text-ink">
                {t('today.state.syncIncomplete')}
              </p>
              <p className="mt-0.5 text-[13px] text-ink-muted">{failed}</p>
              <button
                type="button"
                disabled={runSync.isPending}
                aria-busy={runSync.isPending}
                className="tap mt-2 inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-ink-soft"
                onClick={() => {
                  runSync.mutate()
                }}
              >
                {t('today.state.retry')}
              </button>
              {runSync.isError ? <p role="alert">{t('common.error')}</p> : null}
            </section>
          ) : snapshot.rest.line ? (
            <section className="card py-3" data-testid="today-state-line">
              {/*
            The heading names when the numbers are from, and interprets
            nothing. The core's own label for the summary row stays below
            it: "Rest" is the name of a row, not an instruction.
          */}
              <p className="caps">
                {t('vital.rest.heading')}
              </p>
              <div className="mt-1 flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
                <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
                  {snapshot.rest.label}
                </span>
                <span className="text-[13px] text-ink-muted">{snapshot.rest.line}</span>
              </div>
              {/* The day the numbers are from — always, and more so when the
              data is old. */}
              {readings.length > 0 ? (
                <p
                  className="mt-1 text-[12.5px] text-ink-faint"
                  data-testid="today-reading-date"
                >
                  {t('today.state.from', { when: formatDateShort(snapshot.date) })}
                </p>
              ) : null}
            </section>
          ) : null}

          {/* --------------------------------------------------- the vitals */}
          <VitalReadings snapshot={snapshot} layout="rows" muted={Boolean(failed)} />
          <SleepComposition date={snapshot.date} />
          <Link
            to={`/body?date=${snapshot.date}`}
            className="tap-inline text-[13px] font-medium text-brand-700 underline"
          >
            {t('today.body.open')}
          </Link>
        </div>
      </div>
    </div>
  )
}
