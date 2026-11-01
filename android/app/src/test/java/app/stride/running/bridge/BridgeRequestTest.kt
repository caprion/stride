package app.stride.running.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The shaping of a request, tested without a phone.
 *
 * The web app's bridge sends `{ method, path, body }`; the plugin's
 * contract is `apiCall(path, body)`, and the method travels inside the
 * body. So the method is taken *out* of the body before the core sees it,
 * and the body the core gets is the one the caller wrote.
 */
class BridgeRequestTest {

    @Test
    fun `a request that names no method is a read`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = null)
        assertEquals("GET", request.method)
        assertNull(request.body)
    }

    @Test
    fun `a body that names no method is a read`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = """{"limit":10}""")
        assertEquals("GET", request.method)
        assertEquals("""{"limit":10}""", request.body)
    }

    /** The shim puts the method in the body; that is where it is read. */
    @Test
    fun `the method is read out of the body`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = """{"method":"POST","key":"x"}""")
        assertEquals("POST", request.method)
    }

    @Test
    fun `the method is taken out of the body the core sees`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = """{"method":"POST","key":"x"}""")
        val body = request.body ?: ""
        assertFalse("the core's body still has the method", body.contains("method"))
        assertTrue(body.contains("key"))
    }

    @Test
    fun `a method named as an argument is understood`() {
        val request = BridgeRequestShape.of(method = "post", bodyJson = """{"key":"x"}""")
        assertEquals("POST", request.method)
        assertEquals("""{"key":"x"}""", request.body)
    }

    @Test
    fun `an argument beats the body`() {
        val request = BridgeRequestShape.of(method = "GET", bodyJson = """{"method":"POST"}""")
        assertEquals("GET", request.method)
    }

    @Test
    fun `a body that is not an object is carried as it is`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = """[1,2,3]""")
        assertEquals("GET", request.method)
        assertEquals("""[1,2,3]""", request.body)
    }

    @Test
    fun `a body that is not JSON is carried as it is`() {
        val request = BridgeRequestShape.of(method = null, bodyJson = "not json at all")
        assertEquals("GET", request.method)
        assertEquals("not json at all", request.body)
    }

    @Test
    fun `an empty body is no body`() {
        val request = BridgeRequestShape.of(method = "GET", bodyJson = "   ")
        assertEquals("GET", request.method)
        assertNull(request.body)
    }
}
