package app.stride.running.security

/**
 * The framing of an envelope file.
 *
 * **The file is:**
 *
 * ```text
 * nonce length (one byte) | nonce | ciphertext + GCM tag
 * ```
 *
 * That is all there is: no magic, no version. The version is in the file's
 * *name* — `garmin-credential.v1`, `database-key.v1` — because a name is
 * readable and a byte is not.
 *
 * **The nonce is in front of the ciphertext** because the caller stores the
 * result and needs it back: GCM needs the same nonce to decrypt what it
 * encrypted with, and the nonce is not a secret, so it is written in the
 * clear beside the ciphertext it belongs to. Its length is written first,
 * as one byte, so the reader knows where the ciphertext starts.
 *
 * The framing is here, on its own, with no Android and no Keystore in it,
 * because **it is the part that can drift**: the writer and the reader are
 * two functions in two files, and a change to one that does not change the
 * other is a file that cannot be read any more. Here they are one object,
 * and one test holds them together.
 */
object EnvelopeFraming {

    /** The shortest nonce GCM uses. */
    const val NONCE_MIN = 12

    /** The longest nonce this framing accepts. */
    const val NONCE_MAX = 16

    /**
     * The bytes of an envelope.
     *
     * @throws IllegalArgumentException when the nonce is not a length this
     *   framing can write.
     */
    fun pack(nonce: ByteArray, ciphertext: ByteArray): ByteArray {
        require(nonce.size in NONCE_MIN..NONCE_MAX) {
            "the nonce is ${nonce.size} bytes; it must be $NONCE_MIN to $NONCE_MAX"
        }
        val out = ByteArray(1 + nonce.size + ciphertext.size)
        out[0] = nonce.size.toByte()
        nonce.copyInto(out, 1)
        ciphertext.copyInto(out, 1 + nonce.size)
        return out
    }

    /**
     * The nonce and the ciphertext of an envelope.
     *
     * @throws IllegalArgumentException when the bytes are not an envelope:
     *   an empty file, a nonce length that is not one, or a file that was
     *   cut short.
     */
    fun unpack(bytes: ByteArray): Envelope {
        require(bytes.size > 1) { "the envelope is empty" }
        val nonceLength = bytes[0].toInt()
        require(nonceLength in NONCE_MIN..NONCE_MAX) {
            "the envelope's nonce length is $nonceLength; it must be $NONCE_MIN to $NONCE_MAX"
        }
        require(bytes.size > 1 + nonceLength) { "the envelope is truncated" }
        return Envelope(
            nonce = bytes.copyOfRange(1, 1 + nonceLength),
            ciphertext = bytes.copyOfRange(1 + nonceLength, bytes.size),
        )
    }
}

/**
 * An envelope, taken apart.
 *
 * @property nonce the nonce the ciphertext was made with.
 * @property ciphertext the ciphertext, with its GCM tag.
 */
data class Envelope(
    val nonce: ByteArray,
    val ciphertext: ByteArray,
) {
    /** Two envelopes are the same when their bytes are. */
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Envelope) return false
        return nonce.contentEquals(other.nonce) && ciphertext.contentEquals(other.ciphertext)
    }

    /** The hash code of the bytes. */
    override fun hashCode(): Int = 31 * nonce.contentHashCode() + ciphertext.contentHashCode()
}
