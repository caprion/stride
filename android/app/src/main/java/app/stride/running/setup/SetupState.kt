package app.stride.running.setup

import app.stride.running.R

/**
 * The states of the Garmin setup screen.
 *
 * The screen is a small state machine, and this is the whole of it. There
 * is no Android in this file, so the machine can be exercised without a
 * phone: what the screen says in each state is decided here, and the words
 * themselves live in `res/values/strings.xml`.
 */
enum class SetupPhase {

    /** No key on the phone yet. The field is there, waiting for one. */
    Idle,

    /** The key is being checked against Garmin. */
    Checking,

    /** Garmin accepted the key and the pull is running. */
    Syncing,

    /** Garmin is connected and the runs have arrived. */
    Connected,

    /** Garmin did not accept the key. The field stays where it is. */
    Refused,

    /** The pull did not finish. */
    Failed,
}

/**
 * What the screen is showing.
 *
 * `runs` is how many runs the phone has — the number that grows while the
 * pull is running. `detail` is the reason, when there is one.
 */
data class SetupUi(
    val phase: SetupPhase,
    val runs: Int = 0,
    val reason: SetupFailure? = null,
)

/**
 * Why a setup attempt did not finish.
 *
 * The screen carries a reason, never an error message. An exception message
 * can carry anything the failing code was holding, and this screen was
 * holding a Garmin key. The reason chooses a line from the native string
 * catalogue; the raw failure never reaches the screen or a log.
 */
enum class SetupFailure {
    /** The phone's local store could not be opened or written. */
    LocalStore,

    /** Garmin did not answer the key check. */
    GarminUnreachable,

    /** Garmin is rate limiting the phone. */
    RateLimited,

    /** Garmin answered, but not with a decision about the key. */
    KeyCheckFailed,

    /** The key was accepted, but the pull did not finish. */
    PullFailed,
}

/**
 * One line of the screen: a string resource, and the number it counts.
 *
 * The count is `null` for a line that does not count anything. `plural` says
 * the resource is a plural, so "1 run" and "160 runs" both come out right.
 */
data class Line(
    val resource: Int,
    val count: Int? = null,
    val plural: Boolean = false,
)

/**
 * Whether the setup screen opens itself.
 *
 * The app opens the native screen on the first launch: an athlete with no
 * key has an app that cannot show their runs. Two things stop it:
 *
 * * a key — once there is one, the screen has nothing left to ask for, and
 *   it must not come back on every launch;
 * * a dismissal — the athlete closed it without connecting, and that is a
 *   decision too.
 */
object SetupEntry {

    /** Whether the screen should open itself. */
    fun shouldOpen(hasKey: Boolean, dismissed: Boolean): Boolean = !hasKey && !dismissed
}

/**
 * The copy, decided by the state.
 *
 * The words are in `strings.xml`; this is which of them the screen says,
 * and when. Nothing else in the app decides this, so there is one answer to
 * "what does the screen show when Garmin refuses the key".
 *
 * **Every line here is a `status` string.** The screen tells the athlete
 * what is happening and what the state is; it does not tell them what to do
 * about their training, and it does not judge the key.
 */
object SetupCopy {

    /** The status line: what is happening. */
    fun status(ui: SetupUi): Line = when (ui.phase) {
        SetupPhase.Idle -> Line(R.string.garmin_not_connected)
        SetupPhase.Checking -> Line(R.string.garmin_checking)
        SetupPhase.Syncing -> Line(R.string.garmin_syncing)
        // A negative count means the runs have not been counted yet, so the
        // screen says "Connected" and fills the number in when it has it.
        SetupPhase.Connected -> if (ui.runs < 0) {
            Line(R.string.garmin_connected)
        } else {
            Line(R.plurals.garmin_connected_runs, ui.runs, plural = true)
        }
        SetupPhase.Refused -> Line(R.string.garmin_refused)
        SetupPhase.Failed -> Line(R.string.garmin_failed)
    }

    /** The count line: "12 runs", and only while the pull is running. */
    fun progress(ui: SetupUi): Line? = when (ui.phase) {
        SetupPhase.Syncing -> Line(R.plurals.garmin_runs, ui.runs, plural = true)
        else -> null
    }

    /**
     * The line under the state, when there is a reason to add one.
     *
     * A failed setup says what happened and what remains available. It never
     * carries a stack trace, a response body or an exception message.
     */
    fun detail(ui: SetupUi): Line? = when (ui.reason) {
        SetupFailure.LocalStore -> Line(R.string.garmin_failed_local)
        SetupFailure.GarminUnreachable -> Line(R.string.garmin_failed_unreachable)
        SetupFailure.RateLimited -> Line(R.string.garmin_failed_rate_limited)
        SetupFailure.KeyCheckFailed -> Line(R.string.garmin_failed_key_check)
        SetupFailure.PullFailed -> Line(R.string.garmin_failed_pull)
        null -> null
    }
}
