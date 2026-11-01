package app.stride.running.security

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * The only part of a Garmin key a native surface may show.
 */
class KeyIdentifierTest {

    @Test
    fun `the identifier is the last four characters`() {
        assertEquals("9f3a", KeyIdentifier.lastFour("xxxxxxxxxxxxxxxxxxxx9f3a"))
    }

    @Test
    fun `the identifier never contains more than four key characters`() {
        val identifier = KeyIdentifier.lastFour("abcdefghijklmnop")
        assertEquals(4, identifier?.length)
        assertEquals("mnop", identifier)
    }

    @Test
    fun `there is no identifier without a key`() {
        assertNull(KeyIdentifier.lastFour(null))
        assertNull(KeyIdentifier.lastFour(""))
        assertNull(KeyIdentifier.lastFour("   "))
    }
}
