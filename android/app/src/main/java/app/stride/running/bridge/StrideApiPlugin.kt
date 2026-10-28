package app.stride.running.bridge

import android.util.Log
import com.getcapacitor.JSObject
import com.getcapacitor.Plugin
import com.getcapacitor.PluginCall
import com.getcapacitor.PluginMethod
import com.getcapacitor.annotation.CapacitorPlugin
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
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
                val needsTheKey = method.equals("POST", ignoreCase = true) &&
                    path.startsWith("/api/v1/sync/")
                val key = if (needsTheKey) Secrets.garminKey(context) else null
                if (needsTheKey && key == null) {
                    call.reject(
                        "Garmin is not connected on this phone.",
                        "400",
                    )
                    return@execute
                }
                if (key != null) StrideCore.setGarminKey(key)

                try {
                    val answer = StrideCore.call(method, path, body)
                    call.resolve(JSObject().put("json", answer))
                } finally {
                    if (key != null) StrideCore.setGarminKey("")
                }
            } catch (error: Throwable) {
                Log.w("StrideApi", "the request failed: ${error.message}")
                call.reject(error.message ?: "the request could not be completed", "500")
            }
        }
    }
}
