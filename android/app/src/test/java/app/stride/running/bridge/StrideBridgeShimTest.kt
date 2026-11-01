package app.stride.running.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.mozilla.javascript.Context
import org.mozilla.javascript.Scriptable
import org.mozilla.javascript.ScriptableObject
import java.io.File

/**
 * The bridge shim, tested by running it.
 *
 * The shim is JavaScript, and it is the piece the web app talks to. A shim
 * that is only read is a shim that is not tested, so **the shipped file is
 * evaluated here** — `src/main/assets/public/stride-bridge.js`, the very
 * file the APK serves — against a fake plugin, and the test asserts what
 * the web app would see.
 *
 * The web app's transport is:
 *
 * ```ts
 * const answer = await window.__stride_bridge__.apiCall({ method, path, body })
 * return answer ? JSON.parse(answer) : null
 * ```
 *
 * so the shim has to take `{ method, path, body }`, hand it to the plugin,
 * and answer with the plugin's JSON string.
 */
class StrideBridgeShimTest {

    /** The shim, as it ships. */
    private val shim: String by lazy {
        val file = File("src/main/assets/public/stride-bridge.js")
        assertTrue("the shim is at ${file.absolutePath}", file.isFile)
        file.readText()
    }

    /**
     * The fake Capacitor runtime.
     *
     * It is the shape of the real one: `window.Capacitor.Plugins.StrideApi`
     * with an `apiCall` that takes one options object and answers with a
     * thenable. The call is recorded, so the test can see what the shim
     * asked for.
     *
     * The thenable chains the way a promise chains: a failure with nobody
     * to tell it to stays failed, so the shim cannot turn a failure into an
     * answer.
     */
    private fun fakePlugin(answer: String, fails: Boolean): String = """
        var __calls = [];
        window.Capacitor = {
          Plugins: {
            StrideApi: {
              apiCall: function (options) {
                __calls.push(options);
                var __failed = $fails;
                var __answer = $answer;
                var __fail = function (onFulfilled, onRejected) {
                  if (__failed) {
                    var __reason = new Error('the route is not there');
                    if (onRejected) { return onRejected(__reason); }
                    return { then: function (next, nextRejected) {
                      return nextRejected ? nextRejected(__reason) : this;
                    } };
                  }
                  return onFulfilled ? onFulfilled(__answer) : null;
                };
                return { then: __fail };
              }
            }
          }
        };
    """.trimIndent()

    /** A scope with the fake runtime and the shim installed in it. */
    private fun page(
        context: Context,
        answer: String = "{ json: '{}' }",
        fails: Boolean = false,
    ): ScriptableObject {
        val scope = context.initSafeStandardObjects()
        scope.put("window", scope, scope)
        context.evaluateString(scope, fakePlugin(answer, fails), "fake-capacitor.js", 1, null)
        context.evaluateString(scope, shim, "stride-bridge.js", 1, null)
        return scope
    }

    /** The recorded call, as a map. */
    private fun firstCall(scope: ScriptableObject): Map<*, *> {
        val calls = scope.get("__calls", scope) as? Scriptable
        assertNotNull("the plugin was not called", calls)
        val size = (calls!!.get("length", scope) as Number).toInt()
        assertEquals("the plugin was called once", 1, size)
        return calls.get(0, scope) as Map<*, *>
    }

    @Test
    fun `the shim installs the bridge the app talks to`() {
        val context = Context.enter()
        try {
            val scope = page(context)
            val bridge = scope.get("__stride_bridge__", scope) as? ScriptableObject
            assertNotNull("the bridge was not installed", bridge)
            assertNotNull(
                "the bridge has no apiCall",
                bridge?.get("apiCall", bridge),
            )
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `the app's shape goes to the plugin with the method in the body`() {
        val context = Context.enter()
        try {
            val scope = page(context)
            context.evaluateString(
                scope,
                "window.__stride_bridge__.apiCall({ method: 'POST', path: '/api/v1/activities', body: { limit: 10 } });",
                "call.js",
                1,
                null,
            )

            val call = firstCall(scope)
            assertEquals("/api/v1/activities", call["path"])
            val body = call["body"] as Map<*, *>
            assertEquals("the method did not travel in the body", "POST", body["method"])
            assertEquals(10.0, (body["limit"] as Number).toDouble(), 0.0)
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `the plugin's answer comes back as the JSON string`() {
        val context = Context.enter()
        try {
            val scope = page(context, answer = "{ json: '{\"items\":[1,2]}' }")
            val answer = context.evaluateString(
                scope,
                "window.__stride_bridge__.apiCall({ method: 'GET', path: '/api/v1/activities' })",
                "call.js",
                1,
                null,
            )
            assertEquals("{\"items\":[1,2]}", answer as String)
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `an error from the plugin comes back as an error`() {
        val context = Context.enter()
        try {
            val scope = page(context, fails = true)
            val outcome = context.evaluateString(
                scope,
                """
                var __outcome = 'nothing';
                window.__stride_bridge__.apiCall({ method: 'GET', path: '/api/v1/nope' })
                  .then(function (value) { __outcome = 'answered: ' + value; },
                        function (error) { __outcome = 'failed: ' + error.message; });
                __outcome;
                """.trimIndent(),
                "call.js",
                1,
                null,
            )
            assertEquals("failed: the route is not there", outcome as String)
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `the app's own body is not changed`() {
        val context = Context.enter()
        try {
            val scope = page(context)
            val outcome = context.evaluateString(
                scope,
                """
                var __body = { limit: 10 };
                window.__stride_bridge__.apiCall({ method: 'POST', path: '/x', body: __body });
                __body.method === undefined ? 'untouched' : 'changed';
                """.trimIndent(),
                "call.js",
                1,
                null,
            )
            assertEquals("the app's body was changed", "untouched", outcome as String)
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `a request with no body still has a method`() {
        val context = Context.enter()
        try {
            val scope = page(context)
            context.evaluateString(
                scope,
                "window.__stride_bridge__.apiCall({ path: '/api/v1/health' });",
                "call.js",
                1,
                null,
            )

            val body = firstCall(scope)["body"] as Map<*, *>
            assertEquals("GET", body["method"])
        } finally {
            Context.exit()
        }
    }

    @Test
    fun `the shim adds no route of its own`() {
        // The shim forwards; it does not decide. The route table, the
        // refusal of the key routes and the redaction are the plugin's, and
        // the shim names no route.
        assertTrue(shim.contains("StrideApi"))
        assertFalse("the shim names a route", shim.contains("/activities"))
        assertFalse("the shim names a route", shim.contains("/api/v1"))
    }
}
