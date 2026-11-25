/**
 * The plan's week: **the kinds of run, the distances and the paces**.
 *
 * When a programme is in force, the week is read from the **adopted
 * plan** — the dated copy the athlete adopted — and not from a baseline
 * inferred from the history. The copy carries the sessions, their kinds,
 * their distances and the paces that were worked out on the day of the
 * adoption.
 *
 * ## The paces carry the threshold they were worked out from
 *
 * A pace with no threshold beside it is a number with no owner. So the
 * card names **what the plan was paced with** — the threshold and the
 * day it was read, or the plan's own rule and the race it was read
 * from — and the sessions' paces are read under it. The threshold is
 * the one in force on the day of the adoption: a later threshold does
 * not re-pace a plan that is running.
 *
 * ## What the card does not say
 *
 * It does not say whether the athlete is keeping up, whether the level
 * suits them, or what to do next. The plan's cells are **intent** and
 * the watch's zones are **evidence**, and the two are never merged into
 * one number.
 */

import { t } from '../i18n'
import { formatDateMedium } from '../lib/format'
import type { AdoptedSession, AdoptedWeek } from '../lib/types'

/**
 * One session of the adopted week.
 *
 * **The kinds of run are the runner's wording** — "tempo", "long run",
 * "easy" — and the method's own name travels beside it as a title, so
 * the two never lead at once.
 */
function SessionRow({ entry }: { entry: AdoptedSession }) {
  return (
    <li
      className="flex items-baseline justify-between gap-3 border-b border-edge py-1.5 last:border-b-0"
      data-testid={`plan-week-session-${entry.date}`}
    >
      <span className="min-w-0">
        <span className="text-[12px] text-ink-muted">{entry.weekday}</span>
        <span className="ml-1.5 text-[13px] font-semibold text-ink-soft">{entry.kind}</span>
        {entry.method_kind ? (
          <span className="ml-1.5 text-[11.5px] text-ink-faint">{entry.method_kind}</span>
        ) : null}
        {entry.distance ? (
          <span className="ml-1.5 text-[12.5px] text-ink-muted">{entry.distance}</span>
        ) : null}
        {entry.detail ? (
          <span className="ml-1.5 text-[11.5px] text-ink-faint">{entry.detail}</span>
        ) : null}
      </span>
      <span className="shrink-0 text-right">
        {entry.pace ? (
          <span className="text-[12.5px] tabular-nums text-ink" data-testid={`plan-week-pace-${entry.date}`}>
            {entry.pace}
          </span>
        ) : entry.no_pace ? (
          <span className="text-[11.5px] text-ink-muted" data-testid={`plan-week-no-pace-${entry.date}`}>
            {entry.no_pace}
          </span>
        ) : null}
        {entry.finish_pace ? (
          <span className="ml-1.5 text-[11.5px] tabular-nums text-ink-muted">
            {entry.finish_pace}
          </span>
        ) : null}
      </span>
    </li>
  )
}

/**
 * The adopted week, with its paces.
 *
 * **The card is the picture**: the sessions of the week, their kinds,
 * their distances and their paces, under the line that says what the
 * plan was paced with.
 */
export function AdoptedWeekCard({ week }: { week: AdoptedWeek }) {
  const pacing = week.pacing
  return (
    <section className="card mt-3 px-4 py-3" data-testid="plan-adopted-week">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="caps">{week.label}</h2>
        <span className="text-[11.5px] text-ink-faint">
          {formatDateMedium(week.start_date)} – {formatDateMedium(week.end_date)}
        </span>
      </div>
      <p className="mt-0.5 text-[12px] text-ink-muted" data-testid="plan-week-source">
        {week.source_line}
      </p>

      {/*
        What the plan was paced with. **A pace with no threshold beside
        it is a number with no owner**, so the threshold and the day it
        was read stand above the paces.
      */}
      {pacing.threshold_line || pacing.rule_line || pacing.missing ? (
        <p className="mt-1.5 text-[12px] text-ink-soft" data-testid="plan-week-paced-with">
          {pacing.threshold_line ?? pacing.rule_line ?? pacing.missing}
        </p>
      ) : null}

      <ul className="mt-1.5" data-testid="plan-week-sessions">
        {week.sessions.map((session) => (
          <SessionRow key={session.date} entry={session} />
        ))}
      </ul>

      {week.shift?.line ? (
        <p className="mt-1.5 text-[11.5px] text-ink-muted" data-testid="plan-week-shift">
          {week.shift.line}
        </p>
      ) : null}

      {/*
        The tier's entry requirement beside the athlete's own mileage:
        **two facts, side by side, unranked**.
      */}
      {week.entry.requirement || week.entry.athlete ? (
        <dl className="mt-2 border-t border-edge pt-1.5 text-[11.5px]">
          {week.entry.requirement ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.week.entry')}</dt>
              <dd className="text-ink-soft" data-testid="plan-week-entry">
                {week.entry.requirement}
              </dd>
            </div>
          ) : null}
          {week.entry.athlete ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.bases.mileage')}</dt>
              <dd className="text-ink-soft" data-testid="plan-week-mileage">
                {week.entry.athlete}
              </dd>
            </div>
          ) : null}
        </dl>
      ) : null}
    </section>
  )
}
