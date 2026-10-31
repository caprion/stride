package app.stride.running.sync

/**
 * What a pull is allowed to do about a refused key.
 *
 * Garmin's Personal API key is exchanged for a bearer token that lasts
 * about a day, and **Garmin has no refresh token**: a "refresh" is a second
 * exchange of the same key. So a 401 is not a dead end — it is a sign that
 * the exchange has to happen again.
 *
 * But it is not a loop either. The rule is:
 *
 * * **one** re-exchange, and **one** retry;
 * * and then the pull stops, and the phone says the key was refused.
 *
 * Two exchanges with the same key that both come back refused mean the key
 * itself is the problem, and asking again would be asking the same question
 * twice and getting the same answer.
 *
 * It is a plain object with no Android in it, so the rule can be tested
 * without a phone.
 */
object PullPolicy {

    /** How many re-exchanges there are. One. */
    const val MAX_RE_EXCHANGES = 1

    /** What the pull does next. */
    enum class Step {
        /** Exchange the key again, and try the pull once more. */
        ReExchangeAndRetry,

        /** Stop. The key was refused after the re-exchange. */
        Stop,
    }

    /**
     * What a pull does after the key was refused.
     *
     * `exchanges` is how many exchanges have already happened for this
     * pull, including the first one.
     */
    fun afterRefusal(exchanges: Int): Step =
        if (exchanges <= MAX_RE_EXCHANGES) Step.ReExchangeAndRetry else Step.Stop

    /**
     * The sentence for a pull that stopped because the key was refused
     * again.
     *
     * It says what happened and what is left to do, and it does not blame
     * the athlete: the key may have been rotated, or revoked at Garmin.
     */
    const val NEEDS_RECONNECT =
        "Garmin refused the key again, so the sync stopped. A new key may be needed."
}
