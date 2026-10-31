package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The one-retry rule, tested without a phone.
 *
 * *"The Garmin access token is short-lived (24 h) and disposable: a 401
 * triggers exactly one re-exchange and one retry, never a loop."*
 */
class PullPolicyTest {

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

    @Test
    fun `the sentence says what happened and does not blame the athlete`() {
        assertEquals(
            "Garmin refused the key again, so the sync stopped. A new key may be needed.",
            PullPolicy.NEEDS_RECONNECT,
        )
    }
}
