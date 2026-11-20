/**
 * The run film: the run's own timeline, and the readout that moves with it.
 *
 * The pace, the heart rate and the ground on one axis, and the athlete's
 * own zones behind them. **Faster is higher**: the pace's axis is
 * inverted, so the line rises when the runner goes faster, and the two
 * scales are told apart by their own colours.
 *
 * The drawing is the app's own — an SVG it draws itself — and every label
 * on it is the **core's own words**: "148 bpm" on the heart-rate axis,
 * "5:24 /km" on the pace's. The app prints no domain number of its own.
 *
 * The readout answers the one question the athlete has while the film
 * plays: *where am I?* — the elapsed time, the distance, the pace, the
 * heart rate, the ground, and the zone.
 */

import { useLayoutEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode, RefObject } from 'react'
import { t } from '../i18n'
import { axisTicks, robustRange } from '../lib/chart'
import { ABSENT, renderedText } from '../lib/format'
import { zoneColour } from '../lib/zones'
import type { FilmSample, RunFilm } from '../lib/types'

/** The room the drawing keeps for its axes. */
const PAD = { top: 34, right: 68, bottom: 36, left: 58 }

/** The zone ribbon, behind everything else. */
const RIBBON_TOP = 4
const RIBBON_HEIGHT = 12

/** The pace and the heart rate. */
const PLOT_TOP = 34
const PLOT_BOTTOM = 228

/** The ground, in a band of its own under the two lines. */
const GROUND_TOP = 236
const GROUND_BOTTOM = 264

/** The height of the drawing. */
const HEIGHT = 300

/** The pace's colour — the athlete's own signal. */
const PACE_COLOUR = '#D99A0B'
/** The heart rate's colour. */
const HEART_COLOUR = '#1E5E96'
/** The ground's colour. */
const GROUND_COLOUR = '#3F6E9E'

/** The seconds a sample is worth when the playhead moves. */
const SCRUB_STEP = 1

