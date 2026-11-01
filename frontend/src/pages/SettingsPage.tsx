/**
 * Settings.
 *
 * Three inputs change what the app says: the weekly goal, a pause, and
 * the connection. Everything else here is plumbing — the import, the
 * storage and About.
 *
 * **The voice is the hard part of this screen.** Nothing on it tells the
 * athlete what to do with their training, and nothing on it is a verdict.
 * "Delete all data" is a fact about a button, not a command to the
 * athlete; the promise is a fact about the app.
 *
 * Three rules the screen holds:
 *
 *   - **the goal is one number, and a blank is no goal.** The app never
 *     invents a default, and a goal of zero would read as a failure
 *     rather than as the absence of one;
 *   - **the key is never here.** The connection shows the last four
 *     characters of the key and nothing else — and on the phone the
 *     action opens the native setup screen, because the field is the
 *     desktop's alone;
 *   - **the one destructive action says exactly what it deletes**, and it
 *     takes the word.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { connectGarmin } from '../lib/connect'
import { formatDateShort, formatRelative, renderedText } from '../lib/format'
import { currentHost } from '../lib/transport'
import type { PauseReason } from '../lib/types'
import { t } from '../i18n'
import { FitImport } from '../components/FitImport'
import { KeyField } from '../components/KeyField'
import { PlanImport } from '../components/PlanImport'
import { PageHeader } from '../ui/layout/PageHeader'
import {
  Choice,
  Field,
  Notice,
  SettingRow,
  SettingsGroup,
} from '../ui/settings/SettingsGroup'

/** The reasons a pause can have. The athlete's own words for it. */
const REASONS: { value: string; label: string }[] = [
  { value: 'travel', label: t('settings.pause.reason.travel') },
  { value: 'sick', label: t('settings.pause.reason.sick') },
  { value: 'injured', label: t('settings.pause.reason.injured') },
  { value: '', label: t('settings.pause.reason.none') },
]

/** The word the delete takes. */
const CONFIRMATION = t('settings.delete.word')

