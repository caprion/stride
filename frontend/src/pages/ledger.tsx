/**
 * The ledger: **the two blocks of zone time**, side by side.
 *
 * The time the athlete spent in zones has **two bases**, and the card
 * draws both of them and never adds them up:
 *
 *   - **the app's own zone time** — the time in each of the athlete's
 *     own bands, measured over **the time the athlete was running**;
 *   - **the watch's own zone minutes** — the time in each of the watch's
 *     five zones, measured over **the whole run**.
 *
 * ## Why two blocks and not one number
 *
 * The two measurements are of different things. The app's bands are the
 * athlete's own bands, read on the model their watch was set to on the
 * run's day; the watch's minutes are the watch's own, and where each of
 * those zones starts is not recorded here. **Adding them together would
 * be a second fact that neither of them says.**
 *
 * So the card draws the two blocks as two blocks, with their own
 * headings and their own basis, and the **overlap** underneath: the
 * stretch of time where both bases exist, the two measurements side by
 * side, and the distance between them. The overlap is the app's check
 * against itself — a fact about the two measurements, and not a verdict
 * on either.
 *
 * ## Absence is not zero
 *
 * A band a week held none is **white space**: the cell is empty, and the
 * hole is the finding. A whole block with no value says why, in the
 * core's own words, and shows no number at all.
 */

import type { ReactNode } from 'react'
import { t } from '../i18n'
import type { RenderedValue, StimulusInsight } from '../lib/types'
import { SourceLine } from './ladder'

/** One band of a block: its label, its time and its share. */
interface Band {
  key: string
  label: string
  time: string | null
  /** The share, in the core's own words: "27.22%". */
  shareText: string | null
  /** The share as a number, for the width of the bar. */
  share: number | null
}

/**
 * A block's bands, as the drawing reads them.
 *
 * The app's own bands are the three the core names; the watch's own are
 * its five zones.
 */
function bandsOf(kinds: StimulusInsight['kinds']): Band[] {
  return kinds.map((kind) => ({
    key: kind.key,
    label: kind.label,
    time: kind.time?.text ?? null,
    shareText: kind.share?.text ?? null,
    share: kind.share?.raw ?? null,
  }))
}

/** The watch's own zones, as the drawing reads them. */
function sourceBandsOf(zones: StimulusInsight['source_zones']): Band[] {
  return zones.map((zone) => ({
    key: `z${zone.zone}`,
    label: `Z${zone.zone}`,
    time: zone.time?.text ?? null,
    shareText: zone.share?.text ?? null,
    share: zone.share?.raw ?? null,
  }))
}

/**
 * The bars of a block.
 *
 * **The bar is the share**: the width is the band's own share of the
 * measured time, and a band with no value draws no bar at all.
 */
function BlockBars({ bands, testId }: { bands: Band[]; testId: string }) {
  if (bands.length === 0) return null
  return (
    <ul className="space-y-1.5" data-testid={testId}>
      {bands.map((band) => (
        <li key={band.key}>
          <div className="flex items-baseline justify-between gap-3">
            <span className="text-[12.5px] text-ink-soft">{band.label}</span>
            <span className="text-[12.5px] tabular-nums text-ink">
              {band.time ?? ''}
              {band.shareText ? <span className="ml-1 text-ink-muted">{band.shareText}</span> : null}
            </span>
          </div>
          <div
            className="mt-1 h-1.5 w-full overflow-hidden rounded-full"
            style={{ background: 'var(--chart-band)' }}
          >
            {band.share !== null ? (
              <div
                className="h-full rounded-full"
                style={{
                  width: `${Math.max(0, Math.min(100, band.share))}%`,
                  background: 'var(--chart-line)',
                }}
                data-testid={`${testId}-${band.key}`}
              />
            ) : null}
          </div>
        </li>
      ))}
    </ul>
  )
}

/** One cell of a block's weeks: a band's time in a week, or nothing. */
interface WeekCell {
  key: string
  label: string
  time: RenderedValue | null
}

/**
 * A block's own weeks.
 *
 * **A week that held none is a hole**: the cell is empty, and the hole
 * says what it is — a week with no time in that band, not a week with no
 * band.
 */
