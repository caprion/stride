package app.stride.running.sync

import android.content.Context
import androidx.work.Worker
import androidx.work.WorkerParameters
import app.stride.running.core.StrideCore
import app.stride.running.settings.AppSettings

/**
 * The background pull.
 *
 * The worker is the same code the "Sync now" button runs, so the two cannot
 * disagree about what a pull does.
 *
 * Three things it will not do:
 *
 * * it never puts the Garmin key in the WorkManager's input or output data —
 *   the key is unwrapped from the Keystore inside the worker and dropped
 *   again;
 * * it does not decrypt while the phone is locked — it defers instead;
 * * it does not pretend a failed pull was a success — a failure is a failure,
 *   and the phone says so.
 */
class GarminSyncWorker(
    context: Context,
    parameters: WorkerParameters,
) : Worker(context, parameters) {

    override fun doWork(): Result {
        val context = applicationContext

        // A locked phone cannot use the Keystore key, so there is nothing to
        // do but come back later.
        if (SyncEngine.isLocked(context)) {
            return Result.retry()
        }

        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            SyncNotifications.finished(
                context,
                SyncOutcome(ok = false, message = problem, error = problem),
            )
            return Result.retry()
        }

        val settings = AppSettings(context)
        val outcome = SyncEngine.run(context, settings.syncDays.toLong())

        SyncNotifications.finished(context, outcome)

        return when {
            outcome.ok -> Result.success()
            // A locked phone or a rate limit comes back later; anything else
            // is worth one more try and then the phone says "action needed".
            outcome.error == "locked" || outcome.error == "failed" -> Result.retry()
            else -> Result.failure()
        }
    }
}
