import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'
import { formatDateShort, renderedText } from '../lib/format'
import { t } from '../i18n'
import { QueryError } from './QueryError'

const STAGES = [
  { key: 'deep_sleep', label: 'body.sleep.deep', color: '#124A7C' },
  { key: 'rem_sleep', label: 'body.sleep.rem', color: '#7BA8CD' },
  { key: 'light_sleep', label: 'body.sleep.light', color: '#D6E6F3' },
  { key: 'awake', label: 'body.sleep.awake', color: '#F2B01E' },
] as const

export function SleepComposition({
  date,
  beforeRun = false,
}: {
  date: string
  beforeRun?: boolean
}) {
  const sleep = useQuery({
    queryKey: ['sleep', date],
    queryFn: () => api.physiologySleep({ from: date, to: date }),
  })
  if (sleep.isError)
    return (
      <QueryError
        retry={() => {
          void sleep.refetch()
        }}
      />
    )
  if (sleep.isPending)
    return <p className="my-3 text-[13px] text-ink-muted">{t('common.loading')}</p>
  const night = sleep.data.days.find((day) => day.date === date)
  if (!night || (!night.sleep && !STAGES.some(({ key }) => night[key]))) return null
  const total = STAGES.reduce((sum, { key }) => sum + Math.max(0, night[key]?.raw ?? 0), 0)

  return (
    <section className="card my-3 px-4 py-3" data-testid="sleep-composition" data-date={date}>
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h2 className="text-[13px] font-semibold text-ink">
          {beforeRun
            ? t('body.sleep.run')
            : t('body.sleep.heading', { date: formatDateShort(date) })}
        </h2>
        <span className="text-[15px] font-semibold tabular-nums">
          {renderedText(night.sleep)}
        </span>
      </div>
      {beforeRun ? (
        <p className="mt-1 text-[12px] text-ink-muted">{formatDateShort(date)}</p>
      ) : null}
      {total > 0 ? (
        <div
          className="mt-3 flex h-5 overflow-hidden rounded"
          role="img"
          aria-label={t('body.sleep.composition')}
        >
          {STAGES.map(({ key, color }) => (
            <span
              key={key}
              data-stage={key}
              style={{
                backgroundColor: color,
                width: `${(Math.max(0, night[key]?.raw ?? 0) / total) * 100}%`,
              }}
            />
          ))}
        </div>
      ) : (
        <p className="mt-2 text-[13px] text-ink-muted">{t('body.sleep.noStages')}</p>
      )}
      <dl className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
        {STAGES.map(({ key, label, color }) => (
          <div key={key}>
            <dt className="flex items-center gap-1.5 text-[12px] text-ink-muted">
              <span className="h-2 w-2 rounded-sm" style={{ backgroundColor: color }} />
              {t(label)}
            </dt>
            <dd className="text-[13px] font-medium tabular-nums">
              {night[key] ? renderedText(night[key]) : t('body.sleep.missing')}
            </dd>
          </div>
        ))}
      </dl>
      <p className="mt-3 text-[12px] text-ink-muted">{t('body.sleep.note')}</p>
    </section>
  )
}
