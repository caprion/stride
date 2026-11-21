/**
 * The answers the wall draws on.
 *
 * The wide Progress is a wall: every panel the router answers, on one
 * screen. These are the answers the panels that are not the rolling view
 * read — the week, the year, the pace, the recent runs, the body's own
 * night, and the two windows of the athlete profile.
 *
 * **Nothing here is computed.** Each answer is the router's, and the
 * panels draw them.
 */

import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'

/**
 * The two windows of the athlete profile.
 *
 * The router's window is measured in weeks — seven days to a week — so a
 * month is four weeks and a quarter is thirteen. The card names each
 * window with the answer's **own** count of days, so the number on the
 * card is the router's and not the app's.
 */
export const RADAR_SHORT_WEEKS = 4
export const RADAR_LONG_WEEKS = 13

/** How many runs the wall's table shows. */
const RECENT_ACTIVITIES = 8

/**
 * The wall's answers.
 *
 * The rolling view, the load, the progress and the streaks come from
 * `useProgress` — they follow the range the screen is showing. The rest
 * are the wall's own: the windows the profile is drawn over are the two
 * the profile card names, whatever the range is.
 */
export function useWall(range: number) {
  /** This week, against the same days of the week before. */
  const today = useQuery({ queryKey: ['insights-today'], queryFn: () => api.insightsToday() })

  /** The year: the months, the calendar of the days and what changed. */
  const year = useQuery({ queryKey: ['insights-year'], queryFn: () => api.insightsYear() })

  /** The pace: the training week by week. */
  const pace = useQuery({
    queryKey: ['insights-pace', range],
    queryFn: () => api.insightsPace(range),
  })

  /** The most recent runs, for the wall's table. */
  const activities = useQuery({
    queryKey: ['activities', 'recent'],
    queryFn: () => api.activities({ limit: RECENT_ACTIVITIES }),
  })

  /** The short window of the athlete profile. */
  const shortWindow = useQuery({
    queryKey: ['insights-rolling', RADAR_SHORT_WEEKS],
    queryFn: () => api.insightsRolling(RADAR_SHORT_WEEKS),
  })

  /** The long window of the athlete profile. */
  const longWindow = useQuery({
    queryKey: ['insights-rolling', RADAR_LONG_WEEKS],
    queryFn: () => api.insightsRolling(RADAR_LONG_WEEKS),
  })

  /** Last night, as the body reads it. */
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: () => api.vitals() })

  return { today, year, pace, activities, shortWindow, longWindow, vitals }
}

/** What the hook returns. */
export type WallAnswers = ReturnType<typeof useWall>
