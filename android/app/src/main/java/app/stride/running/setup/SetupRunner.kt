package app.stride.running.setup

import android.app.Application
import android.os.Handler
import android.os.Looper
import android.util.Log
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
import app.stride.running.sync.ForgetGarminKey
import app.stride.running.sync.GarminFailure
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

        /** Garmin could not be reached. */
        Unreachable,

        /** Garmin is rate limiting the phone. */
        RateLimited,

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
            try {
                val problem = StrideCore.ensureOpen(app)
                if (problem != null) {
                    finish(SetupUi(SetupPhase.Failed, reason = SetupFailure.LocalStore))
                    return@execute
                }

                when (test(key)) {
                    Verdict.Refused -> {
                        // Garmin did not want it. The key is not kept, and the
                        // field stays where the athlete left it.
                        finish(SetupUi(SetupPhase.Refused))
                        return@execute
                    }
                    Verdict.Unreachable -> {
                        finish(
                            SetupUi(
                                SetupPhase.Failed,
                                reason = SetupFailure.GarminUnreachable,
                            ),
                        )
                        return@execute
                    }
                    Verdict.RateLimited -> {
                        finish(
                            SetupUi(
                                SetupPhase.Failed,
                                reason = SetupFailure.RateLimited,
                            ),
                        )
                        return@execute
                    }
                    Verdict.Failed -> {
                        finish(
                            SetupUi(
                                SetupPhase.Failed,
                                reason = SetupFailure.KeyCheckFailed,
                            ),
                        )
                        return@execute
                    }
                    Verdict.Accepted -> Unit
                }

                // Garmin took it, so the phone keeps it. The core is told
                // the fact of the connection; it never keeps this copy.
                Secrets.saveGarminKey(app, key)
                StrideCore.setGarminConnected(true)

                publish(SetupUi(SetupPhase.Syncing))

                // The pull that follows the key. The periodic job takes over
                // after this one.
                SyncScheduler.schedule(app)

                startCounting()
                val outcome = SyncEngine.run(app, FIRST_PULL_DAYS)
                stopCounting()

                if (!outcome.ok) {
                    finish(
                        if (outcome.error == SyncEngine.ERROR_REFUSED) {
                            SetupUi(SetupPhase.Refused)
                        } else {
                            SetupUi(
                                SetupPhase.Failed,
                                reason = SetupFailure.PullFailed,
                            )
                        },
                    )
                    return@execute
                }

                // The count is read after the pull, so it is the count of the
                // runs that have actually landed.
                finish(SetupUi(SetupPhase.Connected, runs = SyncEngine.runCount(app)))
            } catch (_: Throwable) {
                // The exception's message is not logged: an exception may
                // carry the value the failing code was holding, and this
                // code was holding a Garmin key.
                finish(SetupUi(SetupPhase.Failed, reason = SetupFailure.LocalStore))
            }
        }
    }

    /** Forget the key. The imported runs stay on the phone. */
    fun forget() {
        ForgetGarminKey.onPhone(app)
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
        } catch (_: Throwable) {
            Log.w(TAG, "the runs could not be counted")
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
     * before the key is worth having. The route's safe detail is classified
     * so that a refused key, an unreachable Garmin and a rate limit do not
     * all become the same sentence.
     */
    private fun test(key: String): Verdict {
        if (!hasNetwork()) return Verdict.Unreachable

        // The body is the api's: the route reads the key from "key".
        val body = JSONObject().put("key", key).toString()
        val envelope = try {
            JSONObject(StrideCore.call("POST", PERSONAL_KEY_TEST_ROUTE, body))
        } catch (_: Throwable) {
            Log.w(TAG, "the key could not be checked")
            return Verdict.Failed
        }
        if (envelope.optBoolean("ok")) return Verdict.Accepted

        val error = envelope.optJSONObject("error")
        val status = envelope.optInt("status", 0)
        val code = error?.optString("code")
        val detail = error?.optString("detail").orEmpty()
        return when (GarminFailure.classify(status, code, detail)) {
            GarminFailure.Kind.KeyRejected -> Verdict.Refused
            GarminFailure.Kind.Unreachable -> Verdict.Unreachable
            GarminFailure.Kind.RateLimited -> Verdict.RateLimited
            GarminFailure.Kind.Unknown -> Verdict.Failed
        }
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
        counting = counter.scheduleWithFixedDelay(
            {
                if (phase != SetupPhase.Syncing) return@scheduleWithFixedDelay
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
         * The route that checks a key against Garmin.
         *
         * It is the frozen route table's route, without the prefix the
         * desktop may use: the core's router matches on the path without
         * the query string, and the prefix is cut in one place.
         */
        const val PERSONAL_KEY_TEST_ROUTE = "/auth/garmin/personal-key/test"

        /**
         * How far back the first pull reaches.
         *
         * Thirty days is the phone's ordinary window. The core bounds the
         * work inside that window as well — at most the newest activities
         * and the newest wellness days — because Garmin answers only a few
         * requests a minute. The first pull therefore asks for what the
         * phone can use, not for every run the athlete has ever recorded.
         */
        const val FIRST_PULL_DAYS = 30L
    }
}
