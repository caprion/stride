/**
 * The year, as a film.
 *
 * Twelve months, each with its own character: the distance, the runs, the
 * longest run, the form at the month's end, the shape of the weeks inside
 * it and the mix of the zones it was trained in. And the year as a
 * **calendar of the days**, coloured by how much of that day was in the
 * high zones. At the end, **what changed**: the diff against the
 * athlete's own past, in facts.
 *
 * The numbers are the core's and the sentences are the core's: the answer
 * arrives with its values rendered and its diff written, and the app
 * shows them as they stand.
 *
 * **The calendar says when and how hard, and never where.** There is no
 * position and no place name in it. Where the athlete ran is the
 * heatmap's surface, and it has its own screen and its own disclosure.
 *
 * The page is reached from Progress, and it is a page and not a
 * destination: the five destinations stay the five.
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { formatCount, renderedText } from '../lib/format'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'
import { YearCalendar } from '../components/YearCalendar'
import type { YearMonth } from '../lib/types'

/**
 * The colours of the five zones, in the app's own palette.
 *
 * Azure for the low zones and amber for the high ones: the strip says
 * where the time went, and nothing here is red, because nothing here is
 * a problem.
 */
const ZONE_COLOURS: Record<string, string> = {
  Z1: 'var(--chart-band)',
  Z2: 'var(--brand-300)',
  Z3: 'var(--brand-500)',
  Z4: 'var(--accent-300)',
  Z5: 'var(--accent-600)',
}

/**
 * One month of the year.
 *
 * The month's own facts, the shape of the weeks it touches and the mix of
 * the zones it was trained in. A month with no run has no numbers: the
 * card shows the em dash, never a zero.
 */
function MonthCard({ month }: { month: YearMonth }) {
  const weeks = month.weeks
  const longest = Math.max(...weeks.map((week) => week.distance?.raw ?? 0), 1)

  return (
    <section className="card py-3" data-testid="year-month" data-month={month.key}>
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="text-[15px] font-semibold leading-tight text-ink">{month.label}</h3>
        <span className="text-[12px] text-ink-faint">{month.key.slice(0, 4)}</span>
      </div>

      <dl className="mt-2 grid grid-cols-2 gap-x-3 gap-y-2 sm:grid-cols-4">
        {[
          { key: 'distance', label: t('vital.distance'), value: renderedText(month.distance) },
          { key: 'runs', label: t('year.month.runs'), value: formatCount(month.sessions) },
          { key: 'longest', label: t('year.month.longest'), value: renderedText(month.longest) },
          { key: 'form', label: t('year.month.form'), value: renderedText(month.form) },
        ].map((stat) => (
          <div key={stat.key}>
            <dt className="text-[11.5px] text-ink-muted">{stat.label}</dt>
            <dd
              className="text-[17px] font-semibold tabular-nums text-ink"
              data-testid={`year-month-${month.key}-${stat.key}`}
            >
              {stat.value}
            </dd>
          </div>
        ))}
      </dl>

      {/*
        The shape of the weeks the month touches. A week with nothing in
        it keeps its place in the shape and holds no distance: a gap in
        the line, not a failure.
      */}
      <div
        className="mt-3 flex h-11 items-end gap-1"
        data-testid="year-month-weeks"
        aria-label={t('year.month.weeks')}
      >
        {weeks.map((week) => {
          const distance = week.distance?.raw ?? 0
          const height = Math.max(3, (distance / longest) * 100)
          return (
            <span
              key={week.start}
              className={`block flex-1 rounded-t-[3px] ${
                distance > 0 ? 'bg-brand-400' : 'bg-surface-sunken'
              }`}
              style={{ height: `${height}%` }}
              data-testid="year-month-week"
              data-week={week.start}
              title={t('year.week.title', {
                week: week.start_label,
                distance: week.distance?.text ?? t('year.day.none'),
              })}
            />
          )
        })}
      </div>

      {/*
        The mix of the zones the month was trained in. The widths are the
        shares the core sent; the strip is a drawing, not a second
        rendering of the numbers.
      */}
      <div className="mt-3 flex items-baseline justify-between gap-2">
        <span className="text-[11.5px] text-ink-faint">{t('year.month.mix')}</span>
        <span className="text-[11.5px] text-ink-muted" data-testid="year-month-high">
          {month.high ? t('year.month.high', { share: month.high.text }) : t('year.day.noMix')}
        </span>
      </div>
      <div className="mt-1 flex h-2.5 gap-px overflow-hidden rounded" data-testid="year-month-mix">
        {month.mix.map((zone) => (
          <span
            key={zone.zone}
            style={{
              flexGrow: zone.share?.raw ?? 0,
              background: ZONE_COLOURS[zone.zone] ?? 'var(--chart-band)',
            }}
            title={t('year.zone.title', {
              zone: zone.zone,
              share: zone.share?.text ?? t('year.day.noMix'),
            })}
            data-testid="year-month-zone"
            data-zone={zone.zone}
          />
        ))}
      </div>
    </section>
  )
}

