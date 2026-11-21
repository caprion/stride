/**
 * The flyby, on the run's own page.
 *
 * **The run detail's map is the flyby.** It is the same map the film page
 * draws — the track coloured by the pace, the marker on the runner, the
 * camera following — and the playhead is driven by the run's own samples:
 * the seconds the watch recorded, and the distance it recorded at each of
 * them.
 *
 * The athlete opens a run and the run is already moving: there is no
 * second screen to choose between the map and the film. The film is still
 * there — it is the deep view, the ground profile and the scrubbing — and
 * the run detail links to it, the way it always has.
 *
 * The street tiles are behind an explicit action, exactly as they are on
 * the film: the athlete asks for the OpenStreetMap tiles, and the app
 * says what they reveal.
 */

import { t } from '../i18n'
import { FlybyMap } from './FlybyMap'
import { useFilmPlayback } from './useFilmPlayback'
import type { GeoPoint } from './RouteMap'
import type { FilmSample } from '../lib/types'

/** The flyby, and the run it is about. */
export function RunFlyby({
  points,
  samples,
}: {
  points: readonly GeoPoint[]
  samples: FilmSample[]
}) {
  /** The run's own length: the last sample's elapsed time. */
  const duration = samples[samples.length - 1]?.elapsed.raw ?? 0

  /** The film's own playback, shared with the film page. */
  const { playhead, playing, toggle } = useFilmPlayback(duration)

  /**
   * Whether there is anything to play.
   *
   * A run with no file has no samples, and so no series for the camera to
   * follow: the map is still the run's own route, but there is no play
   * control that would do nothing.
   */
  const playable = samples.length > 0

  return (
    <div data-testid="activity-flyby">
      {playable ? (
        <div className="mb-2 flex flex-wrap items-center gap-3">
          <button
            type="button"
            className="tap inline-flex items-center rounded-md border border-edge bg-surface px-3 text-[13px] font-semibold text-brand-700"
            onClick={toggle}
            data-testid="activity-flyby-play"
            aria-pressed={playing}
          >
            {playing ? t('film.pause') : t('film.play')}
          </button>
          <span className="text-[12px] text-ink-muted">{t('film.speed')}</span>
        </div>
      ) : null}
      <FlybyMap points={points} samples={samples} playhead={playhead} />
    </div>
  )
}
