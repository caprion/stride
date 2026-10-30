package app.stride.running.setup

import android.app.Application
import android.os.Handler
import android.os.Looper
import android.util.Log
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
import app.stride.running.sync.SyncEngine
import app.stride.running.sync.SyncScheduler
import org.json.JSONObject
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.ScheduledFuture
import java.util.concurrent.TimeUnit

/**
 * The work behind the setup screen.
 *
 * **One key, one pull, one answer.** The runner takes the key, checks it
 * against Garmin, keeps it if Garmin wants it, starts the pull and reports
 * what is happening while it runs. The screen shows what the runner says.
 *
 * The pull runs here, on this thread, rather than as a background job, for
 * one reason: the athlete is watching, and the number of runs that arrived
 * is the answer to the question they asked. A job would do the same work
 * with nobody watching, and the screen would have to guess.
 *
 * The periodic job is still scheduled: the pull that follows the key is
 * the one that happens now, and the one that happens every six hours after
 * that is the scheduler's.
 */
class SetupRunner(
    private val app: Application,
    private val listener: Listener,
) {

    /** What the runner tells the screen. */
    interface Listener {
        /** The state changed. It is called on the main thread. */
        fun onState(ui: SetupUi)
    }

    /** What Garmin said about the key. */
    private enum class Verdict {
        /** Garmin took it. */
        Accepted,

        /** Garmin did not take it. */
        Refused,

        /** The check could not be made. */
        Failed,
    }

    private val main = Handler(Looper.getMainLooper())

    private val worker: java.util.concurrent.ExecutorService =
        Executors.newSingleThreadExecutor { runnable ->
            Thread(runnable, "stride-setup").apply { isDaemon = true }
        }

    private val counter: ScheduledExecutorService =
        Executors.newSingleThreadScheduledExecutor { runnable ->
            Thread(runnable, "stride-count").apply { isDaemon = true }
        }

    /** The count job, while there is one. */
    @Volatile
    private var counting: ScheduledFuture<*>? = null

    /** The phase the screen is in. The counter only counts while it syncs. */
    @Volatile
    private var phase: SetupPhase = SetupPhase.Idle

    /** Whether a pull is running. One at a time. */
    @Volatile
    private var busy = false

    /**
     * Take the key, check it, keep it and pull.
     *
     * The key is handed over once and dropped: it is written to the
     * Keystore-wrapped envelope, and the pull unwraps it again for the
     * length of the exchange. It is never logged and never given to the
     * page.
     */
    fun connect(key: String) {
        if (busy) return
        busy = true
        publish(SetupUi(SetupPhase.Checking))

        worker.execute {
            val problem = StrideCore.ensureOpen(app)
            if (problem != null) {
                finish(SetupUi(SetupPhase.Failed, detail = problem))
                return@execute
            }

            when (test(key)) {
                Verdict.Refused -> {
                    // Garmin did not want it. The key is not kept, and the
                    // field stays where the athlete left it.
                    finish(SetupUi(SetupPhase.Refused))
                    return@execute
                }
                Verdict.Failed -> {
                    finish(SetupUi(SetupPhase.Failed, detail = "the key could not be checked"))
                    return@execute
                }
                Verdict.Accepted -> Unit
            }

            // Garmin took it, so the phone keeps it.
            Secrets.saveGarminKey(app, key)

            publish(SetupUi(SetupPhase.Syncing))

            // The pull that follows the key. The periodic job takes over
            // after this one.
            SyncScheduler.schedule(app)

            startCounting()
            val outcome = SyncEngine.run(app, FIRST_PULL_DAYS)
            stopCounting()

            if (!outcome.ok) {
                finish(SetupUi(SetupPhase.Failed, detail = outcome.message))
                return@execute
            }

            // The count is read after the pull, so it is the count of the
            // runs that have actually landed.
            finish(SetupUi(SetupPhase.Connected, runs = SyncEngine.runCount(app)))
        }
    }

    /** Forget the key. The imported runs stay on the phone. */
    fun forget() {
        Secrets.forgetGarminKey(app)
        SyncScheduler.cancel(app)
        // The core forgets it too: the fact of the connection goes with
        // the key. Clearing the copy in the core's memory and saying the
        // phone is not connected are two things, and both are needed here.
        StrideCore.setGarminKey("")
        StrideCore.setGarminConnected(false)
        publish(SetupUi(SetupPhase.Idle))
    }

    /**
     * How many runs the phone has.
     *
     * The count is read off the main thread, because it is a database read
     * and the screen must not wait for it.
     */
    fun count(onCount: (Int) -> Unit) {
        try {
            counter.execute {
                val runs = SyncEngine.runCount(app)
                main.post { onCount(runs) }
            }
        } catch (error: Throwable) {
            Log.w(TAG, "the runs could not be counted: ${error.message}")
        }
    }

    /** Stop the work. The activity is going away. */
    fun close() {
        counting?.cancel(true)
        counter.shutdownNow()
        worker.shutdown()
    }

    /**
     * Whether Garmin accepts the key.
     *
     * The check is the token exchange and a profile read: both have to work
     * before the key is worth having.
     *
     * The route answers every failure the same way — a 400 with "the Garmin
     * API key was refused" — so the phone cannot tell a wrong key from a
     * Garmin that is down. It can tell a phone with no network from a phone
     * with one, and that is the difference it acts on: a phone with no
     * signal is told Garmin could not be reached, and a phone with a signal
     * is told the key was refused.
     */
    private fun test(key: String): Verdict {
        val body = JSONObject().put("api_key", key).toString()
        val envelope = try {
            JSONObject(StrideCore.call("POST", "/api/v1/auth/garmin/personal-key/test", body))
        } catch (error: Throwable) {
            Log.w(TAG, "the key could not be checked: ${error.message}")
            return Verdict.Failed
        }
        if (envelope.optBoolean("ok")) return Verdict.Accepted

        // The reason is in the log, not on the screen: the screen says the
        // key was refused, and that is what the athlete has to act on.
        val detail = envelope.optJSONObject("error")?.optString("detail").orEmpty()
        Log.w(TAG, "Garmin refused the key: $detail")
        return if (!hasNetwork()) Verdict.Failed else Verdict.Refused
    }

    /** Whether the phone has a network to reach Garmin over. */
    private fun hasNetwork(): Boolean {
        val manager = app.getSystemService(Application.CONNECTIVITY_SERVICE)
            as? android.net.ConnectivityManager
            ?: return true
        val network = manager.activeNetwork ?: return false
        val capabilities = manager.getNetworkCapabilities(network) ?: return false
        return capabilities.hasCapability(android.net.NetworkCapabilities.NET_CAPABILITY_INTERNET)
    }

    /**
     * Count the runs while the pull runs.
     *
     * The count is the number of runs on the phone, read from the database
     * the pull is writing to. It grows as the pull lands them, which is
     * what "12 runs" is: the number that has arrived so far.
     */
    private fun startCounting() {
        counting = counter.scheduleAtFixedRate(
            {
                if (phase != SetupPhase.Syncing) return@scheduleAtFixedRate
                val runs = SyncEngine.runCount(app)
                publish(SetupUi(SetupPhase.Syncing, runs = runs))
            },
            0,
            700,
            TimeUnit.MILLISECONDS,
        )
    }

    /** Stop counting. */
    private fun stopCounting() {
        counting?.cancel(false)
        counting = null
    }

    /** Report a state, on the main thread. */
    private fun publish(ui: SetupUi) {
        phase = ui.phase
        main.post { listener.onState(ui) }
    }

    /** Report the last state of a run, and let the next one start. */
    private fun finish(ui: SetupUi) {
        busy = false
        publish(ui)
    }

    companion object {
        private const val TAG = "StrideSetup"

        /**
         * How far back the first pull reaches.
         *
         * A year. The pull that follows the key is the one the athlete is
         * watching, and it is the one that has to bring the history: a
         * thirty-day window would connect the phone and leave the athlete
         * looking at a handful of runs and thinking the sync had failed
         * again. The periodic pull after this one uses the phone's own
         * window, which is the right size for keeping up to date.
         *
         * The core caps the work at 300 activities, so the window cannot
         * run away however long the history is.
         */
        const val FIRST_PULL_DAYS = 365L
    }
}
