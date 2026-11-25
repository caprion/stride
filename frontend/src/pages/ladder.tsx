/**
 * The ladder: the training paces, drawn.
 *
 * **The ladder is a picture** (D11): the anchors, the rungs with their
 * values, their bands and their rules. The sentence is the caption, and
 * the caption is the core's own — the race the paces came from, and the
 * chain that produced the number.
 *
 * ## The two axes
 *
 * Every rung carries **two** things, and the card never turns one into
 * the other:
 *
 *   - the **heart-rate band** — the method's rule, drawn on the
 *     athlete's own heart-rate axis, in the method's amber;
 *   - the **pace** — the rung's own pace, drawn on the pace axis, in
 *     the athlete's own azure when it hangs on their own race and in the
 *     method's amber when it hangs on the method's rule.
 *
 * **The pace axis runs fast on the left**, the way a runner reads a pace
 * and not the way a number reads.
 *
 * ## What the card carries, and what it does not
 *
 * The card carries the picture, the labels, the numbers, the period and
 * the reason something is missing. **The explanation is a click away**:
 * the source line opens the model's own detail, and a rung's rule opens
 * beside it with its citation. Nothing else explains anything.
 *
 * ## Absence is not zero
 *
 * A rung with no pace is a **hollow marker**: the track stays empty and
 * the reason is written where the pace would have been. A ladder with no
 * race at all draws its axes and says what it needs — never a band of
 * zero, and never a card of zeroes.
 */

import { t } from '../i18n'
import type { PacesInsight, RungRow } from '../lib/types'

/** A band on an axis: where it starts, where it ends. */
interface Band {
  /** The lower edge, in the axis's own unit. */
  from: number
  /** The upper edge, in the axis's own unit. */
  to: number
  /**
   * Whether the band is a ceiling — "below 128 bpm".
   *
   * A ceiling has one edge, and the band runs from the axis's own floor
   * up to it.
   */
  ceiling: boolean
}

/**
 * The heart-rate band of a rung.
 *
 * The band is the method's, and it is either a range or a ceiling. A rung
 * with no band at all has no value for it, and the reason is the core's.
 */
function heartBand(rung: RungRow): Band | null {
  const { from_bpm: from, to_bpm: to, below_bpm: below } = rung.heart_rate
  if (from !== null && to !== null) return { from, to, ceiling: false }
  if (below !== null) return { from: below, to: below, ceiling: true }
  return null
}

/**
 * The pace band of a rung.
 *
 * The long run has two ends — the pace it starts at and the pace it
 * finishes at — and they are drawn as one band with the arrow at the
 * faster end.
 */
function paceBand(rung: RungRow): Band | null {
  const fast = rung.pace.fast_s_per_km
  const slow = rung.pace.slow_s_per_km
  if (fast === null || slow === null) return null
  return { from: Math.min(fast, slow), to: Math.max(fast, slow), ceiling: false }
}

/**
 * Whether the rung's numbers hang on the athlete's own race.
 *
 * **The colours say whose number it is** — the method's rule or the
 * athlete's own data — and not how it compares. Nothing here is a
 * verdict, and no colour marks a problem.
 */
function isAthleteOwn(rung: RungRow): boolean {
  return rung.anchor === 'goal_race'
}

/** The edges of an axis, over every band that sits on it. */
function edges(bands: (Band | null)[]): { min: number; max: number } | null {
  const edges = bands.filter((band): band is Band => band !== null)
  if (edges.length === 0) return null
  let min = Number.POSITIVE_INFINITY
  let max = Number.NEGATIVE_INFINITY
  for (const band of edges) {
    min = Math.min(min, band.from)
    max = Math.max(max, band.to)
  }
  if (min === max) {
    // One value on the axis: give it room, so the band is drawable.
    min -= 1
    max += 1
  }
  return { min, max }
}

/**
 * A band, drawn on its axis.
 *
 * **A band with no value is white space**: the track is drawn and the
 * bar is not, and the reason stands where the band would have been.
 */
function BandBar({
  band,
  scale,
  colour,
  testId,
}: {
  band: Band | null
  scale: { min: number; max: number } | null
  colour: string
  testId: string
}) {
  if (!band || !scale) {
    return (
      <span
        className="block h-[7px] w-full rounded-full"
        style={{ background: 'var(--chart-grid)' }}
        data-testid={`${testId}-empty`}
      />
    )
  }
  const span = scale.max - scale.min
  const from = band.ceiling ? scale.min : band.from
  const left = ((from - scale.min) / span) * 100
  const width = ((band.to - from) / span) * 100
  return (
    <span
      className="relative block h-[7px] w-full rounded-full"
      style={{ background: 'var(--chart-grid)' }}
    >
      <span
        className="absolute inset-y-0 rounded-full"
        style={{
          left: `${Math.max(0, Math.min(100, left))}%`,
          width: `${Math.max(2, Math.min(100 - left, width))}%`,
          background: colour,
        }}
        data-testid={testId}
      />
    </span>
  )
}

