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
 * **The shell owns this rule, and the client does not.**
 * `docs/contract/core-contract.md` §7 — "The retry rule, and who owns it":
 * the Garmin client performs exactly one token exchange and one HTTP
 * attempt per call and answers the typed failure `needs_reconnect` when the
 * token has expired; the *caller* — this file — may make one more attempt.
 * If the client retried as well, the phone would exchange and retry twice
 * and "exactly one" would be a coincidence rather than a rule.
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
     * The typed failure the core answers with when the token has expired.
     *
     * The contract's name for it. The core may also answer with a bare 401,
     * so both are read as a refusal.
     */
    const val NEEDS_RECONNECT = "needs_reconnect"

    /**
     * The other typed failure that means the same thing.
     *
     * The contract's error list (`docs/plan.html` §6.3) carries
     * `garmin_rejected` for a key Garmin will not take; from the phone's
     * point of view it is the same situation — the key has to be exchanged
     * again, once, or it has to be replaced.
     */
    const val GARMIN_REJECTED = "garmin_rejected"

    /**
     * What a pull does after the key was refused.
     *
     * `exchanges` is how many exchanges have already happened for this
     * pull, including the first one.
     */
    fun afterRefusal(exchanges: Int): Step =
        if (exchanges <= MAX_RE_EXCHANGES) Step.ReExchangeAndRetry else Step.Stop

    /**
     * Whether the core's answer is a refusal.
     *
     * A refusal is the typed failure `needs_reconnect`, the typed failure
     * `garmin_rejected`, or a plain 401 — the three shapes the answer can
     * take, and all three mean the same thing to the phone: the key has to
     * be exchanged again, once.
     */
    fun isRefusal(status: Int, code: String?): Boolean =
        status == 401 ||
            code.equals(NEEDS_RECONNECT, ignoreCase = true) ||
            code.equals(GARMIN_REJECTED, ignoreCase = true)

    /**
     * The pull, with the rule in it.
     *
     * `attempt` is called with the attempt's number — 1, then 2 — and
     * answers what happened. The loop is here, in an object with no Android
     * in it, so the rule can be proved without a phone:
     *
     * * a first attempt that works is **not** retried;
     * * a first attempt that is refused is retried **once**;
     * * a second refusal stops the pull, and the phone says the key was
     *   refused.
     *
     * The attempt is the seam's: one call, one token exchange, one HTTP
     * attempt (see `docs/contract/core-contract.md` §7).
     */
    fun drive(attempt: (Int) -> SyncOutcome): SyncOutcome {
        var exchanges = 0
        var outcome = attempt(++exchanges)

        while (outcome.error == SyncErrors.REFUSED &&
            afterRefusal(exchanges) == Step.ReExchangeAndRetry
        ) {
            outcome = attempt(++exchanges)
        }

        if (outcome.error == SyncErrors.REFUSED) {
            return SyncOutcome(
                ok = false,
                message = NEEDS_RECONNECT_MESSAGE,
                error = SyncErrors.REFUSED,
            )
        }

        return outcome
    }

    /**
     * The sentence for a pull that stopped because the key was refused
     * again.
     *
     * It says what happened and what is left to do, and it does not blame
     * the athlete: the key may have been rotated, or revoked at Garmin.
     */
    const val NEEDS_RECONNECT_MESSAGE =
        "Garmin refused the key again, so the sync stopped. A new key may be needed."
}
