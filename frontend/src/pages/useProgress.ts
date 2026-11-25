/**
 * The answers Progress draws on.
 *
 * One hook, so that the dashboard and the narrow layout ask the same
 * questions of the same routes in the same window: the progress answer,
 * the load, the rolling view and the streaks.
 *
 * The window is the **range** the screen is showing, and every answer
 * follows it: the api is asked for that many weeks, and the weeks the
 * charts draw are the calendar weeks of that window.
 */

import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'
import { axisTicks, calendarWeekStarts } from '../lib/chart'
import { formatDateShort } from '../lib/format'
import { shiftDate } from '../lib/history'
import type { ConsistencyPoint, PacePoint, VolumePoint } from './progressPanels'

/**
 * The two windows the athlete's shape is drawn over.
 *
 * **The router's window is measured in weeks**, so the short window is
 * four weeks — twenty-eight days — and the longer one thirteen — ninety-one
 * days: the nearest windows to a month and to a quarter. The panel names
 * the window the answer reports, so the reader is never told a period the
 * router did not answer.
 *
 * The shape's own window is chosen on the profile's scope control, and
 * the two shapes on the chart are the window and the window before it of
 * the same length — see `PROFILE_SCOPES` in `useWall`.
 */
export const SHORT_WINDOW_WEEKS = 4
export const QUARTER_WINDOW_WEEKS = 13

/** The answers, and the series the charts draw from them. */
export function useProgress(range: number) {
  const progress = useQuery({
    queryKey: ['insights-progress', range],
    queryFn: () => api.insightsProgress(range),
  })
  // The load has a route of its own, and it follows the same range: the
  // number is never read apart from the control that changes it.
  const load = useQuery({
    queryKey: ['insights-load', range],
    queryFn: () => api.insightsLoad(range),
  })
  // The rolling view follows the same range for the same reason.
  const rolling = useQuery({
    queryKey: ['insights-rolling', range],
    queryFn: () => api.insightsRolling(range),
  })
  // The streaks: a count and the days it covers.
  const streaks = useQuery({
    queryKey: ['insights-streaks'],
    queryFn: () => api.insightsStreaks(),
  })
  /**
   * This week, against the same days of the week before.
   *
   * **The week is not a range.** The screen's range is the window the
   * charts are drawn over; this week is the week the athlete is in, and
   * it does not follow the range.
   */
  const today = useQuery({ queryKey: ['insights-today'], queryFn: () => api.insightsToday() })

  /**
   * The window's runs.
   *
   * **The pictures are drawn from the feed.** The week strip, the
   * records' timeline and the habit's day marks are all facts about the
   * runs themselves — the date and the distance — and the feed already
   * carries them. The window starts a week earlier, so that the strip
   * has the previous week's days to stand beside this week's.
   */
  const windowRuns = useQuery({
    queryKey: ['activities', 'window', rolling.data?.window.from, rolling.data?.window.to],
    queryFn: () =>
      api.activities({
        since: shiftDate(rolling.data!.window.from, -7),
        until: rolling.data!.window.to,
      }),
    enabled: rolling.data !== undefined,
  })
  // The challenges: the athlete's own, against the same window.
  const challenges = useQuery({
    queryKey: ['insights-challenges', range],
    queryFn: () => api.insightsChallenges(range),
  })
  /**
   * The ledger: the athlete's own bands, the shape rows, and the two
   * bases the time was measured on.
   *
   * It follows the range for the same reason the rest of the screen
   * does: the period the ledger covers is named with it.
   */
  const stimulus = useQuery({
    queryKey: ['insights-stimulus', range],
    queryFn: () => api.insightsStimulus(range),
  })
  /**
   * The shape's two windows.
   *
   * The same route as the rolling view, asked for the two windows the
   * shape is drawn over. They are windows of their own: the panel names
   * each one, and the two shapes are read against each other.
   */
  const shapeShort = useQuery({
    queryKey: ['insights-rolling', SHORT_WINDOW_WEEKS],
    queryFn: () => api.insightsRolling(SHORT_WINDOW_WEEKS),
  })
  const shapeLong = useQuery({
    queryKey: ['insights-rolling', QUARTER_WINDOW_WEEKS],
    queryFn: () => api.insightsRolling(QUARTER_WINDOW_WEEKS),
  })

  const data = progress.data

  /**
   * The weeks of the window.
   *
   * The chart windows use calendar weeks and preserve missing
   * observations rather than compressing gaps or substituting zero, so a
   * week with nothing in it is a gap in the line.
   */
  const weeks = data ? calendarWeekStarts(data.date, range) : []

  const volume: VolumePoint[] = weeks.map((date) => {
    const week = data?.weekly_volume.find((entry) => entry.week_start === date)
    return {
      label: formatDateShort(date),
      weekStart: date,
      km: week?.distance.raw == null ? null : week.distance.raw / 1000,
      distanceText: week?.distance.text ?? null,
    }
  })

  const pace: PacePoint[] = weeks.map((date) => {
    const week = data?.pace_at_hr.find((entry) => entry.week_start === date)
    return {
      label: formatDateShort(date),
      paceValue: week?.pace?.raw ?? null,
      paceText: week?.pace?.text ?? null,
      hrValue: week?.heart_rate?.raw ?? null,
      hrText: week?.heart_rate?.text ?? null,
    }
  })

  const consistency: ConsistencyPoint[] = weeks.map((date) => ({
    label: formatDateShort(date),
    sessions: data?.consistency.find((entry) => entry.week_start === date)?.sessions ?? null,
  }))

  /**
   * The axes' labels.
   *
   * The volume's and the pace's are the core's own words — "38.6 km",
   * "5:12 /km" — so every label carries its unit. The sessions are a
   * count the api sends as a plain number, so their unit is the app's.
   */
  const volumeTicks = axisTicks(
    volume.map((week) => ({ value: week.km, text: week.distanceText })),
  )
  const paceTicks = axisTicks(
    pace.map((week) => ({ value: week.paceValue ?? null, text: week.paceText ?? null })),
  )

  return {
    progress,
    load,
    rolling,
    streaks,
    challenges,
    stimulus,
    today,
    windowRuns,
    shape: { short: shapeShort, long: shapeLong },
    data,
    weeks,
    volume,
    pace,
    consistency,
    volumeTicks,
    paceTicks,
  }
}

/** What the hook returns. */
export type ProgressAnswers = ReturnType<typeof useProgress>
