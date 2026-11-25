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
 * ## The week against the week that happened
 *
 * The same card carries **the comparison**: what happened on the day of
 * each planned session, how the distance run stands against the distance
 * planned, and the pace against the plan's pace — **only when both of
 * them exist**. A session with no pace (the general aerobic day carries
 * a heart-rate band and no pace) is compared on its distance or on
 * nothing, and never against a pace that is not there.
 *
 * **Done and not done are facts.** There is no grade, no percentage, no
 * traffic light, and the word "missed" is not in the app.
 *
 * ## What the card does not say
 *
 * It does not say whether the athlete is keeping up, whether the level
 * suits them, or what to do next. The plan's cells are **intent** and
 * the watch's zones are **evidence**, and the two are never merged into
 * one number. It never compares two plans, and it never says which
 * plan is better.
 */

import { Link } from 'react-router-dom'
import { t } from '../i18n'
import { formatDateMedium, renderedText } from '../lib/format'
import type { AdoptedSession, PlanWeek } from '../lib/types'

/**
 * The plain status of a session, in the app's own words.
 *
 * **Done, not done, planned** — the plan engine's statuses, and nothing
 * else. A day that is not a run has no status at all: a rest day is
 * neither done nor not done.
 */
export function statusText(status: AdoptedSession['status']): string | null {
  switch (status) {
    case 'done':
      return t('plan.status.done')
    case 'not_done':
      return t('plan.status.notDone')
    case 'planned':
      return t('plan.status.planned')
    default:
      return null
  }
}

/**
 * One session of the adopted week.
 *
 * **The kinds of run are the runner's wording** — "tempo", "long run",
 * "easy" — and the method's own name travels beside it as a title, so
 * the two never lead at once.
 *
 * The comparison sits under the session: what was run on its day, the
 * difference against the distance planned, and the pace against the
 * plan's pace when both of them exist.
 */
function SessionRow({ entry }: { entry: AdoptedSession }) {
  const status = statusText(entry.status)
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
        {/*
          What happened on the day, and how it stands against the plan.
          **The distance that was run**, the difference against the
          distance planned, and the pace against the plan's pace.
        */}
        {entry.actual ? (
          <span
            className="ml-1.5 text-[12.5px] tabular-nums text-ink"
            data-testid={`plan-week-actual-${entry.date}`}
          >
            {renderedText(entry.actual)}
          </span>
        ) : null}
        {entry.difference ? (
          <span
            className="ml-1.5 text-[12px] text-ink-muted"
            data-testid={`plan-week-difference-${entry.date}`}
          >
            {entry.difference}
          </span>
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
        {entry.pace_comparison ? (
          <span
            className="mt-0.5 block text-[11.5px] text-ink-muted"
            data-testid={`plan-week-pace-comparison-${entry.date}`}
          >
            {entry.pace_comparison}
          </span>
        ) : null}
        {status ? (
          <span
            className="mt-0.5 block text-[11.5px] text-ink-muted"
            data-testid={`plan-week-status-${entry.date}`}
          >
            {status}
          </span>
        ) : null}
      </span>
    </li>
  )
}

/**
 * The adopted week, with its paces and with what happened on its days.
 *
 * **The card is the picture**: the sessions of the week, their kinds,
 * their distances and their paces, under the line that says what the
 * plan was paced with — and the comparison of the week against the week
 * that happened.
 */
export function AdoptedWeekCard({ week }: { week: PlanWeek }) {
  const adopted = week.adopted
  if (!adopted) return null
  const pacing = adopted.pacing
  const totals = week.against?.totals ?? null
  return (
    <section className="card mt-3 px-4 py-3" data-testid="plan-adopted-week">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="caps">{adopted.label}</h2>
        <span className="text-[11.5px] text-ink-faint">
          {formatDateMedium(adopted.start_date)} – {formatDateMedium(adopted.end_date)}
        </span>
      </div>
      <p className="mt-0.5 text-[12px] text-ink-muted" data-testid="plan-week-source">
        {adopted.source_line}
      </p>

      {/*
        The week's totals against the plan's — the way the week card
        compares this week with last week.
      */}
      {totals ? (
        <div className="mt-1.5" data-testid="plan-week-totals">
          <p className="text-[14px] font-semibold text-ink" data-testid="plan-week-totals-line">
            {totals.line}
          </p>
          {totals.difference ? (
            <p
              className="text-[12.5px] tabular-nums text-ink-muted"
              data-testid="plan-week-totals-difference"
            >
              {renderedText(totals.difference)}
            </p>
          ) : null}
        </div>
      ) : null}

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
        {adopted.sessions.map((session) => (
          <SessionRow key={session.date} entry={session} />
        ))}
      </ul>

      {adopted.shift?.line ? (
        <p className="mt-1.5 text-[11.5px] text-ink-muted" data-testid="plan-week-shift">
          {adopted.shift.line}
        </p>
      ) : null}

      {/*
        The tier's entry requirement beside the athlete's own mileage:
        **two facts, side by side, unranked**.
      */}
      {adopted.entry.requirement || adopted.entry.athlete ? (
        <dl className="mt-2 border-t border-edge pt-1.5 text-[11.5px]">
          {adopted.entry.requirement ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.week.entry')}</dt>
              <dd className="text-ink-soft" data-testid="plan-week-entry">
                {adopted.entry.requirement}
              </dd>
            </div>
          ) : null}
          {adopted.entry.athlete ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.bases.mileage')}</dt>
              <dd className="text-ink-soft" data-testid="plan-week-mileage">
                {adopted.entry.athlete}
              </dd>
            </div>
          ) : null}
        </dl>
      ) : null}
    </section>
  )
}

