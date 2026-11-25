package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * What a pull asks for, read out of the body.
 *
 * The body of `/sync/run` says whether the pull is the **older history**,
 * whether it runs in the background, and how wide a window it wants. The
 * phone reads the body in one place, so the rules are here — and they are
 * tested without a phone.
 */
class SyncRequestTest {

    @Test
    fun `a body that names nothing asks for the recent pull`() {
        assertFalse(SyncRequest.asksForHistory(null))
        assertFalse(SyncRequest.asksForHistory(""))
        assertFalse(SyncRequest.asksForHistory("{}"))
        assertFalse(SyncRequest.asksForHistory("""{"backfill":false}"""))
    }

    @Test
    fun `the older history is asked for by name`() {
        assertTrue(SyncRequest.asksForHistory("""{"backfill":true}"""))
    }

    @Test
    fun `a body that is not an object names nothing`() {
        assertFalse(SyncRequest.asksForHistory("not json"))
        assertFalse(SyncRequest.asksForHistory("[1,2]"))
        assertTrue(SyncRequest.asksForBackground("not json"))
    }

    @Test
    fun `the window is bounded`() {
        assertEquals(30, SyncRequest.daysOf("""{"days":30}""", 365))
        assertEquals(
            PullWindow.MAX_DAYS.toInt(),
            SyncRequest.daysOf("""{"days":10000}""", 365),
        )
    }

    @Test
    fun `a window that names nothing is the phone's own`() {
        assertEquals(365, SyncRequest.daysOf(null, 365))
        assertEquals(365, SyncRequest.daysOf("{}", 365))
        assertEquals(365, SyncRequest.daysOf("""{"days":0}""", 365))
        assertEquals(365, SyncRequest.daysOf("""{"days":-5}""", 365))
    }

    @Test
    fun `the pull runs in the background unless the body says otherwise`() {
        assertTrue(SyncRequest.asksForBackground(null))
        assertTrue(SyncRequest.asksForBackground("{}"))
        assertFalse(SyncRequest.asksForBackground("""{"background":false}"""))
        assertTrue(SyncRequest.asksForBackground("""{"background":true}"""))
    }
}
