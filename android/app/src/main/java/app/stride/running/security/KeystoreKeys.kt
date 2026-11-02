package app.stride.running.security

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * The Keystore keys.
 *
 * Two keys, two aliases, two jobs:
 *
 * * [ALIAS_GARMIN] wraps the Garmin Personal API key;
 * * [ALIAS_DATABASE] wraps the SQLCipher key.
 *
 * They are separate on purpose: replacing the Garmin key must not orphan the
 * history, and the database key is never derived from the credential.
 *
 * The keys are **non-exportable**. The app asks the Keystore to encrypt and
 * decrypt; the key material never enters the process, never enters the
 * WebView, and never enters a log.
 *
 * The keys are marked `setUnlockedDeviceRequired`, so a locked phone cannot
 * use them. That is the point: a worker that runs while the phone is locked
 * finds the key unusable and defers, rather than decrypting the athlete's
 * history in the background.
 *
 * **That flag is why the app's floor is Android 15.** On Android 12 to 14
 * the platform does not enforce it the way the documentation says it does,
 * and a promise the platform breaks is not a promise.
 */
object KeystoreKeys {

    /** The alias of the key that wraps the Garmin credential. */
    const val ALIAS_GARMIN = "stride.garmin.personal-key.v1"

    /** The alias of the key that wraps the SQLCipher database key. */
    const val ALIAS_DATABASE = "stride.database.key.v1"

    private const val ANDROID_KEYSTORE = "AndroidKeyStore"
    private const val TRANSFORMATION = "AES/GCM/NoPadding"
    private const val TAG_BITS = 128

    /**
     * The wrapping key for an alias, created if it is not there.
     *
     * AES-256-GCM, with a fresh nonce for every encryption and
     * `setRandomizedEncryptionRequired`, so the same plaintext never produces
     * the same ciphertext twice.
     */
    fun key(alias: String): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        (keyStore.getKey(alias, null) as? SecretKey)?.let { return it }

        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        val builder = KeyGenParameterSpec.Builder(
            alias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setRandomizedEncryptionRequired(true)

        // The key is unusable while the phone is locked. The background
        // worker checks the lock state and defers rather than failing.
        builder.setUnlockedDeviceRequired(true)

        generator.init(builder.build())
        return generator.generateKey()
    }

    /**
     * Encrypt with a wrapping key.
     *
     * The nonce is written in front of the ciphertext, because the caller
     * stores the result and needs it back — and the framing of the file is
     * [EnvelopeFraming]'s, so the reader and the writer cannot drift
     * apart.
     */
    fun encrypt(alias: String, plaintext: ByteArray): ByteArray {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key(alias))
        return EnvelopeFraming.pack(cipher.iv, cipher.doFinal(plaintext))
    }

    /**
     * Decrypt with a wrapping key.
     *
     * @throws javax.crypto.AEADBadTagException when the bytes were not
     *   written by this key — a wrong key, a truncated file, a device restore.
     * @throws IllegalArgumentException when the bytes are not an envelope at
     *   all — an empty file, or one that was cut short.
     */
    fun decrypt(alias: String, bytes: ByteArray): ByteArray {
        val envelope = EnvelopeFraming.unpack(bytes)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(
            Cipher.DECRYPT_MODE,
            key(alias),
            GCMParameterSpec(TAG_BITS, envelope.nonce),
        )
        return cipher.doFinal(envelope.ciphertext)
    }

    /** Whether the alias has a key. */
    fun exists(alias: String): Boolean {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        return keyStore.containsAlias(alias)
    }

    /**
     * Forget a wrapping key. The envelope it protected becomes unreadable.
     *
     * The answer says whether the alias is gone. An alias that was not
     * there counts as removed: there is nothing left to use.
     */
    fun delete(alias: String): Boolean {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        if (!keyStore.containsAlias(alias)) return true
        keyStore.deleteEntry(alias)
        return !keyStore.containsAlias(alias)
    }
}
