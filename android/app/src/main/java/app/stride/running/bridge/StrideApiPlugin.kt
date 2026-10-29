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
import app.stride.running.R
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
                // What the phone does with this request. The decisions live
                // in PhoneRoutes, which has no Android in it and is unit
                // tested; the side effects live here.
                when (PhoneRoutes.decide(method, path)) {
                    PhoneRoutes.Decision.GarminKeyEntry -> {
                        // The Garmin key never enters the WebView.
                        //
                        // The settings screen has a key field, because the
                        // desktop needs one: the desktop's key goes to the
                        // desktop's server. On the phone there is no server,
                        // so the route is answered with the native setup
                        // screen instead, where the key is typed into the app
                        // and straight into the Keystore.
                        openNativeSetup(context)
                        call.reject(
                            context.getString(R.string.garmin_key_not_in_browser),
                            "400",
                        )
                        return@execute
                    }

                    PhoneRoutes.Decision.GarminLogin -> {
                        // The phone never asks for, and never stores, a
                        // Garmin password.
                        call.reject(
                            context.getString(R.string.garmin_password_not_supported),
                            "501",
                        )
                        return@execute
                    }

                    PhoneRoutes.Decision.GarminDeveloperKey -> {
                        call.reject(
                            context.getString(R.string.garmin_developer_key_not_supported),
                            "501",
                        )
                        return@execute
                    }

                    PhoneRoutes.Decision.GarminPull -> {
                        // A pull needs the Garmin key, and the key comes
                        // from the Keystore — never from the page.
                        val key = Secrets.garminKey(context)
                        if (key == null) {
                            call.reject(
                                context.getString(R.string.garmin_not_connected_error),
                                "400",
                            )
                            return@execute
                        }
                        if (isBackgroundPull(body)) {
                            // A background pull is a job: it survives the app
                            // being killed, and the phone has one scheduler
                            // for it. The job unwraps the key from the
                            // Keystore itself, so nothing about the
                            // credential is in WorkManager's database.
                            SyncScheduler.pullNow(context, requestedDays(context, body))
                        } else {
                            // A pull the athlete is watching happens now, and
                            // the core needs the key for it.
                            StrideCore.setGarminKey(key)
                        }
                    }

                    PhoneRoutes.Decision.Forward -> Unit
                }

                // A phone with no screen lock is a phone anyone can pick up.
                // Real data — pulled runs, imported files — does not land on
                // one.
                val writesRealData =
                    PhoneRoutes.decide(method, path) == PhoneRoutes.Decision.GarminPull ||
                        path == "/api/v1/auth/garmin/fit-import"
                if (writesRealData && !DeviceCredential.hasCredential(context)) {
                    call.reject(context.getString(R.string.screen_lock_required), "403")
                    return@execute
                }
                val isLogout = method.equals("POST", ignoreCase = true) &&
                    path == "/api/v1/auth/garmin/logout"

                try {
                    val answer = StrideCore.call(method, path, body)
                    call.resolve(JSObject().put("json", answer))
                } finally {
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
