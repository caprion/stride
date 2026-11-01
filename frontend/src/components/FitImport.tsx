/**
 * The FIT import.
 *
 * A file from any watch, dropped on the desktop or picked on the phone.
 * The import is **an inline row on the list**, never a modal: the feed
 * stays where it is, and the answer lands in the feed.
 *
 * A file that is already in the log is a duplicate: it adds no row, and
 * it is reported once, in one line — "Already have this run (12 Oct,
 * 10.2 km)".
 */

import { useMutation } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { fileAsBase64 } from '../lib/files'
import { formatDayMonth } from '../lib/format'
import { currentHost } from '../lib/transport'
import { t } from '../i18n'

export function FitImport() {
  const [answer, setAnswer] = useState<string | null>(null)
  const onPhone = currentHost() === 'native'

  const send = useMutation({
    mutationFn: async (file: File) => api.importFit({ fit_base64: await fileAsBase64(file) }),
    onSuccess: (result) => {
      if (result.duplicate && result.date) {
        setAnswer(
          t('runs.duplicate', {
            date: formatDayMonth(result.date),
            distance: result.distance?.text ?? '',
          }),
        )
        return
      }
      setAnswer(t('runs.import.done'))
    },
    onError: () => {
      setAnswer(t('runs.import.failed'))
    },
  })

  const take = (file: File | undefined) => {
    if (!file) return
    setAnswer(null)
    send.mutate(file)
  }

  return (
    <div data-testid="fit-import">
      <label
        onDragOver={(event) => event.preventDefault()}
        onDrop={(event) => {
          event.preventDefault()
          take(event.dataTransfer.files[0])
        }}
        data-testid="fit-drop"
        className={`tap flex cursor-pointer flex-col items-center justify-center rounded-md border border-dashed border-edge bg-surface-soft px-4 text-center ${
          onPhone ? 'py-3' : 'py-4'
        }`}
      >
        <input
          type="file"
          accept=".fit"
          className="hidden"
          data-testid="fit-file"
          onChange={(event) => {
            take(event.target.files?.[0])
            event.target.value = ''
          }}
        />
        {onPhone ? null : (
          <>
            <p className="text-[13.5px] text-ink-soft">{t('runs.import.drop')}</p>
            <p className="mt-0.5 text-[12.5px] text-ink-faint">{t('runs.import.hint')}</p>
          </>
        )}
        <span className="mt-2 text-[14px] font-semibold text-brand-600">
          {send.isPending ? t('runs.importing') : t('runs.import')}
        </span>
      </label>

      {/* The answer, in the list and not in a modal. */}
      {answer ? (
        <p className="mt-2 text-[13.5px] text-ink-soft" data-testid="fit-answer">
          {answer}
        </p>
      ) : null}
    </div>
  )
}
