/**
 * The physiological-cost curve: the share of the maximum against time.
 *
 * **A second reading of the same athlete**, and not a second ladder: no
 * rungs, and no pace bands repeated. The vertical axis is the share of
 * the athlete's maximum a duration holds; the horizontal axis is the
 * time it is held for, on a log scale.
 *
 * ## What is on the curve
 *
 *   - the **curve** — the relation itself, in the engine's own grey;
 *   - the **athlete's own races** — each at its own time and the share
 *     its own pace cost, in the athlete's azure;
 *   - the **method's sessions** — each a band over its duration and its
 *     prescribed pace, in the method's amber;
 *   - the **field test's marker** — the test's own place in time;
 *   - the **pace equivalents** — along the bottom.
 *
 * ## What it never does
 *
 * It never judges: nothing here says a session is too hard, and nothing
 * ranks the athlete's races against the method's sessions. And it never
 * turns the method's heart-rate bands into a share of the maximum: the
 * two are different numbers, and converting one into the other needs the
 * athlete's lactate relation, which the app does not have.
 */

import { t } from '../i18n'
import type { CurveInsight, CurveSessionRow } from '../lib/types'
import { SourceLine } from './ladder'

/** The drawing's own geometry. */
const PLOT = { x0: 34, x1: 306, y0: 18, y1: 168 }
const MINUTES_LOW = 4
const MINUTES_HIGH = 130
const PERCENT_LOW = 35
const PERCENT_HIGH = 100

/** The horizontal place of a duration, on a log scale. */
function xOf(minutes: number): number {
  const low = Math.log10(MINUTES_LOW)
  const high = Math.log10(MINUTES_HIGH)
  const at = Math.log10(Math.max(MINUTES_LOW, Math.min(MINUTES_HIGH, minutes)))
  return PLOT.x0 + ((at - low) / (high - low)) * (PLOT.x1 - PLOT.x0)
}

/** The vertical place of a share of the maximum. */
function yOf(percent: number): number {
  const at = Math.max(PERCENT_LOW, Math.min(PERCENT_HIGH, percent))
  return PLOT.y1 - ((at - PERCENT_LOW) / (PERCENT_HIGH - PERCENT_LOW)) * (PLOT.y1 - PLOT.y0)
}

/** The ticks of the time axis. */
const MINUTE_TICKS = [5, 10, 20, 45, 60, 120]

/**
 * The curve's drawing.
 *
 * **The relation is a line, the races are dots, and the method's
 * sessions are bands.** The three are told apart by their shape as well
 * as by their colour, so the drawing is read without the legend.
 */
export function CurvePicture({ curve }: { curve: CurveInsight }) {
  const points = curve.curve
    .filter((point) => point.percent_max.raw !== null)
    .map((point) => `${xOf(point.minutes)},${yOf(point.percent_max.raw ?? 0)}`)
    .join(' ')

  return (
    <svg
      viewBox="0 0 320 210"
      preserveAspectRatio="xMinYMin meet"
      className="block h-auto w-full"
      role="img"
      aria-label={t('progress.wall.curve.heading')}
      data-testid="curve-picture"
    >
      {/* The grid, and the two axes' own ticks. */}
      {MINUTE_TICKS.map((minutes) => (
        <g key={minutes}>
          <line
            x1={xOf(minutes)}
            x2={xOf(minutes)}
            y1={PLOT.y0}
            y2={PLOT.y1}
            stroke="var(--chart-grid)"
            strokeWidth={1}
          />
          <text
            x={xOf(minutes)}
            y={PLOT.y1 + 12}
            fontSize={7.5}
            fill="var(--chart-axis)"
            textAnchor="middle"
          >
            {minutes}
          </text>
        </g>
      ))}
      {[50, 75, 100].map((percent) => (
        <g key={percent}>
          <line
            x1={PLOT.x0}
            x2={PLOT.x1}
            y1={yOf(percent)}
            y2={yOf(percent)}
            stroke="var(--chart-grid)"
            strokeWidth={1}
          />
          <text x={PLOT.x0 - 4} y={yOf(percent) + 3} fontSize={7.5} fill="var(--chart-axis)" textAnchor="end">
            {percent}
          </text>
        </g>
      ))}

      {/* The curve: the relation itself, in the engine's own grey. */}
      {points ? (
        <polyline
          points={points}
          fill="none"
          stroke="var(--chart-axis)"
          strokeWidth={2}
          data-testid="curve-line"
        />
      ) : null}

      {/* The method's sessions: a band over its duration and its pace. */}
      {curve.sessions.map((session) => (
        <SessionBand key={session.kind} session={session} />
      ))}

      {/* The athlete's own races: a dot at their own time and share. */}
      {curve.races.map((race) => (
        <circle
          key={`${race.label}-${race.date}`}
          cx={xOf(race.minutes)}
          cy={yOf(race.percent_max.raw ?? 0)}
          r={3.2}
          fill="var(--chart-line)"
          data-testid={`curve-race-${race.date}`}
        />
      ))}

      {/* The field test's own place in time: a hollow marker. */}
      {curve.field_test ? (
        <circle
          cx={xOf(curve.field_test.minutes)}
          cy={yOf(curve.field_test.percent_max.raw ?? 0)}
          r={3.2}
          fill="none"
          stroke="var(--chart-axis)"
          strokeWidth={1.4}
          data-testid="curve-field-test"
        />
      ) : null}

      {/* The axes' own names. */}
      <text x={PLOT.x0} y={PLOT.y0 - 6} fontSize={7.5} fill="var(--chart-axis)">
        {t('curve.card.percent')}
      </text>
      <text x={PLOT.x1} y={PLOT.y1 + 24} fontSize={7.5} fill="var(--chart-axis)" textAnchor="end">
        {t('curve.card.minutes')}
      </text>
    </svg>
  )
}

