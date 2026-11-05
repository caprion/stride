/**
 * The activity detail page.
 *
 * What the run was, what it meant, and how it sat against the plan. The
 * numbers in the header, the numbers in the story and the numbers in the
 * chart are the same numbers.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
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
import { StreetRouteMap } from '../components/StreetRouteMap'
import { SleepComposition } from '../components/SleepComposition'
import { QueryError } from '../components/QueryError'
import { PageHeader } from '../ui/layout/PageHeader'
import { historyReturn } from '../lib/history'

export function ActivityPage({ id }: { id: string }) {
  const [splitsOpen, setSplitsOpen] = useState(true)
  const [elevationOpen, setElevationOpen] = useState(true)
  const [params] = useSearchParams()
  const returnTo = historyReturn(params.get('returnTo'))
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

  const back = (
    <Link to={returnTo} className="tap-inline mb-2 text-[13px] text-brand-700 underline">
      {t('activity.back')}
    </Link>
  )

  if (activity.isError && !data) {
    return (
      <>
        {back}
        <PageHeader title={t('activity.heading')} />
        <QueryError
          retry={() => {
            void activity.refetch()
          }}
        />
      </>
    )
  }
  if (!data) {
    return (
      <div>
        {back}
        <PageHeader title={t('activity.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const samples = data.samples.map((sample) => ({
    // The chart's own scale: minutes into the run.
    minute: sample.offset_s / 60,
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
      {back}
      <PageHeader
        title={data.name || sportLabel(data.sport_type)}
        meta={`${sportLabel(data.sport_type)} · ${formatDateMedium(data.local_date)}`}
        action={
          <Link
            to={`/compare/${data.id}?${new URLSearchParams({ returnTo })}`}
            className="tap-inline rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700"
          >
            {t('runs.compareWith')}
          </Link>
        }
      />
      <nav className="mb-3 flex flex-wrap gap-4 text-[13px] text-brand-700">
        <a href="#activity-splits" className="tap-inline underline">
          {t('activity.splits.heading')}
        </a>
        <a href="#activity-note" className="tap-inline underline">
          {t('activity.note.heading')}
        </a>
        <Link to={`/body?date=${data.local_date}`} className="tap-inline underline">
          {t('today.body.open')}
        </Link>
      </nav>
      {activity.isError ? (
        <QueryError
          retry={() => {
            void activity.refetch()
          }}
        />
      ) : null}

      <div className="activity-grid">
        <div>
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
              {data.heart_rate ? (
                <span>
                  <span className="text-[11px] uppercase tracking-[0.07em] text-ink-faint">
                    {t('vital.heartRate')}
                  </span>
                  <span className="ml-2 text-[17px] font-semibold tabular-nums">
                    {renderedText(data.heart_rate)}
                  </span>
                </span>
              ) : null}
            </div>
          </section>

          {/* ------------------------------------------------------- the story */}
          {/*
        The sentence the feed shows, and the one extra fact the detail
        knows: the run's own shape, in the core's words. The why is said
        only when it is worth a sentence — when a split stands out from
        the run's average pace — so when it is absent, the sentence
        stands alone.
      */}
          <section className="card mt-3 px-4 py-3">
            <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              {t('activity.story.heading')}
            </h2>
            <p className="mt-1 text-[14.5px] text-ink-soft">{data.story}</p>
            {data.why ? (
              <p className="mt-1 text-[14.5px] text-ink-soft" data-testid="activity-why">
                {data.why}
              </p>
            ) : null}
          </section>

          {/* ----------------------------------------------------------- map */}
          {data.geo_points.length > 1 ? (
            <section className="card mt-3 px-4 py-3" data-testid="activity-route">
              <h2 className="text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                {t('activity.route.heading')}
              </h2>
              <div className="mt-2">
                <StreetRouteMap key={id} points={data.geo_points} />
              </div>
            </section>
          ) : null}
        </div>
        <div>
          {/* --------------------------------------------------------- effort */}
          <section className="card mt-3 px-4 py-3" data-testid="activity-effort">
            <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              {t('activity.effort.heading')}
            </h2>
            <p className="mb-3 text-[12px] text-ink-muted">{t('activity.effort.cursor')}</p>
            {zones.isError ? (
              <QueryError
                retry={() => {
                  void zones.refetch()
                }}
              />
            ) : null}
            <p className="mb-1 text-[12px] font-medium text-ink-muted">
              {t('vital.heartRate')}
            </p>
            <div style={{ width: '100%', height: 150 }}>
              <ResponsiveContainer width="100%" height="100%">
                <LineChart
                  data={samples}
                  syncId={`effort-${id}`}
                  margin={{ top: 4, right: 4, bottom: 0, left: 0 }}
                >
                  <CartesianGrid stroke="#E4EBF0" vertical={false} />
                  <XAxis
                    dataKey="minute"
                    type="number"
                    domain={[0, 'dataMax']}
                    tickCount={5}
                    tick={{ fontSize: 11, fill: '#61788D' }}
                  />
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
                        { hrText?: string | null; paceText?: string | null } | undefined
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
              <div className="mt-3" data-testid="activity-zones">
                <p className="text-[11px] text-ink-muted">{t('activity.zones.basis')}</p>
                <ul className="mt-2 space-y-2">
                  {zones.data.zones.map((band, index) => (
                    <li
                      key={band.label}
                      className="grid grid-cols-[32px_1fr_auto] items-center gap-2 text-[12px] text-ink-muted"
                    >
                      <span className="font-semibold text-ink-soft">{band.label}</span>
                      <span className="min-w-0">
                        <span className="block h-2 rounded bg-brand-50">
                          <span
                            className="block h-full rounded"
                            style={{
                              backgroundColor: [
                                '#A9C9E2',
                                '#7BA8CD',
                                '#1E5E96',
                                '#124A7C',
                                '#D99A0B',
                              ][index % 5],
                              width: `${(100 * Math.max(0, band.time?.raw ?? 0)) / Math.max(1, ...(zones.data?.zones ?? []).map((zone) => zone.time?.raw ?? 0))}%`,
                            }}
                          />
                        </span>
                        {band.range ? (
                          <span className="mt-1 block text-[11px]">{band.range}</span>
                        ) : null}
                      </span>
                      <span className="tabular-nums">{renderedText(band.time)}</span>
                    </li>
                  ))}
                </ul>
                {zones.data.without_heart_rate ? (
                  <p className="mt-2 text-[11px] text-ink-muted">
                    {t('activity.zones.missing')} ·{' '}
                    {renderedText(zones.data.without_heart_rate)}
                  </p>
                ) : null}
              </div>
            ) : null}
            {zones.data?.unavailable ? (
              <p
                className="mt-2 text-[12px] text-ink-faint"
                data-testid="activity-zones-unavailable"
              >
                {zones.data.unavailable}
              </p>
            ) : null}
            <p className="mb-1 mt-3 text-[12px] font-medium text-ink-muted">
              {t('vital.pace')}
            </p>
            <div style={{ width: '100%', height: 120 }}>
              <ResponsiveContainer width="100%" height="100%">
                <LineChart
                  data={samples}
                  syncId={`effort-${id}`}
                  margin={{ top: 4, right: 4, bottom: 0, left: 0 }}
                >
                  <CartesianGrid stroke="#E4EBF0" vertical={false} />
                  <XAxis
                    dataKey="minute"
                    type="number"
                    domain={[0, 'dataMax']}
                    tickCount={5}
                    tick={{ fontSize: 11, fill: '#61788D' }}
                  />
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
                        { hrText?: string | null; paceText?: string | null } | undefined
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
          <section id="activity-splits" className="card mt-3 px-4 py-3">
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
              <table className="mt-2 w-full table-fixed text-[12px]">
                <thead>
                  <tr className="border-b border-edge text-ink-muted">
                    <th scope="col" className="py-2 text-left font-medium">
                      {t('vital.distance')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.pace')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.time')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.heartRate')}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {data.splits.map((split) => (
                    <tr key={split.index} className="border-b border-edge last:border-b-0">
                      <th scope="row" className="py-2 text-left font-medium text-ink-soft">
                        {renderedText(split.distance)}
                      </th>
                      <td className="py-2 text-right font-semibold tabular-nums text-brand-700">
                        {renderedText(split.pace)}
                      </td>
                      <td className="py-2 text-right tabular-nums text-ink">
                        {renderedText(split.duration)}
                      </td>
                      <td className="py-2 text-right tabular-nums text-ink-muted">
                        {renderedText(split.heart_rate)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
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
                {elevationOpen ? t('activity.hideElevation') : t('activity.showElevation')}
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
                        (item?.payload as { elevationText?: string } | undefined)
                          ?.elevationText ?? ''
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
        </div>
      </div>

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
      <section id="activity-note" className="card mt-3 px-4 py-3" data-testid="activity-note">
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
            {saveNote.isError ? (
              <p role="alert" className="mb-2 text-[13px] text-ink-muted">
                {t('common.saveFailed')}
              </p>
            ) : null}
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

      <SleepComposition date={data.local_date} beforeRun />
      {/* --------------------------------------------------------- actions */}
      {/*
        The note and the comparison are here, and the FIT file — the
        run's original bytes, the source of truth — where the host can
        deliver a file: a download on the desktop, the native share sheet
        on the phone. The phone's sheet is not there yet, so the control
        is only where it can deliver something.
      */}
      <div className="mt-3 flex flex-wrap gap-4 pb-2">
        <Link
          to={`/compare/${data.id}?${new URLSearchParams({ returnTo })}`}
          className="tap-inline text-[13px] font-medium text-brand-600 underline underline-offset-2"
        >
          {t('runs.compareWith')}
        </Link>
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
