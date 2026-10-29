package app.stride.running.core

import android.content.Context
import android.os.Build
import android.util.Log
import app.stride.running.security.CredentialUnavailable
import app.stride.running.security.CredentialUnreadable
import org.json.JSONObject
import uniffi.stride.strideCall
import uniffi.stride.strideGarminSync
import uniffi.stride.strideOpen
import uniffi.stride.strideSetGarminConnected
import uniffi.stride.strideSetGarminKey
import uniffi.stride.strideStatus
import java.io.File

/**
 * The Rust core, in this process.
 *
 * There is no server and no port: the phone loads `libuniffi_stride.so` and
 * calls it through UniFFI. The Kotlin side never learns about a route — it
 * forwards a method, a path and a body, and reads the answer.
 *
 * The database is the phone's own: a SQLCipher file in `noBackupFilesDir`,
 * opened with the key the Keystore protects. The key is handed to the core
 * once, at start-up, and never leaves the process.
 */
object StrideCore {

    private const val TAG = "StrideCore"

    /** The name of the native library the APK packages. */
    private const val LIBRARY = "uniffi_stride"

    /** The database file, inside the storage no backup carries. */
    private const val DATABASE = "stride.db"

    private val lock = Any()

    @Volatile
    private var loaded = false

    @Volatile
    private var open = false

    /** Whether the core has been asked for anything yet. */
    fun isOpen(): Boolean = open

    /**
     * Load the library and open the database.
     *
     * The answer is `null` when the core is ready, and a plain sentence when
     * it is not — the phone says "unlock the phone" or "the database could
     * not be opened" rather than pretending.
     */
    fun ensureOpen(context: Context): String? = synchronized(lock) {
        if (open) return null
        try {
            loadLibrary()
        } catch (error: UnsatisfiedLinkError) {
            return "the Stride core could not be loaded: ${error.message}"
        }

        val database = File(context.noBackupFilesDir, DATABASE).absolutePath
        val key = try {
            app.stride.running.security.Secrets.databaseKeyHex(context)
        } catch (error: CredentialUnavailable) {
            // The phone is locked. The key is fine and the database is where
            // it was; come back when the phone is unlocked.
            Log.i(TAG, "the database key cannot be used while the phone is locked")
            return "the phone is locked. Unlock it and try again — your runs are still here."
        } catch (error: CredentialUnreadable) {
            Log.w(TAG, "the database key is gone: ${error.message}")
            return "this phone's key store was reset, so the local database cannot be opened. " +
                "Reconnect Garmin and import again."
        } catch (error: Exception) {
            return lockedOrBroken(error)
        }

        val answer = try {
            strideOpen(database, key)
        } catch (error: Exception) {
            return "the database could not be opened: ${error.message}"
        }

        val parsed = JSONObject(answer)
        if (parsed.optBoolean("ok")) {
            open = true
            Log.i(TAG, "the core is open on ${parsed.optJSONObject("body")?.optString("db")}")
            return null
        }
        parsed.optJSONObject("error")?.optString("detail")
            ?: "the database could not be opened"
    }

    /**
     * Call a route.
     *
     * The answer is the envelope, as a JSON string:
     *
     * ```text
     * {"ok": true,  "status": 200, "body": { ... }}
     * {"ok": false, "status": 404, "error": {"detail": "..."}}
     * ```
     */
    fun call(method: String, path: String, body: String?): String =
        strideCall(method, path, body)

    /**
     * Pull from Garmin.
     *
     * The key is used for the exchange and then dropped by the caller. The
     * bearer token stays in the core's memory.
     */
    fun garminSync(apiKey: String, days: Long): String = strideGarminSync(apiKey, days)

    /**
     * Put the Garmin key in the core's memory, or take it out again.
     *
     * The key is in memory for the length of one pull and no longer.
     */
    fun setGarminKey(apiKey: String) {
        strideSetGarminKey(apiKey)
    }

    /**
     * Say whether the phone holds a Garmin key.
     *
     * The key itself lives in the Keystore, and the core never keeps it:
     * this is the fact of it, so the app can say "Garmin is connected"
     * without the key being in the process's memory, and without the page
     * ever seeing it.
     */
    fun setGarminConnected(connected: Boolean) {
        strideSetGarminConnected(connected)
    }

    /** What the core is, for the phone's own diagnostics. */
    fun status(): String = strideStatus()

    /** Load the native library, once. */
    private fun loadLibrary() {
        if (loaded) return
        synchronized(lock) {
            if (loaded) return
            // The library is in the APK's native directory. Loading it here
            // puts its symbols in the process, which is what JNA needs to
            // find them on Android.
            System.loadLibrary(LIBRARY)
            loaded = true
        }
    }

    /**
     * The message for a Keystore failure.
     *
     * A locked phone is not a failure: the key is unusable until the phone is
     * unlocked, and the worker defers.
     */
    private fun lockedOrBroken(error: Exception): String {
        val name = error.javaClass.simpleName
        Log.w(TAG, "the Keystore refused the key: $name")
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            "the phone is locked, or the key was invalidated — unlock the phone and try again"
        } else {
            "the Stride key store is not available"
        }
    }
}
