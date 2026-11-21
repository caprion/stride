/**
 * The film's playback: the playhead, and the hand on it.
 *
 * **One playhead, everywhere the run is played.** The film page and the
 * run detail's flyby play the same run the same way — sixty times its
 * own time — and this hook is where that lives, so the two cannot drift
 * apart: the camera on the run's own page follows the runner exactly as
 * the film page's does.
 *
 * The playhead is the run's own time in seconds: the seconds the watch
 * recorded, which are the numbers the core rendered. Nothing here rounds
 * a time or invents one.
 */

import { useCallback, useEffect, useRef, useState } from 'react'

/**
 * How much faster the film plays than the run happened.
 *
 * A run is an hour; the athlete wants to see the shape of it, not to sit
 * through it again. The speed is the film's own and it is said on the
 * screen.
 */
export const FILM_SPEED = 60

/** The playhead, and the hand on it. */
export interface FilmPlayback {
  /** Where the playhead is, in the run's own seconds. */
  playhead: number
  /** Whether the film is playing. */
  playing: boolean
  /** Move the playhead, and stop the film where the athlete left it. */
  scrub: (seconds: number) => void
  /** Play, or pause where the film is. */
  toggle: () => void
}

/**
 * The playback of one run.
 *
 * The duration is the run's own: the last sample's elapsed time.
 */
export function useFilmPlayback(duration: number): FilmPlayback {
  const [playhead, setPlayhead] = useState(0)
  const [playing, setPlaying] = useState(false)
  /** The playhead the animation reads, so the frame never waits for a
   * render to catch up. */
  const position = useRef(0)
  const frame = useRef(0)

  /** Move the playhead, and stop the film where the athlete left it. */
  const scrub = useCallback((seconds: number) => {
    setPlaying(false)
    position.current = seconds
    setPlayhead(seconds)
  }, [])

  /** The film, playing at sixty times the run's own time. */
  useEffect(() => {
    if (!playing || duration <= 0) return
    let last = performance.now()
    const tick = (now: number) => {
      const delta = (now - last) / 1000
      last = now
      const next = position.current + delta * FILM_SPEED
      if (next >= duration) {
        position.current = duration
        setPlayhead(duration)
        setPlaying(false)
        return
      }
      position.current = next
      setPlayhead(next)
      frame.current = requestAnimationFrame(tick)
    }
    frame.current = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame.current)
  }, [playing, duration])

  /** Play, or pause where the film is. */
  const toggle = useCallback(() => {
    if (playing) {
      setPlaying(false)
      return
    }
    // A film that has run its course starts again.
    if (position.current >= duration) {
      position.current = 0
      setPlayhead(0)
    }
    setPlaying(true)
  }, [playing, duration])

  return { playhead, playing, scrub, toggle }
}
