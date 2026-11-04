package app.stride.running.data

import org.junit.Assert.*
import org.junit.Test
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.nio.file.Files
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

class BackupArchiveTest {
    @Test
    fun `expired orphan cleanup preserves active pending fresh and unrelated data`() {
        val root = Files.createTempDirectory("stride-cleanup-test").toFile()
        val outside = Files.createTempDirectory("stride-outside-test").toFile()
        val now = System.currentTimeMillis()
        val active = BackupArchive.createStage(root)
        try {
            val expired = root.resolve("transfer-orphan").apply { mkdir(); setLastModified(1) }
            val pending = root.resolve("transfer-pending").apply { mkdir(); setLastModified(1) }
            val fresh = root.resolve("transfer-fresh").apply { mkdir(); setLastModified(now) }
            val unrelated = root.resolve("database").apply { mkdir(); setLastModified(1) }
            outside.resolve("keep").writeText("synthetic")
            Files.createSymbolicLink(root.resolve("transfer-link").toPath(), outside.toPath())
            active.setLastModified(1)
            BackupArchive.cleanupExpired(root, pending.name, now)
            assertFalse(expired.exists())
            for (file in listOf(active, pending, fresh, unrelated, outside.resolve("keep"))) assertTrue(file.exists())
        } finally {
            BackupArchive.removeStage(active)
            Files.deleteIfExists(root.resolve("transfer-link").toPath())
            root.deleteRecursively()
            outside.deleteRecursively()
        }
    }

    @Test
    fun `only an encrypted pending archive inside owned staging can resume`() {
        val root = Files.createTempDirectory("stride-pending-test").toFile()
        try {
            val stage = root.resolve("transfer-pending").apply { mkdir() }
            val archive = stage.resolve("backup.zip").apply { writeBytes(byteArrayOf(1)) }
            assertEquals(archive.canonicalFile, BackupArchive.resume(root, stage.name))
            assertNull(BackupArchive.resume(root, "../transfer-pending"))
            assertNull(BackupArchive.resume(root, "other"))
            assertNull(BackupArchive.resume(root, "transfer-missing"))
        } finally {
            root.deleteRecursively()
        }
    }
    @Test
    fun `archive round trip retains encrypted bytes and derives a portable key`() {
        val source = Files.createTempDirectory("stride-backup-test").toFile()
        val target = Files.createTempDirectory("stride-restore-test").toFile()
        try {
            source.resolve("manifest.json").writeText("""{"synthetic":true}""")
            val encrypted = ByteArray(1024) { (it % 251).toByte() }
            source.resolve("stride.db").writeBytes(encrypted)
            val salt = BackupArchive.newSalt()
            val bytes = ByteArrayOutputStream()
            BackupArchive.write(source, salt, bytes)
            val restoredSalt = BackupArchive.read(ByteArrayInputStream(bytes.toByteArray()), target)
            assertArrayEquals(salt, restoredSalt)
            assertArrayEquals(encrypted, target.resolve("stride.db").readBytes())
            val passphrase = "synthetic passphrase, not a credential".toCharArray()
            val first = BackupArchive.key(passphrase, salt)
            assertTrue(passphrase.all { it == '\u0000' })
            assertEquals(first, BackupArchive.key("synthetic passphrase, not a credential".toCharArray(), restoredSalt))
            assertNotEquals(first, BackupArchive.key("another synthetic passphrase".toCharArray(), restoredSalt))
        } finally {
            source.deleteRecursively()
            target.deleteRecursively()
        }
    }

    @Test
    fun `path traversal unexpected entries and incomplete archives are refused`() {
        for (name in listOf("../outside.db", "/stride.db", "other.json", "stride.db")) {
            val target = Files.createTempDirectory("stride-invalid-backup-test").toFile()
            try {
                val bytes = ByteArrayOutputStream()
                ZipOutputStream(bytes).use { zip ->
                    zip.putNextEntry(ZipEntry(name))
                    zip.write(byteArrayOf(1))
                    zip.closeEntry()
                }
                assertThrows(IllegalArgumentException::class.java) {
                    BackupArchive.read(ByteArrayInputStream(bytes.toByteArray()), target)
                }
            } finally {
                target.deleteRecursively()
            }
        }
    }
}