/**
 * The legend of the calendar: the steps of the colouring, in the app's
 * own words.
 */
const LEGEND: { key: CatalogueKey; colour: string }[] = [
  { key: 'year.calendar.legend.noMix', colour: 'var(--surface-sunken)' },
  { key: 'year.calendar.legend.low', colour: 'var(--chart-band)' },
  { key: 'year.calendar.legend.mid', colour: 'var(--brand-300)' },
  { key: 'year.calendar.legend.high', colour: 'var(--accent-200)' },
  { key: 'year.calendar.legend.top', colour: 'var(--accent-400)' },
]

export function YearPage() {
  const year = useQuery({
    queryKey: ['insights-year'],
    queryFn: () => api.insightsYear(),
  })

  const data = year.data

  return (
    <div>
      <Link
        to="/progress"
        className="tap-inline mb-2 text-[13px] text-brand-700 underline"
        data-testid="year-back"
      >
        {t('year.back')}
      </Link>
      <PageHeader
        title={t('year.heading')}
        meta={
          data ? (
            <>
              {t('year.window', { from: data.window.from_label, to: data.window.to_label })}
              {' · '}
              {t('year.comparison', { days: data.comparison.days })}
            </>
          ) : (
            t('common.loading')
          )
        }
      />

      {year.isError && !data ? (
        <section className="card py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void year.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      ) : null}

      {!data ? (
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      ) : (
        <>
          {/* ------------------------------------------------- the months */}
          {/*
            Nothing recorded at all: the year says so, and invents no
            numbers. The months are still there - the year is twelve
            months - and each of them holds none.
          */}
          {data.months.every((month) => month.sessions === 0) ? (
            <p
              className="mb-2 text-[13px] text-ink-muted"
              data-testid="year-empty"
            >
              {t('year.empty')}
            </p>
          ) : null}
          <section className="mt-1" data-testid="year-months">
            <h2 className="mb-2 caps">
              {t('year.months.heading')}
            </h2>
            <div className="grid gap-3 sm:grid-cols-2">
              {data.months.map((month) => (
                <MonthCard key={month.key} month={month} />
              ))}
            </div>
          </section>

          {/* ------------------------------------------------ the calendar */}
          {/*
            The year as a calendar: every day of the window, coloured by
            how much of that day was in the high zones.

            **Days, never places.** The grid says when the athlete ran and
            how hard; it says nothing about where, and that is the
            heatmap's surface to say.
          */}
          <section className="card mt-3 py-3" data-testid="year-calendar-card">
            <h2 className="mb-1 caps">
              {t('year.calendar.heading')}
            </h2>
            <p className="mb-3 text-[13px] text-ink-soft">{t('year.calendar.note')}</p>
            <YearCalendar days={data.days} />
            <div className="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1">
              <span className="text-[11.5px] text-ink-faint">{t('year.calendar.legend')}</span>
              {LEGEND.map((step) => (
                <span
                  key={step.key}
                  className="inline-flex items-center gap-1 text-[11.5px] text-ink-muted"
                >
                  <i
                    className="inline-block h-2.5 w-2.5 rounded-[3px]"
                    style={{ background: step.colour }}
                  />
                  {t(step.key)}
                </span>
              ))}
            </div>
          </section>

          {/* ---------------------------------------------- what changed */}
          {/*
            The diff against the athlete's own past, in the core's own
            sentences. There is no verdict and nothing that tells the
            athlete what to do: the sentences say what the records say.
          */}
          <section className="card mt-3 py-3" data-testid="year-changed">
            <h2 className="mb-1 caps">
              {t('year.changed.heading')}
            </h2>
            <p className="mb-2 text-[13px] text-ink-soft">{t('year.changed.note')}</p>
            <ul className="space-y-1.5" data-testid="year-changed-lines">
              {data.changed.map((line) => (
                <li key={line} className="text-[13.5px] leading-snug text-ink-soft">
                  {line}
                </li>
              ))}
            </ul>
          </section>
        </>
      )}
    </div>
  )
}
