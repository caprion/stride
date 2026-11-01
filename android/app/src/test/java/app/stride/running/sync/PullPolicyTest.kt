package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The one-retry rule, tested without a phone.
 *
 * *"The shell owns the retry. The client does not."*
 * (`docs/contract/core-contract.md` §7 — "The retry rule, and who owns
 * it")
 *
 * The client performs exactly one token exchange and one HTTP attempt per
 * call and answers `needs_reconnect` when the token has expired. The caller
 * may make **one** more attempt, and then it stops.
 *
 * The contract says both sides assert it: the client test asserts one
 * exchange per call, and this one asserts one retry per sync.
 */
class PullPolicyTest {

    /** An attempt that was refused. */
    private fun refused(): SyncOutcome =
        SyncOutcome(ok = false, message = "the token has expired", error = SyncErrors.REFUSED)

    /** An attempt that worked. */
    private fun worked(): SyncOutcome =
        SyncOutcome(ok = true, message = "Imported 12 new runs", activities = 12)

    @Test
    fun `the first refusal leads to one re-exchange`() {
        assertEquals(
            PullPolicy.Step.ReExchangeAndRetry,
            PullPolicy.afterRefusal(exchanges = 1),
        )
    }

    @Test
    fun `the refusal after the re-exchange stops the pull`() {
        assertEquals(
            PullPolicy.Step.Stop,
            PullPolicy.afterRefusal(exchanges = 2),
        )
    }

    @Test
    fun `there is never a third exchange`() {
        for (exchanges in 3..10) {
            assertEquals(
                "a pull exchanged $exchanges times",
                PullPolicy.Step.Stop,
                PullPolicy.afterRefusal(exchanges),
            )
        }
    }

    @Test
    fun `there is exactly one re-exchange`() {
        assertEquals(1, PullPolicy.MAX_RE_EXCHANGES)
    }

    /** One retry per sync: a refusal is tried again exactly once. */
    @Test
    fun `a refused pull is retried exactly once`() {
        var attempts = 0
        val outcome = PullPolicy.drive { _ ->
            attempts += 1
            refused()
        }

        assertEquals("the pull made two attempts", 2, attempts)
        assertFalse(outcome.ok)
        assertEquals(SyncErrors.REFUSED, outcome.error)
    }

    /** And never a third time, however long the key is refused. */
    @Test
    fun `a pull that is refused again stops there`() {
        var attempts = 0
        PullPolicy.drive { _ ->
            attempts += 1
            refused()
        }

        assertEquals("the pull made two attempts", 2, attempts)
    }

    @Test
    fun `a pull that works first time is not retried`() {
        var attempts = 0
        val outcome = PullPolicy.drive { _ ->
            attempts += 1
            worked()
        }

        assertEquals("the pull made one attempt", 1, attempts)
        assertTrue(outcome.ok)
        assertEquals(12, outcome.activities)
    }

    @Test
    fun `a pull that works on the retry stops there`() {
        var attempts = 0
        val outcome = PullPolicy.drive { number ->
            attempts += 1
            if (number == 1) refused() else worked()
        }

        assertEquals("the pull made two attempts", 2, attempts)
        assertTrue(outcome.ok)
    }

    @Test
    fun `a refusal is a refusal whatever shape it takes`() {
        assertTrue(PullPolicy.isRefusal(status = 401, code = null))
        assertTrue(PullPolicy.isRefusal(status = 0, code = "needs_reconnect"))
        assertTrue(PullPolicy.isRefusal(status = 0, code = "needs_reconnect".uppercase()))
        assertTrue(PullPolicy.isRefusal(status = 0, code = "garmin_rejected"))
    }

    @Test
    fun `another failure is not a refusal`() {
        assertFalse(PullPolicy.isRefusal(status = 500, code = null))
        assertFalse(PullPolicy.isRefusal(status = 0, code = "stale_data"))
        assertFalse(PullPolicy.isRefusal(status = 404, code = null))
    }

    @Test
    fun `the sentence says what happened and does not blame the athlete`() {
        assertEquals(
            "Garmin refused the key again, so the sync stopped. A new key may be needed.",
            PullPolicy.NEEDS_RECONNECT_MESSAGE,
        )
    }
}
