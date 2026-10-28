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
 * The same code answers the "Sync now" button and the background worker, so
 * the two cannot drift apart.
 */
object SyncEngine {

    private const val TAG = "StrideSync"

    /** Run a pull. */
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
                error = "not connected",
            )
        }

        if (isLocked(context)) {
            return SyncOutcome(
                ok = false,
                message = "The phone is locked. The pull will run when you unlock it.",
                error = "locked",
            )
        }

        return try {
            val answer = StrideCore.garminSync(key, days)
            parse(answer, days)
        } catch (error: Throwable) {
            Log.w(TAG, "the pull failed: ${error.message}")
            SyncOutcome(
                ok = false,
                message = error.message ?: "Garmin could not be reached.",
                error = "failed",
            )
        }
    }

    /** Whether the phone is locked. */
    fun isLocked(context: Context): Boolean {
        val keyguard = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        return keyguard?.isDeviceLocked == true
    }

    /** Read the core's answer. */
    private fun parse(answer: String, days: Long): SyncOutcome {
        val envelope = JSONObject(answer)
        if (!envelope.optBoolean("ok")) {
            val detail = envelope.optJSONObject("error")?.optString("detail")
                ?: "Garmin could not be reached."
            return SyncOutcome(ok = false, message = detail, error = detail)
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
