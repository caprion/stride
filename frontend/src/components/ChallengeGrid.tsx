/**
 * The challenge grid: the athlete's own challenges, week by week.
 *
 * **A week is met or it is not.** The grid is a row per challenge and a
 * column per week, and the cell says what happened: a filled mark for a
 * week that reached the target, a quiet one for a week that did not.
 *
 * **Nothing here is a failure.** The week that did not reach the target
 * is a week, and it is drawn quietly rather than crossed out: this
 * product has no red, and a row of red crosses beside a goal the athlete
 * set themselves is the opposite of what the grid is for. The **streak**
 * at the end is a count of weeks in a row — a fact, not a grade — and
 * the **pattern** across the row is the reading: the athlete sees their
 * own weeks and draws the conclusion.
 *
 * **A week with no record in it is not a rest week.** The record does
 * not cover every week, and a week the watch did not record is a week
 * the answer knows nothing about. It is drawn as neither met nor missed.
 */

import { t } from '../i18n'
import { renderedText } from '../lib/format'
import type { ChallengeRow } from '../lib/types'

/** One challenge's row of weeks. */
function ChallengeLine({ challenge }: { challenge: ChallengeRow }) {
  const met = challenge.weeks.filter((week) => week.met).length
  return (
    <tr data-testid={`challenge-line-${challenge.key}`}>
      <th
        scope="row"
        className="whitespace-nowrap py-2 pr-3 text-left align-middle text-[12.5px] font-medium text-ink-soft"
      >
        {challenge.label}
        <span className="mt-0.5 block text-[11px] text-ink-faint">
          {t('challenge.met', { n: met, of: challenge.weeks.length })}
          {challenge.streak
            ? ` \u00b7 ${t('challenge.inARow', { n: renderedText(challenge.streak) })}`
            : ''}
        </span>
      </th>
      {challenge.weeks.map((week) => (
        <td key={week.week_start} className="p-0.5 text-center">
          <span
            className="mx-auto block size-2.5 rounded-full"
            style={{
              background: week.met ? 'var(--brand-500)' : 'var(--surface-sunken)',
            }}
            title={`${week.label} · ${week.measured.text}`}
            data-testid={`challenge-cell-${challenge.key}-${week.week_start}`}
            data-met={week.met ? 'true' : 'false'}
          />
        </td>
      ))}
    </tr>
  )
}

export function ChallengeGrid({
  challenges,
  weeks,
}: {
  challenges: ChallengeRow[]
  /** The week labels, oldest first — the column headings. */
  weeks: { week_start: string; label: string }[]
}) {
  if (challenges.length === 0) {
    return (
      <p className="text-[13px] text-ink-muted" data-testid="challenge-grid-empty">
        {t('challenge.empty')}
      </p>
    )
  }

  return (
    <div className="overflow-x-auto" data-testid="challenge-grid">
      <table className="w-full border-collapse">
        <thead>
          <tr>
            {/* The corner of the grid: it names the rows for a screen
                reader and carries nothing a reader can see. */}
            <th
              scope="col"
              aria-label={t('challenge.challenges')}
              className="pb-1 text-left text-[11px] font-medium text-ink-faint"
            />
            {weeks.map((week) => (
              <th
                key={week.week_start}
                scope="col"
                className="w-8 whitespace-nowrap pb-1 text-[10px] font-normal text-ink-faint"
              >
                {/* The week, as the api wrote it: "7 Sep 2026". The
                    heading is the day and the month. */}
                {week.label.split(' ').slice(0, 2).join(' ')}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {challenges.map((challenge) => (
            <ChallengeLine key={challenge.key} challenge={challenge} />
          ))}
        </tbody>
      </table>
    </div>
  )
}
