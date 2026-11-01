package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The shell's reading of Garmin's failures.
 */
class GarminFailureTest {

    @Test
    fun `a refused key is a refused key`() {
        assertEquals(
            GarminFailure.Kind.KeyRejected,
            GarminFailure.classify(401, "garmin_rejected", "Garmin refused the Personal API key"),
        )
    }

    @Test
    fun `an expired session is a refused key`() {
        assertEquals(
            GarminFailure.Kind.KeyRejected,
            GarminFailure.classify(401, "needs_reconnect", "the Garmin session ended"),
        )
    }

    @Test
    fun `an unreachable Garmin is not a refused key`() {
        assertEquals(
            GarminFailure.Kind.Unreachable,
            GarminFailure.classify(401, "garmin_rejected", "Garmin could not be reached"),
        )
    }

    @Test
    fun `a rate limit is not a refused key`() {
        assertEquals(
            GarminFailure.Kind.RateLimited,
            GarminFailure.classify(401, "garmin_rejected", "Garmin is rate limiting the request"),
        )
    }
}
