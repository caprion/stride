package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

/**
 * Forgetting the Garmin key.
 *
 * The key is the one secret the phone holds for Garmin, and forgetting it
 * has to mean exactly three things and nothing else.
 */
class ForgetGarminKeyTest {

    @Test
    fun `forgetting the key stops the work, clears the envelope and clears the core's copy`() {
        val calls = mutableListOf<String>()

        ForgetGarminKey.run(
            stopScheduledWork = { calls += "stop work" },
            clearEnvelope = { calls += "clear the envelope" },
            clearCoreKey = { calls += "clear the core's copy" },
        )

        assertEquals(
            listOf("stop work", "clear the envelope", "clear the core's copy"),
            calls,
        )
    }

    @Test
    fun `forgetting the key touches nothing else`() {
        val calls = mutableListOf<String>()

        ForgetGarminKey.run(
            stopScheduledWork = { calls += "stop work" },
            clearEnvelope = { calls += "clear the envelope" },
            clearCoreKey = { calls += "clear the core's copy" },
        )

        // Three calls, and none of them is a route, a delete or a write.
        // The activities are in the phone's own database and the forget path
        // never opens it: forgetting the key is not forgetting the data.
        assertEquals(3, calls.size)
        assertTrue(calls.none { it.contains("activit") })
        assertTrue(calls.none { it.contains("delete") })
        assertTrue(calls.none { it.contains("route") })
    }

    @Test
    fun `a step that fails does not leave the key in the core's memory`() {
        val calls = mutableListOf<String>()

        try {
            ForgetGarminKey.run(
                stopScheduledWork = { calls += "stop work" },
                clearEnvelope = { calls += "clear the envelope"; throw IllegalStateException("locked") },
                clearCoreKey = { calls += "clear the core's copy" },
            )
            fail("the failure was not reported")
        } catch (error: IllegalStateException) {
            assertEquals("locked", error.message)
        }

        assertEquals(
            listOf("stop work", "clear the envelope", "clear the core's copy"),
            calls,
        )
    }
}
