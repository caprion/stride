package app.stride.running.sync

import android.app.KeyguardManager
import android.content.Context
import android.util.Log
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
import org.json.JSONObject

/**
 * The pull itself.
 *
 * The same code answers the "Sync now" button, the background worker and the
 * setup screen, so the three cannot drift apart.
 *
 * **The rule about a refused key lives in [PullPolicy].** The core performs
 * one token exchange and one attempt per call and answers `needs_reconnect`
 * when the token has expired; this engine may make **one** more attempt, and
 * then it stops. See `docs/contract/core-contract.md` §7 — "The retry rule,
 * and who owns it".
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
     * The pull is attempted, and a refusal is answered with one re-exchange
     * and one retry. The count of exchanges is what the policy reads, so the
     * rule cannot be broken by a second attempt sneaking in.
     */
    fun run(context: Context, days: Long): SyncOutcome {
        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            return SyncOutcome(ok = false, message = problem, error = problem)
        }

        val key = Secrets.garminKey(context)
        if (key.isNullOrBlank()) {
            return SyncOutcome(
                ok = false,
                message = "Garmin is not connected on this phone.",
                error = ERROR_NOT_CONNECTED,
            )
        }

        if (isLocked(context)) {
            return SyncOutcome(
                ok = false,
                message = "The phone is locked. The pull will run when you unlock it.",
                error = ERROR_LOCKED,
            )
        }

        // The retry is the policy's, and the policy has no Android in it.
        return PullPolicy.drive { _ -> attempt(context, key, days) }
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
        parse(answer, days)
    } catch (error: Throwable) {
        Log.w(TAG, "the pull failed: ${error.message}")
        SyncOutcome(
            ok = false,
            message = error.message ?: "Garmin could not be reached.",
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
     * It is the number the database holds, so it grows as a pull lands them:
     * that is what "12 runs" is while the pull runs, and what "160 runs" is
     * when it is over. A phone that cannot be opened has no runs to count,
     * and the count is zero.
     */
    fun runCount(context: Context): Int {
        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            Log.i(TAG, "the runs cannot be counted: $problem")
            return 0
        }
        return try {
            val envelope = JSONObject(StrideCore.call("GET", "/activities?per_page=1", null))
            if (!envelope.optBoolean("ok")) {
                0
            } else {
                envelope.optJSONObject("body")?.optInt("total", 0) ?: 0
            }
        } catch (error: Throwable) {
            Log.w(TAG, "the runs could not be counted: ${error.message}")
            0
        }
    }

    /** Read the core's answer. */
    private fun parse(answer: String, days: Long): SyncOutcome {
        val envelope = JSONObject(answer)
        if (!envelope.optBoolean("ok")) {
            val status = envelope.optInt("status", 0)
            val error = envelope.optJSONObject("error")
            val detail = error?.optString("detail") ?: "Garmin could not be reached."
            val code = error?.optString("code")
            return SyncOutcome(
                ok = false,
                message = detail,
                // A refusal is a refusal whatever shape it takes: the typed
                // failure the contract names, or a plain 401.
                error = if (PullPolicy.isRefusal(status, code)) ERROR_REFUSED else ERROR_FAILED,
            )
        }
        val body = envelope.optJSONObject("body") ?: JSONObject()
        val found = body.optInt("activities_found", 0)
        val created = body.optInt("activities_new", 0)
        val wellness = body.optInt("wellness_days", 0)
        val errors = body.optJSONArray("errors")
        val errorCount = errors?.length() ?: 0
        val message = when {
            errorCount > 0 -> {
                val first = errors?.optJSONObject(0)?.optString("error") ?: "a problem"
                "Imported $created new runs. One problem: $first"
            }
            created == 0 && found == 0 -> "Garmin had no runs in the last $days days."
            else -> "Imported $created new runs and $wellness days of wellness."
        }
        return SyncOutcome(
            ok = true,
            message = message,
            activitiesFound = found,
            activitiesNew = created,
            wellnessDays = wellness,
        )
    }
}
