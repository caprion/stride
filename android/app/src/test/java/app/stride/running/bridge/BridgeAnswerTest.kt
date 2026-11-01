package app.stride.running.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The answer the app is handed, tested without a phone.
 *
 * **The app is handed the body, not the envelope.** The desktop's router
 * puts the body on the wire; the phone must hand the web app the same
 * bytes, or a number could differ between the two hosts without either of
 * them being wrong.
 */
class BridgeAnswerTest {

    @Test
    fun `a successful answer hands over the body`() {
        val answer = BridgeAnswerShape.of("""{"ok":true,"status":200,"body":{"items":[1,2]}}""")
        assertTrue(answer.ok)
        assertEquals("""{"items":[1,2]}""", answer.text)
        assertEquals(200, answer.status)
        assertNull(answer.detail)
    }

    @Test
    fun `a body that is an array is carried through`() {
        val answer = BridgeAnswerShape.of("""{"ok":true,"status":200,"body":[1,2,3]}""")
        assertTrue(answer.ok)
        assertEquals("[1,2,3]", answer.text)
    }

    @Test
    fun `a body that is a string is carried through as a string`() {
        val answer = BridgeAnswerShape.of("""{"ok":true,"status":200,"body":"hello"}""")
        assertTrue(answer.ok)
        assertEquals("\"hello\"", answer.text)
    }

    @Test
    fun `a body that is nothing is null`() {
        val answer = BridgeAnswerShape.of("""{"ok":true,"status":204}""")
        assertTrue(answer.ok)
        assertEquals("null", answer.text)
    }

    @Test
    fun `a failure is a failure and not a body`() {
        val answer = BridgeAnswerShape.of(
            """{"ok":false,"status":404,"error":{"detail":"there is no such route"}}""",
        )
        assertFalse(answer.ok)
        assertNull(answer.text)
        assertEquals(404, answer.status)
        assertEquals("there is no such route", answer.detail)
    }

    @Test
    fun `a failure without a detail still says something`() {
        val answer = BridgeAnswerShape.of("""{"ok":false,"status":500}""")
        assertFalse(answer.ok)
        assertEquals(500, answer.status)
        assertTrue(answer.detail?.isNotBlank() == true)
    }

    @Test
    fun `an answer that is not JSON is a failure and not a success`() {
        val answer = BridgeAnswerShape.of("this is not JSON")
        assertFalse(answer.ok)
        assertEquals(BridgeAnswerShape.FALLBACK_STATUS, answer.status)
        assertTrue(answer.detail?.isNotBlank() == true)
    }
}
