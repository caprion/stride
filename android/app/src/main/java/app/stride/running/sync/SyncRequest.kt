package app.stride.running.sync

import org.json.JSONObject

/**
 * What a pull asks for.
 *
 * The body of `/sync/run` says three things, and the phone reads them in
 * one place:
 *
 * * **whether the pull runs in the background** — the phone's own job, or
 *   the app's own thread;
 * * **whether it is the older history** — the dated checkpointed walk
 *   back through the months the athlete asked for;
 * * **how wide a window it wants** — bounded, always, by
 *   [PullWindow].
 *
 * **It is a plain object with no Android in it**, so the rules can be
 * tested without a phone: the same discipline as [PullPolicy] and
 * `PhoneRoutes`.
 */
object SyncRequest {

    /** The field that asks for the older history. */
    const val FIELD_BACKFILL = "backfill"

    /** The field that asks for the pull to run in the background. */
    const val FIELD_BACKGROUND = "background"

    /** The field that names the window. */
    const val FIELD_DAYS = "days"

    /**
     * Whether the request asks for the **older history**.
     *
     * The older history is the dated, checkpointed walk back through the
     * months: the phone's own `stride_garmin_backfill`, one bounded
     * window at a time, with the cursor kept between them.
     *
     * A body that names nothing asks for the recent pull, and a body that
     * is not JSON at all asks for nothing in particular.
     */
    fun asksForHistory(body: String?): Boolean {
        val parsed = parsed(body) ?: return false
        return parsed.optBoolean(FIELD_BACKFILL, false)
    }

    /**
     * Whether the request asks for the pull to run in the background.
     *
     * **The background is the default.** A body that names nothing is the
     * phone's own scheduled job, and only a body that says otherwise is
     * asked for on the app's own thread.
     */
    fun asksForBackground(body: String?): Boolean {
        val parsed = parsed(body) ?: return true
        return parsed.optBoolean(FIELD_BACKGROUND, true)
    }

    /**
     * The window the request asks for, or the phone's own.
     *
     * **The window is bounded.** A request for a thousand days is a
     * request for the largest window the phone asks for, and a request for
     * nothing is the phone's own setting.
     */
    fun daysOf(body: String?, fallback: Int): Int {
        val parsed = parsed(body) ?: return fallback
        val asked = parsed.optInt(FIELD_DAYS, 0)
        if (asked <= 0) return fallback
        return asked.coerceIn(PullWindow.MIN_DAYS.toInt(), PullWindow.MAX_DAYS.toInt())
    }

    /**
     * The body, as an object.
     *
     * A body that is not a JSON object names nothing: a bare string or an
     * array is a body and not a place to keep a field.
     */
    private fun parsed(body: String?): JSONObject? {
        val text = body?.trim()?.takeIf { it.isNotEmpty() } ?: return null
        return runCatching { JSONObject(text) }.getOrNull()
    }
}
