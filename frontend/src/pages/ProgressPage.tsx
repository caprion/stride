/**
 * Progress.
 *
 * The trajectory, forgivingly. A week with nothing in it is a gap in the
 * line, not a failure. There is no percentage without a real baseline, no
 * grade, and no counter for a week that had no plan.
 *
 * **Two arrangements of one screen.** On the wide screens it is the
 * dashboard: the panels, laid out in a grid, each one a question with its
 * answer. On a phone that grid is twelve tiles of numbers without
 * context, so the narrow layout is **an index**: six questions, the
 * athlete's answer to each, and a doorway into the panel that answers it.
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
import { useProgress } from './useProgress'
import {
  AdherencePanel,
  BestsPanel,
  ConsistencyPanel,
  DaytimePanel,
  DistancesPanel,
  EddingtonPanel,
  LoadPanel,
  MonthsPanel,
  PaceAtHrPanel,
  ProfilePanel,
  RollingPanel,
  ShapePanel,
  VolumePanel,
  WeekdayPanel,
  YearsPanel,
  ZonesDistributionPanel,
  ZonesPanel,
} from './progressPanels'

export function ProgressPage() {
  const [range, setRange] = useState<number>(12)
  const [chosenWeek, setChosenWeek] = useState('')
  const navigate = useNavigate()
  const answers = useProgress(range)
  const { progress, load, rolling, data } = answers
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

      {/* ------------------------------------------------- the dashboard */}
      {/*
        The wide arrangement: the panels, in a grid. The narrow layout
        replaces this with the index below — one arrangement or the
        other, never both, and the band that decides is the container's.
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
          {/* The pace page is reached from here: it is a page about the
              history, not a destination of its own. */}
          <Link
            className="tap-inline text-[13px] font-medium text-brand-700 underline"
            to="/pace"
            data-testid="progress-pace-link"
          >
            {t('pace.open')}
          </Link>
          {/* The year, as a film: the twelve months, the calendar of the
              days and what changed. It is reached from here. */}
          <Link
            className="tap-inline text-[13px] font-medium text-brand-700 underline"
            to="/year"
            data-testid="progress-year-link"
          >
            {t('year.open')}
          </Link>
        </div>
        <div className="progress-grid mt-3">
          {/* ------------------------------------------ the Eddington number */}
          {/*
            The number, the days still needed for the next one and the
            longest distance in a day — over the window the screen is
            showing. The window is named on the card, so the number is
            never read apart from the period it covers.
          */}
          <EddingtonPanel data={data} />

          {/* ------------------------------------------ the training load */}
          {/*
            The load, as values and a scenario.

            The card says what the numbers are, what window they describe
            and what the model assumed — the coefficient, where the maximum
            heart rate came from and the day the zone model was read, and
            the two windows. There is no score, no grade, no verdict and no
            instruction: the band of the ratio is named as a range, and the
            reader decides what it means.

            A load with no value says so, with the reason: never a zero.
          */}
          <LoadPanel load={load.data} error={load.isError} />

          {/* --------------------------------------- the rolling window */}
          {/*
            The window against the window before it.

            A comparison, not a verdict: the two numbers side by side and
            the difference between them. The card names the window it
            describes, so a number is never read apart from the period it
            covers.
          */}
          <RollingPanel rolling={rolling.data} error={rolling.isError} />

          {/* ------------------------------------------ the split of zones */}
          {/*
            How the training was distributed across the athlete's own
            zones: low (Z1–Z2), moderate (Z3), high (Z4–Z5). The shares
            are facts about the window, and each says which way it moved
            against the window before it.

            There is no target range here: a range that says how the split
            ought to be is a prescription, and this product does not
            prescribe. The model the split was measured against is named
            with the day it was read.
          */}
          <ZonesPanel rolling={rolling.data} />

          {/* --------------------------------------- the zone distribution */}
          {/*
            The distribution of the time across the athlete's own zones:
            how long they spent in each of the five, and what share of the
            time in the zones that was.

            The ranges are the model's own floors, and the model is named
            below — a band that cannot say where it came from is not
            inspectable.
          */}
          <ZonesDistributionPanel rolling={rolling.data} />

          {/* --------------------------------------- the distance breakdown */}
          {/*
            The runs by how far they went: the buckets, their runs, their
            distance, their climb and their time. The bucket size is taken
            from the longest run in the window, so the buckets are the
            athlete's own scale.
          */}
          <DistancesPanel rolling={rolling.data} />

          {/* -------------------------------------------- the training profile */}
          {/*
            Seven facets of the athlete's shape over the window: the
            volume, the consistency, the intensity, the duration, the
            density, the sport variety and the days of the week.

            **Not a radar.** A polygon over fixed anchors is a claim about a
            shape, and the anchors are somebody else's coaching
            assumptions. These are values, and each says which way it moved
            against the window before it. The reader draws the shape.
          */}
          <ProfilePanel rolling={rolling.data} />

          {/* ----------------------------------------------- the athlete's shape */}
          {/*
            The same six facets of the training, over two windows, drawn
            on one chart: the shape of the short window and the shape of
            the long one, each axis scaled to the larger of the two.

            It is a panel of the wall, and it opens on its own like the
            others — the phone reaches it through the index.
          */}
          <ShapePanel
            short={answers.shape.short.data}
            long={answers.shape.long.data}
            error={answers.shape.short.isError || answers.shape.long.isError}
          />

          {/* -------------------------------------------- the weekday split */}
          <WeekdayPanel rolling={rolling.data} />

          {/* --------------------------------------------- the daytime split */}
          <DaytimePanel rolling={rolling.data} />

          {/* ------------------------------------------------- the months */}
          <MonthsPanel rolling={rolling.data} />

          {/* -------------------------------------------------- the years */}
          <YearsPanel rolling={rolling.data} />

          {/* --------------------------------------------- weekly volume */}
          <VolumePanel
            volume={answers.volume}
            ticks={answers.volumeTicks}
            typicalWeek={data.typical_week}
            onWeekClick={(weekStart) => {
              navigate(runsPath(weekStart, shiftDate(weekStart, 6)))
            }}
          />

          {/* -------------------------------------------- pace at heart rate */}
          <PaceAtHrPanel
            pace={answers.pace}
            ticks={answers.paceTicks}
            window={t('progress.index.pace.weeks', { n: range })}
          />

          {/* ------------------------------------------ bests & milestones */}
          <BestsPanel bests={data.bests} />

          {/* ---------------------------------------------- consistency */}
          <ConsistencyPanel consistency={answers.consistency} />

          {/* ------------------------------------------------- adherence */}
          <AdherencePanel adherence={data.adherence} />
        </div>

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
