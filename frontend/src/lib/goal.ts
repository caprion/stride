/**
 * The athlete's own settings that are not Garmin data: an optional weekly
 * goal and a pause.
 *
 * They live in the WebView's local storage, on the device, and never in the
 * database: the schema is frozen (a desktop export must open on the phone),
 * and neither of these is training data. They are the two inputs v1 allows
 * itself beside reading Garmin, because both change what the app should say:
 *
 * * **the weekly goal** replaces the old hard-coded 50 km. With no goal set,
 *   the bar measures against the athlete's usual week, worked out from their
 *   own runs, and says so;
 * * **a pause** (travel, sick, injured, a break) silences nudges and freezes
 *   the consistency streak instead of letting it break.
 */

import { useCallback, useEffect, useState } from 'react'

const GOAL_KEY = 'stride.weeklyGoalKm'
const PAUSE_KEY = 'stride.pause'
const EVENT = 'stride-settings'

export type PauseReason = 'travel' | 'sick' | 'injured' | 'break'

export interface Pause {
  reason: PauseReason
  /** The last paused day, `YYYY-MM-DD`. */
  until: string
}

function read<T>(key: string): T | null {
  try {
    const raw = window.localStorage.getItem(key)
    return raw ? (JSON.parse(raw) as T) : null
  } catch {
    return null
  }
}

function write(key: string, value: unknown) {
  try {
    if (value == null) window.localStorage.removeItem(key)
    else window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Storage can be unavailable (private mode); the setting just doesn't stick.
  }
  window.dispatchEvent(new Event(EVENT))
}

function useStored<T>(key: string): [T | null, (value: T | null) => void] {
  const [value, setValue] = useState<T | null>(() => read<T>(key))
  useEffect(() => {
    const sync = () => setValue(read<T>(key))
    window.addEventListener(EVENT, sync)
    window.addEventListener('storage', sync)
    return () => {
      window.removeEventListener(EVENT, sync)
      window.removeEventListener('storage', sync)
    }
  }, [key])
  const set = useCallback((next: T | null) => write(key, next), [key])
  return [value, set]
}

/** The optional weekly goal, in kilometres. */
export function useWeeklyGoal(): [number | null, (km: number | null) => void] {
  const [goal, setGoal] = useStored<number>(GOAL_KEY)
  const valid = typeof goal === 'number' && goal > 0 && goal < 1000 ? goal : null
  return [valid, setGoal]
}

/** The current pause, or `null` when none is active today. */
export function usePause(): [Pause | null, (pause: Pause | null) => void] {
  const [pause, setPause] = useStored<Pause>(PAUSE_KEY)
  const today = new Date().toISOString().slice(0, 10)
  const active = pause && pause.until >= today ? pause : null
  return [active, setPause]
}

export const PAUSE_LABEL: Record<PauseReason, string> = {
  travel: 'Travelling',
  sick: 'Sick',
  injured: 'Injured',
  break: 'Taking a break',
}

/**
 * The weekly target the bars measure against: the athlete's goal when set,
 * otherwise their usual week. `null` when there is neither.
 */
export function weeklyTarget(
  goalKm: number | null,
  usualM: number | null | undefined,
): { km: number; source: 'goal' | 'usual' } | null {
  if (goalKm) return { km: goalKm, source: 'goal' }
  if (usualM && usualM > 0) return { km: Math.round(usualM / 100) / 10, source: 'usual' }
  return null
}
