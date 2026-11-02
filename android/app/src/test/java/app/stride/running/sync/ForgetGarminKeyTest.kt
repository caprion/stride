package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * What forgetting the Garmin key does, and what it leaves alone.
 */
class ForgetGarminKeyTest {

    @Test
    fun `the work stops before the envelope and the core copy are cleared`() {
        val order = mutableListOf<String>()
        ForgetGarminKey.run(
            stopScheduledWork = { order += "work" },
            clearEnvelope = { order += "envelope" },
            clearCoreKey = { order += "core" },
        )
        assertEquals(listOf("work", "envelope", "core"), order)
    }

    @Test
    fun `all three steps are attempted even when one fails`() {
        val order = mutableListOf<String>()
        try {
            ForgetGarminKey.run(
                stopScheduledWork = { order += "work" },
                clearEnvelope = { throw IllegalStateException("envelope") },
                clearCoreKey = { order += "core" },
            )
        } catch (_: IllegalStateException) {
            // The first failure is reported after every step was tried.
        }
        assertEquals(listOf("work", "core"), order)
    }

    @Test
    fun `the report keeps the local database`() {
        val report = ForgetGarminKey.Report(
            scheduledPullsStopped = true,
            credentialEnvelopeRemoved = true,
            wrappingKeyRemoved = true,
            memoryCopyCleared = true,
        )
        assertTrue(report.databaseKept)
    }
}
