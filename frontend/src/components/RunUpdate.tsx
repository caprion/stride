/**
 * The run's update: the model's own reading of the run.
 *
 * **It is not the product's voice.** The product's numbers stay the
 * product's — computed in Rust, from the stored series, with their
 * sources named — and this panel sits below them, beside them, and not
 * inside them.
 *
 * Three things make that plain on the screen:
 *
 *   - the update is **attributed**: the writer and the reviewer are named
 *     under it, so nobody has to guess whose words these are;
 *   - the **checks** are listed under it: the numbers, the directions and
 *     the register, and what they found;
 *   - and the athlete can **dismiss** it. The update is stored either
 *     way; dismissing it is the reader's own business.
 */

import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { t } from '../i18n'

/** The check's name, as the app names it. */
function checkLabel(check: string): string {
  switch (check) {
    case 'groundedness':
      return t('activity.update.check.groundedness')
    case 'direction':
      return t('activity.update.check.direction')
    case 'register':
      return t('activity.update.check.register')
    case 'shape':
      return t('activity.update.check.shape')
    default:
      return check
  }
}

export function RunUpdate({ id }: { id: string }) {
  const [dismissed, setDismissed] = useState(false)
  const query = useQuery({
    queryKey: ['activity-update', id],
    queryFn: () => api.activityUpdate(id),
  })
  // The payload the model was given: the athlete can read it, so the
  // reading is never a black box.
  const payload = useQuery({
    queryKey: ['activity-update-payload', id],
    queryFn: () => api.activityUpdatePayload(id),
  })

  const update = query.data?.update
  if (!update) return null

  if (dismissed) {
    return (
      <section className="card mt-3 px-4 py-3" data-testid="activity-update-dismissed">
        <p className="text-[13px] text-ink-muted">{t('activity.update.dismissed')}</p>
        <button
          type="button"
          className="tap-inline mt-1 text-[13px] font-medium text-brand-600 underline underline-offset-2"
          data-testid="activity-update-show"
          onClick={() => {
            setDismissed(false)
          }}
        >
          {t('activity.update.showAgain')}
        </button>
      </section>
    )
  }

  return (
    <section className="card mt-3 px-4 py-3" data-testid="activity-update">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('activity.update.heading')}
        </h2>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
          data-testid="activity-update-dismiss"
          onClick={() => {
            setDismissed(true)
          }}
        >
          {t('activity.update.dismiss')}
        </button>
      </div>

      <p className="mt-1 text-[14.5px] text-ink-soft" data-testid="activity-update-draft">
        {update.draft}
      </p>
      <p className="mt-1 text-[12px] text-ink-muted" data-testid="activity-update-attribution">
        {update.reviewer
          ? t('activity.update.attribution', {
              writer: update.writer,
              reviewer: update.reviewer,
            })
          : t('activity.update.attribution.unchanged', { writer: update.writer })}
      </p>

      {payload.data?.text ? (
        <details className="mt-2 text-[12px] text-ink-muted">
          <summary className="tap-inline cursor-pointer">
            {t('activity.update.sent.heading')}
          </summary>
          <pre
            className="mt-1 whitespace-pre-wrap text-[12px] text-ink-faint"
            data-testid="activity-update-payload"
          >
            {payload.data.text}
          </pre>
        </details>
      ) : null}

      <div className="mt-2">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('activity.update.checks.heading')}
        </h3>
        <ul className="mt-1 space-y-1" data-testid="activity-update-checks">
          {update.checks.map((check) => (
            <li key={check.check} className="text-[12px] text-ink-muted">
              <span className="font-medium text-ink-soft">{checkLabel(check.check)}</span>
              {check.findings.map((finding) => (
                <span key={finding} className="ml-2">
                  {finding}
                </span>
              ))}
            </li>
          ))}
        </ul>
      </div>
    </section>
  )
}
