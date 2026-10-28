package app.stride.running.setup

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.text.method.PasswordTransformationMethod
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import app.stride.running.R
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets
import app.stride.running.sync.SyncScheduler
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * The native Garmin setup screen.
 *
 * **The credential is entered here, in the app, and never in the WebView.**
 * A WebView is a browser: anything in it is exposed to the page, to an
 * injection, and to a future dependency. So the key is pasted into a native
 * field, validated against Garmin, wrapped by a Keystore key and written to
 * `noBackupFilesDir` — and the page never sees it, and there is no endpoint
 * that returns it.
 *
 * The key goes to Garmin and nowhere else. It has to: Garmin's token
 * endpoint is the only thing that can issue the bearer token the pull uses.
 */
class GarminSetupActivity : AppCompatActivity() {

    private val worker = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "stride-setup").apply { isDaemon = true }
    }

    private lateinit var keyField: EditText
    private lateinit var status: TextView
    private lateinit var connect: Button
    private lateinit var disconnect: Button

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_garmin_setup)

        keyField = findViewById(R.id.garmin_key)
        status = findViewById(R.id.garmin_status)
        connect = findViewById(R.id.garmin_connect)
        disconnect = findViewById(R.id.garmin_disconnect)

        keyField.transformationMethod = PasswordTransformationMethod.getInstance()
        keyField.setHorizontallyScrolling(false)

        findViewById<Button>(R.id.garmin_open_garmin).setOnClickListener {
            openGarmin()
        }
        findViewById<Button>(R.id.garmin_show).setOnClickListener {
            val shown = keyField.transformationMethod == null
            keyField.transformationMethod =
                if (shown) PasswordTransformationMethod.getInstance() else null
            keyField.setSelection(keyField.text?.length ?: 0)
        }
        connect.setOnClickListener { connect() }
        disconnect.setOnClickListener { disconnect() }

        refresh()
    }

    /** What the screen shows when it opens. */
    private fun refresh() {
        val connected = !Secrets.garminKey(this).isNullOrBlank()
        status.text = if (connected) {
            getString(R.string.garmin_connected)
        } else {
            getString(R.string.garmin_not_connected)
        }
        disconnect.visibility = if (connected) View.VISIBLE else View.GONE
        connect.text = getString(
            if (connected) R.string.garmin_replace_key else R.string.garmin_connect
        )
    }

    /** Open Garmin's key page in the browser. */
    private fun openGarmin() {
        val intent = Intent(
            Intent.ACTION_VIEW,
            Uri.parse("https://connect.garmin.com/app/settings/personalApiToken"),
        )
        try {
            startActivity(intent)
        } catch (_: Exception) {
            Toast.makeText(this, R.string.garmin_no_browser, Toast.LENGTH_LONG).show()
        }
    }

    /** Validate the key against Garmin and, if it works, keep it. */
    private fun connect() {
        val key = keyField.text?.toString()?.trim().orEmpty()
        if (key.length < 20) {
            status.text = getString(R.string.garmin_key_too_short)
            return
        }

        status.text = getString(R.string.garmin_checking)
        connect.isEnabled = false

        worker.execute {
            val problem = StrideCore.ensureOpen(applicationContext)
            if (problem != null) {
                runOnUiThread {
                    status.text = problem
                    connect.isEnabled = true
                }
                return@execute
            }

            // The key is checked before it is kept: the token exchange and a
            // profile read both have to work before the key is worth having.
            val body = JSONObject().put("api_key", key).toString()
            val answer = StrideCore.call("POST", "/api/v1/auth/garmin/personal-key/test", body)
            val envelope = JSONObject(answer)

            runOnUiThread {
                connect.isEnabled = true
                if (!envelope.optBoolean("ok")) {
                    status.text = envelope.optJSONObject("error")?.optString("detail")
                        ?: getString(R.string.garmin_rejected)
                    return@runOnUiThread
                }
                Secrets.saveGarminKey(this, key)
                // The buffer is dropped: the key is not kept in the field's
                // state, and it is not in the saved instance state.
                keyField.setText("")
                refresh()
                SyncScheduler.schedule(this)
                SyncScheduler.pullNow(this)
                Toast.makeText(this, R.string.garmin_connected_toast, Toast.LENGTH_LONG).show()
            }
        }
    }

    /** Forget the key. The imported runs stay on the phone. */
    private fun disconnect() {
        Secrets.forgetGarminKey(this)
        SyncScheduler.cancel(this)
        refresh()
        Toast.makeText(this, R.string.garmin_disconnected_toast, Toast.LENGTH_LONG).show()
    }
}
