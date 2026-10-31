package app.stride.running.sync

import android.app.KeyguardManager
import android.content.Context
import android.util.Log
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
import org.json.JSONObject

/**
 * One pull from Garmin.
 *
 * The result is a sentence the phone can show, and the counts behind it. A
 * pull that failed is not a success with a zero: the phone keeps the data it
 * has and says plainly that it could not reach Garmin.
 */
data class SyncOutcome(
    /** Whether the pull finished. */
    val ok: Boolean,
    /** What happened, in one sentence. */
    val message: String,
    /** How many runs Garmin had in the window. */
    val activitiesFound: Int = 0,
    /** How many of them the phone did not have. */
    val activitiesNew: Int = 0,
    /** How many days of wellness arrived. */
    val wellnessDays: Int = 0,
    /** The failure, if there was one. */
    val error: String? = null,
)

/**
 * The pull itself.
 *
 * The same code answers the "Sync now" button, the background worker and the
 * setup screen, so the three cannot drift apart.
 *
 * **A refused key is re-exchanged once.** Garmin's token lasts about a day
 * and there is no refresh token, so a 401 is answered with one more
 * exchange and one more try — and then the pull stops. See
 * [PullPolicy].
 */
object SyncEngine {

    private const val TAG = "StrideSync"

    /** The failure of a pull that Garmin refused the key for. */
    const val ERROR_REFUSED = "refused"

    /** The failure of a pull on a locked phone. */
    const val ERROR_LOCKED = "locked"

    /** The failure of a pull that could not reach Garmin. */
    const val ERROR_FAILED = "failed"

    /** The failure of a pull with no key on the phone. */
    const val ERROR_NOT_CONNECTED = "not connected"

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

        var exchanges = 0
        var outcome = attempt(context, key, days)
        exchanges += 1

        // One re-exchange, and one retry. The exchange happens inside the
        // core: Garmin has no refresh token, so the key is exchanged again.
        while (outcome.error == ERROR_REFUSED &&
            PullPolicy.afterRefusal(exchanges) == PullPolicy.Step.ReExchangeAndRetry
        ) {
            outcome = attempt(context, key, days)
            exchanges += 1
        }

        if (outcome.error == ERROR_REFUSED) {
            return SyncOutcome(
                ok = false,
                message = PullPolicy.NEEDS_RECONNECT,
                error = ERROR_REFUSED,
            )
        }

        return outcome
    }

    /**
     * One attempt: the exchange and the pull.
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
            val detail = envelope.optJSONObject("error")?.optString("detail")
                ?: "Garmin could not be reached."
            return SyncOutcome(
                ok = false,
                message = detail,
                error = if (status == 401) ERROR_REFUSED else ERROR_FAILED,
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
