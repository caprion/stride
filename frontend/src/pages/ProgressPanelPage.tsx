/**
 * One panel of Progress, on its own.
 *
 * The narrow Progress is an index: six questions, six answers, and a
 * doorway into the panel that answers each one. The wide Progress is the
 * wall: every panel on one screen, and each card's "open →" a doorway
 * into the panel that answers it. This is that doorway — the **existing
 * panel component**, drawn once, at whatever width the phone is, with the
 * window it covers named on it.
 *
 * It is a route and not a destination: the four destinations stay the
 * four, and this is reached from Progress and from nowhere else.
 */

import { useState } from 'react'
import type { ReactNode } from 'react'
import { Link, Navigate, useParams } from 'react-router-dom'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'
import { useStaleness } from '../lib/staleness'
import { PROGRESS_PANELS, type ProgressPanel } from './ProgressIndex'
import { RangeSelector } from './progressRange'
import {
  BestsPanel,
  ConsistencyPanel,
  LoadPanel,
  PaceAtHrPanel,
  RollingPanel,
  VolumePanel,
  ZonesDistributionPanel,
  ZonesPanel,
} from './progressPanels'
import { WeekBody } from './progressWall'
import { PlanAgainstCard } from './planWeek'
import { LadderCard } from './ladder'
import { CurveCard } from './curve'
import { LedgerCard } from './ledger'
import { useProgress } from './useProgress'
import { useWall } from './useWall'

function isPanel(name: string | undefined): name is ProgressPanel {
  return PROGRESS_PANELS.includes(name as ProgressPanel)
}

/**
 * A panel that is not one of the app's own cards: the wall's own bodies,
 * in a card of their own.
 *
 * The panel route is the doorway into the panel that answers a card of
 * the wall, and it is the same answer in the same card.
 */
function PanelCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="card px-4 py-3">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {title}
      </h2>
      {children}
    </section>
  )
}

export function ProgressPanelPage() {
  const { panel } = useParams()
  const [range, setRange] = useState(12)
  const answers = useProgress(range)
  const wall = useWall(range)
  const stale = useStaleness()

  // A name that is not a panel is not a page: back to the index.
  if (!isPanel(panel)) return <Navigate to="/progress" replace />

  const { progress, load, rolling } = answers

  if (progress.isError && !progress.data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <section className="card py-4" role="alert">
          <p>{t('common.error')}</p>
        </section>
      </div>
    )
  }

  if (!progress.data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  // Nothing recorded: the panel says so, and invents no numbers.
  if (progress.data.weekly_volume.length === 0) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <div className="card py-8 text-center" data-testid="progress-empty">
          <p className="text-[15px] font-semibold text-ink">{t('progress.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('progress.empty.body')}</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <Link
        to="/progress"
        className="tap-inline mb-2 text-[13px] text-brand-700 underline"
        data-testid="progress-panel-back"
      >
        {t('progress.back')}
      </Link>
      <PageHeader
        title={t('progress.heading')}
        meta={
          <>
            {t('progress.question')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />
      <RangeSelector range={range} onChange={setRange} testId="progress-panel-range" />

      {/*
        The panel is one card, and it is capped: a single panel does not
        need the whole of a wide screen.
      */}
      <div className="mt-3 max-w-[680px] space-y-3" data-testid={`progress-panel-${panel}`}>
        {panel === 'week' ? (
          <>
            <PanelCard title={t('progress.wall.thisWeek.heading')}>
              <WeekBody today={answers.today.data} />
              {/*
                **The week against the plan**, in the same panel: the
                plan's own source, what it was paced with, the week's
                totals against the plan's, and every planned session with
                what happened on its day.
              */}
              <PlanAgainstCard week={answers.planWeek.data} bare />
            </PanelCard>
            <RollingPanel rolling={rolling.data} error={rolling.isError} />
          </>
        ) : null}
        {panel === 'load' ? <LoadPanel load={load.data} error={load.isError} /> : null}
        {panel === 'pace' ? (
          <>
            <LadderCard paces={wall.paces.data} error={wall.paces.isError} hoist />
            <PaceAtHrPanel
              pace={answers.pace}
              ticks={answers.paceTicks}
              window={t('progress.index.pace.weeks', { n: range })}
            />
          </>
        ) : null}
        {panel === 'curve' ? (
          <CurveCard curve={wall.curve.data} error={wall.curve.isError} hoist />
        ) : null}
        {panel === 'zone-time' ? (
          <>
            <LedgerCard stimulus={answers.stimulus.data} error={answers.stimulus.isError} hoist />
            <ZonesDistributionPanel rolling={rolling.data} />
            <ZonesPanel rolling={rolling.data} />
          </>
        ) : null}
        {panel === 'volume' ? (
          <>
            <VolumePanel
              volume={answers.volume}
              ticks={answers.volumeTicks}
              typicalWeek={progress.data.typical_week}
            />
            <ConsistencyPanel consistency={answers.consistency} />
          </>
        ) : null}
        {panel === 'bests' ? <BestsPanel bests={progress.data.bests} /> : null}
      </div>
    </div>
  )
}
