package app.stride.running

import android.content.Context
import android.os.Bundle
import android.util.Log
import app.stride.running.bridge.StrideApiPlugin
import app.stride.running.core.StrideCore
import app.stride.running.security.AppGate
import app.stride.running.settings.AppSettings
import app.stride.running.sync.SyncNotifications
import app.stride.running.sync.SyncScheduler
import com.getcapacitor.BridgeActivity
import com.getcapacitor.CapConfig
import org.json.JSONObject
import java.io.File

/**
 * The app.
 *
 * The WebView loads the **bundled** app — `file:///android_asset/public` —
 * and the phone talks to no server. The one thing that can change at runtime
 * is the server URL, and it is empty by default: the app is the whole thing,
 * and it points at Garmin, not at a server.
 *
 * The Rust core is opened here, once, so the first bridge call does not have
 * to.
 */
class MainActivity : BridgeActivity() {

    /**
     * The app gate.
     *
     * The phone holds a year of health data; a phone left unlocked in a bag
     * must not show it. The gate asks the device for its credential when the
     * app opens and after it has been in the background for a minute.
     */
    private val gate by lazy { AppGate(this) }

    override fun onCreate(savedInstanceState: Bundle?) {
        // The plugin has to be registered before the bridge is built.
        registerPlugin(StrideApiPlugin::class.java)

        // The server URL is a runtime setting. When it is empty — the default
        // — the bundled app is loaded and the phone talks to no server.
        val url = AppSettings(this).serverUrl
        if (url.isNotEmpty()) {
            config = CapConfig.loadFromFile(this, writeConfig(this, url))
        }

        super.onCreate(savedInstanceState)

        SyncNotifications.ensureChannel(this)

        // The gate goes up before anything is shown.
        gate.lock()

        // The core is opened in the background: the Keystore needs the phone
        // to be unlocked, and the WebView must not wait for it.
        Thread({
            val problem = StrideCore.ensureOpen(applicationContext)
            if (problem != null) {
                Log.i("Stride", "the core is not open: $problem")
                return@Thread
            }
            SyncScheduler.schedule(applicationContext)
        }, "stride-open").apply { isDaemon = true }.start()
    }

    override fun onStart() {
        super.onStart()
        gate.onForeground()
    }

    override fun onStop() {
        gate.onBackground()
        super.onStop()
    }

    /**
     * A copy of the packaged configuration with the server URL set.
     *
     * The packaged configuration is the shipped one; the copy is what the
     * phone reads, so the setting survives a restart without the APK being
     * rebuilt.
     */
    private fun writeConfig(context: Context, url: String): String {
        val target = File(context.filesDir, "capacitor.config.json")
        val json = try {
            val text = context.assets.open("capacitor.config.json")
                .bufferedReader()
                .use { it.readText() }
            JSONObject(text)
        } catch (error: Exception) {
            Log.w("Stride", "the packaged configuration could not be read: ${error.message}")
            JSONObject()
        }
        val server = json.optJSONObject("server") ?: JSONObject()
        server.put("url", url)
        json.put("server", server)
        target.writeText(json.toString())
        return target.absolutePath
    }
}
