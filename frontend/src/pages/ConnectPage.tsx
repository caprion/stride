/**
 * Connect — the first run.
 *
 * **This page is not part of the app.** It stands alone: no chrome, no
 * tabs, no destinations. It is what a stranger sees before the app
 * exists for them, and it disappears the moment there is a key — the app
 * opens on Today instead.
 *
 * It is the **desktop's** page. The phone's first run is a native screen
 * with a native field, because the key never enters a WebView: the
 * phone's bridge refuses the key routes by design, and the phone's own
 * setup screen owns the key. So there is no field here on the phone, and
 * this page is not on the phone at all.
 *
 * "Not now" is an honest answer: the app opens empty, with no invented
 * numbers anywhere.
 */

import { useNavigate } from 'react-router-dom'
import { KeyField } from '../components/KeyField'
import { t } from '../i18n'
import { Brand } from '../ui/layout/Brand'

export function ConnectPage() {
  const navigate = useNavigate()

  return (
    <div className="connect-page">
      <div className="connect-card">
        <div className="mb-5">
          <Brand />
        </div>

        {/* The promise, in one line, before anything else. */}
        <p className="text-[19px] font-semibold leading-snug text-ink" data-testid="connect-promise">
          {t('connect.promise')}
        </p>

        <div className="mt-5">
          <KeyField onConnected={() => navigate('/', { replace: true })} />
        </div>

        <div className="mt-6 flex flex-wrap items-center gap-3">
          <a
            href={t('connect.key.linkUrl')}
            target="_blank"
            rel="noreferrer"
            className="tap-inline text-[13.5px] font-medium text-brand-600 underline underline-offset-2"
            data-testid="connect-link"
          >
            {t('connect.key.link')}
          </a>
          <button
            type="button"
            className="tap-inline text-[13.5px] font-medium text-ink-muted underline underline-offset-2"
            data-testid="connect-not-now"
            onClick={() => navigate('/', { replace: true })}
          >
            {t('connect.notNow')}
          </button>
        </div>
      </div>
    </div>
  )
}
