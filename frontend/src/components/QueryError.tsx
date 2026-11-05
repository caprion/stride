import { t } from '../i18n'

export function QueryError({ retry }: { retry: () => void }) {
  return (
    <section className="card my-3 px-4 py-3" role="alert">
      <p className="text-[14px] font-medium">{t('common.error')}</p>
      <button
        type="button"
        className="tap mt-2 rounded-md border border-edge px-3 text-[13px]"
        onClick={retry}
      >
        {t('today.state.retry')}
      </button>
    </section>
  )
}
