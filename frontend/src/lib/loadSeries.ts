/**
 * The load's series, as the charts draw them.
 *
 * **The numbers were already there.** The router's answer carries the
 * day-by-day series — the acute load, the chronic load and the form at
 * every day of the window — and the seven days the recovery scenario
 * projects forward. The panel showed the *last* value of each and
 * nothing else, so the shape of the training was on the wire and not on
 * the screen.
 *
 * This file holds the three preparations, apart from the drawing:
 *
 * - the **series points** — the three lines;
 * - the **day bars** — the calendar the days' loads sit on;
 * - the **projection points** — the form the scenario expects.
 *
 * ## The days keep their places
 *
 * **A day the record does not cover is a gap, not a missing day.** The
 * bars are laid on the calendar, so two runs a week apart are a week
 * apart on the chart. Compressing the gaps would put them next to each
 * other and claim they were consecutive.
 *
 * **A day with no load is a day with no bar, and a day with a load of
 * zero is a day with no bar too** — one is the absence of a value and the
 * other is the value zero. The caption says which is which, because the
 * two look the same and are not.
 *
 * ## The scale always holds the zero
 *
 * The form's zero is the line the scenario talks about, so the scale
 * reaches it: a chart whose scale stopped above the zero could not say
 * whether the form had turned.
 */

import { eachDayOfInterval, format, parseISO } from 'date-fns'
import { formatDayMonth } from './format'
import type { LoadInsight } from './types'

/**
 * How many labels a chart's horizontal axis carries.
 *
 * **Five would run into each other in a card.** The load's card is a
 * column of the wall's grid on a wide screen and a card on a phone, and
 * a date is half a word long: four of them fit across, with the first
 * and the last anchored to the ends of the chart.
 */
export const AXIS_LABELS = 4

/** The states of the load's series. */
type LoadStates = NonNullable<LoadInsight['load']>['states']

/** The days of the load's series. */
type LoadDays = NonNullable<LoadInsight['load']>['days']

/** The seven days the scenario projects. */
type LoadProjection = NonNullable<LoadInsight['scenario']>['projection']

/** One point of the load's series: a day, and the three numbers on it. */
export interface LoadSeriesPoint {
  /**
   * The label the horizontal axis shows: "7 Sep", or nothing on a day
   * the axis skips.
   */
  label: string
  /** The acute load: the seven-day weighted average. */
  acute: number | null
  /** The chronic load: the forty-two-day weighted average. */
  chronic: number | null
  /** The form: the chronic load minus the acute one. */
  form: number | null
  /** The core's own words for the acute load. */
  acuteText: string
  /** The core's own words for the chronic load. */
  chronicText: string
  /** The core's own words for the form. */
  formText: string
}

/** One bar: a day of the window and the load on it. */
export interface LoadDayBar {
  /** The label the horizontal axis shows, or nothing on a day it skips. */
  label: string
  /** The day's load, or null when the day has no load at all. */
  load: number | null
  /** The core's own words for the day's load. */
  loadText: string | null
}

/** One point of the projection: a day of the projection and the form. */
export interface LoadProjectionPoint {
  /** The day of the projection: one is the day after the series ends. */
  label: string
  /** The projected form. */
  form: number | null
  /** The core's own words for it. */
  formText: string
}

/**
 * The indices the horizontal axis labels.
 *
 * The days keep their places; only the *labels* are thinned. A chart
 * with a label on every one of seventy-eight days has no labels at all —
 * and a chart with too many has them running into each other.
 *
 * The first and the last are always among them: the ends of the window
 * are the two days the reader is most likely to want, and the tick
 * anchors them to the ends of the chart so neither is cut.
 */
export function labelIndices(count: number): number[] {
  if (count <= 0) return []
  if (count === 1) return [0]
  const labels = Math.min(AXIS_LABELS, count)
  const step = (count - 1) / (labels - 1)
  const indices: number[] = []
  for (let index = 0; index < labels; index += 1) {
    indices.push(Math.round(index * step))
  }
  return [...new Set(indices)]
}

/**
 * A domain that holds the zero.
 *
 * The form's zero is the line the scenario talks about, so the scale
 * always reaches it: a chart whose scale stopped above the zero could
 * not say whether the form had turned.
 */
export function domainWithZero(values: (number | null)[]): [number, number] {
  const numbers = values.filter(
    (value): value is number => value !== null && Number.isFinite(value),
  )
  const low = Math.min(0, ...numbers)
  const high = Math.max(0, ...numbers)
  if (high > low) return [low, high]
  return [low - 1, high + 1]
}

/**
 * The three lines, day by day.
 *
 * The numbers are the core's own, raw and rendered; the chart draws the
 * raw ones and the tooltip shows the rendered ones.
 */
export function loadSeriesPoints(states: LoadStates): LoadSeriesPoint[] {
  const labelled = new Set(labelIndices(states.length))
  return states.map((state, index) => ({
    label: labelled.has(index) ? formatDayMonth(state.date) : '',
    acute: state.acute.raw,
    chronic: state.chronic.raw,
    form: state.form.raw,
    acuteText: state.acute.text,
    chronicText: state.chronic.text,
    formText: state.form.text,
  }))
}

/**
 * The days' loads, laid on the calendar of the window.
 *
 * The bars are one per calendar day from `from` to `to`, so the days the
 * record does not cover are gaps. A day the record covers but that has
 * no load carries no bar either — and its load is null, not zero.
 */
export function loadDayBars(days: LoadDays, from: string, to: string): LoadDayBar[] {
  const start = parseISO(from)
  const end = parseISO(to)
  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) return []

  const byDate = new Map(days.map((day) => [day.date, day]))
  const calendar = eachDayOfInterval({ start, end })
  const labelled = new Set(labelIndices(calendar.length))
  return calendar.map((date, index) => {
    const iso = format(date, 'yyyy-MM-dd')
    const day = byDate.get(iso)
    return {
      label: labelled.has(index) ? formatDayMonth(iso) : '',
      load: day?.load?.raw ?? null,
      loadText: day?.load?.text ?? null,
    }
  })
}

/**
 * The form the scenario projects, one point per projected day.
 *
 * The projection carries day numbers and not dates: the core says one is
 * the day after the series ends, and the panel says so too.
 */
export function loadProjectionPoints(projection: LoadProjection): LoadProjectionPoint[] {
  return projection.map((day) => ({
    label: String(day.day),
    form: day.form.raw,
    formText: day.form.text,
  }))
}
