package app.stride.running.security

/**
 * The safe identifier for a Garmin key.
 *
 * Settings needs to say which key is connected, but the key itself must
 * never leave the native surface that owns it. This object is the one
 * place that reduces a key to an identifier: the last four characters and
 * nothing else.
 *
 * The result is deliberately not the key, a prefix of the key, a hash of
 * the key or any reversible encoding of it. Four characters identify the
 * key for the athlete; they do not authenticate anything.
 */
object KeyIdentifier {

    /** How many characters of the key an identifier carries. */
    const val VISIBLE_CHARACTERS = 4

    /**
     * The identifier for a key, or `null` when there is no key.
     *
     * The caller drops the full key as soon as this returns. This object
     * does not keep it.
     */
    fun lastFour(key: String?): String? {
        val value = key?.trim().orEmpty()
        if (value.isEmpty()) return null
        return value.takeLast(VISIBLE_CHARACTERS)
    }
}