function BlockWeeks({
  weeks,
  cells,
  testId,
}: {
  /** The weeks of the block, in order. */
  weeks: { week_start: string; week_start_label: string }[]
  /** The cells, week by week: the bands of that week, in the order. */
  cells: WeekCell[][]
  testId: string
}) {
  if (weeks.length === 0 || cells.length === 0) return null
  const bandCount = cells[0]?.length ?? 0
  // Each band's own busiest week: the scale that makes the row readable.
  const busiest = Array.from({ length: bandCount }, (_, bandIndex) =>
    Math.max(
      0,
      ...cells.map((week) => week[bandIndex]?.time?.raw ?? 0),
    ),
  )
  return (
    <div className="mt-2" data-testid={testId}>
      <div className="flex gap-[3px]">
        <div className="flex w-[62px] shrink-0 flex-col justify-between text-[10px] text-ink-muted">
          {Array.from({ length: bandCount }, (_, bandIndex) => (
            <span key={bandIndex} className="flex h-[14px] items-center">
              {cells[0]?.[bandIndex]?.label ?? ''}
            </span>
          ))}
        </div>
        <div className="flex flex-1 gap-[3px]">
          {weeks.map((week, weekIndex) => (
            <div key={week.week_start} className="flex flex-1 flex-col justify-between">
              {Array.from({ length: bandCount }, (_, bandIndex) => {
                const cell = cells[weekIndex]?.[bandIndex]
                const seconds = cell?.time?.raw ?? null
                const scale = busiest[bandIndex] ?? 0
                const share =
                  seconds === null || scale <= 0 ? 0 : Math.max(0, Math.min(1, seconds / scale))
                return (
                  <span key={cell?.key ?? bandIndex} className="flex h-[14px] items-end">
                    <span
                      className="block w-full rounded-[1px]"
                      style={{
                        height: seconds === null ? '0%' : `${Math.max(8, share * 100)}%`,
                        background: 'var(--chart-line)',
                      }}
                      data-testid={`${testId}-${week.week_start}-${cell?.key ?? bandIndex}`}
                    />
                  </span>
                )
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

/**
 * One block of the ledger.
 *
 * **A block has a heading and a basis.** The heading says whose
 * measurement it is, and the basis says what was measured — the time the
 * athlete was running, or the whole run.
 */
function LedgerBlock({
  heading,
  basis,
  bars,
  children,
  testId,
}: {
  heading: string
  basis: string
  bars: Band[]
  children?: ReactNode
  testId: string
}) {
  return (
    <div data-testid={testId}>
      <h3 className="caps-sm">{heading}</h3>
      <p className="text-[11.5px] text-ink-faint">{basis}</p>
      {bars.length > 0 ? (
        <div className="mt-1.5">
          <BlockBars bands={bars} testId={`${testId}-bars`} />
        </div>
      ) : null}
      {children}
    </div>
  )
}

/**
 * The overlap: where both bases exist.
 *
 * **The two measurements stand side by side, and the gap between them is
 * stated.** They are never added, and neither is corrected by the other.
 */
function Overlap({ overlap }: { overlap: StimulusInsight['overlap'] }) {
  if (!overlap) return null
  return (
    <div className="mt-3 border-t border-edge pt-2" data-testid="ledger-overlap">
      <h3 className="caps-sm">{t('ledger.overlap')}</h3>
      <p className="text-[11.5px] text-ink-faint">
        {overlap.from_label} – {overlap.to_label}
      </p>
      <dl className="mt-1.5 grid grid-cols-2 gap-x-3 gap-y-1.5 sm:grid-cols-3">
        <div>
          <dt className="text-[11px] text-ink-muted">{t('ledger.block.app')}</dt>
          <dd className="text-[13px] tabular-nums text-ink" data-testid="ledger-overlap-app">
            {overlap.app_moving.text}
          </dd>
        </div>
        <div>
          <dt className="text-[11px] text-ink-muted">{t('ledger.block.source')}</dt>
          <dd className="text-[13px] tabular-nums text-ink" data-testid="ledger-overlap-source">
            {overlap.source_elapsed.text}
          </dd>
        </div>
        <div>
          <dt className="text-[11px] text-ink-muted">{t('ledger.overlap.gap')}</dt>
          <dd className="text-[13px] tabular-nums text-ink" data-testid="ledger-overlap-gap">
            {overlap.difference.text}
          </dd>
        </div>
      </dl>
    </div>
  )
}

/**
 * The ledger's card: the two blocks, and the overlap.
 *
 * **The card is the picture.** Each block is drawn on its own, with its
 * own basis, and the overlap reads underneath them.
 */
export function LedgerCard({
  stimulus,
  error,
  bare = false,
  hoist = false,
}: {
  /** The ledger's answer. */
  stimulus: StimulusInsight | undefined
  /** Whether the answer could not be read. */
  error: boolean
  /** The card is the wall's, and the heading is already said. */
  bare?: boolean
  /** Whether this screen hoists the source line itself. */
  hoist?: boolean
}) {
  const body = error && !stimulus ? (
    <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
  ) : !stimulus ? (
    <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
  ) : (
    <>
      {hoist ? (
        <SourceLine source={stimulus.source} sources={stimulus.sources} testId="ledger-source-line" />
      ) : null}
      {stimulus.unavailable ? (
        <p className="text-[13px] text-ink-muted" data-testid="ledger-unavailable">
          {stimulus.unavailable}
        </p>
      ) : null}
      {/*
        The period the ledger covers. **"The last 12 weeks"** — the
        runner's own words, and the dates behind them.
      */}
      <p className="text-[11.5px] text-ink-faint" data-testid="ledger-window">
        {t('common.window.weeks', { n: stimulus.window.weeks })} · {stimulus.window.from_label} –{' '}
        {stimulus.window.to_label}
      </p>
      {/*
        The two blocks. **They are two blocks and not one number**: the
        app's own bands, measured over the time the athlete was running,
        and the watch's own zone minutes, measured over the whole run.
      */}
      <div className="mt-1 grid gap-4 sm:grid-cols-2" data-testid="ledger-blocks">
        <LedgerBlock
          heading={t('ledger.block.app')}
          basis={t('ledger.block.app.note')}
          bars={bandsOf(stimulus.kinds)}
          testId="ledger-app"
        >
          <BlockWeeks
            weeks={stimulus.weeks.map((week) => ({
              week_start: week.week_start,
              week_start_label: week.week_start_label,
            }))}
            cells={stimulus.weeks.map((week) => week.bands)}
            testId="ledger-app-weeks"
          />
        </LedgerBlock>
        <LedgerBlock
          heading={t('ledger.block.source')}
          basis={t('ledger.block.source.note')}
          bars={sourceBandsOf(stimulus.source_zones)}
          testId="ledger-source"
        >
          <BlockWeeks
            weeks={stimulus.source_weeks.map((week) => ({
              week_start: week.week_start,
              week_start_label: week.week_start_label,
            }))}
            cells={stimulus.source_weeks.map((week) =>
              week.zones.map((time, index) => ({
                key: `z${index + 1}`,
                label: `Z${index + 1}`,
                time,
              })),
            )}
            testId="ledger-source-weeks"
          />
        </LedgerBlock>
      </div>
      <Overlap overlap={stimulus.overlap} />
      {stimulus.longest ? (
        <div className="mt-2 text-[12px]" data-testid="ledger-longest">
          <span className="text-ink-muted">{t('ledger.longest')} · </span>
          <span className="text-ink">{stimulus.longest.date_label}</span>
          {stimulus.longest.distance ? (
            <span className="text-ink-muted"> · {stimulus.longest.distance.text}</span>
          ) : null}
          <span className="text-ink-muted"> · {stimulus.longest.time.text}</span>
        </div>
      ) : null}
      {stimulus.repeated ? (
        <div className="mt-1 text-[12px]" data-testid="ledger-repeated">
          <span className="text-ink-muted">{t('ledger.repeated')} · </span>
          <span className="text-ink">{stimulus.repeated.time.text}</span>
          <span className="text-ink-muted"> · {stimulus.repeated.runs}</span>
        </div>
      ) : stimulus.repeated_missing ? (
        <div className="mt-1 text-[12px] text-ink-muted" data-testid="ledger-repeated-missing">
          {t('ledger.repeated')} · {stimulus.repeated_missing}
        </div>
      ) : null}
      {stimulus.more_runs ? (
        <p className="mt-2 text-[11.5px] text-ink-faint" data-testid="ledger-more">
          {t('ledger.more')}
        </p>
      ) : null}
    </>
  )

  if (bare) return <div data-testid="ledger-card">{body}</div>

  return (
    <section className="card px-4 py-3" data-testid="ledger-card">
      <h2 className="mb-2 caps">{t('progress.wall.time.heading')}</h2>
      {body}
    </section>
  )
}