/**
 * The ladder's own drawing.
 *
 * **Two axes, one set of rows.** Each rung is a row: the rung's name and
 * its value above, and the two bands below — the heart rate on the left,
 * the pace on the right — each on its own scale, so a band is read
 * against the other bands of the same axis and never against the other
 * axis.
 */
export function LadderPicture({ rungs }: { rungs: RungRow[] }) {
  const hearts = rungs.map((rung) => heartBand(rung))
  const paces = rungs.map((rung) => paceBand(rung))
  const heartScale = edges(hearts)
  const paceScale = edges(paces)

  return (
    <div data-testid="ladder-picture">
      <div className="grid grid-cols-2 gap-3 text-[10.5px] uppercase tracking-[0.07em] text-ink-faint">
        <span>{t('pace.card.hr')}</span>
        <span>{t('pace.card.pace')}</span>
      </div>
      {rungs.length === 0 ? (
        <p className="mt-2 text-[13px] text-ink-muted" data-testid="ladder-picture-empty">
          {t('pace.card.empty')}
        </p>
      ) : null}
      {rungs.map((rung, index) => {
        const heart = hearts[index] ?? null
        const pace = paces[index] ?? null
        const colour = isAthleteOwn(rung) ? 'var(--chart-line)' : 'var(--chart-reference)'
        const value = rung.pace.value ?? rung.pace.missing
        return (
          <div key={rung.kind} className="mt-2 border-t border-edge pt-1.5">
            <div className="flex items-baseline justify-between gap-2">
              <span className="text-[13px] font-semibold text-ink-soft" data-testid={`ladder-rung-${rung.kind}`}>
                {rung.label}
              </span>
              <span
                className={`text-[12.5px] tabular-nums ${value ? 'text-ink' : 'text-ink-muted'}`}
                data-testid={`ladder-value-${rung.kind}`}
              >
                {value ?? ''}
              </span>
            </div>
            <div className="mt-1 grid grid-cols-2 gap-3">
              <span className="block">
                <BandBar
                  band={heart}
                  scale={heartScale}
                  colour="var(--chart-reference)"
                  testId={`ladder-hr-${rung.kind}`}
                />
                <span className="mt-0.5 block text-[10.5px] tabular-nums text-ink-muted">
                  {rung.heart_rate.value ?? rung.heart_rate.missing ?? ''}
                </span>
              </span>
              <span className="block">
                <BandBar
                  band={pace}
                  scale={paceScale}
                  colour={colour}
                  testId={`ladder-pace-${rung.kind}`}
                />
                <span className="mt-0.5 block text-[10.5px] tabular-nums text-ink-muted">
                  {rung.pace.value ?? ''}
                </span>
              </span>
            </div>
            {/*
              The rule, on demand. **The explanation lives here** — beside
              the rung it belongs to, with its citation — and nowhere
              else.
            */}
            {rung.rule.line || rung.rule.citation ? (
              <details className="mt-1 text-[11px] text-ink-faint" data-testid={`ladder-rule-${rung.kind}`}>
                <summary className="cursor-pointer">{t('pace.card.rule')}</summary>
                <p className="mt-0.5 text-[11.5px] text-ink-muted">
                  {rung.rule.line ?? ''}
                  {/*
                    The citation, when the rule's own wording does not
                    carry it: the core's line usually ends with it, and
                    it is not written twice.
                  */}
                  {rung.rule.citation && !rung.rule.line?.includes(rung.rule.citation)
                    ? ` (${rung.rule.citation})`
                    : ''}
                </p>
              </details>
            ) : null}
          </div>
        )
      })}
      {/* The legend: the colours say whose number each one is. */}
      {rungs.length > 0 ? (
        <ul className="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-[10.5px] text-ink-faint">
          <li className="inline-flex items-center gap-1.5">
            <i
              className="inline-block h-[7px] w-4 rounded-full"
              style={{ background: 'var(--chart-reference)' }}
            />
            {t('pace.legend.method')}
          </li>
          <li className="inline-flex items-center gap-1.5">
            <i
              className="inline-block h-[7px] w-4 rounded-full"
              style={{ background: 'var(--chart-line)' }}
            />
            {t('pace.legend.athlete')}
          </li>
        </ul>
      ) : null}
      <p className="mt-2 text-[11.5px] leading-snug text-ink-faint">{t('pace.card.note')}</p>
    </div>
  )
}

/**
 * The source line, hoisted, with the lines behind it behind a click.
 *
 * **Once per screen.** The screen names the zone model and the day it
 * was read; the lines the answer was built from travel behind the
 * click.
 */
