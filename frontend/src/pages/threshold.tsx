/**
 * The threshold: **the four candidates, the choice, the dated history**.
 *
 * The app **computes the candidates**; the athlete **chooses one**. The
 * choice is a dated fact with its source, and the app never says which
 * candidate is right.
 *
 * ## What the card shows
 *
 * Each candidate is a card of **the value, the source and the date**:
 *
 *   - the **watch's own zone model** — the floor of the fifth zone, a
 *     heart rate;
 *   - **from a race** — the hour-pace band, a pace;
 *   - **from a field test** — the estimate from a run, a heart rate and
 *     a pace;
 *   - **the number the athlete brought** — their own.
 *
 * **What each candidate cannot know is not on the card.** That line is
 * the explanation, and the explanation is not product content: the card
 * shows the value, the source and the date, and the rule behind a
 * candidate opens beside it on demand.
 *
 * ## The choice
 *
 * The choice in force is named with its date, and the choices that came
 * before it stay in the **dated history** — one threshold at a time, and
 * a plan adopted in March keeps the number it was paced with.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { t } from '../i18n'
import { api } from '../lib/api'
import { ApiError } from '../lib/transport'
import { formatDateMedium } from '../lib/format'
import type { CandidateRow, ThresholdInsight } from '../lib/types'

/**
 * The bases a choice can be made for.
 *
 * **The app holds one base at a time**, and the choice says which it was
 * made for: the base's bands are read against the threshold, and the two
 * bases read it differently.
 */
const BASES = [
  { key: 'pfitzinger', label: 'Pfitzinger' },
  { key: 'eighty_twenty', label: '80/20' },
] as const

/**
 * The labels a typed number can carry.
 *
 * **The app never invents one**: the athlete says where the number came
 * from, and the list is the core's own.
 */
const TYPED_LABELS = [
  { key: 'my own number', label: 'my own number' },
  { key: 'lab test', label: 'a lab test' },
  { key: 'coach', label: 'a coach' },
  { key: 'another app', label: 'another app' },
  { key: 'book or table', label: 'a book or a table' },
] as const

/**
 * One candidate, as a card of value, source and date.
 *
 * **A candidate with no value says why**, in the core's own words, and
 * shows no number at all.
 */
function CandidateCard({
  candidate,
  inForce,
  onChoose,
  busy,
}: {
  candidate: CandidateRow
  /** Whether this candidate's value is the choice in force. */
  inForce: boolean
  /** The athlete's act of choosing it. */
  onChoose: () => void
  /** Whether a choice is being written. */
  busy: boolean
}) {
  const hasValue = candidate.value !== null
  return (
    <div className="rounded-md border border-edge px-2.5 py-2" data-testid={`threshold-candidate-${candidate.source}`}>
      <div className="flex items-baseline justify-between gap-2">
        <span className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
          {candidate.label}
        </span>
        {inForce ? (
          <span className="text-[11px] font-semibold text-brand-700" data-testid={`threshold-in-force-${candidate.source}`}>
            {t('threshold.card.inUse')}
          </span>
        ) : null}
      </div>
      <p
        className={`mt-1 text-[17px] font-semibold tabular-nums ${hasValue ? 'text-ink' : 'text-ink-muted'}`}
        data-testid={`threshold-value-${candidate.source}`}
      >
        {candidate.value ?? candidate.missing ?? ''}
      </p>
      {candidate.date ? (
        <p className="text-[11.5px] text-ink-muted" data-testid={`threshold-date-${candidate.source}`}>
          {formatDateMedium(candidate.date)}
        </p>
      ) : null}
      {candidate.rule?.line ? (
        <details className="mt-1 text-[11px] text-ink-faint" data-testid={`threshold-rule-${candidate.source}`}>
          <summary className="cursor-pointer">{t('pace.card.rule')}</summary>
          <p className="mt-0.5 text-[11.5px] text-ink-muted">{candidate.rule.line}</p>
        </details>
      ) : null}
      {hasValue ? (
        <button
          type="button"
          className="tap mt-1.5 text-[12px] font-semibold text-brand-700 underline"
          data-testid={`threshold-choose-${candidate.source}`}
          disabled={busy}
          onClick={onChoose}
        >
          {t('threshold.card.choose')}
        </button>
      ) : null}
    </div>
  )
}

/**
 * The threshold's card.
 *
 * **The choice in force is named first**, and the four candidates follow
 * it: the athlete's own threshold, and the four ways of working it out.
 */
