package app.stride.running.security

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.GCMParameterSpec

/**
 * The envelope's format, tested against itself.
 *
 * The writer and the reader of the envelope file are two functions in two
 * files, and a change to one that does not change the other is a file that
 * cannot be read any more. So the framing is one object, and this test
 * holds it together:
 *
 * * **what is written is what is read** — the bytes `pack` writes are the
 *   bytes `unpack` reads, in the same order, with the same lengths;
 * * **the shape is the shape that is documented** — the length byte, then
 *   the nonce, then the ciphertext, and nothing else;
 * * **a real GCM envelope survives the round trip** — the framing is not
 *   merely self-consistent, it carries what the cipher wrote.
 *
 * The Keystore itself cannot be reached from a unit test — it is the
 * platform's — so the cipher here is the JVM's own AES-GCM, with the same
 * transformation the Keystore uses (`AES/GCM/NoPadding`, a 128-bit tag).
 * What is under test is the framing, not the key store.
 */
class EnvelopeFramingTest {

    /** A nonce of twelve bytes — the length GCM uses. */
    private val nonce = ByteArray(12) { it.toByte() }

    /** Some ciphertext. */
    private val ciphertext = ByteArray(24) { (it * 7).toByte() }

    @Test
    fun `the bytes are the length, the nonce and the ciphertext`() {
        val bytes = EnvelopeFraming.pack(nonce, ciphertext)

        assertEquals(1 + nonce.size + ciphertext.size, bytes.size)
        assertEquals("the nonce length is not the first byte", nonce.size.toByte(), bytes[0])
        assertArrayEquals("the nonce is not in front", nonce, bytes.copyOfRange(1, 1 + nonce.size))
        assertArrayEquals(
            "the ciphertext is not behind the nonce",
            ciphertext,
            bytes.copyOfRange(1 + nonce.size, bytes.size),
        )
    }

    @Test
    fun `what is written is what is read`() {
        val envelope = EnvelopeFraming.unpack(EnvelopeFraming.pack(nonce, ciphertext))
        assertArrayEquals(nonce, envelope.nonce)
        assertArrayEquals(ciphertext, envelope.ciphertext)
    }

    @Test
    fun `an empty file is not an envelope`() {
        assertThrows(IllegalArgumentException::class.java) {
            EnvelopeFraming.unpack(ByteArray(0))
        }
    }

    @Test
    fun `a file cut short in the nonce is not an envelope`() {
        val bytes = EnvelopeFraming.pack(nonce, ciphertext)
        assertThrows(IllegalArgumentException::class.java) {
            EnvelopeFraming.unpack(bytes.copyOfRange(0, 1 + nonce.size - 1))
        }
    }

    @Test
    fun `a file cut short in the ciphertext is left to the cipher`() {
        // The framing cannot tell a short ciphertext from a whole one: the
        // GCM tag is what fails, and that is the cipher's job.
        val bytes = EnvelopeFraming.pack(nonce, ciphertext)
        val envelope = EnvelopeFraming.unpack(bytes.copyOfRange(0, bytes.size - 1))
        assertEquals(ciphertext.size - 1, envelope.ciphertext.size)
        assertArrayEquals(nonce, envelope.nonce)
    }

    @Test
    fun `a nonce length that is not one is not an envelope`() {
        val bytes = ByteArray(40)
        bytes[0] = 7
        assertThrows(IllegalArgumentException::class.java) {
            EnvelopeFraming.unpack(bytes)
        }
    }

    @Test
    fun `a nonce that cannot be written is refused`() {
        assertThrows(IllegalArgumentException::class.java) {
            EnvelopeFraming.pack(ByteArray(8), ciphertext)
        }
    }

    /** The framing carries a real GCM envelope, in and out. */
    @Test
    fun `a real AES-GCM envelope survives the round trip`() {
        val key = KeyGenerator.getInstance("AES").apply { init(256) }.generateKey()
        val plaintext = "the Garmin key, or the database key".toByteArray()

        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key)
        val sealed = EnvelopeFraming.pack(cipher.iv, cipher.doFinal(plaintext))

        val envelope = EnvelopeFraming.unpack(sealed)
        val opened = Cipher.getInstance("AES/GCM/NoPadding")
        opened.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, envelope.nonce))
        val read = opened.doFinal(envelope.ciphertext)

        assertArrayEquals(plaintext, read)
    }

    @Test
    fun `the version is in the file name and not in the bytes`() {
        assertTrue(Secrets.CREDENTIAL_FILE.endsWith(".v1"))
        assertTrue(Secrets.DATABASE_FILE.endsWith(".v1"))

        // The bytes carry no magic and no version: they are the nonce's
        // length, the nonce, and the ciphertext — and nothing else.
        val bytes = EnvelopeFraming.pack(nonce, ciphertext)
        assertEquals(1 + nonce.size + ciphertext.size, bytes.size)
    }
}
