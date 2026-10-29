package app.stride.running.setup

import android.app.Application
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.text.method.PasswordTransformationMethod
import android.view.View
import android.view.WindowManager
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import app.stride.running.R
import app.stride.running.security.DeviceCredential
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.sync.SyncEngine

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
 *
 * **One screen, and it does the work.** The key is entered once. Saving it
 * *is* the request to sync: the screen checks the key, keeps it, pulls, and
 * says what happened — "Checking…", "Syncing…" with the number of runs as
 * they arrive, and then "Connected · 160 runs", and then it goes away.
 * There is no second screen and no second button.
 *
 * There is no server field. The phone has no server: it *is* the app.
 */
class GarminSetupActivity : AppCompatActivity(), SetupRunner.Listener {

    private lateinit var keyField: EditText
    private lateinit var status: TextView
    private lateinit var progress: TextView
    private lateinit var connect: Button
    private lateinit var disconnect: Button
    private lateinit var notNow: Button

    private lateinit var runner: SetupRunner

    /** Whether a Garmin key is on the phone. */
    private var connected = false

    /**
     * The device's credential, asked for again before the credential changes.
     *
     * Replacing the Garmin key or disconnecting it is not something that
     * should happen because somebody picked up an unlocked phone. So the
     * device is asked to vouch for the athlete once more — the same
     * credential the app gate asks for.
     */
    private val credential: ActivityResultLauncher<Intent> =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            DeviceCredential.credentialAnswered(result.resultCode)
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // The screen holds a credential: no screenshots, no recents
        // thumbnail, no screen recording of the key being pasted.
        window.setFlags(
            WindowManager.LayoutParams.FLAG_SECURE,
            WindowManager.LayoutParams.FLAG_SECURE,
        )
        setContentView(R.layout.activity_garmin_setup)

        keyField = findViewById(R.id.garmin_key)
        status = findViewById(R.id.garmin_status)
        progress = findViewById(R.id.garmin_progress)
        connect = findViewById(R.id.garmin_connect)
        disconnect = findViewById(R.id.garmin_disconnect)
        notNow = findViewById(R.id.garmin_not_now)

        runner = SetupRunner(application as Application, this)

        keyField.transformationMethod = PasswordTransformationMethod.getInstance()
        keyField.setHorizontallyScrolling(false)

        findViewById<Button>(R.id.garmin_show).setOnClickListener {
            val shown = keyField.transformationMethod == null
            keyField.transformationMethod =
                if (shown) PasswordTransformationMethod.getInstance() else null
            keyField.setSelection(keyField.text?.length ?: 0)
        }
        findViewById<Button>(R.id.garmin_open_garmin).setOnClickListener {
            openGarmin()
        }
        connect.setOnClickListener { withCredential { connect() } }
        disconnect.setOnClickListener { withCredential { disconnect() } }
        notNow.setOnClickListener { dismissSetup() }

        // Leaving the screen without connecting is a dismissal: the screen
        // does not come back on the next launch.
        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    dismissSetup()
                }
            },
        )

        refresh()
    }

    override fun onDestroy() {
        runner.close()
        super.onDestroy()
    }

    /**
     * Ask the device for its credential, and do the thing only if it says yes.
     *
     * A phone with no screen lock has nothing to ask for; the credential is
     * then simply absent and the action goes ahead — the gate has already
     * decided what that phone may keep.
     */
    private fun withCredential(action: () -> Unit) {
        DeviceCredential.prompt(
            this,
            credential,
            onGranted = { action() },
            onRefused = {
                status.text = getString(R.string.garmin_credential_refused)
            },
        )
    }

    /** What the screen shows when it opens. */
    private fun refresh() {
        connected = hasKey()
        if (connected) {
            // The phone has a key: the screen says so, and counts the runs
            // in the background rather than blocking the first frame.
            show(SetupUi(SetupPhase.Connected, runs = -1))
            runner.count count@{ runs ->
                if (isFinishing || isDestroyed) return@count
                if (connected) show(SetupUi(SetupPhase.Connected, runs = runs))
            }
        } else {
            show(SetupUi(SetupPhase.Idle))
        }
    }

    /** Whether the phone holds a Garmin key. */
    private fun hasKey(): Boolean = try {
        !Secrets.garminKey(this).isNullOrBlank()
    } catch (_: Exception) {
        // A locked phone cannot read the key. It has not got one as far as
        // the screen is concerned, and the athlete is asked for it again.
        false
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

    /**
     * Validate the key against Garmin and, if it works, keep it and pull.
     *
     * The runner does the work and reports the states; this is the button.
     */
    private fun connect() {
        val key = keyField.text?.toString()?.trim().orEmpty()
        if (key.length < 20) {
            status.text = getString(R.string.garmin_key_too_short)
            return
        }
        runner.connect(key)
    }

    /** Forget the key. The imported runs stay on the phone. */
    private fun disconnect() {
        runner.forget()
        connected = false
        show(SetupUi(SetupPhase.Idle))
    }

    /**
     * Leave the screen, and remember that it was left.
     *
     * The screen is not the app: it is the one thing the athlete has to do
     * once. Being dismissed is a decision, and it is kept.
     */
    private fun dismissSetup() {
        AppSettings(this).setupDismissed = true
        finish()
    }

    /** The state the runner reports. */
    override fun onState(ui: SetupUi) {
        if (ui.phase == SetupPhase.Syncing) {
            // The key is in the Keystore now. The buffer is dropped: it is
            // not kept in the field, and it is not in the saved state.
            keyField.setText("")
        }
        if (ui.phase == SetupPhase.Connected) {
            connected = true
            // A key is on the phone, so the screen has nothing left to ask
            // for.
            AppSettings(this).setupDismissed = true
        }
        show(ui)
        if (ui.phase == SetupPhase.Connected) {
            // The screen goes away once it has said what it came to say.
            leaveSoon()
        }
    }

    /** Show a state. */
    private fun show(ui: SetupUi) {
        status.text = render(SetupCopy.status(ui))
        val line = SetupCopy.progress(ui)
        when {
            // The count, while the pull runs.
            line != null -> {
                progress.visibility = View.VISIBLE
                progress.text = render(line)
            }
            // The reason, when there is one.
            ui.detail != null -> {
                progress.visibility = View.VISIBLE
                progress.text = ui.detail
            }
            else -> progress.visibility = View.GONE
        }

        val working = ui.phase == SetupPhase.Checking || ui.phase == SetupPhase.Syncing
        connect.isEnabled = !working
        keyField.isEnabled = !working
        disconnect.visibility = if (connected) View.VISIBLE else View.GONE
        notNow.visibility = if (connected || working) View.GONE else View.VISIBLE
        connect.setText(if (connected) R.string.garmin_replace_key else R.string.garmin_connect)
    }

    /** A line of the screen: the words, and the number they count. */
    private fun render(line: Line): String {
        val count = line.count ?: return getString(line.resource)
        return if (line.plural) {
            resources.getQuantityString(line.resource, count, count)
        } else {
            getString(line.resource, count)
        }
    }

    /** Leave the screen in a moment, so the last words are read. */
    private fun leaveSoon() {
        status.postDelayed({ if (!isFinishing) finish() }, 1_500L)
    }
}
