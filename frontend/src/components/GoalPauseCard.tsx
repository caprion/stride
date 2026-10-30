/**
 * The two things the athlete can tell Stride: a weekly goal, and a pause.
 *
 * Both stay on this device (see `lib/goal.ts`). Neither is required: with
 * no goal the week is measured against the athlete's usual week, and with
 * no pause the app behaves as it always has.
 */

import { useState } from 'react'
import { PAUSE_LABEL, type PauseReason, usePause, useWeeklyGoal } from '../lib/goal'
import { Button, Card, CardHeader } from './ui'

const REASONS: PauseReason[] = ['travel', 'sick', 'injured', 'break']

function inDays(days: number): string {
  const date = new Date()
  date.setDate(date.getDate() + days)
  return date.toISOString().slice(0, 10)
}

export function GoalPauseCard() {
  const [goal, setGoal] = useWeeklyGoal()
  const [pause, setPause] = usePause()
  const [draft, setDraft] = useState(goal ? String(goal) : '')
  const [reason, setReason] = useState<PauseReason>('travel')
  const [days, setDays] = useState(7)

  return (
    <Card>
      <CardHeader title="Goal and pauses" subtitle="Optional. Both stay on this device." />
      <div className="space-y-5 px-5 pb-5">
        <div>
          <p className="text-[14px] font-medium text-ink">Weekly distance goal</p>
          <p className="mt-0.5 text-[13px] text-ink-muted">
            Leave it empty and the week is measured against your usual week.
          </p>
          <div className="mt-2 flex items-center gap-2">
            <input
              inputMode="decimal"
              value={draft}
              onChange={(event) => setDraft(event.target.value.replace(/[^0-9.]/g, ''))}
              placeholder="e.g. 30"
              className="h-9 w-24 rounded-md border border-edge bg-surface px-3 text-[14px] text-ink"
              aria-label="Weekly goal in km"
            />
            <span className="text-[13px] text-ink-muted">km</span>
            <Button
              size="md"
              variant="secondary"
              onClick={() => {
                const value = Number(draft)
                setGoal(Number.isFinite(value) && value > 0 ? value : null)
              }}
            >
              Save
            </Button>
            {goal ? (
              <button
                type="button"
                className="text-[13px] font-semibold text-brand-600"
                onClick={() => {
                  setGoal(null)
                  setDraft('')
                }}
              >
                Clear
              </button>
            ) : null}
          </div>
        </div>

        <div>
          <p className="text-[14px] font-medium text-ink">Pause</p>
          <p className="mt-0.5 text-[13px] text-ink-muted">
            Travelling, sick or injured? A pause stops nudges and keeps your streak waiting.
          </p>
          {pause ? (
            <div className="mt-2 flex flex-wrap items-center gap-3">
              <p className="text-[14px] text-ink">
                {PAUSE_LABEL[pause.reason]} until {pause.until}
              </p>
              <Button size="md" variant="secondary" onClick={() => setPause(null)}>
                End pause
              </Button>
            </div>
          ) : (
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <select
                value={reason}
                onChange={(event) => setReason(event.target.value as PauseReason)}
                className="h-9 rounded-md border border-edge bg-surface px-2 text-[14px] text-ink"
                aria-label="Why"
              >
                {REASONS.map((value) => (
                  <option key={value} value={value}>
                    {PAUSE_LABEL[value]}
                  </option>
                ))}
              </select>
              <select
                value={days}
                onChange={(event) => setDays(Number(event.target.value))}
                className="h-9 rounded-md border border-edge bg-surface px-2 text-[14px] text-ink"
                aria-label="For how long"
              >
                {[3, 7, 14, 28].map((value) => (
                  <option key={value} value={value}>
                    {value} days
                  </option>
                ))}
              </select>
              <Button
                size="md"
                variant="secondary"
                onClick={() => setPause({ reason, until: inDays(days) })}
              >
                Pause
              </Button>
            </div>
          )}
        </div>
      </div>
    </Card>
  )
}