/** The film, and the playhead that moves over it. */
export function RunFilm({
  film,
  playhead,
  playing,
  onScrub,
  onToggle,
}: {
  film: RunFilm
  playhead: number
  playing: boolean
  onScrub: (seconds: number) => void
  onToggle: () => void
}) {
  const host = useRef<HTMLDivElement>(null)
  const width = useWidth(host)
  const samples = film.samples

  /** The run's own length: the last sample's elapsed time. */
  const duration = useMemo(() => {
    const last = samples[samples.length - 1]
    return last?.elapsed.raw ?? 0
  }, [samples])

  /** The sample the playhead is over. */
  const current = useMemo(() => sampleAt(samples, playhead), [samples, playhead])

  /** Which sample that is — the slider's own step. */
  const index = useMemo(() => sampleIndexAt(samples, playhead), [samples, playhead])

  /** The ground under the playhead. */
  const ground = useMemo(() => groundAt(film.ground, playhead), [film.ground, playhead])

  /** The zone the playhead is in, in the core's words. */
  const zoneName = useMemo(() => {
    if (film.zones.length === 0) return ABSENT
    if (current?.zone === null || current?.zone === undefined) {
      return film.outside_label ?? ABSENT
    }
    return film.zones[current.zone]?.label ?? ABSENT
  }, [film.zones, film.outside_label, current])

  /** The drawing. It changes when the run or the width changes. */
  const drawing = useMemo(
    () => draw(samples, film.ground, film.zones.length > 0, width, duration),
    [samples, film.ground, film.zones.length, width, duration],
  )

  /** The head of the playhead, where it is now. */
  const head = useMemo(
    () => (duration > 0 ? (playhead / duration) * (width - PAD.left - PAD.right) + PAD.left : 0),
    [playhead, duration, width],
  )

  /** Where the runner is, on the chart. */
  const headHeart = current?.heart_rate?.raw ?? null
  const headPace = current?.pace?.raw ?? null

  /** Move the playhead where the pointer is. */
  const seek = (clientX: number) => {
    const node = host.current
    if (!node || duration <= 0) return
    const box = node.getBoundingClientRect()
    const x = clientX - box.left
    const ratio = (x - PAD.left) / Math.max(1, width - PAD.left - PAD.right)
    onScrub(Math.min(duration, Math.max(0, ratio * duration)))
  }

  const dragging = useRef(false)

  return (
    <div>
      {/* ------------------------------------------------- the controls */}
      <div className="mb-2 flex flex-wrap items-center gap-3">
        <button
          type="button"
          className="tap inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700"
          onClick={onToggle}
          data-testid="film-play"
          aria-pressed={playing}
        >
          {playing ? t('film.pause') : t('film.play')}
        </button>
        <input
          type="range"
          className="min-w-[160px] flex-1 accent-brand-600"
          min={0}
          max={Math.max(1, samples.length - 1)}
          step={SCRUB_STEP}
          value={index}
          onChange={(event) => {
            const at = samples[Number(event.target.value)]
            if (at) onScrub(at.elapsed.raw ?? 0)
          }}
          aria-label={t('film.scrub')}
          data-testid="film-scrub"
        />
        <span className="text-[12px] text-ink-muted">{t('film.speed')}</span>
      </div>

      {/* ---------------------------------------------------- the chart */}
      <div
        ref={host}
        className="relative touch-none"
        style={{ cursor: 'ew-resize' }}
        data-testid="film-chart"
        onPointerDown={(event) => {
          dragging.current = true
          event.currentTarget.setPointerCapture(event.pointerId)
          seek(event.clientX)
        }}
        onPointerMove={(event) => {
          if (dragging.current) seek(event.clientX)
        }}
        onPointerUp={() => {
          dragging.current = false
        }}
        onPointerCancel={() => {
          dragging.current = false
        }}
      >
        {width > 0 ? (
          <svg
            viewBox={`0 0 ${width} ${HEIGHT}`}
            width={width}
            height={HEIGHT}
            role="img"
            aria-label={t('film.chart.alt')}
            data-testid="film-svg"
          >
            {drawing.grid}
            {drawing.ribbon}
            {drawing.clip}
            {drawing.ground}
            {drawing.lines}
            {drawing.axes}
            {width > 0 ? (
              <g data-testid="film-playhead">
                <line
                  x1={head}
                  x2={head}
                  y1={RIBBON_TOP}
                  y2={GROUND_BOTTOM}
                  stroke="#112B40"
                  strokeWidth={1.5}
                  opacity={0.75}
                />
                {headHeart !== null && drawing.yHeart ? (
                  <circle
                    cx={head}
                    cy={drawing.yHeart(headHeart)}
                    r={4.5}
                    fill={HEART_COLOUR}
                    stroke="#FFFFFF"
                    strokeWidth={2}
                  />
                ) : null}
                {headPace !== null && drawing.yPace ? (
                  <circle
                    cx={head}
                    cy={drawing.yPace(headPace)}
                    r={4.5}
                    fill={PACE_COLOUR}
                    stroke="#FFFFFF"
                    strokeWidth={2}
                  />
                ) : null}
              </g>
            ) : null}
          </svg>
        ) : null}
      </div>

      {/* --------------------------------------------------- the readout */}
      <div
        className="mt-3 grid grid-cols-2 gap-x-4 gap-y-3 sm:grid-cols-3"
        data-testid="film-readout"
      >
        <Readout label={t('film.elapsed')} value={current ? current.elapsed.text : ABSENT} />
        <Readout
          label={t('vital.distance')}
          value={current ? renderedText(current.distance) : ABSENT}
        />
        <Readout label={t('vital.pace')} value={current ? renderedText(current.pace) : ABSENT} />
        <Readout
          label={t('vital.heartRate')}
          value={current ? renderedText(current.heart_rate) : ABSENT}
        />
        <Readout label={t('film.ground')} value={ground ? ground.elevation.text : ABSENT} />
        <Readout label={t('film.now')} value={zoneName} />
      </div>

      {/* ------------------------------------------------ the zone legend */}
      {film.zones.length > 0 ? (
        <div className="mt-3" data-testid="film-zones">
          <h3 className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('film.zones.heading')}
          </h3>
          <ul className="mt-1 flex flex-wrap gap-x-4 gap-y-1">
            {film.zones.map((zone, index) => (
              <li key={zone.label} className="flex items-center gap-1.5 text-[11px] text-ink-muted">
                <span
                  className="inline-block h-2.5 w-4 rounded-sm"
                  style={{ backgroundColor: zoneColour(index) }}
                />
                <span className="font-medium text-ink-soft">{zone.label}</span>
                {zone.range ? <span>{zone.range}</span> : null}
              </li>
            ))}
            {film.outside_label ? (
              <li className="flex items-center gap-1.5 text-[11px] text-ink-muted">
                <span
                  className="inline-block h-2.5 w-4 rounded-sm"
                  style={{ backgroundColor: zoneColour(null) }}
                />
                <span>{film.outside_label}</span>
              </li>
            ) : null}
          </ul>
          {film.model?.source ? (
            <p className="mt-1 text-[11px] text-ink-muted" data-testid="film-zones-source">
              {film.model.source}
            </p>
          ) : null}
        </div>
      ) : null}
    </div>
  )
}

