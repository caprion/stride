/**
 * The run update's own settings.
 *
 * Four things the athlete chooses, and one thing the app promises:
 *
 *   - **the feature's state** — off until it is turned on, and nothing is
 *     written about a run until then;
 *   - **the two models** — the one that writes and the one that reviews,
 *     with the family each belongs to. The reviewer's family has to
 *     differ from the writer's: a reviewer of the writer's own family
 *     repeats the writer's misreadings;
 *   - **the base URLs** — a local one is the loopback address, and a remote
 *     one is a provider that sees the payload;
 *   - **and what would be sent**: the run's own facts, and nothing else.
 *
 * The **key is not here**: the settings name the environment variable it
 * is read from, so it never reaches the database, an export or a log line.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { t } from '../i18n'
import { Field, Notice, SettingRow, SettingsGroup } from '../ui/settings/SettingsGroup'

export function UpdateSettings() {
  const client = useQueryClient()
  const query = useQuery({ queryKey: ['update-settings'], queryFn: api.updateSettings })
  const [writerModel, setWriterModel] = useState<string | null>(null)
  const [writerFamily, setWriterFamily] = useState<string | null>(null)
  const [writerBaseUrl, setWriterBaseUrl] = useState<string | null>(null)
  const [reviewerModel, setReviewerModel] = useState<string | null>(null)
  const [reviewerFamily, setReviewerFamily] = useState<string | null>(null)
  const [reviewerBaseUrl, setReviewerBaseUrl] = useState<string | null>(null)
  const [allowRemote, setAllowRemote] = useState<boolean | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const save = useMutation({
    mutationFn: (change: Parameters<typeof api.saveUpdateSettings>[0]) =>
      api.saveUpdateSettings(change),
    onSuccess: (settings) => {
      client.setQueryData(['update-settings'], settings)
      setNotice(t('settings.update.saved'))
      setWriterModel(null)
      setWriterFamily(null)
      setWriterBaseUrl(null)
      setReviewerModel(null)
      setReviewerFamily(null)
      setReviewerBaseUrl(null)
      setAllowRemote(null)
    },
    onError: () => {
      setNotice(t('settings.update.sameFamily'))
    },
  })

  const state = query.data
  if (query.isPending) {
    return (
      <SettingsGroup title={t('settings.update.heading')} testId="settings-update">
        <p className="text-[13px] text-ink-muted">{t('common.loading')}</p>
      </SettingsGroup>
    )
  }

  const remote = allowRemote ?? state?.allow_remote ?? false
  const writerModelValue = writerModel ?? state?.writer_model ?? ''
  const writerFamilyValue = writerFamily ?? state?.writer_family ?? ''
  const writerBaseUrlValue = writerBaseUrl ?? state?.writer_base_url ?? ''
  const reviewerModelValue = reviewerModel ?? state?.reviewer_model ?? ''
  const reviewerFamilyValue = reviewerFamily ?? state?.reviewer_family ?? ''
  const reviewerBaseUrlValue = reviewerBaseUrl ?? state?.reviewer_base_url ?? ''

  return (
    <SettingsGroup title={t('settings.update.heading')} testId="settings-update">
      <SettingRow
        label={t('settings.update.state')}
        value={state?.enabled ? t('settings.update.on') : t('settings.update.off')}
        testId="settings-update-state"
      />

      <div className="mt-3 grid gap-3">
        <Field
          label={t('settings.update.writer')}
          value={writerModelValue}
          onChange={setWriterModel}
          placeholder={t('settings.update.model')}
          testId="settings-update-writer-model"
        />
        <Field
          label={`${t('settings.update.writer')} — ${t('settings.update.family')}`}
          value={writerFamilyValue}
          onChange={setWriterFamily}
          testId="settings-update-writer-family"
        />
        <Field
          label={`${t('settings.update.writer')} — ${t('settings.update.baseUrl')}`}
          value={writerBaseUrlValue}
          onChange={setWriterBaseUrl}
          testId="settings-update-writer-url"
        />
        <Field
          label={t('settings.update.reviewer')}
          value={reviewerModelValue}
          onChange={setReviewerModel}
          placeholder={t('settings.update.model')}
          testId="settings-update-reviewer-model"
        />
        <Field
          label={`${t('settings.update.reviewer')} — ${t('settings.update.family')}`}
          value={reviewerFamilyValue}
          onChange={setReviewerFamily}
          testId="settings-update-reviewer-family"
        />
        <Field
          label={`${t('settings.update.reviewer')} — ${t('settings.update.baseUrl')}`}
          value={reviewerBaseUrlValue}
          onChange={setReviewerBaseUrl}
          testId="settings-update-reviewer-url"
        />
      </div>

      <label className="mt-3 flex items-start gap-2 text-[13px] text-ink-muted">
        <input
          type="checkbox"
          className="tap mt-0.5"
          checked={remote}
          data-testid="settings-update-remote"
          onChange={(event) => {
            setAllowRemote(event.target.checked)
          }}
        />
        <span>{t('settings.update.allowRemote')}</span>
      </label>

      <div className="mt-3">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
          {t('settings.update.sent')}
        </h3>
        <p className="mt-1 text-[13px] text-ink-muted" data-testid="settings-update-sent">
          {t('settings.update.sent.body')}
        </p>
        {state?.key_env ? (
          <SettingRow
            label={t('settings.update.keyEnv')}
            value={state.key_env}
            testId="settings-update-key-env"
          />
        ) : null}
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-3">
        <button
          type="button"
          className="tap inline-flex items-center rounded-md bg-brand-600 px-4 text-[14px] font-semibold text-white disabled:opacity-50"
          disabled={save.isPending}
          data-testid="settings-update-save"
          onClick={() => {
            setNotice(null)
            save.mutate({
              enabled: state?.enabled ?? false,
              allow_remote: remote,
              writer_model: writerModelValue,
              writer_family: writerFamilyValue,
              writer_base_url: writerBaseUrlValue,
              reviewer_model: reviewerModelValue,
              reviewer_family: reviewerFamilyValue,
              reviewer_base_url: reviewerBaseUrlValue,
            })
          }}
        >
          {t('settings.update.save')}
        </button>
        <button
          type="button"
          className="tap-inline text-[13px] font-medium text-ink-muted underline underline-offset-2 disabled:opacity-50"
          disabled={save.isPending}
          data-testid="settings-update-toggle"
          onClick={() => {
            setNotice(null)
            save.mutate({
              enabled: !(state?.enabled ?? false),
              allow_remote: remote,
              writer_model: writerModelValue,
              writer_family: writerFamilyValue,
              writer_base_url: writerBaseUrlValue,
              reviewer_model: reviewerModelValue,
              reviewer_family: reviewerFamilyValue,
              reviewer_base_url: reviewerBaseUrlValue,
            })
          }}
        >
          {state?.enabled ? t('settings.update.off') : t('settings.update.on')}
        </button>
      </div>
      <Notice text={notice} testId="settings-update-notice" />
    </SettingsGroup>
  )
}