/**
 * One of the method's own sessions, as a band on the curve.
 *
 * **A session with no length has no band**: the long run's length comes
 * with the plan's own weeks, and until it does the session is named and
 * not drawn.
 */
function SessionBand({ session }: { session: CurveSessionRow }) {
  const from = session.from_minutes
  const to = session.to_minutes
  const fromPercent = session.from_percent_max?.raw ?? null
  const toPercent = session.to_percent_max?.raw ?? null
  if (from === null || to === null || fromPercent === null || toPercent === null) return null
  return (
    <line
      x1={xOf(from)}
      y1={yOf(fromPercent)}
      x2={xOf(to)}
      y2={yOf(toPercent)}
      stroke="var(--chart-reference)"
      strokeWidth={4}
      strokeLinecap="round"
      data-testid={`curve-session-${session.kind}`}
    />
  )
}

/**
 * The curve's card.
 *
 * **The picture, then the labels, then the numbers.** The card carries
 * the drawing, the legend, the races and the sessions — and the note
 * that says the two axes are not converted into one another.
 */
export function CurveCard({
  curve,
  error,
  bare = false,
  hoist = false,
}: {
  curve: CurveInsight | undefined
  error: boolean
  bare?: boolean
  hoist?: boolean
}) {
  const body = error && !curve ? (
    <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
  ) : !curve ? (
    <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  ) : (
    <>
      {hoist ? (
        <SourceLine source={curve.source} sources={curve.sources} testId="curve-source" />
      ) : null}
      {curve.unavailable ? (
        <p className="text-[13px] text-ink-muted" data-testid="curve-unavailable">
          {curve.unavailable}
        </p>
      ) : null}
      {curve.curve.length > 0 ? (
        <div className="mt-1">
          <CurvePicture curve={curve} />
        </div>
      ) : null}
      <CurveLegend curve={curve} />
      {curve.races.length > 0 ? (
        <div className="mt-2" data-testid="curve-races">
          <h3 className="caps-sm">{t('curve.card.races')}</h3>
          <ul className="mt-1 space-y-0.5">
            {curve.races.map((race) => (
              <li
                key={`${race.label}-${race.date}`}
                className="flex items-baseline justify-between gap-3 text-[12px]"
              >
                <span className="text-ink-soft">
                  {race.label} · {race.date_label}
                </span>
                <span className="tabular-nums text-ink-muted">
                  {race.time} · {race.percent_max.text}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {curve.sessions.length > 0 ? (
        <div className="mt-2" data-testid="curve-sessions">
          <h3 className="caps-sm">{t('curve.card.sessions')}</h3>
          <ul className="mt-1 space-y-0.5">
            {curve.sessions.map((session) => (
              <li
                key={session.kind}
                className="flex items-baseline justify-between gap-3 text-[12px]"
              >
                <span className="text-ink-soft">{session.label}</span>
                <span className="tabular-nums text-ink-muted">
                  {session.pace?.value ?? session.missing ?? ''}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {curve.equivalents.length > 0 ? (
        <div className="mt-2" data-testid="curve-equivalents">
          <h3 className="caps-sm">{t('curve.card.equivalents')}</h3>
          <ul className="mt-1 space-y-0.5">
            {curve.equivalents.map((equivalent) => (
              <li
                key={equivalent.distance}
                className="flex items-baseline justify-between gap-3 text-[12px]"
              >
                <span className="text-ink-soft">{equivalent.distance}</span>
                <span className="tabular-nums text-ink-muted">{equivalent.time ?? ''}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      <p className="mt-2 text-[11.5px] leading-snug text-ink-faint">{t('curve.card.note')}</p>
    </>
  )

  if (bare) return <div data-testid="curve-card">{body}</div>

  return (
    <section className="card px-4 py-3" data-testid="curve-card">
      <h2 className="mb-2 caps">{t('progress.wall.curve.heading')}</h2>
      {body}
    </section>
  )
}

/**
 * The curve's legend.
 *
 * **The shapes say what each thing is**, so the legend is a reading of
 * the drawing and not a key to a mystery.
 */
function CurveLegend({ curve }: { curve: CurveInsight }) {
  if (curve.curve.length === 0 && curve.races.length === 0 && curve.sessions.length === 0) {
    return null
  }
  return (
    <ul className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[10.5px] text-ink-faint">
      <li className="inline-flex items-center gap-1.5">
        <i className="inline-block h-[2px] w-4" style={{ background: 'var(--chart-axis)' }} />
        {t('curve.card.curve')}
      </li>
      <li className="inline-flex items-center gap-1.5">
        <i
          className="inline-block h-2 w-2 rounded-full"
          style={{ background: 'var(--chart-line)' }}
        />
        {t('curve.card.races')}
      </li>
      <li className="inline-flex items-center gap-1.5">
        <i
          className="inline-block h-[3px] w-4 rounded-full"
          style={{ background: 'var(--chart-reference)' }}
        />
        {t('curve.card.sessions')}
      </li>
      {curve.field_test ? (
        <li className="inline-flex items-center gap-1.5">
          <i
            className="inline-block h-2 w-2 rounded-full border"
            style={{ borderColor: 'var(--chart-axis)' }}
          />
          {t('curve.card.fieldTest')}
        </li>
      ) : null}
    </ul>
  )
}