/** One number of the readout. */
function Readout({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">{label}</div>
      <div className="text-[17px] font-semibold tabular-nums text-ink">{value}</div>
    </div>
  )
}

/** The sample the playhead is over: the last one at or before it. */
export function sampleAt(samples: FilmSample[], seconds: number): FilmSample | null {
  const index = sampleIndexAt(samples, seconds)
  return index >= 0 ? (samples[index] as FilmSample) : (samples[0] ?? null)
}

/**
 * Which sample the playhead is over.
 *
 * The index is a count of samples — the slider's own step — and not a
 * rounding of a time: the app never rounds a domain value of its own.
 */
export function sampleIndexAt(samples: FilmSample[], seconds: number): number {
  let found = -1
  for (let index = 0; index < samples.length; index += 1) {
    const at = samples[index]?.elapsed.raw ?? 0
    if (at > seconds) break
    found = index
  }
  return found >= 0 ? found : 0
}

/** The ground under the playhead: the last point at or before it. */
function groundAt(
  ground: RunFilm['ground'],
  seconds: number,
): RunFilm['ground'][number] | null {
  let found: RunFilm['ground'][number] | null = null
  for (const point of ground) {
    if (point.offset_s > seconds) break
    found = point
  }
  return found ?? ground[0] ?? null
}

/**
 * The width of an element, measured as it changes.
 *
 * The drawing is measured before it is painted — a chart that is drawn at
 * the wrong width and corrected afterwards flickers — and the observer
 * keeps it right when the window changes.
 */
function useWidth(ref: RefObject<HTMLDivElement>): number {
  const [width, setWidth] = useState(0)
  useLayoutEffect(() => {
    const node = ref.current
    if (!node) return
    const measure = () => {
      const next = Math.round(node.getBoundingClientRect().width)
      setWidth((current) => (current === next ? current : next))
    }
    measure()
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      const next = Math.round(
        entry?.contentRect?.width ?? node.getBoundingClientRect().width,
      )
      setWidth((current) => (current === next ? current : next))
    })
    observer.observe(node)
    return () => observer.disconnect()
  }, [ref])
  return width
}

/** What the drawing holds, and the scales it draws on. */
interface Drawing {
  grid: ReactNode
  ribbon: ReactNode
  lines: ReactNode
  axes: ReactNode
  ground: ReactNode
  clip: ReactNode
  yHeart: ((value: number) => number) | null
  yPace: ((value: number) => number) | null
}

/**
 * The drawing of the run.
 *
 * It is built once per run and per width, and the playhead only moves
 * over it: the film plays smoothly because the shape of the run is not
 * redrawn sixty times a second.
 */
