/**
 * One panel of Progress, on its own.
 *
 * The narrow Progress is an index: six questions, six answers, and a
 * doorway into the panel that answers each one. This is that doorway —
 * the **existing panel component**, drawn once, at whatever width the
 * phone is, with the window it covers named on it.
 *
 * It is a route and not a destination: the four destinations stay the
 * four, and this is reached from Progress and from nowhere else.
 */

import { useState } from 'react'
import { Link, Navigate, useParams } from 'react-router-dom'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'
import { StreakCard } from '../components/StreakCard'
import { useStaleness } from '../lib/staleness'
import { PROGRESS_PANELS, type ProgressPanel } from './ProgressIndex'
import { RangeSelector } from './progressRange'
import {
  EddingtonPanel,
  LoadPanel,
  PaceAtHrPanel,
  RollingPanel,
  YearsPanel,
  ZonesPanel,
} from './progressPanels'
import { useProgress } from './useProgress'

function isPanel(name: string | undefined): name is ProgressPanel {
  return PROGRESS_PANELS.includes(name as ProgressPanel)
}

export function ProgressPanelPage() {
  const { panel } = useParams()
  const [range, setRange] = useState(12)
  const answers = useProgress(range)
  const stale = useStaleness()

  // A name that is not a panel is not a page: back to the index.
  if (!isPanel(panel)) return <Navigate to="/progress" replace />

  const { progress, load, rolling, streaks } = answers

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
        {t('progress.index.back')}
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
        {panel === 'window' ? (
          <RollingPanel rolling={rolling.data} error={rolling.isError} />
        ) : null}
        {panel === 'load' ? <LoadPanel load={load.data} error={load.isError} /> : null}
        {panel === 'year' ? <YearsPanel rolling={rolling.data} /> : null}
        {panel === 'pace' ? (
          <PaceAtHrPanel
            pace={answers.pace}
            ticks={answers.paceTicks}
            window={t('progress.index.pace.weeks', { n: range })}
          />
        ) : null}
        {panel === 'zones' ? <ZonesPanel rolling={rolling.data} /> : null}
        {panel === 'habit' ? (
          <>
            {streaks.data ? (
              <StreakCard
                streaks={streaks.data}
                className="card py-3"
                testId="progress-streaks"
              />
            ) : null}
            {progress.data ? <EddingtonPanel data={progress.data} /> : null}
          </>
        ) : null}
      </div>
    </div>
  )
}