export function ThresholdCard() {
  const query = useQuery({
    queryKey: ['insights-threshold'],
    queryFn: () => api.insightsThreshold(),
  })
  const client = useQueryClient()
  const [base, setBase] = useState<string>('pfitzinger')
  const [typed, setTyped] = useState('')
  const [typedLabel, setTypedLabel] = useState<string>('my own number')

  const choice = useMutation({
    mutationFn: (body: Parameters<typeof api.chooseThreshold>[0]) => api.chooseThreshold(body),
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: ['insights-threshold'] })
    },
  })

  const data: ThresholdInsight | undefined = query.data
  const candidates = data?.candidates ?? []
  const typedCandidate = candidates.find((candidate) => candidate.source === 'typed')
  const failure = choice.error instanceof ApiError ? choice.error : null

  const choose = (source: string) => {
    choice.mutate({ source, base })
  }

  const chooseTyped = () => {
    const bpm = Number(typed)
    if (!Number.isFinite(bpm) || bpm <= 0) return
    choice.mutate({
      source: 'typed',
      base,
      heart_rate_bpm: bpm,
      label: typedLabel,
    })
  }

  return (
    <div data-testid="threshold-card">
      {query.isError && !data ? (
        <p className="text-[13px] text-ink-muted">{t('common.error')}</p>
      ) : null}
      {!data ? (
        <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
      ) : (
        <>
          {/* The choice in force: the value, the source and the date. */}
          {data.in_force ? (
            <div className="mb-2" data-testid="threshold-choice">
              <p className="text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
                {t('threshold.card.inUse')}
              </p>
              <p className="text-[19px] font-semibold tabular-nums text-ink">
                {data.in_force.value ?? ''}
              </p>
              <p className="text-[11.5px] text-ink-muted">
                {formatDateMedium(data.in_force.origin_date ?? data.in_force.entered_on)}
              </p>
            </div>
          ) : null}

          {/* The four candidates. */}
          <div className="grid gap-2 sm:grid-cols-2" data-testid="threshold-candidates">
            {candidates
              .filter((candidate) => candidate.source !== 'typed')
              .map((candidate) => (
                <CandidateCard
                  key={candidate.source}
                  candidate={candidate}
                  inForce={data.in_force?.source === candidate.source}
                  busy={choice.isPending}
                  onChoose={() => choose(candidate.source)}
                />
              ))}
          </div>

          {/* The number the athlete brought. */}
          {typedCandidate ? (
            <div className="mt-2" data-testid="threshold-typed">
              <h3 className="caps-sm">{t('threshold.card.typed')}</h3>
              <div className="mt-1 flex flex-wrap items-end gap-2">
                <label className="flex flex-col gap-1 text-[11.5px] text-ink-muted">
                  {t('threshold.card.typedField')}
                  <input
                    type="number"
                    inputMode="numeric"
                    className="tap w-[110px] rounded-md border border-edge bg-surface px-2 py-1 text-[13px] text-ink"
                    value={typed}
                    data-testid="threshold-typed-number"
                    onChange={(event) => setTyped(event.target.value)}
                  />
                </label>
                <label className="flex flex-col gap-1 text-[11.5px] text-ink-muted">
                  {t('threshold.card.typedSource')}
                  <select
                    className="tap rounded-md border border-edge bg-surface px-2 py-1 text-[13px] text-ink"
                    value={typedLabel}
                    data-testid="threshold-typed-label"
                    onChange={(event) => setTypedLabel(event.target.value)}
                  >
                    {TYPED_LABELS.map((option) => (
                      <option key={option.key} value={option.key}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  type="button"
                  className="tap text-[12px] font-semibold text-brand-700 underline"
                  data-testid="threshold-choose-typed"
                  disabled={choice.isPending}
                  onClick={chooseTyped}
                >
                  {t('threshold.card.choose')}
                </button>
              </div>
            </div>
          ) : null}

          {/* The base the choice is made for. */}
          <label className="mt-2 flex flex-col gap-1 text-[11.5px] text-ink-muted">
            {t('threshold.card.base')}
            <select
              className="tap rounded-md border border-edge bg-surface px-2 py-1 text-[13px] text-ink"
              value={base}
              data-testid="threshold-base"
              onChange={(event) => setBase(event.target.value)}
            >
              {BASES.map((option) => (
                <option key={option.key} value={option.key}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>

          {choice.isSuccess ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="threshold-saved">
              {t('threshold.card.saved')}
            </p>
          ) : null}
          {failure ? (
            <p className="mt-2 text-[12px] text-ink-muted" data-testid="threshold-refused">
              {t('threshold.card.refused')}
            </p>
          ) : null}

          {/* The choices that came before: the dated history. */}
          {data.history.length > 0 ? (
            <div className="mt-3" data-testid="threshold-history">
              <h3 className="caps-sm">{t('threshold.card.history')}</h3>
              <ul className="mt-1 space-y-0.5">
                {data.history.map((entry) => (
                  <li key={entry.date} className="text-[12px] text-ink-muted">
                    {entry.line ?? entry.value ?? ''}
                    {entry.in_force ? (
                      <span className="ml-1 font-semibold text-brand-700">
                        {t('threshold.card.inUse')}
                      </span>
                    ) : null}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </>
      )}
    </div>
  )
}