function draw(
  samples: FilmSample[],
  ground: RunFilm['ground'],
  hasZones: boolean,
  width: number,
  duration: number,
): Drawing {
  if (width <= 0 || duration <= 0 || samples.length === 0) {
    return {
      grid: null,
      ribbon: null,
      lines: null,
      axes: null,
      ground: null,
      clip: null,
      yHeart: null,
      yPace: null,
    }
  }

  const plotWidth = width - PAD.left - PAD.right
  const x = (seconds: number) => PAD.left + (seconds / duration) * plotWidth

  const heartRates = samples
    .map((sample) => sample.heart_rate?.raw)
    .filter((value): value is number => value !== null && value !== undefined)
  const paces = samples
    .map((sample) => sample.pace?.raw)
    .filter((value): value is number => value !== null && value !== undefined)

  /**
   * The two scales.
   *
   * They cover the middle of the readings: a walk break's pace of nineteen
   * minutes a kilometre would otherwise squash the whole series into the
   * top of the chart, and the readings outside the scale are drawn past
   * the edge of the drawing, which is what a chart does with them.
   */
  const heartRange = robustRange(heartRates)
  const paceRange = robustRange(paces)

  const hrMin = heartRange[0] - 4
  const hrMax = heartRange[1] + 4
  const yHeart = (value: number) =>
    PLOT_BOTTOM - ((value - hrMin) / (hrMax - hrMin || 1)) * (PLOT_BOTTOM - PLOT_TOP)

  // The pace's axis is inverted: the fastest pace is at the top, so the
  // line rises when the runner goes faster.
  const paceFast = paceRange[0] - 4
  const paceSlow = paceRange[1] + 4
  const yPace = (value: number) =>
    PLOT_BOTTOM - ((paceSlow - value) / (paceSlow - paceFast || 1)) * (PLOT_BOTTOM - PLOT_TOP)

  // The minute marks along the bottom: the chart's own scale.
  const minutes = Math.max(1, Math.ceil(duration / 60))
  const step = minutes <= 4 ? 1 : minutes <= 12 ? 2 : minutes <= 40 ? 5 : 15
  const ticks: ReactNode[] = []
  for (let minute = 0; minute <= minutes; minute += step) {
    const at = x(minute * 60)
    ticks.push(
      <line
        key={`grid-${minute}`}
        x1={at}
        x2={at}
        y1={PLOT_TOP}
        y2={GROUND_BOTTOM}
        stroke="#E4EBF0"
      />,
      <text
        key={`label-${minute}`}
        x={at}
        y={HEIGHT - 10}
        textAnchor="middle"
        fill="#61788D"
        fontSize={11}
      >
        {minute} min
      </text>,
    )
  }

  // The axes: the core's own words for the numbers.
  //
  // Only the readings the scale covers get a label: a reading outside it
  // is drawn past the edge of the drawing, and a label for it would be a
  // label for something the reader cannot see.
  const heartTicks = axisTicks(
    samples
      .filter((sample) => {
        const value = sample.heart_rate?.raw
        return (
          value !== null &&
          value !== undefined &&
          value >= heartRange[0] &&
          value <= heartRange[1]
        )
      })
      .map((sample) => ({
        value: sample.heart_rate?.raw ?? null,
        text: sample.heart_rate?.text ?? null,
      })),
  )
  const paceTicks = axisTicks(
    samples
      .filter((sample) => {
        const value = sample.pace?.raw
        return (
          value !== null && value !== undefined && value >= paceRange[0] && value <= paceRange[1]
        )
      })
      .map((sample) => ({ value: sample.pace?.raw ?? null, text: sample.pace?.text ?? null })),
  )
  const leftLabels = heartTicks.map((tick) => (
    <text
      key={`hr-${tick.value}`}
      x={PAD.left - 8}
      y={yHeart(tick.value) + 4}
      textAnchor="end"
      fill="#61788D"
      fontSize={11}
      data-testid="chart-axis-label"
    >
      {tick.text}
    </text>
  ))
  const rightLabels = paceTicks.map((tick) => (
    <text
      key={`pace-${tick.value}`}
      x={width - PAD.right + 8}
      y={yPace(tick.value) + 4}
      textAnchor="start"
      fill="#61788D"
      fontSize={11}
      data-testid="chart-axis-label"
    >
      {tick.text}
    </text>
  ))

  // The two axes' names, and whether they fit side by side.
  const titles = axisTitles(width)

  // The zone ribbon: one segment for each run of samples in the same zone.
  const ribbon: ReactNode[] = []
  if (hasZones) {
    let start = 0
    for (let index = 1; index <= samples.length; index += 1) {
      const zone = index < samples.length ? samples[index]?.zone ?? null : null
      const first = samples[start]
      const previous = first?.zone ?? null
      if (index === samples.length || zone !== previous) {
        const from = x(first?.elapsed.raw ?? 0)
        const to = x(samples[index - 1]?.elapsed.raw ?? 0)
        ribbon.push(
          <rect
            key={`zone-${start}`}
            x={from}
            y={RIBBON_TOP}
            width={Math.max(1, to - from)}
            height={RIBBON_HEIGHT}
            fill={zoneColour(previous)}
          />,
        )
        start = index
      }
    }
  }

  // The two lines.
  const heartPoints = samples
    .filter((sample) => sample.heart_rate?.raw !== null && sample.heart_rate?.raw !== undefined)
    .map((sample) => `${px(x(sample.elapsed.raw ?? 0))},${px(yHeart(sample.heart_rate?.raw ?? 0))}`)
    .join(' ')
  const pacePoints = samples
    .filter((sample) => sample.pace?.raw !== null && sample.pace?.raw !== undefined)
    .map((sample) => `${px(x(sample.elapsed.raw ?? 0))},${px(yPace(sample.pace?.raw ?? 0))}`)
    .join(' ')

  // The ground: the same altitude on the run's own time axis.
  const lowest = groundMin(ground)
  const highest = groundMax(ground)
  const groundPoints = ground.map((point) => {
    const y =
      GROUND_BOTTOM -
      (((point.elevation.raw ?? 0) - lowest) / (highest - lowest || 1)) *
        (GROUND_BOTTOM - GROUND_TOP)
    return `${px(x(point.offset_s))},${px(y)}`
  })

  return {
    grid: <g>{ticks}</g>,
    ribbon: <g>{ribbon}</g>,
    clip: (
      <clipPath id="film-plot">
        <rect x={PAD.left} y={PLOT_TOP} width={plotWidth} height={PLOT_BOTTOM - PLOT_TOP} />
      </clipPath>
    ),
    ground:
      groundPoints.length > 1 ? (
        <g>
          <polygon
            points={`${PAD.left},${GROUND_BOTTOM} ${groundPoints.join(' ')} ${width - PAD.right},${GROUND_BOTTOM}`}
            fill={GROUND_COLOUR}
            opacity={0.22}
          />
          <polyline points={groundPoints.join(' ')} fill="none" stroke={GROUND_COLOUR} strokeWidth={1.5} />
        </g>
      ) : null,
    lines: (
      <g clipPath="url(#film-plot)">
        <polyline points={heartPoints} fill="none" stroke={HEART_COLOUR} strokeWidth={2} />
        <polyline points={pacePoints} fill="none" stroke={PACE_COLOUR} strokeWidth={2.25} />
      </g>
    ),
    axes: (
      <g>
        {leftLabels}
        {rightLabels}
        {/*
          The two axes' names. The pace's carries the reading of the
          chart — "faster is higher" — so it is the one that stays when
          the drawing is too narrow for both, and the heart rate's own
          axis says "bpm" anyway.
        */}
        <text x={PAD.left + 4} y={PLOT_TOP - 8} fill={PACE_COLOUR} fontSize={11} fontWeight={700}>
          {titles.pace}
        </text>
        {titles.heart ? (
          <text
            x={width - PAD.right - 4}
            y={PLOT_TOP - 8}
            textAnchor="end"
            fill={HEART_COLOUR}
            fontSize={11}
            fontWeight={700}
          >
            {titles.heart}
          </text>
        ) : null}
      </g>
    ),
    yHeart: heartRates.length > 0 ? yHeart : null,
    yPace: paces.length > 0 ? yPace : null,
  }
}

