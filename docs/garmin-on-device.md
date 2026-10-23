# Garmin Personal API connection on Android

**Design date: 2026-10-23**

This design assumes Android talks to Garmin's Personal API directly. The key is
only sent to Garmin's token endpoint; Stride stores it locally. It must never
be sent to a Stride server, analytics, crash reporting, logs, clipboard history,
or WorkManager input data. Garmin's Personal API is documented for personal
use, with pull delivery, and includes activity JSON/FIT and health JSON [Garmin
program overview](https://developer.garmin.com/gc-developer-program/overview/).

## First-run and key entry

Show the value before asking for a credential:

> **Your runs, with the detail behind them**
>
> Connect Garmin to bring your recent runs into Stride, including routes,
> splits and heart-rate detail. You can also see sleep, resting heart rate,
> stress and Body Battery alongside your training. Stride checks Garmin for
> updates in the background and keeps a copy on this phone.
>
> **Connect Garmin** · **Maybe later**

On tapping Connect, show a short setup screen. Open the Garmin key page in a
Custom Tab/browser using `ACTION_VIEW` to
`https://connect.garmin.com/app/settings/personalApiToken`; label it **Open
Garmin to create a key**. Garmin requires the user to be signed in on Garmin
Connect Web; its current FAQ says the key is displayed once, and can be set to
expire after 30, 90, 180 days or one year [Garmin FAQ](https://developer.garmin.com/gc-developer-program/personal/personal-faq/).
The user returns with Android's normal app-resume lifecycle (`onResume`); do not
infer success from that. Keep the entry screen and its **I have my key** action
available whether they return from a browser, tab, or another app. No custom
deep-link callback is needed or expected from Garmin.

### Entry UI and validation

Use one multiline-capable, single-line text field labelled **Personal API key**,
with paste support, a reveal/hide eye, and a clear button. Start masked;
disable autocorrect, suggestions, and personalized learning; use a monospace
font while revealed. Do not auto-trim internal characters or transform case;
trim surrounding whitespace at submit. Do not persist drafts, include the
value in saved UI state, clipboard, logs, screenshots, or analytics. Add a
small **Where do I find this?** link back to Garmin.

Before storage, run a synchronous logical validation as a cancellable network
operation: POST the trimmed key to Garmin's token endpoint with
`Authorization: ApiKey <key>`, then GET the profile settings endpoint with the
returned bearer token. This proves both token exchange and a basic profile
read, as the repo's server `validate()` does. Do not save the key or token until
both calls succeed and the user/profile result is displayed for confirmation.
Never echo the key in errors. Reject blank or implausibly short input locally,
but do not claim a shape check proves validity. A profile failure after token
success is a distinct connect failure; keep the key only in memory and offer
retry.

Reassurance copy, shown immediately above the field:

> **Your key stays on this phone.** Stride sends it directly to Garmin to get a
> short-lived access token. Stride uses that token to read your Garmin profile,
> runs and wellness data, then saves the imported data on this device. We
> never send your key to Stride's servers. Garmin shows the key only once, so
> keep it private. You can remove it here or revoke it in Garmin Connect at any
> time.

### Connection errors (exact UI copy)

- Wrong key / Garmin 401 or 403 on token exchange: **“Garmin didn’t accept this
  key. Check that you copied the whole key, or create a new one in Garmin
  Connect. Your key hasn’t been saved.”** CTA: **Try again** / **Open Garmin**.
- Expired or revoked key (same observable 401/403): **“This Garmin key has
  expired or was revoked. Create a new key in Garmin Connect, then replace it
  here. Your imported runs are still on this phone.”** CTA: **Replace key**.
  Garmin allows expiry choices and says deleting a key invalidates its
  associated access tokens. The API response alone may not distinguish expiry
  from revocation or a typo; say “may have” if the client cannot establish
  which through Garmin settings.
- No network / timeout / DNS / transport error: **“Can’t reach Garmin right
  now. Check your connection and try again. Your key hasn’t been saved.”** CTA:
  **Try again**.
- Garmin 429: **“Garmin is temporarily limiting requests. We’ll wait before
  trying again. Your key hasn’t been saved.”** CTA: **Okay**; show a retry time
  only when Garmin provides `Retry-After`.
- Token exchange succeeds, profile request fails: **“Garmin accepted the key,
  but Stride couldn’t read your profile yet. We haven’t saved the key. Try
  again in a little while.”** CTA: **Try again**. If profile returns 401/403
  after one token refresh, use the expired/revoked message instead.
- Other server response: **“Garmin couldn’t complete the connection
  (HTTP {status}). Your key hasn’t been saved. Try again later.”** Avoid
  displaying Garmin response bodies, which can contain implementation detail.

Treat all token-endpoint 401/403 outcomes as invalid/expired/revoked, not
specifically “wrong password.” Follow with one profile call to establish
working read access. Do not retry a 429 immediately.

## Credential storage and lifetime

Use **an encrypted private app file with an Android Keystore AES-256-GCM key**.
This avoids `EncryptedSharedPreferences`, which AndroidX now marks deprecated,
and keeps the credential out of an unencrypted database. Store only the
long-lived Garmin key in this file; keep the short-lived bearer token in
memory. Exclude the credential file from Android backup/device transfer. If
the Keystore key is invalidated or the app data is restored without its key,
delete the unreadable blob and ask the athlete to reconnect. Android's
Keystore keeps key material non-exportable and performs cryptographic
operations without exposing the key to the app process [Android Keystore
system](https://developer.android.com/privacy-and-security/keystore).

Kotlin sketch (omits file I/O and error handling; write atomically, set private
permissions, and exclude the file in backup rules):

```kotlin
private const val ALIAS = "stride.garmin.personal-key.v1"

fun keystoreKey(): SecretKey {
    val ks = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    (ks.getKey(ALIAS, null) as? SecretKey)?.let { return it }
    val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
    generator.init(KeyGenParameterSpec.Builder(
        ALIAS,
        KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
    ).setBlockModes(KeyProperties.BLOCK_MODE_GCM)
     .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
     .setKeySize(256)
     .setRandomizedEncryptionRequired(true)
     .build())
    return generator.generateKey()
}

fun encryptKey(apiKey: String): ByteArray {
    val cipher = Cipher.getInstance("AES/GCM/NoPadding")
    cipher.init(Cipher.ENCRYPT_MODE, keystoreKey())
    val iv = cipher.iv
    val ciphertext = cipher.doFinal(apiKey.toByteArray(Charsets.UTF_8))
    return byteArrayOf(iv.size.toByte()) + iv + ciphertext
}
```

The backend client caches bearer token in memory and refreshes at 23 hours,
one hour before Garmin's documented 24-hour maximum. Android should do the
same with an in-memory token and an expiry timestamp: exchange the saved API
key when no usable token exists or within 60 seconds of expiry. On a data GET
401, force one token exchange and retry that GET once, mirroring
`GarminPersonalClient._get`; a second auth failure disables sync and asks for
key replacement. Garmin says there is no refresh token; obtain a new access
token using the key [Garmin FAQ](https://developer.garmin.com/gc-developer-program/personal/personal-faq/).
Do not persist JWTs to disk or put them in WorkManager Data.

### Revoke / replace

In Settings → Garmin connection provide **Disconnect Garmin** and
**Replace key**. Disconnect immediately cancels unique periodic and one-time
sync work, clears in-memory token and encrypted key, clears pending retry state,
and marks the connection disconnected. Imported activities and health rows
remain on the phone; offer a separate **Delete Garmin data from this phone**
action with its own confirmation if deletion is desired.

Also show **Revoke key in Garmin Connect** opening the Garmin key settings page.
State plainly: “This opens Garmin Connect. Delete the Stride key there to stop
Garmin access, then return here.” Because Garmin offers no redirect/callback,
on app resume refresh connection status by exchanging the key once (rate
limited and user-initiated only). If rejected, clear the cached bearer token,
mark **Action needed**, preserve key until user chooses Disconnect/Replace,
and show the expired/revoked message. Garmin says deleting the key invalidates
the key and all associated access tokens [Garmin FAQ](https://developer.garmin.com/gc-developer-program/personal/personal-faq/).

## Pull cadence

Garmin's API is pull-only, so the app must poll. Schedule a **daily** unique
periodic work request, plus a one-time immediate sync after confirmed connect
and a user-triggered **Pull now** job. Use `NetworkType.CONNECTED` (metered
connections allowed), `BatteryNotLow=true`; do not require charging. Requiring
charging or unmetered Wi-Fi could postpone work indefinitely for a runner who
doesn't charge overnight or has no Wi-Fi. Cancel work on disconnect.

| Pull | First run | Regular run | Data and request budget |
|---|---|---|---|
| Activities | Most recent 30 days, paginated if needed; download detail and FIT for new activities | Search last 7 days (overlap handles late uploads/edits); fetch detail/FIT only for new activities; update summaries if changed | One list request plus roughly two calls (detail + FIT) per new activity. Repo sync window defaults to 90 days and skips detail/FIT for already imported activities; use that caching principle on device. |
| Wellness | Last 30 days, day by day | Yesterday and today (revisit today because metrics fill in), with occasional bounded repair of the last 7 days | Daily summary is the backbone; fetch sleep, HRV and daily HR per day as needed. Body Battery is a range call and should be fetched once per window, not per day. Avoid fetching the full 30-day window daily. |

The repo's `sync_wellness()` loops date-by-date and makes daily summary, sleep,
HRV and heart-rate calls, while body-battery reports are cached once per run.
Its 30-day default is a practical backfill target, not a verified Garmin
retention guarantee. The stated “about a month wellness / four months
activities” limits were not confirmed in Garmin's public Personal API FAQ or
overview during this research. Do not promise those as fixed limits. Start
with the newest 30 wellness days and newest 30 activity days; let the athlete
see the actual imported range. Older history can be requested as a separate,
rate-limited backfill if the endpoint supports it.

After key validation, tell the user **“Connected. Stride is importing up to
30 days of runs and wellness now. This can take a few minutes; you can leave
the app.”** Enqueue a one-time immediate worker, but describe it as starting
now, not completing instantly. The first request is intentionally bounded to
avoid a burst of detail/FIT calls and allow progress updates. Once it finishes,
schedule daily work.

Android `PeriodicWorkRequest` has a hard **15-minute minimum interval**, but
it does not promise exact execution time. WorkManager waits for constraints and
system scheduling/battery optimization (including Doze); actual work can be
late. The documentation explicitly says periodic work is inexact. A daily
interval is suitable; do not promise “at 6am.” A manual one-time request is
eligible promptly when connected, but still subject to OS scheduling and
constraints. Sources: [Define work requests](https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started/define-work),
[PeriodicWorkRequest reference](https://developer.android.com/reference/androidx/work/PeriodicWorkRequest),
checked 2026-10-23.

Example scheduling sketch:

```kotlin
val constraints = Constraints.Builder()
    .setRequiredNetworkType(NetworkType.CONNECTED)
    .setRequiresBatteryNotLow(true)
    .build()

val daily = PeriodicWorkRequestBuilder<GarminSyncWorker>(1, TimeUnit.DAYS)
    .setConstraints(constraints)
    .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.MINUTES)
    .addTag("garmin-sync")
    .build()
WorkManager.getInstance(context).enqueueUniquePeriodicWork(
    "garmin-daily-sync", ExistingPeriodicWorkPolicy.UPDATE, daily
)
```

### 429s and honest freshness

Honor `Retry-After` if present; persist the next eligible retry time, stop the
current batch, and do not spin through all remaining activities. If no header
exists, exponential backoff with jitter: 15 minutes, 30 minutes, 1 hour, 2
hours, then cap at 6 hours; reset after success. For WorkManager's retry
mechanism, return `Result.retry()` with exponential backoff, but don't rely on
WorkManager alone to implement Garmin's server-provided wait. The Android docs
allow exponential/linear backoff and say delays are inexact [Define work
requests](https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started/define-work).
The repo README reports Garmin returning 429s more often from datacentre IPs
than residential egress, but that is a repo observation, not an official
published Garmin quota. A handset generally avoids Stride's datacentre egress;
per-key/IP limits still mean batching and backoff matter. Avoid automatic rapid
retries and never run duplicate overlapping workers.

Maintain separate timestamps: **Last successful sync** updates only when the
required activity and wellness jobs both finish successfully; also show
**Runs updated** and **Wellness updated** separately when one succeeds and the
other fails. On 429/network error say **“Last successful pull: … · Next try
after …”**; never bump the success timestamp for an attempted or partial pull.
Persist partial progress and per-category errors so a retry resumes without
re-downloading FIT files already imported.

## Sync status surface

Settings → Garmin connection card:

- Connected to Garmin (profile name if available), with a masked-key indicator;
- Last successful pull timestamp, and separate last runs / wellness timestamps;
- Current state: **Syncing runs (3 of 5)**, **Waiting for network**, **Waiting
  for Garmin**, **Action needed**, or **Up to date**;
- On failure, a plain-language reason and retry time; for expired/revoked key,
  **Replace key** and **Open Garmin** actions;
- **Pull now**, disabled during an active sync, and **Disconnect Garmin**.

The first-run explanation must precede the key screen. After confirmation,
show progress and the bounded import statement before users leave the page.

## Research notes and limits

Sources checked 2026-10-23:

- Garmin Connect Personal API FAQ, current page: key one-time display, expiry
  options, revoke behavior, 24-hour token lifetime, no refresh tokens:
  [Garmin FAQ](https://developer.garmin.com/gc-developer-program/personal/personal-faq/).
- Garmin Connect Developer Program overview, current page: Personal delivery
  is pull-only; activity JSON/FIT and health JSON:
  [Garmin overview](https://developer.garmin.com/gc-developer-program/overview/).
- Android Developers, **Define work requests**, checked 2026-10-23: 15-minute
  minimum, constraints, inexact scheduling, backoff:
  [Android background work](https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started/define-work).
- AndroidX `PeriodicWorkRequest` API reference, checked 2026-10-23: OS battery
  optimization/Doze can delay work; 15-minute minimum:
  [PeriodicWorkRequest](https://developer.android.com/reference/androidx/work/PeriodicWorkRequest).
- Android Developers, **Android Keystore system**, checked 2026-10-23:
  [Keystore](https://developer.android.com/privacy-and-security/keystore).
- AndroidX `EncryptedSharedPreferences` API reference, checked 2026-10-23:
  deprecated; warns against Auto Backup restore:
  [EncryptedSharedPreferences](https://developer.android.com/reference/androidx/security/crypto/EncryptedSharedPreferences).
- Repo `README.md`, `backend/app/ingest/garmin_personal.py`,
  `backend/app/services/personal_sync.py`, and `backend/app/api/auth.py`, read
  2026-10-23: actual implementation details and repo-specific rate-limit
  observations; not independent official Garmin guarantees.

I could not verify an official fixed per-key/IP Garmin rate quota or the
“roughly one month wellness / four months activities” retention windows. The
Personal API FAQ does not state those limits. Treat both as observed behavior
to validate during rollout, and surface the successfully imported date range
instead of promising complete history.
