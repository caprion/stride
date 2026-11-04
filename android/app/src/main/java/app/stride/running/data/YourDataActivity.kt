package app.stride.running.data

import android.net.Uri
import android.os.Bundle
import android.text.InputType
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import app.stride.running.R
import app.stride.running.core.StrideCore
import app.stride.running.security.AppGate
import app.stride.running.security.DeviceCredential
import org.json.JSONObject
import uniffi.stride.strideBackup
import uniffi.stride.strideHistoryExport
import java.io.File
import java.io.IOException
import java.time.LocalDate
import java.util.concurrent.Executors

class YourDataActivity : AppCompatActivity() {
    private val worker = Executors.newSingleThreadExecutor()
    private lateinit var gate: AppGate
    private lateinit var status: TextView
    private lateinit var passphrase: EditText
    private lateinit var confirmation: EditText
    private lateinit var from: EditText
    private lateinit var to: EditText
    private val controls = mutableListOf<View>()
    private var busy = false
    private var pendingArchive: File? = null
    private var pendingPassphrase: CharArray? = null
    private var pendingAction: (() -> Unit)? = null
    private var pendingText: Pair<String, String>? = null
    private var awaitingPicker = false
    private var archiveSavedForRecreation = false
    private val saveCsv = registerForActivityResult(ActivityResultContracts.CreateDocument("text/csv")) { saveText(it) }
    private val saveJson = registerForActivityResult(ActivityResultContracts.CreateDocument("application/json")) { saveText(it) }

    private val saveDocument = registerForActivityResult(ActivityResultContracts.CreateDocument("application/zip")) { uri ->
        val archive = pendingArchive
        awaitingPicker = false
        if (archive != null) {
            if (uri == null) {
                pendingArchive = null
                clean(archive.parentFile!!)
                message(R.string.data_cancelled)
            } else whenUnlocked {
                pendingArchive = null
                task {
                    try {
                        val output = contentResolver.openOutputStream(uri, "wt") ?: error("No output")
                        output.use { stream -> archive.inputStream().use { it.copyTo(stream) } }
                        getString(R.string.data_saved)
                    } finally {
                        clean(archive.parentFile!!)
                    }
                }
            }
        } else message(R.string.data_interrupted)
    }

