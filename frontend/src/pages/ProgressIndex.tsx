/**
 * The two indexes: **the phone's Progress and the phone's athlete
 * profile**.
 *
 * On the wide screens Progress is a wall: a grid of cards, each one a
 * question with its answer. On a phone that grid is a pile of numbers
 * without context — the app telling the athlete how much data it holds.
 * So the narrow layout is an **index**: the wall's cards, one row each,
 * the athlete's answer to each, and a doorway into the panel that
 * answers it.
 *
 * **The index is the wall's card list.** One list, one order, two
 * widths: the rows are the cards of the model's own order, and the row
 * opens the panel the card opens. There is no second index of the same
 * things — the areas' link lists are gone, because a second, worse index
 * of the same eleven cards is a second thing to read.
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
 */

import { Link } from 'react-router-dom'
import { ChevronRight } from 'lucide-react'
import { ABSENT, formatCount, formatDateMedium, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import type { RollingInsight } from '../lib/types'
import type { ProgressAnswers } from './useProgress'
import { LOAD_EMPTY } from './progressPanels'
import {
  WALL_CARDS,
  WALL_PANELS,
  cardRoute,
  panelOpening,
  panelRoute,
  panelOfCard,
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
 * The answers are the app's existing cards' answers, read from the same
 * answers the cards draw: nothing here is computed a second time, and
 * nothing here is a number the app formatted on its own.
 */
export function indexRows(answers: ProgressAnswers): IndexRow[] {
  const { progress, load, rolling } = answers

  /* ------------------------------------------------- this week, and before */
  const week = answers.today.data?.week
  const weekRow: IndexRow = {
    id: 'week',
    panel: 'week',
    question: t('progress.wall.thisWeek.heading'),
    answer: week
      ? `${renderedText(week.distance)} · ${renderedText(week.difference)}`
      : answers.today.isError
        ? t('common.error')
        : t('common.loading'),
    window: week
      ? t('progress.wall.week.window', {
          from: formatDateMedium(week.days[0]?.date ?? ''),
          to: formatDateMedium(week.days[week.days.length - 1]?.date ?? ''),
          beforeFrom: formatDateMedium(week.previous.from),
          beforeTo: formatDateMedium(week.previous.to),
        })
      : null,
    empty: !week,
  }

  /* ------------------------------------------------------ the training load */
  const loadData = load.data
  const loadState = loadData?.load?.states?.[loadData.load.states.length - 1]
  const loadRow: IndexRow = {
    id: 'load',
    panel: 'load',
    question: t('progress.wall.load.heading'),
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

  /* ------------------------------------------------------- the weeks */
  const rollingData = rolling.data
  const weeksRow: IndexRow = {
    id: 'weeks',
    panel: 'volume',
    question: t('progress.wall.weeks.heading'),
    answer: rollingData
      ? `${renderedText(rollingData.distance.current)} · ${counted(
          rollingData.sessions.current,
          'progress.index.run',
          'progress.index.runs',
        )}`
      : rolling.isError
        ? t('common.error')
        : t('common.loading'),
    window: rollingData
      ? t('progress.rolling.range', {
          from: rollingData.window.from_label,
          to: rollingData.window.to_label,
        })
      : null,
    empty: !rollingData,
  }

  /* ---------------------------------------------- where the time went */
  const shares = rollingData?.zones.shares ?? []
  const timeRow: IndexRow = {
    id: 'time',
    panel: 'zone-time',
    question: t('progress.wall.time.heading'),
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

  /* -------------------------------------------------------------- the pace */
  const paceWeeks = progress.data?.pace_at_hr ?? []
  const paceWeek = paceWeeks[paceWeeks.length - 1]
  const paceRow: IndexRow = {
    id: 'pace',
    panel: 'pace',
    question: t('progress.wall.pace.heading'),
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

  /* ------------------------------------------------- bests & milestones */
  const bests = progress.data?.bests ?? []
  const bestsRow: IndexRow = {
    id: 'bests',
    panel: 'bests',
    question: t('progress.wall.bests.heading'),
    answer: bests.length
      ? bests.map((best) => `${best.value}`).join(' · ')
      : progress.isError
        ? t('common.error')
        : progress.data
          ? t('progress.rolling.noRows')
          : t('common.loading'),
    window: progress.data
      ? t('progress.index.habit.window', {
          from: formatDateMedium(progress.data.period.from),
          to: formatDateMedium(progress.data.period.to),
        })
      : null,
    empty: bests.length === 0,
  }

  return [weekRow, loadRow, weeksRow, timeRow, paceRow, bestsRow]
}

/**
 * The questions the athlete profile asks.
 *
 * **The profile's five cards, and the athlete's answer to each**: the
 * shape, the months, the schedule, the habit and the goals. Every number
 * in them is the core's own rendering, and the window each answer covers
 * is named under it.
 */
export function profileRows(answers: ProgressAnswers, profile: RollingInsight | undefined): IndexRow[] {
  const { rolling, challenges, streaks } = answers

  /* --------------------------------------------------------- the shape */
  const facets = profile?.profile ?? []
  // **Two of the seven.** The row is a doorway, and the shape's own
  // seven facets are on the card and on the panel behind it.
  const shown = facets.filter((facet) => facet.key === 'volume' || facet.key === 'consistency')
  const shapeRow: IndexRow = {
    id: 'shape',
    panel: 'shape',
    question: t('progress.wall.shape.heading'),
    answer: shown.length
      ? shown.map((facet) => `${facet.label} ${renderedText(facet.total)}`).join(' · ')
      : t('common.loading'),
    window: profile
      ? t('progress.rolling.window', { days: profile.window.days })
      : null,
    empty: facets.length === 0,
  }

  /* -------------------------------------------------------- the months */
  const months = (rolling.data?.months ?? []).filter((month) => month.sessions > 0)
  const monthsRow: IndexRow = {
    id: 'months',
    panel: 'months',
    question: t('progress.wall.months.heading'),
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

  /* ------------------------------------------------------- the schedule */
  const weekdays = (rolling.data?.weekday ?? []).filter((day) => day.sessions > 0)
  const scheduleRow: IndexRow = {
    id: 'schedule',
    panel: 'weekday',
    question: t('progress.wall.schedule.heading'),
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

  /* ---------------------------------------------------------- the habit */
  const streakDays = streaks.data?.days.current ?? null
  const streakWeeks = streaks.data?.weeks.current ?? null
  const eddington = answers.progress.data?.eddington ?? null
  const habitRow: IndexRow = {
    id: 'habit',
    panel: 'habit',
    question: t('progress.wall.habit.heading'),
    answer: [
      streakDays
        ? counted(streakDays.count, 'progress.index.day', 'progress.index.days')
        : t('today.streak.none'),
      streakWeeks
        ? counted(streakWeeks.count, 'progress.index.week', 'progress.index.weeks')
        : t('today.streak.none'),
      eddington ? t('progress.index.eddington', { n: formatCount(eddington.number) }) : ABSENT,
    ].join(' · '),
    window: answers.progress.data
      ? t('progress.index.habit.window', {
          from: formatDateMedium(answers.progress.data.period.from),
          to: formatDateMedium(answers.progress.data.period.to),
        })
      : null,
    empty: !streakDays && !streakWeeks && !eddington,
  }

  /* ----------------------------------------------------------- the goals */
  const challengeRows = challenges.data?.challenges ?? []
  const goalsRow: IndexRow = {
    id: 'goals',
    panel: 'challenges',
    question: t('progress.wall.goals.heading'),
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

  return [shapeRow, monthsRow, scheduleRow, habitRow, goalsRow]
}

/**
 * The rows of an index, in the wall's order.
 *
 * The model holds the order, and the index reads it: the rows of a
 * destination are the cards of that destination, in the model's order.
 */
function rowsInOrder(rows: IndexRow[], destination: 'progress' | 'profile'): IndexRow[] {
  const order = WALL_CARDS.filter((card) => card.destination === destination).map(
    (card) => card.id,
  )
  return order
    .map((id) => rows.find((row) => row.id === id))
    .filter((row): row is IndexRow => row !== undefined)
}

/**
 * The rows of an index, and the cards they stand for.
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
 * The narrow Progress: **the wall's cards, one row each**.
 *
 * The six cards of the work, the athlete's answer to each, and a doorway
 * into the panel that answers it.
 */
export function ProgressIndex({ answers }: { answers: ProgressAnswers }) {
  const rows = rowsInOrder(indexRows(answers), 'progress')

  return (
    <div data-testid="progress-index">
      <p className="lede">{t('progress.index.intro')}</p>
      <IndexRows rows={rows} prefix="progress-index" origin="/progress" />
      <p className="mt-3 text-[12px] leading-snug text-ink-faint">
        {t('progress.index.footnote')}
      </p>
    </div>
  )
}

/**
 * The narrow athlete profile: **the profile's cards, one row each**.
 *
 * The five cards of the long pattern — the shape, the months, the
 * schedule, the habit and the goals — and the athlete's answer to each.
 */
export function ProfileIndex({
  answers,
  profile,
}: {
  answers: ProgressAnswers
  /** The focused window's answer: the shape the rows are about. */
  profile: RollingInsight | undefined
}) {
  const rows = rowsInOrder(profileRows(answers, profile), 'profile')

  return (
    <div data-testid="profile-index">
      <p className="lede">{t('progress.profile.index.intro')}</p>
      <IndexRows rows={rows} prefix="profile-index" origin="/profile" />
      <p className="mt-3 text-[12px] leading-snug text-ink-faint">
        {t('progress.index.footnote')}
      </p>
    </div>
  )
}

/**
 * The cards of a destination, in the model's order.
 *
 * The index and the wall draw the same list, and the panel route opens
 * the panel a card opens — so the three cannot drift apart.
 */
export function cardsOfDestination(destination: 'progress' | 'profile'): WallCard[] {
  return WALL_CARDS.filter((card) => card.destination === destination)
}

/** The route a card opens, and the panel behind it. */
export function openingOf(card: WallCard): { to: string; panel: WallPanel | null } {
  return { to: cardRoute(card.id), panel: panelOfCard(card.id) }
}

/** The route a panel opens under. */
export { panelRoute }
