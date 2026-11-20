/**
 * The year as a calendar: every day of the window, coloured by how much
 * of that day was in the high zones.
 *
 * **Days, never places.** The grid carries the zone mix of the days and
 * nothing else: no position, no place name, nothing that says where the
 * athlete ran. Where the athlete ran is the **heatmap's** surface, and it
 * has its own screen, its own disclosure and its own rules. This one
 * says *when* and *how hard*.
 *
 * The drawing is the app's own: a rectangle per day, in an SVG it draws
 * itself. There is no tile server, no geocoder and no third-party service
 * in the path that draws it — the same rule the heatmap follows.
 *
 * The colour is the **share of the day's time in the zones that was in
 * the high zones** (Z4–Z5), and the steps are the app's own: azure for
 * the quiet days, amber for the hard ones. Nothing here is red, because
 * nothing here is a problem.
 */

import { formatDateMedium } from '../lib/format'
import { t } from '../i18n'
import type { YearDay } from '../lib/types'

/** The day with no mix: no run, or a run whose file carries no heart rate. */
const NO_MIX = '#E9EFF4'

/**
 * The steps of the colouring, from the quietest day to the hardest.
 *
 * Four steps and the neutral one: the share of the day that was in the
 * high zones, banded the way the mock bands it — under 15 %, 15–35 %,
 * 35–60 % and 60 % and over.
 */
const STEPS = ['#D6E6F3', '#7BA8CD', '#FBDB93', '#F2B01E']

/** The size of a day in the drawing's own units. */
const CELL = 0.86

/**
 * The colour of a day, by the share of it that was in the high zones.
 *
 * `null` is the day's own absence: the day has no mix at all, and it is
 * drawn in the neutral colour — never as a day with nothing in the high
 * zones, which is a different thing.
 */
export function dayColour(high: number | null): string {
  if (high === null) return NO_MIX
  if (high < 15) return STEPS[0] as string
  if (high < 35) return STEPS[1] as string
  if (high < 60) return STEPS[2] as string
  return STEPS[3] as string
}

/** The weekday of a day, Monday first: 0 is Monday. */
function weekdayOf(iso: string): number {
  const date = new Date(`${iso}T00:00:00`)
  if (Number.isNaN(date.getTime())) return 0
  return (date.getDay() + 6) % 7
}

export function YearCalendar({ days }: { days: YearDay[] }) {
  if (days.length === 0) return null

  // The first day's weekday decides where the grid starts: the weeks are
  // the athlete's own, Monday through Sunday.
  const offset = weekdayOf(days[0]?.date ?? '')
  const columns = Math.ceil((offset + days.length) / 7)

  return (
    <svg
      viewBox={`0 0 ${columns} 7`}
      preserveAspectRatio="xMinYMin meet"
      className="block h-auto w-full"
      role="img"
      aria-label={t('year.calendar.alt')}
      data-testid="year-calendar"
    >
      {days.map((day, index) => {
        const column = Math.floor((offset + index) / 7)
        const row = (offset + index) % 7
        const high = day.high?.raw ?? null
        return (
          <rect
            key={day.date}
            x={column}
            y={row}
            width={CELL}
            height={CELL}
            rx={0.12}
            fill={dayColour(high)}
            data-testid="year-calendar-day"
            data-date={day.date}
            data-high={day.high?.raw ?? ''}
          >
            <title>
              {t('year.day.title', {
                date: formatDateMedium(day.date),
                distance: day.distance?.text ?? t('year.day.none'),
                high: day.high?.text ?? t('year.day.noMix'),
              })}
            </title>
          </rect>
        )
      })}
    </svg>
  )
}
