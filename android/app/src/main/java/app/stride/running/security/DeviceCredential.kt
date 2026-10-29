package app.stride.running.security

import android.app.Activity
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.activity.result.ActivityResultLauncher
import app.stride.running.R

/**
 * The device's own credential.
 *
 * The phone holds a year of health data. A phone left unlocked in a bag
 * should not show it, so the app asks the device for its credential — the
 * screen lock, or the biometric the phone has in front of it.
 *
 * **No credential of the app's own.** The app never stores a password, a
 * PIN or a biometric template: the device's credential is the device's, and
 * the Keystore already refuses to work while the phone is locked.
 *
 * Two ways in, because the platforms differ:
 *
 * * API 28 and later: `BiometricPrompt`, with the device credential
 *   allowed, so the athlete can use either;
 * * before that: the system's own "confirm device credential" screen.
 *
 * A phone with **no** screen lock at all has nothing to ask for. The gate
 * opens (there is no credential to check) and the *pull* is refused
 * separately, because real data must not land on an unprotected phone.
 */
object DeviceCredential {

    /** Whether the phone has a screen lock or a biometric. */
    fun hasCredential(context: Context): Boolean {
        val keyguard = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        return keyguard?.isDeviceSecure == true
    }

    /**
     * Ask the device for its credential.
     *
     * `onGranted` runs when the phone has vouched for the athlete; `onRefused`
     * when the attempt was cancelled.
     */
    fun prompt(
        activity: androidx.fragment.app.FragmentActivity,
        launcher: ActivityResultLauncher<Intent>,
        onGranted: () -> Unit,
        onRefused: () -> Unit,
    ) {
        if (!hasCredential(activity)) {
            // Nothing to ask for. The pull is refused elsewhere.
            onGranted()
            return
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            BiometricGate.prompt(activity, onGranted, onRefused)
            return
        }

        val keyguard = activity.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        val intent = keyguard?.createConfirmDeviceCredentialIntent(
            activity.getString(R.string.gate_title),
            activity.getString(R.string.gate_description),
        )
        if (intent == null) {
            onRefused()
            return
        }
        // The system's own screen: the athlete types their PIN, pattern or
        // password, and the phone says yes or no.
        keyguardLauncher = launcher
        keyguardOutcome = onGranted
        keyguardRefusal = onRefused
        launcher.launch(intent)
    }

    /** The result of the system's credential screen. */
    private var keyguardLauncher: ActivityResultLauncher<Intent>? = null
    private var keyguardOutcome: (() -> Unit)? = null
    private var keyguardRefusal: (() -> Unit)? = null

    /**
     * Whether the system's credential screen answered.
     *
     * The launcher cannot carry a callback of its own here without the
     * activity owning it, so the outcome is held between the launch and the
     * answer.
     */
    fun credentialAnswered(resultCode: Int) {
        if (resultCode == Activity.RESULT_OK) {
            keyguardOutcome?.invoke()
        } else {
            keyguardRefusal?.invoke()
        }
        keyguardOutcome = null
        keyguardRefusal = null
        keyguardLauncher = null
    }
}
