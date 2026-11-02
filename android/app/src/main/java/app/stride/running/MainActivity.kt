package app.stride.running

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import app.stride.running.bridge.StrideApiPlugin
import app.stride.running.core.StrideCore
import app.stride.running.security.AppGate
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.setup.GarminSetupActivity
import app.stride.running.setup.SetupEntry
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

    /**
     * Whether the Garmin setup screen has been opened from here.
     *
     * It is opened once, on the first launch, and never again: not because
     * it was dismissed, and not because the app was restarted.
     */
    private var setupOpened = false

    /**
     * Whether the phone held a Garmin key at the last foreground pass.
     *
     * The WebView may have asked Today for its data while the native setup
     * screen was in front of it. When the key appears or disappears, the
     * page is reloaded once so it reads the local database as it is now.
     */
    private var keyAtLastForeground = false

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

        keyAtLastForeground = hasGarminKey()
        SyncNotifications.ensureChannel(this)

        // The setup screen waits for the gate: it is opened behind the
        // credential, never in front of it. The listener is set first: a
        // phone with no screen lock has nothing to unlock, so the gate
        // opens at once and says so.
        gate.onUnlocked = {
            openSetupIfNeeded()
            reloadIfTheConnectionChanged()
        }

        // The gate goes up before anything is shown.
        gate.lock()

        // The recents surface must not be a picture of the athlete's health.
        //
        // Every screen of this app is health data — runs, splits, heart
        // rates, sleep — so the window is secure: the task switcher shows a
        // blank card, and a screenshot of the app is a blank image. The
        // credential screen sets the same flag for the same reason.
        window.setFlags(
            WindowManager.LayoutParams.FLAG_SECURE,
            WindowManager.LayoutParams.FLAG_SECURE,
        )

        // The core is opened in the background: the Keystore needs the phone
        // to be unlocked, and the WebView must not wait for it.
        Thread({
            val problem = StrideCore.ensureOpen(applicationContext)
            if (problem != null) {
                Log.i("Stride", "the core is not open")
                return@Thread
            }
            // The phone's key lives in the Keystore and the core never keeps
            // it. The core is told the *fact* of it, so the app can say
            // "Garmin is connected" after a restart without the key being in
            // memory.
            val key = try {
                Secrets.garminKey(applicationContext)
            } catch (_: Exception) {
                Log.i("Stride", "the Garmin key could not be read")
                null
            }
            StrideCore.setGarminConnected(!key.isNullOrBlank())
            SyncScheduler.schedule(applicationContext)
            // The WebView may have asked for Today before the core was
            // open. Read the local database once it is.
            runOnUiThread { bridge?.webView?.reload() }
        }, "stride-open").apply { isDaemon = true }.start()
    }

    override fun onStart() {
        super.onStart()
        gate.onForeground()
        if (gate.isUnlocked) openSetupIfNeeded()
    }

    override fun onResume() {
        super.onResume()
        reloadIfTheConnectionChanged()
    }

    override fun onStop() {
        gate.onBackground()
        super.onStop()
    }

    /**
     * Open the Garmin setup screen, once.
     *
     * The app without a key cannot show the athlete their runs, so the one
     * thing the app needs from them is asked for on the first launch: the
     * native screen, where the key is typed into the app and never into the
     * page.
     *
     * Three conditions, and all three have to hold:
     *
     * * there is no key — once there is one, the app has what it needs and
     *   the screen has nothing left to ask for;
     * * the athlete has not dismissed the screen — being dismissed is a
     *   decision, and it is kept;
     * * the screen has not been opened from here already.
     */
    private fun openSetupIfNeeded() {
        if (setupOpened) return
        if (!SetupEntry.shouldOpen(hasGarminKey(), AppSettings(this).setupDismissed)) return

        setupOpened = true
        try {
            startActivity(Intent(this, GarminSetupActivity::class.java))
        } catch (_: Exception) {
            Log.w("Stride", "the setup screen could not be opened")
        }
    }

    /**
     * Reload Today once when setup changed the connection.
     *
     * The setup screen sits in front of the WebView. The page's first read
     * can therefore happen before the first pull has landed. This brings
     * the page back to the foreground state without reloading on every
     * ordinary foreground pass.
     */
    private fun reloadIfTheConnectionChanged() {
        val connected = hasGarminKey()
        val changed = AppSettings(this).consumeConnectionChanged()
        if (!changed && connected == keyAtLastForeground) return
        keyAtLastForeground = connected
        bridge?.webView?.reload()
    }

    /** Whether the phone holds a Garmin key. */
    private fun hasGarminKey(): Boolean = try {
        !Secrets.garminKey(this).isNullOrBlank()
    } catch (_: Exception) {
        // A phone whose key store has been reset has no key, and the
        // setup screen is the way to put one back.
        Log.i("Stride", "the Garmin key could not be read")
        false
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
        } catch (_: Exception) {
            Log.w("Stride", "the packaged configuration could not be read")
            JSONObject()
        }
        val server = json.optJSONObject("server") ?: JSONObject()
        server.put("url", url)
        json.put("server", server)
        target.writeText(json.toString())
        return target.absolutePath
    }
}
