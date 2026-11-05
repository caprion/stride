import { format, parseISO } from 'date-fns'
import { t } from '../i18n'
import { formatDateShort } from '../lib/format'
import { shiftDate, validDate, weekDates } from '../lib/history'

export function DateNavigator({
  date,
  onDate,
}: {
  date: string
  onDate: (date: string) => void
}) {
  return (
    <section className="card mb-3 overflow-hidden" aria-label={t('body.history.controls')}>
      <div className="flex flex-wrap items-end justify-between gap-2 px-3 py-2">
        <button
          type="button"
          className="tap rounded-md border border-edge px-2 text-[13px] text-brand-700"
          aria-label={t('history.previousWeek')}
          onClick={() => onDate(shiftDate(date, -7))}
        >
          &larr;
        </button>
        <label className="flex min-w-0 flex-col text-[12px] text-ink-muted">
          {t('body.history.date')}
          <input
            type="date"
            className="tap min-w-0 rounded-md border border-edge bg-surface px-2 text-ink"
            value={date}
            onChange={(event) => {
              const next = validDate(event.target.value)
              if (next) onDate(next)
            }}
          />
        </label>
        <button
          type="button"
          className="tap rounded-md border border-edge px-2 text-[13px] text-brand-700"
          aria-label={t('history.nextWeek')}
          onClick={() => onDate(shiftDate(date, 7))}
        >
          &rarr;
        </button>
      </div>
      <div className="grid grid-cols-7 border-t border-edge">
        {weekDates(date).map((day) => (
          <button
            type="button"
            key={day}
            className={`tap flex flex-col items-center justify-center py-2 text-[12px] ${day === date ? 'bg-brand-600 text-white' : 'text-ink-muted hover:bg-brand-50'}`}
            aria-label={formatDateShort(day)}
            aria-pressed={day === date}
            onClick={() => onDate(day)}
          >
            <span>{format(parseISO(day), 'EEE')}</span>
            <span className="font-semibold">{format(parseISO(day), 'd')}</span>
          </button>
        ))}
      </div>
    </section>
  )
}
