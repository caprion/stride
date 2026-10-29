/**
 * The plan sources: where a plan came from, and the words it is judged with.
 *
 * **Two kinds of plan, two vocabularies, never mixed.**
 *
 * * A **prescribed** plan — Garmin's scheduled workouts or an imported
 *   document — is judged *hit*, *missed pace*, *short*, *missed*,
 *   *substituted*, *bonus*. Somebody wrote it down.
 * * An **inferred** plan — the athlete's own usual week, derived from their
 *   history — is judged *less than usual*, *more than usual*, *on a usual
 *   day*, *no usual session*. Nobody wrote it down, so nothing was missed.
 *
 * The words for the inferred case come from the payload
 * (`verdict_label`); the map here is the fallback for the case where the
 * payload does not carry them.
 */

import type { PlanSource, PlanSourceResult } from '../types_phase2'

/** The tone of a badge, from the shared kit. */
export type VerdictTone = 'neutral' | 'brand' | 'positive' | 'warn' | 'negative' | 'info'

/** The tone each verdict is shown with. */
const TONES: Record<string, VerdictTone> = {
  // A prescription: somebody wrote this plan down.
  hit: 'positive',
  missed_pace: 'warn',
  short: 'warn',
  missed: 'negative',
  substituted: 'info',
  bonus: 'brand',
  // A baseline: nobody prescribed this, so nothing is a failure.
  less_than_usual: 'info',
  more_than_usual: 'brand',
  on_a_usual_day: 'positive',
  no_usual_session: 'neutral',
}

/** The words each verdict is shown with, when the payload has none. */
const LABELS: Record<string, string> = {
  hit: 'Hit',
  missed_pace: 'Missed pace',
  short: 'Short',
  missed: 'Missed',
  substituted: 'Substituted',
  bonus: 'Bonus',
  less_than_usual: 'less than usual',
  more_than_usual: 'more than usual',
  on_a_usual_day: 'on a usual day',
  no_usual_session: 'no usual session',
}

/** The tone a verdict is shown with. */
export function verdictTone(verdict: string | null | undefined): VerdictTone {
  if (!verdict) return 'neutral'
  return TONES[verdict] ?? 'neutral'
}

/**
 * The words a verdict is shown with.
 *
 * The payload's own words win: the inferred case carries its vocabulary in
 * `verdict_label`, and the app must not put the prescription's words on a
 * baseline.
 */
export function verdictLabel(
  result: { verdict?: string | null; verdict_label?: string | null } | null | undefined,
): string {
  const label = result?.verdict_label?.trim()
  if (label) return label
  const verdict = result?.verdict ?? ''
  return LABELS[verdict] ?? (verdict ? verdict.replace(/_/g, ' ') : '—')
}

/** The verdicts a baseline can produce. Never "missed". */
export const BASELINE_VERDICTS = [
  'less_than_usual',
  'more_than_usual',
  'on_a_usual_day',
  'no_usual_session',
]

/**
 * Read the sources out of whatever the route answered with.
 *
 * The route is new, so the reader is tolerant: a list, an `items` list, a
 * `sources` list or a single source all mean the same thing here.
 */
export function readPlanSources(payload: unknown): PlanSource[] {
  if (Array.isArray(payload)) return payload.filter(isSource)
  if (!payload || typeof payload !== 'object') return []
  const record = payload as Record<string, unknown>
  for (const key of ['items', 'sources', 'plans']) {
    const value = record[key]
    if (Array.isArray(value)) return value.filter(isSource)
  }
  return isSource(record) ? [record as PlanSource] : []
}

/** Whether a value is a plan source. */
function isSource(value: unknown): value is PlanSource {
  if (!value || typeof value !== 'object') return false
  const record = value as Record<string, unknown>
  return typeof record.prescribed === 'boolean'
}

/** The prescribed source — the plan the athlete wrote down. */
export function prescribedSource(sources: PlanSource[]): PlanSource | null {
  return (
    sources.find((source) => source.prescribed && (source.sessions?.length ?? 0) > 0) ??
    sources.find((source) => source.prescribed) ??
    null
  )
}

/** The inferred source — the athlete's own usual week. */
export function inferredSource(sources: PlanSource[]): PlanSource | null {
  return sources.find((source) => !source.prescribed) ?? null
}

/**
 * Whether a session is still in the future.
 *
 * A session that has not happened is not a failure: it reads "Not yet", in
 * a neutral badge, whatever the verdict says.
 */
export function isFutureSession(date: string, today: string): boolean {
  return date > today
}

/** The result of a session, whatever shape it arrived in. */
export function resultOf(
  session: { result?: PlanSourceResult | null } | null | undefined,
): PlanSourceResult | null {
  return session?.result ?? null
}
