package app.stride.running.security

import android.content.Context
import android.security.keystore.KeyPermanentlyInvalidatedException
import java.io.File
import java.io.IOException

/**
 * One encrypted file in `noBackupFilesDir`.
 *
 * The file holds one secret — the Garmin credential, or the database key —
 * wrapped by a Keystore key. The bytes are:
 *
 * ```text
 * nonce length (one byte) | nonce | ciphertext + GCM tag
 * ```
 *
 * No magic and no version in the bytes: the version is in the file's
 * *name* (`garmin-credential.v1`, `database-key.v1`), which is readable,
 * and the framing itself is [EnvelopeFraming]'s — the same object the
 * writer and the reader use, so the two cannot drift apart.
 *
 * The file is written atomically (a temporary file, then a rename) so that a
 * crash halfway through cannot leave a half-written envelope that decrypts to
 * nothing, and it lives in `noBackupFilesDir`, so no backup and no device
 * transfer carries it.
 */
class EnvelopeFile(
    private val context: Context,
    private val name: String,
    private val alias: String,
) {

    /** The file itself. */
    private fun file(): File = File(context.noBackupFilesDir, name)

    /** Whether the envelope is there. */
    fun exists(): Boolean = file().exists()

    /**
     * Write a secret.
     *
     * The plaintext is not kept: it is encrypted and dropped.
     */
    fun write(plaintext: ByteArray) {
        val sealed = KeystoreKeys.encrypt(alias, plaintext)
        val target = file()
        val temporary = File(target.parentFile, "$name.tmp")
        try {
            temporary.writeBytes(sealed)
            temporary.setReadable(false, false)
            temporary.setWritable(false, false)
            temporary.setReadable(true, true)
            temporary.setWritable(true, true)
            if (!temporary.renameTo(target)) {
                // Some filesystems refuse the rename; write in place instead.
                target.writeBytes(sealed)
                temporary.delete()
            }
        } catch (error: IOException) {
            temporary.delete()
            throw error
        }
    }

    /**
     * Read the secret, or `null` when there is none.
     *
     * **A file that cannot be read is never deleted.** There are two very
     * different reasons, and they are not the same thing:
     *
     * * the wrapping key cannot be used *right now* — the phone is locked —
     *   and the file is perfectly good;
     * * the wrapping key was *invalidated* — the device was wiped, or the
     *   key was invalidated with the lock state — and the file is unreadable
     *   for good.
     *
     * The first is temporary and the second is a loss, and the app has to
     * tell them apart: deleting the file on the first would throw away the
     * credential, and writing a new one over it would orphan the database
     * the key opens.
     */
    fun read(): ByteArray? {
        val target = file()
        if (!target.exists()) return null
        return try {
            KeystoreKeys.decrypt(alias, target.readBytes())
        } catch (error: KeyPermanentlyInvalidatedException) {
            throw CredentialUnreadable(
                "the key that protected this file is gone; it cannot be read again",
                error,
            )
        } catch (error: Exception) {
            throw CredentialUnavailable(
                "the key that protects this file cannot be used right now",
                error,
            )
        }
    }

    /** Read the secret as a string. */
    fun readString(): String? = read()?.toString(Charsets.UTF_8)

    /** Write the secret as a string. */
    fun writeString(plaintext: String) {
        write(plaintext.toByteArray(Charsets.UTF_8))
    }

    /**
     * Forget the secret.
     *
     * The answer says whether the envelope is gone. A caller that removes
     * a credential must not claim that it was removed when the file is
     * still there, so the filesystem's answer is passed on rather than
     * hidden.
     */
    fun clear(): Boolean {
        val target = file()
        val targetRemoved = !target.exists() || target.delete()
        val temporary = File(context.noBackupFilesDir, "$name.tmp")
        val temporaryRemoved = !temporary.exists() || temporary.delete()
        return targetRemoved && temporaryRemoved
    }
}
