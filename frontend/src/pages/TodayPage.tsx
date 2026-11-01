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
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { formatDateMedium, formatDistance, formatDurationHuman } from '../lib/format'
import type { PlanSession } from '../lib/types'
import { t } from '../i18n'
import { Sparkline } from '../components/Sparkline'
import { VitalRow } from '../components/VitalRow'
import { PageHeader } from '../ui/layout/PageHeader'

/** How long ago the last sync was, in words — only when it matters. */
function stalenessText(lastSyncAt: string | null | undefined): string | null {
  if (!lastSyncAt) return null
  const hours = (Date.now() - new Date(lastSyncAt).getTime()) / 3_600_000
  if (!Number.isFinite(hours) || hours < 36) return null
  const days = Math.round(hours / 24)
  if (days >= 1) return t('today.state.stale.days', { n: days })
  return t('today.state.stale.hours', { n: Math.round(hours) })
}

function SessionCard({
  planned,
  comparison,
}: {
  planned: PlanSession
  comparison: string | null
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
    </Link>
  )
}

export function TodayPage() {
  const today = useQuery({ queryKey: ['insights-today'], queryFn: api.insightsToday })
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: api.vitals })
  const sync = useQuery({ queryKey: ['sync-status'], queryFn: api.syncStatus })

  const insight = today.data
  const snapshot = vitals.data

  if (today.isPending || vitals.isPending) {
    return (
      <div>
        <PageHeader title={t('today.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  if (!insight || !snapshot) {
    return (
      <div>
        <PageHeader title={t('today.heading')} />
        <div className="card px-4 py-6 text-center">
          <p className="text-[14px] text-ink-soft">{t('today.state.noData')}</p>
          <button
            type="button"
            className="tap mt-3 inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white"
          >
            {t('today.state.noDataAction')}
          </button>
        </div>
      </div>
    )
  }

  const lastRun = insight.last_run
  const stale = stalenessText(sync.data?.last_sync_at)
  const nights = snapshot.history.sleep.length

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
      </section>

      {/* --------------------------------------------------- the vitals */}
      <section className="card mt-3 px-4 py-1" data-testid="today-vitals">
        {snapshot.vitals.map((vital) => (
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

      {/* -------------------------------------------- the plan's session */}
      {insight.planned ? (
        <section className="mt-3">
          <SessionCard
            planned={insight.planned}
            comparison={insight.distance_comparison}
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
              {formatDistance(lastRun.distance_m)} ·{' '}
              {formatDurationHuman(lastRun.moving_time_s)} ·{' '}
              {formatDateMedium(lastRun.local_date)}
            </span>
          </Link>
        </section>
      ) : null}
    </div>
  )
}
