package app.stride.running.sync

import android.content.Context
import android.content.pm.ServiceInfo
import androidx.work.ForegroundInfo
import androidx.work.Worker
import androidx.work.WorkerParameters
import app.stride.running.R
import app.stride.running.core.StrideCore
import app.stride.running.security.DeviceCredential
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
 *
 * **A refused key is re-exchanged once, and retried once.** The rule lives
 * in [PullPolicy] and the engine carries it out; the worker does not have to
 * know about it, and cannot get it wrong by accident.
 */
class GarminSyncWorker(
    context: Context,
    parameters: WorkerParameters,
) : Worker(context, parameters) {

    override fun doWork(): Result {
        val context = applicationContext
        val settings = AppSettings(context)

        // The job runs around fixed local boundaries. Once it starts, the
        // boundary it just handled is behind it, so the phone names the
        // following one when it reports a failure.
        settings.nextPullAtMillis = SyncSchedule.nextBoundary(
            nowMillis = System.currentTimeMillis(),
            intervalHours = settings.syncIntervalHours.toLong(),
        )

        // A pull can outlast the system's patience with a background job, so
        // the job holds a foreground notification while it runs: that is the
        // foreground service the design asks for, with the system's own
        // bookkeeping rather than a hand-rolled one.
        setForegroundAsync(foregroundInfo())

        // A locked phone cannot use the Keystore key, so there is nothing to
        // do but come back later.
        if (SyncEngine.isLocked(context)) {
            return Result.retry()
        }

        // A phone with **no screen lock** is a different case from a locked
        // one: it is a phone that anybody can pick up, and a pull would put
        // real runs on it. The foreground refuses the same pull; the
        // background has to refuse it too, or the rule only holds while
        // somebody is watching.
        if (!DeviceCredential.hasCredential(context)) {
            SyncNotifications.finished(
                context,
                SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_no_screen_lock),
                    error = SyncEngine.ERROR_FAILED,
                ),
            )
            return Result.failure()
        }

        val problem = StrideCore.ensureOpen(context)
        if (problem != null) {
            SyncNotifications.finished(
                context,
                SyncOutcome(
                    ok = false,
                    message = context.getString(R.string.sync_error_local),
                    error = SyncEngine.ERROR_FAILED,
                ),
            )
            return Result.failure()
        }

        // The window travels with the job; the default is the setting.
        val days = inputData.getInt(SyncScheduler.KEY_DAYS, 0)
            .takeIf { it > 0 }
            ?: AppSettings(context).syncDays
        val outcome = SyncEngine.run(context, days.toLong())

        SyncNotifications.finished(context, outcome)

        return when (SyncWorkerDecision.decide(outcome)) {
            SyncWorkerDecision.Decision.Success -> Result.success()
            // A locked phone did not start an exchange, so it can wait for
            // the phone to become usable. Every other failure has already
            // had the shell's one retry and stops this run.
            SyncWorkerDecision.Decision.WaitForUnlock -> Result.retry()
            SyncWorkerDecision.Decision.Failure -> Result.failure()
        }
    }

    /** The notification the job holds while it runs. */
    private fun foregroundInfo(): ForegroundInfo {
        val notification = SyncNotifications.progress(applicationContext)
        return ForegroundInfo(
            SyncNotifications.PROGRESS_ID,
            notification,
            ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
        )
    }
}
