package app.stride.running.security

import android.content.Context
import java.io.IOException
import java.security.SecureRandom

/**
 * The two secrets the phone holds.
 *
 * The Garmin credential and the database key live in two envelopes, wrapped by
 * two Keystore keys, in two files that no backup carries.
 *
 * **Neither of them is ever returned to the WebView, and neither is ever
 * logged.** The Garmin key is read from here by the native setup screen and
 * by the background worker, and by nothing else.
 */
object Secrets {

    /**
     * The file the Garmin credential is in.
     *
     * **The version is in the name.** The bytes of an envelope carry no
     * magic and no version — they are the nonce's length, the nonce and
     * the ciphertext — because a name is readable and a byte is not.
     */
    const val CREDENTIAL_FILE = "garmin-credential.v1"

    /** The file the SQLCipher database key is in. */
    const val DATABASE_FILE = "database-key.v1"

    /** The Garmin Personal API key, encrypted. */
    fun credential(context: Context): EnvelopeFile =
        EnvelopeFile(context, CREDENTIAL_FILE, KeystoreKeys.ALIAS_GARMIN)

    /** The SQLCipher database key, encrypted. */
    fun databaseKey(context: Context): EnvelopeFile =
        EnvelopeFile(context, DATABASE_FILE, KeystoreKeys.ALIAS_DATABASE)

    /**
     * The 32-byte SQLCipher key, hex-encoded.
     *
     * It is generated once, on the first run, and then kept in the envelope:
     * the same key has to open the same database tomorrow. It is never derived
     * from the Garmin credential, from the device id, or from a constant.
     *
     * **It is never replaced.** A key that cannot be read right now — the
     * phone is locked — is a temporary failure, and writing a new key over
     * the envelope would leave the database unopenable and the athlete's
     * history lost with it. So an envelope that exists and cannot be read is
     * reported, not overwritten.
     */
    fun databaseKeyHex(context: Context): String {
        val envelope = databaseKey(context)
        // `read` throws when the envelope exists and cannot be decrypted.
        val existing = envelope.readString()
        if (existing != null && existing.length == 64) return existing

        if (envelope.exists()) {
            throw CredentialUnreadable(
                "the database key is there but cannot be read; it has not been replaced",
            )
        }

        val key = ByteArray(32)
        SecureRandom().nextBytes(key)
        val hex = key.joinToString("") { byte -> "%02x".format(byte) }
        envelope.writeString(hex)
        return hex
    }

    /**
     * The Garmin Personal API key, or `null`.
     *
     * The caller gets the plaintext and is expected to drop it as soon as the
     * exchange is done. It is never logged and never given to the WebView.
     */
    fun garminKey(context: Context): String? = credential(context).readString()

    /** Save the Garmin Personal API key. */
    fun saveGarminKey(context: Context, key: String) {
        credential(context).writeString(key.trim())
    }

    /**
     * Forget the Garmin Personal API key and its wrapping key.
     *
     * The envelope goes first. If the filesystem refuses to remove it, the
     * call fails rather than claiming that a credential is gone while its
     * encrypted envelope is still on disk. The wrapping key is then
     * removed as well; the report says whether both are gone.
     */
    fun forgetGarminKey(context: Context): Removal {
        val envelopeRemoved = credential(context).clear()
        if (!envelopeRemoved) {
            throw IOException("the encrypted Garmin credential could not be removed")
        }
        val wrappingKeyRemoved = KeystoreKeys.delete(KeystoreKeys.ALIAS_GARMIN)
        if (!wrappingKeyRemoved) {
            throw IOException("the Garmin credential's Keystore protection could not be removed")
        }
        return Removal(
            envelopeRemoved = envelopeRemoved,
            wrappingKeyRemoved = wrappingKeyRemoved,
        )
    }

    /** What was removed from the phone's credential store. */
    data class Removal(
        /** The encrypted credential file is gone. */
        val envelopeRemoved: Boolean,

        /** The non-exportable Keystore alias is gone. */
        val wrappingKeyRemoved: Boolean,
    )
}
