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
import app.stride.running.security.CredentialUnavailable
import app.stride.running.security.CredentialUnreadable
import app.stride.running.security.DeviceCredential
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.sync.ForgetGarminKey
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
                // What the phone does with this request. The decisions live
                // in PhoneRoutes, which has no Android in it and is unit
                // tested; the side effects live here.
                val decision = PhoneRoutes.decide(method, path)

                // A phone with no screen lock is a phone anyone can pick up.
                // Real data — pulled runs, imported files — does not land on
                // one.
                //
                // The check is **here**, before the pull is queued and
                // before the key is handed to the core. It used to sit
                // further down, after both of those had happened, which
                // made the refusal a formality: the pull was already in
                // WorkManager's queue and the key was already in the
                // core's memory by the time the request was turned away.
                if (PhoneRoutes.writesRealData(decision, path) &&
                    !DeviceCredential.hasCredential(context)
                ) {
                    call.reject(context.getString(R.string.screen_lock_required), "403")
                    return@execute
                }

                if (decision == PhoneRoutes.Decision.GarminKeyEntry) {
                    // The Garmin key never enters the WebView.
                    //
                    // The settings screen has a key field, because the
                    // desktop needs one: the desktop's key goes to the
                    // desktop's server. On the phone there is no server,
                    // so the route is answered with the native setup
                    // screen instead, where the key is typed into the app
                    // and straight into the Keystore.
                    //
                    // The screen opens **before** the core is asked for
                    // anything: the core's database needs the phone to be
                    // unlocked, and the screen is where the athlete unlocks
                    // it. A locked phone must still get the one screen that
                    // has the key field in it.
                    openNativeSetup(context)
                    call.reject(
                        context.getString(R.string.garmin_key_not_in_browser),
                        "400",
                    )
                    return@execute
                }

                val problem = StrideCore.ensureOpen(context)
                if (problem != null) {
                    call.reject(problem, "503")
                    return@execute
                }

                // The Garmin key is needed for a pull and for nothing else.
                // The Kotlin side unwraps it from the Keystore for the length
                // of the call and hands it over; the core drops it again as
                // soon as the call is over.
                when (decision) {
                    PhoneRoutes.Decision.GarminKeyEntry -> {
                        // Handled above, before the core was opened.
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
                        val key = try {
                            Secrets.garminKey(context)
                        } catch (error: CredentialUnavailable) {
                            call.reject(
                                context.getString(R.string.garmin_key_locked),
                                "503",
                            )
                            return@execute
                        } catch (error: CredentialUnreadable) {
                            call.reject(
                                context.getString(R.string.garmin_key_gone),
                                "403",
                            )
                            return@execute
                        }
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
                // one. The rule is checked above, before anything was
                // queued or handed over.
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
                        //
                        // It is the same three steps as the "Forget the
                        // key" method, in the same place: one place decides
                        // what happens to the Garmin key.
                        try {
                            ForgetGarminKey.onPhone(context)
                        } catch (error: Throwable) {
                            Log.w("StrideApi", "the key could not be forgotten: ${error.message}")
                        }
                    }
                }
            } catch (error: Throwable) {
                Log.w("StrideApi", "the request failed: ${error.message}")
                call.reject(error.message ?: "the request could not be completed", "500")
            }
        }
    }



    /**
     * **Sync now.**
     *
     * The phone's Settings has a "Sync now" button, and the pull it asks
     * for is the phone's own background job: the key is in the Keystore and
     * the worker unwraps it there. So the page needs nothing for this — not
     * the key, not a server, not a route.
     *
     * It enqueues a **one-time** request — not the periodic one — and
     * resolves as soon as the work is **queued**: the app says "Syncing…"
     * and the count of runs arrives by itself, when the pull has landed
     * them. The promise is "queued", not "done".
     *
     * The answer is about the work, never about the key: the key is not
     * read here, not logged here and not sent back.
     */
    @PluginMethod
    fun syncNow(call: PluginCall) {
        val context = context
        if (context == null) {
            call.reject("the app has no context", "500")
            return
        }
        worker.execute {
            try {
                SyncScheduler.pullNow(context, AppSettings(context).syncDays)
                call.resolve(JSObject().put("queued", true))
            } catch (error: Throwable) {
                Log.w("StrideApi", "the pull could not be queued: ${error.message}")
                call.reject(
                    error.message ?: "the pull could not be queued",
                    "500",
                )
            }
        }
    }

    /**
     * **Forget the key.**
     *
     * The other half of the phone's Settings, and the one destructive
     * action it has.
     *
     * It clears the Keystore-wrapped envelope, stops the scheduled and the
     * queued work, and clears the core's in-memory copy of the key — so the
     * next call cannot use it.
     *
     * **The imported runs stay on the phone.** Forgetting the key is not
     * forgetting the data: the activities are in the phone's own database
     * and nothing here deletes one.
     *
     * It never returns, logs or transmits the key. It is about the key's
     * *existence*, and nothing more.
     */
    @PluginMethod
    fun forgetGarminKey(call: PluginCall) {
        val context = context
        if (context == null) {
            call.reject("the app has no context", "500")
            return
        }
        worker.execute {
            try {
                ForgetGarminKey.onPhone(context)
                call.resolve(JSObject().put("forgotten", true))
            } catch (error: Throwable) {
                Log.w("StrideApi", "the key could not be forgotten: ${error.message}")
                call.reject(
                    error.message ?: "the key could not be forgotten",
                    "500",
                )
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
