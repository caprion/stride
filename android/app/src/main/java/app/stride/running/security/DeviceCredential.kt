package app.stride.running.security

import android.app.KeyguardManager
import android.content.Context
import androidx.activity.result.ActivityResultLauncher
import app.stride.running.R

/**
 * The device's own credential.
 *
 * The phone holds a year of health data. A phone left unlocked in a bag
 * should not show it, so the app asks the device for its credential — the
 * screen lock, or the biometric the phone has in front of it.
 *
 * **No credential of the app's own.** The app never stores a password, a PIN
 * or a biometric template: the device's credential is the device's, and the
 * Keystore already refuses to work while the phone is locked.
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
     *
     * **One way in.** The app's floor is Android 15, so the platform's
     * `BiometricPrompt` — with the device credential allowed — is always
     * there. The older "confirm device credential" screen is not needed and
     * is not kept: a path that cannot run is a path that can only rot.
     */
    fun prompt(
        activity: androidx.fragment.app.FragmentActivity,
        onGranted: () -> Unit,
        onRefused: () -> Unit,
    ) {
        if (!hasCredential(activity)) {
            // Nothing to ask for. The pull is refused elsewhere.
            onGranted()
            return
        }
        BiometricGate.prompt(activity, onGranted, onRefused)
    }
}