/**
 * A pixel on the drawing.
 *
 * The drawing's own geometry: a coordinate, to a tenth of a pixel, so the
 * path of the run is not a string of seventeen decimals.
 */
function px(value: number): number {
  return Math.round(value * 10) / 10
}

/**
 * The two axes' names, and whether they fit side by side.
 *
 * The pace's name carries the reading of the chart — *faster is higher* —
 * so it is the one that stays when the drawing is narrow, and the heart
 * rate's own axis says "bpm" anyway.
 */
function axisTitles(width: number): { pace: string; heart: string | null } {
  const pace = t('film.axis.pace')
  const heart = t('film.axis.heartRate')
  const room = width - PAD.left - PAD.right - 8
  if (textWidth(pace) + textWidth(heart) + 16 <= room) {
    return { pace, heart }
  }
  return { pace, heart: null }
}

/**
 * The width of a piece of text, as the drawing will set it.
 *
 * The drawing measures its own labels: a canvas knows how wide a string
 * is, and the drawing must not put two labels on top of each other.
 */
function textWidth(text: string): number {
  const context = measuring()
  if (!context) return text.length * 6
  context.font = `700 11px ${fontFamily()}`
  return context.measureText(text).width
}

/** The canvas the drawing measures its labels with. */
let measuringContext: CanvasRenderingContext2D | null | undefined

function measuring(): CanvasRenderingContext2D | null {
  if (measuringContext === undefined) {
    measuringContext =
      typeof document === 'undefined' ? null : document.createElement('canvas').getContext('2d')
  }
  return measuringContext
}

/** The app's own font, as the labels are set in it. */
function fontFamily(): string {
  if (typeof document === 'undefined') return 'system-ui, sans-serif'
  return getComputedStyle(document.body).fontFamily || 'system-ui, sans-serif'
}

/** The lowest point of the ground. */
function groundMin(ground: RunFilm['ground']): number {
  const values = ground
    .map((point) => point.elevation.raw ?? 0)
    .filter((value) => Number.isFinite(value))
  return values.length > 0 ? Math.min(...values) : 0
}

/** The highest point of the ground. */
function groundMax(ground: RunFilm['ground']): number {
  const values = ground
    .map((point) => point.elevation.raw ?? 0)
    .filter((value) => Number.isFinite(value))
  return values.length > 0 ? Math.max(...values) : 0
}
