package app.stride.running.sync

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import app.stride.running.MainActivity
import app.stride.running.R
import app.stride.running.settings.AppSettings

/**
 * The notifications.
 *
 * Two rules, both from the design:
 *
 * * a notification says **generic progress or connection text**, never a
 *   route, a heart rate, an activity name or a health conclusion;
 * * the channel is low-importance, so the phone never makes a noise about a
 *   background pull.
 *
 * The count of runs is in the notification — "Synced 12 activities" —
 * because a count is not a conclusion, and it is the answer the athlete
 * asked for. A failed pull names what could not be reached and when the
 * next scheduled pull is due. It is one notification per scheduled pull:
 * the shell's single retry happens inside the pull and does not turn one
 * missed pull into a trail of alerts.
 */
object SyncNotifications {

    /** The channel the sync notifications go to. */
    const val CHANNEL_ID = "stride.sync"

    /** The id of the ongoing notification. The worker holds it while it runs. */
    const val PROGRESS_ID = 4101
    private const val RESULT_ID = 4102

    /** Create the channel. Android 8 and later need one. */
    fun ensureChannel(context: Context) {
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as? NotificationManager
            ?: return
        val channel = NotificationChannel(
            CHANNEL_ID,
            context.getString(R.string.sync_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        ).apply {
            description = context.getString(R.string.sync_channel_description)
            setShowBadge(false)
        }
        // Creating an existing channel updates its app-owned name and
        // description without changing the channel's importance or the
        // athlete's choice about it.
        manager.createNotificationChannel(channel)
    }

    /** The ongoing notification of a pull that is running. */
    fun progress(context: Context): Notification {
        ensureChannel(context)
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setContentTitle(context.getString(R.string.sync_in_progress_title))
            .setContentText(context.getString(R.string.sync_in_progress_text))
            .setSmallIcon(R.drawable.ic_stat_stride)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .setContentIntent(openApp(context))
            .build()
    }

    /** The notification of a finished pull. */
    fun finished(context: Context, outcome: SyncOutcome) {
        ensureChannel(context)
        val notification = if (outcome.ok) {
            NotificationCompat.Builder(context, CHANNEL_ID)
                .setContentTitle(notificationText(context, outcome))
                .setContentText(context.getString(R.string.sync_done_detail))
                .setSmallIcon(R.drawable.ic_stat_stride)
                .setPriority(NotificationCompat.PRIORITY_LOW)
                .setAutoCancel(true)
                .setContentIntent(openApp(context))
                .build()
        } else {
            NotificationCompat.Builder(context, CHANNEL_ID)
                .setContentTitle(context.getString(R.string.sync_failed_title))
                .setContentText(failureText(context, outcome))
                .setSmallIcon(R.drawable.ic_stat_stride)
                .setPriority(NotificationCompat.PRIORITY_LOW)
                .setAutoCancel(true)
                .setContentIntent(openApp(context))
                .build()
        }
        notify(context, RESULT_ID, notification)
    }

    /**
     * The line the success notification says.
     *
     * "Synced 12 activities" — the count, and nothing about what the runs
     * were. The count is in the plural form, so one activity and twelve
     * activities both come out right.
     */
    fun notificationText(context: Context, outcome: SyncOutcome): String =
        context.resources.getQuantityString(
            R.plurals.sync_done_activities,
            outcome.activities,
            outcome.activities,
        )

    /**
     * The line a failed pull says.
     *
     * It names the failure and the next scheduled pull. WorkManager's time
     * is inexact, so the phone names the local boundary it scheduled
     * around rather than pretending to know the exact second.
     */
    fun failureText(context: Context, outcome: SyncOutcome): String {
        val nextPullAt = AppSettings(context).nextPullAtMillis
        return if (nextPullAt == null) {
            context.getString(R.string.sync_failed_next_unknown, outcome.message)
        } else {
            context.getString(
                R.string.sync_failed_next_pull,
                outcome.message,
                SyncSchedule.formatTime(context, nextPullAt),
            )
        }
    }

    /** Show a notification, if the phone allows it. */
    private fun notify(context: Context, id: Int, notification: Notification) {
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return
        try {
            manager.notify(id, notification)
        } catch (_: SecurityException) {
            // Android 13 and later need the POST_NOTIFICATIONS permission.
            // A refusal never disables the import or the browsing.
        }
    }

    /** The intent that opens the app. */
    private fun openApp(context: Context): PendingIntent {
        val intent = Intent(context, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        return PendingIntent.getActivity(
            context,
            0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
    }
}
