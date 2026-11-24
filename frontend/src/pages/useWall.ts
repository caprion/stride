/**
 * The answers the wall draws on.
 *
 * The wide Progress is a wall: every panel the router answers, on one
 * screen. These are the answers the panels that are not the rolling view
 * read — the week, the year, the pace, the recent runs, the body's own
 * night, and the three windows of the athlete profile.
 *
 * **Nothing here is computed.** Each answer is the router's, and the
 * panels draw them.
 */

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'
import type { ProfileWindow } from '../lib/types'
import { shiftDate } from '../lib/history'

/**
 * The windows the athlete profile is drawn over.
 *
 * **Thirty, sixty and ninety days.** The chart draws all three of them,
 * one on the other, and the picker says which of them the rest of the
 * page is about — the numbers under the chart, the training profile, the
 * splits and the months.
 *
 * The lengths are in days because the profile asks for them in days: a
 * thirty-day window is not four weeks, and the router answers the days
 * exactly. The window before the window is the **same length**, so the
 * comparison the table makes is between two periods of equal size.
 */
export const PROFILE_SCOPES = [
  { days: 30, labelKey: 'progress.profile.scope.30' },
  { days: 60, labelKey: 'progress.profile.scope.60' },
  { days: 90, labelKey: 'progress.profile.scope.90' },
] as const

/**
 * The scope the profile is drawn over until the athlete chooses another.
 *
 * Ninety days: the longest of the three, and the one that holds a whole
 * season of habits.
 */
export const PROFILE_SCOPE_DEFAULT = 90

/**
 * The wall's answers.
 *
 * The rolling view, the load, the progress and the streaks come from
 * `useProgress` — they follow the range the screen is showing. The rest
 * are the wall's own: the three windows the profile is drawn over are the
 * three the profile card names, whatever the range is.
 */
export function useWall(range: number) {
  /** The year: the months, the calendar of the days and what changed. */
  const year = useQuery({ queryKey: ['insights-year'], queryFn: () => api.insightsYear() })

  /** The pace: the training week by week. */
  const pace = useQuery({
    queryKey: ['insights-pace', range],
    queryFn: () => api.insightsPace(range),
  })

  /**
   * The three windows of the athlete profile.
   *
   * **Three answers, one chart.** The chart draws all of them, one on
   * the other, so the profile's picture is the whole shape — and the
   * picker's job is not to choose a window for the chart but to say which
   * window the rest of the page is about.
   *
   * The query keys carry the days, so a window of days never answers for
   * a window of weeks.
   */
  const profile30 = useQuery({
    queryKey: ['insights-rolling', 'days', 30],
    queryFn: () => api.insightsRollingDays(30),
  })
  const profile60 = useQuery({
    queryKey: ['insights-rolling', 'days', 60],
    queryFn: () => api.insightsRollingDays(60),
  })
  const profile90 = useQuery({
    queryKey: ['insights-rolling', 'days', 90],
    queryFn: () => api.insightsRollingDays(90),
  })

  /** The window the page is about: the one the athlete has chosen. */
  const [profileDays, setProfileDays] = useState<number>(PROFILE_SCOPE_DEFAULT)
  const profileScope =
    PROFILE_SCOPES.find((scope) => scope.days === profileDays) ??
    (PROFILE_SCOPES[2] as (typeof PROFILE_SCOPES)[number])

  const profileWindows: ProfileWindow[] = [
    { days: 30, rolling: profile30.data },
    { days: 60, rolling: profile60.data },
    { days: 90, rolling: profile90.data },
  ]
  const focused = profileWindows.find((window) => window.days === profileScope.days)
  const focusedQuery =
    profileScope.days === 30 ? profile30 : profileScope.days === 60 ? profile60 : profile90

  /**
   * The profile's runs.
   *
   * **The pictures are drawn from the feed.** The schedule's days, the
   * habit's day marks and the records' timeline are facts about the runs
   * themselves, and the feed already carries them. The window starts a
   * week earlier, so that a strip has the previous week's days to stand
   * beside this week's.
   */
  const profileRuns = useQuery({
    queryKey: [
      'activities',
      'profile',
      focused?.rolling?.window.from,
      focused?.rolling?.window.to,
    ],
    queryFn: () =>
      api.activities({
        since: shiftDate(focused!.rolling!.window.from, -7),
        until: focused!.rolling!.window.to,
      }),
    enabled: focused?.rolling !== undefined,
  })

  return {
    year,
    pace,
    /** The three windows the chart draws, one on the other. */
    profileWindows,
    /** The focused window's answer: the numbers the page is about. */
    profile: focused?.rolling,
    /** Whether the focused window could not be read. */
    profileError: focusedQuery.isError,
    profileScope,
    setProfileDays,
    profileRuns,
  }
}

/** What the hook returns. */
export type WallAnswers = ReturnType<typeof useWall>
