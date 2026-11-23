/**
 * Progress.
 *
 * The trajectory, forgivingly. A week with nothing in it is a gap in the
 * line, not a failure. There is no percentage without a real baseline, no
 * grade, and no counter for a week that had no plan.
 *
 * **Two arrangements of one screen.** On the wide screens it is the
 * **wall**: every panel the router answers, on one screen, in the mock's
 * cards — a name, the answer, and the way into the panel that answers it.
 * On a phone that wall is a pile of numbers without context, so the
 * narrow layout is **an index**: six questions, the athlete's answer to
 * each, and a doorway into the panel that answers it.
 *
 * The band is decided by the container, in CSS — the same rule every
 * other band in the app follows.
 */

import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useStaleness } from '../lib/staleness'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'
import { QueryError } from '../components/QueryError'
import { runsPath, shiftDate } from '../lib/history'
import { formatCount, formatDateShort, renderedText } from '../lib/format'
import { ProgressIndex } from './ProgressIndex'
import { RangeSelector } from './progressRange'
import { ProgressWall } from './progressWall'
import { useProgress } from './useProgress'
import { useWall } from './useWall'


export function ProgressPage() {
  const [range, setRange] = useState<number>(12)
  const [chosenWeek, setChosenWeek] = useState('')
  const navigate = useNavigate()
  const answers = useProgress(range)
  const wall = useWall(range)
  const { progress, data } = answers
  const stale = useStaleness()

  if (progress.isError && !data) {
    return (
      <div>
        <PageHeader title={t('progress.heading')} />
        <section className="card py-4" role="alert">
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
        <div className="card py-8 text-center" data-testid="progress-empty">
          <p className="text-[15px] font-semibold text-ink">{t('progress.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('progress.empty.body')}</p>
        </div>
      </div>
    )
  }

  const weeks = answers.weeks
  const selectedWeek = weeks.includes(chosenWeek) ? chosenWeek : weeks[weeks.length - 1]!

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
      {/*
        The core's sentence, and the window it covers. It is the screen's
        lead at every width: the narrow layout says the same thing in its
        rows, so the lead does not change with the band.
      */}
      <section className="card py-3" data-testid="progress-headline">
        {/* The headline is the core's sentence (core-contract §8). */}
        <p className="text-[19px] font-semibold leading-tight text-ink">{data.headline}</p>
        <p className="mt-1 text-[12px] text-ink-muted">
          {formatDateShort(data.period.from)} - {formatDateShort(data.period.to)}
        </p>
        <div className="progress-summary">
          <dl
            className="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-4"
            data-testid="progress-summary"
          >
            {[
              { label: t('vital.distance'), value: renderedText(data.period.distance) },
              { label: t('vital.time'), value: renderedText(data.period.duration) },
              {
                label: t('progress.summary.sessions'),
                value: formatCount(data.period.sessions),
              },
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
        </div>
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
      {/*
        The window's control, above both arrangements: the range is the
        screen's, and every answer under it follows it.
      */}
      <RangeSelector range={range} onChange={setRange} />

      {/* --------------------------------------------------- the wall */}
      {/*
        The wide arrangement: the wall of panels, in the mock's cards.
        The narrow layout replaces this with the index below — one
        arrangement or the other, never both, and the band that decides
        is the container's.
      */}
      <div className="progress-dashboard" data-testid="progress-dashboard">
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

        <ProgressWall
          answers={answers}
          wall={wall}
          onWeekClick={(weekStart) => {
            navigate(runsPath(weekStart, shiftDate(weekStart, 6)))
          }}
        />


        <p className="mt-4 text-[12.5px] text-ink-faint">
          {formatCount(data.weekly_volume.length)} weeks ·{' '}
          {t('progress.sessionsPerWeek', {
            n: data.sessions_per_week?.text ?? formatCount(null),
          })}
        </p>
        <p className="mt-1 text-[12px] text-ink-muted">{t('progress.averageNote')}</p>
      </div>

      {/* ------------------------------------------------------- the index */}
      {/*
        The narrow arrangement: six questions, the athlete's answer to
        each, and the doorway into the panel that answers it.
      */}
      <div className="progress-index" data-testid="progress-index-layout">
        <ProgressIndex answers={answers} />
      </div>
    </div>
  )
}