    private val openDocument = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        val phrase = pendingPassphrase
        awaitingPicker = false
        if (phrase != null) {
            if (uri == null) {
                pendingPassphrase = null
                phrase.fill('\u0000')
                message(R.string.data_cancelled)
            } else whenUnlocked {
                pendingPassphrase = null
                restore(uri, phrase)
            }
        } else message(R.string.data_interrupted)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
        if (!DeviceCredential.hasCredential(this)) {
            finish()
            return
        }
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            val padding = (20 * resources.displayMetrics.density).toInt()
            setPadding(padding, padding, padding, padding)
        }
        fun text(id: Int) = TextView(this).also {
            it.setText(id)
            it.textSize = 16f
            it.setPadding(0, 12, 0, 12)
            content.addView(it)
        }
        text(R.string.data_title).textSize = 28f
        text(R.string.data_explanation)
        passphrase = passwordField(R.string.data_passphrase, content)
        confirmation = passwordField(R.string.data_confirm, content)
        button(content, R.string.data_backup) { createBackup() }
        text(R.string.data_restore_explanation)
        button(content, R.string.data_restore) {
            val phrase = password()
            if (phrase.size < 12) {
                phrase.fill('\u0000')
                message(R.string.data_passphrase_short)
                return@button
            }
            clearFields()
            pendingPassphrase = phrase
            setBusy(true)
            try {
                awaitingPicker = true
                openDocument.launch(arrayOf("application/zip", "application/octet-stream"))
            } catch (_: android.content.ActivityNotFoundException) {
                awaitingPicker = false
                pendingPassphrase?.fill('\u0000')
                pendingPassphrase = null
                message(R.string.data_no_picker)
            }
        }
        text(R.string.data_export_explanation)
        from = dateField(R.string.data_from, LocalDate.now().minusYears(1).toString(), content)
        to = dateField(R.string.data_to, LocalDate.now().toString(), content)
        button(content, R.string.data_csv) { exportHistory("csv") }
        button(content, R.string.data_json) { exportHistory("json") }
        status = text(R.string.data_ready).apply { accessibilityLiveRegion = View.ACCESSIBILITY_LIVE_REGION_POLITE }
        setContentView(ScrollView(this).apply { addView(content) })
        gate = AppGate(this)
        gate.onUnlocked = {
            pendingAction?.also { pendingAction = null }?.invoke()
            updateControls()
        }
        val savedArchive = savedInstanceState?.getString("pending_archive")
        if (savedArchive != null) {
            pendingArchive = try {
                BackupArchive.resume(noBackupFilesDir, savedArchive)
            } catch (_: IOException) {
                Log.w("StrideData", "The pending encrypted archive could not be reopened")
                null
            }
            awaitingPicker = pendingArchive != null
            busy = awaitingPicker
        }
        if (savedInstanceState?.getBoolean("pending_operation") == true && pendingArchive == null) {
            status.setText(R.string.data_interrupted)
        }
        worker.execute {
            try {
                BackupArchive.cleanupExpired(noBackupFilesDir, savedArchive)
            } catch (_: IOException) {
                Log.w("StrideData", "Expired encrypted staging could not be cleaned")
                runOnUiThread { if (!isDestroyed) status.setText(R.string.data_failed) }
            }
        }
        gate.lock()
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putBoolean("pending_operation", busy)
        outState.remove("pending_archive")
        if (awaitingPicker && pendingArchive != null) {
            outState.putString("pending_archive", pendingArchive!!.parentFile!!.name)
            archiveSavedForRecreation = true
        }
        super.onSaveInstanceState(outState)
    }

    override fun onStart() {
        super.onStart()
        archiveSavedForRecreation = false
        if (::gate.isInitialized) gate.onForeground()
    }

    override fun onStop() {
        if (::gate.isInitialized) gate.onBackground()
        super.onStop()
    }

    override fun onDestroy() {
        pendingPassphrase?.fill('\u0000')
        clearFields()
        if (!archiveSavedForRecreation || isFinishing) pendingArchive?.parentFile?.let { clean(it) }
        pendingText = null
        pendingAction = null
        worker.shutdown()
        super.onDestroy()
    }

    private fun createBackup() {
        val phrase = password()
        val repeated = CharArray(confirmation.length()) { confirmation.text[it] }
        val matches = phrase.contentEquals(repeated)
        repeated.fill('\u0000')
        if (phrase.size < 12 || !matches) {
            phrase.fill('\u0000')
            message(if (!matches) R.string.data_mismatch else R.string.data_passphrase_short)
            return
        }
        clearFields()
        task(onSuccess = {
            try {
                awaitingPicker = true
                saveDocument.launch("stride-backup-${LocalDate.now()}.stride.zip")
            } catch (_: android.content.ActivityNotFoundException) {
                awaitingPicker = false
                pendingArchive?.parentFile?.let { clean(it) }
                pendingArchive = null
                message(R.string.data_no_picker)
            }
        }) {
            val stage = BackupArchive.createStage(noBackupFilesDir)
            try {
                ensureCore()
                val data = File(stage, "data")
                val salt = BackupArchive.newSalt()
                val key = BackupArchive.key(phrase, salt)
                checkAnswer(strideBackup(data.absolutePath, key, false))
                val archive = File(stage, "backup.zip")
                archive.outputStream().use { BackupArchive.write(data, salt, it) }
                if (isDestroyed) {
                    clean(stage)
                } else {
                    pendingArchive = archive
                }
                getString(R.string.data_choose_destination)
            } catch (error: Exception) {
                clean(stage)
                throw error
            } finally {
                phrase.fill('\u0000')
            }
        }
    }

    private fun restore(uri: Uri, password: CharArray) {
        task {
            val stage = BackupArchive.createStage(noBackupFilesDir)
            try {
                ensureCore()
                val input = contentResolver.openInputStream(uri) ?: error("No input")
                val salt = input.use { BackupArchive.read(it, stage) }
                val key = BackupArchive.key(password, salt)
                checkAnswer(strideBackup(stage.absolutePath, key, true))
                StrideCore.historyChanged()
                getString(R.string.data_restored)
            } finally {
                password.fill('\u0000')
                clean(stage)
            }
        }
    }

    private fun ensureCore() {
        check(StrideCore.ensureOpen(applicationContext) == null) { "Core unavailable" }
    }

    private fun exportHistory(format: String) {
        val start = from.text.toString()
        val end = to.text.toString()
        try {
            require(LocalDate.parse(start) <= LocalDate.parse(end))
        } catch (_: Exception) {
            message(R.string.data_invalid_dates)
            return
        }
        task(onSuccess = {
            val name = pendingText?.first ?: return@task
            try {
                awaitingPicker = true
                if (format == "csv") saveCsv.launch(name) else saveJson.launch(name)
            } catch (_: android.content.ActivityNotFoundException) {
                awaitingPicker = false
                pendingText = null
                message(R.string.data_no_picker)
            }
        }) {
            ensureCore()
            val answer = strideHistoryExport(start, end, format)
            checkAnswer(answer)
            val body = JSONObject(answer).getJSONObject("body")
            pendingText = body.getString("filename") to body.getString("content")
            getString(R.string.data_choose_export_destination)
        }
    }

    private fun saveText(uri: Uri?) {
        awaitingPicker = false
        val text = pendingText?.second
        if (text == null) {
            message(R.string.data_interrupted)
            return
        }
        if (uri == null) {
            pendingText = null
            message(R.string.data_cancelled)
            return
        }
        whenUnlocked {
            pendingText = null
            task {
                val output = contentResolver.openOutputStream(uri, "wt") ?: error("No output")
                output.bufferedWriter(Charsets.UTF_8).use { it.write(text) }
                getString(R.string.data_export_saved)
            }
        }
    }

    private fun checkAnswer(answer: String) {
        val result = JSONObject(answer)
        if (!result.getBoolean("ok")) {
            throw BackupFailure(result.optJSONObject("error")?.optString("detail") ?: getString(R.string.data_failed))
        }
    }

    private fun task(onSuccess: (() -> Unit)? = null, work: () -> String) {
        setBusy(true)
        status.setText(R.string.data_working)
        worker.execute {
            try {
                val result = work()
                runOnUiThread {
                    if (!isDestroyed) {
                        status.text = result
                        if (onSuccess != null) onSuccess() else setBusy(false)
                    }
                }
            } catch (error: Exception) {
                Log.w("StrideData", "The file operation was not completed")
                runOnUiThread {
                    if (!isDestroyed) {
                        status.text = if (error is BackupFailure) error.message else getString(R.string.data_failed)
                        setBusy(false)
                    }
                }
            }
        }
    }

    private fun whenUnlocked(action: () -> Unit) {
        if (gate.isUnlocked) action() else {
            pendingAction = action
            gate.lock()
        }
    }

    private fun password() = CharArray(passphrase.length()) { passphrase.text[it] }

    private fun clearFields() {
        if (::passphrase.isInitialized) passphrase.text.clear()
        if (::confirmation.isInitialized) confirmation.text.clear()
    }

    private fun passwordField(label: Int, parent: LinearLayout) = EditText(this).apply {
        setHint(label)
        contentDescription = getString(label)
        inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        isSaveEnabled = false
        importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS
        minHeight = (48 * resources.displayMetrics.density).toInt()
        parent.addView(this)
        controls.add(this)
    }

    private fun dateField(label: Int, initial: String, parent: LinearLayout) = EditText(this).apply {
        setHint(label)
        contentDescription = getString(label)
        setText(initial)
        inputType = InputType.TYPE_CLASS_DATETIME or InputType.TYPE_DATETIME_VARIATION_DATE
        minHeight = (48 * resources.displayMetrics.density).toInt()
        parent.addView(this)
        controls.add(this)
    }

    private fun button(parent: LinearLayout, label: Int, action: () -> Unit) {
        val button = Button(this).apply {
            setText(label)
            setOnClickListener { if (gate.isUnlocked && !busy) action() }
        }
        parent.addView(button)
        controls.add(button)
    }

    private fun message(id: Int) {
        status.setText(id)
        setBusy(false)
    }

    private fun setBusy(value: Boolean) {
        busy = value
        updateControls()
    }

    private fun updateControls() {
        controls.forEach { it.isEnabled = !busy && ::gate.isInitialized && gate.isUnlocked }
    }

    private fun clean(stage: File) {
        try {
            if (!BackupArchive.removeStage(stage)) Log.w("StrideData", "Encrypted temporary files could not be removed")
        } catch (_: IOException) {
            Log.w("StrideData", "Encrypted temporary files could not be resolved")
        }
    }

    private class BackupFailure(message: String) : Exception(message)
}
