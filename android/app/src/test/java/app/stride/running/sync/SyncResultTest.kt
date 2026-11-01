package app.stride.running.sync

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The reads of the api's DTOs, tested without a phone.
 *
 * **A field that is not in the DTO reads as zero**, and a phone that says
 * "0 runs" after a real pull is worse than no phone at all. So the reads
 * are pinned to the DTO's own names:
 *
 * ```text
 * SyncRun    { activities, wellness_days, finished_at, generated_at }
 * SyncStatus { last_sync_at, last_error, activity_count }
 * ```
 *
 * The second test is the one that matters: it feeds the shell the fields
 * an earlier version of the shell read — `activities_found`,
 * `activities_new`, a `total` and an `errors` array — and asserts that
 * they are **not** what the shell reads. If the DTO ever grows one of
 * them, this test says so and the read can be written deliberately.
 */
class SyncResultTest {

    @Test
    fun `a pull's counts come from the DTO's fields`() {
        val body = JSONObject(
            """{"activities":12,"wellness_days":30,"finished_at":"2026-03-07T12:00:00Z","generated_at":"2026-03-07T12:00:01Z"}""",
        )
        val counts = SyncResult.counts(body)
        assertEquals(12, counts.activities)
        assertEquals(30, counts.wellnessDays)
    }

    @Test
    fun `the fields an earlier shell read are not the DTO's fields`() {
        val body = JSONObject(
            """{"activities_found":12,"activities_new":3,"total":160,"errors":[{"error":"a problem"}]}""",
        )
        val counts = SyncResult.counts(body)
        assertEquals(0, counts.activities)
        assertEquals(0, counts.wellnessDays)
    }

    @Test
    fun `a pull that brought nothing says zero`() {
        val body = JSONObject("""{"activities":0,"wellness_days":0}""")
        val counts = SyncResult.counts(body)
        assertEquals(0, counts.activities)
        assertEquals(0, counts.wellnessDays)
    }

    @Test
    fun `no body is no counts`() {
        val counts = SyncResult.counts(null)
        assertEquals(0, counts.activities)
        assertEquals(0, counts.wellnessDays)
    }

    @Test
    fun `the runs the phone has come from the status DTO`() {
        val body = JSONObject(
            """{"last_sync_at":"2026-03-07T12:00:00Z","last_error":null,"activity_count":160}""",
        )
        assertEquals(160, SyncResult.storedRuns(body))
    }

    @Test
    fun `the feed's page is not the phone's history`() {
        // The feed has no total, and a page of ten is not the count.
        val page = JSONObject("""{"activities":[],"next_cursor":null,"generated_at":"2026-03-07T12:00:00Z"}""")
        assertEquals(0, SyncResult.storedRuns(page))
    }
}