export function SettingsPage() {
  const settings = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const connection = useQuery({ queryKey: ['auth-status'], queryFn: api.authStatus })
  const health = useQuery({ queryKey: ['health'], queryFn: api.health })
  const client = useQueryClient()

  const [goal, setGoal] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [goalNotice, setGoalNotice] = useState<string | null>(null)
  const [pauseNotice, setPauseNotice] = useState<string | null>(null)
  const [pauseStart, setPauseStart] = useState<string | null>(null)
  const [pauseEnd, setPauseEnd] = useState<string | null>(null)
  const [pauseReason, setPauseReason] = useState<string | null>(null)
  const [showKey, setShowKey] = useState(false)
  const [confirming, setConfirming] = useState(false)
  const [confirmation, setConfirmation] = useState('')

  const save = useMutation({
    mutationFn: (change: Parameters<typeof api.saveSettings>[0]) => api.saveSettings(change),
    onSuccess: (state) => {
      // The api answers with the whole state, so the screen never has to
      // guess what it did with the number it sent.
      client.setQueryData(['settings'], state)
    },
  })

  const exportBackup = useMutation({
    mutationFn: () => api.exportBackup(),
    onSuccess: (result) => {
      setNotice(result.exported ? t('settings.storage.exported') : t('settings.storage.exportFailed'))
    },
  })

  const deleteAll = useMutation({
    mutationFn: () => api.deleteEverything(confirmation),
    onSuccess: (result) => {
      setNotice(result.deleted ? t('settings.storage.deleted') : null)
      setConfirming(false)
      setConfirmation('')
    },
  })

  const state = settings.data
  const link = connection.data
  const version = health.data?.version ?? null

  if (settings.isPending || connection.isPending) {
    return (
      <div>
        <PageHeader title={t('settings.heading')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  if (!state) {
    return (
      <div>
        <PageHeader title={t('settings.heading')} />
        <div className="card px-4 py-6 text-center">
          <p className="text-[14px] text-ink-soft">{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-3 inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft"
            onClick={() => {
              settings.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </div>
      </div>
    )
  }

  // The goal's field: the number the api sent, and what the athlete has
  // typed since. A blank field is no goal.
  const goalSent = state.goal.km === null ? '' : String(state.goal.km)
  const goalValue = goal ?? goalSent
  const goalChanged = goal !== null && goal !== goalSent

  // The pause's fields: the api's pause, until the athlete changes one.
  const startValue = pauseStart ?? state.pause?.start_date ?? ''
  const endValue = pauseEnd ?? state.pause?.end_date ?? ''
  const reasonValue = pauseReason ?? state.pause?.reason ?? ''

  return (
    <div>
      <PageHeader title={t('settings.heading')} />

      <div className="settings-grid space-y-3">
        {/* ------------------------------------------------ the goal */}
        <SettingsGroup title={t('settings.goal.heading')} testId="settings-goal">
          <Field
            label={t('settings.goal.unit')}
            value={goalValue}
            inputMode="decimal"
            onChange={setGoal}
            testId="settings-goal-input"
          />
          <p className="mt-1.5 text-[13px] text-ink-muted">{t('settings.goal.empty')}</p>
          <div className="mt-2 flex flex-wrap items-center gap-3">
            <button
              type="button"
              className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
              disabled={!goalChanged || save.isPending}
              data-testid="settings-goal-save"
              onClick={() => {
                const km = goalValue.trim() === '' ? null : Number(goalValue.trim())
                if (km !== null && !Number.isFinite(km)) return
                setGoalNotice(km === null ? t('settings.goal.cleared') : t('settings.goal.saved'))
                setGoal(null)
                save.mutate({ goal_km: km })
              }}
            >
              {t('settings.goal.save')}
            </button>
            {state.goal.value ? (
              <span className="text-[13px] text-ink-muted" data-testid="settings-goal-value">
                {renderedText(state.goal.value)}
              </span>
            ) : (
              <span className="text-[13px] text-ink-muted" data-testid="settings-goal-value">
                {t('settings.goal.none')}
              </span>
            )}
          </div>
          <Notice text={goalNotice} testId="settings-goal-notice" />
        </SettingsGroup>

        {/* ----------------------------------------------- the pause */}
        <SettingsGroup title={t('settings.pause.heading')} testId="settings-pause">
          {state.pause ? (
            <p className="text-[14px] font-medium text-ink" data-testid="settings-pause-state">
              {state.pause.reason
                ? t('common.paused.untilReason', {
                    date: formatDateShort(state.pause.end_date),
                    reason: t(
                      state.pause.reason === 'travel'
                        ? 'settings.pause.reason.travel'
                        : state.pause.reason === 'sick'
                          ? 'settings.pause.reason.sick'
                          : 'settings.pause.reason.injured',
                    ),
                  })
                : t('common.paused.until', { date: formatDateShort(state.pause.end_date) })}
            </p>
          ) : null}

          <div className="mt-2 settings-pair">
            <Field
              label={t('settings.pause.start')}
              type="date"
              value={startValue}
              onChange={setPauseStart}
              testId="settings-pause-start"
            />
            <Field
              label={t('settings.pause.end')}
              type="date"
              value={endValue}
              onChange={setPauseEnd}
              testId="settings-pause-end"
            />
          </div>
          <div className="mt-3">
            <Choice
              label={t('settings.pause.reason')}
              value={reasonValue}
              options={REASONS}
              onChange={setPauseReason}
              testId="settings-pause-reason"
            />
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-3">
            <button
              type="button"
              className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
              disabled={startValue === '' || endValue === '' || save.isPending}
              data-testid="settings-pause-save"
              onClick={() => {
                setPauseNotice(t('settings.pause.saved'))
                save.mutate({
                  pause: {
                    start_date: startValue,
                    end_date: endValue,
                    reason: reasonValue === '' ? null : (reasonValue as PauseReason),
                  },
                })
                setPauseStart(null)
                setPauseEnd(null)
                setPauseReason(null)
              }}
            >
              {t('settings.pause.save')}
            </button>
            {state.pause ? (
              <button
                type="button"
                className="tap-inline text-[13.5px] font-medium text-ink-muted underline underline-offset-2"
                data-testid="settings-pause-clear"
                onClick={() => {
                  setPauseNotice(t('settings.pause.cleared'))
                  save.mutate({ pause: null })
                  setPauseStart(null)
                  setPauseEnd(null)
                  setPauseReason(null)
                }}
              >
                {t('settings.pause.clear')}
              </button>
            ) : null}
          </div>
          <Notice text={pauseNotice} testId="settings-pause-notice" />
        </SettingsGroup>

        {/* ------------------------------------------ the connection */}
        <SettingsGroup title={t('settings.connection.heading')} testId="settings-connection">
          <SettingRow
            label={t('settings.connection.status')}
            value={link?.connected ? t('settings.connection.connected') : t('settings.connection.notConnected')}
            testId="settings-connection-state"
          />
          <SettingRow
            label={t('settings.connection.key')}
            value={link?.key_last4 ?? '—'}
            testId="settings-connection-key"
          />
          <SettingRow
            label={t('settings.connection.lastSync')}
            value={
              link?.last_sync_at
                ? formatRelative(link.last_sync_at)
                : t('settings.connection.never')
            }
            testId="settings-connection-sync"
          />
          <p className="mt-1 text-[12.5px] text-ink-faint">{t('settings.connection.keyNote')}</p>

          {showKey ? (
            <div className="mt-3">
              <KeyField />
            </div>
          ) : (
            <div className="mt-2">
              <button
                type="button"
                className="tap inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft"
                data-testid="settings-reconnect"
                onClick={() => {
                  // On the phone the key belongs to the native setup
                  // screen, and the bridge refuses the key routes: so the
                  // action opens that screen and nothing else.
                  if (currentHost() === 'native') {
                    void connectGarmin()
                    return
                  }
                  setShowKey(true)
                }}
              >
                {currentHost() === 'native'
                  ? t('settings.connection.setup')
                  : t('settings.connection.reconnect')}
              </button>
            </div>
          )}
        </SettingsGroup>

        {/* ----------------------------------------------- the import */}
        <SettingsGroup title={t('settings.import.heading')} testId="settings-import">
          <p className="mb-2 text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
            {t('settings.import.fit')}
          </p>
          <FitImport />
          <p className="mb-2 mt-4 text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
            {t('settings.import.plan')}
          </p>
          <PlanImport />
        </SettingsGroup>

        {/* -------------------------------------------------- about */}
        <SettingsGroup title={t('settings.about.heading')} testId="settings-about">
          <SettingRow
            label={t('settings.about.version')}
            value={version ?? '—'}
            testId="settings-about-version"
          />
          <SettingRow
            label={t('settings.about.licence')}
            value={t('settings.about.licenceValue')}
            testId="settings-about-licence"
          />
          <SettingRow
            label={t('settings.about.source')}
            value={
              <a
                href={t('settings.about.sourceUrl')}
                target="_blank"
                rel="noreferrer"
                className="underline underline-offset-2"
              >
                {t('settings.about.sourceValue')}
              </a>
            }
            testId="settings-about-source"
          />
          <p className="mt-2 text-[14px] text-ink-soft" data-testid="settings-about-promise">
            {t('common.promise')}
          </p>
        </SettingsGroup>

        {/* ----------------------------------------------- the storage */}
        <SettingsGroup title={t('settings.storage.heading')} testId="settings-storage">
          <SettingRow
            label={t('settings.storage.size')}
            value={renderedText(state.storage.size)}
            testId="settings-storage-size"
          />
          <div className="mt-2 flex flex-wrap items-center gap-3">
            <button
              type="button"
              className="tap inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft disabled:opacity-50"
              disabled={exportBackup.isPending}
              data-testid="settings-export"
              onClick={() => {
                setNotice(null)
                exportBackup.mutate()
              }}
            >
              {t('settings.storage.export')}
            </button>
          </div>

          {/* The one destructive action, and it is last. */}
          <div className="mt-4 border-t border-edge pt-3">
            {confirming ? (
              <div data-testid="settings-delete-confirm">
                <p className="text-[13.5px] text-ink-soft">{t('settings.delete.what')}</p>
                <p className="text-[13.5px] text-ink-soft">{t('settings.delete.undo')}</p>
                <p className="mt-2 text-[13px] text-ink-muted">{t('settings.delete.confirm')}</p>
                <input
                  type="text"
                  value={confirmation}
                  autoComplete="off"
                  spellCheck={false}
                  onChange={(event) => setConfirmation(event.target.value)}
                  data-testid="settings-delete-input"
                  className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 font-mono text-[14px] text-ink outline-none focus:border-brand-400"
                />
                <div className="mt-3 flex flex-wrap items-center gap-3">
                  <button
                    type="button"
                    className="tap inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft disabled:opacity-50"
                    disabled={confirmation !== CONFIRMATION || deleteAll.isPending}
                    data-testid="settings-delete-confirm-button"
                    onClick={() => {
                      deleteAll.mutate()
                    }}
                  >
                    {t('settings.delete.action')}
                  </button>
                  <button
                    type="button"
                    className="tap-inline text-[13.5px] font-medium text-ink-muted underline underline-offset-2"
                    onClick={() => {
                      setConfirming(false)
                      setConfirmation('')
                    }}
                  >
                    {t('common.close')}
                  </button>
                </div>
              </div>
            ) : (
              <button
                type="button"
                className="tap inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft"
                data-testid="settings-delete"
                onClick={() => {
                  setNotice(null)
                  setConfirming(true)
                }}
              >
                {t('settings.storage.delete')}
              </button>
            )}
          </div>
          <Notice text={notice} testId="settings-notice" />
        </SettingsGroup>
      </div>
    </div>
  )
}
