package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The background job's answer after a pull.
 */
class SyncWorkerDecisionTest {

    @Test
    fun `only the history worker resumes a normal budget pause`() {
        val outcome = SyncOutcome(ok = true, message = "more history", morePending = true)
        assertEquals(SyncWorkerDecision.Decision.Success, SyncWorkerDecision.decide(outcome))
        assertEquals(SyncWorkerDecision.Decision.Continue, SyncWorkerDecision.decide(outcome, backfill = true))
    }

    @Test
    fun `a transient failure waits while a refusal stops`() {
        assertEquals(
            SyncWorkerDecision.Decision.Continue,
            SyncWorkerDecision.decide(SyncOutcome(false, "rate limited", error = SyncErrors.RATE_LIMITED, retryable = true)),
        )
        assertEquals(
            SyncWorkerDecision.Decision.Failure,
            SyncWorkerDecision.decide(SyncOutcome(false, "refused", error = SyncErrors.REFUSED)),
        )
    }

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
