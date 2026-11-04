package app.stride.running.sync

/**
 * The history target, not the amount downloaded in one job.
 * The core checkpoints dated work and separately bounds each HTTP attempt.
 */
object PullWindow {

    /** Approximately twelve months, with older history requested explicitly. */
    const val DEFAULT_DAYS = 365L

    /** The largest window the phone asks for. */
    const val MAX_DAYS = 3650L

    /** The smallest useful window. */
    const val MIN_DAYS = 1L

    /** Keep a requested window inside the phone's bound. */
    fun bounded(days: Long): Long = days.coerceIn(MIN_DAYS, MAX_DAYS)
}
