package app.stride.running.setup

import app.stride.running.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * The screen's states, and the words each of them shows.
 *
 * The words themselves live in `res/values/strings.xml`; these are the
 * decisions about which of them the screen says, and when.
 */
class SetupStateTest {

    @Test
    fun `no key yet`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Idle))
        assertEquals(R.string.garmin_not_connected, line.resource)
        assertNull(line.count)
        assertNull(SetupCopy.progress(SetupUi(SetupPhase.Idle)))
    }

    @Test
    fun `the key is being checked`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Checking))
        assertEquals(R.string.garmin_checking, line.resource)
        assertNull(SetupCopy.progress(SetupUi(SetupPhase.Checking)))
    }

    @Test
    fun `the pull is running, and the count is with it`() {
        val state = SetupUi(SetupPhase.Syncing, runs = 12)
        assertEquals(R.string.garmin_syncing, SetupCopy.status(state).resource)

        val progress = SetupCopy.progress(state)
        assertEquals(R.plurals.garmin_runs, progress?.resource)
        assertEquals(12, progress?.count)
        assertEquals(true, progress?.plural)
    }

    @Test
    fun `connected, with the number of runs`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Connected, runs = 160))
        assertEquals(R.plurals.garmin_connected_runs, line.resource)
        assertEquals(160, line.count)
        assertEquals(true, line.plural)
        // The count is in the status line, so there is no second line.
        assertNull(SetupCopy.progress(SetupUi(SetupPhase.Connected, runs = 160)))
    }

    @Test
    fun `connected, before the runs have been counted`() {
        // A negative count means "not counted yet", and the screen says
        // "Connected" rather than "Connected · 0 runs".
        val line = SetupCopy.status(SetupUi(SetupPhase.Connected, runs = -1))
        assertEquals(R.string.garmin_connected, line.resource)
        assertNull(line.count)
    }

    @Test
    fun `refused`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Refused))
        assertEquals(R.string.garmin_refused, line.resource)
        assertNull(line.count)
    }

    @Test
    fun `the pull did not finish`() {
        val line = SetupCopy.status(SetupUi(SetupPhase.Failed, detail = "the phone is locked"))
        assertEquals(R.string.garmin_failed, line.resource)
    }

    @Test
    fun `the setup screen opens itself once, and only without a key`() {
        // The first launch: no key, and the athlete has not said no.
        assertEquals(true, SetupEntry.shouldOpen(hasKey = false, dismissed = false))

        // A key is there: the app has what it needs, and the screen never
        // comes back.
        assertEquals(false, SetupEntry.shouldOpen(hasKey = true, dismissed = false))

        // The athlete dismissed it. That is a decision, and it is kept.
        assertEquals(false, SetupEntry.shouldOpen(hasKey = false, dismissed = true))

        // Both.
        assertEquals(false, SetupEntry.shouldOpen(hasKey = true, dismissed = true))
    }
}
