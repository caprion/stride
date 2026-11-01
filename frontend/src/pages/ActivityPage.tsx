/**
 * The activity detail page.
 *
 * What the run was, what it meant, and how it sat against the plan. The
 * numbers in the header, the numbers in the story and the numbers in the
 * chart are the same numbers.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { formatDateMedium, renderedText, sportLabel } from '../lib/format'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'

export function ActivityPage({ id }: { id: string }) {
  const [splitsOpen, setSplitsOpen] = useState(false)
  const [elevationOpen, setElevationOpen] = useState(false)
  const [noteOpen, setNoteOpen] = useState(false)
  const [note, setNote] = useState('')
  const client = useQueryClient()

  const activity = useQuery({ queryKey: ['activity', id], queryFn: () => api.activity(id) })
  const data = activity.data

  useEffect(() => {
    if (data?.note !== undefined && data?.note !== null) setNote(data.note)
  }, [data?.note])

  const saveNote = useMutation({
    mutationFn: (value: string | null) => api.saveNote(id, value),
    onSuccess: (answer) => {
      client.setQueryData(['activity', id], (previous: unknown) => {
        const run = previous as { note?: string | null } | undefined
        return run ? { ...run, note: answer.note } : previous
      })
      setNoteOpen(false)
    },
  })

  if (!data) {
    return (
      <div>
        <PageHeader title={t('activity.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const samples = data.samples.map((sample) => ({
    // The chart's own scale: minutes into the run.
    minute: Math.round(sample.offset_s / 60),
    hr: sample.heart_rate?.raw ?? null,
    hrText: sample.heart_rate?.text ?? null,
    pace: sample.pace?.raw ?? null,
    paceText: sample.pace?.text ?? null,
  }))
  const elevation = data.elevation.map((point) => ({
    // The chart's own scale: the distance along the run, in kilometres.
    // The words beside it are the core's — the tooltip shows the text the
    // API sent, not a number this app rounded.
    km: (point.distance.raw ?? 0) / 1000,
    m: point.elevation.raw ?? 0,
    elevationText: point.elevation.text,
  }))

  return (
    <div>
      <PageHeader
        title={sportLabel(data.sport_type)}
        meta={formatDateMedium(data.local_date)}
      />

      {/* ------------------------------------- the three headline numbers */}
      <section className="card px-4 py-3" data-testid="activity-detail">
        <div className="stat-strip">
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.distance')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {renderedText(data.distance)}
            </span>
          </span>
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.time')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {renderedText(data.duration)}
            </span>
          </span>
          <span>
            <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
              {t('vital.pace')}
            </span>
            <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
              {renderedText(data.pace)}
            </span>
          </span>
        </div>
      </section>

      {/* ------------------------------------------------------- the story */}
      {data.story ? (
        <section className="card mt-3 px-4 py-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.story.heading')}
          </h2>
          <p className="mt-1 text-[14.5px] text-ink-soft">{data.story}</p>
        </section>
      ) : null}

      {/* --------------------------------------------------------- effort */}
      <section className="card mt-3 px-4 py-3">
        <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('activity.effort.heading')}
        </h2>
        <div style={{ width: '100%', height: 150 }}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: -20 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={38} />
              <Tooltip
                formatter={(_value, name, item) => {
                  const payload = item?.payload as
                    | { hrText?: string | null; paceText?: string | null }
                    | undefined
                  // The tooltip shows the core's words, not the chart's
                  // numbers.
                  if (name === 'hr') return payload?.hrText ?? ''
                  return payload?.paceText ?? ''
                }}
              />
              <Line type="monotone" dataKey="hr" stroke="#1E5E96" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        <div style={{ width: '100%', height: 120 }} className="mt-2">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: -20 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={38} />
              <Tooltip
                formatter={(_value, name, item) => {
                  const payload = item?.payload as
                    | { hrText?: string | null; paceText?: string | null }
                    | undefined
                  if (name === 'hr') return payload?.hrText ?? ''
                  return payload?.paceText ?? ''
                }}
              />
              <Line
                type="monotone"
                dataKey="pace"
                stroke="#D99A0B"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </section>

      {/* -------------------------------------------------------- splits */}
      <section className="card mt-3 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.splits.heading')}
          </h2>
          <button
            type="button"
            onClick={() => setSplitsOpen((open) => !open)}
            className="tap text-[13px] font-medium text-brand-600 underline underline-offset-2"
          >
            {splitsOpen ? t('activity.hideSplits') : t('activity.showSplits')}
          </button>
        </div>
        {splitsOpen ? (
          <ul className="mt-2">
            {data.splits.map((split) => (
              <li
                key={split.index}
                className="grid grid-cols-[1fr_auto_auto] items-baseline gap-x-3 border-b border-edge py-1.5 last:border-b-0"
              >
                <span className="text-[13px] text-ink-soft">
                  {renderedText(split.distance)}
                </span>
                <span className="text-[13px] tabular-nums text-ink">
                  {renderedText(split.duration)}
                </span>
                <span className="text-[13px] tabular-nums text-ink-muted">
                  {renderedText(split.heart_rate)}
                </span>
              </li>
            ))}
          </ul>
        ) : null}
      </section>

      {/* ------------------------------------------------------ elevation */}
      <section className="card mt-3 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.elevation.heading')}
          </h2>
          <button
            type="button"
            onClick={() => setElevationOpen((open) => !open)}
            className="tap text-[13px] font-medium text-brand-600 underline underline-offset-2"
          >
            {t('activity.showElevation')}
          </button>
        </div>
        {elevationOpen ? (
          <div style={{ width: '100%', height: 140 }} className="mt-2">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={elevation} margin={{ top: 4, right: 4, bottom: 0, left: -14 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis dataKey="km" tick={{ fontSize: 11, fill: '#61788D' }} />
                <YAxis tick={{ fontSize: 11, fill: '#61788D' }} width={40} />
                <Tooltip
                  formatter={(_value, _name, item) =>
                    (item?.payload as { elevationText?: string } | undefined)?.elevationText ?? ''
                  }
                />
                <Area type="monotone" dataKey="m" stroke="#3F6E9E" fill="#D6E6F3" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        ) : null}
      </section>

      {/* -------------------------------------------------- against the plan */}
      {data.plan_line ? (
        <section className="card mt-3 px-4 py-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.plan.heading')}
          </h2>
          <p className="mt-1 text-[14px] text-ink-soft" data-testid="activity-plan-line">
            {data.plan_line.text}
          </p>
        </section>
      ) : null}

      {/* ----------------------------------------------------- the note */}
      {/*
        The note is the only free text the product stores, and it belongs
        to the run it is about. It is edited here, on the run's own page —
        the settings screen holds the settings, not the notes.
      */}
      <section className="card mt-3 px-4 py-3" data-testid="activity-note">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
            {t('activity.note.heading')}
          </h2>
          <button
            type="button"
            className="tap text-[13px] font-medium text-brand-600 underline underline-offset-2"
            data-testid="activity-note-edit"
            onClick={() => {
              setNoteOpen((open) => !open)
            }}
          >
            {data.note ? t('activity.note.edit') : t('runs.addNote')}
          </button>
        </div>

        {noteOpen ? (
          <div className="mt-2">
            <textarea
              value={note}
              rows={3}
              placeholder={t('activity.note.placeholder')}
              onChange={(event) => setNote(event.target.value)}
              data-testid="activity-note-input"
              className="tap w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink outline-none focus:border-brand-400"
            />
            <div className="mt-2 flex flex-wrap items-center gap-3">
              <button
                type="button"
                className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
                disabled={saveNote.isPending}
                data-testid="activity-note-save"
                onClick={() => {
                  saveNote.mutate(note.trim() === '' ? null : note)
                }}
              >
                {t('activity.note.save')}
              </button>
              <button
                type="button"
                className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
                onClick={() => {
                  setNoteOpen(false)
                  setNote(data.note ?? '')
                }}
              >
                {t('common.close')}
              </button>
            </div>
          </div>
        ) : data.note ? (
          <p className="mt-1 text-[14.5px] text-ink-soft" data-testid="activity-note-text">
            {data.note}
          </p>
        ) : (
          <p className="mt-1 text-[13.5px] text-ink-muted" data-testid="activity-note-empty">
            {t('activity.note.none')}
          </p>
        )}
      </section>

      {/* --------------------------------------------------------- actions */}
      <div className="mt-3 flex flex-wrap gap-4 pb-2">
        <a
          href={`/compare/${data.id}`}
          className="tap-inline text-[13px] font-medium text-brand-600 underline underline-offset-2"
        >
          {t('runs.compareWith')}
        </a>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
          onClick={() => {
            setNoteOpen(true)
          }}
        >
          {t('runs.addNote')}
        </button>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2"
        >
          {t('activity.export')}
        </button>
      </div>
    </div>
  )
}
