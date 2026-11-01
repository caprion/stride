package app.stride.running.sync

/**
 * The result of a pull.
 *
 * A pull that failed is not a success with a zero: the phone keeps the data
 * it has and says plainly that it could not reach Garmin.
 *
 * This is the whole of the result, and it has no Android in it, so the retry
 * rule can be exercised without a phone.
 */
data class SyncOutcome(
    /** Whether the pull finished. */
    val ok: Boolean,
    /** What happened, in one sentence. */
    val message: String,
    /**
     * How many runs the pull brought in.
     *
     * The api's `SyncRun.activities` — the count the pull reports, not a
     * count the phone keeps of its own.
     */
    val activities: Int = 0,
    /** How many days of wellness the pull brought in. */
    val wellnessDays: Int = 0,
    /** The failure, if there was one. */
    val error: String? = null,
)

/**
 * The failures a pull can end with.
 *
 * They are codes, not sentences: a sentence is what the athlete reads, and
 * it is decided in one place.
 */
object SyncErrors {

    /** Garmin refused the key, or the token had expired. */
    const val REFUSED = "refused"

    /** The phone was locked, so the key could not be used. */
    const val LOCKED = "locked"

    /** Garmin could not be reached. */
    const val FAILED = "failed"

    /** There is no key on the phone. */
    const val NOT_CONNECTED = "not connected"
}
