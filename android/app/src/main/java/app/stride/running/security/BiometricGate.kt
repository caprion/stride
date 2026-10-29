package app.stride.running.security

import android.os.Build
import androidx.annotation.RequiresApi
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import app.stride.running.R

/**
 * The biometric prompt.
 *
 * The platform's `android.hardware.biometrics.BiometricPrompt` no longer has
 * the pieces this needs — its `Authenticators` and its `PromptInfo` are gone
 * from the API 36 platform — so the prompt is AndroidX's, which is what the
 * platform API turned into.
 *
 * The credential the device accepts is the athlete's: a strong biometric, or
 * the screen lock. Which one is offered is the phone's business, not the
 * app's; the app only asks the device to say yes or no.
 */
@RequiresApi(Build.VERSION_CODES.P)
object BiometricGate {

    /** Ask the phone who is holding it. */
    fun prompt(activity: FragmentActivity, onGranted: () -> Unit, onRefused: () -> Unit) {
        val executor = ContextCompat.getMainExecutor(activity)
        val prompt = BiometricPrompt(
            activity,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    onGranted()
                }

                override fun onAuthenticationError(code: Int, message: CharSequence) {
                    // The athlete cancelled, or the hardware refused. Either
                    // way the app stays locked.
                    onRefused()
                }

                override fun onAuthenticationFailed() {
                    // A wrong finger: the prompt stays up and asks again.
                }
            },
        )

        val builder = BiometricPrompt.PromptInfo.Builder()
            .setTitle(activity.getString(R.string.gate_title))
            .setSubtitle(activity.getString(R.string.gate_description))

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            builder.setAllowedAuthenticators(
                BiometricManager.Authenticators.BIOMETRIC_STRONG or
                    BiometricManager.Authenticators.DEVICE_CREDENTIAL,
            )
        } else {
            // Before API 30 the device credential is a separate switch.
            builder.setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
            builder.setDeviceCredentialAllowed(true)
        }

        prompt.authenticate(builder.build())
    }
}
