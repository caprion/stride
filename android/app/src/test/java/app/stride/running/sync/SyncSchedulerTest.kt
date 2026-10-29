package app.stride.running.sync

import androidx.work.NetworkType
import androidx.work.OutOfQuotaPolicy
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The work request the app's "Sync now" enqueues.
 *
 * The request is the only thing the page hands the phone, so what it
 * carries — and what it does not — is the contract.
 */
class SyncSchedulerTest {

    @Test
    fun `the pull now is a one-time request for the sync worker`() {
        val request = SyncScheduler.oneTimeRequest(30)

        assertEquals(GarminSyncWorker::class.java.name, request.workSpec.workerClassName)
        assertFalse(
            "a one-time request must not be a periodic one",
            request.workSpec.isPeriodic,
        )
    }

    @Test
    fun `the request carries the periodic tag and the one-shot tag`() {
        val request = SyncScheduler.oneTimeRequest(30)

        // Both, because the app's "is a pull running" answer comes from the
        // tags: a query for the shared tag and a query for the one-shot name
        // must both find this pull. The third tag is WorkManager's own — the
        // worker's class name.
        assertTrue(request.tags.contains(SyncScheduler.PERIODIC_NAME))
        assertTrue(request.tags.contains(SyncScheduler.ONESHOT_NAME))
        assertEquals(
            setOf(
                SyncScheduler.PERIODIC_NAME,
                SyncScheduler.ONESHOT_NAME,
                GarminSyncWorker::class.java.name,
            ),
            request.tags,
        )
    }

    @Test
    fun `the window travels with the work and the key does not`() {
        val request = SyncScheduler.oneTimeRequest(30)

        assertEquals(30, request.workSpec.input.getInt(SyncScheduler.KEY_DAYS, 0))
        // One entry, and it is a number of days: the Garmin key is never in
        // WorkManager's database.
        assertEquals(1, request.workSpec.input.size())
    }

    @Test
    fun `the window is clamped to what the core accepts`() {
        assertEquals(
            1,
            SyncScheduler.oneTimeRequest(0).workSpec.input.getInt(SyncScheduler.KEY_DAYS, 0),
        )
        assertEquals(
            90,
            SyncScheduler.oneTimeRequest(400).workSpec.input.getInt(SyncScheduler.KEY_DAYS, 0),
        )
    }

    @Test
    fun `the pull needs a network`() {
        val request = SyncScheduler.oneTimeRequest(30)

        assertEquals(
            NetworkType.CONNECTED,
            request.workSpec.constraints.requiredNetworkType,
        )
    }

    @Test
    fun `the pull is expedited, so the phone does not wait six hours for it`() {
        val request = SyncScheduler.oneTimeRequest(30)

        assertTrue(request.workSpec.expedited)
        assertEquals(
            OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST,
            request.workSpec.outOfQuotaPolicy,
        )
    }

    @Test
    fun `the periodic request is the other one, and it is periodic`() {
        val request = SyncScheduler.periodicRequest(6)

        assertEquals(GarminSyncWorker::class.java.name, request.workSpec.workerClassName)
        assertTrue(request.workSpec.isPeriodic)
        assertEquals(
            setOf(SyncScheduler.PERIODIC_NAME, GarminSyncWorker::class.java.name),
            request.tags,
        )
    }
}
