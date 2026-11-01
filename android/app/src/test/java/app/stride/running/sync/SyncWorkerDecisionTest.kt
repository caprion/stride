package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The background job's answer after a pull.
 */
class SyncWorkerDecisionTest {

    @Test
    fun `a finished pull succeeds`() {
        assertEquals(
            SyncWorkerDecision.Decision.Success,
            SyncWorkerDecision.decide(SyncOutcome(ok = true, message = "finished")),
        )
    }

    @Test
    fun `an exchange failure stops the job after the shell's retry`() {
        assertEquals(
            SyncWorkerDecision.Decision.Failure,
            SyncWorkerDecision.decide(
                SyncOutcome(ok = false, message = "unreachable", error = SyncErrors.FAILED),
            ),
        )
    }

    @Test
    fun `a locked phone waits because no exchange happened`() {
        assertEquals(
            SyncWorkerDecision.Decision.WaitForUnlock,
            SyncWorkerDecision.decide(
                SyncOutcome(ok = false, message = "locked", error = SyncErrors.LOCKED),
            ),
        )
    }
}