export function SourceLine({
  source,
  sources,
  testId = 'ladder-source',
}: {
  /** The one line the screen names. */
  source: string | null
  /** The lines behind the click. */
  sources: { kind: string; line: string | null; date_label: string | null }[]
  testId?: string
}) {
  if (!source) return null
  return (
    <p className="text-[11.5px] text-ink-faint" data-testid={testId}>
      {source}
      {sources.length > 1 ? (
        <details className="ml-1 inline" data-testid={`${testId}-sources`}>
          <summary className="inline cursor-pointer">{t('pace.card.sources')}</summary>
          <ul className="mt-1 space-y-0.5">
            {sources.map((line) => (
              <li key={`${line.kind}:${line.date_label ?? ''}`}>
                {line.line}
                {line.date_label ? ` · ${line.date_label}` : ''}
              </li>
            ))}
          </ul>
        </details>
      ) : null}
    </p>
  )
}

/**
 * The ladder's caption: the race the paces came from, and the chain.
 *
 * **The picture cannot say where it came from** — the caption does, in
 * the core's own words.
 */
function LadderCaption({ paces }: { paces: PacesInsight }) {
  const race = paces.sources.find((line) => line.kind === 'race')
  return (
    <div className="mt-2 text-[11.5px] leading-snug text-ink-faint" data-testid="ladder-caption">
      {race?.line ? <p>{race.line}</p> : null}
      {paces.chain ? <p>{paces.chain}</p> : null}
    </div>
  )
}

/**
 * The race paces the rungs hang on.
 *
 * **The anchors are named**, and the two models that produce them are
 * kept side by side: the gap between them is a fact about two models,
 * and the card never merges them.
 */
function LadderAnchors({ paces }: { paces: PacesInsight }) {
  if (paces.equivalents.length === 0) return null
  return (
    <div className="mt-2" data-testid="ladder-anchors">
      <h3 className="caps-sm">{t('pace.card.anchors')}</h3>
      <ul className="mt-1 space-y-0.5">
        {paces.equivalents.map((equivalent) => (
          <li
            key={equivalent.distance}
            className="flex items-baseline justify-between gap-3 text-[12px]"
          >
            <span className="text-ink-soft">{equivalent.distance}</span>
            <span className="tabular-nums text-ink-muted">
              {equivalent.time ?? ''}
              {equivalent.pace ? ` · ${equivalent.pace}` : ''}
            </span>
          </li>
        ))}
      </ul>
    </div>
  )
}

/**
 * The pace card: the ladder, its caption and its anchors.
 *
 * **The card is the picture.** The source line is hoisted by the screen
 * that draws it, and the card carries the rest: the rungs, the values,
 * the bands, the rules behind a click and the reason a value is missing.
 */
export function LadderCard({
  paces,
  error,
  bare = false,
  hoist = false,
}: {
  /** The ladder's answer. */
  paces: PacesInsight | undefined
  /** Whether the answer could not be read. */
  error: boolean
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
  /** Whether this screen hoists the source line itself. */
  hoist?: boolean
}) {
  const body = error && !paces ? (
    <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
  ) : !paces ? (
    <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  ) : (
    <>
      {hoist ? (
        <SourceLine source={paces.source} sources={paces.sources} testId="ladder-source" />
      ) : null}
      {paces.unavailable ? (
        <p className="text-[13px] text-ink-muted" data-testid="ladder-unavailable">
          {paces.unavailable}
        </p>
      ) : null}
      <div className="mt-1">
        <LadderPicture rungs={paces.rungs} />
      </div>
      <LadderAnchors paces={paces} />
      <LadderCaption paces={paces} />
    </>
  )

  if (bare) return <div data-testid="ladder-card">{body}</div>

  return (
    <section className="card px-4 py-3" data-testid="ladder-card">
      <h2 className="mb-2 caps">{t('progress.wall.pace.heading')}</h2>
      {body}
    </section>
  )
}

/**
 * The panel behind the pace card.
 *
 * **The panel is the answer**: the ladder in full, its source line
 * hoisted above it, and the doorway into the curve that is the second
 * reading of the same athlete.
 */
export function LadderPanel({
  paces,
  error,
  onCurve,
}: {
  paces: PacesInsight | undefined
  error: boolean
  /** The doorway into the cost curve. */
  onCurve: () => void
}) {
  return (
    <div data-testid="progress-panel-ladder">
      <LadderCard paces={paces} error={error} hoist />
      <button
        type="button"
        className="tap-inline mt-2 text-[13px] font-semibold text-brand-700 underline"
        data-testid="ladder-curve-link"
        onClick={onCurve}
      >
        {t('progress.wall.curve.heading')}
      </button>
    </div>
  )
}
