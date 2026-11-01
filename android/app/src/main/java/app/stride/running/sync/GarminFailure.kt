package app.stride.running.sync

/**
 * What Garmin's answer means to the phone.
 *
 * Garmin's failures do not all arrive with the same status, and the
 * router's frozen error code is deliberately broad. The shell still has to
 * say the right thing: a key that was refused is not a network that could
 * not be reached, and the two lead to different next steps.
 *
 * The classifier is a plain object with no Android in it, so the rules can
 * be tested without a phone. It reads only the status, the frozen error code
 * and the router's safe detail — never the Garmin key and never a response
 * body.
 */
object GarminFailure {

    /** What the phone understands about a failed Garmin exchange. */
    enum class Kind {
        /** Garmin refused the key. */
        KeyRejected,

        /** Garmin could not be reached. */
        Unreachable,

        /** Garmin is rate limiting the phone. */
        RateLimited,

        /** Garmin answered in a shape the phone does not recognise. */
        Unknown,
    }

    /**
     * Classify a failed exchange.
     *
     * The router uses `garmin_rejected` for Garmin failures more broadly
     * than its name suggests, so the detail is read before the code. The
     * detail is a fixed sentence from the core and carries no response
     * body.
     */
    fun classify(status: Int, code: String?, detail: String): Kind {
        val text = detail.trim()
        if (text.contains("rate limit", ignoreCase = true)) return Kind.RateLimited
        if (text.contains("could not be reached", ignoreCase = true)) return Kind.Unreachable

        if (code.equals(PullPolicy.NEEDS_RECONNECT, ignoreCase = true)) {
            return Kind.KeyRejected
        }
        if (status == 401 || code.equals(PullPolicy.GARMIN_REJECTED, ignoreCase = true)) {
            return Kind.KeyRejected
        }
        return Kind.Unknown
    }
}
