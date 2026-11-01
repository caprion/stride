package app.stride.running.setup

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The setup screen's states, tested without a phone.
 *
 * The plan's flow is one line: *Connect → Checking… → Syncing… → Connected ·
 * 160 runs.* The states are the machine behind that line, and the machine
 * is here so it can be exercised without a phone.
 */
class SetupTest {

    @Test
    fun `the screen opens itself once, and not again`() {
        // No key, not dismissed: the screen opens. The athlete has to do it
        // once, and the app is useless without it.
        assertTrue(SetupEntry.shouldOpen(hasKey = false, dismissed = false))
    }

    @Test
    fun `a key means the screen has nothing to ask for`() {
        assertFalse(SetupEntry.shouldOpen(hasKey = true, dismissed = false))
    }

    @Test
    fun `a dismissal is a decision and it is kept`() {
        assertFalse(SetupEntry.shouldOpen(hasKey = false, dismissed = true))
    }

    @Test
    fun `the copy of each state is a string resource`() {
        val phases = SetupPhase.entries
        for (phase in phases) {
            val line = SetupCopy.status(SetupUi(phase))
            assertNotNull("$phase has no status line", line)
            assertTrue("$phase has no resource", line.resource != 0)
        }
    }

    @Test
    fun `the count only appears while the pull is running`() {
        assertNotNull(SetupCopy.progress(SetupUi(SetupPhase.Syncing, runs = 12)))
        assertEquals(null, SetupCopy.progress(SetupUi(SetupPhase.Idle)))
        assertEquals(null, SetupCopy.progress(SetupUi(SetupPhase.Connected, runs = 12)))
    }

    @Test
    fun `the connected line counts the runs`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Connected, runs = 160))
        assertTrue(line.plural)
        assertEquals(160, line.count)
    }

    @Test
    fun `the connected line without a count says only that it is connected`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Connected, runs = -1))
        assertEquals(null, line.count)
    }
}
