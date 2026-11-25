/**
 * Today: **the plan's options, with the plan's pacing**.
 *
 * The day's sessions are the plan's own — the session the plan has for
 * the day, and the other sessions it holds — and when a programme is in
 * force they carry **the pace the plan was worked out with**.
 *
 * **A pace with no threshold beside it is a number with no owner.** So
 * the card names what the plan was paced with once, above the sessions:
 * the threshold and the day it was read, or the plan's own rule and the
 * race it was read from. The paces are read under it.
 *
 * The paces are the **adopted** plan's — the dated copy — and not the
 * app's own arithmetic: a later threshold does not re-pace a plan that
 * is running.
 */

import { t } from '../i18n'
import type { AdoptedSession, AdoptedWeek } from '../lib/types'

/**
 * The pace one session of the day carries, if the plan gave it one.
 *
 * **A session with no pace says why** — in the core's own words — and
 * shows no pace at all.
 */
export function SessionPace({ entry }: { entry: AdoptedSession | undefined }) {
  if (!entry) return null
  if (entry.pace) {
    return (
      <span className="text-[12.5px] tabular-nums text-ink" data-testid={`today-pace-${entry.date}`}>
        {entry.pace}
      </span>
    )
  }
  if (entry.no_pace) {
    return (
      <span className="text-[11.5px] text-ink-muted" data-testid={`today-pace-${entry.date}`}>
        {entry.no_pace}
      </span>
    )
  }
  return null
}

/**
 * The plan's pacing, named once.
 *
 * The threshold and the day it was read, or the plan's own rule and the
 * race it was read from — and the reason when there is no pace at all.
 */
export function PlanPacing({ week }: { week: AdoptedWeek | null | undefined }) {
  if (!week) return null
  const line =
    week.pacing.threshold_line ?? week.pacing.rule_line ?? week.pacing.missing ?? null
  if (!line) return null
  return (
    <p className="text-[11.5px] text-ink-faint" data-testid="today-pacing">
      {t('today.options.pacing')} · {line}
    </p>
  )
}
