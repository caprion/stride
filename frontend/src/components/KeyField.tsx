/**
 * The key, and the field it is typed into.
 *
 * **The key never leaves the device it is typed on.** On the desktop the
 * field is here and the key goes to the desktop's own server; on the
 * phone this screen does not exist and the native setup screen owns the
 * key. The bridge refuses the key routes, so there is nothing to send.
 *
 * The field is masked while the key is in it: the key is a credential,
 * and a credential is not something to read back off a shoulder.
 */

import { useMutation } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { t } from '../i18n'

/** What the screen is doing. */
type Phase = 'idle' | 'checking' | 'syncing' | 'connected'

export function KeyField({ onConnected }: { onConnected?: () => void }) {
  const [key, setKey] = useState('')
  const [phase, setPhase] = useState<Phase>('idle')
  const [notice, setNotice] = useState<string | null>(null)

  const test = useMutation({
    mutationFn: () => api.testGarminPersonalKey(key),
    onSuccess: (answer) => {
      setNotice(answer.ok ? t('connect.testOk') : t('connect.testRefused'))
    },
    onError: () => {
      setNotice(t('connect.rejected'))
    },
  })

  const connect = useMutation({
    mutationFn: async () => {
      setPhase('checking')
      await api.setGarminPersonalKey(key)
      setPhase('syncing')
      await api.syncRun()
      return api.syncStatus()
    },
    onSuccess: (status) => {
      setPhase('connected')
      setNotice(t('connect.connected', { n: status.activity_count }))
      onConnected?.()
    },
    onError: () => {
      setPhase('idle')
      setNotice(t('connect.rejected'))
    },
  })

  const busy = test.isPending || connect.isPending
  const blank = key.trim().length === 0

  return (
    <div data-testid="key-field">
      <label className="block">
        <span className="block text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
          {t('connect.key.label')}
        </span>
        <input
          type="password"
          value={key}
          autoComplete="off"
          spellCheck={false}
          onChange={(event) => setKey(event.target.value)}
          data-testid="key-input"
          className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 font-mono text-[14px] text-ink outline-none focus:border-brand-400"
        />
      </label>

      <p className="mt-1.5 text-[13px] text-ink-muted">{t('connect.key.note')}</p>

      <div className="mt-3 flex flex-wrap items-center gap-3">
        <button
          type="button"
          className="tap inline-flex items-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft disabled:opacity-50"
          disabled={busy || blank}
          onClick={() => {
            setNotice(null)
            test.mutate()
          }}
          data-testid="key-test"
        >
          {t('connect.test')}
        </button>
        <button
          type="button"
          className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
          disabled={busy || blank}
          onClick={() => {
            setNotice(null)
            connect.mutate()
          }}
          data-testid="key-connect"
        >
          {phase === 'checking'
            ? t('connect.checking')
            : phase === 'syncing'
              ? t('connect.syncing')
              : t('connect.action')}
        </button>
      </div>

      {notice ? (
        <p className="mt-2 text-[13.5px] text-ink-soft" data-testid="key-notice">
          {notice}
        </p>
      ) : null}
    </div>
  )
}
