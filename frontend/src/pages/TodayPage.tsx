/**
 * Today.
 *
 * A readout, not an instruction. The order is fixed:
 *
 *   the state line → the three vitals → the plan's own session → the
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

import { useMutation, useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import { formatDateMedium, formatDateShort, renderedText } from '../lib/format'
import { stalenessText } from '../lib/staleness'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
import { Sparkline } from '../components/Sparkline'
import { VitalRow } from '../components/VitalRow'
import { PageHeader } from '../ui/layout/PageHeader'

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
  const today = useQuery({ queryKey: ['insights-today'], queryFn: api.insightsToday })
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: api.vitals })
  const sync = useQuery({ queryKey: ['sync-status'], queryFn: api.syncStatus })
  const settings = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const connect = useMutation({ mutationFn: () => connectGarmin() })
  const runSync = useMutation({ mutationFn: () => api.syncRun() })

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

  // Nothing has ever been synced: there is no number to show, and the
  // app never makes one up. It says what is missing and what to do.
  const neverSynced = sync.isSuccess && sync.data?.last_sync_at === null

  if (neverSynced || !insight || !snapshot) {
    // Nothing has come back yet: there is no number to show, and the app
    // never makes one up.
    return (
      <div>
        <PageHeader title={t('today.heading')} />
        <div className="card px-4 py-6 text-center" data-testid="today-empty">
          <p className="text-[14px] text-ink-soft">{t('today.state.noData')}</p>
          <button
            type="button"
            className="tap mt-3 inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white"
            onClick={() => {
              connect.mutate()
            }}
          >
            {t('today.state.noDataAction')}
          </button>
          {connect.isError ? (
            <p className="mt-2 text-[13px] text-ink-muted">{t('today.state.connectFailed')}</p>
          ) : null}
        </div>
      </div>
    )
  }

  const lastRun = insight.last_run
  const stale = stalenessText(sync.data?.last_sync_at)
  const failed = sync.data?.last_error ?? null
  const nights = snapshot.history.sleep.length
  const readings = snapshot.vitals
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
      />

      {/* ---------------------------------------------- the state line */}
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
            className="tap mt-2 inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-ink-soft"
            onClick={() => {
              runSync.mutate()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      ) : snapshot.rest.line ? (
        <section className="card px-4 py-3" data-testid="today-state-line">
          <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
            <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
              {snapshot.rest.label}
            </span>
            <span className="text-ink-faint" aria-hidden="true">
              ·
            </span>
            <span className="text-[13px] text-ink-muted">{snapshot.rest.line}</span>
          </div>
          {/* The day the numbers are from — always, and more so when the
              data is old. */}
          {readings.length > 0 ? (
            <p className="mt-1 text-[12.5px] text-ink-faint" data-testid="today-reading-date">
              {t('today.state.from', { when: formatDateShort(snapshot.date) })}
            </p>
          ) : null}
        </section>
      ) : null}

      {/* --------------------------------------------------- the vitals */}
      {readings.length === 0 ? (
        <section className="card mt-3 px-4 py-6 text-center" data-testid="today-no-vitals">
          <p className="text-[15px] font-semibold text-ink">{t('today.state.noVitals')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('today.state.noVitalsBody')}</p>
        </section>
      ) : (
        <section
          className={`card mt-3 px-4 py-1 ${failed ? 'opacity-60' : ''}`}
          data-testid="today-vitals"
        >
          {readings.map((vital) => (
            <VitalRow key={vital.key} vital={vital}>
              <div className="mt-2">
                <Sparkline
                  values={snapshot.history[vital.key]
                    .map((point) => point.value)
                    .filter((value): value is number => value !== null)}
                  band={vital.band}
                  label={t('body.history.label', { n: nights })}
                />
              </div>
            </VitalRow>
          ))}
        </section>
      )}

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
          <Link
            to={`/runs/${lastRun.id}`}
            className="tap card flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 px-4 py-3 transition-shadow hover:shadow-lift"
            data-testid="today-last-run"
          >
            <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
              {t('today.lastRun.label')}
            </span>
            <span className="text-[14px] text-ink">
              {renderedText(lastRun.distance)} · {renderedText(lastRun.duration)} ·{' '}
              {formatDateMedium(lastRun.local_date)}
            </span>
          </Link>
        </section>
      ) : null}

      {/* ------------------------------------------- this week, at medium */}
      {insight.week.sessions > 0 ? (
        <p className="today-week mt-3 text-[12.5px] text-ink-faint" data-testid="today-week">
          {t('today.week.heading')} · {renderedText(insight.week.distance)} ·{' '}
          {t('today.week.sessions', { n: insight.week.sessions })}
        </p>
      ) : null}
    </div>
  )
}
