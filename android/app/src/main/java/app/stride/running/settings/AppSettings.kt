package app.stride.running.settings

import android.content.Context
import androidx.core.content.edit
import app.stride.running.sync.PullWindow

/**
 * The phone's settings.
 *
 * Plain `SharedPreferences`, because none of this is a secret — the window,
 * the interval, whether the phone may talk to Garmin on its own, and the
 * server URL — which is **empty by default**, because the phone is the whole
 * thing and it points at Garmin, not at a server.
 */
class AppSettings(context: Context) {

    private val preferences = context.getSharedPreferences(NAME, Context.MODE_PRIVATE)

    /**
     * The server URL the WebView loads. Empty means "the bundled app".
     *
     * **HTTPS only.** The WebView is the app's only surface, and the origin
     * it loads is the origin the bridge is injected into — so a cleartext
     * target would put the app's own origin on the wire in the clear. The
     * build refuses a cleartext URL for the same reason; this is the
     * runtime half of that rule.
     */
    var serverUrl: String
        get() = preferences.getString(KEY_SERVER_URL, "") ?: ""
        set(value) {
            val trimmed = value.trim()
            if (trimmed.isNotEmpty() && !trimmed.startsWith("https://")) {
                // Refused rather than stored: a setting that is not there is
                // the bundled app, which is the whole app.
                preferences.edit { remove(KEY_SERVER_URL) }
                return
            }
            preferences.edit { putString(KEY_SERVER_URL, trimmed) }
        }

    /** How many days a pull covers. */
    var syncDays: Int
        get() = preferences.getInt(KEY_SYNC_DAYS, PullWindow.DEFAULT_DAYS.toInt())
        set(value) = preferences.edit {
            putInt(
                KEY_SYNC_DAYS,
                value.coerceIn(PullWindow.MIN_DAYS.toInt(), PullWindow.MAX_DAYS.toInt()),
            )
        }

    /** Whether the phone pulls on its own. */
    var autoSync: Boolean
        get() = preferences.getBoolean(KEY_AUTO_SYNC, true)
        set(value) = preferences.edit { putBoolean(KEY_AUTO_SYNC, value) }

    /** How many hours between the pulls. */
    var syncIntervalHours: Int
        get() = preferences.getInt(KEY_SYNC_INTERVAL, 6)
        set(value) = preferences.edit { putInt(KEY_SYNC_INTERVAL, value.coerceIn(1, 24)) }

    /** Whether the phone may use the network at all. */
    var networkEnabled: Boolean
        get() = preferences.getBoolean(KEY_NETWORK, true)
        set(value) = preferences.edit { putBoolean(KEY_NETWORK, value) }

    /**
     * Whether the Garmin setup screen has been dealt with.
     *
     * The screen opens itself on the first launch: an athlete with no key
     * has an app that cannot show their runs, so the one thing the app
     * needs from them is asked for once. This flag is what "once" means.
     *
     * It is set when the athlete connects, and when they dismiss the screen
     * without connecting — being dismissed is a decision too, and it is
     * kept.
     */
    var setupDismissed: Boolean
        get() = preferences.getBoolean(KEY_SETUP_DISMISSED, false)
        set(value) = preferences.edit { putBoolean(KEY_SETUP_DISMISSED, value) }

    /**
     * Whether the native setup screen changed the Garmin connection.
     *
     * The WebView may have read Today while the native screen was in
     * front. This one-bit hand-off tells it to read the local database
     * again when the screen closes. It carries no credential.
     */
    var connectionChanged: Boolean
        get() = preferences.getBoolean(KEY_CONNECTION_CHANGED, false)
        set(value) = preferences.edit { putBoolean(KEY_CONNECTION_CHANGED, value) }

    /** Read and clear the connection-changed hand-off. */
    fun consumeConnectionChanged(): Boolean {
        val changed = connectionChanged
        if (changed) preferences.edit { remove(KEY_CONNECTION_CHANGED) }
        return changed
    }

    /**
     * When the next periodic pull is scheduled, in epoch milliseconds.
     *
     * WorkManager does not promise an exact second, but the phone does name
     * the local boundary it scheduled around when it reports a failure.
     */
    var nextPullAtMillis: Long?
        get() {
            val value = preferences.getLong(KEY_NEXT_PULL_AT, NO_NEXT_PULL)
            return if (value == NO_NEXT_PULL) null else value
        }
        set(value) = preferences.edit {
            if (value == null) {
                remove(KEY_NEXT_PULL_AT)
            } else {
                putLong(KEY_NEXT_PULL_AT, value)
            }
        }

    companion object {
        private const val NO_NEXT_PULL = -1L
        private const val NAME = "stride.settings"
        private const val KEY_SERVER_URL = "server_url"
        private const val KEY_SYNC_DAYS = "sync_days"
        private const val KEY_AUTO_SYNC = "auto_sync"
        private const val KEY_SYNC_INTERVAL = "sync_interval_hours"
        private const val KEY_NETWORK = "network_enabled"
        private const val KEY_SETUP_DISMISSED = "setup_dismissed"
        private const val KEY_NEXT_PULL_AT = "next_pull_at"
        private const val KEY_CONNECTION_CHANGED = "connection_changed"
    }
}
