/**
 * The two indexes: **the phone's Progress and the phone's athlete
 * profile**.
 *
 * On the wide screens Progress is a wall: a grid of panels, each one a
 * question with its answer. On a phone that grid is a pile of numbers
 * without context — the app telling the athlete how much data it holds.
 * So the narrow layout is an **index**: a few questions, the athlete's
 * answer to each, and a doorway into the panel that answers it.
 *
 * **There are two indexes now.** The athlete profile is a destination of
 * its own, so the phone has an index for it too: the picture of the
 * training, the questions the long pattern answers, and the areas' links
 * under them. The two indexes carry the two halves of the model — the
 * work's areas on Progress, the profile's areas on the athlete profile —
 * and between them every card of the wall is one link away.
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
 * The panels the rows open are the app's existing panels, drawn one at
 * a time by the panel route — under the destination they belong to, and
 * a panel of the athlete profile opened from Progress carries the place
 * it was opened from, so that its back link says where the reader came
 * from.
 *
 * **And the areas under the questions.** The wall is grouped into task
 * areas, and each index carries the areas of its own destination: the
 * panels the questions do not open already, one link each, under the
 * heading of the area they belong to. The areas are a way into the rest
 * of the wall and nothing more — they carry no answers and no numbers.
 */

import { Link } from 'react-router-dom'
import { ChevronRight } from 'lucide-react'
import { ABSENT, formatCount, formatDateMedium, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import type { ProfileWindow } from '../lib/types'
import type { ProgressAnswers } from './useProgress'
import { LOAD_EMPTY, ShapeBody } from './progressPanels'
import {
  WALL_CARDS,
  WALL_PANELS,
  areasOf,
  cardRoute,
  panelOpening,
  panelRoute,
  type TaskArea,
  type WallCard,
  type WallPanel,
} from './taskAreas'

/**
 * A count with its unit — in the number's own singular or plural.
 *
 * One day is one day and not "1 days", and the unit is a word of the
 * app's, so it lives in the catalogue like every other one.
 */
function counted(count: number, one: CatalogueKey, many: CatalogueKey): string {
  return t(count === 1 ? one : many, { n: formatCount(count) })
}

/**
 * The panels of the wall.
 *
 * The phone's indexes open them, and the panel route draws one at a
 * time. The list is the model's, so the panels of the two destinations
 * cannot drift apart.
 */
export const PROGRESS_PANELS = WALL_PANELS

/** The name of a panel of the wall. */
export type ProgressPanel = WallPanel

/** One row of an index. */
export interface IndexRow {
  /** What the row is called, for the test and the key. */
  id: string
  /** The panel the row opens. */
  panel: WallPanel
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
 * The rows of Progress, and the answers in them.
 *
 * The answers are the app's existing panels' answers, read from the same
 * answers the panels draw: nothing here is computed a second time, and
 * nothing here is a number the app formatted on its own.
 */
export function indexRows(answers: ProgressAnswers): IndexRow[] {
  const { progress, load, rolling, streaks } = answers

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

  return [windowRow, loadRow, yearRow, paceRow, zonesRow, habitRow]
}

/**
 * The questions the athlete profile asks.
 *
 * **Four questions, one for each of the areas the long pattern is
 * made of**: the goals the athlete set, the schedule the runs follow,
 * the months the history holds and the months in a row. Every number in
 * them is the core's own rendering, and the window each answer covers is
 * named under it.
 */
export function profileRows(answers: ProgressAnswers): IndexRow[] {
  const { rolling, challenges, streaks } = answers

  /* -------------------------------------------------------------- the goals */
  const challengeRows = challenges.data?.challenges ?? []
  const goalsRow: IndexRow = {
    id: 'goals',
    panel: 'challenges',
    question: t('progress.profile.index.goals.question'),
    answer: challengeRows.length
      ? challengeRows
          .map((challenge) =>
            t('challenge.met', {
              n: challenge.weeks.filter((week) => week.met).length,
              of: challenge.weeks.length,
            }),
          )
          .join(' · ')
      : challenges.isError
        ? t('common.error')
        : t('progress.profile.index.goals.empty'),
    window: challenges.data
      ? `${t('progress.rolling.range', {
          from: challenges.data.window.from_label,
          to: challenges.data.window.to_label,
        })} · ${t('progress.index.weeks', { n: challenges.data.window.weeks })}`
      : null,
    empty: challengeRows.length === 0,
  }

  /* ----------------------------------------------------------- the schedule */
  const weekdays = (rolling.data?.weekday ?? []).filter((day) => day.sessions > 0)
  const scheduleRow: IndexRow = {
    id: 'schedule',
    panel: 'weekday',
    question: t('progress.profile.index.schedule.question'),
    answer: weekdays.length
      ? weekdays.map((day) => day.weekday).join(' · ')
      : rolling.isError
        ? t('common.error')
        : t('progress.rolling.noRows'),
    window: rolling.data
      ? t('progress.rolling.window', { days: rolling.data.window.days })
      : null,
    empty: weekdays.length === 0,
  }

  /* ----------------------------------------------------------- the seasons */
  const months = (rolling.data?.months ?? []).filter((month) => month.sessions > 0)
  const seasonsRow: IndexRow = {
    id: 'seasons',
    panel: 'months',
    question: t('progress.profile.index.seasons.question'),
    answer: months.length
      ? months.map((month) => `${month.label} ${renderedText(month.distance)}`).join(' · ')
      : rolling.isError
        ? t('common.error')
        : t('progress.rolling.noRows'),
    window: rolling.data
      ? t('progress.rolling.window', { days: rolling.data.window.days })
      : null,
    empty: months.length === 0,
  }

  /* --------------------------------------------------------- the months in a row */
  const streakMonths = streaks.data?.months.current ?? null
  const habitRow: IndexRow = {
    id: 'months',
    panel: 'habit',
    question: t('progress.profile.index.habit.question'),
    answer: streakMonths
      ? counted(
          streakMonths.count,
          'progress.profile.index.month',
          'progress.profile.index.months',
        )
      : t('today.streak.none'),
    window: streakMonths?.start_label
      ? t('today.streak.since', { date: streakMonths.start_label })
      : null,
    empty: !streakMonths,
  }

  return [goalsRow, scheduleRow, seasonsRow, habitRow]
}

/**
 * The panel the picture opens: the athlete profile, which has the
 * numbers under the shape.
 */
const SHAPE_ROUTE = '/profile/shape'

/**
 * The areas of the split, and the panels each one still has to open.
 *
 * The questions open some of the wall's panels, and the picture opens
 * the athlete profile's own. Every other panel of the wall is left to
 * the areas: the areas are the ways into the rest of the wall, and they
 * carry no answers and no numbers of their own — the wall's cards carry
 * the numbers, and the questions carry the answers.
 *
 * **An area with nothing left in it is not drawn.** A heading with
 * nothing under it is a heading that says nothing.
 */
export interface AreaLinks {
  /** The area, in the model's order. */
  area: TaskArea
  /** The panels of the area that the index does not open already. */
  cards: WallCard[]
}

/**
 * The areas of one destination, and the panels they carry.
 *
 * A panel the index opens already — a question's own panel, or the
 * picture's — is not opened a second time: the area carries only what is
 * left.
 */
export function areaLinks(rows: IndexRow[], destination: 'progress' | 'profile'): AreaLinks[] {
  const opened = new Set<string>([
    ...rows.map((row) => panelRoute(row.panel)),
    ...(destination === 'profile' ? [SHAPE_ROUTE] : []),
  ])
  return areasOf(destination)
    .map((area) => ({
      area,
      cards: area.cards
        .map((id) => WALL_CARDS.find((card) => card.id === id))
        .filter((card): card is WallCard => card !== undefined)
        .filter((card) => !opened.has(card.to)),
    }))
    .filter((entry) => entry.cards.length > 0)
}

/**
 * The rows of an index, and the areas under them.
 *
 * One list of rows in one card, the way the app's own lists are drawn:
 * the question on its own line, the answer under it, and the window the
 * answer covers under that. The row opens the panel that answers it.
 */
function IndexRows({
  rows,
  prefix,
  origin,
}: {
  rows: IndexRow[]
  prefix: string
  /** The place the reader is on, so that a panel of the other
      destination can send the reader back. */
  origin: '/progress' | '/profile'
}) {
  return (
    <ul className="card overflow-hidden card-flush" data-testid={`${prefix}-rows`}>
      {rows.map((row) => {
        const opening = panelOpening(row.panel, origin)
        return (
          <li key={row.id} className="border-b border-edge last:border-b-0">
            <Link
              to={opening.to}
              state={opening.state}
            data-testid={`${prefix}-${row.id}`}
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
                data-testid={`${prefix}-${row.id}-answer`}
              >
                {row.answer}
              </span>
              {row.window ? (
                <span
                  className="mt-0.5 block text-[11.5px] leading-tight text-ink-muted"
                  data-testid={`${prefix}-${row.id}-window`}
                >
                  {row.window}
                </span>
              ) : null}
            </span>
            <ChevronRight size={18} className="mt-0.5 shrink-0 text-ink-faint" />
          </Link>
          </li>
        )
      })}
    </ul>
  )
}

/**
 * The areas under the questions: the way into the panels the questions
 * do not open already.
 */
function IndexAreas({
  areas,
  prefix,
}: {
  areas: AreaLinks[]
  prefix: string
}) {
  return (
    <nav className="card mt-3 overflow-hidden card-flush" data-testid={`${prefix}-areas`}>
      {areas.map(({ area, cards }) => (
        <section key={area.id} className="px-4 py-3" data-testid={`${prefix}-areas-${area.id}`}>
          <h3 className="mb-1 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t(area.titleKey)}
          </h3>
          <ul>
            {cards.map((card) => (
              <li key={card.id} className="border-t border-edge first:border-t-0">
                <Link
                  to={cardRoute(card.id)}
                  data-testid={`${prefix}-area-${card.id}`}
                  className="tap block py-1.5 text-[14px] font-medium text-ink"
                >
                  {t(card.titleKey)}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </nav>
  )
}

/**
 * The narrow Progress: **an index of the questions**.
 *
 * The six questions of the work, the athlete's answer to each, and a
 * doorway into the panel that answers it. The athlete profile's picture
 * is not here: it is the profile's own, and the profile has an index of
 * its own.
 */
export function ProgressIndex({ answers }: { answers: ProgressAnswers }) {
  const rows = indexRows(answers)
  const areas = areaLinks(rows, 'progress')

  return (
    <div data-testid="progress-index">
      <p className="lede">{t('progress.index.intro')}</p>
      <IndexRows rows={rows} prefix="progress-index" origin="/progress" />
      {/*
        The areas: the wall's own shape, and the way into the panels the
        six questions do not open already. A link per panel, and nothing
        else — the numbers are on the wall's cards, and the answers are
        in the questions above.
      */}
      <IndexAreas areas={areas} prefix="progress-index" />
      <p className="mt-3 text-[12px] leading-snug text-ink-faint">
        {t('progress.index.footnote')}
      </p>
    </div>
  )
}

/**
 * The narrow athlete profile: **the picture, the questions, and the
 * areas**.
 *
 * **And a picture above the questions.** The athlete profile is the one
 * answer in this product that is a shape and nothing else — six axes,
 * three windows, and not a number in the drawing. It answers "what does
 * my training look like?" in one glance, so it is above the list rather
 * than inside it. It opens the panel that has the numbers under it.
 */
export function ProfileIndex({
  answers,
  shape,
}: {
  answers: ProgressAnswers
  /** The three windows the athlete profile is drawn over. */
  shape: {
    windows: ProfileWindow[]
    days: number
    onScope: (days: number) => void
  }
}) {
  const rows = profileRows(answers)
  const areas = areaLinks(rows, 'profile')

  return (
    <div data-testid="profile-index">
      <section className="card mb-3 px-4 py-3" data-testid="profile-index-shape">
        <div className="mb-1 flex items-baseline justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('progress.index.shape.heading')}
          </h2>
          <Link
            to={SHAPE_ROUTE}
            className="tap-inline shrink-0 text-[12.5px] font-semibold text-brand-700 underline"
            data-testid="profile-index-shape-open"
          >
            {t('progress.index.shape.open')}
          </Link>
        </div>
        <ShapeBody windows={shape.windows} error={false} days={shape.days} table={false} />
      </section>
      <p className="lede">{t('progress.profile.index.intro')}</p>
      <IndexRows rows={rows} prefix="profile-index" origin="/profile" />
      <IndexAreas areas={areas} prefix="profile-index" />
      <p className="mt-3 text-[12px] leading-snug text-ink-faint">
        {t('progress.index.footnote')}
      </p>
    </div>
  )
}
