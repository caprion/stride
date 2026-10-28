package app.stride.running.settings

import android.content.Context

/**
 * The phone's settings.
 *
 * Plain `SharedPreferences`, because none of this is a secret: the window, the
 * interval, whether the phone may talk to Garmin on its own, and the server
 * URL — which is **empty by default**, because the phone is the whole thing
 * and it points at Garmin, not at a server.
 */
class AppSettings(context: Context) {

    private val preferences = context.getSharedPreferences(NAME, Context.MODE_PRIVATE)

    /** The server URL the WebView loads. Empty means "the bundled app". */
    var serverUrl: String
        get() = preferences.getString(KEY_SERVER_URL, "") ?: ""
        set(value) = preferences.edit().putString(KEY_SERVER_URL, value.trim()).apply()

    /** How many days a pull covers. */
    var syncDays: Int
        get() = preferences.getInt(KEY_SYNC_DAYS, 30)
        set(value) = preferences.edit().putInt(KEY_SYNC_DAYS, value.coerceIn(1, 90)).apply()

    /** Whether the phone pulls on its own. */
    var autoSync: Boolean
        get() = preferences.getBoolean(KEY_AUTO_SYNC, true)
        set(value) = preferences.edit().putBoolean(KEY_AUTO_SYNC, value).apply()

    /** How many hours between the pulls. */
    var syncIntervalHours: Int
        get() = preferences.getInt(KEY_SYNC_INTERVAL, 6)
        set(value) = preferences.edit().putInt(KEY_SYNC_INTERVAL, value.coerceIn(1, 24)).apply()

    /** Whether the phone may use the network at all. */
    var networkEnabled: Boolean
        get() = preferences.getBoolean(KEY_NETWORK, true)
        set(value) = preferences.edit().putBoolean(KEY_NETWORK, value).apply()

    companion object {
        private const val NAME = "stride.settings"
        private const val KEY_SERVER_URL = "server_url"
        private const val KEY_SYNC_DAYS = "sync_days"
        private const val KEY_AUTO_SYNC = "auto_sync"
        private const val KEY_SYNC_INTERVAL = "sync_interval_hours"
        private const val KEY_NETWORK = "network_enabled"
    }
}
