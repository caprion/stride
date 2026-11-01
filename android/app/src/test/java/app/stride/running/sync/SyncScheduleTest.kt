package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.ZoneId
import java.time.ZonedDateTime

/**
 * The time the phone names for its next pull.
 */
class SyncScheduleTest {

    private val zone = ZoneId.of("Asia/Kolkata")

    @Test
    fun `the default cadence names the next local boundary`() {
        val now = ZonedDateTime.of(2026, 5, 12, 13, 30, 0, 0, zone)
        assertEquals(
            ZonedDateTime.of(2026, 5, 12, 14, 0, 0, 0, zone),
            SyncSchedule.nextBoundary(now, intervalHours = 6),
        )
    }

    @Test
    fun `the evening boundary is twenty hundred`() {
        val now = ZonedDateTime.of(2026, 5, 12, 19, 1, 0, 0, zone)
        assertEquals(
            ZonedDateTime.of(2026, 5, 12, 20, 0, 0, 0, zone),
            SyncSchedule.nextBoundary(now, intervalHours = 6),
        )
    }

    @Test
    fun `a boundary that has arrived is behind the phone`() {
        val now = ZonedDateTime.of(2026, 5, 12, 20, 0, 0, 0, zone)
        assertEquals(
            ZonedDateTime.of(2026, 5, 13, 2, 0, 0, 0, zone),
            SyncSchedule.nextBoundary(now, intervalHours = 6),
        )
    }
}
