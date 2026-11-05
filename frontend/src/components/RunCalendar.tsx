import { useQuery } from '@tanstack/react-query'
import { addMonths, endOfMonth, format, parseISO } from 'date-fns'
import { useState } from 'react'
import { t } from '../i18n'
import { api } from '../lib/api'
import { formatDateShort, renderedText } from '../lib/format'
import { QueryError } from './QueryError'

export function RunCalendar({
  from,
  to,
  onRange,
}: {
  from: string
  to: string
  onRange: (from: string, to: string) => void
}) {
  const [expanded, setExpanded] = useState(false)
  const month = from.slice(0, 7)
  const calendar = useQuery({
    queryKey: ['calendar', month],
    queryFn: () =>
      api.calendar(
        month
          ? { year: Number(month.slice(0, 4)), month: Number(month.slice(5, 7)) }
          : undefined,
      ),
  })
  const data = calendar.data
  if (calendar.isError)
    return (
      <QueryError
        retry={() => {
          void calendar.refetch()
        }}
      />
    )
  if (!data) return <p>{t('common.loading')}</p>
  const first = `${data.year}-${String(data.month).padStart(2, '0')}-01`
  const chooseMonth = (next: string) =>
    onRange(next, format(endOfMonth(parseISO(next)), 'yyyy-MM-dd'))
  const offset = (parseISO(first).getDay() + 6) % 7

  return (
    <section className="runs-calendar card overflow-hidden" aria-label={t('runs.calendar')}>
      <div className="flex items-center justify-between gap-1 px-2 py-2">
        <button
          type="button"
          className="tap rounded-md px-2 text-brand-700"
          aria-label={t('history.previousMonth')}
          onClick={() => chooseMonth(format(addMonths(parseISO(first), -1), 'yyyy-MM-dd'))}
        >
          &larr;
        </button>
        <h2 className="text-[14px] font-semibold">{format(parseISO(first), 'MMMM yyyy')}</h2>
        <button
          type="button"
          className="tap rounded-md px-2 text-brand-700"
          aria-label={t('history.nextMonth')}
          onClick={() => chooseMonth(format(addMonths(parseISO(first), 1), 'yyyy-MM-dd'))}
        >
          &rarr;
        </button>
      </div>
      <button
        type="button"
        className="run-calendar-toggle tap w-full border-t border-edge text-[13px] text-brand-700"
        aria-expanded={expanded}
        onClick={() => setExpanded(!expanded)}
      >
        {expanded ? t('runs.calendar.hide') : t('runs.calendar')}
      </button>
      <div
        className={`run-calendar-days grid-cols-7 border-t border-edge ${expanded ? 'is-open' : ''}`}
      >
        {Array.from({ length: 7 }, (_, index) => (
          <span key={index} className="py-2 text-center text-[11px] text-ink-muted">
            {format(new Date(2026, 0, 5 + index), 'EEEEE')}
          </span>
        ))}
        {Array.from({ length: offset }, (_, index) => (
          <span key={`empty-${index}`} />
        ))}
        {data.days.map((day) => (
          <button
            type="button"
            key={day.date}
            className={`tap flex flex-col items-center justify-center py-1 text-[12px] ${day.date >= from && from && (!to || day.date <= to) ? 'bg-brand-50 font-semibold text-brand-700' : 'text-ink-muted'}`}
            aria-label={`${formatDateShort(day.date)}: ${t('today.week.sessions', { n: day.activities.length })}${day.activities.length ? `, ${renderedText(day.distance)}` : ''}`}
            aria-pressed={from === day.date && to === day.date}
            onClick={() => onRange(day.date, day.date)}
          >
            <span>{day.day}</span>
            <span
              className={`mt-1 h-1 w-1 rounded-full ${day.activities.length ? 'bg-brand-600' : 'bg-transparent'}`}
            />
          </button>
        ))}
      </div>
      <p className="border-t border-edge px-3 py-2 text-[11px] text-ink-muted">
        {t('runs.calendar.note')}
      </p>
    </section>
  )
}
