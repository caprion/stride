package app.stride.running.sync

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import app.stride.running.MainActivity
import app.stride.running.R

/**
 * The notifications.
 *
 * Two rules, both from the design:
 *
 * * a notification says **generic progress or reconnect text**, never a route,
 *   a heart rate, an activity name or a health conclusion;
 * * the channel is low-importance, so the phone never makes a noise about a
 *   background pull.
 */
object SyncNotifications {

    /** The channel the sync notifications go to. */
    const val CHANNEL_ID = "stride.sync"

    private const val PROGRESS_ID = 4101
    private const val RESULT_ID = 4102

    /** Create the channel. Android 8 and later need one. */
    fun ensureChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as? NotificationManager
            ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            context.getString(R.string.sync_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        ).apply {
            description = context.getString(R.string.sync_channel_description)
            setShowBadge(false)
        }
        manager.createNotificationChannel(channel)
    }

    /** The ongoing notification of a pull that is running. */
    fun progress(context: Context): Notification {
        ensureChannel(context)
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setContentTitle(context.getString(R.string.sync_in_progress_title))
            .setContentText(context.getString(R.string.sync_in_progress_text))
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .setContentIntent(openApp(context))
            .build()
    }

    /** The notification of a finished pull. */
    fun finished(context: Context, outcome: SyncOutcome) {
        ensureChannel(context)
        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setContentTitle(
                if (outcome.ok) {
                    context.getString(R.string.sync_done_title)
                } else {
                    context.getString(R.string.sync_failed_title)
                }
            )
            .setContentText(outcome.message)
            .setSmallIcon(
                if (outcome.ok) {
                    android.R.drawable.stat_notify_sync_done
                } else {
                    android.R.drawable.stat_notify_error
                }
            )
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setAutoCancel(true)
            .setContentIntent(openApp(context))
            .build()
        notify(context, RESULT_ID, notification)
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
