/**
 * The run film: one run, playable.
 *
 * The pace, the heart rate and the ground on one axis, the athlete's own
 * zones behind them, and the route beside it with a camera that follows
 * the runner. The readout says where you are.
 *
 * **It is reached by an explicit action**, from the run detail, the way
 * the Street map and the heatmap are reached: the film is a deep dive into
 * one run, not a destination of its own, and the four destinations stay the
 * four.
 *
 * **Nothing leaves the device.** The route is drawn with the same tiles the
 * run detail's Street map uses, with the same disclosure beside it, and
 * the run's own series never leaves the machine.
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { formatDateMedium, renderedText } from '../lib/format'
import { t } from '../i18n'
import { FlybyMap } from '../components/FlybyMap'
import { QueryError } from '../components/QueryError'
import { RunFilm } from '../components/RunFilm'
import { useFilmPlayback } from '../components/useFilmPlayback'
import { PageHeader } from '../ui/layout/PageHeader'

/** The film, and the run it is about. */
export function FilmPage({ id }: { id: string }) {
  const run = useQuery({ queryKey: ['activity', id], queryFn: () => api.activity(id) })
  const film = useQuery({ queryKey: ['activity-film', id], queryFn: () => api.runFilm(id) })

  const duration = film.data?.samples[film.data.samples.length - 1]?.elapsed.raw ?? 0

  /** The film's own playback: the playhead, and the hand on it. */
  const { playhead, playing, scrub, toggle } = useFilmPlayback(duration)

  const back = (
    <Link to={`/runs/${id}`} className="tap-inline mb-2 text-[13px] text-brand-700 underline">
      {t('film.back')}
    </Link>
  )

  if (film.isError && !film.data) {
    return (
      <>
        {back}
        <PageHeader title={t('film.heading')} />
        <QueryError
          retry={() => {
            void film.refetch()
          }}
        />
      </>
    )
  }

  if (!film.data) {
    return (
      <div>
        {back}
        <PageHeader title={t('film.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  const data = film.data
  const empty = Boolean(data.unavailable) || data.samples.length === 0
  const meta = run.data
    ? `${run.data.name} · ${formatDateMedium(run.data.local_date)} · ${renderedText(run.data.distance)} · ${renderedText(run.data.duration)}`
    : undefined

  return (
    <div>
      {back}
      <PageHeader title={t('film.heading')} meta={meta} />

      {run.isError ? (
        <QueryError
          retry={() => {
            void run.refetch()
          }}
        />
      ) : null}

      {empty ? (
        <section className="card py-6" data-testid="film-empty">
          <p className="text-[15px] font-semibold text-ink">{t('film.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted" data-testid="film-empty-reason">
            {data.unavailable ?? t('film.empty.body')}
          </p>
        </section>
      ) : (
        <>
          <div className="film-layout">
            <section className="card py-3" data-testid="film">
              {run.data?.name ? (
                <h2 className="caps">
                  {run.data.name}
                </h2>
              ) : null}
              <RunFilm
                film={data}
                playhead={playhead}
                playing={playing}
                onScrub={scrub}
                onToggle={toggle}
              />
              {data.zones_unavailable ? (
                <p className="mt-2 text-[12px] text-ink-faint" data-testid="film-zones-unavailable">
                  {data.zones_unavailable}
                </p>
              ) : null}
            </section>

            <section className="card py-3" data-testid="film-route">
              <div className="flex flex-wrap items-baseline justify-between gap-2">
                <h2 className="caps">
                  {t('film.route.heading')}
                </h2>
                <span className="text-[12px] text-ink-muted">{t('film.route.coloured')}</span>
              </div>
              <div className="mt-2">
                <FlybyMap
                  points={run.data?.geo_points ?? []}
                  samples={data.samples}
                  playhead={playhead}
                />
              </div>
            </section>
          </div>

          {/* ------------------------------------------------- the splits */}
          {data.splits.length > 0 ? (
            <section className="card mt-3 py-3" data-testid="film-splits">
              <h2 className="caps">
                {t('film.splits.heading')}
              </h2>
              <table className="mt-2 w-full table-fixed text-[12px]">
                <thead>
                  <tr className="border-b border-edge text-ink-muted">
                    <th scope="col" className="w-14 py-2 text-left font-medium">
                      {t('film.splits.column')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.distance')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.pace')}
                    </th>
                    <th scope="col" className="py-2 text-right font-medium">
                      {t('vital.heartRate')}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {data.splits.map((split) => (
                    <tr key={split.lap_index} className="border-b border-edge last:border-b-0">
                      <th scope="row" className="py-1 text-left font-medium text-ink-soft">
                        <button
                          type="button"
                          className="tap inline-flex items-center justify-center rounded-md border border-edge bg-surface px-2 text-[12px] font-semibold text-brand-700"
                          aria-label={t('film.splits.jump')}
                          data-testid={`film-split-${split.lap_index}`}
                          onClick={() => {
                            scrub(split.offset_s)
                          }}
                        >
                          {split.lap_index}
                        </button>
                      </th>
                      <td className="py-2 text-right tabular-nums text-ink">
                        {renderedText(split.distance)}
                      </td>
                      <td className="py-2 text-right tabular-nums text-ink-soft">
                        {renderedText(split.pace)}
                      </td>
                      <td className="py-2 text-right tabular-nums text-ink-muted">
                        {renderedText(split.heart_rate)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </section>
          ) : null}
        </>
      )}
    </div>
  )
}
