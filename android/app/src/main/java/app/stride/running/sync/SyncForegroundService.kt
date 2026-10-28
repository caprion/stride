package app.stride.running.sync

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.ServiceCompat
import app.stride.running.settings.AppSettings
import java.util.concurrent.Executors

/**
 * A long pull, in the foreground.
 *
 * WorkManager's workers are bounded, and a pull that runs past the limit is
 * killed with the app. This service is the answer for a pull the athlete
 * asked for and is watching: it holds the notification, so the system does
 * not take the app away mid-pull.
 *
 * The notification says "syncing" and nothing else — no route, no heart rate,
 * no activity name.
 */
class SyncForegroundService : Service() {

    private val worker = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "stride-sync-service").apply { isDaemon = true }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val notification = SyncNotifications.progress(this)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            ServiceCompat.startForeground(
                this,
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        val days = AppSettings(this).syncDays.toLong()
        worker.execute {
            val outcome = try {
                SyncEngine.run(this, days)
            } catch (error: Throwable) {
                SyncOutcome(
                    ok = false,
                    message = error.message ?: "the pull could not be completed",
                    error = "failed",
                )
            }
            SyncNotifications.finished(this, outcome)
            stopSelf(startId)
        }

        return START_NOT_STICKY
    }

    override fun onDestroy() {
        worker.shutdownNow()
        super.onDestroy()
    }

    companion object {
        private const val NOTIFICATION_ID = 4100

        /** Start the service. */
        fun start(context: Context) {
            val intent = Intent(context, SyncForegroundService::class.java)
            context.startForegroundService(intent)
        }
    }
}
