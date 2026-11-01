package app.stride.running.bridge

import org.json.JSONArray
import org.json.JSONObject

/**
 * The core's answer, as the app is handed it.
 *
 * The core answers with an envelope:
 *
 * ```text
 * {"ok": true,  "status": 200, "body": { ... }}
 * {"ok": false, "status": 404, "error": {"detail": "..."}}
 * ```
 *
 * **The app is handed the body, not the envelope.** The desktop's router
 * puts the body on the wire and the status in the status line, and the two
 * hosts must hand the web app the same bytes — otherwise a number could
 * differ between the phone and the rig without either of them being
 * wrong.
 *
 * A failure is a failure: it is not a body with an error in it, and the
 * app is told so by a rejection.
 *
 * It is a plain object with no Android in it, so the answer can be read
 * without a phone.
 */
data class BridgeAnswer(
    /** Whether the request succeeded. */
    val ok: Boolean,
    /** The response body, as text. `null` when there was no body. */
    val text: String?,
    /** The status the core answered with. */
    val status: Int,
    /** Why it failed, when it failed. */
    val detail: String?,
)

/**
 * How the core's envelope becomes the app's answer.
 */
object BridgeAnswerShape {

    /** The status a failure without one gets. */
    const val FALLBACK_STATUS = 500

    /**
     * The answer the app is handed.
     *
     * An envelope that is not JSON at all is a failure: the core always
     * answers with an envelope, so anything else is something the shell
     * cannot make sense of, and saying so is better than passing it on.
     */
    fun of(envelopeJson: String): BridgeAnswer {
        val envelope = runCatching { JSONObject(envelopeJson) }.getOrNull()
            ?: return BridgeAnswer(
                ok = false,
                text = null,
                status = FALLBACK_STATUS,
                detail = "the core answered with something that is not JSON",
            )

        val status = envelope.optInt("status", if (envelope.optBoolean("ok")) 200 else FALLBACK_STATUS)
        if (!envelope.optBoolean("ok")) {
            val detail = envelope.optJSONObject("error")?.optString("detail")
                ?: "the request could not be completed"
            return BridgeAnswer(ok = false, text = null, status = status, detail = detail)
        }

        return BridgeAnswer(ok = true, text = bodyText(envelope), status = status, detail = null)
    }

    /**
     * The response body, as the text the app parses.
     *
     * The body's shape is the core's business, so every shape is carried
     * through: an object, an array, a string, a number. A body that is
     * nothing at all is `null`, so the app reads "no answer" rather than a
     * half-written one.
     */
    fun bodyText(envelope: JSONObject): String {
        val body = envelope.opt("body")
        return when (body) {
            null -> "null"
            is JSONObject -> body.toString()
            is JSONArray -> body.toString()
            is String -> JSONObject.quote(body)
            else -> body.toString()
        }
    }
}
