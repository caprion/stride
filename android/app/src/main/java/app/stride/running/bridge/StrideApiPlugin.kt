package app.stride.running.bridge

import android.content.Context
import android.content.Intent
import android.util.Log
import app.stride.running.R
import app.stride.running.core.StrideCore
import app.stride.running.security.AppLock
import app.stride.running.security.CredentialUnavailable
import app.stride.running.security.CredentialUnreadable
import app.stride.running.security.DeviceCredential
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.setup.GarminSetupActivity
import app.stride.running.sync.ForgetGarminKey
import app.stride.running.sync.SyncScheduler
import com.getcapacitor.JSObject
import com.getcapacitor.Plugin
import com.getcapacitor.PluginCall
import com.getcapacitor.PluginMethod
import com.getcapacitor.annotation.CapacitorPlugin
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * The bridge between the web app and the Rust core.
 *
 * **One method.** `apiCall(path, body)` forwards a request to the core and
 * returns the answer. The routing, the authorisation and the SQL all stay
 * in Rust; the Kotlin side never learns what a route is, so a new route
 * needs no change here.
 *
 * The method of the request travels with the body, as `"method"`, so the
 * call has the shape the plan gives it — a path and a body — and a body
 * that does not name a method is a read. A caller that does name a method
 * as a separate argument is understood too, because the old app's bridge
 * had that shape and the two must not drift apart.
 *
 * There is no port and no local server: any app on the device can reach
 * `127.0.0.1`, so a server would hand the Garmin data to every other app on
 * the phone. The plugin bridge has no port at all.
 *
 * The answer is a JSON string rather than an object, because a Capacitor
 * plugin call cannot carry a top-level JSON array, and the core's envelope is
 * the one the app reads either way.
 *
 * ## What the bridge refuses, and why
 *
 * * **A route that is not in the frozen table.** The bridge may call the
 *   routes of the frozen route table and no others. It is not a
 *   general-purpose escape hatch: there is no way to reach a file, a
 *   command or a path that the router does not have.
 * * **The Garmin key.** The routes that would carry it are answered by the
 *   native setup screen and never reach the core — and the decision is made
 *   on the path *without* the query string, so a query string cannot turn
 *   the guard off.
 * * **Real data on a locked screen.** The gate is enforced on the way in,
 *   not by an overlay the page could paint over.
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
        val path = call.getString("path")
        if (path.isNullOrBlank()) {
            call.reject("a request needs a path", "400")
            return
        }
        val body = call.getString("body")
        // The method travels with the body, so the call is `apiCall(path,
        // body)`. A separate "method" argument is accepted too, because
        // that is the shape the old app's bridge had.
        val method = call.getString("method")?.takeIf { it.isNotBlank() }
            ?: methodIn(body)
            ?: "GET"
        // The method is not part of the request the core sees: it is how
        // the request is made, not what it carries.
        val forwardBody = withoutMethod(body)
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

                // **A route that is not in the frozen table is refused.**
                // The bridge is not a general-purpose escape hatch: it may
                // only call the router's routes.
                if (!decision.known) {
                    call.reject(context.getString(R.string.bridge_unknown_route), "404")
                    return@execute
                }

                // **The Garmin key never enters the WebView.**
                //
                // The settings screen has a key field, because the desktop
                // needs one: the desktop's key goes to the desktop's server.
                // On the phone there is no server, so the route is answered
                // with the native setup screen instead, where the key is
                // typed into the app and straight into the Keystore.
                //
                // The screen opens **before** the core is asked for
                // anything: the core's database needs the phone to be
                // unlocked, and the screen is where the athlete unlocks
                // it. A locked phone must still get the one screen that has
                // the key field in it.
                if (decision.carriesKey) {
                    openNativeSetup(context)
                    call.reject(
                        context.getString(R.string.garmin_key_not_in_browser),
                        "400",
                    )
                    return@execute
                }

                // **A locked screen gets no real data.**
                //
                // The check is here, on the way in, so it cannot be turned
                // off by anything the page does.
                if (decision.carriesData && !AppLock.isUnlocked()) {
                    call.reject(context.getString(R.string.bridge_locked), "403")
                    return@execute
                }

                // A phone with no screen lock is a phone anyone can pick up.
                // Real data — pulled runs, imported files — does not land on
                // one.
                //
                // The check is **before** the credential is unwrapped, so a
                // phone that must not write real data never has the key in
                // memory at all.
                if (decision.carriesData && decision.kind != PhoneRoutes.Kind.Data &&
                    !DeviceCredential.hasCredential(context)
                ) {
                    call.reject(context.getString(R.string.screen_lock_required), "403")
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
                var key: String? = null
                when (decision.kind) {
                    PhoneRoutes.Kind.Pull -> {
                        // A pull needs the Garmin key, and the key comes
                        // from the Keystore — never from the page.
                        key = try {
                            Secrets.garminKey(context)
                        } catch (error: CredentialUnavailable) {
                            call.reject(context.getString(R.string.garmin_key_locked), "503")
                            return@execute
                        } catch (error: CredentialUnreadable) {
                            call.reject(context.getString(R.string.garmin_key_gone), "403")
                            return@execute
                        }
                        if (key.isNullOrBlank()) {
                            call.reject(context.getString(R.string.garmin_not_connected_error), "400")
                            return@execute
                        }
                        if (isBackgroundPull(body)) {
                            // A background pull is a job: it survives the app
                            // being killed, and the phone has one scheduler
                            // for it. The job unwraps the key from the
                            // Keystore itself, so nothing about the
                            // credential is in WorkManager's database.
                            //
                            // The page is answered now, with the fact that
                            // the work is queued — the count of runs arrives
                            // by itself, in the notification, when the pull
                            // has landed them.
                            SyncScheduler.pullNow(context, requestedDays(context, body))
                            call.resolve(
                                JSObject().put(
                                    "json",
                                    """{"ok":true,"status":202,"body":{"queued":true}}""",
                                ),
                            )
                            return@execute
                        } else {
                            // A pull the athlete is watching happens now, and
                            // the core needs the key for it.
                            StrideCore.setGarminKey(key)
                        }
                    }

                    else -> Unit
                }

                val isLogout = decision.kind == PhoneRoutes.Kind.Logout

                try {
                    val answer = StrideCore.call(method, path, forwardBody)
                    // The last line of defence: an answer that carries the
                    // key does not reach the page.
                    call.resolve(JSObject().put("json", KeyGuard.redact(answer, key)))
                } finally {
                    if (decision.kind == PhoneRoutes.Kind.Pull) {
                        // The key was in the core's memory for the pull and
                        // no longer: the copy in the Keystore is the one
                        // that lasts, and the phone is still connected.
                        StrideCore.setGarminKey("")
                    }
                    if (isLogout) {
                        // The credential lives in the Keystore, so the
                        // shell is what forgets it. The core's copy goes
                        // with it, and the scheduled pull stops.
                        //
                        // It is the same thing as the native "Forget the
                        // key" button does, in the same place: one place
                        // decides what happens to the Garmin key.
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

    /** The method the body names, if it names one. */
    private fun methodIn(body: String?): String? {
        if (body.isNullOrBlank()) return null
        val parsed = runCatching { JSONObject(body) }.getOrNull() ?: return null
        return parsed.optString("method").takeIf { it.isNotBlank() }
    }

    /** The body without the method, for the core. */
    private fun withoutMethod(body: String?): String? {
        if (body.isNullOrBlank()) return body
        val parsed = runCatching { JSONObject(body) }.getOrNull() ?: return body
        if (!parsed.has("method")) return body
        parsed.remove("method")
        return parsed.toString()
    }

    /** Open the native setup screen. */
    private fun openNativeSetup(context: Context) {
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
    private fun requestedDays(context: Context, body: String?): Int {
        val parsed = body?.let { runCatching { JSONObject(it) }.getOrNull() }
        val days = parsed?.optInt("days", 0) ?: 0
        return if (days > 0) days.coerceIn(1, 90) else AppSettings(context).syncDays
    }
}
