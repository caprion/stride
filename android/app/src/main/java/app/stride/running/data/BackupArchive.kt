package app.stride.running.data

import org.json.JSONObject
import java.io.File
import java.io.InputStream
import java.io.IOException
import java.io.OutputStream
import java.nio.file.Files
import java.security.SecureRandom
import java.util.Base64
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream
import java.util.zip.ZipOutputStream
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec

object BackupArchive {
    private const val ITERATIONS = 210_000
    private const val LIMIT = 512L * 1024 * 1024
    private val entries = setOf("backup.json", "manifest.json", "stride.db")
    private val activeStages = mutableSetOf<String>()

    fun newSalt(): ByteArray = ByteArray(32).also { SecureRandom().nextBytes(it) }

    @Synchronized
    fun createStage(root: File): File =
        Files.createTempDirectory(root.toPath(), "transfer-").toFile().also {
            activeStages.add(it.canonicalPath)
        }

    @Synchronized
    fun removeStage(stage: File): Boolean {
        activeStages.remove(stage.canonicalPath)
        return stage.deleteRecursively()
    }

    @Synchronized
    fun cleanupExpired(root: File, pendingDirectory: String?, now: Long = System.currentTimeMillis()) {
        val canonicalRoot = root.canonicalFile
        val stages = root.listFiles() ?: throw IOException("Backup staging could not be inspected")
        for (stage in stages) {
            if (!stage.name.startsWith("transfer-") || stage.name == pendingDirectory ||
                !stage.isDirectory || stage.lastModified() >= now - 86_400_000L ||
                Files.isSymbolicLink(stage.toPath()) || stage.canonicalFile.parentFile != canonicalRoot ||
                stage.canonicalPath in activeStages
            ) continue
            if (!stage.deleteRecursively()) throw IOException("Expired encrypted staging could not be removed")
        }
    }

    @Synchronized
    fun resume(root: File, directory: String): File? {
        if (!directory.startsWith("transfer-") || File(directory).name != directory) return null
        val stage = File(root, directory).canonicalFile
        val archive = File(stage, "backup.zip").canonicalFile
        return archive.takeIf {
            stage.parentFile == root.canonicalFile && it.parentFile == stage && it.isFile
        }?.also {
            activeStages.add(stage.path)
        }
    }

    fun key(passphrase: CharArray, salt: ByteArray): String {
        require(salt.size == 32) { "Invalid backup salt." }
        val spec = PBEKeySpec(passphrase, salt, ITERATIONS, 256)
        try {
            val bytes = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(spec).encoded
            return try {
                bytes.joinToString("") { "%02x".format(it.toInt() and 255) }
            } finally {
                bytes.fill(0)
            }
        } finally {
            spec.clearPassword()
            passphrase.fill('\u0000')
        }
    }

    fun write(directory: File, salt: ByteArray, output: OutputStream) {
        require(salt.size == 32) { "Invalid backup salt." }
        val metadata = JSONObject()
            .put("format", "stride-phone-backup")
            .put("version", 1)
            .put("kdf", "PBKDF2WithHmacSHA256")
            .put("iterations", ITERATIONS)
            .put("salt", Base64.getEncoder().encodeToString(salt))
            .toString().toByteArray(Charsets.UTF_8)
        ZipOutputStream(output).use { zip ->
            zip.setLevel(0)
            zip.putNextEntry(ZipEntry("backup.json"))
            zip.write(metadata)
            zip.closeEntry()
            for (name in listOf("manifest.json", "stride.db")) {
                zip.putNextEntry(ZipEntry(name))
                File(directory, name).inputStream().use { copyBounded(it, zip, if (name == "stride.db") LIMIT else 1024 * 1024L) }
                zip.closeEntry()
            }
        }
    }

    fun read(input: InputStream, directory: File): ByteArray {
        require(directory.isDirectory && directory.list()?.isEmpty() == true) { "Backup staging is not empty." }
        val seen = mutableSetOf<String>()
        ZipInputStream(input).use { zip ->
            while (true) {
                val entry = zip.nextEntry ?: break
                require(!entry.isDirectory && entry.name in entries && seen.add(entry.name)) { "Unexpected backup contents." }
                File(directory, entry.name).outputStream().use { output ->
                    copyBounded(zip, output, if (entry.name == "stride.db") LIMIT else 1024 * 1024L)
                }
                zip.closeEntry()
            }
        }
        require(seen == entries) { "Incomplete backup." }
        val metadata = JSONObject(File(directory, "backup.json").readText(Charsets.UTF_8))
        require(
            metadata.getString("format") == "stride-phone-backup" &&
                metadata.getInt("version") == 1 &&
                metadata.getString("kdf") == "PBKDF2WithHmacSHA256" &&
                metadata.getInt("iterations") == ITERATIONS
        ) { "Unsupported backup format." }
        return Base64.getDecoder().decode(metadata.getString("salt")).also {
            require(it.size == 32) { "Invalid backup salt." }
        }
    }

    private fun copyBounded(input: InputStream, output: OutputStream, limit: Long) {
        val buffer = ByteArray(64 * 1024)
        var total = 0L
        while (true) {
            val count = input.read(buffer)
            if (count < 0) break
            total += count
            require(total <= limit) { "The backup exceeds the supported size." }
            output.write(buffer, 0, count)
        }
    }
}
