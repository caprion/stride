package app.stride.running.sync

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequest
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.PeriodicWorkRequest
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

    /**
     * The periodic request: the pull that follows the key on its own.
     *
     * It carries [PERIODIC_NAME], which is the tag every Garmin pull
     * carries: a query for that tag answers "is a pull scheduled".
     */
    fun periodicRequest(intervalHours: Long): PeriodicWorkRequest {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(androidx.work.NetworkType.CONNECTED)
            .setRequiresBatteryNotLow(true)
            .build()

        return PeriodicWorkRequestBuilder<GarminSyncWorker>(
            intervalHours.coerceAtLeast(1),
            TimeUnit.HOURS,
        )
            .setConstraints(constraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.MINUTES)
            .addTag(PERIODIC_NAME)
            .build()
    }

    /**
     * The one-time request: the pull the athlete asks for now.
     *
     * **The tags are the contract with the app.** Every pull carries
     * [PERIODIC_NAME] — "a Garmin pull" — and the one-time one carries
     * [ONESHOT_NAME] as well, so a query for either name finds it. The app's
     * "is a pull running" answer comes from the tags, and a pull that only
     * carried one of them would be invisible to half of those queries.
     *
     * The window travels as work data: a number of days is not a secret.
     * The Garmin key does **not** travel — the worker unwraps it from the
     * Keystore itself, so nothing about the credential is in WorkManager's
     * database.
     */
    fun oneTimeRequest(days: Int): OneTimeWorkRequest {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(androidx.work.NetworkType.CONNECTED)
            .build()

        return OneTimeWorkRequestBuilder<GarminSyncWorker>()
            .setConstraints(constraints)
            .setInputData(workDataOf(KEY_DAYS to days.coerceIn(1, 90)))
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 15, TimeUnit.MINUTES)
            .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
            .addTag(PERIODIC_NAME)
            .addTag(ONESHOT_NAME)
            .build()
    }

    /** Schedule the periodic pull, replacing whatever was there. */
    fun schedule(context: Context) {
        val settings = AppSettings(context)
        if (!settings.autoSync || !settings.networkEnabled) {
            cancel(context)
            return
        }

        val request = periodicRequest(settings.syncIntervalHours.toLong())

        WorkManager.getInstance(context).enqueueUniquePeriodicWork(
            PERIODIC_NAME,
            ExistingPeriodicWorkPolicy.UPDATE,
            request,
        )
    }

    /**
     * Pull now, in the background.
     *
     * **One** unique one-shot work, so a second "Sync now" replaces the
     * first rather than running beside it. The request is [oneTimeRequest]:
     * a one-time request, never the periodic one.
     */
    fun pullNow(context: Context, days: Int = AppSettings(context).syncDays) {
        val request = oneTimeRequest(days)

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
