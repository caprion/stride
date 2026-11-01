package app.stride.running.setup

import android.Manifest
import android.app.Application
import android.content.ActivityNotFoundException
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.text.method.PasswordTransformationMethod
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.browser.customtabs.CustomTabsIntent
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import app.stride.running.R
import app.stride.running.security.DeviceCredential
import app.stride.running.security.KeyIdentifier
import app.stride.running.security.Secrets
import app.stride.running.settings.AppSettings
import app.stride.running.sync.SyncEngine

/**
 * The native Garmin setup screen.
 *
 * **The credential is entered here, in the app, and never in the WebView.**
 * A WebView is a browser: anything in it is exposed to the page, to an
 * injection, and to a future dependency. So the key is pasted into a native
 * field, validated against Garmin, wrapped by a non-exportable Keystore
 * key and written to `noBackupFilesDir` — and the page never sees it, and
 * there is no endpoint that returns it.
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
 *
 * **The screen states the promise.** "Your data stays on this phone." is on
 * the screen, at the top, before the field — because the field is the one
 * thing the app asks for, and the promise is the reason it is safe to ask.
 *
 * **One link, under the field.** It goes to Garmin's key page in a Custom
 * Tab, so the athlete comes back to the field with what they pasted still in
 * it. The label carries the one detail that matters — pick the one-year
 * expiry — because a 30-day key means the app stops working every month.
 */
class GarminSetupActivity : AppCompatActivity(), SetupRunner.Listener {

    private lateinit var keyField: EditText
    private lateinit var status: TextView
    private lateinit var progress: TextView
    private lateinit var keyIdentifier: TextView
    private lateinit var connect: Button
    private lateinit var forget: Button
    private lateinit var notNow: Button

    private lateinit var runner: SetupRunner

    /**
     * Ask for notification permission once the connection exists.
     *
     * A refused permission never changes the import or the pull; it only
     * means the phone stays quiet.
     */
    private val notificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { }

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

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // The screen holds a credential — no screenshots, no recents
        // thumbnail, no screen recording of the key being pasted.
        window.setFlags(
            WindowManager.LayoutParams.FLAG_SECURE,
            WindowManager.LayoutParams.FLAG_SECURE,
        )
        setContentView(R.layout.activity_garmin_setup)

        keyField = findViewById(R.id.garmin_key)
        status = findViewById(R.id.garmin_status)
        progress = findViewById(R.id.garmin_progress)
        keyIdentifier = findViewById(R.id.garmin_key_identifier)
        connect = findViewById(R.id.garmin_connect)
        forget = findViewById(R.id.garmin_forget)
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
        findViewById<TextView>(R.id.garmin_open_garmin).setOnClickListener {
            openGarmin()
        }
        connect.setOnClickListener { withCredential { testKey() } }
        forget.setOnClickListener { withCredential { forget() } }
        notNow.setOnClickListener { dismissSetup() }

