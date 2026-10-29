package app.stride.running.security

import android.app.KeyguardManager
import android.content.Context
import android.graphics.Color
import android.os.SystemClock
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import androidx.activity.ComponentActivity
import androidx.fragment.app.FragmentActivity
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import app.stride.running.R

/**
 * The app gate.
 *
 * The phone holds a year of health data. A phone left unlocked in a bag
 * should not show it, so the app asks the device for its credential — the
 * screen lock, or the biometric — when it opens, and again after it has
 * been in the background for a minute.
 *
 * **What it is not.** The gate is a *foreground* gate: it stops a borrowed
 * phone from showing the athlete's history. It is not a key of its own. The
 * database key is in the Keystore, and the Keystore already refuses to work
 * while the phone is locked — so the background pull defers until the phone
 * is unlocked, and this gate does not stand in its way.
 *
 * The overlay is opaque: the app behind it must not appear in the recents
 * thumbnail, which is a picture of the athlete's health.
 */
class AppGate(private val activity: FragmentActivity) {

    /** How long the app may be in the background before the gate closes. */
    private val graceMillis = 60_000L

    private var overlay: View? = null
    private var unlocked = false
    private var backgroundedAt = 0L

    /**
     * Called when the gate opens.
     *
     * The app asks the device for its credential when it opens, and the
     * setup screen — the one thing the athlete has to do once — waits for
     * that answer: the screen is opened behind the gate, never in front of
     * it.
     */
    var onUnlocked: (() -> Unit)? = null

    /** Whether the gate is open. */
    val isUnlocked: Boolean
        get() = unlocked

    /**
     * Whether the phone was locked when the app went away.
     *
     * The design says the gate closes *immediately* when the device locks.
     * The keyguard's own state is the signal: when the screen goes off the
     * phone is locked, and the app is sent to the background at the same
     * moment. So the app remembers that it was locked, and asks again when it
     * comes back.
     */
    private var backgroundedWhileLocked = false

    /** The system's credential screen, and its answer. */
    private val credentialLauncher: ActivityResultLauncher<android.content.Intent> =
        activity.registerForActivityResult(
            ActivityResultContracts.StartActivityForResult(),
        ) { result ->
            DeviceCredential.credentialAnswered(result.resultCode)
        }

    /** The app came to the foreground. */
    fun onForeground() {
        val away = SystemClock.elapsedRealtime() - backgroundedAt
        if (!unlocked || away > graceMillis || backgroundedWhileLocked) {
            lock()
        }
    }

    /** The app went to the background. */
    fun onBackground() {
        backgroundedAt = SystemClock.elapsedRealtime()
        backgroundedWhileLocked = deviceLocked()
    }

    /** Whether the phone is locked and wants the athlete to authenticate. */
    private fun deviceLocked(): Boolean {
        val keyguard = activity.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        return keyguard?.isDeviceLocked == true
    }

    /** Close the gate and show the app. */
    fun dismiss() {
        unlocked = true
        overlay?.let { view ->
            (view.parent as? ViewGroup)?.removeView(view)
        }
        overlay = null
        onUnlocked?.invoke()
    }

    /** Open the gate and ask the device who is holding the phone. */
    fun lock() {
        unlocked = false
        show()
        prompt()
    }

    /** The overlay, if it is up. */
    private fun show() {
        if (overlay != null) return

        val root = activity.findViewById<FrameLayout>(android.R.id.content) ?: return

        val panel = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(48, 48, 48, 48)
            // Opaque: the recents thumbnail must not show the app behind it.
            setBackgroundColor(Color.parseColor("#0B0F14"))
        }

        val title = TextView(activity).apply {
            text = activity.getString(R.string.app_name)
            textSize = 26f
            setTextColor(Color.WHITE)
        }
        val message = TextView(activity).apply {
            text = activity.getString(R.string.gate_locked)
            textSize = 15f
            setTextColor(Color.parseColor("#B9C2CC"))
            setPadding(0, 24, 0, 24)
        }
        val unlock = Button(activity).apply {
            text = activity.getString(R.string.gate_unlock)
            setOnClickListener { prompt() }
        }

        panel.addView(title)
        panel.addView(message)
        panel.addView(unlock)

        val view = FrameLayout(activity).apply {
            setBackgroundColor(Color.parseColor("#0B0F14"))
            addView(
                panel,
                FrameLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT,
                ),
            )
        }
        root.addView(
            view,
            FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT,
            ),
        )
        overlay = view
    }

    /** Ask the device for its credential. */
    private fun prompt() {
        DeviceCredential.prompt(
            activity,
            credentialLauncher,
            onGranted = { dismiss() },
            onRefused = { /* the app stays locked */ },
        )
    }
}
