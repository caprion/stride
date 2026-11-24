/**
 * The activity detail page.
 *
 * What the run was, what it meant, and how it sat against the plan. The
 * numbers in the header, the numbers in the story and the numbers in the
 * chart are the same numbers.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useMemo, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { api } from '../lib/api'
import { canDeliverFile, downloadFile, fileName } from '../lib/download'
import { formatDateMedium, renderedText, sportLabel } from '../lib/format'
import { t } from '../i18n'
import { RunFlyby } from '../components/RunFlyby'
import { SleepComposition } from '../components/SleepComposition'
import { QueryError } from '../components/QueryError'
import { RunUpdate } from '../components/RunUpdate'
import { RunEffortChart } from '../components/RunEffortChart'
import { PageHeader } from '../ui/layout/PageHeader'
import { historyReturn } from '../lib/history'
import { ZONE_COLOURS } from '../lib/zones'

export function ActivityPage({ id }: { id: string }) {
  const [splitsOpen, setSplitsOpen] = useState(true)
  const [params] = useSearchParams()
  const returnTo = historyReturn(params.get('returnTo'))
  const [noteOpen, setNoteOpen] = useState(false)
  const [note, setNote] = useState('')
  const [fitNotice, setFitNotice] = useState<string | null>(null)
  const client = useQueryClient()

  const activity = useQuery({ queryKey: ['activity', id], queryFn: () => api.activity(id) })
  /**
   * The run's own samples, as the film carries them.
   *
   * The flyby is driven by them — the playhead over the seconds the watch
   * recorded, and the distance it recorded at each of them — so the run
   * detail and the film page read the same series.
   */
  const film = useQuery({ queryKey: ['activity-film', id], queryFn: () => api.runFilm(id) })
  const zones = useQuery({
    queryKey: ['fitness-zones', id],
    queryFn: () => api.fitnessZones(id),
  })
  const form = useQuery({
    queryKey: ['fitness-fingerprint', id],
    queryFn: () => api.fitnessFingerprint(id),
  })
  const data = activity.data
  /** The film's samples: one array, held still while the page renders. */
  const filmSamples = useMemo(() => film.data?.samples ?? [], [film.data])

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

  /**
   * The athlete's own zones, as the answer carries them.
   *
   * They are drawn behind the heart rate, on the run's own data: the
   * athlete's configuration, and not a judgement about the run.
   */
  const zoneBands = zones.data?.zones ?? []

  return (
    <div>
      {back}
      <PageHeader
        title={data.name || sportLabel(data.sport_type)}
        meta={`${sportLabel(data.sport_type)} · ${formatDateMedium(data.local_date)}`}
        action={
          <div className="flex flex-wrap gap-2">
            {/*
        The film is reached by an explicit action, the way the Street map
        is: the run detail keeps its own panels, and the film is the deep
        dive into this one run.
      */}
            <Link
              to={`/runs/${data.id}/film`}
              className="tap-inline rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700"
              data-testid="activity-film"
            >
              {t('activity.film')}
            </Link>
            <Link
              to={`/compare/${data.id}?${new URLSearchParams({ returnTo })}`}
              className="tap-inline rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700"
            >
              {t('runs.compareWith')}
            </Link>
          </div>
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
          <section className="card py-3" data-testid="activity-detail">
            <div className="stat-strip">
              <span>
                <span className="caps-sm">
                  {t('vital.distance')}
                </span>
                <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
                  {renderedText(data.distance)}
                </span>
              </span>
              <span>
                <span className="caps-sm">
                  {t('vital.time')}
                </span>
                <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
                  {renderedText(data.duration)}
                </span>
              </span>
              <span>
                <span className="caps-sm">
                  {t('vital.pace')}
                </span>
                <span className="ml-2 text-[17px] font-semibold tabular-nums text-ink">
                  {renderedText(data.pace)}
                </span>
              </span>
              {data.heart_rate ? (
                <span>
                  <span className="caps-sm">
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
          <section className="card mt-3 py-3">
            <h2 className="caps">
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
          {/*
        The flyby, on the run's own page: the same map the film page
        draws, with the camera on the runner and the track coloured by
        the pace. The film itself is the deep view — the ground profile,
        the zone bands, the scrubbing — and the action in the header
        opens it.
      */}
          {data.geo_points.length > 1 ? (
            <section className="card mt-3 py-3" data-testid="activity-route">
              <div className="flex flex-wrap items-baseline justify-between gap-2">
                <h2 className="caps">{t('activity.route.heading')}</h2>
                {filmSamples.length > 0 ? (
                  <span className="text-[12px] text-ink-muted">{t('film.route.coloured')}</span>
                ) : null}
              </div>
              <div className="mt-2">
                <RunFlyby key={id} points={data.geo_points} samples={filmSamples} />
              </div>
            </section>
          ) : null}
        </div>
        <div>
          {/* --------------------------------------------------------- effort */}
          <section className="card mt-3 py-3" data-testid="activity-effort">
            <h2 className="mb-2 caps">
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
            {/*
              The run's one chart: the pace and the heart rate over the
              run's own elapsed time, with the athlete's own bands
              behind the heart rate, and one cursor over both.
            */}
            <RunEffortChart samples={data.samples} zones={zoneBands} runId={data.id} />
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
                              backgroundColor: ZONE_COLOURS[index % ZONE_COLOURS.length],
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
                {zones.data.outside ? (
                  <p className="mt-2 text-[11px] text-ink-muted">
                    {t('activity.zones.missing')} · {renderedText(zones.data.outside)}
                  </p>
                ) : null}
                {/* The time the athlete was not moving. It is not time in
                    a zone, and it is not outside one: it is the stretches
                    they stood still. */}
                {zones.data.stopped && (zones.data.stopped.raw ?? 0) > 0 ? (
                  <p className="mt-1 text-[11px] text-ink-muted" data-testid="activity-zones-stopped">
                    {t('activity.zones.stopped')} · {renderedText(zones.data.stopped)}
                  </p>
                ) : null}
              </div>
            ) : null}
            {/*
        The model's own source line: where the bands came from, and the
        day the model was read. It is the core's sentence — the app shows
        it as it arrives — and it is a line under the bands, not a card.
      */}
            {zones.data?.model?.source ? (
              <p
                className="mt-2 text-[11px] text-ink-muted"
                data-testid="activity-zones-source"
              >
                {zones.data.model.source}
              </p>
            ) : null}
            {zones.data?.unavailable ? (
              <p
                className="mt-2 text-[12px] text-ink-faint"
                data-testid="activity-zones-unavailable"
              >
                {zones.data.unavailable}
              </p>
            ) : null}
          </section>

          {/* -------------------------------------------------------- splits */}
          <section id="activity-splits" className="card mt-3 py-3">
            <div className="flex items-center justify-between gap-3">
              <h2 className="caps">
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
                    <tr key={split.lap_index} className="border-b border-edge last:border-b-0">
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

          {/* ----------------------------------------------------------- form */}
          {/*
        The run's own mechanics, as **observations**: the cadence, the
        pace, the heart rate as they moved across the run, and the
        decoupling between them.

        Each row is a value and, where the run has one, the direction it
        moved — the sign of the difference. There is no score here and no
        composite: the numbers are what the run measured, and the reader
        decides what they mean. A metric the run cannot answer says why,
        in the core's own words.
      */}
          <section className="card mt-3 py-3" data-testid="activity-form">
            <h2 className="caps">
              {t('activity.form.heading')}
            </h2>
            {form.isError ? (
              <QueryError
                retry={() => {
                  void form.refetch()
                }}
              />
            ) : null}
            {form.data?.unavailable ? (
              <p className="mt-1 text-[12px] text-ink-faint" data-testid="activity-form-unavailable">
                {form.data.unavailable}
              </p>
            ) : null}
            {form.data && form.data.observations.length > 0 ? (
              <ul className="mt-2 space-y-1" data-testid="activity-form-observations">
                {form.data.observations.map((observation) => (
                  <li
                    key={observation.key}
                    className="flex flex-wrap items-baseline justify-between gap-x-3 text-[12px]"
                  >
                    <span className="text-ink-muted">{observation.label}</span>
                    {observation.value ? (
                      <span className="ml-auto tabular-nums text-ink">
                        {renderedText(observation.value)}
                      </span>
                    ) : (
                      <span className="ml-auto text-right text-ink-faint">
                        {observation.reason}
                      </span>
                    )}
                  </li>
                ))}
              </ul>
            ) : null}
          </section>
        </div>
      </div>

      {/* ----------------------------------------------------- the update */}
      {/*
        The model's own reading of the run, below the app's own facts.

        It is attributed, it carries the checks it passed, and it can be
        dismissed. The app's numbers are above it and they are the app's.
      */}
      <RunUpdate id={id} />

      {/* -------------------------------------------------- against the plan */}
      {data.plan_line ? (
        <section className="card mt-3 py-3">
          <h2 className="caps">
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
      <section id="activity-note" className="card mt-3 py-3" data-testid="activity-note">
        <div className="flex items-center justify-between gap-3">
          <h2 className="caps">
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
