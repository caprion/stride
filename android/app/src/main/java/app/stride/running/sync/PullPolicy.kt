package app.stride.running.sync

/**
 * What a pull is allowed to do about a failed attempt.
 *
 * Garmin's Personal API key is exchanged for a bearer token that lasts
 * about a day, and **Garmin has no refresh token**: a "refresh" is a second
 * exchange of the same key. So a 401 is not a dead end — it is a sign that
 * the exchange has to happen again.
 *
 * But it is not a loop either. The rule is:
 *
 * * the client makes **one** token exchange and **one** HTTP attempt;
 * * the shell makes **one** more attempt;
 * * and then the pull stops and the failure is surfaced.
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

    /** How many retries there are. One. */
    const val MAX_RETRIES = 1

    /** How many re-exchanges there are. One. */
    const val MAX_RE_EXCHANGES = MAX_RETRIES

    /** What the pull does next. */
    enum class Step {
        /** Make one more attempt. It may perform one fresh token exchange. */
        Retry,

        /** Stop. The failure is surfaced to the phone. */
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
     * What a pull does after an attempt.
     *
     * `attempt` is the attempt's number — 1, then 2. The first failure may
     * be followed by one retry; the second failure stops the pull.
     */
    fun afterAttempt(attempt: Int): Step =
        if (attempt <= MAX_RETRIES) Step.Retry else Step.Stop

    /**
     * What the pull does after a refused key.
     *
     * `exchanges` is how many exchanges have already happened for this
     * pull, including the first one.
     */
    fun afterRefusal(exchanges: Int): Step = afterAttempt(exchanges)

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
     * * a first attempt that fails is retried **once**;
     * * a second failure stops the pull, and the phone says what happened.
     *
     * The attempt is the seam's: one call, one token exchange, one HTTP
     * attempt (see `docs/contract/core-contract.md` §7).
     */
    fun drive(attempt: (Int) -> SyncOutcome): SyncOutcome {
        var attemptNumber = 1
        var outcome = attempt(attemptNumber)

        while (!outcome.ok && retryable(outcome) && afterAttempt(attemptNumber) == Step.Retry) {
            attemptNumber += 1
            outcome = attempt(attemptNumber)
        }

        return outcome
    }

    /**
     * Whether a failed exchange is one the shell may try once more.
     *
     * A refused key and an unreachable Garmin are both one more exchange.
     * A local condition — a locked phone or no credential — is not: those
     * are checked before an exchange starts and never reach this policy.
     */
    private fun retryable(outcome: SyncOutcome): Boolean =
        outcome.error == SyncErrors.REFUSED || outcome.error == SyncErrors.FAILED
}
