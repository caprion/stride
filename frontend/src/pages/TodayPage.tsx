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
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { distanceComparison } from '../lib/direction'
import { formatDateMedium, formatDistance, formatDurationHuman } from '../lib/format'
import type { Direction, TodayInsight, VitalsSnapshot } from '../lib/types'
import { t } from '../i18n'
import { DirectionText } from '../components/DirectionText'
import { Sparkline } from '../components/Sparkline'
import { VitalRow } from '../components/VitalRow'
import { PageHeader } from '../ui/layout/PageHeader'

/** The number of signals below their usual, and the direction that follows. */
function restSummary(snapshot: VitalsSnapshot): {
  below: number
  total: number
  direction: Direction
} {
  const below = snapshot.vitals.filter((vital) => vital.direction === 'could be better').length
  const total = snapshot.vitals.length
  const direction: Direction = below === 0 ? 'better than usual' : 'could be better'
  return { below, total, direction }
}

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
  planned: NonNullable<TodayInsight['planned']>
  comparison: ReturnType<typeof distanceComparison>
}) {
  const distance = planned.target_distance_m
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
      <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
        <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
          {t('today.session.label')}
        </span>
        <span className="text-[17px] font-semibold text-ink">
          {distance === null ? planned.session_type : formatDistance(distance)}
        </span>
        <span className="text-[13px] text-ink-soft">· {planned.session_type}</span>
        <span className="text-[13px] text-ink-muted">· {t(sourceKey)}</span>
      </div>
      {/*
        The distance comparison appears only when the gap from the target
        is beyond 10%, and it sits inside the session card as a second
        line — never as a banner, never coloured as a warning.
      */}
      {comparison ? (
        <p className="mt-1 text-[13px] text-ink-muted" data-testid="today-distance-comparison">
          {formatDistance(comparison.actual_m)} —{' '}
          {t(comparison.over ? 'today.distance.more' : 'today.distance.less', {
            gap: formatDistance(comparison.gap_m),
            target: formatDistance(comparison.target_m),
          })}
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

  const summary = restSummary(snapshot)
  const lastRun = insight.last_run
  const comparison = distanceComparison(lastRun?.distance_m ?? null, insight.target_distance_m)
  const stale = stalenessText(sync.data?.last_sync_at)

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
            {t('today.rest.label')}
          </span>
          <DirectionText direction={summary.direction} />
          <span className="text-ink-faint" aria-hidden="true">
            ·
          </span>
          <span className="text-[13px] text-ink-muted">
            {t('today.rest.summary', { below: summary.below, total: summary.total })}
          </span>
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
                band={
                  vital.baseline
                    ? { lower: vital.baseline.lower, upper: vital.baseline.upper }
                    : null
                }
                label={t('body.history.label', { n: 30 })}
              />
            </div>
          </VitalRow>
        ))}
      </section>

      {/* -------------------------------------------- the plan's session */}
      {insight.planned ? (
        <section className="mt-3">
          <SessionCard planned={insight.planned} comparison={comparison} />
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
                {option.target_distance_m === null
                  ? option.session_type
                  : formatDistance(option.target_distance_m)}{' '}
                · {option.session_type} · {formatDateMedium(option.date)}
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
