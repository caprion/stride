package app.stride.running.sync

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The bound on what one pull may ask Garmin for.
 */
class PullWindowTest {

    @Test
    fun `the ordinary history target is about twelve months`() {
        assertEquals(365L, PullWindow.DEFAULT_DAYS)
        assertEquals(30L, PullWindow.bounded(30L))
    }

    @Test
    fun `a wider request is bounded`() {
        assertEquals(PullWindow.MAX_DAYS, PullWindow.bounded(10000L))
    }

    @Test
    fun `an empty request has a smallest useful window`() {
        assertEquals(PullWindow.MIN_DAYS, PullWindow.bounded(0L))
        assertEquals(PullWindow.MIN_DAYS, PullWindow.bounded(-10L))
    }
}
