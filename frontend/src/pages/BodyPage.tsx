/**
 * Body — the evidence behind the morning readout.
 *
 * The same three signals Today shows, with the history behind them and
 * the band they are measured against. The patterns live here and nowhere
 * else: they are facts about the past, and they always carry their
 * sample size.
 */

import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'
import { t } from '../i18n'
import { Sparkline } from '../components/Sparkline'
import { VitalRow } from '../components/VitalRow'
import { PageHeader } from '../ui/layout/PageHeader'

export function BodyPage() {
  const vitals = useQuery({ queryKey: ['physiology-vitals'], queryFn: api.vitals })
  const snapshot = vitals.data

  if (!snapshot) {
    return (
      <div>
        <PageHeader title={t('body.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  return (
    <div>
      <PageHeader title={t('body.heading')} />

      {/* --------------------------------------------- the three signals */}
      <section className="signal-grid" data-testid="body-signals">
        {snapshot.vitals.map((vital) => (
          <div key={vital.key} className="card px-4 py-2">
            <VitalRow vital={vital}>
              <div className="mt-2">
                <Sparkline
                  values={snapshot.history[vital.key]
                    .map((point) => point.value)
                    .filter((value): value is number => value !== null)}
                  band={
                    vital.baseline
                      ? { lower: vital.baseline.lower, upper: vital.baseline.upper }
                      : null
                  }
                  height={44}
                  label={t('body.history.label', { n: 30 })}
                />
              </div>
            </VitalRow>
          </div>
        ))}
      </section>

      {/* --------------------------------------------- how to read this */}
      <section className="card mt-3 px-4 py-3">
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('body.howToRead.heading')}
        </h2>
        <p className="mt-1 text-[13.5px] text-ink-soft">
          {t('body.howToRead.band', { n: 30 })}
        </p>
        <p className="text-[13.5px] text-ink-soft">{t('body.howToRead.inside')}</p>
      </section>

      {/* ------------------------------------------------------ patterns */}
      {snapshot.patterns.length > 0 ? (
        <section className="card mt-3 px-4 py-3" data-testid="body-patterns">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('body.patterns.heading')}
          </h2>
          <ul className="mt-1.5 space-y-1.5">
            {snapshot.patterns.map((pattern) => (
              <li key={pattern.id} className="text-[14px] text-ink-soft">
                {t(pattern.key, { n: pattern.n, ...pattern.params })}
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </div>
  )
}
