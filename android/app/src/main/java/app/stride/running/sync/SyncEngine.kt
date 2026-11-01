package app.stride.running.sync

import android.app.KeyguardManager
import android.content.Context
import android.util.Log
import app.stride.running.R
import app.stride.running.core.StrideCore
import app.stride.running.security.CredentialUnavailable
import app.stride.running.security.CredentialUnreadable
import app.stride.running.security.Secrets
import org.json.JSONObject

/**
 * The pull itself.
 *
 * The same code answers the "Sync now" button, the background worker and
 * the setup screen, so the three cannot drift apart.
 *
 * **The rule about a failed attempt lives in [PullPolicy].** The core
 * performs one token exchange and one attempt per call and answers
 * `needs_reconnect` when the token has expired; this engine may make **one**
 * more attempt, and then it stops. See
 * `docs/contract/core-contract.md` §7 — "The retry rule, and who owns it".
 */
object SyncEngine {

    private const val TAG = "StrideSync"

    /** The failure of a pull that Garmin refused the key for. */
    const val ERROR_REFUSED = SyncErrors.REFUSED

    /** The failure of a pull on a locked phone. */
    const val ERROR_LOCKED = SyncErrors.LOCKED

    /** The failure of a pull that could not reach Garmin. */
    const val ERROR_FAILED = SyncErrors.FAILED

    /** The failure of a pull with no key on the phone. */
    const val ERROR_NOT_CONNECTED = SyncErrors.NOT_CONNECTED

    /**
     * Run a pull.
     *
     * The pull is attempted, and a failed exchange is answered with one
     * more attempt. The count of attempts is what the policy reads, so the
     * rule cannot be broken by a second attempt sneaking in.
     */
    fun run(context: Context, days: Long): SyncOutcome {
        val window = PullWindow.bounded(days)

        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            return SyncOutcome(
                ok = false,
                message = context.getString(R.string.sync_error_local),
                error = ERROR_FAILED,
            )
        }

        if (isLocked(context)) {
            return SyncOutcome(
                ok = false,
                message = context.getString(R.string.sync_error_locked),
                error = ERROR_LOCKED,
            )
        }

        val savedKey = try {
            Secrets.garminKey(context)
        } catch (_: CredentialUnavailable) {
            return SyncOutcome(
                ok = false,
                message = context.getString(R.string.sync_error_locked),
                error = ERROR_LOCKED,
            )
        } catch (_: CredentialUnreadable) {
            return SyncOutcome(
                ok = false,
                message = context.getString(R.string.sync_error_local),
                error = ERROR_FAILED,
            )
        }

        if (savedKey.isNullOrBlank()) {
            return SyncOutcome(
                ok = false,
                message = context.getString(R.string.sync_error_not_connected),
                error = ERROR_NOT_CONNECTED,
            )
        }

        val attemptKey = savedKey
        // The retry is the policy's, and the policy has no Android in it.
        val outcome = PullPolicy.drive { _ -> attempt(context, attemptKey, window) }
        // The exchange dropped the core's temporary key. The credential is
        // still in the Keystore, so the core is told the separate fact that
        // the phone remains connected.
        StrideCore.setGarminConnected(true)
        // The plaintext key is no longer needed by the shell. The copy in
        // the Keystore is the one that lasts.
        return outcome
    }

    /**
     * One attempt: the exchange and the pull.
     *
     * The seam's contract says this is one token exchange and one HTTP
     * attempt, and that the answer may be `needs_reconnect` — the policy
     * reads that and decides whether there is one more attempt.
     */
    private fun attempt(context: Context, key: String, days: Long): SyncOutcome = try {
        val answer = StrideCore.garminSync(key, days)
        parse(context, answer)
    } catch (_: Throwable) {
        // The exception's message is not logged: it may carry a value from
        // the exchange.
        Log.w(TAG, "the pull could not be completed")
        SyncOutcome(
            ok = false,
            message = context.getString(R.string.sync_error_unreachable),
            error = ERROR_FAILED,
        )
    }

    /** Whether the phone is locked. */
    fun isLocked(context: Context): Boolean {
        val keyguard = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        return keyguard?.isDeviceLocked == true
    }

    /**
     * How many runs the phone has.
     *
     * It is the number the store holds, so it grows as a pull lands them:
     * that is what "12 runs" is while the pull runs, and what "160 runs" is
     * when it is over. A phone that cannot be opened has no runs to count,
     * and the count is zero.
     *
     * The count comes from the api's `SyncStatus.activity_count` — the
     * number of runs the store holds — and not from the length of a page
     * of the feed: a page of ten is not the phone's history, and the feed
     * has no total.
     */
    fun runCount(context: Context): Int {
        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            Log.i(TAG, "the runs cannot be counted")
            return 0
        }
        return try {
            val envelope = JSONObject(StrideCore.call("GET", "/sync/status", null))
            if (!envelope.optBoolean("ok")) {
                0
            } else {
                SyncResult.storedRuns(envelope.optJSONObject("body"))
            }
        } catch (_: Throwable) {
            Log.w(TAG, "the runs could not be counted")
            0
        }
    }

    /** Read the core's answer. */
    private fun parse(context: Context, answer: String): SyncOutcome {
        val envelope = JSONObject(answer)
        if (!envelope.optBoolean("ok")) {
            val status = envelope.optInt("status", 0)
            val error = envelope.optJSONObject("error")
            val detail = error?.optString("detail").orEmpty()
            val code = error?.optString("code")

            return when (GarminFailure.classify(status, code, detail)) {
                GarminFailure.Kind.KeyRejected -> SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_refused),
                    error = ERROR_REFUSED,
                )
                GarminFailure.Kind.Unreachable -> SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_unreachable),
                    error = ERROR_FAILED,
                )
                GarminFailure.Kind.RateLimited -> SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_rate_limited),
                    error = ERROR_FAILED,
                )
                GarminFailure.Kind.Unknown -> SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_failed),
                    error = ERROR_FAILED,
                )
            }
        }

        // The api's `SyncRun`: { activities, wellness_days, finished_at,
        // generated_at }. Those are the fields it has, and the only ones
        // read here — see SyncResult.
        val counts = SyncResult.counts(envelope.optJSONObject("body"))
        return SyncOutcome(
            ok = true,
            message = context.getString(R.string.sync_pull_finished),
            activities = counts.activities,
            wellnessDays = counts.wellnessDays,
        )
    }
}
