package app.stride.running.bridge

import android.content.Intent
import android.util.Log
import app.stride.running.setup.GarminSetupActivity
import com.getcapacitor.JSObject
import com.getcapacitor.Plugin
import com.getcapacitor.PluginCall
import com.getcapacitor.PluginMethod
import com.getcapacitor.annotation.CapacitorPlugin
import app.stride.running.core.StrideCore
import app.stride.running.security.DeviceCredential
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.sync.SyncScheduler
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * The bridge between the web app and the Rust core.
 *
 * **One method.** `apiCall(method, path, body)` forwards a request to the
 * core and returns the answer. The routing, the authorisation and the SQL
 * all stay in Rust; the Kotlin side never learns what a route is, so a new
 * route needs no change here.
 *
 * There is no port and no local server: any app on the device can reach
 * `127.0.0.1`, so a server would hand the Garmin data to every other app on
 * the phone. The plugin bridge has no port at all.
 *
 * The answer is a JSON string rather than an object, because a Capacitor
 * plugin call cannot carry a top-level JSON array, and the core's envelope is
 * the one the SPA reads either way.
 */
@CapacitorPlugin(name = "StrideApi")
class StrideApiPlugin : Plugin() {

    private val worker = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "stride-bridge").apply { isDaemon = true }
    }

    /**
     * Call a route in the Rust core.
     *
     * The work happens off the WebView's thread: a Garmin pull takes longer
     * than a frame, and the WebView must stay responsive while it happens.
     */
    @PluginMethod
    fun apiCall(call: PluginCall) {
        val method = call.getString("method")
        val path = call.getString("path")
        if (method.isNullOrBlank() || path.isNullOrBlank()) {
            call.reject("a request needs a method and a path", "400")
            return
        }
        val body = call.getString("body")
        val context = context
        if (context == null) {
            call.reject("the app has no context", "500")
            return
        }

        worker.execute {
            try {
                val problem = StrideCore.ensureOpen(context)
                if (problem != null) {
                    call.reject(problem, "503")
                    return@execute
                }

                // The Garmin key is needed for a pull and for nothing else.
                // The Kotlin side unwraps it from the Keystore for the length
                // of the call and hands it over; the core drops it again as
                // soon as the call is over.
                // The routes the phone does not have.
                //
                // The settings screen offers the desktop's three ways to
                // connect: a Personal API key, a Garmin account with a
                // password, and a developer key. The phone has exactly one of
                // them — the Personal API key — and it never stores a Garmin
                // password. So the other two are refused here, in the app,
                // before the password ever leaves the page.
                phoneOnlyRefusal(method, path)?.let { refusal ->
                    call.reject(refusal, "501")
                    return@execute
                }

                // The Garmin key never enters the WebView.
                //
                // The settings screen has a key field, because the desktop
                // needs one: the desktop's key goes to the desktop's server.
                // On the phone there is no server, so the field has no
                // business existing — and the route that would carry the key
                // is intercepted here and answered with the native setup
                // screen instead, where the key is typed into the app and
                // straight into the Keystore.
                if (isKeyEntry(method, path)) {
                    openNativeSetup(context)
                    call.reject(
                        "The Garmin key is entered in the Stride app, not in the browser. " +
                            "The setup screen has opened.",
                        "400",
                    )
                    return@execute
                }

                val isPull = method.equals("POST", ignoreCase = true) &&
                    path.startsWith("/api/v1/sync/")

                // A phone with no screen lock is a phone anyone can pick up.
                // Real data — pulled runs, imported files — does not land on
                // one.
                val writesRealData = isPull ||
                    path == "/api/v1/auth/garmin/fit-import"
                if (writesRealData && !DeviceCredential.hasCredential(context)) {
                    call.reject(
                        "Set a screen lock on this phone first. Stride will not keep your " +
                            "runs and your health data on a phone anyone can pick up.",
                        "403",
                    )
                    return@execute
                }
                var key: String? = null
                if (isPull) {
                    key = Secrets.garminKey(context)
                    if (key == null) {
                        call.reject(
                            "Garmin is not connected on this phone.",
                            "400",
                        )
                        return@execute
                    }
                    if (isBackgroundPull(body)) {
                        // A background pull is a job: it survives the app
                        // being killed, and the phone has one scheduler for
                        // it. The job unwraps the key from the Keystore
                        // itself, so nothing about the credential is in
                        // WorkManager's database.
                        SyncScheduler.pullNow(context, requestedDays(context, body))
                    } else {
                        // A pull the athlete is watching happens now, and
                        // the core needs the key for it.
                        StrideCore.setGarminKey(key)
                    }
                }

                val isLogout = method.equals("POST", ignoreCase = true) &&
                    path.startsWith("/api/v1/auth/garmin/logout")

                try {
                    val answer = StrideCore.call(method, path, body)
                    call.resolve(JSObject().put("json", answer))
                } finally {
                    if (key != null) StrideCore.setGarminKey("")
                    if (isLogout) {
                        // The credential lives in the Keystore, so the
                        // shell is what forgets it. The core's copy goes
                        // with it, and the scheduled pull stops.
                        StrideCore.setGarminKey("")
                        Secrets.forgetGarminKey(context)
                        SyncScheduler.cancel(context)
                    }
                }
            } catch (error: Throwable) {
                Log.w("StrideApi", "the request failed: ${error.message}")
                call.reject(error.message ?: "the request could not be completed", "500")
            }
        }
    }

    /**
     * The answer for a route the phone does not have.
     *
     * The phone connects with a Personal API key and nothing else: no
     * password, no OAuth client, no consumer secret. The routes that would
     * carry one are refused with the reason, so the athlete is not left
     * typing a password into a form that was never going to work.
     */
    private fun phoneOnlyRefusal(method: String, path: String): String? {
        if (!method.equals("POST", ignoreCase = true)) return null
        return when (path) {
            "/api/v1/auth/garmin/login",
            "/api/v1/auth/garmin/mfa" ->
                "This phone connects with a Personal API key. It never asks for, and never " +
                    "stores, your Garmin password."
            "/api/v1/auth/garmin/developer-key",
            "/api/v1/auth/garmin/exchange" ->
                "The phone connects with a Personal API key. There is no developer key here, " +
                    "and no OAuth client to use one with."
            else -> null
        }
    }

    /**
     * Whether the request is one that would carry the Garmin key.
     *
     * Both the connect route and the test route carry it in the body, and
     * the body comes from the page. The phone answers neither.
     */
    private fun isKeyEntry(method: String, path: String): Boolean {
        if (!method.equals("POST", ignoreCase = true)) return false
        return path == "/api/v1/auth/garmin/personal-key" ||
            path == "/api/v1/auth/garmin/personal-key/test"
    }

    /** Open the native setup screen. */
    private fun openNativeSetup(context: android.content.Context) {
        val host = activity
        if (host == null) {
            Log.w("StrideApi", "there is no activity to open the setup screen from")
            return
        }
        host.runOnUiThread {
            try {
                host.startActivity(Intent(host, GarminSetupActivity::class.java))
            } catch (error: Exception) {
                Log.w("StrideApi", "the setup screen could not be opened: ${error.message}")
            }
        }
    }

    /** Whether the request asks for a background pull. */
    private fun isBackgroundPull(body: String?): Boolean {
        val parsed = body?.let { runCatching { JSONObject(it) }.getOrNull() } ?: return true
        return parsed.optBoolean("background", true)
    }

    /** The window the request asks for, or the phone's own. */
    private fun requestedDays(context: android.content.Context, body: String?): Int {
        val parsed = body?.let { runCatching { JSONObject(it) }.getOrNull() }
        val days = parsed?.optInt("days", 0) ?: 0
        return if (days > 0) days.coerceIn(1, 90) else AppSettings(context).syncDays
    }
}
