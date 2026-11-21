/**
 * The range: the window the screen is showing.
 *
 * The control and the numbers it changes are the same thing: a number is
 * never read apart from the window it covers, so the window's control
 * sits with the screen that shows it. The api sends the weeks it has and
 * the app draws the window the athlete asks for; a window with less
 * behind it shows less and never invents a week.
 */

import { t } from '../i18n'

/**
 * The ranges the charts can show.
 *
 * **The range is the chart's own business** — the api sends the weeks it
 * has, and the app draws the window the athlete asks for. A chart with
 * less behind it shows less; it never invents a week.
 */
export const RANGES = [4, 8, 12, 26, 52] as const

export function RangeSelector({
  range,
  onChange,
  testId = 'progress-range',
}: {
  range: number
  onChange: (weeks: number) => void
  /** The test id of the control, and the prefix of its buttons. */
  testId?: string
}) {
  return (
    <div className="mt-3 flex flex-wrap items-center gap-2" data-testid={testId}>
      <span className="caps">
        {t('progress.range.label')}
      </span>
      <div className="flex flex-wrap gap-1">
        {RANGES.map((weeks) => (
          <button
            key={weeks}
            type="button"
            onClick={() => {
              onChange(weeks)
            }}
            aria-pressed={range === weeks}
            data-testid={`${testId}-${weeks}`}
            className={`tap rounded-md px-2.5 text-[12.5px] font-semibold ${
              range === weeks ? 'bg-brand-50 text-brand-700' : 'text-ink-muted'
            }`}
          >
            {t('progress.range.weeks', { n: weeks })}
          </button>
        ))}
      </div>
    </div>
  )
}
