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
  ReferenceArea,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { axisTicks } from '../lib/chart'
import { canDeliverFile, downloadFile, fileName } from '../lib/download'
import { formatDateMedium, renderedText, sportLabel } from '../lib/format'
import { t } from '../i18n'
import { axisTick } from '../components/ChartAxis'
import { PageHeader } from '../ui/layout/PageHeader'

export function ActivityPage({ id }: { id: string }) {
  const [splitsOpen, setSplitsOpen] = useState(false)
  const [elevationOpen, setElevationOpen] = useState(false)
  const [noteOpen, setNoteOpen] = useState(false)
  const [note, setNote] = useState('')
  const [fitNotice, setFitNotice] = useState<string | null>(null)
  const client = useQueryClient()

  const activity = useQuery({ queryKey: ['activity', id], queryFn: () => api.activity(id) })
  const zones = useQuery({
    queryKey: ['fitness-zones', id],
    queryFn: () => api.fitnessZones(id),
  })
  const data = activity.data

  useEffect(() => {
    if (data?.note !== undefined && data?.note !== null) setNote(data.note)
  }, [data?.note])

  const exportFit = useMutation({
    mutationFn: () => api.activityFit(id),
    onSuccess: (bytes) => {
      downloadFile(fileName('activity', 'fit'), bytes, 'application/octet-stream')
      setFitNotice(t('activity.exported'))
    },
    onError: () => {
      setFitNotice(t('activity.exportFailed'))
    },
  })

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
  /**
   * The axes' labels.
   *
   * They are the core's own words — "148 bpm", "5:24 /km" — so every
   * label carries its unit. A bare number on the pace axis would be the
   * app printing a domain number on its own.
   */
  const heartRateTicks = axisTicks(
    data.samples.map((sample) => ({
      value: sample.heart_rate?.raw ?? null,
      text: sample.heart_rate?.text ?? null,
    })),
  )
  const paceTicks = axisTicks(
    data.samples.map((sample) => ({
      value: sample.pace?.raw ?? null,
      text: sample.pace?.text ?? null,
    })),
  )
  const elevation = data.elevation.map((point) => ({
    // The chart's own scale: the distance along the run, in kilometres.
    // The words beside it are the core's — the tooltip shows the text the
    // API sent, not a number this app rounded.
    km: (point.distance.raw ?? 0) / 1000,
    m: point.elevation.raw ?? 0,
    elevationText: point.elevation.text,
  }))
  // The elevation's axis, in the core's words: "40 m".
  const elevationTicks = axisTicks(
    data.elevation.map((point) => ({
      value: point.elevation.raw,
      text: point.elevation.text,
    })),
  )

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
      <section className="card mt-3 px-4 py-3" data-testid="activity-effort">
        <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
          {t('activity.effort.heading')}
        </h2>
        <p className="mb-1 text-[12px] font-medium text-ink-muted">{t('vital.heartRate')}</p>
        <div style={{ width: '100%', height: 150 }}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis
                ticks={heartRateTicks.map((tick) => tick.value)}
                tick={axisTick(heartRateTicks)}
                width={64}
                tickLine={false}
                axisLine={false}
                domain={['dataMin - 5', 'dataMax + 5']}
              />
              {/*
                The athlete's own zones: the bands the core computed, on
                the numbers the core rendered the words from. They are
                bands of the athlete's history, not a judgement — nothing
                here says what the run should have been.
              */}
              {(zones.data?.zones ?? [])
                .filter((band) => band.from_bpm !== null && band.to_bpm !== null)
                .map((band) => (
                  <ReferenceArea
                    key={band.label}
                    y1={band.from_bpm ?? 0}
                    y2={band.to_bpm ?? 0}
                    fill="#D6E6F3"
                    fillOpacity={0.35}
                    stroke="#A9C9E2"
                    strokeOpacity={0.8}
                  />
                ))}
              <Tooltip
                formatter={(_value, _name, item) => {
                  const payload = item?.payload as
                    | { hrText?: string | null; paceText?: string | null }
                    | undefined
                  // The tooltip shows the core's words, not the chart's
                  // numbers.
                  return payload?.hrText ?? ''
                }}
              />
              <Line
                type="monotone"
                name={t('vital.heartRate')}
                dataKey="hr"
                stroke="#1E5E96"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
        {zones.data && zones.data.zones.length > 0 ? (
          <ul className="mt-2 flex flex-wrap gap-x-4 gap-y-1" data-testid="activity-zones">
            {zones.data.zones.map((band) => (
              <li key={band.label} className="text-[12px] text-ink-muted">
                <span className="font-semibold text-ink-soft">{band.label}</span>
                {band.range ? <span> · {band.range}</span> : null}
                {band.time ? <span> · {renderedText(band.time)}</span> : null}
              </li>
            ))}
          </ul>
        ) : null}
        {zones.data?.unavailable ? (
          <p className="mt-2 text-[12px] text-ink-faint" data-testid="activity-zones-unavailable">
            {zones.data.unavailable}
          </p>
        ) : null}
        <p className="mb-1 mt-3 text-[12px] font-medium text-ink-muted">{t('vital.pace')}</p>
        <div style={{ width: '100%', height: 120 }}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={samples} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
              <CartesianGrid stroke="#E4EBF0" vertical={false} />
              <XAxis dataKey="minute" tick={{ fontSize: 11, fill: '#61788D' }} />
              <YAxis
                ticks={paceTicks.map((tick) => tick.value)}
                tick={axisTick(paceTicks)}
                width={64}
                tickLine={false}
                axisLine={false}
                domain={['dataMin - 5', 'dataMax + 5']}
              />
              <Tooltip
                formatter={(_value, _name, item) => {
                  const payload = item?.payload as
                    | { hrText?: string | null; paceText?: string | null }
                    | undefined
                  return payload?.paceText ?? ''
                }}
              />
              <Line
                type="monotone"
                name={t('vital.pace')}
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
              <AreaChart data={elevation} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                <CartesianGrid stroke="#E4EBF0" vertical={false} />
                <XAxis dataKey="km" tick={{ fontSize: 11, fill: '#61788D' }} />
                <YAxis
                  ticks={elevationTicks.map((tick) => tick.value)}
                  tick={axisTick(elevationTicks)}
                  width={64}
                  tickLine={false}
                  axisLine={false}
                  domain={['dataMin - 5', 'dataMax + 5']}
                />
                <Tooltip
                  formatter={(_value, _name, item) =>
                    (item?.payload as { elevationText?: string } | undefined)?.elevationText ?? ''
                  }
                />
                <Area
                  type="monotone"
                  name={t('activity.elevation.heading')}
                  dataKey="m"
                  stroke="#3F6E9E"
                  fill="#D6E6F3"
                />
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
      {/*
        The note and the comparison are here, and the FIT file — the
        run's original bytes, the source of truth — where the host can
        deliver a file: a download on the desktop, the native share sheet
        on the phone. The phone's sheet is not there yet, so the control
        is only where it can deliver something.
      */}
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
        {canDeliverFile() ? (
          <button
            type="button"
            data-testid="activity-export"
            className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2 disabled:opacity-50"
            disabled={exportFit.isPending}
            onClick={() => {
              setFitNotice(null)
              exportFit.mutate()
            }}
          >
            {t('activity.export')}
          </button>
        ) : null}
        {fitNotice ? (
          <p className="w-full text-[13px] text-ink-muted" data-testid="activity-export-notice">
            {fitNotice}
          </p>
        ) : null}
      </div>
    </div>
  )
}
