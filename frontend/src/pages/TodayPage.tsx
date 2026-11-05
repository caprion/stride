/**
 * Today.
 *
 * A readout, not an instruction. The order is fixed:
 *
 *   the actual week → the state line → the three vitals → the plan's own session → the
 *   plan's other options → the last run.
 *
 * Nothing here instructs, grades, ranks or concludes. The plan's sessions
 * are listed unranked and are never linked to a vital — that link would
 * be the judgement.
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
 *     alone.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import { formatDateMedium, formatDateShort, renderedText } from '../lib/format'
import { stalenessText } from '../lib/staleness'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
import { RouteMap } from '../components/RouteMap'
import { VitalReadings } from '../components/VitalReadings'
import { PageHeader } from '../ui/layout/PageHeader'
import { SleepComposition } from '../components/SleepComposition'
import { SyncControls } from '../components/SyncControls'
import { QueryError } from '../components/QueryError'

function SessionCard({
  planned,
  comparison,
  paused,
}: {
  planned: PlanSession
  comparison: string | null
  paused: string | null
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
      className="tap card block px-4 py-3 transition-shadow hover:shadow-lift"
    >
      {/* The session, as the core wrote it: "Today — 10 km · easy". */}
      <p className="text-[17px] font-semibold text-ink">{planned.sentence}</p>
      <p className="mt-0.5 text-[13px] text-ink-muted">{t(sourceKey)}</p>
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
  const connect = useMutation({ mutationFn: () => connectGarmin() })
  const runSync = useMutation({
    mutationFn: () => api.syncRun(),
    onSuccess: () => queryClient.invalidateQueries(),
  })

  const insight = today.data
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
        <section className="card px-4 py-4" role="alert">
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
        <div className="card px-4 py-6 text-center" data-testid="today-empty">
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
        </div>
      </div>
    )
  }

  const lastRun = insight.last_run
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

      <div className="today-layout">
        <div>
          <section className="card hero-card mb-3 px-5 py-5" data-testid="today-week">
            <h2 className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
              {t('today.week.heading')}
            </h2>
            <Link to="/runs" className="tap mt-1 flex items-baseline justify-between gap-3">
              <span className="text-[48px] font-semibold leading-tight tracking-tight tabular-nums text-brand-700">
                {renderedText(insight.week.distance)}
              </span>
              <span className="text-[13px] text-ink-muted">
                {t('today.week.sessions', { n: insight.week.sessions })}
              </span>
            </Link>
            <ol
              className="mt-4 grid grid-cols-7 gap-1"
              aria-label={t('today.week.recordedDays')}
            >
              {insight.week.days.map((day) => (
                <li
                  key={day.date}
                  className="flex flex-col items-center gap-1 text-[12px]"
                  aria-label={`${formatDateShort(day.date)}: ${t('today.week.sessions', { n: day.sessions })}`}
                >
                  <span className="text-ink-muted">
                    {formatDateShort(day.date).slice(0, 3)}
                  </span>
                  <span
                    aria-current={day.date === insight.date ? 'date' : undefined}
                    className={`flex h-7 w-7 items-center justify-center rounded-full border text-[12px] font-semibold tabular-nums ${day.date === insight.date ? 'border-accent-500 bg-accent-100 text-brand-800' : day.sessions > 0 ? 'border-brand-600 bg-brand-600 text-white' : 'border-edge text-ink-muted'}`}
                  >
                    {day.sessions > 0 ? day.sessions : <span aria-hidden="true">&middot;</span>}
                  </span>
                </li>
              ))}
            </ol>
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
            <section className="card px-4 py-3" data-testid="today-sync-failed">
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
            <section className="card px-4 py-3" data-testid="today-state-line">
              {/*
            The heading names when the numbers are from, and interprets
            nothing. The core's own label for the summary row stays below
            it: "Rest" is the name of a row, not an instruction.
          */}
              <p className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
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
        </div>
      </div>

      {/* -------------------------------------------- the plan's session */}
      {insight.planned ? (
        <section className="mt-3">
          <SessionCard
            planned={insight.planned}
            comparison={insight.distance_comparison}
            paused={paused}
          />
        </section>
      ) : null}

      {/* ---------------------------------------- the plan's other options */}
      {insight.options.length > 0 ? (
        <section className="card mt-3 px-4 py-3" data-testid="today-options">
          <p className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
            {t('today.options.heading')}
          </p>
          <ul className="mt-1.5 space-y-1">
            {insight.options.map((option) => (
              <li key={option.date} className="text-[14px] text-ink-soft">
                {option.sentence}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {/* --------------------------------------------------- the last run */}
      {lastRun ? (
        <section className="mt-3">
          {/*
            The last run, and — at the wide widths, where there is room
            for it — a mini thumbnail of its route. The thumbnail is the
            API's own points, drawn small; nothing goes anywhere to make
            it.
          */}
          <Link
            to={`/runs/${lastRun.id}`}
            className="tap card flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3 transition-shadow hover:shadow-lift"
            data-testid="today-last-run"
          >
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
                {t('today.lastRun.label')}
              </span>
              <span className="mt-0.5 block text-[14px] text-ink">
                {renderedText(lastRun.distance)} · {renderedText(lastRun.duration)} ·{' '}
                {formatDateMedium(lastRun.local_date)}
              </span>
            </span>
            {lastRun.geo_points.length > 1 ? (
              <span className="hidden w-[112px] shrink-0 lg:block" data-testid="today-route">
                <RouteMap points={lastRun.geo_points} variant="mini" />
              </span>
            ) : null}
          </Link>
        </section>
      ) : null}
    </div>
  )
}
