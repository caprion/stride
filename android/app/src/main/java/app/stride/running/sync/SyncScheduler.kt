package app.stride.running.sync

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import app.stride.running.settings.AppSettings
import java.util.concurrent.TimeUnit

/**
 * When the phone pulls.
 *
 * One unique periodic request, so there is never a second one running beside
 * the first. The constraints are the honest ones: a network, and enough
 * battery. The phone does not ask to be exempt from battery optimisation,
 * and it does not use an exact alarm.
 *
 * The interval is six hours — a product choice, not a promise: WorkManager's
 * minimum is fifteen minutes and its execution is inexact, so the pull may
 * happen later than that. The phone shows the time of the last successful
 * pull rather than pretending it is fresh.
 */
object SyncScheduler {

    /** The name of the periodic work. Unique. */
    const val PERIODIC_NAME = "stride-garmin-sync"

    /** The name of the one-shot work. Unique. */
    const val ONESHOT_NAME = "stride-garmin-sync-now"

    /** Schedule the periodic pull, replacing whatever was there. */
    fun schedule(context: Context) {
        val settings = AppSettings(context)
        if (!settings.autoSync || !settings.networkEnabled) {
            cancel(context)
            return
        }

        val constraints = Constraints.Builder()
            .setRequiredNetworkType(androidx.work.NetworkType.CONNECTED)
            .setRequiresBatteryNotLow(true)
            .build()

        val request = PeriodicWorkRequestBuilder<GarminSyncWorker>(
            settings.syncIntervalHours.toLong().coerceAtLeast(1),
            TimeUnit.HOURS,
        )
            .setConstraints(constraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.MINUTES)
            .addTag(PERIODIC_NAME)
            .build()

        WorkManager.getInstance(context).enqueueUniquePeriodicWork(
            PERIODIC_NAME,
            ExistingPeriodicWorkPolicy.UPDATE,
            request,
        )
    }

    /** Pull now, in the background. */
    fun pullNow(context: Context) {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(androidx.work.NetworkType.CONNECTED)
            .build()

        val request = OneTimeWorkRequestBuilder<GarminSyncWorker>()
            .setConstraints(constraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 15, TimeUnit.MINUTES)
            .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
            .addTag(ONESHOT_NAME)
            .build()

        WorkManager.getInstance(context).enqueueUniqueWork(
            ONESHOT_NAME,
            ExistingWorkPolicy.REPLACE,
            request,
        )
    }

    /** Stop pulling: the key is gone, so there is nothing to pull with. */
    fun cancel(context: Context) {
        WorkManager.getInstance(context).cancelUniqueWork(PERIODIC_NAME)
        WorkManager.getInstance(context).cancelUniqueWork(ONESHOT_NAME)
    }

    /** Whether a pull is queued or running. */
    fun isRunning(context: Context): Boolean {
        val infos = WorkManager.getInstance(context)
            .getWorkInfosForUniqueWork(PERIODIC_NAME)
            .get()
        return infos.any { !it.state.isFinished }
    }
}
