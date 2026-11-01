package app.stride.running.sync

/**
 * How much history one pull may ask Garmin for.
 *
 * Garmin's Personal API answers only a few requests a minute. The shell's
 * window is therefore bounded before the request reaches the core, and the
 * core bounds the activities and wellness days inside that window as well.
 *
 * The phone asks for the ordinary thirty-day window. A pull that reached
 * for everything would spend the request limit before any run had landed.
 */
object PullWindow {

    /** The ordinary number of days one pull covers. */
    const val DEFAULT_DAYS = 30L

    /** The largest window the phone asks for. */
    const val MAX_DAYS = 30L

    /** The smallest useful window. */
    const val MIN_DAYS = 1L

    /** Keep a requested window inside the phone's bound. */
    fun bounded(days: Long): Long = days.coerceIn(MIN_DAYS, MAX_DAYS)
}
