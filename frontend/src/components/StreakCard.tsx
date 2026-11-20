/**
 * The streak: a count and the days it covers.
 *
 * A streak is a count and a date — never a verdict, never a score. The
 * card is the same one Today shows and the same one the narrow Progress
 * opens on its own, so the numbers read the same wherever they appear.
 *
 * The card is only there when the answer is: a failed query is not a
 * streak of zero, and it must not be drawn as one.
 */

import type { StreakInsight } from '../lib/types'
import { formatCount } from '../lib/format'
import { t } from '../i18n'

export function StreakCard({
  streaks,
  className = 'card px-4 py-3',
  testId = 'today-streak',
}: {
  streaks: StreakInsight
  className?: string
  /** The test id of the card, and the prefix of the rows inside it. */
  testId?: string
}) {
  const rows = [
    {
      key: 'days-current',
      label: t('today.streak.days'),
      streak: streaks.days.current ?? null,
    },
    {
      key: 'days-longest',
      label: t('today.streak.daysLongest'),
      streak: streaks.days.longest ?? null,
    },
    {
      key: 'weeks-current',
      label: t('today.streak.weeks'),
      streak: streaks.weeks.current ?? null,
    },
    {
      key: 'weeks-longest',
      label: t('today.streak.weeksLongest'),
      streak: streaks.weeks.longest ?? null,
    },
  ] as const

  return (
    <section className={className} data-testid={testId}>
      <h2 className="text-[12px] font-semibold uppercase tracking-wide text-ink-muted">
        {t('today.streak.heading')}
      </h2>
      <ul className="mt-1.5 space-y-1.5">
        {rows.map((row) => (
          <li key={row.key}>
            <div className="flex items-baseline justify-between gap-3">
              <span className="text-[13.5px] text-ink-soft">{row.label}</span>
              <span
                className="text-[16px] font-semibold tabular-nums text-ink"
                data-testid={`${testId}-${row.key}`}
              >
                {row.streak ? formatCount(row.streak.count) : t('today.streak.none')}
              </span>
            </div>
            {row.streak ? (
              <p className="text-[11.5px] text-ink-faint">
                {/* The api's own rendering of the days — the
                    core's words, not a date the app formats. */}
                {row.streak.end && row.streak.end_label
                  ? t('today.streak.range', {
                      from: row.streak.start_label,
                      to: row.streak.end_label,
                    })
                  : t('today.streak.since', {
                      date: row.streak.start_label,
                    })}
              </p>
            ) : null}
          </li>
        ))}
      </ul>
    </section>
  )
}
