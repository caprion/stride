package app.stride.running.bridge

/**
 * The last line of defence for the Garmin key.
 *
 * The key is never *given* to the page: the routes that would carry it are
 * refused before they reach the core, and the core never answers with it.
 * This is the belt to that pair of braces: if an answer ever did carry the
 * key, it is taken out of it before the page sees a single character.
 *
 * It is a plain object with no Android in it, so it can be tested without a
 * phone — and a guard that is only tested on a phone is a guard that is not
 * tested.
 */
object KeyGuard {

    /** What replaces the key in an answer. */
    const val REDACTED = "[the Garmin key is not returned]"

    /**
     * Whether an answer carries the key.
     *
     * The key is compared as it is and as it would appear in a JSON string,
     * so an escaped form counts too.
     */
    fun carriesKey(payload: String, key: String?): Boolean {
        if (key.isNullOrBlank()) return false
        if (payload.contains(key)) return true
        val escaped = escapeForJson(key)
        return escaped != key && payload.contains(escaped)
    }

    /**
     * Take the key out of an answer.
     *
     * An answer that carries the key is replaced by one that says the key is
     * not returned. The key is not merely hidden in place: the answer is
     * refused outright, because an answer that had the key in it has already
     * failed somewhere else, and half of it is not better than none of it.
     */
    fun redact(payload: String, key: String?): String {
        if (!carriesKey(payload, key)) return payload
        return """{"ok":false,"status":500,"error":{"detail":"$REDACTED"}}"""
    }

    /**
     * The key as it would appear inside a JSON string.
     */
    private fun escapeForJson(value: String): String = buildString {
        for (character in value) {
            when (character) {
                '"' -> append("\\\"")
                '\\' -> append("\\\\")
                '\n' -> append("\\n")
                '\r' -> append("\\r")
                '\t' -> append("\\t")
                else -> append(character)
            }
        }
    }
}
