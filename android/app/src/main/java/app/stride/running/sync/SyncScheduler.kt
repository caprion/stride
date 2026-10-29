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
import androidx.work.workDataOf
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

    /** The work data key for the window. A number of days; not a secret. */
    const val KEY_DAYS = "days"

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

    /**
     * Pull now, in the background.
     *
     * The window travels as work data: a number of days is not a secret.
     * The Garmin key does **not** travel — the worker unwraps it from the
     * Keystore itself, so nothing about the credential is in WorkManager's
     * database.
     */
    fun pullNow(context: Context, days: Int = AppSettings(context).syncDays) {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(androidx.work.NetworkType.CONNECTED)
            .build()

        val request = OneTimeWorkRequestBuilder<GarminSyncWorker>()
            .setConstraints(constraints)
            .setInputData(workDataOf(KEY_DAYS to days.coerceIn(1, 90)))
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
}
