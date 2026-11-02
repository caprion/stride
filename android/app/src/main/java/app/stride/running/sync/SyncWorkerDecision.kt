package app.stride.running.sync

/**
 * What the background job does after a pull.
 *
 * A locked phone is not a failed pull: no token exchange happened, so the
 * job waits for the phone to become usable. Every exchange failure stops
 * that run. WorkManager's own retry is not used for it, because the shell
 * has already made the one retry it is allowed to make.
 */
object SyncWorkerDecision {

    /** The job's next step. */
    enum class Decision {
        /** The pull finished. */
        Success,

        /** The pull stopped after its one retry. */
        Failure,

        /** No exchange happened because the phone was locked. */
        WaitForUnlock,
    }

    /** The decision for a pull's outcome. */
    fun decide(outcome: SyncOutcome): Decision = when {
        outcome.ok -> Decision.Success
        outcome.error == SyncErrors.LOCKED -> Decision.WaitForUnlock
        else -> Decision.Failure
    }
}
