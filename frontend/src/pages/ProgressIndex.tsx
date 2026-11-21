/**
 * The narrow Progress: **an index of the questions**.
 *
 * On the wide screens Progress is a dashboard: a grid of panels, each one
 * a question with its answer. On a phone that grid is twelve tiles of
 * numbers without context — the app telling the athlete how much data it
 * holds. So the narrow layout is an **index**: seven rows, each one a
 * question with the athlete's answer in it, and a doorway to the panel
 * that answers it.
 *
 * The rules the rows follow:
 *
 *   - **Every number is the answer to that question**, in the app's own
 *     wording and the core's own rendering — "58.4 km", "+40.2 km",
 *     "5:24 /km at 148 bpm".
 *   - **Every row names the window it covers.** A number without its
 *     period is not a fact.
 *   - **Nothing on a row is a verdict.** A row states what the answer is;
 *     the reader decides what it means.
 *   - **A row with no answer says why**, in the core's own words, and
 *     shows no number at all. Absence is not zero.
 *
 * The panels the rows open are the app's existing panels, drawn one at a
 * time by the panel route.
 */

import { Link } from 'react-router-dom'
import { ChevronRight } from 'lucide-react'
import { ABSENT, formatCount, formatDateMedium, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import type { ProgressAnswers } from './useProgress'
import { LOAD_EMPTY } from './progressPanels'

/**
 * A count with its unit — in the number's own singular or plural.
 *
 * One day is one day and not "1 days", and the unit is a word of the
 * app's, so it lives in the catalogue like every other one.
 */
function counted(count: number, one: CatalogueKey, many: CatalogueKey): string {
  return t(count === 1 ? one : many, { n: formatCount(count) })
}

/** The panels a row can open. */
export const PROGRESS_PANELS = ['window', 'load', 'year', 'pace', 'zones', 'habit', 'shape'] as const
export type ProgressPanel = (typeof PROGRESS_PANELS)[number]

/** One row of the index. */
interface IndexRow {
  /** What the row is called, for the test and the key. */
  id: string
  /** The panel the row opens. */
  panel: ProgressPanel
  /** The question. */
  question: string
  /** The athlete's answer to it. */
  answer: string
  /** The window the answer covers. */
  window: string | null
  /**
   * Whether the row has no value at all.
   *
   * A row with no value says why — in the core's own words — and shows
   * no number: absence is not zero, and it is not a value either. It is
   * drawn as what it is, and not as an answer.
   */
  empty: boolean
}

/**
 * The rows, and the answers in them.
 *
 * The answers are the app's existing panels' answers, read from the same
 * answers the panels draw: nothing here is computed a second time, and
 * nothing here is a number the app formatted on its own.
 */
export function indexRows(answers: ProgressAnswers): IndexRow[] {
  const { progress, load, rolling, streaks, shape } = answers

  /* ---------------------------------------------- the window, and before */
  const rollingData = rolling.data
  const windowRow: IndexRow = {
    id: 'window',
    panel: 'window',
    question: t('progress.index.window.question'),
    answer: rollingData
      ? `${renderedText(rollingData.distance.current)} · ${renderedText(rollingData.distance.difference)}`
      : rolling.isError
        ? t('common.error')
        : t('common.loading'),
    window: rollingData
      ? `${t('progress.rolling.window', { days: rollingData.window.days })} · ${t(
          'progress.rolling.range',
          { from: rollingData.window.from_label, to: rollingData.window.to_label },
        )}`
      : null,
    empty: !rollingData,
  }

  /* ------------------------------------------------------ the training load */
  const loadData = load.data
  const loadState = loadData?.load?.states?.[loadData.load.states.length - 1]
  const loadRow: IndexRow = {
    id: 'load',
    panel: 'load',
    question: t('progress.index.load.question'),
    answer: loadData
      ? loadData.load && loadState
        ? `${t('progress.load.acute')} ${renderedText(loadState.acute)} · ${t(
            'progress.load.form',
          )} ${renderedText(loadState.form)}`
        : t(LOAD_EMPTY[loadData.unavailable ?? ''] ?? 'progress.load.empty.other')
      : load.isError
        ? t('common.error')
        : t('common.loading'),
    window: loadState ? t('progress.load.asOf', { date: loadState.label }) : null,
    empty: !loadData || !loadData.load,
  }

  /* ------------------------------------------------------------- the year */
  const rollingYears = rollingData?.years ?? []
  const year = rollingYears[0]
  const months = rollingData?.months.length ?? 0
  const yearRow: IndexRow = {
    id: 'year',
    panel: 'year',
    question: t('progress.index.year.question'),
    answer: year
      ? `${renderedText(year.distance)} · ${counted(year.sessions, 'progress.index.run', 'progress.index.runs')}`
      : rolling.isError
        ? t('common.error')
        : rollingData
          ? t('progress.rolling.noRows')
          : t('common.loading'),
    window: year
      ? `${year.label} · ${counted(months, 'progress.index.month', 'progress.index.months')}`
      : null,
    empty: !year,
  }

  /* -------------------------------------------------------------- the pace */
  const paceWeeks = progress.data?.pace_at_hr ?? []
  const paceWeek = paceWeeks[paceWeeks.length - 1]
  const paceRow: IndexRow = {
    id: 'pace',
    panel: 'pace',
    question: t('progress.index.pace.question'),
    answer: paceWeek
      ? `${renderedText(paceWeek.pace)} at ${renderedText(paceWeek.heart_rate)}`
      : progress.isError
        ? t('common.error')
        : progress.data
          ? t('progress.rolling.noRows')
          : t('common.loading'),
    window: paceWeek
      ? t('progress.index.pace.week', { date: formatDateMedium(paceWeek.week_start) })
      : null,
    empty: !paceWeek,
  }

  /* -------------------------------------------------------------- the zones */
  const shares = rollingData?.zones.shares ?? []
  const zonesRow: IndexRow = {
    id: 'zones',
    panel: 'zones',
    question: t('progress.index.zones.question'),
    answer: shares.length
      ? shares.map((share) => `${share.label} ${renderedText(share.share)}`).join(' · ')
      : rollingData
        ? (rollingData.zones.unavailable ?? t('common.error'))
        : rolling.isError
          ? t('common.error')
          : t('common.loading'),
    window: rollingData
      ? t('progress.rolling.window', { days: rollingData.window.days })
      : null,
    empty: shares.length === 0,
  }

  /* -------------------------------------------------------------- the habit */
  const streakDays = streaks.data?.days.current ?? null
  const streakWeeks = streaks.data?.weeks.current ?? null
  const eddington = progress.data?.eddington ?? null
  const habitRow: IndexRow = {
    id: 'habit',
    panel: 'habit',
    question: t('progress.index.habit.question'),
    answer: [
      streakDays
        ? counted(streakDays.count, 'progress.index.day', 'progress.index.days')
        : t('today.streak.none'),
      streakWeeks
        ? counted(streakWeeks.count, 'progress.index.week', 'progress.index.weeks')
        : t('today.streak.none'),
      eddington ? t('progress.index.eddington', { n: formatCount(eddington.number) }) : ABSENT,
    ].join(' · '),
    window: progress.data
      ? t('progress.index.habit.window', {
          from: formatDateMedium(progress.data.period.from),
          to: formatDateMedium(progress.data.period.to),
        })
      : null,
    empty: !streakDays && !streakWeeks && !eddington,
  }

  /* -------------------------------------------------------------- the shape */
  /**
   * The shape: the athlete's own facets over the shorter of the two
   * windows. The answer is the core's own rendering of the volume and of
   * the consistency — the days of the window that carry a run — and the
   * window is named beside it.
   */
  const shapeData = shape.short.data
  const volume = shapeData?.profile.find((facet) => facet.key === 'volume')?.value ?? null
  const consistency =
    shapeData?.profile.find((facet) => facet.key === 'consistency')?.value ?? null
  const shapeRow: IndexRow = {
    id: 'shape',
    panel: 'shape',
    question: t('progress.index.shape.question'),
    answer: shapeData
      ? `${renderedText(volume)} · ${
          consistency?.raw === null || consistency?.raw === undefined
            ? ABSENT
            : counted(consistency.raw, 'progress.index.day', 'progress.index.days')
        }`
      : shape.short.isError
        ? t('common.error')
        : t('common.loading'),
    window: shapeData
      ? t('progress.rolling.window', { days: shapeData.window.days })
      : null,
    empty: !shapeData,
  }

  return [windowRow, loadRow, yearRow, paceRow, zonesRow, habitRow, shapeRow]
}

/**
 * The index itself.
 *
 * A list of rows in one card, the way the app's own lists are drawn: the
 * question on its own line, the answer under it, and the window the
 * answer covers under that. The row opens the panel that answers it.
 */
export function ProgressIndex({ answers }: { answers: ProgressAnswers }) {
  const rows = indexRows(answers)

  return (
    <div data-testid="progress-index">
      <p className="lede">{t('progress.index.intro')}</p>
      <ul className="card overflow-hidden card-flush" data-testid="progress-index-rows">
        {rows.map((row) => (
          <li key={row.id} className="border-b border-edge last:border-b-0">
            <Link
              to={`/progress/${row.panel}`}
              data-testid={`progress-index-${row.id}`}
              className="tap flex items-start justify-between gap-3 px-4 py-2.5"
            >
              <span className="min-w-0 flex-1">
                <span className="block text-[14.5px] font-semibold leading-tight text-ink">
                  {row.question}
                </span>
                <span
                  className={`mt-0.5 block text-[14px] leading-tight ${
                    row.empty ? 'text-ink-muted' : 'font-semibold tabular-nums text-ink-soft'
                  }`}
                  data-testid={`progress-index-${row.id}-answer`}
                >
                  {row.answer}
                </span>
                {row.window ? (
                  <span
                    className="mt-0.5 block text-[11.5px] leading-tight text-ink-muted"
                    data-testid={`progress-index-${row.id}-window`}
                  >
                    {row.window}
                  </span>
                ) : null}
              </span>
              <ChevronRight size={18} className="mt-0.5 shrink-0 text-ink-faint" />
            </Link>
          </li>
        ))}
      </ul>
      <p className="mt-3 text-[12px] leading-snug text-ink-faint">
        {t('progress.index.footnote')}
      </p>
    </div>
  )
}