        // The screen is not a page of the app: it is the one thing the
        // athlete has to do once. The system back button closes it, and
        // closing it is a decision that is kept.
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
        // The field is not part of the activity's saved state and does not
        // outlive the screen. The key itself is already in the Keystore, or
        // it was never accepted.
        keyField.setText("")
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
            onGranted = { action() },
            onRefused = {
                status.text = getString(R.string.garmin_credential_refused)
            },
        )
    }

    /** What the screen shows when it opens. */
    private fun refresh() {
        val key = savedKey()
        connected = !key.isNullOrBlank()
        showIdentifier(key)
        if (connected) {
            // The phone has a key: the screen says so, and counts the runs
            // in the background rather than blocking the first frame.
            show(SetupUi(SetupPhase.Connected, runs = -1))
            runner.count { runs ->
                if (isFinishing || isDestroyed) return@count
                if (connected) show(SetupUi(SetupPhase.Connected, runs = runs))
            }
        } else {
            show(SetupUi(SetupPhase.Idle))
        }
    }

    /** The saved key, read only long enough to learn whether there is one. */
    private fun savedKey(): String? = try {
        Secrets.garminKey(this)
    } catch (_: Exception) {
        // A locked phone cannot read the key. It has not got one as far as
        // the screen is concerned, and the athlete is asked for it again.
        null
    }

    /**
     * Show the key's identifier, never the key.
     *
     * This is a native surface. The WebView cannot reach it, and the only
     * part of the key on the screen is its last four characters.
     */
    private fun showIdentifier(key: String?) {
        val identifier = KeyIdentifier.lastFour(key)
        if (identifier == null) {
            keyIdentifier.visibility = View.GONE
            keyIdentifier.text = ""
        } else {
            keyIdentifier.visibility = View.VISIBLE
            keyIdentifier.text = getString(R.string.garmin_key_identifier, identifier)
        }
    }

    /**
     * Open Garmin's key page.
     *
     * **In a Custom Tab.** A Custom Tab is a page *over* the app: the
     * athlete closes it and comes straight back to the field they were
     * filling in, with what they had pasted still in it. A plain browser
     * takes them out to a browser they then have to find their way back
     * from, and the field is empty when they get there.
     *
     * A phone with no browser that can host a Custom Tab gets the plain
     * view instead: it is worse, and it is better than nothing.
     */
    private fun openGarmin() {
        val url = GarminKeyPage.URL.toUri()
        try {
            CustomTabsIntent.Builder()
                .setShowTitle(true)
                .build()
                .launchUrl(this, url)
        } catch (_: ActivityNotFoundException) {
            openInBrowser(url)
        }
    }

    /** Open the key page in whatever browser the phone has. */
    private fun openInBrowser(url: Uri) {
        try {
            startActivity(Intent(Intent.ACTION_VIEW, url))
        } catch (_: Exception) {
            Toast.makeText(this, R.string.garmin_no_browser, Toast.LENGTH_LONG).show()
        }
    }

    /**
     * Validate the key against Garmin and, if it works, keep it and pull.
     *
     * The runner does the work and reports the states; this is the button.
     */
    private fun testKey() {
        val key = keyField.text?.toString()?.trim().orEmpty()
        if (key.length < 20) {
            status.text = getString(R.string.garmin_key_too_short)
            return
        }
        runner.connect(key)
    }

    /** Forget the key. The imported runs stay on the phone. */
    private fun forget() {
        try {
            runner.forget()
            connected = false
            AppSettings(this).connectionChanged = true
            showIdentifier(null)
            show(SetupUi(SetupPhase.Idle))
            Toast.makeText(this, R.string.forget_key_result, Toast.LENGTH_LONG).show()
        } catch (_: Throwable) {
            Toast.makeText(this, R.string.forget_key_incomplete, Toast.LENGTH_LONG).show()
        }
    }

    /**
     * Leave the screen, and remember that it was left.
     *
     * The screen is not the app: it is the one thing the athlete has to do
     * once. Being dismissed is a decision, and it is kept.
     */
    private fun dismissSetup() {
        AppSettings(this).setupDismissed = true
        Toast.makeText(this, R.string.garmin_not_now_result, Toast.LENGTH_LONG).show()
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
            AppSettings(this).connectionChanged = true
            showIdentifier(savedKey())
            askForNotificationPermission()
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
        val progressLine = SetupCopy.progress(ui)
        val detailLine = SetupCopy.detail(ui)
        when {
            // The count, while the pull runs.
            progressLine != null -> {
                progress.visibility = View.VISIBLE
                progress.text = render(progressLine)
            }
            // The reason, when there is one.
            detailLine != null -> {
                progress.visibility = View.VISIBLE
                progress.text = render(detailLine)
            }
            else -> progress.visibility = View.GONE
        }

        val working = ui.phase == SetupPhase.Checking || ui.phase == SetupPhase.Syncing
        connect.isEnabled = !working
        keyField.isEnabled = !working
        forget.visibility = if (connected) View.VISIBLE else View.GONE
        notNow.visibility = if (connected || working) View.GONE else View.VISIBLE
        connect.setText(R.string.garmin_test_key)
    }

    /** Ask for notification permission, if the phone has not answered it. */
    private fun askForNotificationPermission() {
        val granted = ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS,
        ) == PackageManager.PERMISSION_GRANTED
        if (!granted) notificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
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
