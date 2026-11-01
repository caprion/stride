package app.stride.running.bridge

import org.json.JSONObject

/**
 * A request, as the core sees it.
 *
 * The method is how the request is made; the body is what it carries. They
 * are separate here, and they are separate in the core's router too.
 */
data class BridgeRequest(
    /** The method. `GET` unless the request names one. */
    val method: String,
    /** The body the core sees: the caller's body without the method. */
    val body: String?,
)

/**
 * The shaping of a request.
 *
 * **The method travels in the body.** The plugin's contract is
 * `apiCall(path, body)` — a path and a body — and the body is where the
 * method belongs: a method says how the request is made, and the body is
 * what the request carries. So a request that names its method in the body
 * is read that way, and the method is taken *out* of the body before the
 * core sees it: the core's router is asked for a method and a body, and
 * the body it gets is the one the caller wrote.
 *
 * A request that names the method as an argument of its own is read that
 * way too — the old app's bridge had that shape, and the two must not
 * drift apart.
 *
 * It is a plain object with no Android in it, so the shaping can be tested
 * without a phone.
 */
object BridgeRequestShape {

    /** The method a request that names none has. */
    const val DEFAULT_METHOD = "GET"

    /** The field the method travels in. */
    const val METHOD_FIELD = "method"

    /**
     * The request the core is asked for.
     *
     * `method` is the method named as an argument, if there is one;
     * `bodyJson` is the body as JSON, if there is one.
     */
    fun of(method: String?, bodyJson: String?): BridgeRequest {
        val body = bodyJson?.trim()?.takeIf { it.isNotEmpty() }
        val named = method?.trim()?.takeIf { it.isNotEmpty() }
            ?: body?.let { methodIn(it) }
            ?: DEFAULT_METHOD
        return BridgeRequest(named.uppercase(), withoutMethod(body))
    }

    /**
     * The method the body names, if it names one.
     *
     * A body that is not a JSON object names nothing: a bare string or an
     * array is a body and not a place to keep a method.
     */
    private fun methodIn(body: String): String? {
        val parsed = runCatching { JSONObject(body) }.getOrNull() ?: return null
        return parsed.optString(METHOD_FIELD).takeIf { it.isNotBlank() }
    }

    /**
     * The body without the method.
     *
     * A body that is not a JSON object is passed through as it is: there is
     * no method field to take out of it.
     */
    private fun withoutMethod(body: String?): String? {
        if (body == null) return null
        val parsed = runCatching { JSONObject(body) }.getOrNull() ?: return body
        if (!parsed.has(METHOD_FIELD)) return body
        parsed.remove(METHOD_FIELD)
        return parsed.toString()
    }
}