/**
 * The plan's week against the week that happened.
 *
 * **One question, one answer.** Where the week stands against the plan:
 * the plan's own source, what it was paced with, the week's totals
 * against the plan's, and every planned session with what happened on
 * its day.
 *
 * The four states are the answer's own:
 *
 *   - **no plan adopted** — the app says so and points at the
 *     programmes there are to adopt;
 *   - **the plan has not started** — the app says when it starts;
 *   - **the week is running** — the comparison so far;
 *   - **the plan has passed** — the app says when its last week ended.
 *
 * **Done and not done are facts.** There is no grade and no percentage,
 * and the app never compares two plans.
 */
export function PlanAgainstCard({
  week,
  bare = false,
}: {
  week: PlanWeek | undefined
  /**
   * Whether the block is drawn inside another card.
   *
   * On the wall the block lives **inside** the card that answers "this
   * week", so it brings no card of its own; on its own page it is a card
   * like any other.
   */
  bare?: boolean
}) {
  const against = week?.against
  if (!against) return null
  const totals = against.totals
  const sessions = week?.adopted?.sessions ?? []

  return (
    <section
      className={bare ? 'mt-3 border-t border-edge pt-2' : 'card px-4 py-3'}
      data-testid="plan-against"
    >
      <h2 className="caps">{t('plan.against.heading')}</h2>

      {/*
        The state. **A fact, in the core's own words** — and the door to
        the programmes when there is no plan at all. The app never offers
        a programme the athlete did not adopt.
      */}
      {against.state_line ? (
        <p className="mt-1 text-[13px] text-ink-soft" data-testid="plan-against-note">
          {against.state_line}
        </p>
      ) : null}
      {against.choose ? (
        <Link
          to="/plan/bases"
          className="tap-inline mt-1 block text-[13px] text-brand-700 underline"
          data-testid="plan-against-choose"
        >
          {against.choose}
        </Link>
      ) : null}

      {/* The plan's own source, and what it was paced with. */}
      {against.source_line ? (
        <p className="mt-1 text-[12px] text-ink-muted" data-testid="plan-against-source">
          {against.source_line}
        </p>
      ) : null}
      {against.paced_with ? (
        <p className="text-[12px] text-ink-muted" data-testid="plan-against-paced">
          {against.paced_with}
        </p>
      ) : null}

      {/*
        The week's totals against the plan's — the way the week card
        compares this week with last week.
      */}
      {totals ? (
        <div className="mt-2" data-testid="plan-against-totals">
          <p className="text-[15px] font-semibold text-ink" data-testid="plan-against-totals-line">
            {totals.line}
          </p>
          {totals.difference ? (
            <p
              className="text-[12.5px] tabular-nums text-ink-muted"
              data-testid="plan-against-difference"
            >
              {renderedText(totals.difference)}
            </p>
          ) : null}
        </div>
      ) : null}

      {/*
        The sessions of the plan's week, each with what happened on its
        day. The plan's own row is the one above; the comparison sits
        under it.
      */}
      {sessions.length > 0 ? (
        <ul className="mt-2" data-testid="plan-against-sessions">
          {sessions.map((session) => (
            <li
              key={session.date}
              className="flex items-baseline justify-between gap-3 border-b border-edge py-1 last:border-b-0"
              data-testid={`plan-against-session-${session.date}`}
            >
              <span className="min-w-0 text-[12.5px] text-ink-soft">
                <span className="text-ink-muted">{session.weekday.slice(0, 3)}</span>{' '}
                <span className="font-semibold">{session.kind}</span>
                {session.distance ? (
                  <span className="ml-1.5 text-ink-muted">{session.distance}</span>
                ) : null}
                {session.actual ? (
                  <span className="ml-1.5 tabular-nums text-ink">
                    {renderedText(session.actual)}
                  </span>
                ) : null}
                {session.difference ? (
                  <span className="ml-1.5 text-ink-muted">{session.difference}</span>
                ) : null}
                {session.pace_comparison ? (
                  <span
                    className="ml-1.5 text-ink-muted"
                    data-testid={`plan-against-pace-${session.date}`}
                  >
                    {session.pace_comparison}
                  </span>
                ) : null}
              </span>
              <span className="shrink-0 text-[11.5px] text-ink-muted">
                {statusText(session.status)}
              </span>
            </li>
          ))}
        </ul>
      ) : null}

      {/*
        The tier's entry requirement beside the athlete's own mileage:
        **two facts, side by side, unranked**. The app never says which
        level fits.
      */}
      {against.entry?.requirement || against.entry?.athlete ? (
        <dl className="mt-2 border-t border-edge pt-1.5 text-[11.5px]">
          {against.entry.requirement ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.against.entry')}</dt>
              <dd className="text-ink-soft" data-testid="plan-against-entry">
                {against.entry.requirement}
              </dd>
            </div>
          ) : null}
          {against.entry.athlete ? (
            <div className="flex items-baseline justify-between gap-3">
              <dt className="text-ink-muted">{t('plan.against.mileage')}</dt>
              <dd className="text-ink-soft" data-testid="plan-against-mileage">
                {against.entry.athlete}
              </dd>
            </div>
          ) : null}
        </dl>
      ) : null}
    </section>
  )
}
