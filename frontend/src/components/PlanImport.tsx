/**
 * The plan import.
 *
 * A plan file — markdown, CSV, JSON or Garmin's own CSV export — becomes
 * a plan block. The file that cannot be read is refused: an unrecognised
 * session is unclassified, never labelled "easy" by guesswork.
 */

import { useMutation } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../lib/api'
import { fileAsBase64 } from '../lib/files'
import { t } from '../i18n'

export function PlanImport() {
  const [answer, setAnswer] = useState<string | null>(null)

  const send = useMutation({
    mutationFn: async (file: File) =>
      api.planImport({ file: await fileAsBase64(file), name: file.name }),
    onSuccess: (result) => {
      setAnswer(result.imported ? t('plan.import.done') : t('runs.import.failed'))
    },
    onError: () => {
      setAnswer(t('runs.import.failed'))
    },
  })

  return (
    <div data-testid="plan-import">
      <label className="tap flex cursor-pointer items-center justify-center rounded-md border border-edge bg-surface px-4 text-[14px] font-semibold text-ink-soft">
        <input
          type="file"
          accept=".md,.csv,.json,.txt"
          className="hidden"
          data-testid="plan-file"
          onChange={(event) => {
            const file = event.target.files?.[0]
            event.target.value = ''
            if (!file) return
            setAnswer(null)
            send.mutate(file)
          }}
        />
        {send.isPending ? t('runs.importing') : t('plan.import')}
      </label>
      {answer ? (
        <p className="mt-2 text-[13.5px] text-ink-soft" data-testid="plan-import-answer">
          {answer}
        </p>
      ) : null}
    </div>
  )
}
