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

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'

/**
 * The windows the athlete profile is drawn over.
 *
 * **The comparison is the window and the window before it, of the same
 * length.** A window of three months against a window of six is not a
 * comparison: it is two different periods drawn on one chart, and the
 * shorter one always looks smaller. The router's own answer already
 * carries the equal-length window before it, so the scope is a choice of
 * length and nothing more.
 *
 * The lengths are in months because that is how the runner thinks of
 * them, and in weeks because the router measures in weeks.
 */
export const PROFILE_SCOPES = [
  { months: 3, weeks: 13 },
  { months: 6, weeks: 26 },
  { months: 12, weeks: 52 },
] as const

/**
 * The scope the profile is drawn over until the athlete chooses another.
 *
 * Three months: the most recent third of a year, and the shortest of
 * the three.
 */
export const PROFILE_SCOPE_DEFAULT = 3

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

  /**
   * The athlete profile's window, and the window before it.
   *
   * **One answer.** The router's answer carries the window and the
   * equal-length window before it, so asking it once gives both — and
   * asking it twice, at two different lengths, is what made the two
   * shapes on the chart incomparable.
   */
  const [profileMonths, setProfileMonths] = useState<number>(PROFILE_SCOPE_DEFAULT)
  const profileScope =
    PROFILE_SCOPES.find((scope) => scope.months === profileMonths) ??
    (PROFILE_SCOPES[1] as (typeof PROFILE_SCOPES)[number])
  const profile = useQuery({
    queryKey: ['insights-rolling', profileScope.weeks],
    queryFn: () => api.insightsRolling(profileScope.weeks),
  })

  /** Last night, as the body reads it. */
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: () => api.vitals() })

  return {
    today,
    year,
    pace,
    activities,
    profile,
    profileScope,
    setProfileMonths,
    vitals,
  }
}

/** What the hook returns. */
export type WallAnswers = ReturnType<typeof useWall>
