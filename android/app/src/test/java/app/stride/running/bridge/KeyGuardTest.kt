package app.stride.running.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The last line of defence, tested without a phone.
 *
 * The key is never *given* to the page: the routes that would carry it are
 * refused before they reach the core, and the core never answers with it.
 * This is the belt to that pair of braces.
 */
class KeyGuardTest {

    /**
     * A placeholder credential.
     *
     * A run of `x`s: obviously not a key, and shaped so the repository's own
     * scanners and GitHub's push protection both leave it alone. What the
     * test needs is a string that is not in the answer — any string will do.
     */
    private val key = "xxxxxxxxxxxxxxxx"

    @Test
    fun `an answer without the key is handed on unchanged`() {
        val answer = """{"ok":true,"status":200,"body":{"activities":[]}}"""
        assertEquals(answer, KeyGuard.redact(answer, key))
        assertFalse(KeyGuard.carriesKey(answer, key))
    }

    @Test
    fun `an answer with the key is refused outright`() {
        val answer = """{"ok":true,"status":200,"body":{"api_key":"xxxxxxxxxxxxxxxx"}}"""
        assertTrue(KeyGuard.carriesKey(answer, key))
        val redacted = KeyGuard.redact(answer, key)
        assertFalse(redacted.contains(key))
        assertTrue(redacted.contains(KeyGuard.REDACTED))
    }

    @Test
    fun `an escaped key counts as the key`() {
        val awkward = """a"b\c"""
        val answer = """{"ok":true,"body":{"key":"a\"b\\c"}}"""
        assertTrue(KeyGuard.carriesKey(answer, awkward))
        assertFalse(KeyGuard.redact(answer, awkward).contains("a\\\"b"))
    }

    @Test
    fun `no key means nothing to find`() {
        val answer = """{"ok":true,"body":{"api_key":"xxxxxxxxxxxxxxxx"}}"""
        assertFalse(KeyGuard.carriesKey(answer, null))
        assertFalse(KeyGuard.carriesKey(answer, ""))
        assertEquals(answer, KeyGuard.redact(answer, null))
    }
}
